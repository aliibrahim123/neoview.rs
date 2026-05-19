use std::cell::UnsafeCell;

use crate::reactive::{
	Error, PropId, PropIndex, SlabId, Store,
	prop::Prop,
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

	pub fn add_prop<T: 'static>(&self, value: T) -> Result<PropId<T>, Error> {
		if self.store().ref_count.get() != 0 {
			return Err(Error::LiveRefs);
		}
		let slab = self.store.slabs().get_mut(&self.id).unwrap();
		if slab.props().len() == PropIndex::MAX {
			return Err(Error::OverCapacity);
		}
		Ok(slab.add_prop(value))
	}
	fn add_prop_panicing<T: 'static>(&self, value: T) -> PropId<T> {
		match self.add_prop(value) {
			Ok(id) => id,
			Err(Error::LiveRefs) => struct_change_while_life_refs(),
			Err(Error::OverCapacity) => panic!("slab ({}) is full", self.id),
			_ => unreachable!(),
		}
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

#[derive(Debug)]
pub struct SlabData {
	id: SlabId,
	props: UnsafeCell<Vec<Prop>>,
	pub global: bool,
}
impl SlabData {
	pub fn new(id: SlabId, global: bool) -> Self {
		Self { id, props: UnsafeCell::new(Vec::new()), global }
	}
	pub fn props(&self) -> &mut Vec<Prop> {
		unsafe { &mut *self.props.get() }
	}
	pub fn add_prop<T: 'static>(&self, value: T) -> PropId<T> {
		let props = self.props();
		let ind = props.len();
		props.push(Prop::new(value));
		PropId::new(self.id.value(), ind as u16)
	}
	pub fn get_prop<T: 'static>(&self, id: PropId<T>) -> &Prop {
		&self.props()[id.prop_index().value() as usize]
	}
}
