use std::{
	cell::{Cell, RefCell, UnsafeCell},
	mem::transmute,
	ops::DerefMut,
	panic::Location,
	ptr,
};

use rustc_hash::FxHashMap;
use slotmap::SlotMap;

use crate::reactive::{
	Error, PropId, ROSignal, SlabId, WOSignal,
	prop::{ItemId, Prop, PropStatus},
	signal::{MutGuard, ReadGuard, Signal},
	slab::{Slab, SlabData},
	struct_change_while_life_refs,
	updater::{Effect, Updater, start_track_panicing},
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TrackResult {
	pub read: Vec<PropId<()>>,
	pub written: Vec<PropId<()>>,
}
impl TrackResult {
	pub(crate) fn destruct(self) -> (Vec<PropId<()>>, Vec<PropId<()>>) {
		(self.read, self.written)
	}
}

#[derive(Debug)]
pub struct Store {
	pub(crate) props: UnsafeCell<SlotMap<ItemId, Prop>>,
	pub(crate) slabs: RefCell<FxHashMap<SlabId, SlabData>>,
	next_slab: Cell<SlabId>,

	pub(crate) updater: RefCell<Updater>,

	pub(crate) ref_count: Cell<u64>,
	tracking: RefCell<Option<TrackResult>>,
}
impl Default for Store {
	fn default() -> Self {
		Store {
			ref_count: Cell::new(0),
			props: UnsafeCell::new(SlotMap::default()),
			slabs: RefCell::new(FxHashMap::default()),
			next_slab: Cell::new(SlabId(0)),
			updater: RefCell::new(Updater::default()),
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
	pub(crate) fn props(&self) -> &SlotMap<ItemId, Prop> {
		unsafe { &*self.props.get() }
	}
	pub(crate) fn props_mut(&self) -> &mut SlotMap<ItemId, Prop> {
		unsafe { &mut *self.props.get() }
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
		let id = self.next_slab.get();
		self.slabs.borrow_mut().insert(id, SlabData::default());
		self.next_slab.set(SlabId(id.0 + 1));
		Ok(Slab { store: self, id })
	}
	pub fn slab(&self, id: SlabId) -> Result<Slab<'_>, Error> {
		if !self.slabs.borrow().contains_key(&id) {
			return Err(Error::Removed);
		}
		Ok(Slab { store: self, id })
	}
	pub fn remove_slab(&self, id: SlabId) -> Result<(), Error> {
		if self.ref_count.get() != 0 {
			return Err(Error::LiveRefs);
		}
		let mut slabs = self.slabs.borrow_mut();
		let slab = slabs.get(&id).ok_or(Error::Removed)?;

		let props = self.props_mut();
		for id in &slab.props {
			props.remove(*id);
		}

		self.updater.borrow_mut().remove_items(&slab.effects, &slab.props);

		slabs.remove(&id);
		Ok(())
	}

	fn get_prop(&self, id: ItemId) -> Result<&Prop, Error> {
		self.props().get(id).ok_or(Error::Removed)
	}
	pub fn try_peek<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<ReadGuard<'scope, T>, Error> {
		ReadGuard::new(self, self.get_prop(id.0)?).ok_or(Error::UnderMut)
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

	pub fn try_read<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<ReadGuard<'scope, T>, Error> {
		let guard = self.try_peek(id)?;
		self.track_read(id);
		Ok(guard)
	}
	pub fn read<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> ReadGuard<'scope, T> {
		self.track_read(id);
		self.peek(id)
	}

	pub fn try_read_mut<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<MutGuard<'scope, T>, Error> {
		let Some(guard) = MutGuard::new(self, self.get_prop(id.0)?) else {
			return Err(Error::LiveRefs);
		};
		self.track_write(id);
		Ok(guard)
	}
	pub fn read_mut<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> MutGuard<'scope, T> {
		match self.try_read_mut(id) {
			Ok(guard) => guard,
			Err(Error::Removed) => panic!("getting removed property ({id})"),
			Err(Error::LiveRefs) => panic!("mutating property ({id}) having live references"),
			_ => unreachable!(),
		}
	}

	pub fn try_write<T: 'static>(&self, id: PropId<T>, value: T) -> Result<(), Error> {
		self.get_prop(id.0)?.set(value);
		self.track_write(id);
		Ok(())
	}
	pub fn write<T: 'static>(&self, id: PropId<T>, value: T) {
		match self.try_write(id, value) {
			Ok(()) => (),
			Err(Error::Removed) => panic!("setting removed property ({id})"),
			_ => unreachable!(),
		}
	}

	pub fn add_prop<T: 'static>(&self, value: T) -> Result<PropId<T>, Error> {
		if self.ref_count.get() != 0 {
			return Err(Error::LiveRefs);
		}
		let id = self.props_mut().insert(Prop::new(value));
		Ok(PropId::new(id))
	}
	fn add_prop_panicing<T: 'static>(&self, value: T) -> PropId<T> {
		let Ok(id) = self.add_prop(value) else { struct_change_while_life_refs() };
		id
	}

	pub fn signal<T: 'static>(&self, value: T) -> Signal<'_, T> {
		Signal { store: self, prop: self.add_prop_panicing(value) }
	}
	pub fn ro_signal<T: 'static>(&self, value: T) -> ROSignal<'_, T> {
		ROSignal { store: self, prop: self.add_prop_panicing(value) }
	}
	pub fn wo_signal<T: 'static>(&self, value: T) -> WOSignal<'_, T> {
		WOSignal { store: self, prop: self.add_prop_panicing(value) }
	}
	pub fn revive<I: Ids>(&self, ids: I) -> I::Signals<'_> {
		I::revive(self, ids)
	}
	pub fn try_revive<T: 'static>(&self, id: PropId<T>) -> Option<Signal<'_, T>> {
		self.props().contains_key(id.0).then(|| Signal { store: self, prop: id })
	}

	#[track_caller]
	pub fn effect<'store>(&'store self, fun: impl FnMut() + 'store) {
		self.updater.borrow_mut().add_effect(self, fun, None, Location::caller());
	}
	#[track_caller]
	pub fn effect_manual<'store>(
		&'store self, read: Vec<PropId<()>>, write: Vec<PropId<()>>, fun: impl FnMut() + 'store,
	) {
		let mut updater = self.updater.borrow_mut();
		updater.add_effect(self, fun, Some((read, write)), Location::caller());
	}

	pub(crate) fn computed_manual<'store, T: 'static>(
		&'store self, mut fun: impl FnMut() -> T + 'store, loc: &'static Location,
	) -> ROSignal<'store, T> {
		start_track_panicing(self);
		let value = fun();
		let TrackResult { read, written } = self.end_track().unwrap();

		let id = self.add_prop_panicing(value);

		let fun = move || self.write(id, fun());
		let mut updater = self.updater.borrow_mut();
		updater.add_effect(self, fun, Some((read, written)), loc);

		ROSignal { store: self, prop: id }
	}

	#[track_caller]
	pub fn computed<'store, T: 'static>(
		&'store self, fun: impl FnMut() -> T + 'store,
	) -> ROSignal<'store, T> {
		self.computed_manual(fun, Location::caller())
	}

	pub fn status_of<T: 'static>(&self, id: PropId<T>) -> PropStatus {
		let Ok(prop) = self.get_prop(id.0) else { return PropStatus::Removed };
		prop.status()
	}
	pub fn has_live_refs(&self) -> bool {
		self.ref_count.get() != 0
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
	pub fn end_track(&self) -> Result<TrackResult, Error> {
		self.tracking.take().ok_or(Error::NotTracking)
	}
	pub fn track_read<T: 'static>(&self, id: PropId<T>) {
		if let Some(tracking) = self.tracking.borrow_mut().deref_mut() {
			if !tracking.read.contains(&id.erase_type()) {
				tracking.read.push(id.erase_type());
			}
		}
	}
	pub fn track_write<T: 'static>(&self, id: PropId<T>) {
		if let Some(tracking) = self.tracking.borrow_mut().deref_mut() {
			if !tracking.written.contains(&id.erase_type()) {
				tracking.written.push(id.erase_type());
			}
		}
	}

	pub fn force_update<T: 'static>(&self, id: PropId<T>) {
		todo!()
	}
}

pub trait Ids {
	type Signals<'scope>;
	fn revive(store: &Store, ids: Self) -> Self::Signals<'_>;
}
impl<T: 'static> Ids for PropId<T> {
	type Signals<'scope> = Signal<'scope, T>;
	fn revive(store: &Store, id: Self) -> Self::Signals<'_> {
		Signal { store, prop: id }
	}
}
macro_rules! id_tuple {
	[$($item:ident : $ind:tt),*] => {
		impl<$($item: 'static),*> Ids for ($(PropId<$item>),*,) {
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
