use tokio::sync::watch;

pub struct SockManager {
	active_conn: usize,
	waker: watch::Sender<bool>
}

impl SockManager {
	pub fn new(waker: watch::Sender<bool>) -> Self {
		/* TODO: */
		let _ = waker.send(true);
		Self {
			active_conn: 0, 
			waker
		}
	}
}
