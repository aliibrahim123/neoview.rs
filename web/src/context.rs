//! Defines and manages [`DomContext`].

use core::task;
use std::{
	cell::{RefCell, RefMut},
	collections::VecDeque,
	ops::{Deref, DerefMut},
	pin::Pin,
	rc::{Rc, Weak},
	sync::atomic::{AtomicU64, Ordering},
	task::Poll,
};

use neoview::{Context, Error, GlobalStoreProv, Store, StoreProv};
use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use web_sys::{Element, window};

use crate::chunk::{ChunkBuild, ChunkData, ChunkId, RemovableChunk};

/// A unique identifier for a [`DomContext`].
///
/// This is used when retrieving a [`DomContext`] using [`use_ctx`].
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
/// It is created by [`new_ctx`], wraps a root [`Element`], and exposes its [`Store`] through [`StoreProv`].
/// ```
/// let handle = new_ctx(root_el, CtxOptions::default());
/// let ctx = handle.borrow_mut();
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
	pub(crate) chunks: SlotMap<ChunkId, ChunkData>,
}

/// Creates a new [`DomContext`].
///
/// This function creates a new [`DomContext`] wrapping a given root [`Element`] and taking [`CtxOptions`], and it returns a [`CtxHandle`] to it.
///
/// The root element can be in the DOM tree or outside it. It can be an HTML element, an SVG element, or any other element.
///
/// # Example
/// ```
/// let el = windows().unwrap().document().unwrap().create_element("div").unwrap();
/// let handle = new_ctx(el, CtxOptions::default());
/// let ctx = handle.borrow().unwrap();
/// ```
pub fn new_ctx(root_el: Element, opts: CtxOptions) -> CtxHandle {
	let ctx = DomContext {
		id: ContextId::next(),
		options: opts,
		root_el,
		store: Store::default(),
		chunks: SlotMap::default(),
	};
	CtxHandle::new(ctx)
}

impl DomContext {
	/// Returns the [`ContextId`] of this `DomContext`.
	pub fn id(&self) -> ContextId {
		self.id
	}
	/// Returns the root [`Element`] of this `DomContext`.
	pub fn root_el(&self) -> Element {
		self.root_el.clone()
	}

	/// create a new [`CtxHandle`] from this `DomContext`.
	pub fn new_handle(&self) -> CtxHandle {
		CTX_MAP.with_borrow(|map| {
			let ctx = &map.get(&self.id).unwrap().box_;
			CtxHandle { id: self.id, ctx: Weak::upgrade(ctx).unwrap() }
		})
	}

	/// Creates a new [`Chunk`] and returns its [`ChunkId`].
	fn new_chunk_id(&mut self) -> ChunkId {
		self.chunks.insert(ChunkData::default())
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

	/// Creates a [`ChunkBuild`] targeting a new [`Element`] of a given `tag`.
	///
	/// This is a shorthand for [`new_chunk(document.create_element(tag))`](DomContext::new_chunk).
	///
	/// # Example
	/// ```
	/// let mut build = root_build.ctx().new_chunk_tagged("div");
	/// chunk!(build, "hello world");
	/// build.build();
	/// chunk!(root_build, el);
	/// ```
	pub fn new_chunk_tagged(&mut self, tag: &str) -> ChunkBuild<'_> {
		let id = self.new_chunk_id();
		let el = window().unwrap().document().unwrap().create_element(tag).unwrap();
		ChunkBuild::new(self, id, None, el)
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

/// The function of [`use_ctx`]
type UseRequest = Box<dyn FnOnce(&mut DomContext)>;

/// [`CTX_MAP`] value
struct CtxUnit {
	box_: Weak<RefCell<DomContext>>,
	requests: VecDeque<UseRequest>,
}

thread_local!(
	/// A weak map storing [`DomContext`]s.
	static CTX_MAP: RefCell<FxHashMap<ContextId, CtxUnit>> = Default::default();
);

/// A handle to a [`DomContext`].
///
/// Since events require access to the [`DomContext`], the [`DomContext`] cannot be stored directly by value. A handle is provided instead.
///
/// It provide several methods to access the [`DomContext`]: a primitive [`borrow`](CtxHandle::borrow), a defered [`use_ctx`](CtxHandle::use_ctx), and async oriented [`acquire`](CtxHandle::acquire).
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
		CTX_MAP.with_borrow_mut(|map| {
			map.insert(id, CtxUnit { box_: weak, requests: VecDeque::new() })
		});
		Self { id, ctx }
	}
	/// Returns the [`ContextId`] of the [`DomContext`].
	pub fn id(&self) -> ContextId {
		self.id
	}
	/// Returns a mutable reference to the [`DomContext`].
	///
	/// It return a gaurd that flushes updates when dropped. And returns `None` if the [`DomContext`] has active mutation.
	///
	/// # Example
	/// ```
	/// let handle = new_ctx(root_el, CtxOptions::default());
	/// let ctx = handle.borrow_mut().unwrap();
	/// // ...
	/// ```
	pub fn borrow(&self) -> Option<impl DerefMut<Target = DomContext>> {
		self.ctx.try_borrow_mut().ok().map(ContextRef)
	}

	/// Use the [`DomContext`] when it is available.
	///
	/// It call the provided function directly if the [`DomContext`] is available (has no active mutation), otherwise it defer it until the [`DomContext`] become available, with updates flushing.
	///
	/// this is a version of [`use_ctx`] for cases that need to own the [`CtxHandle`].
	///
	/// # Example
	/// ```
	/// let time = build.prop(0);
	/// chunk!(build, "time: ", count);
	/// set_interval(move || handle.use_ctx(move |ctx| ctx.update(time, |v| *v += 1)), 1000);
	/// ```
	pub fn use_ctx(&self, fun: impl FnOnce(&mut DomContext) + 'static) {
		match self.borrow() {
			Some(mut ctx) => fun(&mut ctx),
			None => CTX_MAP.with_borrow_mut(|map| {
				map.get_mut(&self.id).unwrap().requests.push_back(Box::new(fun));
			}),
		}
	}

	/// Acquire the [`DomContext`] in the async style.
	///
	/// it return a [`Future`] that resolve to a gaurd providing mutable access to the [`DomContext`] when it is available (has no active mutation), while also flushing updates.
	///
	/// # Example
	/// ```
	/// spawn_local(async move {
	///    while let Some(content) = some_stream.next().await {
	///         let mut ctx = handle.acquire().await;
	///         let mut build = ctx.new_chunk(el);
	///         chunk!(build, content);
	///         build.build();
	///     }
	/// })
	/// ```
	pub fn acquire(&self) -> impl Future<Output = impl DerefMut<Target = DomContext>> {
		pub struct Task<'ctx> {
			handle: &'ctx CtxHandle,
		}
		impl<'ctx> Future for Task<'ctx> {
			type Output = ContextRef<'ctx>;
			fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
				match self.handle.ctx.try_borrow_mut() {
					Ok(ctx) => Poll::Ready(ContextRef(ctx)),
					Err(_) => {
						let waker = cx.waker().clone();
						CTX_MAP.with_borrow_mut(|map| {
							let requests = &mut map.get_mut(&self.handle.id).unwrap().requests;
							requests.push_back(Box::new(move |_| waker.wake_by_ref()));
						});
						Poll::Pending
					}
				}
			}
		}
		Task { handle: self }
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

/// A gaurd around [`DomContext`] access.
///
/// it flushs updates and run defered uses when dropped.
#[derive(Debug)]
pub struct ContextRef<'a>(RefMut<'a, DomContext>);
impl Deref for ContextRef<'_> {
	type Target = DomContext;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for ContextRef<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
impl Drop for ContextRef<'_> {
	fn drop(&mut self) {
		Store::flush_updates(&mut *self.0);
		let ctx_id = self.0.id;
		while let Some(fun) =
			CTX_MAP.with_borrow_mut(|map| map.get_mut(&ctx_id).unwrap().requests.pop_front())
		{
			fun(&mut self.0);
			Store::flush_updates(&mut *self.0);
		}
	}
}

/// Use a [`DomContext`] when it is available.
///
/// it take the [`DomContext`] [`ContextId`], then call the provided function if the [`DomContext`] is available (has no active mutation), otherwise it defer it until the [`DomContext`] become available.
///
/// this function flushes updates, and return [`Error::Removed`] if the [`DomContext`] was dropped previously.
///
/// this is a version of [`CtxHandle::use_ctx`] for cases that doesnt need to own the [`CtxHandle`].
///
/// # Example
/// ```
/// set_interval(move |id| {
///     let res = use_ctx(ctx_id, move |ctx| ctx.update(time, |v| *v += 1));
///     if let Err(err) = res {
///         clear_interval(id);
///     }
/// }, 1000);
/// ```
pub fn use_ctx(id: ContextId, fun: impl FnOnce(&mut DomContext) + 'static) -> Result<(), Error> {
	let res = CTX_MAP.with_borrow_mut(|map| {
		let unit = map.get_mut(&id).ok_or(Error::Removed)?;
		let ctx = unit.box_.upgrade().unwrap();
		if ctx.try_borrow_mut().is_err() {
			unit.requests.push_back(Box::new(fun));
			Ok(None)
		} else {
			Ok(Some((ctx, fun)))
		}
	})?;
	// call the function outside `CTX_MAP` borrow to avoid deadlock
	if let Some((ctx, fun)) = res {
		let mut ctx = ctx.borrow_mut();
		fun(&mut ctx);
		Store::flush_updates(&mut *ctx);
	}
	Ok(())
}
