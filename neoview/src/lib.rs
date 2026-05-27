mod context;
mod prop;
mod store;
mod updater;

pub use neoview_macro::chunk;

pub use {
	context::{Context, GlobalStoreProv, LocalStoreProv, ScopedStoreProv, StoreProv},
	prop::{PropId, SlabId},
	store::{Store, TrackResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
	Removed,
	Tracking,
	NotTracking,
}
