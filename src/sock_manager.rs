use std::{io::{Error, ErrorKind, Result}, path::PathBuf, sync::{Arc, RwLock}};

use tokio::sync::{mpsc, watch};

use crate::sock_conntask::{ConnEvent, ConnHandle, ConnTask};

pub type RouteTable = Arc<RwLock<Vec<ConnHandle>>>;

const MAX_CONSOLE: usize = 32;
const MAX_CONSOLE_REQ: usize = 32;

pub struct SockManager {
	active_conn: usize,

	/* To kill main */
	waker: watch::Sender<bool>,

	/* To recv conn task die event */
	conn_event_rx: mpsc::Receiver<ConnEvent>,

	routes: RouteTable,
	
	endpoints: Vec<PathBuf>
}

impl SockManager {
	pub async fn new_and_connect(waker: watch::Sender<bool>, endpoints: Vec<PathBuf>) -> Result<Self> {
		let mut active_conn = 0;
		let mut routes: Vec<ConnHandle> = vec![];

		if endpoints.len() > MAX_CONSOLE {
			return Err(Error::new(
				ErrorKind::InvalidInput, 
				format!("Endpoints count exceeds MAX_CONSOLE limit ({})", MAX_CONSOLE)
			));
		}
		let (conn_event_tx, conn_event_rx) = mpsc::channel(MAX_CONSOLE);
		for (id, endpoint) in endpoints.iter().enumerate() {
			let (conn_tx, conn_rx) = mpsc::channel(MAX_CONSOLE_REQ);
			let mut ctask = ConnTask::new(id as u32, &endpoint, conn_event_tx.clone(), conn_rx).await?;
			let conn_handle = ConnHandle::new(conn_tx);

			tokio::spawn(async move {
				ctask.run().await;
			});

			routes.push(conn_handle);
			active_conn += 1;
		}
		if active_conn > 0 {
			let _ = waker.send(true);
		}
		Ok(Self {
			active_conn,
			conn_event_rx,
			routes: Arc::new(RwLock::new(routes)),
			waker,
			endpoints
		})
	}

	pub async fn run(&mut self) {
		while let Some(m) = self.conn_event_rx.recv().await {
			match m {
				ConnEvent::Dead { console_id } => 
					eprintln!("sock: socket {} disconnect", 
						self.endpoints[console_id as usize].to_str().unwrap()),
			}
			
			self.active_conn -= 1;
			if self.active_conn == 0 {
				let _ = self.waker.send(false);
				break;
			}
		}

	}

	pub fn routes(&self) -> RouteTable {
		self.routes.clone()
	}
}
