use std::os::fd::AsFd;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::time::Duration;

use tokio::net::UnixStream;
use tokio::time::MissedTickBehavior;
use tokio::time::interval;

use zbus::zvariant;
use zbus::{conn::Builder, proxy, zvariant::OwnedFd};

use crate::console::{BhyvegcImage, Console};

const REFRESH_RATE_HZ: f64 = 60.0;

#[proxy(interface = "org.qemu.Display1.Listener", assume_defaults = true)]
pub trait Listener {
    fn cursor_define(
        &self,
        width: i32,
        height: i32,
        hot_x: i32,
        hot_y: i32,
        data: &[u8],
    ) -> zbus::Result<()>;

    fn disable(&self) -> zbus::Result<()>;

    fn mouse_set(&self, x: i32, y: i32, on: i32) -> zbus::Result<()>;

    fn scanout(
        &self,
        width: u32,
        height: u32,
        stride: u32,
        pixman_format: u32,
        data: &[u8],
    ) -> zbus::Result<()>;

    #[zbus(name = "ScanoutDMABUF")]
    #[allow(clippy::too_many_arguments)]
    fn scanout_dmabuf(
        &self,
        dmabuf: zbus::zvariant::Fd<'_>,
        width: u32,
        height: u32,
        stride: u32,
        fourcc: u32,
        modifier: u64,
        y0_top: bool,
    ) -> zbus::Result<()>;

    #[allow(clippy::too_many_arguments)]
    fn update(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        stride: u32,
        pixman_format: u32,
        data: &[u8],
    ) -> zbus::Result<()>;

    #[zbus(name = "UpdateDMABUF")]
    fn update_dmabuf(&self, x: i32, y: i32, width: i32, height: i32) -> zbus::Result<()>;

    #[zbus(property)]
    fn interfaces(&self) -> zbus::Result<Vec<String>>;
}

pub struct ListenerHandler {
    console: Console,
}

impl ListenerHandler {
    pub fn new(console: Console) -> Self {
        Self { console }
    }

    /* TODO: only work in 1 console, we need make mult-console subscribe the listener instead of new listener */
	pub async fn connect_and_run(&self, fd: OwnedFd) {
        let std_ownedfd = std::os::fd::OwnedFd::from(fd);

        let std_stream = StdUnixStream::from(std_ownedfd);
        std_stream.set_nonblocking(true).unwrap();

        let stream = UnixStream::from_std(std_stream).unwrap();

        let conn = Builder::unix_stream(stream)
            .p2p()
            .build()
            .await
            .expect("dbus: listener connection fail");

        let proxy = ListenerProxy::new(&conn)
            .await
            .expect("dbus: listener proxy init fail");


        let refresh_interval = Duration::from_secs_f64(1.0 / REFRESH_RATE_HZ);

        let mut ticker = interval(refresh_interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut cache: Option<BhyvegcImage> = None;
        loop {
            ticker.tick().await;

            if let Err(_e) = ListenerHandler::update_display(
                &self,
                &proxy,
                &mut cache).await {
                    break;
            }
        }
	}

    async fn update_display(
        &self,
        proxy: &ListenerProxy<'_>,
        cache: &mut Option<BhyvegcImage>,
    ) -> zbus::Result<()> {
        let gc_update = match self.console.console_poll_image().await {
            Ok(update) => update,
            Err(e) => {
                let _ = proxy.disable().await;
                return Err(e.into());
            }
        };
        let need_scanout = match cache {
            Some(c) if c.generation != gc_update.generation => {
                *c = match self.console.console_get_image().await {
                    Ok(image) => image,
                    Err(e) => {
                        let _ = proxy.disable().await;
                        return Err(e.into());
                    }
                };
                true
            }
            Some(c) if gc_update.neen_scanout(c) => {
                c.vgamode = gc_update.vgamode;
                c.height = gc_update.height;
                c.width = gc_update.width;
                true
            }
            Some(_) => false,
            None => {
                *cache = match self.console.console_get_image().await {
                    Ok(update) => Some(update),
                    Err(e) => {
                        let _ = proxy.disable().await;
                        return Err(e.into());
                    }
                };
                true
            }
        };

        if need_scanout {
            let c = cache.as_ref().unwrap();
            proxy.scanout_dmabuf(
                zvariant::Fd::from(c.dmabuf.as_fd()),
                c.width,
                c.height,
                /* TODO: */
                c.width * 4,
                /* TODO: */
                0x34325258,
                0,
                true)
            .await
            .inspect_err(|e| eprintln!("dbus: listener: scanout fail with: {}", e))?;
        }

        if need_scanout {
            let c = cache.as_ref().unwrap();
            proxy.update_dmabuf(
                0,
                0,
                c.width as i32,
                c.height as i32)
            .await
            .inspect_err(|e| eprintln!("dbus: listener: update fail with: {}", e))?;
        } else if gc_update.need_update() {
            proxy.update_dmabuf(
                gc_update.dirty.x,
                gc_update.dirty.y,
                gc_update.dirty.width,
                gc_update.dirty.height)
            .await
            .inspect_err(|e| eprintln!("dbus: listener: update fail with: {}", e))?;
        }
        Ok(())
    }
}
