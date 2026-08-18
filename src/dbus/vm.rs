use sha2::Digest;
use zbus::interface;
use crate::{console::Console, sock_manager::RouteTable};

pub struct VMInterface {
	pub name: String,
	pub uuid: String,
	pub console_ids: Vec<u32>,
}

impl VMInterface {
	pub async fn new(routes: RouteTable) -> std::io::Result<Self> {
		let console = Console::new(0, routes);
		let vm_info = console.vm_info().await?;
		let hash: [u8; 16] = sha2::Sha256::digest(&vm_info.name)
			[..16]
			.try_into()
			.unwrap();
		let id = uuid::Uuid::from_bytes(hash);
		Ok(Self {
			name: vm_info.name,
			uuid: id.to_string(),
			console_ids: console.console_ids(),
		})
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
