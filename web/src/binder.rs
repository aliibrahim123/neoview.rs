use std::sync::atomic::{AtomicU64, Ordering};

use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use web_sys::Element;

static CUR_EL_ID: AtomicU64 = AtomicU64::new(1);
pub fn next_el_id() -> u64 {
	CUR_EL_ID.fetch_add(1, Ordering::Relaxed)
}
static CUR_CHUNK_ID: AtomicU64 = AtomicU64::new(1);
pub fn next_chunk_id() -> u64 {
	CUR_CHUNK_ID.fetch_add(1, Ordering::Relaxed)
}

#[wasm_bindgen(module = "neoview-web-binder")]
extern "C" {
	pub fn construct(target_el: &Element, chunk_id: u64, build_codes: Vec<u8>, args: Vec<JsValue>);
	pub fn remove_chunk(chunk_id: u64);
}

const COMMON_NAMES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/common_names.rs"));

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Buf(pub Vec<u8>);
impl Buf {
	pub fn push(&mut self, byte: u8) {
		self.0.push(byte);
	}
	pub fn push_slice(&mut self, slice: &[u8]) {
		self.0.extend_from_slice(slice);
	}
	pub fn push_vuint(&mut self, mut value: u64) {
		let mut there_input = true;

		while there_input {
			let byte = value as u8 & 0b0111_1111;
			value >>= 7;
			self.0.push(if value == 0 { byte } else { byte | 0b1000_0000 });
			there_input = value != 0;
		}
	}
	pub fn push_str(&mut self, str: &str) {
		self.push_vuint(str.len() as u64);
		self.push_slice(str.as_bytes());
	}
	pub fn push_name(&mut self, str: &str) {
		match COMMON_NAMES.binary_search(&str) {
			Ok(id) => self.push_vuint((id << 1) as u64 | 1),
			Err(_) => self.push_str(str),
		}
	}
}
