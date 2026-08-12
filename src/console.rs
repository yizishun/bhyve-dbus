pub struct Console;

pub struct VMInfo {
	pub name: String,
}

pub struct BhyvegcImage {

}

pub struct KeyEvent {

}

pub struct PtrEvent {

}

impl VMInfo {
	pub async fn vm_info() -> Self {
		/* TODO: socket */
		Self {
			name: "default".to_string()
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