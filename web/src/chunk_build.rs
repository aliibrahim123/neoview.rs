use std::ops::{Deref, DerefMut};

use neoview::{ScopedStoreProv, SlabId, StoreProv};
use slotmap::new_key_type;
use web_sys::Element;

use crate::{build_codes::BuildCodes, context::DomContext};

new_key_type!(
	pub struct ChunkId;
);

#[derive(Debug)]
pub struct ChunkBuild<'ctx> {
	pub(crate) ctx: &'ctx mut DomContext,
	pub(crate) id: ChunkId,
	pub(crate) slab: Option<SlabId>,
	pub(crate) base_el: Element,
	pub(crate) build_codes: BuildCodes,
}
impl<'ctx> ChunkBuild<'ctx> {
	pub(crate) fn new(
		ctx: &'ctx mut DomContext, id: ChunkId, slab: Option<SlabId>, base_el: Element,
	) -> Self {
		Self { ctx, slab, base_el, id, build_codes: BuildCodes::new() }
	}
	pub fn base_el(&self) -> Element {
		self.base_el.clone()
	}
	pub fn finish(self) -> Element {
		self.build_codes.construct(self.ctx, &self.base_el, self.id);
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
pub struct RemovableChunk<'ctx>(ChunkBuild<'ctx>);
impl<'ctx> RemovableChunk<'ctx> {
	pub(crate) fn new(ctx: &'ctx mut DomContext, id: ChunkId, base_el: Element) -> Self {
		let slab = ctx.store().create_slab();
		Self(ChunkBuild::new(ctx, id, Some(slab), base_el))
	}
	pub fn finish(self) -> (Element, impl FnOnce(&mut DomContext)) {
		let id = self.0.id;
		let slab = self.0.slab.unwrap();
		let el = self.0.finish();
		(el.clone(), move |ctx| {
			ctx.chunk_el_map.remove(id);
			ctx.store().remove_slab(slab);
			el.remove();
		})
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
