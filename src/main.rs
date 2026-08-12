pub mod console;
mod dbus;
mod sock_manager;

use tokio::{spawn, sync::watch, task::JoinHandle};
use zbus::{Connection, Result, conn};
use crate::{dbus::set_up_dbus_server, sock_manager::SockManager};

#[tokio::main]
async fn main() -> Result<()> {
    let (tx, mut rx) = watch::channel(false);
    let manager = SockManager::new(tx);
    let handle: JoinHandle<Result<()>> = spawn(async move {
        let mut connect : Option<Connection> = None;
        loop {
            /* Waiting for the connection and disconnection */
            if rx.changed().await.is_err() {
                break;
            }

            let has_conn = *rx.borrow_and_update();

            if has_conn && connect.is_none() {
                connect = Some(set_up_dbus_server().await?);
            } else if !has_conn {
                connect = None;
            }
        }
        Ok(())
    });
    handle.await.expect("unexpected failure")?;
    Ok(())
}
