pub(crate) mod prop;
pub(crate) mod signal;
pub(crate) mod slab;
pub(crate) mod store;
pub(crate) mod updater;
pub use {
	prop::{PropId, PropStatus, SlabId},
	signal::{
		MutGuard, ROSignal, ReadGuard, ReadableSignal, Signal, SignalBase, WOSignal, WritableSignal,
	},
	slab::Slab,
	store::{Store, TrackResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
	Removed,
	LiveRefs,
	UnderMut,
	Tracking,
	NotTracking,
}
pub(crate) fn struct_change_while_life_refs() -> ! {
	panic!("can not do a structural change while there is live references")
}
