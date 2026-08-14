use std::{io::Result, path::PathBuf};

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::console::{BhyvegcImage, BhyvegcImageUpdate, KeyEvent, PtrEvent, VMInfo};

pub struct CtrlTask {
	ctrl_rx: mpsc::Receiver<CtrlOp>,
	conn_event_tx: mpsc::Sender<ConnEvent>,
}

pub struct PollTask {
	poll_rx: mpsc::Receiver<PollOp>,
	conn_event_tx: mpsc::Sender<ConnEvent>,
}

pub enum ConnEvent {
	CtrlDead { console_id: u32 },
	PollDead { console_id: u32 },
}

pub enum CtrlOp {
	GetImage {
		reply: oneshot::Sender<Result<BhyvegcImage>>
	},
	KeyEvent {
		event: KeyEvent,
	},
	PtrEvent {
		event: PtrEvent,
	},
	VmInfo {
		reply: oneshot::Sender<Result<VMInfo>>
	}
}

pub enum PollOp {
	PollImage {
		reply: oneshot::Sender<Result<BhyvegcImageUpdate>>
	}
}

#[derive(Clone)]
pub struct ConnHandle {
	pub ctrl_tx: mpsc::Sender<CtrlOp>,
	pub poll_tx: mpsc::Sender<PollOp>
}

impl ConnHandle {
	pub fn new(ctrl_tx: mpsc::Sender<CtrlOp>, poll_tx: mpsc::Sender<PollOp>) -> Self {
		Self { ctrl_tx, poll_tx }
	}
}

impl CtrlTask {
	pub async fn new(
		path: &PathBuf,
		event_tx: mpsc::Sender<ConnEvent>,
		ctrl_rx: mpsc::Receiver<CtrlOp>,
	) -> Result<CtrlTask> {
		todo!()
	}

	pub async fn run(& mut self) {
		todo!()
	}
}

impl PollTask {
	pub async fn new(
		path: &PathBuf,
		event_tx: mpsc::Sender<ConnEvent>,
		poll_rx: mpsc::Receiver<PollOp>,
	) -> Result<PollTask> {
		todo!()
	}

	pub async fn run(& mut self) {
		todo!()
	}
}
