use std::{cell::RefMut, panic::Location};

use crate::{
	context::Context,
	reactive::{
		Error, PropId, SlabId, Store,
		prop::ItemId,
		signal::{ROSignal, Signal, WOSignal},
		struct_change_while_life_refs,
	},
};

pub struct Slab<'store, Ctx: Context> {
	pub(crate) store: &'store Store<Ctx>,
	pub(crate) id: SlabId,
}
impl<'store, Ctx: Context> Slab<'store, Ctx> {
	pub fn store(&self) -> &Store<Ctx> {
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

	pub fn signal<'scope, T: 'static>(&'scope self, value: T) -> Signal<'scope, T, Ctx> {
		let id = self.add_prop_panicing(value);
		Signal { store: self.store, prop: id }
	}
	pub fn ro_signal<'scope, T: 'static>(&'scope self, value: T) -> ROSignal<'scope, T, Ctx> {
		let id = self.add_prop_panicing(value);
		ROSignal { store: self.store, prop: id }
	}
	pub fn wo_signal<'scope, T: 'static>(&'scope self, value: T) -> WOSignal<'scope, T, Ctx> {
		let id = self.add_prop_panicing(value);
		WOSignal { store: self.store, prop: id }
	}

	#[track_caller]
	pub fn effect(&self, fun: impl FnMut(&Ctx) + 'store) {
		let id = self.store().add_effect(fun, Location::caller());
		self.slab().effects.push(id);
	}
	#[track_caller]
	pub fn effect_manual(
		&self, read: Vec<PropId<()>>, write: Vec<PropId<()>>, fun: impl FnMut(&Ctx) + 'store,
	) {
		let id = self.store().add_effect_manual(read, write, fun, Location::caller());
		self.slab().effects.push(id);
	}
}

#[derive(Debug, Default)]
pub struct SlabData {
	pub props: Vec<ItemId>,
	pub effects: Vec<ItemId>,
}
