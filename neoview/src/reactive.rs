pub(crate) mod prop;
pub(crate) mod signal;
pub(crate) mod slab;
pub(crate) mod store;
pub use {
	prop::{PropId, PropIndex, SlabId},
	store::Store,
};
