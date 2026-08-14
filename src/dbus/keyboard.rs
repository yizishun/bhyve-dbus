use zbus::interface;

use crate::console::Console;

pub struct KeyboardInterface {
	console: Console
}

impl KeyboardInterface {
    pub fn new(console: Console) -> Self {
	Self { console }
    }
}

#[interface(name = "org.qemu.Display1.Keyboard")]
impl KeyboardInterface {
	async fn press(&self, keycode: u32) {
		todo!()
	}

	async fn release(&self, keycode: u32) {
		todo!()
	}

	#[zbus(property)]
	async fn modifiers(&self) -> u32 {
		todo!()
	}
}


