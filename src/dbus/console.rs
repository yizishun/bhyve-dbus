use tokio::sync::broadcast::{self, Sender};
use zbus::{interface, zvariant::OwnedFd};

use crate::{console::{BhyvegcImageUpdate, Console}, dbus::listener::{ListenerHandler, poll_loop}};

pub struct ConsoleInterface {
	pub console: Console,
	pub poller_tx: Sender<BhyvegcImageUpdate>
}

impl ConsoleInterface {
    pub fn new(console: Console) -> Self {
	let (tx, _) = broadcast::channel(16);
	let console_iface = Self {
		console: console.clone(),
		poller_tx: tx.clone()
	};
	tokio::spawn(async move {
		poll_loop(console, tx).await;
	});
	console_iface
    }
}

#[interface(name = "org.qemu.Display1.Console")]
impl ConsoleInterface {
	async fn register_listener(&self, fd: OwnedFd) {
		let mut listener = ListenerHandler::new(self.poller_tx.subscribe());
		println!("register_listener");
		tokio::spawn(async move {
			listener.connect_and_run(fd).await;
		});
	}

	#[zbus(name = "SetUIInfo")]
	async fn set_uiinfo(
		&self,
		width_mm: u16,
		height_mm: u16,
		xoff: i32,
		yoff: i32,
		width: u32,
		height: u32,
	) {
		/* 
		 * TODO: bhyve don't have this ability for now, we need something
		 * like console_set_ui in bhyve 
		 */
	}

	#[zbus(property)]
	async fn label(&self) -> String {
		format!("Console_{}", self.console.id).to_string()
	}

	#[zbus(property)]
	async fn head(&self) -> u32 {
		self.console.id
	}

	#[zbus(property)]
	async fn type_(&self) -> String {
		let image = self.console.console_get_image().await.unwrap();
		match image.vgamode {
			0 => "Graphic".to_string(),
			1 => "Text".to_string(),
			_ => panic!("dbus: type: vagmode invalid")
		}
	}

	#[zbus(property)]
	async fn device_address(&self) -> String {
		self.console.vm_info().await.expect("vm info error").device_address
	}

	#[zbus(property)]
	async fn height(&self) -> u32 {
		let image = self.console.console_get_image().await.unwrap();
		image.height
	}

	#[zbus(property)]
	async fn width(&self) -> u32 {
		let image = self.console.console_get_image().await.unwrap();
		image.width
	}

	#[zbus(property)]
	async fn interfaces(&self) -> &'static [&'static str] {
		&[
			"org.qemu.Display1.Keyboard",
			"org.qemu.Display1.Mouse"
		]
	}

}


