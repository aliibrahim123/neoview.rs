use crate::reactive::{Error, PropId, Store};

pub trait Context: Sized {
	fn store(&mut self) -> &mut Store<Self>;
	fn store_ref(&self) -> &Store<Self>;
}
pub trait ContextStoreExt: Context {
	fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		self.store().prop(value)
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
	fn effect(&mut self, fun: impl FnMut(&mut Self) + 'static) {
		Store::effect(self, fun);
	}
	fn effect_manual(
		&mut self, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Self) + 'static,
	) {
		Store::effect_manual(self, read, write, fun);
	}
	fn computed<T: 'static>(&mut self, fun: impl FnMut(&mut Self) -> T + 'static) -> PropId<T> {
		Store::computed(self, fun)
	}
}
impl<Ctx: Context> ContextStoreExt for Ctx {}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct VoidContext {
	store: Store<Self>,
}
impl Context for VoidContext {
	fn store(&mut self) -> &mut Store<Self> {
		&mut self.store
	}
	fn store_ref(&self) -> &Store<Self> {
		&self.store
	}
}
