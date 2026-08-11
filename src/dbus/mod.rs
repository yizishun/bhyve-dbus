pub mod mouse;
pub mod listener;
pub mod keyboard;
pub mod console;
pub mod vm;

use zbus::{Connection, Result};

use crate::dbus::vm::VM;

pub async fn set_up_dbus_server() -> Result<Connection> {
	let vm = VM::new();
	/* TODO: connect to user specific unix socket more better */
	let connection = Connection::session()
		.await?;
	connection
		.object_server()
		.at("/org/qemu/Display1/VM", vm)
		.await?;
	connection
		.request_name("org.qemu.Display1")
		.await?;
	Ok(connection)
}
