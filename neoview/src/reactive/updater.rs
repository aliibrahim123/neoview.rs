use std::{cell::UnsafeCell, fmt::Debug, panic::Location};

use rustc_hash::{FxHashMap, FxHashSet};
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::{
	context::Context,
	reactive::{PropId, Store, prop::ItemId},
};

pub struct Effect<Ctx> {
	fun: Option<Box<dyn FnMut(&mut Ctx)>>,
	read: Vec<ItemId>,
	write: Vec<ItemId>,
}
impl<Ctx> Debug for Effect<Ctx> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Effect")
			.field("read", &self.read)
			.field("write", &self.write)
			.finish()
	}
}

#[derive(Debug)]
pub struct Updater<Ctx> {
	effects: SlotMap<ItemId, Effect<Ctx>>,
	read_deps: FxHashMap<ItemId, SmallVec<[ItemId; 2]>>,

	pub is_updating: bool,
	pub dirty_props: Vec<ItemId>,
}
impl<Ctx> Default for Updater<Ctx> {
	fn default() -> Self {
		Self {
			effects: SlotMap::default(),
			read_deps: FxHashMap::default(),
			is_updating: false,
			dirty_props: Vec::new(),
		}
	}
}
impl<Ctx: Context> Updater<Ctx> {
	pub fn push_update(&mut self, id: ItemId) {
		if !(self.is_updating || self.dirty_props.contains(&id)) {
			self.dirty_props.push(id);
		}
	}

	pub fn add_effect(
		ctx: &mut Ctx, mut fun: impl FnMut(&mut Ctx) + 'static,
		deps: Option<(Vec<PropId<()>>, Vec<PropId<()>>)>, run_fn: bool,
	) -> ItemId {
		if deps.is_none() {
			start_track_panicing(ctx.store_ref());
		}
		if run_fn {
			fun(ctx)
		}

		let store = ctx.store();

		let (read, write) = match deps {
			Some((read, write)) => (read, write),
			_ => store.end_track().unwrap().destruct(),
		};
		let write = write.into_iter().map(|id| id.0).collect();
		let read = read.into_iter().map(|id| id.0).collect();

		let updater = &mut store.updater;
		let id = updater.effects.insert(Effect { fun: Some(Box::new(fun)), write, read });

		for &read in &store.updater.effects[id].read {
			store.updater.read_deps.entry(read).or_default().push(id);
		}

		id
	}

	pub fn update(ctx: &mut Ctx) {
		ctx.store().updater.is_updating = true;
		while !ctx.store().updater.dirty_props.is_empty() {
			let updater = &mut ctx.store().updater;

			let mut to_run = Vec::new();
			let mut visited = FxHashSet::default();
			let mut visiting = Vec::new();
			for &prop in &updater.dirty_props {
				visit(updater, prop, &mut to_run, &mut visited, &mut visiting);
			}

			updater.dirty_props.clear();
			for &id in to_run.iter().rev() {
				let mut fun = ctx.store().updater.effects[id].fun.take().unwrap();
				fun(ctx);
				ctx.store().updater.effects[id].fun = Some(fun);
			}
		}
		ctx.store().updater.is_updating = false;

		fn visit<Ctx: Context>(
			updater: &Updater<Ctx>, prop: ItemId, to_run: &mut Vec<ItemId>,
			visited: &mut FxHashSet<ItemId>, visiting: &mut Vec<ItemId>,
		) {
			let Some(effects) = updater.read_deps.get(&prop) else { return };
			for &effect in effects {
				if visiting.contains(&effect) {
					panic!("detected circular dependency in an update");
				}
				if visited.contains(&effect) {
					continue;
				}

				visiting.push(effect);
				for &prop in &updater.effects[effect].write {
					visit(updater, prop, to_run, visited, visiting);
				}
				visiting.pop();

				to_run.push(effect);
				visited.insert(effect);
			}
		}
	}

	pub fn remove_items(&mut self, effects: &[ItemId], props: &[ItemId]) {
		for prop in props {
			self.read_deps.remove(prop);
		}
		for id in effects {
			let effect = self.effects.remove(*id).unwrap();
			for read in &effect.read {
				if let Some(effects) = self.read_deps.get_mut(read) {
					effects.retain(|cur| *cur != *id);
					if effects.is_empty() {
						self.read_deps.remove(read);
					}
				}
			}
		}
	}
}

pub fn start_track_panicing<Ctx: Context>(store: &Store<Ctx>) {
	if store.start_track().is_err() {
		panic!("requesting tracking while already tracking");
	}
}
