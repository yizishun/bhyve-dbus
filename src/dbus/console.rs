use zbus::{interface, zvariant::OwnedFd};

use crate::{console::{Console, VMInfo}, dbus::listener::ListenerHandler};

pub struct ConsoleInterface {
	pub id: u32,
}

impl ConsoleInterface {
    pub fn new(id: u32) -> Self {
	Self {
		id,
	}
    }
}

#[interface(name = "org.qemu.Display1.Console")]
impl ConsoleInterface {
	async fn register_listener(&self, fd: OwnedFd) {
		let id = self.id;
		tokio::spawn(async move {
			ListenerHandler::connect(id, fd).await;
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
		format!("Console_{}", self.id).to_string()
	}

	#[zbus(property)]
	async fn head(&self) -> u32 {
		self.id
	}

	#[zbus(property)]
	async fn type_(&self) -> String {
		let image = Console::console_get_image().await;
		match image.vgamode {
			0 => "Graphic".to_string(),
			1 => "Text".to_string(),
			_ => panic!("dbus: type: vagmode invalid")
		}
	}

	#[zbus(property)]
	async fn device_address(&self) -> String {
		VMInfo::vm_info().await.device_address
	}

	#[zbus(property)]
	async fn height(&self) -> u32 {
		let image = Console::console_get_image().await;
		image.height
	}

	#[zbus(property)]
	async fn width(&self) -> u32 {
		let image = Console::console_get_image().await;
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


