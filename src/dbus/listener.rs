use std::os::fd::AsFd;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::time::Duration;

use tokio::net::UnixStream;
use tokio::sync::broadcast;
use tokio::time::MissedTickBehavior;
use tokio::time::interval;

use zbus::zvariant;
use zbus::{conn::Builder, proxy, zvariant::OwnedFd};

use crate::console::{BhyvegcImage, Console, BhyvegcImageUpdate};

const REFRESH_RATE_HZ: f64 = 60.0;

#[proxy(
    interface = "org.qemu.Display1.Listener", 
    default_path = "/org/qemu/Display1/Listener",
    assume_defaults = true
)]
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
    poller_rx: broadcast::Receiver<BhyvegcImageUpdate>,
}

pub async fn poll_loop(console: Console, tx: broadcast::Sender<BhyvegcImageUpdate>) {

    let refresh_interval = Duration::from_secs_f64(1.0 / REFRESH_RATE_HZ);
    let mut ticker = interval(refresh_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    /* TODO: cache here */
    loop {
        ticker.tick().await;
        if tx.receiver_count() == 0 {
            continue;
        }

        match console.console_poll_image().await {
            Ok(update) => {
                let _ = tx.send(update);
            }
            Err(e) => {
                eprintln!("console poller: {e}");
                break;
            }
        }
    }
}

impl ListenerHandler {
    pub fn new(rx: broadcast::Receiver<BhyvegcImageUpdate>) -> Self {
        Self { poller_rx: rx }
    }

	pub async fn connect_and_run(&mut self, fd: OwnedFd) {
        let std_ownedfd = std::os::fd::OwnedFd::from(fd);

        let std_stream = StdUnixStream::from(std_ownedfd);
        std_stream.set_nonblocking(true).unwrap();

        let stream = UnixStream::from_std(std_stream).unwrap();

        let conn = Builder::unix_stream(stream)
            .server(zbus::Guid::generate()).unwrap()
            .p2p()
            .build()
            .await
            .expect("dbus: listener connection fail");

        let proxy = ListenerProxy::new(&conn)
            .await
            .expect("dbus: listener proxy init fail");


        let mut cache: Option<BhyvegcImage> = None;
        loop {
            match self.poller_rx.recv().await {
                Ok(update) => {
                    if let Err(e) = self.update_display(&proxy, &mut cache, update).await {
                        eprintln!("listener update fail: {e}");
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("listener: lagged {} frames, resync", n);
                    cache = None;
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
	}

    async fn update_display(
        &self,
        proxy: &ListenerProxy<'_>,
        cache: &mut Option<BhyvegcImage>,
        gc_update: BhyvegcImageUpdate
    ) -> zbus::Result<()> {
        let need_scanout = cache.as_ref().map_or(true, |c| gc_update.need_scanout(c));

        if need_scanout {
            let c = cache.insert(gc_update.image);
            proxy.scanout_dmabuf(
                zvariant::Fd::from(c.dmabuf.as_fd()),
                c.width,
                c.height,
                /* TODO: */
                c.width * 4,
                /* TODO: */
                0x34325258,
                0,
                false)
            .await
            .inspect_err(|e| eprintln!("dbus: listener: scanout fail with: {}", e))?;
        }

        if BhyvegcImageUpdate::need_update(gc_update.dirty) {
            println!("update!! {} {} {} {} {} {}", 
                cache.as_ref().unwrap().width,
                cache.as_ref().unwrap().height,
                gc_update.dirty.x,
                gc_update.dirty.y, 
                gc_update.dirty.width, 
                gc_update.dirty.height
            );
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
