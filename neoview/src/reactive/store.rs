use std::{
	cell::{Cell, RefCell, UnsafeCell},
	mem::transmute,
	ops::DerefMut,
	panic::Location,
	ptr,
};

use rustc_hash::FxHashMap;
use slotmap::SlotMap;

use crate::{
	context::{Context, VoidContext},
	reactive::{
		Error, PropId, ROSignal, SlabId, WOSignal,
		prop::{ItemId, Prop, PropStatus},
		signal::{MutGuard, ReadGuard, Signal},
		slab::{Slab, SlabData},
		struct_change_while_life_refs,
		updater::Effect,
	},
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TrackResult {
	pub read: Vec<PropId<()>>,
	pub written: Vec<PropId<()>>,
}

type CtxGetter<Ctx> = for<'a> fn(<Ctx as Context>::Id, &'a ()) -> &'a Ctx;

#[derive(Debug)]
pub struct Store<Ctx: Context = VoidContext> {
	pub(crate) props: UnsafeCell<SlotMap<ItemId, Prop>>,
	pub(crate) effects: RefCell<SlotMap<ItemId, Effect<Ctx>>>,

	pub(crate) slabs: RefCell<FxHashMap<SlabId, SlabData>>,
	next_slab: Cell<SlabId>,

	pub(crate) ref_count: Cell<u64>,
	tracking: RefCell<Option<TrackResult>>,

	ctx_id: Ctx::Id,
	ctx_getter: CtxGetter<Ctx>,
}
impl Default for Store<VoidContext> {
	fn default() -> Self {
		Store::new((), VoidContext::get_by_id)
	}
}
impl<Ctx: Context> PartialEq for Store<Ctx> {
	fn eq(&self, other: &Self) -> bool {
		ptr::eq(self, other)
	}
}
impl<Ctx: Context> Store<Ctx> {
	fn new(ctx_id: Ctx::Id, ctx_getter: CtxGetter<Ctx>) -> Self {
		Store {
			ref_count: Cell::new(0),
			props: UnsafeCell::new(SlotMap::with_key()),
			slabs: RefCell::new(FxHashMap::default()),
			next_slab: Cell::new(SlabId(0)),
			effects: RefCell::new(SlotMap::with_key()),
			tracking: RefCell::new(None),
			ctx_id,
			ctx_getter,
		}
	}
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

	pub fn add_slab(&self) -> Result<Slab<'_, Ctx>, Error> {
		if self.ref_count.get() != 0 {
			return Err(Error::LiveRefs);
		}
		let id = self.next_slab.get();
		self.slabs.borrow_mut().insert(id, SlabData::default());
		self.next_slab.set(SlabId(id.0 + 1));
		Ok(Slab { store: self, id })
	}
	pub fn slab(&self, id: SlabId) -> Result<Slab<'_, Ctx>, Error> {
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
		let mut effects = self.effects.borrow_mut();
		for id in &slab.effects {
			effects.remove(*id);
		}
		slabs.remove(&id);
		Ok(())
	}

	fn get_prop(&self, id: ItemId) -> Result<&Prop, Error> {
		self.props().get(id).ok_or(Error::Removed)
	}
	pub fn try_peek<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<ReadGuard<'scope, T, Ctx>, Error> {
		ReadGuard::new(self, self.get_prop(id.0)?).ok_or(Error::UnderMut)
	}
	pub fn peek<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> ReadGuard<'scope, T, Ctx> {
		match self.try_peek(id) {
			Ok(guard) => guard,
			Err(Error::Removed) => panic!("getting removed property ({id})"),
			Err(Error::UnderMut) => panic!("getting property ({id}) under mutation"),
			_ => unreachable!(),
		}
	}

	pub fn try_get<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<ReadGuard<'scope, T, Ctx>, Error> {
		let guard = self.try_peek(id)?;
		self.track_read(id);
		Ok(guard)
	}
	pub fn get<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> ReadGuard<'scope, T, Ctx> {
		self.track_read(id);
		self.peek(id)
	}

	pub fn try_get_mut<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> Result<MutGuard<'scope, T, Ctx>, Error> {
		let Some(guard) = MutGuard::new(self, self.get_prop(id.0)?) else {
			return Err(Error::LiveRefs);
		};
		self.track_write(id);
		Ok(guard)
	}
	pub fn get_mut<'store: 'scope, 'scope, T: 'static>(
		&'store self, id: PropId<T>,
	) -> MutGuard<'scope, T, Ctx> {
		match self.try_get_mut(id) {
			Ok(guard) => guard,
			Err(Error::Removed) => panic!("getting removed property ({id})"),
			Err(Error::LiveRefs) => panic!("mutating property ({id}) having live references"),
			_ => unreachable!(),
		}
	}

	pub fn try_set<T: 'static>(&self, id: PropId<T>, value: T) -> Result<(), Error> {
		self.get_prop(id.0)?.set(value);
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

	pub fn signal<'store: 'scope, 'scope, T: 'static>(
		&'store self, value: T,
	) -> Signal<'scope, T, Ctx> {
		Signal { store: self, prop: self.add_prop_panicing(value) }
	}
	pub fn ro_signal<'store: 'scope, 'scope, T: 'static>(
		&'store self, value: T,
	) -> ROSignal<'scope, T, Ctx> {
		ROSignal { store: self, prop: self.add_prop_panicing(value) }
	}
	pub fn wo_signal<'store: 'scope, 'scope, T: 'static>(
		&'store self, value: T,
	) -> WOSignal<'scope, T, Ctx> {
		WOSignal { store: self, prop: self.add_prop_panicing(value) }
	}
	pub fn revive<Tuple: IdTuple<Ctx>>(&self, ids: Tuple) -> Tuple::Signals<'_> {
		Tuple::revive(self, ids)
	}
	pub fn try_revive<T: 'static>(&self, id: PropId<T>) -> Option<Signal<'_, T, Ctx>> {
		self.props().contains_key(id.0).then(|| Signal { store: self, prop: id })
	}

	pub(crate) fn add_effect<'store>(
		&'store self, mut fun: impl FnMut(&Ctx) + 'store, loc: &'static Location,
	) -> ItemId {
		let ctx = (self.ctx_getter)(self.ctx_id, &());
		self.start_track().unwrap();
		fun(ctx);
		let TrackResult { read, written } = self.end_track().unwrap();

		self.add_effect_manual(read, written, fun, loc)
	}
	pub(crate) fn add_effect_manual<'store>(
		&'store self, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&Ctx) + 'store, loc: &'static Location,
	) -> ItemId {
		let fun = Box::new(fun);
		let fun = unsafe {
			transmute::<Box<dyn FnMut(&Ctx) + 'store>, Box<dyn FnMut(&Ctx) + 'static>>(fun)
		};

		let write = write.into_iter().map(|id| id.0).collect();
		let read = read.into_iter().map(|id| id.0).collect();

		self.effects.borrow_mut().insert(Effect { fun, loc, write, read })
	}

	#[track_caller]
	pub fn effect<'store>(&'store self, fun: impl FnMut(&Ctx) + 'store) {
		self.add_effect(fun, Location::caller());
	}
	#[track_caller]
	pub fn effect_manual<'store>(
		&'store self, read: Vec<PropId<()>>, write: Vec<PropId<()>>, fun: impl FnMut(&Ctx) + 'store,
	) {
		self.add_effect_manual(read, write, fun, Location::caller());
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

pub trait IdTuple<Ctx: Context> {
	type Signals<'scope>;
	fn revive(store: &Store<Ctx>, ids: Self) -> Self::Signals<'_>;
}
macro_rules! id_tuple {
	[$($item:ident : $ind:tt),*] => {
		impl<$($item: 'static),*, Ctx: Context> IdTuple<Ctx> for ($(PropId<$item>),*,) {
			type Signals<'scope> = ($(Signal<'scope, $item, Ctx>),*,);
			fn revive(store: &Store<Ctx>, ids: Self) -> Self::Signals<'_> {
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
