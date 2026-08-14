use tokio::sync::oneshot;

use crate::{sock_conntask::{ConnHandle, CtrlOp, PollOp}, sock_manager::RouteTable};

#[derive(Clone)]
pub struct Console {
	pub id: u32,
	pub name: String,
	pub device_address: String,
	routes: RouteTable,
}

pub struct VMInfo {
	pub name: String,
	pub device_address: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BhyvegcImage {
	pub vgamode: u32,
	pub height: u32,
	pub width: u32,
	pub dmabuf: i32
}

pub struct BhyvegcImageUpdate {
	pub update: bool,
	/* the update position, in pixels. */
	pub x: i32,
	pub y: i32,
	/* the update width, height, in pixels. */
	pub width: i32,
	pub height: i32,
	pub image: BhyvegcImage
}

pub struct KeyEvent {

}

pub struct PtrEvent {

}

impl BhyvegcImage {
    	pub const fn new() -> Self {
		Self {
			vgamode: 0,
			height: 0,
			width: 0,
			dmabuf: 0
		}
	}
}

impl Console {
	pub async fn new(id: u32, routes: RouteTable) -> std::io::Result<Self> {
		/* TODO: socket to get name and device addr */
		let handle = routes.read().unwrap()
			.get(id as usize).cloned().unwrap();
		let (reply, rx) = oneshot::channel();
		handle.ctrl_tx.send(CtrlOp::VmInfo { reply }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		let vm_info = rx.await.map_err(|_| std::io::Error::new(
			std::io::ErrorKind::BrokenPipe,
			"reply channel closed"
		))?.unwrap();
		Ok(Self {
			id,
			routes,
			name: vm_info.name,
			device_address: vm_info.device_address
		})
	}

	pub fn vm_info(&self) -> VMInfo {
		VMInfo {
			name: self.name.clone(),
			device_address: self.device_address.clone()
		}
	}

	pub fn console_ids(&self) -> Vec<u32> {
		/* bhyve currently only support 1 console */
		let size : u32 = self.routes.read().unwrap().len() as u32;
		(0..size).collect()
	}

	pub async fn console_poll_image(&self) -> std::io::Result<BhyvegcImageUpdate> {
		let handle = self.get_handle();
		let (reply, rx) = oneshot::channel();
		handle.poll_tx.send(PollOp::PollImage { reply }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		rx.await.map_err(|_| std::io::Error::new(
			std::io::ErrorKind::BrokenPipe,
			"reply channel closed"
		))?
	}

	pub async fn console_get_image(&self) -> std::io::Result<BhyvegcImage> {
		let handle = self.get_handle();
		let (reply, rx) = oneshot::channel();
		handle.ctrl_tx.send(CtrlOp::GetImage { reply }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		rx.await.map_err(|_| std::io::Error::new(
			std::io::ErrorKind::BrokenPipe,
			"reply channel closed"
		))?
	}

	pub async fn console_key_event(&self, event: KeyEvent) -> std::io::Result<()> {
		let handle = self.get_handle();
		handle.ctrl_tx.send(CtrlOp::KeyEvent { event }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		Ok(())
	}

	pub async fn console_ptr_event(&self, event: PtrEvent) -> std::io::Result<()> {
		let handle = self.get_handle();
		handle.ctrl_tx.send(CtrlOp::PtrEvent { event }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		Ok(())
	}

	fn get_handle(&self) -> ConnHandle {
		self.routes.read().unwrap()
			.get(self.id as usize).cloned().unwrap()
	}
}