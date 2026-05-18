use std::cell::UnsafeCell;

use crate::reactive::{
	Error, PropId, SlabId, Store,
	prop::Prop,
	signal::{ROSignal, Signal, WOSignal},
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
		Ok(self.store.slabs()[&self.id].add_prop(value))
	}

	pub fn signal<'scope, T: 'static>(&'scope self, value: T) -> Signal<'scope, T> {
		let Ok(id) = self.add_prop(value) else {
			panic!("can not do a structural change while there is live references");
		};
		Signal { store: self.store, prop: id }
	}
	pub fn ro_signal<'scope, T: 'static>(&'scope self, value: T) -> ROSignal<'scope, T> {
		let Ok(id) = self.add_prop(value) else {
			panic!("can not do a structural change while there is live references");
		};
		ROSignal { store: self.store, prop: id }
	}
	pub fn wo_signal<'scope, T: 'static>(&'scope self, value: T) -> WOSignal<'scope, T> {
		let Ok(id) = self.add_prop(value) else {
			panic!("can not do a structural change while there is live references");
		};
		WOSignal { store: self.store, prop: id }
	}
}

#[derive(Debug)]
pub struct SlabData {
	id: SlabId,
	props: UnsafeCell<Vec<Prop>>,
}
impl SlabData {
	pub fn new(id: SlabId) -> Self {
		Self { id, props: UnsafeCell::new(Vec::new()) }
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
