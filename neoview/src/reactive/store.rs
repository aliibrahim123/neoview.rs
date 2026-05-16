use std::cell::{Cell, UnsafeCell};

use rustc_hash::FxHashMap;

use crate::reactive::{
	PropId, SlabId,
	signal::{MutGuard, ReadGuard},
	slab::{Slab, SlabData},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GetError {
	Removed,
	UnderMut,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutError {
	Removed,
	LiveRefs,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Removed;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LiveRefs;

#[derive(Debug)]
pub struct Store {
	pub(crate) slabs: UnsafeCell<FxHashMap<SlabId, SlabData>>,
	pub(crate) ref_count: Cell<u64>,
	pub(crate) cur_slab: Cell<SlabId>,
}
impl Default for Store {
	fn default() -> Self {
		Store {
			slabs: UnsafeCell::new(FxHashMap::default()),
			ref_count: Cell::new(0),
			cur_slab: Cell::new(SlabId(0)),
		}
	}
}
impl Store {
	pub(crate) fn slabs(&self) -> &mut FxHashMap<SlabId, SlabData> {
		unsafe { &mut *self.slabs.get() }
	}

	pub(crate) fn inc_ref(&self) {
		self.ref_count.update(|c| c + 1);
	}
	pub(crate) fn dec_ref(&self) {
		self.ref_count.update(|c| c - 1);
	}

	pub fn add_slab(&self) -> Result<Slab<'_>, LiveRefs> {
		if self.ref_count.get() != 0 {
			return Err(LiveRefs);
		}
		let id = self.cur_slab.get();
		self.slabs().insert(id, SlabData::new(id));
		self.cur_slab.set(SlabId(id.0 + 1));
		Ok(Slab { store: self, id })
	}
	pub fn slab(&self, id: SlabId) -> Result<Slab<'_>, Removed> {
		if self.slabs().contains_key(&id) { Ok(Slab { store: self, id }) } else { Err(Removed) }
	}

	pub fn try_get<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<ReadGuard<'scope, T>, GetError> {
		let Some(slab) = self.slabs().get(&id.slab()) else { return Err(GetError::Removed) };
		ReadGuard::new(self, slab.get_prop(id)).ok_or(GetError::UnderMut)
	}
	pub fn get<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> ReadGuard<'scope, T> {
		match self.try_get(id) {
			Ok(guard) => guard,
			Err(GetError::Removed) => panic!("getting removed property ({id})"),
			Err(GetError::UnderMut) => panic!("getting property under mutation ({id})"),
		}
	}

	pub fn try_get_mut<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<MutGuard<'scope, T>, MutError> {
		let Some(slab) = self.slabs().get(&id.slab()) else { return Err(MutError::Removed) };
		MutGuard::new(self, slab.get_prop(id)).ok_or(MutError::LiveRefs)
	}
	pub fn get_mut<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> MutGuard<'scope, T> {
		match self.try_get_mut(id) {
			Ok(guard) => guard,
			Err(MutError::Removed) => panic!("getting removed property ({id})"),
			Err(MutError::LiveRefs) => panic!("mutating property ({id}) having live references"),
		}
	}

	pub fn try_set<T: 'static>(&self, id: PropId<T>, value: T) -> Result<(), Removed> {
		let Some(slab) = self.slabs().get(&id.slab()) else { return Err(Removed) };
		slab.get_prop(id).set(value);
		Ok(())
	}
}
