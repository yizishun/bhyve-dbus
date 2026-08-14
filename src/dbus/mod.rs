pub mod mouse;
pub mod listener;
pub mod keyboard;
pub mod console;
pub mod vm;

use zbus::{Connection, Result};

use crate::{console::Console, dbus::{
	console::ConsoleInterface,
	keyboard::KeyboardInterface,
	mouse::MouseInterface,
	vm::VMInterface
}, sock_manager::RouteTable};

pub async fn set_up_dbus_server(routes: RouteTable) -> Result<Connection> {
	let vm = VMInterface::new(routes.clone()).await;
	/* TODO: connect to user specific unix socket much better */
	let connection = Connection::session()
		.await?;
	let server = connection.object_server();

	for id in &vm.console_ids {
		let console = Console::new(*id, routes.clone()).await.unwrap();
		let consoleiface = ConsoleInterface::new(console.clone());
		let keyboardiface = KeyboardInterface::new(console.clone());
		let mouseiface = MouseInterface::new(console.clone());
		let path = format!("/org/qemu/Display1/Console_{}", id);

		server.at(path.as_str(), consoleiface).await?;
		server.at(path.as_str(), keyboardiface).await?;
		server.at(path.as_str(), mouseiface).await?;
	}

	server.at("/org/qemu/Display1/VM", vm).await?;
	
	connection
		.request_name("org.qemu.Display1")
		.await?;
	Ok(connection)
}
