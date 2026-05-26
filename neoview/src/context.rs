use crate::reactive::{Error, PropId, SlabId, Store};

pub trait Context: Sized + GlobalStoreProv<Ctx = Self> {}

pub trait StoreProv {
	type Ctx: Context;
	fn ctx(&mut self) -> &mut Self::Ctx;
	fn ctx_ref(&self) -> &Self::Ctx;
	fn store(&mut self) -> &mut Store<Self::Ctx> {
		self.ctx().store()
	}
	fn store_ref(&self) -> &Store<Self::Ctx> {
		self.ctx_ref().store_ref()
	}

	fn try_peek<T: 'static>(&self, id: PropId<T>) -> Option<&T> {
		self.store_ref().try_peek(id)
	}
	fn peek<T: 'static>(&self, id: PropId<T>) -> &T {
		self.store_ref().peek(id)
	}
	fn try_read<T: 'static>(&self, id: PropId<T>) -> Option<&T> {
		self.store_ref().try_read(id)
	}
	fn read<T: 'static>(&self, id: PropId<T>) -> &T {
		self.store_ref().read(id)
	}
	fn try_get<T: 'static + Copy>(&self, id: PropId<T>) -> Option<T> {
		self.store_ref().try_get(id)
	}
	fn get<T: 'static + Copy>(&self, id: PropId<T>) -> T {
		self.store_ref().get(id)
	}
	fn try_read_mut<T: 'static>(&mut self, id: PropId<T>) -> Option<&mut T> {
		self.store().try_read_mut(id)
	}
	fn read_mut<T: 'static>(&mut self, id: PropId<T>) -> &mut T {
		self.store().read_mut(id)
	}
	fn try_write<T: 'static>(&mut self, id: PropId<T>, value: T) -> Result<(), Error> {
		self.store().try_write(id, value)
	}
	fn write<T: 'static>(&mut self, id: PropId<T>, value: T) {
		self.store().write(id, value)
	}
	fn try_update<T: 'static>(
		&mut self, id: PropId<T>, fun: impl FnOnce(&mut T),
	) -> Result<(), Error> {
		self.store().try_update(id, fun)
	}
	fn update<T: 'static>(&mut self, id: PropId<T>, fun: impl FnOnce(&mut T)) {
		self.store().update(id, fun)
	}
}

pub trait GlobalStoreProv: StoreProv {
	fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		self.store().prop(value)
	}
	fn effect(&mut self, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		Store::effect(self.ctx(), fun);
	}
	fn effect_manual(
		&mut self, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Self::Ctx) + 'static,
	) {
		Store::effect_manual(self.ctx(), read, write, fun);
	}
	fn computed<T: 'static>(
		&mut self, fun: impl FnMut(&mut Self::Ctx) -> T + 'static,
	) -> PropId<T> {
		Store::computed(self.ctx(), fun)
	}
}

pub trait LocalStoreProv: StoreProv {
	fn slab(&self) -> SlabId;
	fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let slab = self.slab();
		self.store().prop_in(slab, value).unwrap()
	}
	fn effect(&mut self, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		let slab = self.slab();
		Store::effect_in(self.ctx(), slab, fun).unwrap();
	}
	fn effect_manual(
		&mut self, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Self::Ctx) + 'static,
	) {
		let slab = self.slab();
		Store::effect_manual_in(self.ctx(), slab, read, write, fun).unwrap();
	}
	fn computed<T: 'static>(
		&mut self, fun: impl FnMut(&mut Self::Ctx) -> T + 'static,
	) -> PropId<T> {
		let slab = self.slab();
		Store::computed_in(self.ctx(), slab, fun).unwrap()
	}
}

pub trait ScopedStoreProv: StoreProv {
	fn slab(&self) -> Option<SlabId>;
	fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		match self.slab() {
			Some(slab) => self.store().prop_in(slab, value).unwrap(),
			None => self.store().prop(value),
		}
	}
	fn effect(&mut self, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		match self.slab() {
			Some(slab) => Store::effect_in(self.ctx(), slab, fun).unwrap(),
			None => Store::effect(self.ctx(), fun),
		}
	}
	fn effect_manual(
		&mut self, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Self::Ctx) + 'static,
	) {
		match self.slab() {
			Some(slab) => Store::effect_manual_in(self.ctx(), slab, read, write, fun).unwrap(),
			None => Store::effect_manual(self.ctx(), read, write, fun),
		}
	}
	fn computed<T: 'static>(
		&mut self, fun: impl FnMut(&mut Self::Ctx) -> T + 'static,
	) -> PropId<T> {
		match self.slab() {
			Some(slab) => Store::computed_in(self.ctx(), slab, fun).unwrap(),
			None => Store::computed(self.ctx(), fun),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct VoidContext {
	store: Store<Self>,
}
impl StoreProv for VoidContext {
	type Ctx = Self;
	fn ctx(&mut self) -> &mut Self::Ctx {
		self
	}
	fn ctx_ref(&self) -> &Self::Ctx {
		self
	}
	fn store(&mut self) -> &mut Store<Self::Ctx> {
		&mut self.store
	}
	fn store_ref(&self) -> &Store<Self::Ctx> {
		&self.store
	}
}
impl GlobalStoreProv for VoidContext {}
impl Context for VoidContext {}
