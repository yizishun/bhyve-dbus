pub mod mouse;
pub mod listener;
pub mod keyboard;
pub mod console;
pub mod vm;

use zbus::{Connection, Result};

use crate::dbus::{
	console::ConsoleInterface,
	keyboard::KeyboardInterface,
	mouse::MouseInterface,
	vm::VMInterface
};

pub async fn set_up_dbus_server() -> Result<Connection> {
	let vm = VMInterface::new().await;
	/* TODO: connect to user specific unix socket much better */
	let connection = Connection::session()
		.await?;
	let server = connection.object_server();

	for id in &vm.console_ids {
		let console = ConsoleInterface::new(*id);
		let keyboard = KeyboardInterface::new(*id);
		let mouse = MouseInterface::new(*id);
		let path = format!("/org/qemu/Display1/Console_{}", id);

		server.at(path.as_str(), console).await?;
		server.at(path.as_str(), keyboard).await?;
		server.at(path.as_str(), mouse).await?;
	}

	server.at("/org/qemu/Display1/VM", vm).await?;
	
	connection
		.request_name("org.qemu.Display1")
		.await?;
	Ok(connection)
}
