use std::{cell::RefMut, fmt::Debug, marker::PhantomData, panic::Location};

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
impl Debug for Slab<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Slab").field("id", &self.id).finish()
	}
}
impl<'store> Slab<'store> {
	pub(crate) fn new(store: &'store Store, id: SlabId) -> Self {
		Self { store, id }
	}
	pub fn store(&self) -> &'store Store {
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

	pub fn signal<T: 'static>(&self, value: T) -> Signal<'store, T> {
		let id = self.add_prop_panicing(value);
		Signal { store: self.store, prop: id }
	}
	pub fn ro_signal<T: 'static>(&self, value: T) -> ROSignal<'store, T> {
		let id = self.add_prop_panicing(value);
		ROSignal { store: self.store, prop: id }
	}
	pub fn wo_signal<T: 'static>(&self, value: T) -> WOSignal<'store, T> {
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
	fn computed<T: 'static>(&self, fun: impl FnMut() -> T + 'store) -> ROSignal<'store, T> {
		self.store.computed_manual(fun, Location::caller())
	}
}

#[derive(Debug, Default)]
pub struct SlabData {
	pub props: Vec<ItemId>,
	pub effects: Vec<ItemId>,
}
