use zbus::interface;

pub struct KeyboardInterface {
	id: u32
}

impl KeyboardInterface {
    pub fn new(id: u32) -> Self {
	Self { id }
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


