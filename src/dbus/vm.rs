use sha2::Digest;
use zbus::interface;
use crate::console::{Console, VMInfo};

pub struct VMInterface {
	pub name: String,
	pub uuid: String,
	pub console_ids: Vec<u32>,
}

impl VMInterface {
	pub async fn new() -> Self {
		let vm_info = VMInfo::vm_info().await;
		let hash: [u8; 16] = sha2::Sha256::digest(&vm_info.name)
			[..16]
			.try_into()
			.unwrap();
		let id = uuid::Uuid::from_bytes(hash);
		Self {
			name: vm_info.name,
			uuid: id.to_string(),
			console_ids: Console::console_ids().await,
		}
	}
}

#[interface(name = "org.qemu.Display1.VM")]
impl VMInterface {
	#[zbus(property)]
	async fn name(&self) -> &str {
		&self.name
	}

	#[zbus(property, name = "UUID")]
	async fn uuid(&self) -> &str {
		&self.uuid
	}

	#[zbus(property, name = "ConsoleIDs")]
	async fn console_ids(&self) -> &[u32] {
		&self.console_ids
	}

	#[zbus(property(emits_changed_signal = "const"))]
	async fn interfaces(&self) -> &'static [&'static str] {
		&[]
	}
}
