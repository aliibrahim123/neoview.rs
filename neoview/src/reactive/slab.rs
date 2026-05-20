use std::{cell::RefMut, panic::Location};

use crate::reactive::{
	Error, PropId, SlabId, Store,
	prop::ItemId,
	signal::{ROSignal, Signal, WOSignal},
	struct_change_while_life_refs,
};

pub struct Slab<'store> {
	pub(crate) store: &'store Store,
	pub(crate) id: SlabId,
}
impl<'store> Slab<'store> {
	pub fn store(&self) -> &Store {
		self.store
	}
	pub fn id(&self) -> SlabId {
		self.id
	}
	fn slab(&self) -> RefMut<'_, SlabData> {
		let slabs = self.store.slabs.borrow_mut();
		RefMut::map(slabs, |slabs| slabs.get_mut(&self.id).unwrap())
	}
	pub fn add_prop<T: 'static>(&self, value: T) -> Result<PropId<T>, Error> {
		let id = self.store.add_prop(value)?;
		self.slab().props.push(id.0);
		Ok(id)
	}
	fn add_prop_panicing<T: 'static>(&self, value: T) -> PropId<T> {
		let Ok(id) = self.add_prop(value) else { struct_change_while_life_refs() };
		id
	}

	pub fn signal<T: 'static>(&self, value: T) -> Signal<'_, T> {
		let id = self.add_prop_panicing(value);
		Signal { store: self.store, prop: id }
	}
	pub fn ro_signal<T: 'static>(&self, value: T) -> ROSignal<'_, T> {
		let id = self.add_prop_panicing(value);
		ROSignal { store: self.store, prop: id }
	}
	pub fn wo_signal<T: 'static>(&self, value: T) -> WOSignal<'_, T> {
		let id = self.add_prop_panicing(value);
		WOSignal { store: self.store, prop: id }
	}

	#[track_caller]
	pub fn effect(&self, fun: impl FnMut() + 'store) {
		let mut updater = self.store.updater.borrow_mut();
		let id = updater.add_effect(self.store, fun, None, Location::caller());
		self.slab().effects.push(id);
	}
	#[track_caller]
	pub fn effect_manual(
		&self, read: Vec<PropId<()>>, write: Vec<PropId<()>>, fun: impl FnMut() + 'store,
	) {
		let mut updater = self.store.updater.borrow_mut();
		let id = updater.add_effect(self.store, fun, Some((read, write)), Location::caller());
		self.slab().effects.push(id);
	}

	#[track_caller]
	fn computed<'scope, T: 'static>(
		&'scope self, fun: impl FnMut() -> T + 'scope,
	) -> ROSignal<'scope, T> {
		self.store().computed_manual(fun, Location::caller())
	}
}

#[derive(Debug, Default)]
pub struct SlabData {
	pub props: Vec<ItemId>,
	pub effects: Vec<ItemId>,
}
