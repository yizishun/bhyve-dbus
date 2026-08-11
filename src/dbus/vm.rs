use zbus::interface;

pub struct VM {
	name: String,
	uuid: String,
	console_ids: Vec<u32>,
}

impl VM {
	/* TODO: */
	pub fn new() -> Self {
		VM { 
			name: "default".to_string(),
			uuid: "1111".to_string(),
			console_ids: vec![],
		}
	}
}

#[interface(name = "org.qemu.Display1.VM")]
impl VM {
	#[zbus(property)]
	async fn name(&self) -> &str {
		&self.name
	}
	
	#[zbus(property)]
	async fn uuid(&self) -> &str {
		&self.uuid
	}

	#[zbus(property)]
	async fn console_ids(&self) -> &[u32] {
		&self.console_ids
	}

	#[zbus(property(emits_changed_signal = "const"))]
	async fn interfaces(&self) -> &'static [&'static str] {
		&[]
	}
}
