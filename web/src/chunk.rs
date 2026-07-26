//! Defines the chunk and its builds.
use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
};

use neoview::{ScopedStoreProv, SlabId, Store, StoreProv};
use slotmap::new_key_type;
use web_sys::Element;

use crate::{
	apply::Applicable, build_codes::BuildCodes, context::DomContext, prelude::__buildcode::EventFn,
};

new_key_type!(
	/// A unique identifier for a chunk.
	pub struct ChunkId;
);

/// Chunk data.
#[derive(Default)]
pub struct ChunkData {
	pub elements: Vec<Element>,
	/// Event listeners.
	pub events: Vec<Option<EventFn>>,
}
impl Debug for ChunkData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Chunk").field("elements", &self.elements).finish()
	}
}

/// The state of a [`ChunkBuild`].
pub struct BuildState {
	/// The chunk ID.
	pub id: ChunkId,
	/// The slab ID.
	pub slab: Option<SlabId>,
	/// The base element.
	pub base_el: Element,
	pub build_codes: BuildCodes,
	/// A queue of `ref_el` callbacks: (el_id, fun).
	ref_queue: Vec<(u64, RefFn)>,
}

impl Debug for BuildState {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("BuildState")
			.field("id", &self.id)
			.field("slab", &self.slab)
			.field("base_el", &self.base_el)
			.field("build_codes", &self.build_codes)
			.field("ref_queue", &self.ref_queue.iter().map(|v| v.0).collect::<Vec<_>>())
			.finish()
	}
}

/// A chunk under construction.
///
/// The `ChunkBuild` is the interface and builder used while constructing a chunk.
///
/// It borrows the [`DomContext`], targets a base [`Element`] and a specific [scope](Store#slab-management), and exposes the [`Store`] through [`StoreProv`].
///
/// The UI can be appended to in a tree-like manner using the [`chunk`](crate::chunk!) macro and the [`apply`](crate::apply) module.
///
/// The UI definition gets recorded in a buffer and is built in one shot at the end by calling the [`build`](ChunkBuild::build) function.
///
/// Multiple chunks can target the same base [`Element`] because the `ChunkBuild` simply appends its UI to it.
///
/// After a chunk is built, the constructed UI can be altered in whatever way because the bindings target the specific [`Element`]s directly.
///
/// # Example
/// ```
/// let mut build = root_build.ctx().new_chunk_tagged("div");
///
/// let count = build.prop(0);
/// chunk!(build, div(id: "section", style.color: "red") {
///     "hello world",
///     for i in 0..10 {
///         "item ", i, br()
///     }
///     button(on.click: (move |ctx, _| ctx.update(count, |v| *v += 1))) { "count: ", count }
/// });
///
/// build.apply(div((id("section"), style("color", "red"),
///     text("hello world"),
///     move |build: &mut ChunkBuild| for i in 0..10 {
///         build.apply((text(format!("item {i}")), br(())));
///     },
///     button((
///         on("click", move |ctx, _| ctx.update(count, |v| *v += 1)),
///         text("count: "), text(count),
///     )),
/// )));
///
/// let el = build.build();
/// chunk!(root_build, el);
/// ```
#[derive(Debug)]
pub struct ChunkBuild<'ctx> {
	/// The context.
	pub(crate) ctx: &'ctx mut DomContext,
	#[doc(hidden)]
	pub state: BuildState,
}
type RefFn = Box<dyn FnOnce(&mut DomContext, &Element)>;
impl<'ctx> ChunkBuild<'ctx> {
	/// Creates a new [`ChunkBuild`].
	pub(crate) fn new(
		ctx: &'ctx mut DomContext, id: ChunkId, slab: Option<SlabId>, base_el: Element,
	) -> Self {
		let state =
			BuildState { slab, base_el, id, build_codes: BuildCodes::new(), ref_queue: Vec::new() };
		Self { ctx, state }
	}
	/// Returns the base [`Element`] of the chunk.
	pub fn base_el(&self) -> Element {
		self.state.base_el.clone()
	}

	/// Applies the [`Applicable`] to the current element.
	///
	/// See the [`apply`](crate::apply) module for more information.
	///
	/// # Example
	/// ```
	/// let count = build.prop(0);
	/// build.apply(div((id("section"), style("color", "red"),
	///     text("hello world"),
	///     move |build: &mut ChunkBuild| for i in 0..10 {
	///         build.apply((text(format!("item {i}")), br(())));
	///     },
	///     button((
	///         on("click", move |ctx, _| ctx.update(count, |v| *v += 1)),
	///         text("count: "), text(count),
	///     )),
	/// )));
	pub fn apply(&mut self, what: impl Applicable) {
		what.apply(self);
	}

	/// Gets a reference to the current element through a callback.
	///
	/// The callback will be called after the chunk is built but before the [`Element`] is returned.
	///
	/// # Example
	/// ```
	/// build.ref_el(|ctx, el| println!("{}", el.text_content().unwrap()));
	/// ```
	pub fn ref_el(&mut self, fun: impl FnOnce(&mut DomContext, &Element) + 'static) {
		self.state.ref_queue.push((self.state.build_codes.request_id(), Box::new(fun)));
	}

	/// Pause the chunk construction for later continuation.
	///
	/// this method returns the [`DomContext`] and the chunk state inside a [`DormantChunk`] which can be reactivated at any time using [`DormantChunk::wake`].
	///
	/// # Example
	/// ```
	/// chunk!(build, "initial section");
	/// let (ctx, chunk) = build.hibernate();
	/// let ctx_id = ctx.id();
	/// fetch_content().then(move |content| use_ctx(ctx_id, move |ctx| {
	///     let mut build = chunk.wake(ctx);
	///     chunk!(build, content);
	///     build.build();
	/// }));
	/// ```
	pub fn hibernate(self) -> (&'ctx mut DomContext, DormantChunk) {
		(self.ctx, DormantChunk(self.state))
	}

	/// Builds the chunk.
	///
	/// The chunk is built in one shot and appended to the base [`Element`] which gets returned.
	///
	/// # Example
	/// ```
	/// let mut build = root_build.ctx().new_chunk_tagged("div");
	/// chunk!(build, div { "hello world" });
	/// build.build();
	/// chunk!(root_build, el);
	/// ```
	pub fn build(self) -> Element {
		let BuildState { id, base_el, build_codes, ref_queue, .. } = self.state;
		let elements = build_codes.construct(&base_el);
		for (id, fun) in ref_queue {
			fun(self.ctx, &elements[id as usize])
		}
		self.ctx.chunks[id].elements = elements;
		base_el
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
	/// Returns the [`SlabId`] of the chunk.
	fn slab(&self) -> Option<SlabId> {
		self.state.slab
	}
}

/// A chunk that can be removed.
///
/// `RemovableChunk` is a [`ChunkBuild`] that has its own scope and can be removed when needed.
///
/// It implements [`Deref`] to [`ChunkBuild`] so all the functionality of [`ChunkBuild`] can be used.
///
/// The chunk does not get removed if it is dropped or if its element is removed. An explicit call to [`remove`](ChunkRemover::remove) is required.
///
/// # Example
/// ```
/// let mut build = root_build.ctx().removable_chunk("div");
/// chunk!(build, "hello world");
/// let (el, remover) = build.build();
/// let mut remover = Some(remover);
/// chunk!(root_build, el,
///     button(on.click: (move |ctx, _| remover.take().unwrap().remove(ctx))) { "remove" }
/// );
/// ```
#[derive(Debug)]
pub struct RemovableChunk<'ctx>(ChunkBuild<'ctx>);
impl<'ctx> RemovableChunk<'ctx> {
	/// Creates a new [`RemovableChunk`].
	pub(crate) fn new(ctx: &'ctx mut DomContext, id: ChunkId, base_el: Element) -> Self {
		let slab = ctx.store().create_slab();
		Self(ChunkBuild::new(ctx, id, Some(slab), base_el))
	}

	/// Pause the chunk construction for later continuation.
	///
	/// this method returns the [`DomContext`] and the chunk state inside a [`DormantRemovableChunk`] which can be reactivated at any time using [`DormantRemovableChunk::wake`].
	///
	/// # Example
	/// ```
	/// chunk!(build, "initial section");
	/// let (ctx, chunk) = build.hibernate();
	/// let ctx_id = ctx.id();
	/// fetch_content().then(move |content| use_ctx(ctx_id, move |ctx| {
	///     let mut build = chunk.wake(ctx);
	///     chunk!(build, content);
	///     build.build();
	/// }));
	/// ```
	pub fn hibernate(self) -> (&'ctx mut DomContext, DormantRemovableChunk) {
		let (ctx, chunk) = self.0.hibernate();
		(ctx, DormantRemovableChunk(chunk))
	}

	/// Builds the chunk and exports it as an [`Applicable`].
	///
	/// It builds the chunk and then returns it as an [`Applicable`] that inserts the chunk into another chunk and handles removing the chunk when the parent chunk is removed.
	///
	/// # Example
	/// ```
	/// let mut build = root_build.ctx().removable_chunk("div");
	/// chunk!(build, "hello world");
	/// let chunk = build.export();
	/// chunk!(root_build, chunk);
	/// ```
	pub fn export(self) -> impl Applicable {
		let (el, remover) = self.build();
		move |build: &mut ChunkBuild| {
			build.state.build_codes.node(el.into());
			let slab = build.state.slab;
			build.store().add_cleaner(slab, move |ctx| remover.remove(ctx)).unwrap()
		}
	}

	/// Builds the chunk and returns the [`Element`] and [`ChunkRemover`].
	///
	/// # Example
	/// ```
	/// let mut build = root_build.ctx().removable_chunk("div");
	/// chunk!(build, "hello world");
	/// let (el, remover) = build.build();
	/// let mut remover = Some(remover);
	/// chunk!(root_build, el,
	///     button(on.click: (move |ctx, _| remover.take().unwrap().remove(ctx))) { "remove" }
	/// );
	/// ```
	pub fn build(self) -> (Element, ChunkRemover) {
		let id = self.0.state.id;
		let slab = self.0.state.slab.unwrap();
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

/// The remover of a [`RemovableChunk`].
///
/// It cannot be dropped. An explicit call to [`remove`](ChunkRemover::remove) is required.
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
	/// Removes the chunk with its [`Element`] and [slab](Store#slab-management).
	///
	/// This method must be called.
	///
	/// # Example
	/// ```
	/// let mut build = root_build.ctx().removable_chunk("div");
	/// chunk!(build, "hello world");
	/// let (el, remover) = build.build();
	/// let mut remover = Some(remover);
	/// chunk!(root_build, el,
	///     button(on.click: (move |ctx, _| remover.take().unwrap().remove(ctx))) { "remove" }
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

/// An Inactive [`ChunkBuild`].
///
/// it is created by [`ChunkBuild::hibernate`], and can be reactive using [`wake`](DormantChunk::wake).
///
/// # Example
/// ```
/// chunk!(build, "initial section");
/// let (ctx, chunk) = build.hibernate();
/// let ctx_id = ctx.id();
/// fetch_content().then(move |content| use_ctx(ctx_id, move |ctx| {
///     let mut build = chunk.wake(ctx);
///     chunk!(build, content);
///     build.build();
/// }));
/// ```
#[derive(Debug)]
pub struct DormantChunk(BuildState);
impl DormantChunk {
	/// Returns the base [`Element`] of the chunk.
	pub fn base_el(&self) -> Element {
		self.0.base_el.clone()
	}
	/// Returns the [`SlabId`] of the chunk.
	pub fn slab(&self) -> Option<SlabId> {
		self.0.slab
	}
	/// Reactivates the chunk.
	///
	/// it take the [`DomContext`] and returns a [`ChunkBuild`] with the exact state as the original.
	///
	/// ```
	/// chunk!(build, "initial section");
	/// let (ctx, chunk) = build.hibernate();
	/// let ctx_id = ctx.id();
	/// fetch_content().then(move |content| use_ctx(ctx_id, move |ctx| {
	///     let mut build = chunk.wake(ctx);
	///     chunk!(build, content);
	///     build.build();
	/// }));
	/// ```
	pub fn wake(self, ctx: &mut DomContext) -> ChunkBuild<'_> {
		ChunkBuild { ctx, state: self.0 }
	}
}

/// The [`DormantChunk`] version of [`RemovableChunk`].
#[derive(Debug)]
pub struct DormantRemovableChunk(DormantChunk);
impl DormantRemovableChunk {
	/// Reactivates the chunk.
	///
	/// it take the [`DomContext`] and returns a [`RemovableChunk`] with the exact state as the original.
	///
	/// ```
	/// chunk!(build, "initial section");
	/// let (ctx, chunk) = build.hibernate();
	/// let ctx_id = ctx.id();
	/// fetch_content().then(move |content| use_ctx(ctx_id, move |ctx| {
	///     let mut build = chunk.wake(ctx);
	///     chunk!(build, content);
	///     build.build();
	/// }));
	/// ```
	pub fn wake(self, ctx: &mut DomContext) -> RemovableChunk<'_> {
		RemovableChunk(self.0.wake(ctx))
	}
}
impl Deref for DormantRemovableChunk {
	type Target = DormantChunk;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for DormantRemovableChunk {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
