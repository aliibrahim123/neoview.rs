use std::cell::RefMut;

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
		let slabs = self.store().slabs.borrow_mut();
		RefMut::map(slabs, |slabs| slabs.get_mut(&self.id).unwrap())
	}
	pub fn add_prop<T: 'static>(&self, value: T) -> Result<PropId<T>, Error> {
		let id = self.store().add_prop(value)?;
		self.slab().props.push(id.0);
		Ok(id)
	}
	fn add_prop_panicing<T: 'static>(&self, value: T) -> PropId<T> {
		let Ok(id) = self.add_prop(value) else { struct_change_while_life_refs() };
		id
	}

	pub fn signal<'scope, T: 'static>(&'scope self, value: T) -> Signal<'scope, T> {
		let id = self.add_prop_panicing(value);
		Signal { store: self.store, prop: id }
	}
	pub fn ro_signal<'scope, T: 'static>(&'scope self, value: T) -> ROSignal<'scope, T> {
		let id = self.add_prop_panicing(value);
		ROSignal { store: self.store, prop: id }
	}
	pub fn wo_signal<'scope, T: 'static>(&'scope self, value: T) -> WOSignal<'scope, T> {
		let id = self.add_prop_panicing(value);
		WOSignal { store: self.store, prop: id }
	}
}

#[derive(Debug, Default)]
pub struct SlabData {
	pub props: Vec<ItemId>,
}
