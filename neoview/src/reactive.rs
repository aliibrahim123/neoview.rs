pub(crate) mod prop;
pub(crate) mod store;
pub(crate) mod updater;
pub use {
	prop::{PropId, SlabId},
	store::{Store, TrackResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
	Removed,
	Tracking,
	NotTracking,
}
