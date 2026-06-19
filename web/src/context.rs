//! Defines and manages [`DomContext`].

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
/// This is used when retrieving a [`DomContext`] using [`get_ctx`].
/// ```
/// let id = ctx.id();
/// // some time
/// let ctx = get_ctx(id).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextId(pub(crate) u64);
impl ContextId {
	/// Gets the value of the [`ContextId`].
	pub fn value(&self) -> u64 {
		self.0
	}
	/// Returns a new [`ContextId`].
	fn next() -> Self {
		static COUNTER: AtomicU64 = AtomicU64::new(0);
		Self(COUNTER.fetch_add(1, Ordering::Relaxed))
	}
}

/// Options for a [`DomContext`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtxOptions {
	/// Whether to remove the root element when the [`DomContext`] is dropped. The default is `true`.
	pub remove_el_on_drop: bool,
}
impl Default for CtxOptions {
	fn default() -> Self {
		Self { remove_el_on_drop: true }
	}
}

/// The [`Context`] of the `neoview-web` renderer.
///
/// This type is the single owner of the UI. Every interaction requires a mutable reference to it, and the UI is dropped when the `DomContext` is dropped.
///
/// It is created by [`new`](Self::new), wraps a root [`Element`], and exposes its [`Store`] through [`StoreProv`].
/// ```
/// let handle = DomContext::new(root_el, CtxOptions::default());
/// let ctx = handle.borrow_mut();
/// ```
///
/// It is identified by a [`ContextId`] which is used when retrieving it using [`get_ctx`].
/// ```
/// let id = ctx.id();
/// // some time
/// let ctx = get_ctx(id).unwrap();
/// ```
///
/// `DomContext` cannot be stored directly by value. Instead it is owned in a [`CtxHandle`].
#[derive(Debug)]
pub struct DomContext {
	/// The [`ContextId`] of the `DomContext`.
	pub(crate) id: ContextId,
	/// Options for the `DomContext`.
	options: CtxOptions,
	/// The root element of the `DomContext`.
	root_el: Element,
	/// The store of the `DomContext`.
	store: Store<Self>,
	/// The chunks of the `DomContext`.
	pub(crate) chunks: SlotMap<ChunkId, Chunk>,
}
impl DomContext {
	/// Creates a new `DomContext`.
	///
	/// This function creates a new `DomContext` wrapping a given root [`Element`] and taking [`CtxOptions`], and it returns a [`CtxHandle`] to it.
	///
	/// The root element can be in the DOM tree or outside it. It can be an HTML element, an SVG element, or any other element.
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

	/// Returns the [`ContextId`] of this `DomContext`.
	pub fn id(&self) -> ContextId {
		self.id
	}
	/// Returns the root [`Element`] of this `DomContext`.
	pub fn root_el(&self) -> Element {
		self.root_el.clone()
	}
	/// Creates a new [`Chunk`] and returns its [`ChunkId`].
	fn new_chunk_id(&mut self) -> ChunkId {
		self.chunks.insert(Chunk::default())
	}

	/// Creates a [`ChunkBuild`] targeting the root [`Element`].
	///
	/// The scope of the [`ChunkBuild`] is the global scope.
	///
	/// While one is enough, multiple root [`ChunkBuild`]s can be built, and each one appends to the root element.
	///
	/// # Example
	/// ```
	/// let mut build = ctx.root_chunk();
	/// chunk!(build, div { "hello world" });
	/// build.build();
	/// ```
	pub fn root_chunk(&mut self) -> ChunkBuild<'_> {
		let id = self.new_chunk_id();
		ChunkBuild::new(self, id, None, self.root_el.clone())
	}

	/// Creates a [`ChunkBuild`] targeting the base [`Element`].
	///
	/// The scope of the [`ChunkBuild`] is the global scope.
	///
	/// # Example
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

	/// Creates a [`RemovableChunk`] targeting a new [`Element`] of a given `tag`.
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
	/// A weak map storing [`DomContext`]s.
	static CTX_MAP: RefCell<FxHashMap<ContextId, Weak<RefCell<DomContext>>>> = Default::default();
);

/// A handle to a [`DomContext`].
///
/// Since events require access to the [`DomContext`], the [`DomContext`] cannot be stored directly by value. A handle is provided instead.
///
/// The [`DomContext`] is dropped only when all handles to it are dropped.
#[derive(Debug, Clone)]
pub struct CtxHandle {
	/// The ID of the [`DomContext`].
	id: ContextId,
	/// The [`DomContext`] box.
	ctx: Rc<RefCell<DomContext>>,
}
impl CtxHandle {
	/// Integrates a [`DomContext`] into the content map and returns a [`CtxHandle`] to it.
	fn new(ctx: DomContext) -> Self {
		let id = ctx.id;
		let ctx = Rc::new(RefCell::new(ctx));
		let weak = Rc::downgrade(&ctx);
		CTX_MAP.with_borrow_mut(|map| map.insert(id, weak));
		Self { id, ctx }
	}
	/// Returns the [`ContextId`] of the [`DomContext`].
	pub fn id(&self) -> ContextId {
		self.id
	}
	/// Returns a reference to the [`DomContext`].
	pub fn borrow(&self) -> Ref<'_, DomContext> {
		self.ctx.borrow()
	}
	/// Returns a mutable reference to the [`DomContext`].
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

/// Returns a [`CtxHandle`] to the [`DomContext`] of the given [`ContextId`] if it exists.
///
/// This function is only used when handling events. Passing the context directly is the preferred approach in almost all cases.
///
/// Returns `None` if the [`DomContext`] was dropped previously.
///
/// # Example
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

/// Calls a function with a mutable reference to a [`DomContext`] if it exists.
///
/// This function is a convenient wrapper for [`get_ctx`] that also flushes updates.
///
/// It returns `Err(())` if the [`DomContext`] was dropped previously.
///
/// # Example
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
