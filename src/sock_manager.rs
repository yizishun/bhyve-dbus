use std::{io::{Error, ErrorKind, Result}, path::PathBuf, sync::{Arc, RwLock}};

use tokio::sync::{mpsc, watch};

use crate::sock_conntask::{ConnEvent, ConnHandle, CtrlTask, PollTask};

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
		let (conn_event_tx, conn_event_rx) = mpsc::channel(MAX_CONSOLE * 2);
		for endpoint in &endpoints {
			let (ctrl_tx, ctrl_rx) = mpsc::channel(MAX_CONSOLE_REQ);
			let (poll_tx, poll_rx) = mpsc::channel(MAX_CONSOLE_REQ);
			let mut ctask = CtrlTask::new(&endpoint, conn_event_tx.clone(), ctrl_rx).await?;
			let mut ptask = PollTask::new(&endpoint, conn_event_tx.clone(), poll_rx).await?;
			let conn_handle = ConnHandle::new(ctrl_tx, poll_tx);

			tokio::spawn(async move {
				ctask.run().await;
			});
			tokio::spawn(async move {
				ptask.run().await;
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
		/* TODO: check the active connect in a loop, poll the conn_event_rx */
		todo!()
	}

	pub fn routes(&self) -> RouteTable {
		self.routes.clone()
	}
}
