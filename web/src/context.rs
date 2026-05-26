use std::{
	cell::{Ref, RefCell, RefMut},
	rc::{Rc, Weak},
	sync::atomic::{AtomicU64, Ordering},
};

use neoview::{
	context::Context,
	prelude::{GlobalStoreProv, StoreProv},
	reactive::Store,
};
use rustc_hash::FxHashMap;
use web_sys::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextId(u64);
impl ContextId {
	fn next() -> Self {
		static COUNTER: AtomicU64 = AtomicU64::new(0);
		Self(COUNTER.fetch_add(1, Ordering::Relaxed))
	}
}

#[derive(Debug)]
pub struct DomContext {
	id: ContextId,
	root_el: Element,
	store: Store<Self>,
}
impl DomContext {
	pub fn root_el(&self) -> Element {
		self.root_el.clone()
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

pub fn new_ctx(root_el: Element) -> CtxHandle {
	CtxHandle::new(DomContext { id: ContextId::next(), root_el, store: Default::default() })
}
pub fn get_ctx(id: ContextId) -> Option<CtxHandle> {
	CTX_MAP.with_borrow(|map| map.get(&id).and_then(Weak::upgrade).map(|ctx| CtxHandle { ctx }))
}
