use std::sync::atomic::{AtomicU64, Ordering};

use neoview::{
	prelude::{ScopedStoreProv, StoreProv},
	reactive::SlabId,
};
use wasm_bindgen::prelude::JsValue;

use crate::{context::DomContext, wire::Buf};

#[derive(Debug)]
pub struct ChunkBuild<'ctx> {
	ctx: &'ctx mut DomContext,
	slab: Option<SlabId>,
	build_codes: Buf,
	el_stack: Vec<u64>,
	args: Vec<JsValue>,
}
impl StoreProv for ChunkBuild<'_> {
	type Ctx = DomContext;
	fn ctx(&mut self) -> &mut Self::Ctx {
		self.ctx
	}
	fn ctx_ref(&self) -> &Self::Ctx {
		self.ctx
	}
}
impl ScopedStoreProv for ChunkBuild<'_> {
	fn slab(&self) -> Option<SlabId> {
		self.slab
	}
}

impl<'ctx> ChunkBuild<'ctx> {
	const EL_START: u8 = 0;
	const EL_ID: u8 = 1;
	const ATTR: u8 = 2;
	const PROP: u8 = 3;
	const CLASS: u8 = 4;
	const STYLE: u8 = 5;
	const TEXT: u8 = 6;
	const NODE: u8 = 7;
	const END: u8 = 255;
	#[doc(hidden)]
	pub fn __start_el(&mut self, tag: &str) {
		self.build_codes.push(Self::EL_START);
		self.build_codes.push_name(tag);
		self.el_stack.push(0);
	}
	#[doc(hidden)]
	pub fn __el_id(&mut self) {
		static CUR_ID: AtomicU64 = AtomicU64::new(1);
		self.build_codes.push(Self::EL_ID);
		self.el_stack.push(CUR_ID.fetch_add(1, Ordering::Relaxed));
	}
	#[doc(hidden)]
	pub fn __attr(&mut self, name: &str, value: &str) {
		self.build_codes.push(Self::ATTR);
		self.build_codes.push_name(name);
		self.build_codes.push_str(value);
	}
	#[doc(hidden)]
	pub fn __prop(&mut self, name: &str, value: JsValue) {
		self.build_codes.push(Self::PROP);
		self.build_codes.push_str(name);
		self.build_codes.push_vuint(self.args.len() as u64);
		self.args.push(value);
	}
	#[doc(hidden)]
	pub fn __class(&mut self, value: &str) {
		self.build_codes.push(Self::CLASS);
		self.build_codes.push_str(value);
	}
	#[doc(hidden)]
	pub fn __style(&mut self, name: &str, value: &str) {
		self.build_codes.push(Self::STYLE);
		self.build_codes.push_name(name);
		self.build_codes.push_str(value);
	}
	#[doc(hidden)]
	pub fn __text(&mut self, value: &str) {
		self.build_codes.push(Self::TEXT);
		self.build_codes.push_str(value);
	}
	#[doc(hidden)]
	pub fn __node(&mut self, value: JsValue) {
		self.build_codes.push(Self::NODE);
		self.build_codes.push_vuint(self.args.len() as u64);
		self.args.push(value);
	}
	#[doc(hidden)]
	pub fn __end_el(&mut self) {
		self.build_codes.push(Self::END);
		self.el_stack.pop();
	}
}
