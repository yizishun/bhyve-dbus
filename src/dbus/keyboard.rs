use zbus::interface;

use crate::console::{Console, KeyEvent};

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
		let event = KeyEvent {
			down: true,
			keycode: keycode,
			keysym: 0
		};
		self.console.console_key_event(event).await
			.expect("dbus: keyboard: sock fail");
	}

	async fn release(&self, keycode: u32) {
		let event = KeyEvent {
			down: false,
			keycode: keycode,
			keysym: 0
		};
		self.console.console_key_event(event).await
			.expect("dbus: keyboard: sock fail");
	}

	#[zbus(property)]
	async fn modifiers(&self) -> u32 {
		0
	}
}


