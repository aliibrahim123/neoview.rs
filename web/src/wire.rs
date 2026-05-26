use std::ops::Deref;

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
	pub fn push_str(&mut self, string: &str) {
		match COMMON_NAMES.binary_search(&string) {
			Ok(id) => self.push_vuint((id << 1) as u64 | 1),
			Err(id) => {
				self.push_vuint((id << 1) as u64);
				self.push_slice(string.as_bytes());
			}
		}
	}
}
