//! defines the chunk and its builds
use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

use neoview::{ScopedStoreProv, SlabId, Store, StoreProv};
use slotmap::new_key_type;
use web_sys::{Element, Event};

use crate::{apply::Applicable, build_codes::BuildCodes, context::DomContext};

new_key_type!(
	/// a unique identifier for a chunk
	pub struct ChunkId;
);

/// chunk data
#[derive(Default)]
pub struct Chunk {
	pub elements: Vec<Element>,
	/// event listeners
	pub events: Vec<Option<Box<dyn FnMut(&mut DomContext, Event)>>>,
}
impl Debug for Chunk {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Chunk").field("elements", &self.elements).finish()
	}
}

/// a chunk under construction.
///
/// the `ChunkBuild` is the interface and the builder that is used while contructing a chunk.
///
/// it borrows the [`DomContext`], target a base [`Element`] and a specific [scope](Store#slab-management) and expose the [`Store`] through [`StoreProv`].
///
/// ui can be appended to in a tree like manner through the [`chunk`](crate::chunk!) macro and the [`apply`](crate::apply) module.
///
/// the ui definition get recorded in a buffer and is built in oneshot at the end by calling the [`build`](ChunkBuild::build) function.
///
/// multiple chunks can target the same base [`Element`] as the `ChunkBuild` just append its ui to it.
///
/// after a chunk is build, the constructed ui can be altered in whatever way, the bindings target the specific [`Element`]s directly.
///
/// # example
/// ```
/// let el = window().unwrap().document().unwrap().create_element("div").unwrap();
/// let mut build = root_build.ctx().new_chunk(el);
/// chunk!(build, div(id: "section") { "hello world" });
/// build.apply(div((id("section"), text("hello world"))));
/// build.build();
/// chunk!(root_build, el);
/// ```
pub struct ChunkBuild<'ctx> {
	/// the context
	pub(crate) ctx: &'ctx mut DomContext,
	/// the chunk id
	pub(crate) id: ChunkId,
	/// the slab id
	pub(crate) slab: Option<SlabId>,
	/// the base element
	pub(crate) base_el: Element,
	#[doc(hidden)]
	pub build_codes: BuildCodes,
	/// queue of `ref_el` callbacks: (el_id, fun)
	ref_queue: Vec<(u64, Box<dyn FnOnce(&mut DomContext, &Element)>)>,
}
impl<'ctx> ChunkBuild<'ctx> {
	/// creates a new [`ChunkBuild`]
	pub(crate) fn new(
		ctx: &'ctx mut DomContext, id: ChunkId, slab: Option<SlabId>, base_el: Element,
	) -> Self {
		Self { ctx, slab, base_el, id, build_codes: BuildCodes::new(), ref_queue: Vec::new() }
	}
	/// returns the base [`Element`] of the chunk.
	pub fn base_el(&self) -> Element {
		self.base_el.clone()
	}

	/// applies the [`Applicable`] to current element.
	///
	/// see [`apply`](crate::apply) module for more information.
	///
	/// # example
	/// ```
	/// build.apply(div((id("section"), text("hello world"))));
	/// ```
	pub fn apply(&mut self, what: impl Applicable) {
		what.apply(self);
	}

	/// get a reference to the current element through a callback.
	///
	/// the callback will be called after the chunk is built but before the [`Element`] is returned.
	///
	/// # example
	/// ```
	/// build.ref_el(|ctx, el| println!("{}", el.text_content().unwrap()));
	/// ```
	pub fn ref_el(&mut self, fun: impl FnOnce(&mut DomContext, &Element) + 'static) {
		self.ref_queue.push((self.build_codes.request_id(), Box::new(fun)));
	}

	/// builds the chunk.
	///
	/// the chunk is built in oneshot and get appended to the base [`Element`] that gets returned.
	///
	/// # example
	/// ```
	/// let el = window().unwrap().document().unwrap().create_element("div").unwrap();
	/// let mut build = root_build.ctx().new_chunk(el);
	/// chunk!(build, div { "hello world" });
	/// build.build();
	/// chunk!(root_build, el);
	/// ```
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
	/// returns the [`SlabId`] of the chunk.
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

/// a chunk that can be removed.
///
/// `RemovableChunk` is a [`ChunkBuild`] having its own scope and being able to be removed when needed.
///
/// it implements [`Deref`] to [`ChunkBuild`], so all the functionality of [`ChunkBuild`] can be used.
///
/// chunk doesnt get removed if it is dropped or its element is removed, an explicit call to [`remove`](ChunkRemover::remove) is required.
///
/// # example
/// ```
/// let mut build = root_build.ctx().removable_chunk("div");
/// chunk!(build, "hello world");
/// let (el, remover) = build.build();
/// let mut remover = Some(remover);
/// chunk!(root_build, el,
/// 	button(on.click: (move |ctx, _| remover.take().unwrap().remove(ctx))) { "remove" }
/// );
/// ```
#[derive(Debug)]
pub struct RemovableChunk<'ctx>(ChunkBuild<'ctx>);
impl<'ctx> RemovableChunk<'ctx> {
	/// creates a new [`RemovableChunk`]
	pub(crate) fn new(ctx: &'ctx mut DomContext, id: ChunkId, base_el: Element) -> Self {
		let slab = ctx.store().create_slab();
		Self(ChunkBuild::new(ctx, id, Some(slab), base_el))
	}
	/// build the chunk and export it as an [`Applicable`].
	///
	/// it builds the chunk then returns it as an [`Applicable`] that insert the chunk into another chunk and handle removing the chunk when the parent chunk is removed.
	///
	/// # example
	/// ```
	/// let mut build = root_build.ctx().removable_chunk("div");
	/// chunk!(build, "hello world");
	/// let chunk = build.export();
	/// chunk!(root_build, chunk);
	/// ```
	pub fn export(self) -> impl Applicable {
		let (el, remover) = self.build();
		move |build: &mut ChunkBuild| {
			build.build_codes.node(el.into());
			let slab = build.slab;
			build.store().add_cleaner(slab, move |ctx| remover.remove(ctx)).unwrap()
		}
	}

	/// build the chunk and return the [`Element`] and [`ChunkRemover`].
	///
	/// # example
	/// ```
	/// let mut build = root_build.ctx().removable_chunk("div");
	/// chunk!(build, "hello world");
	/// let (el, remover) = build.build();
	/// let mut remover = Some(remover);
	/// chunk!(root_build, el,
	/// 	button(on.click: (move |ctx, _| remover.take().unwrap().remove(ctx))) { "remove" }
	/// );
	/// ```
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

/// the remover of a [`RemovableChunk`].
///
/// it can not be dropped, an explicit call to [`remove`](ChunkRemover::remove) is required to.
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
	/// remove the chunk with its [`Element`] and [slab](Store#slab-management).
	///
	/// this method is required to be called.
	///
	/// # example
	/// ```
	/// let mut build = root_build.ctx().removable_chunk("div");
	/// chunk!(build, "hello world");
	/// let (el, remover) = build.build();
	/// let mut remover = Some(remover);
	/// chunk!(root_build, el,
	/// 	button(on.click: (move |ctx, _| remover.take().unwrap().remove(ctx))) { "remove" }
	/// );
	/// ```
	pub fn remove(self, ctx: &mut DomContext) {
		// the slab may be removed already (like when the final ctx drop).
		_ = Store::remove_slab(ctx, self.slab);
		ctx.chunks.remove(self.id);
		self.el.remove();
		// do not run the panic
		std::mem::forget(self);
	}
}
