use zbus::interface;

pub struct MouseInterface {
	id: u32
}

impl MouseInterface {
    pub fn new(id: u32) -> Self {
	Self { id }
    }
}

#[interface(name = "org.qemu.Display1.Mouse")]
impl MouseInterface {
	async fn press(&self, button: u32) {
		todo!()
	}

	async fn rel_motion(&self, dx: i32, dy: i32) {
		todo!()
	}

	async fn release(&self, button: u32) {
		todo!()
	}

	async fn set_abs_position(&self, x: u32, y: u32) {
		todo!()
	}

	#[zbus(property)]
	async fn is_absolute(&self) -> bool {
		todo!()
	}
}


