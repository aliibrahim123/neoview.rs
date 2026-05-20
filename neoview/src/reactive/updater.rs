use std::{fmt::Debug, mem::transmute, panic::Location};

use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::reactive::{PropId, Store, prop::ItemId};

pub struct Effect {
	pub fun: Box<dyn FnMut()>,
	pub loc: &'static Location<'static>,
	pub read: Vec<ItemId>,
	pub write: Vec<ItemId>,
}
impl Debug for Effect {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Effect")
			.field("loc", &self.loc)
			.field("read", &self.read)
			.field("write", &self.write)
			.finish()
	}
}

#[derive(Debug, Default)]
pub struct Updater {
	effects: SlotMap<ItemId, Effect>,
	read_deps: FxHashMap<ItemId, SmallVec<[ItemId; 2]>>,
}
impl Updater {
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

	pub(crate) fn add_effect<'store>(
		&'store mut self, store: &'store Store, mut fun: impl FnMut() + 'store,
		deps: Option<(Vec<PropId<()>>, Vec<PropId<()>>)>, loc: &'static Location,
	) -> ItemId {
		if deps.is_none() {
			start_track_panicing(store);
		}
		fun();

		let fun = Box::new(fun);
		let fun =
			unsafe { transmute::<Box<dyn FnMut() + 'store>, Box<dyn FnMut() + 'static>>(fun) };

		let (read, write) = match deps {
			Some((read, write)) => (read, write),
			_ => store.end_track().unwrap().destruct(),
		};
		let write = write.into_iter().map(|id| id.0).collect();
		let read = read.into_iter().map(|id| id.0).collect();

		let id = self.effects.insert(Effect { fun, loc, write, read });

		for read in &self.effects[id].read {
			self.read_deps.entry(*read).or_default().push(id);
		}

		id
	}
}

pub fn start_track_panicing(store: &Store) {
	if store.start_track().is_err() {
		panic!("requesting tracking while already tracking");
	}
}
