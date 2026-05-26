use std::{any::Any, cell::RefCell, marker::PhantomData, ops::DerefMut, ptr};

use rustc_hash::FxHashMap;
use slotmap::SlotMap;

use crate::{
	context::Context,
	reactive::{
		Error, PropId, SlabId,
		prop::ItemId,
		updater::{Updater, start_track_panicing},
	},
};

#[derive(Debug, Default)]
pub struct SlabData {
	pub props: Vec<ItemId>,
	pub effects: Vec<ItemId>,
}

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
pub struct Store<Ctx> {
	pub(crate) props: SlotMap<ItemId, Box<dyn Any>>,

	pub(crate) slabs: FxHashMap<SlabId, SlabData>,
	next_slab: SlabId,
	slabs_to_remove: Vec<SlabId>,

	pub(crate) updater: Updater<Ctx>,

	tracking: RefCell<Option<TrackResult>>,
	_marker: PhantomData<Ctx>,
}
impl<Ctx: Context> Default for Store<Ctx> {
	fn default() -> Self {
		Store {
			props: SlotMap::default(),
			slabs: FxHashMap::default(),
			next_slab: SlabId(0),
			slabs_to_remove: Vec::new(),
			updater: Updater::default(),
			tracking: RefCell::new(None),
			_marker: PhantomData,
		}
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
	pub fn remove_slab(&mut self, id: SlabId) -> Result<(), Error> {
		if !self.slabs.contains_key(&id) || self.slabs_to_remove.contains(&id) {
			return Err(Error::Removed);
		}

		if self.updater.is_updating {
			self.slabs_to_remove.push(id);
		} else {
			self._remove_slab(id);
		}
		Ok(())
	}
	fn _remove_slab(&mut self, id: SlabId) {
		let slab = &self.slabs.remove(&id).unwrap();
		for id in &slab.props {
			self.props.remove(*id);
		}
		self.updater.remove_items(&slab.effects, &slab.props);
	}

	pub fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let id = self.props.insert(Box::new(value));
		PropId::new(id)
	}
	pub fn prop_in<T: 'static>(&mut self, slab: SlabId, value: T) -> Result<PropId<T>, Error> {
		if !self.slabs.contains_key(&slab) {
			return Err(Error::Removed);
		}
		let id = self.prop(value);
		self.slabs.get_mut(&slab).unwrap().props.push(id.0);
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
		fun: impl FnMut(&mut Ctx) + 'static,
	) {
		Updater::add_effect(ctx, fun, Some((read, write)), true);
	}
	pub fn effect_in(
		ctx: &mut Ctx, slab: SlabId, fun: impl FnMut(&mut Ctx) + 'static,
	) -> Result<(), Error> {
		if !ctx.store().slabs.contains_key(&slab) {
			return Err(Error::Removed);
		}
		let id = Updater::add_effect(ctx, fun, None, true);
		ctx.store().slabs.get_mut(&slab).unwrap().effects.push(id);
		Ok(())
	}
	pub fn effect_manual_in(
		ctx: &mut Ctx, slab: SlabId, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Ctx) + 'static,
	) -> Result<(), Error> {
		if !ctx.store().slabs.contains_key(&slab) {
			return Err(Error::Removed);
		}
		let id = Updater::add_effect(ctx, fun, Some((read, write)), true);
		ctx.store().slabs.get_mut(&slab).unwrap().effects.push(id);
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
		ctx: &mut Ctx, slab: SlabId, fun: impl FnMut(&mut Ctx) -> T + 'static,
	) -> Result<PropId<T>, Error> {
		if !ctx.store().slabs.contains_key(&slab) {
			return Err(Error::Removed);
		}
		let (id, effect) = Self::computed_manual(ctx, fun);
		let slab = ctx.store().slabs.get_mut(&slab).unwrap();
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

		let store = ctx.store();
		for ind in 0..store.slabs_to_remove.len() {
			let slab = store.slabs_to_remove[ind];
			store._remove_slab(slab);
		}
		store.slabs_to_remove.clear();
	}
}
