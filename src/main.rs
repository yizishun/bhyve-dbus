pub mod console;
mod dbus;
mod sock_manager;
mod sock_conntask;

use tokio::{spawn, sync::watch, task::JoinHandle};
use zbus::{Connection, Result};
use crate::{dbus::set_up_dbus_server, sock_manager::SockManager};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, mut rx) = watch::channel(false);
    /* TODO: replace /tmp/bhyve.sock */
    let mut manager = SockManager::new_and_connect(tx, vec!["/tmp/bhyve.sock".into()]).await?;
    let routes = manager.routes();
    tokio::spawn(async move {
        manager.run().await;
    });
    let handle: JoinHandle<Result<()>> = spawn(async move {
        let mut connect : Option<Connection> = None;
        loop {
            /* Waiting for the connection and disconnection */
            if rx.changed().await.is_err() {
                break;
            }

            let has_conn = *rx.borrow_and_update();

            if has_conn && connect.is_none() {
                connect = Some(set_up_dbus_server(routes.clone()).await?);
            } else if !has_conn {
                connect = None;
            }
        }
        Ok(())
    });
    handle.await.expect("unexpected failure")?;
    Ok(())
}
