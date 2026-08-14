use zbus::{interface, zvariant::OwnedFd};

use crate::{console::Console, dbus::listener::ListenerHandler};

pub struct ConsoleInterface {
	pub console: Console
}

impl ConsoleInterface {
    pub fn new(console: Console) -> Self {
	Self {
		console,
	}
    }
}

#[interface(name = "org.qemu.Display1.Console")]
impl ConsoleInterface {
	async fn register_listener(&self, fd: OwnedFd) {
		let console = self.console.clone();
		let listener = ListenerHandler::new(console);
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
		self.console.device_address.clone()
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


