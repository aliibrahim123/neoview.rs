use std::{
	cell::{Cell, RefCell, UnsafeCell},
	ops::DerefMut,
	ptr,
};

use rustc_hash::FxHashMap;

use crate::reactive::{
	Error, PropId, SlabId,
	prop::PropStatus,
	signal::{MutGuard, ReadGuard, Signal},
	slab::{Slab, SlabData},
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TrackResult {
	pub read: Vec<PropId<()>>,
	pub written: Vec<PropId<()>>,
}

#[derive(Debug)]
pub struct Store {
	pub(crate) slabs: UnsafeCell<FxHashMap<SlabId, SlabData>>,
	pub(crate) ref_count: Cell<u64>,
	pub(crate) cur_slab: Cell<SlabId>,
	pub(crate) tracking: RefCell<Option<TrackResult>>,
}
impl Default for Store {
	fn default() -> Self {
		Store {
			slabs: UnsafeCell::new(FxHashMap::default()),
			ref_count: Cell::new(0),
			cur_slab: Cell::new(SlabId(0)),
			tracking: RefCell::new(None),
		}
	}
}
impl PartialEq for Store {
	fn eq(&self, other: &Self) -> bool {
		ptr::eq(self, other)
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

	pub fn add_slab(&self) -> Result<Slab<'_>, Error> {
		if self.ref_count.get() != 0 {
			return Err(Error::LiveRefs);
		}
		let id = self.cur_slab.get();
		self.slabs().insert(id, SlabData::new(id));
		self.cur_slab.set(SlabId(id.0 + 1));
		Ok(Slab { store: self, id })
	}
	pub fn slab(&self, id: SlabId) -> Result<Slab<'_>, Error> {
		if self.slabs().contains_key(&id) {
			Ok(Slab { store: self, id })
		} else {
			Err(Error::Removed)
		}
	}
	pub fn remove_slab(&self, id: SlabId) -> Result<(), Error> {
		if self.ref_count.get() != 0 {
			return Err(Error::LiveRefs);
		}
		if self.slabs().remove(&id).is_none() {
			return Err(Error::Removed);
		}
		Ok(())
	}

	pub fn try_peek<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<ReadGuard<'scope, T>, Error> {
		let Some(slab) = self.slabs().get(&id.slab()) else { return Err(Error::Removed) };
		ReadGuard::new(self, slab.get_prop(id)).ok_or(Error::UnderMut)
	}
	pub fn peek<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> ReadGuard<'scope, T> {
		match self.try_peek(id) {
			Ok(guard) => guard,
			Err(Error::Removed) => panic!("getting removed property ({id})"),
			Err(Error::UnderMut) => panic!("getting property ({id}) under mutation"),
			_ => unreachable!(),
		}
	}

	pub fn try_get<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<ReadGuard<'scope, T>, Error> {
		let guard = self.try_peek(id)?;
		self.track_read(id);
		Ok(guard)
	}
	pub fn get<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> ReadGuard<'scope, T> {
		self.track_read(id);
		self.peek(id)
	}

	pub fn try_get_mut<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<MutGuard<'scope, T>, Error> {
		let Some(slab) = self.slabs().get(&id.slab()) else { return Err(Error::Removed) };
		let Some(guard) = MutGuard::new(self, slab.get_prop(id)) else {
			return Err(Error::LiveRefs);
		};
		self.track_write(id);
		Ok(guard)
	}
	pub fn get_mut<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> MutGuard<'scope, T> {
		match self.try_get_mut(id) {
			Ok(guard) => guard,
			Err(Error::Removed) => panic!("getting removed property ({id})"),
			Err(Error::LiveRefs) => panic!("mutating property ({id}) having live references"),
			_ => unreachable!(),
		}
	}

	pub fn try_set<T: 'static>(&self, id: PropId<T>, value: T) -> Result<(), Error> {
		let Some(slab) = self.slabs().get(&id.slab()) else { return Err(Error::Removed) };
		slab.get_prop(id).set(value);
		self.track_write(id);
		Ok(())
	}
	pub fn set<T: 'static>(&self, id: PropId<T>, value: T) {
		match self.try_set(id, value) {
			Ok(()) => (),
			Err(Error::Removed) => panic!("setting removed property ({id})"),
			_ => unreachable!(),
		}
	}

	pub fn satus_of<T: 'static>(&self, id: PropId<T>) -> PropStatus {
		let Some(slab) = self.slabs().get(&id.slab()) else { return PropStatus::Removed };
		slab.get_prop(id).status()
	}
	pub fn has_live_refs(&self) -> bool {
		self.ref_count.get() != 0
	}

	pub fn revive<Tuple: IdTuple>(&self, ids: Tuple) -> Tuple::Signals<'_> {
		Tuple::revive(self, ids)
	}

	pub fn is_tracking(&self) -> bool {
		self.tracking.borrow().is_some()
	}
	pub fn start_track(&self) -> Result<(), Error> {
		if self.is_tracking() {
			return Err(Error::Tracking);
		}
		self.tracking.replace(Some(TrackResult::default()));
		Ok(())
	}
	pub fn stop_track(&self) -> Result<TrackResult, Error> {
		self.tracking.take().ok_or(Error::NotTracking)
	}
	pub fn track_read<T: 'static>(&self, id: PropId<T>) {
		if let Some(tracking) = self.tracking.borrow_mut().deref_mut() {
			tracking.read.push(id.erase_type());
		}
	}
	pub fn track_write<T: 'static>(&self, id: PropId<T>) {
		if let Some(tracking) = self.tracking.borrow_mut().deref_mut() {
			tracking.written.push(id.erase_type());
		}
	}

	pub fn force_update<T: 'static>(&self, id: PropId<T>) {
		todo!()
	}
}

pub trait IdTuple {
	type Signals<'scope>;
	fn revive(store: &Store, ids: Self) -> Self::Signals<'_>;
}
macro_rules! id_tuple {
	[$($item:ident : $ind:tt),*] => {
		impl<$($item: 'static),*> IdTuple for ($(PropId<$item>),*,) {
			type Signals<'scope> = ($(Signal<'scope, $item>),*,);
			fn revive(store: &Store, ids: Self) -> Self::Signals<'_> {
				($(Signal { store, prop: ids.$ind }),*,)
			}
		}
	};

}
id_tuple![A: 0];
id_tuple![A: 0, B: 1];
id_tuple![A: 0, B: 1, C: 2];
id_tuple![A: 0, B: 1, C: 2, D: 3];
id_tuple![A: 0, B: 1, C: 2, D: 3, E: 4];
id_tuple![A: 0, B: 1, C: 2, D: 3, E: 4, F: 5];
id_tuple![A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6];
id_tuple![A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7];
