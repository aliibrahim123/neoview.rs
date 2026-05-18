use std::{
	any::Any,
	cell::{Cell, UnsafeCell},
	fmt::{Debug, Display},
	hash::Hash,
	marker::PhantomData,
};

pub struct PropId<T: 'static>(u64, PhantomData<T>);
impl<T> PropId<T> {
	pub(crate) fn new(slab: u64, prop: u16) -> Self {
		Self(slab << 16 | prop as u64, PhantomData)
	}
	pub fn value(&self) -> u64 {
		self.0
	}
	pub fn slab(&self) -> SlabId {
		SlabId(self.0 >> 16)
	}
	pub fn prop_index(&self) -> PropIndex {
		PropIndex((self.0 & 0xFFFF) as u16)
	}
	pub(crate) fn split(&self) -> (SlabId, PropIndex) {
		(self.slab(), self.prop_index())
	}
}
impl<T> Display for PropId<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let (slab, prop) = self.split();
		if f.alternate() { write!(f, "0x{slab}_{prop}") } else { write!(f, "{slab}_{prop}") }
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropIndex(u16);
impl PropIndex {
	pub fn value(&self) -> u64 {
		self.0 as u64
	}
}
impl Display for PropIndex {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if f.alternate() { write!(f, "{:#04x}", self.0) } else { write!(f, "{:04x}", self.0) }
	}
}
impl Debug for PropIndex {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "PropIndex({:#})", self)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlabId(pub(crate) u64);
impl SlabId {
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

#[derive(Debug)]
pub struct Prop {
	value: UnsafeCell<Box<dyn Any>>,
	pending_value: UnsafeCell<Option<Box<dyn Any>>>,
	ref_count: Cell<u32>,
	in_mut: Cell<bool>,
}
impl Prop {
	pub fn new<T: 'static>(value: T) -> Self {
		Self {
			value: UnsafeCell::new(Box::new(value)),
			pending_value: UnsafeCell::new(None),
			ref_count: Cell::new(0),
			in_mut: Cell::new(false),
		}
	}
	pub fn get<T: 'static>(&self) -> Option<&T> {
		if self.in_mut.get() {
			return None;
		}
		self.ref_count.update(|c| c + 1);
		unsafe { &*self.value.get() }.downcast_ref()
	}
	pub fn set<T: 'static>(&self, value: T) {
		if self.ref_count.get() == 0 {
			*unsafe { &mut *self.value.get() }.downcast_mut().unwrap() = value
		} else {
			let pending = unsafe { &mut *self.pending_value.get() };
			if let Some(pending) = pending {
				*pending.downcast_mut().unwrap() = value
			} else {
				*pending = Some(Box::new(value));
			}
		}
	}
	pub fn get_mut<T: 'static>(&self) -> Option<&mut T> {
		if self.ref_count.get() != 0 {
			return None;
		}
		self.in_mut.set(true);
		self.ref_count.update(|c| c + 1);
		unsafe { &mut *self.value.get() }.downcast_mut()
	}
	pub fn unref(&self, is_mut: bool) {
		if is_mut {
			self.in_mut.set(false)
		}
		self.ref_count.update(|c| c - 1);
		if self.ref_count.get() == 0
			&& let Some(pending) = unsafe { &mut *self.pending_value.get() }.take()
		{
			*unsafe { &mut *self.value.get() } = pending;
		}
	}
}
