use std::os::fd::BorrowedFd;
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

        let mut pre_image = BhyvegcImage::new();
        loop {
            ticker.tick().await;

            if let Err(_e) = ListenerHandler::update_display(
                &self,
                &proxy,
                &mut pre_image).await {
                    break;
            }
        }
	}

    async fn update_display(
        &self,
        proxy: &ListenerProxy<'_>,
        pre_image: &mut BhyvegcImage,
    ) -> zbus::Result<()> {
        /* console_poll_image may hang if bhyve find the image is clean. We have skip ticker, so it's fine. */
        let gc_update = match self.console.console_poll_image().await {
            Ok(update) => update,
            Err(e) => {
                let _ = proxy.disable().await;
                return Err(e.into());
            }
        };
        let gc_image = gc_update.image;

        if gc_image != *pre_image {
            let fd = unsafe {
                BorrowedFd::borrow_raw(gc_image.dmabuf)
            };
            proxy.scanout_dmabuf(
                zvariant::Fd::from(fd),
                gc_image.width,
                gc_image.height,
                gc_image.width * 4,
                0x34325258,
                0,
                true)
            .await
            .inspect_err(|e| eprintln!("dbus: listener: scanout fail with: {}", e))?;
        }
        *pre_image = gc_image;

        if gc_update.update {
            proxy.update_dmabuf(
                gc_update.x,
                gc_update.y,
                gc_update.width,
                gc_update.height)
            .await
            .inspect_err(|e| eprintln!("dbus: listener: scanout fail with: {}", e))?;
        }
        Ok(())
    }
}
