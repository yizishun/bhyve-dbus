 /* This file MUST stay in sync with bhyve console_ipc.h */

use std::io::{Error, ErrorKind, Result};

use crate::console::{KeyEvent, PtrEvent};

/* Request commands */
pub const WIRE_CMD_REQ_VM_INFO: u16 = 0x0001;
pub const WIRE_CMD_REQ_GET_IMAGE: u16 = 0x0002;
pub const WIRE_CMD_REQ_POLL_IMAGE: u16 = 0x0003;
pub const WIRE_CMD_REQ_KEY_EVENT: u16 = 0x0004;
pub const WIRE_CMD_REQ_PTR_EVENT: u16 = 0x0005;

/* Response commands = request | 0x8000 */
pub const WIRE_CMD_RESP_VM_INFO: u16 = 0x8001;
pub const WIRE_CMD_RESP_GET_IMAGE: u16 = 0x8002;
pub const WIRE_CMD_RESP_POLL_IMAGE: u16 = 0x8003;
pub const WIRE_CMD_RESP_KEY_EVENT: u16 = 0x8004;
pub const WIRE_CMD_RESP_PTR_EVENT: u16 = 0x8005;

/* flags */
pub const WIRE_FLAG_HAS_FD: u16 = 0x0001;

pub const WIRE_VM_NAME_MAX: usize = 128;
pub const WIRE_DEV_ADDR_MAX: usize = 64;
pub const WIRE_MSG_MAX: usize = 512;
pub const WIRE_ABUF_MAX: usize = 512;

/* header */
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireHdr {
	pub cmd: u16,
	pub flags: u16,
}

/* Per-request payload */
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireReqVmInfo {
	pub hdr: WireHdr,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireReqGetImage {
	pub hdr: WireHdr,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireReqPollImage {
	pub hdr: WireHdr,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireReqKeyEvent {
	pub hdr: WireHdr,
	pub down: u32,
	pub keysym: u32,
	pub keycode: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireReqPtrEvent {
	pub hdr: WireHdr,
	pub button: u32,
	pub x: i32,
	pub y: i32,
}

/* Per-response payload */
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireRespVmInfo {
	pub hdr: WireHdr,
	pub code: u32, /* 0 = success, otherwise errno code */
	pub name: [u8; WIRE_VM_NAME_MAX],     /* NUL-terminated */
	pub dev_addr: [u8; WIRE_DEV_ADDR_MAX], /* NUL-terminated */
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireRespGetImage {
	pub hdr: WireHdr,
	pub code: u32,
	pub generation: u32,
	pub vgamode: u32,
	pub width: u32,
	pub height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireRespPollImage {
	pub hdr: WireHdr,
	pub code: u32,
	pub generation: u32,
	pub vgamode: u32,
	pub width: u32,
	pub height: u32,
	pub dirty_x: i32,
	pub dirty_y: i32,
	pub dirty_w: i32,
	pub dirty_h: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireRespKeyEvent {
	pub hdr: WireHdr,
	pub code: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireRespPtrEvent {
	pub hdr: WireHdr,
	pub code: u32,
}

pub trait WireMsg: Copy {
	const CMD: u16;

	fn new_hdr(flags: Option<u16>) -> WireHdr {
		WireHdr {
			cmd: Self::CMD,
			flags: flags.unwrap_or(0)
		}
	}

	fn as_bytes(&self) -> &[u8] {
		unsafe {
			std::slice::from_raw_parts(
				(self as *const Self) as *const u8,
				std::mem::size_of::<Self>(),
			)
		}
	}

	fn from_bytes(buf: &[u8]) -> Result<Self> {
		if buf.len() != std::mem::size_of::<Self>() {
			return Err(Error::new(
				ErrorKind::InvalidData,
				format!(
					"wire: bad message size: got {}, expect {}",
					buf.len(),
					std::mem::size_of::<Self>()
				),
			));
		}
		let msg = unsafe {
			std::ptr::read_unaligned(buf.as_ptr() as *const Self)
		};
		let hdr = unsafe {
			std::ptr::read_unaligned(buf.as_ptr() as *const WireHdr)
		};
		if hdr.cmd != Self::CMD {
			return Err(Error::new(
				ErrorKind::InvalidData,
				format!(
					"wire: unexpected cmd: got {:#06x}, expect {:#06x}",
					hdr.cmd,
					Self::CMD
				),
			));
		}
		Ok(msg)
	}

}

impl WireMsg for WireReqVmInfo {
	const CMD: u16 = WIRE_CMD_REQ_VM_INFO;
}
impl WireMsg for WireReqGetImage {
	const CMD: u16 = WIRE_CMD_REQ_GET_IMAGE;
}
impl WireMsg for WireReqPollImage {
	const CMD: u16 = WIRE_CMD_REQ_POLL_IMAGE;
}
impl WireMsg for WireReqKeyEvent {
	const CMD: u16 = WIRE_CMD_REQ_KEY_EVENT;
}
impl WireMsg for WireReqPtrEvent {
	const CMD: u16 = WIRE_CMD_REQ_PTR_EVENT;
}
impl WireMsg for WireRespVmInfo {
	const CMD: u16 = WIRE_CMD_RESP_VM_INFO;
}
impl WireMsg for WireRespGetImage {
	const CMD: u16 = WIRE_CMD_RESP_GET_IMAGE;
}
impl WireMsg for WireRespPollImage {
	const CMD: u16 = WIRE_CMD_RESP_POLL_IMAGE;
}
impl WireMsg for WireRespKeyEvent {
	const CMD: u16 = WIRE_CMD_RESP_KEY_EVENT;
}
impl WireMsg for WireRespPtrEvent {
	const CMD: u16 = WIRE_CMD_RESP_PTR_EVENT;
}

impl WireReqVmInfo {
	pub fn new() -> Self {
		Self { hdr: Self::new_hdr(None) }
	}
}

impl WireReqGetImage {
	pub fn new() -> Self {
		Self { hdr: Self::new_hdr(Some(WIRE_FLAG_HAS_FD)) }
	}
}

impl WireReqPollImage {
	pub fn new() -> Self {
		Self { hdr: Self::new_hdr(None) }
	}
}

impl WireReqKeyEvent {
	pub fn new(event: KeyEvent) -> Self {
		Self {
			hdr: Self::new_hdr(None),
			down: event.down as u32,
			keysym: event.keysym,
			keycode: event.keycode
		}
	}
}

impl WireReqPtrEvent {
	pub fn new(event: PtrEvent) -> Self {
		Self {
			hdr: Self::new_hdr(None),
			button: event.button,
			x: event.x,
			y: event.y
		}
	}
}

/* Map a non-zero wire code (errno-style) to std::io::Error. */
pub fn wire_code_to_result<T>(code: u32, value: T) -> Result<T> {
	if code == 0 {
		Ok(value)
	} else {
		Err(Error::from_raw_os_error(code as i32))
	}
}

/* Read a NUL-terminated fixed C string field into a String. */
pub fn wire_cstr_to_string(buf: &[u8]) -> Result<String> {
	let end = buf.iter().position(|&b| b == 0).ok_or_else(|| {
		Error::new(ErrorKind::InvalidData, "wire: string not NUL-terminated")
	})?;
	String::from_utf8(buf[..end].to_vec())
		.map_err(|_| Error::new(ErrorKind::InvalidData, "wire: invalid UTF-8"))
}
