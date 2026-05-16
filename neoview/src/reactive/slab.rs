use std::cell::UnsafeCell;

use crate::reactive::{PropId, SlabId, Store, prop::Prop, store::Removed};

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
	pub fn add_prop<T: 'static>(&self, value: T) -> Result<PropId<T>, Removed> {
		if self.store().ref_count.get() != 0 {
			return Err(Removed);
		}
		Ok(self.store.slabs()[&self.id].add_prop(value))
	}
}

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
