mod dbus;

use zbus::Result;
use crate::dbus::set_up_dbus_server;

#[tokio::main]
async fn main() -> Result<()> {
    let _conn = set_up_dbus_server().await?;
    std::future::pending::<()>().await;
    Ok(())
}
