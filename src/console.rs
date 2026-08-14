pub struct Console;

pub struct VMInfo {
	pub name: String,
	pub device_address: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BhyvegcImage {
	pub vgamode: u32,
	pub height: u32,
	pub width: u32,
	pub dmabuf: i32
}

pub struct BhyvegcImageUpdate {
	pub update: bool,
	/* the update position, in pixels. */
	pub x: i32,
	pub y: i32,
	/* the update width, height, in pixels. */
	pub width: i32,
	pub height: i32,
	pub image: BhyvegcImage
}

pub struct KeyEvent {

}

pub struct PtrEvent {

}

impl BhyvegcImage {
    	pub const fn new() -> Self {
		Self {
			vgamode: 0,
			height: 0,
			width: 0,
			dmabuf: 0
		}
	}
}

impl VMInfo {
	pub async fn vm_info() -> Self {
		/* TODO: socket */
		Self {
			name: "default".to_string(),
			device_address: "pci/0000/02.0".to_string()
		}
	}
}

impl Console {
	pub async fn console_ids() -> Vec<u32> {
		/* bhyve currently only support 1 console */
		vec!(1)
	}

	pub async fn console_poll_image(id: u32) -> std::io::Result<BhyvegcImageUpdate> {
		todo!()
	}

	pub async fn console_get_image(id: u32) -> std::io::Result<BhyvegcImage> {
		todo!()
	}

	pub async fn console_key_event(id: u32, event: KeyEvent) -> std::io::Result<()> {
		todo!();
	}

	pub async fn console_ptr_event(id:u32, event: PtrEvent) -> std::io::Result<()> {
		todo!();
	}
}