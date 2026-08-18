pub mod mouse;
pub mod listener;
pub mod keyboard;
pub mod console;
pub mod multi_touch;
pub mod vm;

use zbus::{Connection, Result, fdo::ObjectManager};

use crate::{console::Console, dbus::{
	console::ConsoleInterface,
	keyboard::KeyboardInterface,
	mouse::MouseInterface,
	multi_touch::MultiTouchInterface,
	vm::VMInterface
}, sock_manager::RouteTable};

pub async fn set_up_dbus_server(routes: RouteTable) -> Result<Connection> {
	let vm = VMInterface::new(routes.clone()).await?;
	/* TODO: connect to user specific unix socket much better */
	let connection = Connection::session()
		.await?;
	let server = connection.object_server();

	for id in &vm.console_ids {
		let console = Console::new(*id, routes.clone());
		let console_iface = ConsoleInterface::new(console.clone());
		let keyboard_iface = KeyboardInterface::new(console.clone());
		let mouse_iface = MouseInterface::new(console.clone());
		let multi_touch_iface = MultiTouchInterface::new(console.clone());
		let path = format!("/org/qemu/Display1/Console_{}", id);

		server.at(path.as_str(), console_iface).await?;
		server.at(path.as_str(), keyboard_iface).await?;
		server.at(path.as_str(), mouse_iface).await?;
		server.at(path.as_str(), multi_touch_iface).await?;
	}

	server.at("/org/qemu/Display1/VM", vm).await?;
	server.at("/org/qemu/Display1", ObjectManager).await?;
	
	connection
		.request_name("org.qemu.Display1")
		.await?;
	connection
		.request_name("org.qemu")
		.await?;
	Ok(connection)
}
