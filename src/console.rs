pub struct Console;

pub struct VMInfo {
	pub name: String,
	pub device_address: String,
}

pub struct BhyvegcImage {
	pub vgamode: u32,
	pub height: u32,
	pub width: u32
}

pub struct KeyEvent {

}

pub struct PtrEvent {

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

	pub async fn console_get_image() -> BhyvegcImage {
		todo!();
	}

	pub async fn console_refresh() {
		todo!();
	}

	pub async fn console_key_event(event: KeyEvent) {
		todo!();
	}

	pub async fn console_ptr_event(event: PtrEvent) {
		todo!();
	}
}