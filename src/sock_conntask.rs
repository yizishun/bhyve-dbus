use std::io::Error;
use std::io::Result;
use std::os::fd::OwnedFd;

use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_seqpacket::UnixSeqpacket;
use tokio_seqpacket::ancillary::AncillaryMessage;

use crate::console::Rect;
use crate::console::{BhyvegcImage, BhyvegcImageUpdate, KeyEvent, PtrEvent, VMInfo};
use crate::sock_wire::WIRE_ABUF_MAX;
use crate::sock_wire::WIRE_MSG_MAX;
use crate::sock_wire::WireMsg;
use crate::sock_wire::WireReqGetImage;
use crate::sock_wire::WireReqKeyEvent;
use crate::sock_wire::WireReqPollImage;
use crate::sock_wire::WireReqPtrEvent;
use crate::sock_wire::WireReqVmInfo;
use crate::sock_wire::WireRespGetImage;
use crate::sock_wire::WireRespKeyEvent;
use crate::sock_wire::WireRespPollImage;
use crate::sock_wire::WireRespPtrEvent;
use crate::sock_wire::WireRespVmInfo;
use crate::sock_wire::wire_cstr_to_string;

pub struct ConnTask {
	id: u32,
	conn_rx: mpsc::Receiver<ConnOp>,
	conn_event_tx: mpsc::Sender<ConnEvent>,
	sock: UnixSeqpacket
}

pub enum ConnEvent {
	/* TODO: add more type of error */
	Dead { console_id: u32 },
}

pub enum ConnOp {
	GetImage {
		reply: oneshot::Sender<Result<BhyvegcImage>>
	},
	PollImage {
		reply: oneshot::Sender<Result<BhyvegcImageUpdate>>
	},
	KeyEvent {
		event: KeyEvent,
		reply: oneshot::Sender<Result<()>>,
	},
	PtrEvent {
		event: PtrEvent,
		reply: oneshot::Sender<Result<()>>,
	},
	VmInfo {
		reply: oneshot::Sender<Result<VMInfo>>
	}
}
#[derive(Clone)]
pub struct ConnHandle {
	pub conn_tx: mpsc::Sender<ConnOp>,
}

impl ConnHandle {
	pub fn new(conn_tx: mpsc::Sender<ConnOp>) -> Self {
		Self { conn_tx }
	}
}

impl ConnTask {
	pub async fn new(
		id: u32,
		path: &std::path::Path,
		conn_event_tx: mpsc::Sender<ConnEvent>,
		conn_rx: mpsc::Receiver<ConnOp>,
	) -> Result<ConnTask> {
		let sock = UnixSeqpacket::connect(path).await?;
		Ok(Self { id, conn_rx, conn_event_tx, sock })
	}

	pub async fn run(&mut self) {
		loop {
			match self.conn_rx.recv().await {
				None => break,
				Some(msg_op) => {
					if self.handle_op(msg_op).await.is_err() {
						break;
					}
				}
			}
			    
		}
		/* report the disconnect */
		let _ = self.conn_event_tx
			.send(ConnEvent::Dead { console_id: self.id })
			.await;
	}

	async fn handle_op(&mut self, op: ConnOp) -> Result<()> {
		match op {
			ConnOp::VmInfo { reply } => {
				let req = WireReqVmInfo::new();
				let mut buffer = [0u8; WIRE_MSG_MAX];
				let len = self.send_and_recv(req, &mut buffer).await?;
				let resp = WireRespVmInfo::from_bytes(&buffer[0..len])?;
				if resp.code != 0 {
					let _ = reply.send(Err(Error::from_raw_os_error(resp.code as i32)));
					return Ok(());
				}

				let vm_info = VMInfo::new(
					wire_cstr_to_string(&resp.name)?,
					wire_cstr_to_string(&resp.dev_addr)?
				);
				let _ = reply.send(Ok(vm_info));
				Ok(())
			},
			ConnOp::GetImage { reply } => {
				let req = WireReqGetImage::new();
				let mut buffer = [0u8; WIRE_MSG_MAX];
				let (len, fd) = self.send_and_recv_with_ancillary(req, &mut buffer).await?;
				let resp = WireRespGetImage::from_bytes(&buffer[0..len])?;
				if resp.code != 0 {
					let _ = reply.send(Err(Error::from_raw_os_error(resp.code as i32)));
					return Ok(());
				}

				let gc_image = BhyvegcImage {
					width: resp.width,
					vgamode: resp.vgamode,
					generation: resp.generation,
					height: resp.height,
					dmabuf: fd
				};

				let _ = reply.send(Ok(gc_image));
				Ok(())
			},
			ConnOp::PollImage { reply } => {
				let req = WireReqPollImage::new();
				let mut buffer = [0u8; WIRE_MSG_MAX];
				let len = self.send_and_recv(req, &mut buffer).await?;
				let resp = WireRespPollImage::from_bytes(&buffer[0..len])?;
				if resp.code != 0 {
					let _ = reply.send(Err(Error::from_raw_os_error(resp.code as i32)));
					return Ok(());
				}

				let gc_update = BhyvegcImageUpdate {
					generation: resp.generation,
					vgamode: resp.vgamode,
					height: resp.height,
					width: resp.width,
					dirty: Rect {
						x: resp.dirty_x,
						y: resp.dirty_y,
						width: resp.dirty_w,
						height: resp.dirty_h,
					}
				};
				let _ = reply.send(Ok(gc_update));
				Ok(())
			},
			ConnOp::KeyEvent { event, reply } => {
				let req = WireReqKeyEvent::new(event);

				let mut buffer = [0u8; WIRE_MSG_MAX];
				let len = self.send_and_recv(req, &mut buffer).await?;
				let resp = WireRespKeyEvent::from_bytes(&buffer[0..len])?;
				if resp.code != 0 {
					let _ = reply.send(Err(Error::from_raw_os_error(resp.code as i32)));
					return Ok(());
				}

				let _ = reply.send(Ok(()));
				Ok(())
			},
			ConnOp::PtrEvent { event, reply } => {
				let req = WireReqPtrEvent::new(event);

				let mut buffer = [0u8; WIRE_MSG_MAX];
				let len = self.send_and_recv(req, &mut buffer).await?;
				let resp = WireRespPtrEvent::from_bytes(&buffer[0..len])?;
				if resp.code != 0 {
					let _ = reply.send(Err(Error::from_raw_os_error(resp.code as i32)));
					return Ok(());
				}

				let _ = reply.send(Ok(()));
				Ok(())
			},
		}
	}

	async fn send_and_recv<T: WireMsg>(&mut self, value: T, buf: &mut [u8]) -> Result<usize> {
		self.sock.send(value.as_bytes()).await?;

		let msg = self.sock.recv(buf).await?;
		let len = msg.bytes_read();
		if len == 0 {
			return Err(Error::new(
				std::io::ErrorKind::UnexpectedEof,
				"wire sock: unexpectedEof"));
		}
		if msg.truncated() {
			return Err(Error::new(
				std::io::ErrorKind::InvalidData,
				"wire sock: too long"));
		}
		Ok(len)
	}
	async fn send_and_recv_with_ancillary<T: WireMsg>(&mut self, value: T, buf: &mut [u8]) -> Result<(usize, OwnedFd)> {
		self.sock.send(value.as_bytes()).await?;
		let mut abuf = [0u8; WIRE_ABUF_MAX];
		let mut recved_fd :Option<OwnedFd> = None;

		let (msg, areader) = self.sock.recv_with_ancillary(buf, &mut abuf).await?;
		for amsg in areader.messages() {
			if let AncillaryMessage::FileDescriptors(fds) = amsg {
				if fds.len() != 1 {
					return Err(Error::new(
						std::io::ErrorKind::InvalidData,
						"wire sock: invalid number of fd"
					))
				}
				let borrawed_fd = fds.get(0).unwrap();
				let owned_fd = borrawed_fd.try_clone_to_owned()?;
				recved_fd = Some(owned_fd);
			}
		}
		let len = msg.bytes_read();
		if len == 0 {
			return Err(Error::new(
				std::io::ErrorKind::UnexpectedEof,
				"wire sock unexpectedEof"));
		}
		if msg.truncated() {
			return Err(Error::new(
				std::io::ErrorKind::InvalidData,
				"wire sock too long"));
		}
		let fd = recved_fd
			.ok_or_else(|| Error::new(
				std::io::ErrorKind::InvalidData,
				"wire sock: no fd received")
			)?;
		Ok((len, fd))
	}

}
