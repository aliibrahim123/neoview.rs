use std::{
	fmt::{Debug, Display},
	hash::Hash,
	ops::{Deref, DerefMut},
};

use crate::reactive::{
	PropId, Store,
	prop::{Prop, PropStatus},
};

macro_rules! guard_common_impls {
	($guard:ident) => {
		impl<T> Deref for $guard<'_, T> {
			type Target = T;
			fn deref(&self) -> &Self::Target {
				self.value
			}
		}
		impl<T: Debug> Debug for $guard<'_, T> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				Debug::fmt(self.value, f)
			}
		}
		impl<T: Display> Display for $guard<'_, T> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				Display::fmt(self.value, f)
			}
		}
	};
}

pub struct ReadGuard<'scope, T> {
	store: &'scope Store,
	prop: &'scope Prop,
	value: &'scope T,
}
impl<'scope, T: 'static> ReadGuard<'scope, T> {
	pub(crate) fn new(store: &'scope Store, prop: &'scope Prop) -> Option<Self> {
		let value = prop.get()?;
		store.inc_ref();
		Some(Self { store, prop, value })
	}
}
guard_common_impls!(ReadGuard);
impl<T> Drop for ReadGuard<'_, T> {
	fn drop(&mut self) {
		self.prop.unref(false);
		self.store.dec_ref();
	}
}

pub struct MutGuard<'scope, T> {
	store: &'scope Store,
	prop: &'scope Prop,
	value: &'scope mut T,
}
impl<'scope, T: 'static> MutGuard<'scope, T> {
	pub(crate) fn new(store: &'scope Store, prop: &'scope Prop) -> Option<Self> {
		let value = prop.get_mut()?;
		store.inc_ref();
		Some(Self { store, prop, value })
	}
}
guard_common_impls!(MutGuard);
impl<T> DerefMut for MutGuard<'_, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.value
	}
}
impl<T> Drop for MutGuard<'_, T> {
	fn drop(&mut self) {
		self.prop.unref(true);
		self.store.dec_ref();
	}
}

pub trait SignalBase<T: 'static> {
	fn store(&self) -> &Store;
	fn prop(&self) -> PropId<T>;
	fn status(&self) -> PropStatus {
		self.store().status_of(self.prop())
	}
}
pub trait ReadableSignal<T: 'static>: SignalBase<T> {
	fn peek(&self) -> ReadGuard<'_, T> {
		self.store().peek(self.prop())
	}
	fn read(&self) -> ReadGuard<'_, T> {
		self.store().read(self.prop())
	}
	fn get(&self) -> T
	where
		T: Copy,
	{
		*self.store().read(self.prop())
	}
	fn track_read(&self) {
		self.store().track_read(self.prop());
	}
}
pub trait WritableSignal<T: 'static>: SignalBase<T> {
	fn write(&self, value: T) {
		self.store().write(self.prop(), value)
	}
	fn update(&self, fun: impl FnOnce(&mut T)) {
		fun(self.store().read_mut(self.prop()).deref_mut())
	}
	fn track_write(&self) {
		self.store().track_write(self.prop());
	}
	fn force_update(&self) {
		self.store().force_update(self.prop())
	}
}

macro_rules! signal_common_impl {
	($type:ident) => {
		impl<T> Debug for $type<'_, T> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "{}({:#})", stringify!($type), self.prop)
			}
		}
		impl<T> PartialEq for $type<'_, T> {
			fn eq(&self, other: &Self) -> bool {
				self.store == other.store && self.prop == self.prop
			}
		}
		impl<T> Eq for $type<'_, T> {}
		impl<T> Hash for $type<'_, T> {
			fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
				(self.store as *const Store).hash(state);
				self.prop.hash(state);
			}
		}
		impl<T: 'static> SignalBase<T> for $type<'_, T> {
			fn store(&self) -> &Store {
				self.store
			}
			fn prop(&self) -> PropId<T> {
				self.prop
			}
		}
	};
}

#[derive(Clone, Copy)]
pub struct Signal<'scope, T: 'static> {
	pub(crate) store: &'scope Store,
	pub(crate) prop: PropId<T>,
}
signal_common_impl!(Signal);
impl<T: 'static> ReadableSignal<T> for Signal<'_, T> {}
impl<T: 'static> WritableSignal<T> for Signal<'_, T> {}
impl<'scope, T: 'static> Signal<'scope, T> {
	pub fn read_mut(&self) -> MutGuard<'scope, T> {
		self.store.read_mut(self.prop)
	}
	pub fn as_readonly(&self) -> ROSignal<'scope, T> {
		ROSignal { store: self.store, prop: self.prop }
	}
	pub fn as_writeonly(&self) -> WOSignal<'scope, T> {
		WOSignal { store: self.store, prop: self.prop }
	}
}

#[derive(Clone, Copy)]
pub struct ROSignal<'scope, T: 'static> {
	pub(crate) store: &'scope Store,
	pub(crate) prop: PropId<T>,
}
signal_common_impl!(ROSignal);
impl<T: 'static> ReadableSignal<T> for ROSignal<'_, T> {}

#[derive(Clone, Copy)]
pub struct WOSignal<'scope, T: 'static> {
	pub(crate) store: &'scope Store,
	pub(crate) prop: PropId<T>,
}
signal_common_impl!(WOSignal);
impl<T: 'static> WritableSignal<T> for WOSignal<'_, T> {}
