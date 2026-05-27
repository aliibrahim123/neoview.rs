use std::{
	ops::{Deref, DerefMut},
	sync::atomic::{AtomicU64, Ordering},
};

use neoview::{ScopedStoreProv, SlabId, StoreProv};
use wasm_bindgen::prelude::JsValue;
use web_sys::Element;

use crate::{binder, context::DomContext, wire::Buf};

#[derive(Debug)]
pub struct ChunkBuild<'ctx> {
	ctx: &'ctx mut DomContext,
	slab: Option<SlabId>,
	base_el: Element,
	build_codes: Buf,
	el_stack: Vec<u64>,
	args: Vec<JsValue>,
}
impl<'ctx> ChunkBuild<'ctx> {
	pub(crate) fn new(ctx: &'ctx mut DomContext, slab: Option<SlabId>, base_el: Element) -> Self {
		Self {
			ctx,
			slab,
			base_el,
			build_codes: Buf::default(),
			el_stack: vec![binder::next_el_id()],
			args: Vec::new(),
		}
	}
	pub fn base_el(&self) -> Element {
		self.base_el.clone()
	}
	pub fn finish(self) -> Element {
		binder::construct(&self.base_el, self.build_codes.0, self.args);
		self.base_el
	}
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

#[derive(Debug)]
pub struct RemovableChunk<'ctx> {
	build: ChunkBuild<'ctx>,
	id: u64,
}
impl<'ctx> RemovableChunk<'ctx> {
	pub fn new(ctx: &'ctx mut DomContext, base_el: Element) -> Self {
		let slab = ctx.store().create_slab();
		Self { build: ChunkBuild::new(ctx, Some(slab), base_el), id: binder::next_chunk_id() }
	}
	pub fn finish(self) -> (Element, impl FnOnce()) {
		(self.build.finish(), move || binder::remove_chunk(self.id))
	}
}
impl<'ctx> Deref for RemovableChunk<'ctx> {
	type Target = ChunkBuild<'ctx>;
	fn deref(&self) -> &Self::Target {
		&self.build
	}
}
impl<'ctx> DerefMut for RemovableChunk<'ctx> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.build
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
		self.build_codes.push(Self::EL_ID);
		self.el_stack.push(binder::next_el_id());
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
