use crate::{PropId, SlabId, Store, store::EffectDeps};

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
	fn peek<T: 'static>(&self, id: PropId<T>) -> &T {
		self.store_ref().peek(id)
	}
	fn read<T: 'static>(&self, id: PropId<T>) -> &T {
		self.store_ref().read(id)
	}
	fn get<T: 'static + Copy>(&self, id: PropId<T>) -> T {
		self.store_ref().get(id)
	}
	fn read_mut<T: 'static>(&mut self, id: PropId<T>) -> &mut T {
		self.store().read_mut(id)
	}
	fn write<T: 'static>(&mut self, id: PropId<T>, value: T) -> T {
		self.store().write(id, value)
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
		Store::effect(self.ctx(), None, EffectDeps::Tracked, fun).unwrap();
	}
	fn effect_ext(&mut self, deps: EffectDeps, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		Store::effect(self.ctx(), None, deps, fun).unwrap();
	}
	fn computed<T: 'static>(
		&mut self, fun: impl FnMut(&mut Self::Ctx) -> T + 'static,
	) -> PropId<T> {
		Store::computed(self.ctx(), None, fun).unwrap()
	}
}

pub trait LocalStoreProv: StoreProv {
	fn slab(&self) -> SlabId;
	fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let slab = self.slab();
		self.store().prop_in(Some(slab), value).unwrap()
	}
	fn effect(&mut self, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		let slab = self.slab();
		Store::effect(self.ctx(), Some(slab), EffectDeps::Tracked, fun).unwrap();
	}
	fn effect_ext(&mut self, deps: EffectDeps, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		let slab = self.slab();
		Store::effect(self.ctx(), Some(slab), deps, fun).unwrap();
	}
	fn computed<T: 'static>(
		&mut self, fun: impl FnMut(&mut Self::Ctx) -> T + 'static,
	) -> PropId<T> {
		let slab = self.slab();
		Store::computed(self.ctx(), Some(slab), fun).unwrap()
	}
}

pub trait ScopedStoreProv: StoreProv {
	fn slab(&self) -> Option<SlabId>;
	fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let slab = self.slab();
		self.store().prop_in(slab, value).unwrap()
	}
	fn effect(&mut self, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		let slab = self.slab();
		Store::effect(self.ctx(), slab, EffectDeps::Tracked, fun).unwrap();
	}
	fn effect_ext(&mut self, deps: EffectDeps, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		let slab = self.slab();
		Store::effect(self.ctx(), slab, deps, fun).unwrap();
	}
	fn computed<T: 'static>(
		&mut self, fun: impl FnMut(&mut Self::Ctx) -> T + 'static,
	) -> PropId<T> {
		let slab = self.slab();
		Store::computed(self.ctx(), slab, fun).unwrap()
	}
}
