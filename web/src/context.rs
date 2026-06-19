//! defines and manages [`DomContext`]

use std::{
	cell::{Ref, RefCell, RefMut},
	rc::{Rc, Weak},
	sync::atomic::{AtomicU64, Ordering},
};

use neoview::{Context, GlobalStoreProv, Store, StoreProv};
use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use web_sys::{Element, window};

use crate::chunk::{Chunk, ChunkBuild, ChunkId, RemovableChunk};

/// A unique identifier for a [`DomContext`].
///
/// this is is used when retrieving a [`DomContext`] using [`get_ctx`].
/// ```
/// let id = ctx.id();
/// // some time
/// let ctx = get_ctx(id).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextId(pub(crate) u64);
impl ContextId {
	/// get the value of the [`ContextId`].
	pub fn value(&self) -> u64 {
		self.0
	}
	/// return a new [`ContextId`].
	fn next() -> Self {
		static COUNTER: AtomicU64 = AtomicU64::new(0);
		Self(COUNTER.fetch_add(1, Ordering::Relaxed))
	}
}

/// options for a [`DomContext`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtxOptions {
	/// whether to remove the root element when the [`DomContext`] is dropped, default: `true`.
	pub remove_el_on_drop: bool,
}
impl Default for CtxOptions {
	fn default() -> Self {
		Self { remove_el_on_drop: true }
	}
}

/// the [`Context`] of the `neoview-web` renderer.
///
/// this type is the single owner of the ui, every interaction requires a mutable reference to the it, and the ui is dropped when the `DomContext` is dropped.
///
/// it is created by [`new`](Self::new), wraps a root [`Element`] and exposes its [`Store`] through [`StoreProv`].
/// ```
/// let handle = DomContext::new(root_el, CtxOptions::default());
/// let ctx = handle.borrow_mut();
/// ```
///
/// it is identified by a [`ContextId`] that is used when retrieving it using [`get_ctx`].
/// ```
/// let id = ctx.id();
/// // some time
/// let ctx = get_ctx(id).unwrap();
/// ```
///
/// `DomContext` can not be stored directly by value, instead it is owned in a [`CtxHandle`].
#[derive(Debug)]
pub struct DomContext {
	/// the [`ContextId`] of the `DomContext`.
	pub(crate) id: ContextId,
	/// options for the `DomContext`.
	options: CtxOptions,
	/// the root element of the `DomContext`.
	root_el: Element,
	/// the store of the `DomContext`.
	store: Store<Self>,
	/// the chunks of the `DomContext`.
	pub(crate) chunks: SlotMap<ChunkId, Chunk>,
}
impl DomContext {
	/// creates a new `DomContext`.
	///
	/// this function creates a new `DomContext` wrapping a given root [`Element`] and taking a [`CtxOptions`], and returns a [`CtxHandle`] to it.
	///
	/// the root element can be in the DOM tree or outside it, and it can be an html element, svg one or any other element.
	///
	/// # Example
	/// ```
	/// let el = windows().unwrap().document().unwrap().create_element("div").unwrap();
	/// let handle = DomContext::new(el, CtxOptions::default());
	/// let ctx = handle.borrow_mut();
	/// ```
	pub fn new(root_el: Element, opts: CtxOptions) -> CtxHandle {
		let ctx = DomContext {
			id: ContextId::next(),
			options: opts,
			root_el,
			store: Store::default(),
			chunks: SlotMap::default(),
		};
		CtxHandle::new(ctx)
	}

	/// returns the [`ContextId`] of this `DomContext`.
	pub fn id(&self) -> ContextId {
		self.id
	}
	/// returns the root [`Element`] of this `DomContext`.
	pub fn root_el(&self) -> Element {
		self.root_el.clone()
	}
	/// creates a new [`Chunk`] and returns its [`ChunkId`].
	fn new_chunk_id(&mut self) -> ChunkId {
		self.chunks.insert(Chunk::default())
	}

	/// creates a [`ChunkBuild`] targeting the root [`Element`].
	///
	/// the scope of the [`ChunkBuild`] is the global scope.
	///
	/// while one is enough, multiple root [`ChunkBuild`]s can be build and each one append to the root element.
	///
	/// # example
	/// ```
	/// let mut build = ctx.root_chunk();
	/// chunk!(build, div { "hello world" });
	/// build.build();
	/// ```
	pub fn root_chunk(&mut self) -> ChunkBuild<'_> {
		let id = self.new_chunk_id();
		ChunkBuild::new(self, id, None, self.root_el.clone())
	}

	/// creates a [`ChunkBuild`] targeting the base [`Element`].
	///
	/// the scope of the [`ChunkBuild`] is the global scope.
	///
	/// # example
	/// ```
	/// let el = window().unwrap().document().unwrap().create_element("div").unwrap();
	/// let mut build = root_build.ctx().new_chunk(el);
	/// chunk!(build, "hello world");
	/// build.build();
	/// chunk!(root_build, el);
	/// ```
	pub fn new_chunk(&mut self, base_el: Element) -> ChunkBuild<'_> {
		let id = self.new_chunk_id();
		ChunkBuild::new(self, id, None, base_el)
	}

	/// creates a [`RemovableChunk`] targeting a new [`Element`] of a given `tag`.
	///
	/// # example
	/// ```
	/// let mut build = root_build.ctx().removable_chunk("div");
	/// chunk!(build, "hello world");
	/// let (el, remover) = build.build();
	/// let mut remover = Some(remover);
	/// chunk!(root_build, el,
	///     button(on.click: (move |ctx, _| remover.take().unwrap().remove(ctx))) { "remove" }
	/// );
	/// ```
	pub fn removable_chunk(&mut self, tag: &str) -> RemovableChunk<'_> {
		let id = self.new_chunk_id();
		// unwrap hell i know, what can i do
		let el = window().unwrap().document().unwrap().create_element(tag).unwrap();
		RemovableChunk::new(self, id, el)
	}
}
impl Context for DomContext {}
impl StoreProv for DomContext {
	type Ctx = Self;
	fn store(&mut self) -> &mut Store<Self> {
		&mut self.store
	}
	fn store_ref(&self) -> &Store<Self> {
		&self.store
	}
	fn ctx(&mut self) -> &mut Self {
		self
	}
	fn ctx_ref(&self) -> &Self {
		self
	}
}
impl GlobalStoreProv for DomContext {}
impl Drop for DomContext {
	fn drop(&mut self) {
		Store::pre_drop(self);
		if self.options.remove_el_on_drop {
			self.root_el.remove();
		}
	}
}

thread_local!(
	/// a weak map storing [`DomContext`].
	static CTX_MAP: RefCell<FxHashMap<ContextId, Weak<RefCell<DomContext>>>> = Default::default();
);

/// a handle to a [`DomContext`].
///
/// since events requires an access to the [`DomContext`], [`DomContext`] can not be stored directly by value, so a handle is provided instead.
///
/// the [`DomContext`] is dropped only when all handles to it are dropped.
#[derive(Debug, Clone)]
pub struct CtxHandle {
	/// the id of the [`DomContext`].
	id: ContextId,
	/// the [`DomContext`] box.
	ctx: Rc<RefCell<DomContext>>,
}
impl CtxHandle {
	/// integrate a [`DomContext`] into the content map and returns a [`CtxHandle`] to it.
	fn new(ctx: DomContext) -> Self {
		let id = ctx.id;
		let ctx = Rc::new(RefCell::new(ctx));
		let weak = Rc::downgrade(&ctx);
		CTX_MAP.with_borrow_mut(|map| map.insert(id, weak));
		Self { id, ctx }
	}
	/// returns the [`ContextId`] of the [`DomContext`].
	pub fn id(&self) -> ContextId {
		self.id
	}
	/// returns a reference to the [`DomContext`].
	pub fn borrow(&self) -> Ref<'_, DomContext> {
		self.ctx.borrow()
	}
	/// returns a mutable reference to the [`DomContext`].
	pub fn borrow_mut(&self) -> RefMut<'_, DomContext> {
		self.ctx.borrow_mut()
	}
}
impl Drop for CtxHandle {
	fn drop(&mut self) {
		// comparing to one since the `ctx` field is not yet dropped
		if Rc::strong_count(&self.ctx) == 1 {
			let id = self.ctx.borrow().id;
			CTX_MAP.with_borrow_mut(|map| map.remove(&id));
		}
	}
}

/// returns a [`CtxHandle`] to the [`DomContext`] of the given [`ContextId`], if it exists.
///
/// this function is only used when handling events, passing context directly is the 99.99% option.
///
/// returns `None` if the [`DomContext`] was dropped before.
///
/// # example
/// ```
/// fn on_event(event: Event) {
///     let ctx = get_ctx(id).unwrap();
///     let mut ctx = ctx.borrow_mut();
///     // ...
///     Store::flush_updates(ctx);
/// }
/// ```
pub fn get_ctx(id: ContextId) -> Option<CtxHandle> {
	CTX_MAP.with_borrow(|map| {
		let ctx = map.get(&id)?;
		Some(CtxHandle { id, ctx: Weak::upgrade(ctx)? })
	})
}

/// call a function with a mutable reference to a [`DomContext`], if it exists.
///
/// this function is a convenient wrapper for [`get_ctx`] that also flushes updates.
///
/// it returns `Err(())` if the [`DomContext`] was dropped before.
///
/// # example
/// ```
/// fn on_event(event: Event) {
///     use_ctx(id, |ctx| {
///         // ...
///     }).unwrap();
/// }
/// ```
pub fn use_ctx(id: ContextId, fun: impl FnOnce(&mut DomContext)) -> Result<(), ()> {
	let ctx = get_ctx(id).ok_or(())?;
	let mut ctx = ctx.borrow_mut();
	fun(&mut ctx);
	Store::flush_updates(&mut *ctx);
	Ok(())
}
