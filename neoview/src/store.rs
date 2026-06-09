use std::{any::Any, cell::RefCell, fmt::Debug, ops::DerefMut, ptr};

use rustc_hash::FxHashMap;
use slotmap::SlotMap;

use crate::{
	Error, PropId, SlabId,
	context::Context,
	prop::ItemId,
	updater::{Updater, start_track_panicing},
};

/// stores items owned by a slab
pub struct SlabData<Ctx> {
	pub props: Vec<ItemId>,
	pub effects: Vec<ItemId>,
	pub cleaner: Vec<Box<dyn FnOnce(&mut Ctx)>>,
}
impl<Ctx> Default for SlabData<Ctx> {
	fn default() -> Self {
		Self { props: Vec::new(), effects: Vec::new(), cleaner: Vec::new() }
	}
}
impl<Ctx> Debug for SlabData<Ctx> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SlabData")
			.field("props", &self.props)
			.field("effects", &self.effects)
			.finish()
	}
}

/// the result of a tracking operation.
///
/// produced by when the tracking operation is ended with [`Store::end_track`].
///
/// # example
/// ```
/// let a = store.prop(1);
/// let b = store.prop(2);
/// store.start_track();
/// store.set(b, store.get(a) + 2);
/// let result = store.end_track();
/// assert_eq!(result.read, [a.erase_type()]);
/// assert_eq!(result.written, [b.erase_type()]);
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TrackResult {
	/// the properties read.
	pub read: Vec<PropId<()>>,
	/// the properties written to.
	pub written: Vec<PropId<()>>,
}
impl TrackResult {
	/// destruct the `TrackResult` into a `(read, written)` pair
	pub(crate) fn destruct(self) -> (Vec<PropId<()>>, Vec<PropId<()>>) {
		(self.read, self.written)
	}
}

pub struct Store<Ctx> {
	pub(crate) props: SlotMap<ItemId, Box<dyn Any>>,

	pub(crate) slabs: FxHashMap<SlabId, SlabData<Ctx>>,
	/// the `SlabId` of the next slab to be added
	next_slab: SlabId,
	/// slabs removed during an update to be deleated at the end of that update
	slabs_to_remove: Vec<SlabId>,

	pub(crate) updater: Updater<Ctx>,

	global_cleaners: Vec<Box<dyn FnOnce(&mut Ctx)>>,
	is_dropped: bool,

	// `RefCell` to not make `read` take mutable reference
	tracking: RefCell<Option<TrackResult>>,
}
impl<Ctx: Context> Default for Store<Ctx> {
	fn default() -> Self {
		Store {
			props: SlotMap::default(),
			slabs: FxHashMap::default(),
			next_slab: SlabId(0),
			slabs_to_remove: Vec::new(),
			updater: Updater::default(),
			global_cleaners: Vec::new(),
			is_dropped: false,
			tracking: RefCell::new(None),
		}
	}
}
impl<Ctx> Debug for Store<Ctx> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Store")
			.field("props", &self.props)
			.field("slabs", &self.slabs)
			.field("slabs_to_remove", &self.slabs_to_remove)
			.field("effects", &self.updater.effects)
			.field("tarcking", &self.tracking)
			.field("is_dropped", &self.is_dropped)
			.finish()
	}
}
impl<Ctx> PartialEq for Store<Ctx> {
	fn eq(&self, other: &Self) -> bool {
		ptr::eq(self, other)
	}
}
impl<Ctx> Eq for Store<Ctx> {}
impl<Ctx: Context> Store<Ctx> {
	pub fn create_slab(&mut self) -> SlabId {
		let id = self.next_slab;
		self.slabs.insert(id, SlabData::default());
		self.next_slab = SlabId(id.0 + 1);
		id
	}
	fn slab(&mut self, slab: SlabId) -> &mut SlabData<Ctx> {
		self.slabs.get_mut(&slab).unwrap()
	}
	pub fn has_slab(&self, slab: SlabId) -> bool {
		self.slabs.contains_key(&slab) && !self.slabs_to_remove.contains(&slab)
	}
	pub fn remove_slab(ctx: &mut Ctx, id: SlabId) -> Result<(), Error> {
		let store = ctx.store();
		if !store.has_slab(id) {
			return Err(Error::Removed);
		}

		if store.updater.is_updating {
			store.slabs_to_remove.push(id);
		} else {
			Store::_remove_slab(ctx, id);
		}
		Ok(())
	}
	fn _remove_slab(ctx: &mut Ctx, id: SlabId) {
		while let Some(cleaner) = ctx.store().slab(id).cleaner.pop() {
			cleaner(ctx)
		}
		let store = ctx.store();
		let slab = &store.slabs.remove(&id).unwrap();
		for id in &slab.props {
			store.props.remove(*id);
		}
		store.updater.remove_items(&slab.effects, &slab.props);
	}

	pub fn add_global_cleaner(&mut self, fun: impl FnOnce(&mut Ctx) + 'static) {
		self.global_cleaners.push(Box::new(fun))
	}
	pub fn add_cleaner_in(
		&mut self, slab: Option<SlabId>, fun: impl FnOnce(&mut Ctx) + 'static,
	) -> Result<(), Error> {
		let Some(slab) = slab else {
			self.add_global_cleaner(fun);
			return Ok(());
		};
		if !self.has_slab(slab) {
			return Err(Error::Removed);
		}
		self.slab(slab).cleaner.push(Box::new(fun));
		Ok(())
	}

	pub fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let id = self.props.insert(Box::new(value));
		PropId::new(id)
	}
	pub fn prop_in<T: 'static>(
		&mut self, slab: Option<SlabId>, value: T,
	) -> Result<PropId<T>, Error> {
		let Some(slab) = slab else {
			return Ok(self.prop(value));
		};
		if !self.has_slab(slab) {
			return Err(Error::Removed);
		}
		let id = self.prop(value);
		self.slab(slab).props.push(id.0);
		Ok(id)
	}

	pub fn contains<T>(&self, id: PropId<T>) -> bool {
		self.props.contains_key(id.0)
	}

	pub fn try_peek<T: 'static>(&self, id: PropId<T>) -> Option<&T> {
		self.props.get(id.0)?.downcast_ref()
	}
	pub fn peek<T: 'static>(&self, id: PropId<T>) -> &T {
		self.try_peek(id).expect("reading removed property")
	}

	pub fn try_read<T: 'static>(&self, id: PropId<T>) -> Option<&T> {
		let value = self.try_peek(id)?;
		self.track_read(id);
		Some(value)
	}
	pub fn read<T: 'static>(&self, id: PropId<T>) -> &T {
		self.track_read(id);
		self.peek(id)
	}

	pub fn try_get<T: 'static + Copy>(&self, id: PropId<T>) -> Option<T> {
		self.try_read(id).copied()
	}
	pub fn get<T: 'static + Copy>(&self, id: PropId<T>) -> T {
		*self.read(id)
	}

	pub fn try_read_mut<T: 'static>(&mut self, id: PropId<T>) -> Option<&mut T> {
		let prop = self.props.get_mut(id.0)?.downcast_mut()?;
		Self::_track_write(&self.tracking, id);
		self.updater.push_update(id.0);
		Some(prop)
	}
	pub fn read_mut<T: 'static>(&mut self, id: PropId<T>) -> &mut T {
		self.try_read_mut(id).expect("mutating removed property")
	}

	pub fn try_write<T: 'static>(&mut self, id: PropId<T>, value: T) -> Result<(), Error> {
		let prop = self.props.get_mut(id.0).ok_or(Error::Removed)?;
		*prop.downcast_mut().unwrap() = value;
		self.track_write(id);
		self.updater.push_update(id.0);
		Ok(())
	}
	pub fn write<T: 'static>(&mut self, id: PropId<T>, value: T) {
		self.try_write(id, value).expect("writing removed property");
	}

	pub fn try_update<T: 'static>(
		&mut self, id: PropId<T>, fun: impl FnOnce(&mut T),
	) -> Result<(), Error> {
		let prop = self.props.get_mut(id.0).ok_or(Error::Removed)?;
		fun(prop.downcast_mut().unwrap());
		self.track_write(id);
		self.updater.push_update(id.0);
		Ok(())
	}
	pub fn update<T: 'static>(&mut self, id: PropId<T>, fun: impl FnOnce(&mut T)) {
		self.try_update(id, fun).expect("updating removed property");
	}

	pub fn effect(ctx: &mut Ctx, fun: impl FnMut(&mut Ctx) + 'static) {
		Updater::add_effect(ctx, fun, None, true);
	}
	pub fn effect_manual(
		ctx: &mut Ctx, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Ctx) + 'static, init_run: bool,
	) {
		Updater::add_effect(ctx, fun, Some((read, write)), init_run);
	}
	pub fn effect_in(
		ctx: &mut Ctx, slab: Option<SlabId>, fun: impl FnMut(&mut Ctx) + 'static,
	) -> Result<(), Error> {
		let Some(slab) = slab else {
			return Ok(Store::effect(ctx, fun));
		};
		if !ctx.store().has_slab(slab) {
			return Err(Error::Removed);
		}
		let id = Updater::add_effect(ctx, fun, None, true);
		ctx.store().slab(slab).effects.push(id);
		Ok(())
	}
	pub fn effect_manual_in(
		ctx: &mut Ctx, slab: Option<SlabId>, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Ctx) + 'static, init_run: bool,
	) -> Result<(), Error> {
		let Some(slab) = slab else {
			return Ok(Store::effect_manual(ctx, read, write, fun, init_run));
		};
		if !ctx.store().has_slab(slab) {
			return Err(Error::Removed);
		}
		let id = Updater::add_effect(ctx, fun, Some((read, write)), init_run);
		ctx.store().slab(slab).effects.push(id);
		Ok(())
	}

	pub(crate) fn computed_manual<T: 'static>(
		ctx: &mut Ctx, mut fun: impl FnMut(&mut Ctx) -> T + 'static,
	) -> (PropId<T>, ItemId) {
		start_track_panicing(ctx.store_ref());
		let value = fun(ctx);
		let store = ctx.store();
		let TrackResult { read, written } = store.end_track().unwrap();

		if !written.is_empty() {
			panic!("computed properties can not write any properties");
		}

		let id = store.prop(value);

		let fun = move |ctx: &mut Ctx| {
			let value = fun(ctx);
			ctx.store().write(id, value);
		};
		let effect = Updater::add_effect(ctx, fun, Some((read, vec![id.erase_type()])), false);

		(id, effect)
	}

	pub fn computed<T: 'static>(
		ctx: &mut Ctx, fun: impl FnMut(&mut Ctx) -> T + 'static,
	) -> PropId<T> {
		Self::computed_manual(ctx, fun).0
	}
	pub fn computed_in<T: 'static>(
		ctx: &mut Ctx, slab: Option<SlabId>, fun: impl FnMut(&mut Ctx) -> T + 'static,
	) -> Result<PropId<T>, Error> {
		let Some(slab) = slab else {
			return Ok(Store::computed(ctx, fun));
		};
		if !ctx.store().has_slab(slab) {
			return Err(Error::Removed);
		}
		let (id, effect) = Self::computed_manual(ctx, fun);
		let slab = ctx.store().slab(slab);
		slab.effects.push(effect);
		slab.props.push(id.0);
		Ok(id)
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
		let mut result = self.tracking.take().ok_or(Error::NotTracking)?;

		result.read.sort_unstable();
		result.read.dedup();
		result.written.sort_unstable();
		result.written.dedup();

		Ok(result)
	}
	pub fn track_read<T: 'static>(&self, id: PropId<T>) {
		if let Some(tracking) = self.tracking.borrow_mut().deref_mut() {
			tracking.read.push(id.erase_type());
		}
	}
	fn _track_write<T: 'static>(tracking: &RefCell<Option<TrackResult>>, id: PropId<T>) {
		if let Some(tracking) = tracking.borrow_mut().deref_mut() {
			tracking.written.push(id.erase_type());
		}
	}
	pub fn track_write<T: 'static>(&self, id: PropId<T>) {
		Self::_track_write(&self.tracking, id);
	}

	pub fn is_updating(&self) -> bool {
		self.updater.is_updating
	}

	pub fn force_update<T: 'static>(&mut self, id: PropId<T>) {
		if !self.updater.dirty_props.contains(&id.0) {
			self.updater.dirty_props.push(id.0);
		}
	}
	pub fn flush_updates(ctx: &mut Ctx) {
		Updater::update(ctx);

		while let Some(slab) = ctx.store().slabs_to_remove.pop() {
			Store::_remove_slab(ctx, slab);
		}
	}

	pub fn pre_drop(ctx: &mut Ctx) {
		let store = ctx.store();
		if store.is_dropped {
			panic!("calling `Store::pre_drop` twice")
		}
		store.is_dropped = true;
		while let Some(&slab) = ctx.store().slabs.keys().next() {
			Store::remove_slab(ctx, slab).unwrap()
		}
		while let Some(cleaner) = ctx.store().global_cleaners.pop() {
			cleaner(ctx)
		}
	}
}
impl<Ctx> Drop for Store<Ctx> {
	fn drop(&mut self) {
		if !self.is_dropped {
			panic!("dropped without calling `Store::pre_drop`")
		}
	}
}
