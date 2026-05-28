use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use web_sys::Element;

use crate::{
	chunk_build::{ChunkBuild, ChunkId},
	context::DomContext,
};

#[wasm_bindgen(module = "neoview-web-binder")]
extern "C" {
	pub fn construct(target_el: &Element, build_codes: Vec<u8>, args: Vec<JsValue>)
	-> Vec<Element>;

}

const COMMON_NAMES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/common_names.rs"));

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Buf(pub Vec<u8>);
impl Buf {}

#[derive(Debug)]
pub struct BuildCodes {
	codes: Vec<u8>,
	args: Vec<JsValue>,
	cur_el_id: u64,
	el_id_stack: Vec<Option<u64>>,
}
impl BuildCodes {
	pub fn new() -> Self {
		Self { codes: Vec::new(), args: Vec::new(), cur_el_id: 1, el_id_stack: vec![Some(0)] }
	}
	pub fn push_slice(&mut self, slice: &[u8]) {
		self.codes.extend_from_slice(slice);
	}
	pub fn push_vuint(&mut self, mut value: u64) {
		let mut there_input = true;

		while there_input {
			let byte = value as u8 & 0b0111_1111;
			value >>= 7;
			self.codes.push(if value == 0 { byte } else { byte | 0b1000_0000 });
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
			Err(_) => {
				self.push_vuint((str.len() << 1) as u64);
				self.push_slice(str.as_bytes());
			}
		}
	}

	const EL_START: u8 = 0;
	const EL_ID: u8 = 1;
	const ATTR: u8 = 2;
	const PROP: u8 = 3;
	const CLASS: u8 = 4;
	const STYLE: u8 = 5;
	const TEXT: u8 = 6;
	const NODE: u8 = 7;
	const END: u8 = 255;

	pub fn start_el(&mut self, tag: &str) {
		self.codes.push(Self::EL_START);
		self.push_name(tag);
		self.el_id_stack.push(None);
	}
	pub fn request_id(&mut self) -> u64 {
		if let Some(id) = self.el_id_stack.last().copied().flatten() {
			return id;
		}
		let id = self.cur_el_id;
		self.cur_el_id += 1;
		self.el_id_stack.push(Some(id));
		self.codes.push(Self::EL_ID);
		id
	}
	pub fn attr(&mut self, name: &str, value: &str) {
		self.codes.push(Self::ATTR);
		self.push_name(name);
		self.push_str(value);
	}
	pub fn prop(&mut self, name: &str, value: JsValue) {
		self.codes.push(Self::PROP);
		self.push_str(name);
		self.push_vuint(self.args.len() as u64);
		self.args.push(value);
	}
	pub fn class(&mut self, value: &str) {
		self.codes.push(Self::CLASS);
		self.push_str(value);
	}
	pub fn style(&mut self, name: &str, value: &str) {
		self.codes.push(Self::STYLE);
		self.push_name(name);
		self.push_str(value);
	}
	pub fn text(&mut self, value: &str) {
		self.codes.push(Self::TEXT);
		self.push_str(value);
	}
	pub fn node(&mut self, value: JsValue) {
		self.codes.push(Self::NODE);
		self.push_vuint(self.args.len() as u64);
		self.args.push(value);
	}
	pub fn end_el(&mut self) {
		self.codes.push(Self::END);
		self.el_id_stack.pop();
	}
	pub fn construct(self, ctx: &mut DomContext, base_el: &Element, id: ChunkId) {
		let els = construct(base_el, self.codes, self.args);
		ctx.chunk_el_map[id] = els;
	}
}
impl<'ctx> ChunkBuild<'ctx> {}

#[doc(hidden)]
pub mod __build_code {
	#[macro_export]
	#[doc(hidden)]
	macro_rules! start_el {
		($build:expr, $el:expr, $tag:expr) => {
			$build.__start_el(stringify!($tag));
		};
	}

	pub use start_el;
}
