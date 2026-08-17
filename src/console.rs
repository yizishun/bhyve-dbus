use std::os::fd::OwnedFd;

use tokio::sync::oneshot;

use crate::{sock_conntask::{ConnHandle, ConnOp}, sock_manager::RouteTable};

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

pub struct BhyvegcImage {
	pub vgamode: u32,
	pub generation: u32,
	pub height: u32,
	pub width: u32,
	pub dmabuf: OwnedFd
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rect {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
}


#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BhyvegcImageUpdate {
	pub dirty: Rect,
	pub generation: u32,
	pub vgamode: u32,
	pub height: u32,
	pub width: u32,
}

pub struct KeyEvent {
	pub down: bool,
	pub keysym: u32,
	pub keycode: u32,
}

pub struct PtrEvent {
	pub button: u32,
	pub x: i32,
	pub y: i32,
}

impl BhyvegcImageUpdate {
	pub fn need_update(&self) -> bool {
		!(self.dirty.height == 0 && self.dirty.width == 0)
	}
	
	pub fn neen_scanout(&self, image: &BhyvegcImage) -> bool {
		self.height != image.height ||
		self.width != image.width ||
		self.vgamode != image.vgamode
	}
}

impl VMInfo {
	pub fn new(name: String, device_address: String) -> Self {
		Self { name, device_address }
	}
}

impl Console {
	pub async fn new(id: u32, routes: RouteTable) -> std::io::Result<Self> {
		/* TODO: socket to get name and device addr */
		let handle = routes.read().unwrap()
			.get(id as usize).cloned().unwrap();
		let (reply, rx) = oneshot::channel();
		handle.conn_tx.send(ConnOp::VmInfo { reply }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		let vm_info = rx.await.map_err(|_| std::io::Error::new(
			std::io::ErrorKind::BrokenPipe,
			"reply channel closed"
		))??;
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
		handle.conn_tx.send(ConnOp::PollImage { reply }).await
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
		handle.conn_tx.send(ConnOp::GetImage { reply }).await
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
		let (reply, rx) = oneshot::channel();
		handle.conn_tx.send(ConnOp::KeyEvent { event, reply }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		rx.await.map_err(|_| std::io::Error::new(
			std::io::ErrorKind::BrokenPipe,
			"reply channel closed"
		))?;
		Ok(())
	}

	pub async fn console_ptr_event(&self, event: PtrEvent) -> std::io::Result<()> {
		let handle = self.get_handle();
		let (reply, rx) = oneshot::channel();
		handle.conn_tx.send(ConnOp::PtrEvent { event, reply }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		rx.await.map_err(|_| std::io::Error::new(
			std::io::ErrorKind::BrokenPipe,
			"reply channel closed"
		))?;
		Ok(())
	}

	fn get_handle(&self) -> ConnHandle {
		self.routes.read().unwrap()
			.get(self.id as usize).cloned().unwrap()
	}
}