//! define item ids
use std::{
	fmt::{Debug, Display},
	hash::Hash,
	marker::PhantomData,
};

use slotmap::{Key, new_key_type};

new_key_type! {
	/// an id of an item used inside the reactivity system
	pub struct ItemId;
}

/// a typeaware id of a reactive property.
///
/// the `PropId` is a [`Copy`]able id used in accessing a specific property.
///
/// it is unique for the lifetime of the [`Store`](crate::Store), but not between different `Stores`.
///
/// it is created by [`Store::prop`](crate::Store::prop)
///
/// # example
/// ```
///	let count = store.prop(0);
/// assert_eq!(store.get(count), 0);
/// ```
#[repr(transparent)]
pub struct PropId<T: 'static>(pub(crate) ItemId, PhantomData<T>);
impl<T> PropId<T> {
	/// create a new `PropId` from a [`ItemId`].
	pub(crate) fn new(id: ItemId) -> Self {
		Self(id, PhantomData)
	}
	/// return the value of the `PropId`.
	///
	/// the value will be unique like the `PropId`, however it is not stable between versions.
	pub fn value(&self) -> u64 {
		self.0.data().as_ffi()
	}
	/// erase the type associated with the `TypeId`.
	///
	/// useful in grouping `PropId`s of different types like in [`Store::effect`](crate::Store::effect).
	pub fn erase_type(&self) -> PropId<()> {
		PropId(self.0, PhantomData)
	}
}
impl<T> Display for PropId<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:#x}", self.value())
	}
}
impl<T> Debug for PropId<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "PropId({:#})", self)
	}
}
impl<T> Clone for PropId<T> {
	fn clone(&self) -> Self {
		Self(self.0, self.1)
	}
}
impl<T> Copy for PropId<T> {}
impl<T> PartialEq for PropId<T> {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0
	}
}
impl<T> PartialOrd for PropId<T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.0.partial_cmp(&other.0)
	}
}
impl<T> Hash for PropId<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}
impl<T> Eq for PropId<T> {}
impl<T> Ord for PropId<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.0.cmp(&other.0)
	}
}

/// an id of a slab.
///
/// the `SlabId` is a [`Copy`]able id for a specific [`Store`](crate::Store) [slab](crate::Store#slab-managment).
///
/// created by [`Store::create_slab`](crate::Store::create_slab)
///
/// # example
/// ```
/// let slab = store.create_slab();
/// assert!(store.has_slab(slab));
/// let count = store.prop_in(slab, 0);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlabId(pub(crate) u64);
impl SlabId {
	/// return the value of the `SlabId`.
	pub fn value(&self) -> u64 {
		self.0 as u64
	}
}
impl Display for SlabId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if f.alternate() { write!(f, "{:#04x}", self.0) } else { write!(f, "{:04x}", self.0) }
	}
}
impl Debug for SlabId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "SlabId({:#})", self)
	}
}
