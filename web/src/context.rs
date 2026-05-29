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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextId(pub(crate) u64);
impl ContextId {
	fn next() -> Self {
		static COUNTER: AtomicU64 = AtomicU64::new(0);
		Self(COUNTER.fetch_add(1, Ordering::Relaxed))
	}
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtxOptions {
	pub remove_on_drop: bool,
}
impl Default for CtxOptions {
	fn default() -> Self {
		Self { remove_on_drop: true }
	}
}

#[derive(Debug)]
pub struct DomContext {
	pub(crate) id: ContextId,
	options: CtxOptions,
	root_el: Element,
	store: Store<Self>,
	pub(crate) chunks: SlotMap<ChunkId, Chunk>,
}
impl DomContext {
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
	pub fn root_el(&self) -> Element {
		self.root_el.clone()
	}
	fn new_chunk_id(&mut self) -> ChunkId {
		self.chunks.insert(Chunk::default())
	}
	pub fn root_chunk(&mut self) -> ChunkBuild<'_> {
		let id = self.new_chunk_id();
		ChunkBuild::new(self, id, None, self.root_el.clone())
	}
	pub fn new_chunk(&mut self, base_el: Element) -> ChunkBuild<'_> {
		let id = self.new_chunk_id();
		ChunkBuild::new(self, id, None, base_el)
	}
	pub fn removable_chunk(&mut self, tag: &str) -> RemovableChunk<'_> {
		let id = self.new_chunk_id();
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
		if self.options.remove_on_drop {
			self.root_el.remove();
		}
	}
}

thread_local!(
	static CTX_MAP: RefCell<FxHashMap<ContextId, Weak<RefCell<DomContext>>>> = Default::default();
);

#[derive(Debug, Clone)]
pub struct CtxHandle {
	ctx: Rc<RefCell<DomContext>>,
}
impl CtxHandle {
	fn new(ctx: DomContext) -> Self {
		let id = ctx.id;
		let ctx = Rc::new(RefCell::new(ctx));
		let weak = Rc::downgrade(&ctx);
		CTX_MAP.with_borrow_mut(|map| map.insert(id, weak));
		Self { ctx }
	}
	pub fn borrow(&self) -> Ref<'_, DomContext> {
		self.ctx.borrow()
	}
	pub fn borrow_mut(&self) -> RefMut<'_, DomContext> {
		self.ctx.borrow_mut()
	}
}
impl Drop for CtxHandle {
	fn drop(&mut self) {
		if Rc::strong_count(&self.ctx) == 1 {
			let id = self.ctx.borrow().id;
			CTX_MAP.with_borrow_mut(|map| map.remove(&id));
		}
	}
}
pub fn get_ctx(id: ContextId) -> Option<CtxHandle> {
	CTX_MAP.with_borrow(|map| map.get(&id).and_then(Weak::upgrade).map(|ctx| CtxHandle { ctx }))
}
