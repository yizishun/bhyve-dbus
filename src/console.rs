use std::os::fd::OwnedFd;

use tokio::sync::oneshot;

use crate::{sock_conntask::{ConnHandle, ConnOp}, sock_manager::RouteTable};

#[derive(Clone)]
pub struct Console {
	pub id: u32,
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


pub struct BhyvegcImageUpdate {
	pub dirty: Rect,
	pub image: BhyvegcImage
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
	pub fn need_update(dirty: Rect) -> bool {
		!(dirty.height == 0 && dirty.width == 0)
	}
	
	pub fn need_scanout(&self, image: &BhyvegcImage) -> bool {
		self.image.height != image.height ||
		self.image.width != image.width ||
		self.image.vgamode != image.vgamode ||
		self.image.generation != image.generation
	}
}

impl VMInfo {
	pub fn new(name: String, device_address: String) -> Self {
		Self { name, device_address }
	}
}

impl Console {
	pub fn new(id: u32, routes: RouteTable) -> Self {
		Self {
			id,
			routes,
		}
	}

	pub async fn vm_info(&self) -> std::io::Result<VMInfo> {
		let handle = self.get_handle();
		let (reply, rx) = oneshot::channel();
		handle.conn_tx.send(ConnOp::VmInfo { reply }).await
			.map_err(|_| std::io::Error::new(
				std::io::ErrorKind::BrokenPipe,
				"connection closed"
			))?;
		rx.await.map_err(|_| std::io::Error::new(
			std::io::ErrorKind::BrokenPipe,
			"reply channel closed"
		))?
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
		))?
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
		))?
	}

	fn get_handle(&self) -> ConnHandle {
		self.routes.read().unwrap()
			.get(self.id as usize).cloned().unwrap()
	}
}