use zbus::interface;

use crate::console::Console;

pub struct MouseInterface {
	console: Console
}

impl MouseInterface {
    pub fn new(console: Console) -> Self {
	Self { console }
    }
}

#[interface(name = "org.qemu.Display1.Mouse")]
impl MouseInterface {
	async fn press(&self, button: u32) {
	}

	async fn rel_motion(&self, dx: i32, dy: i32) {
	}

	async fn release(&self, button: u32) {
	}

	async fn set_abs_position(&self, x: u32, y: u32) {
	}

	#[zbus(property)]
	async fn is_absolute(&self) -> bool {
		true
	}
}


