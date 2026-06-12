use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

use neoview::{ScopedStoreProv, SlabId, Store, StoreProv};
use slotmap::new_key_type;
use web_sys::{Element, Event};

use crate::{apply::Applicable, build_codes::BuildCodes, context::DomContext};

new_key_type!(
	pub struct ChunkId;
);

#[derive(Default)]
pub struct Chunk {
	pub elements: Vec<Element>,
	pub events: Vec<Option<Box<dyn FnMut(&mut DomContext, Event)>>>,
}
impl Debug for Chunk {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Chunk").field("elements", &self.elements).finish()
	}
}

pub struct ChunkBuild<'ctx> {
	pub(crate) ctx: &'ctx mut DomContext,
	pub(crate) id: ChunkId,
	pub(crate) slab: Option<SlabId>,
	pub(crate) base_el: Element,
	#[doc(hidden)]
	pub build_codes: BuildCodes,
	ref_queue: Vec<(u64, Box<dyn FnOnce(&mut DomContext, &Element)>)>,
}
impl<'ctx> ChunkBuild<'ctx> {
	pub(crate) fn new(
		ctx: &'ctx mut DomContext, id: ChunkId, slab: Option<SlabId>, base_el: Element,
	) -> Self {
		Self { ctx, slab, base_el, id, build_codes: BuildCodes::new(), ref_queue: Vec::new() }
	}
	pub fn base_el(&self) -> Element {
		self.base_el.clone()
	}
	pub fn apply(&mut self, what: impl Applicable) {
		what.apply(self);
	}
	pub fn ref_el(&mut self, fun: impl FnOnce(&mut DomContext, &Element) + 'static) {
		self.ref_queue.push((self.build_codes.request_id(), Box::new(fun)));
	}
	pub fn build(self) -> Element {
		let elements = self.build_codes.construct(&self.base_el);
		for (id, fun) in self.ref_queue {
			fun(self.ctx, &elements[id as usize])
		}
		self.ctx.chunks[self.id].elements = elements;
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
impl Debug for ChunkBuild<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ChunkBuild")
			.field("ctx", &self.ctx.id)
			.field("id", &self.id)
			.field("slab", &self.slab)
			.field("base_el", &self.base_el)
			.field("build_codes", &self.build_codes)
			.field("ref_queue", &self.ref_queue.iter().map(|v| v.0).collect::<Vec<_>>())
			.finish()
	}
}

#[derive(Debug)]
pub struct RemovableChunk<'ctx>(ChunkBuild<'ctx>);
impl<'ctx> RemovableChunk<'ctx> {
	pub(crate) fn new(ctx: &'ctx mut DomContext, id: ChunkId, base_el: Element) -> Self {
		let slab = ctx.store().create_slab();
		Self(ChunkBuild::new(ctx, id, Some(slab), base_el))
	}
	pub fn export(self) -> impl Applicable {
		let (el, remover) = self.build();
		move |build: &mut ChunkBuild| {
			build.build_codes.node(el.into());
			if let Some(slab) = build.slab {
				build.store().add_cleaner(Some(slab), move |ctx| remover.remove(ctx)).unwrap()
			}
		}
	}
	pub fn build(self) -> (Element, ChunkRemover) {
		let id = self.0.id;
		let slab = self.0.slab.unwrap();
		let el = self.0.build();
		(el.clone(), ChunkRemover { id, slab, el })
	}
}
impl<'ctx> Deref for RemovableChunk<'ctx> {
	type Target = ChunkBuild<'ctx>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<'ctx> DerefMut for RemovableChunk<'ctx> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
#[derive(Debug)]
pub struct ChunkRemover {
	id: ChunkId,
	slab: SlabId,
	el: Element,
}
impl Drop for ChunkRemover {
	fn drop(&mut self) {
		panic!("dropped without calling `ChunkRemover::remove`")
	}
}
impl ChunkRemover {
	pub fn remove(self, ctx: &mut DomContext) {
		_ = Store::remove_slab(ctx, self.slab);
		ctx.chunks.remove(self.id);
		self.el.remove();
		std::mem::forget(self);
	}
}
