//! define item ids
use std::{
	any::{Any, TypeId},
	fmt::{Debug, Display},
	hash::Hash,
	marker::PhantomData,
};

use slotmap::{Key, new_key_type};

new_key_type! {
	/// an id of an item used inside the reactivity system
	pub struct ItemId;
}

/// a typeaware unique identifier of a reactive property.
///
/// the `PropId` is a [`Copy`]able id used in accessing a specific property.
///
/// it is unique for the lifetime of the [`Store`](crate::Store), but not between different `Stores`.
///
/// it is created by [`Store::prop`](crate::Store::prop)
///
/// # example
/// ```
///    let count = store.prop(0);
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
		*self
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
		Some(self.cmp(other))
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
		self.0
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

/// the property value storage unit.
///
/// optimized for primitive.
#[derive(Debug)]
pub enum PropValue {
	Unit(()),
	Bool(bool),
	Char(char),
	U8(u8),
	U16(u16),
	U32(u32),
	U64(u64),
	USize(usize),
	I8(i8),
	I16(i16),
	I32(i32),
	I64(i64),
	ISize(isize),
	F32(f32),
	F64(f64),
	Str(String),
	Any(Box<dyn Any>),
}
impl PropValue {
	/// create a new `PropValue` from a value.
	pub fn new<T: 'static>(value: T) -> Self {
		let id = TypeId::of::<T>();

		macro_rules! try_cast {
			($ty:ty, $var:ident) => {
				if id == TypeId::of::<$ty>() {
					unsafe {
						let casted = std::ptr::read(&value as *const T as *const $ty);
						std::mem::forget(value);
						return PropValue::$var(casted);
					}
				}
			};
		}

		try_cast!((), Unit);
		try_cast!(bool, Bool);
		try_cast!(char, Char);
		try_cast!(u8, U8);
		try_cast!(u16, U16);
		try_cast!(u32, U32);
		try_cast!(u64, U64);
		try_cast!(usize, USize);
		try_cast!(i8, I8);
		try_cast!(i16, I16);
		try_cast!(i32, I32);
		try_cast!(i64, I64);
		try_cast!(isize, ISize);
		try_cast!(f32, F32);
		try_cast!(f64, F64);
		try_cast!(String, Str);

		PropValue::Any(Box::new(value))
	}
	/// get the value of the `PropValue`.
	pub fn get<T: 'static>(&self) -> &T {
		let id = TypeId::of::<T>();

		macro_rules! try_get {
			($ty:ty, $var:ident) => {
				if id == TypeId::of::<$ty>() {
					match self {
						PropValue::$var(val) => {
							return unsafe { &*(val as *const $ty as *const T) };
						}
						_ => unreachable!(),
					}
				}
			};
		}

		try_get!((), Unit);
		try_get!(bool, Bool);
		try_get!(char, Char);
		try_get!(u8, U8);
		try_get!(u16, U16);
		try_get!(u32, U32);
		try_get!(u64, U64);
		try_get!(usize, USize);
		try_get!(i8, I8);
		try_get!(i16, I16);
		try_get!(i32, I32);
		try_get!(i64, I64);
		try_get!(isize, ISize);
		try_get!(f32, F32);
		try_get!(f64, F64);
		try_get!(String, Str);

		match self {
			PropValue::Any(val) => val.downcast_ref::<T>().unwrap(),
			_ => unreachable!(),
		}
	}
	/// get the mutable value of the `PropValue`.
	pub fn get_mut<T: 'static>(&mut self) -> &mut T {
		let id = TypeId::of::<T>();

		macro_rules! try_get_mut {
			($ty:ty, $var:ident) => {
				if id == TypeId::of::<$ty>() {
					match self {
						PropValue::$var(val) => {
							return unsafe { &mut *(val as *mut $ty as *mut T) };
						}
						_ => unreachable!(),
					}
				}
			};
		}

		try_get_mut!((), Unit);
		try_get_mut!(bool, Bool);
		try_get_mut!(char, Char);
		try_get_mut!(u8, U8);
		try_get_mut!(u16, U16);
		try_get_mut!(u32, U32);
		try_get_mut!(u64, U64);
		try_get_mut!(usize, USize);
		try_get_mut!(i8, I8);
		try_get_mut!(i16, I16);
		try_get_mut!(i32, I32);
		try_get_mut!(i64, I64);
		try_get_mut!(isize, ISize);
		try_get_mut!(f32, F32);
		try_get_mut!(f64, F64);
		try_get_mut!(String, Str);

		match self {
			PropValue::Any(val) => val.downcast_mut::<T>().unwrap(),
			_ => unreachable!(),
		}
	}
}
