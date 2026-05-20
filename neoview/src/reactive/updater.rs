use std::{fmt::Debug, panic::Location};

use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::{
	context::Context,
	reactive::{PropId, Store, prop::ItemId},
};

pub struct Effect<Ctx> {
	fun: Box<dyn FnMut(&mut Ctx)>,
	loc: &'static Location<'static>,
	read: Vec<ItemId>,
	write: Vec<ItemId>,
}
impl<Ctx> Debug for Effect<Ctx> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Effect")
			.field("loc", &self.loc)
			.field("read", &self.read)
			.field("write", &self.write)
			.finish()
	}
}

#[derive(Debug)]
pub struct Updater<Ctx> {
	effects: SlotMap<ItemId, Effect<Ctx>>,
	read_deps: FxHashMap<ItemId, SmallVec<[ItemId; 2]>>,
}
impl<Ctx> Default for Updater<Ctx> {
	fn default() -> Self {
		Self { effects: SlotMap::default(), read_deps: FxHashMap::default() }
	}
}
impl<Ctx: Context> Updater<Ctx> {
	pub(crate) fn remove_items(&mut self, effects: &[ItemId], props: &[ItemId]) {
		for prop in props {
			self.read_deps.remove(prop);
		}
		for id in effects {
			let effect = self.effects.remove(*id).unwrap();
			for read in &effect.read {
				self.read_deps.get_mut(read).map(|effects| effects.retain(|cur| *cur != *id));
			}
		}
	}

	pub(crate) fn add_effect(
		ctx: &mut Ctx, mut fun: impl FnMut(&mut Ctx) + 'static,
		deps: Option<(Vec<PropId<()>>, Vec<PropId<()>>)>, loc: &'static Location,
	) -> ItemId {
		if deps.is_none() {
			start_track_panicing(ctx.store_ref());
		}
		fun(ctx);

		let store = ctx.store();

		let (read, write) = match deps {
			Some((read, write)) => (read, write),
			_ => store.end_track().unwrap().destruct(),
		};
		let write = write.into_iter().map(|id| id.0).collect();
		let read = read.into_iter().map(|id| id.0).collect();

		let updater = &mut store.updater;
		let id = updater.effects.insert(Effect { fun: Box::new(fun), loc, write, read });

		for read in &updater.effects[id].read {
			updater.read_deps.entry(*read).or_default().push(id);
		}

		id
	}
}

pub fn start_track_panicing<Ctx: Context>(store: &Store<Ctx>) {
	if store.start_track().is_err() {
		panic!("requesting tracking while already tracking");
	}
}
