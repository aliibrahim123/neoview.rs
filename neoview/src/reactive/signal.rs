use std::{
	fmt::{Debug, Display},
	hash::Hash,
	ops::{Deref, DerefMut},
};

use crate::{
	context::Context,
	reactive::{
		PropId, Store,
		prop::{Prop, PropStatus},
	},
};

macro_rules! guard_common_impls {
	($guard:ident) => {
		impl<T, Ctx: Context> Deref for $guard<'_, T, Ctx> {
			type Target = T;
			fn deref(&self) -> &Self::Target {
				self.value
			}
		}
		impl<T: Debug, Ctx: Context> Debug for $guard<'_, T, Ctx> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				Debug::fmt(self.value, f)
			}
		}
		impl<T: Display, Ctx: Context> Display for $guard<'_, T, Ctx> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				Display::fmt(self.value, f)
			}
		}
	};
}

pub struct ReadGuard<'scope, T, Ctx: Context> {
	store: &'scope Store<Ctx>,
	prop: &'scope Prop,
	value: &'scope T,
}
impl<'scope, T: 'static, Ctx: Context> ReadGuard<'scope, T, Ctx> {
	pub(crate) fn new(store: &'scope Store<Ctx>, prop: &'scope Prop) -> Option<Self> {
		let value = prop.get()?;
		store.inc_ref();
		Some(Self { store, prop, value })
	}
}
guard_common_impls!(ReadGuard);
impl<T, Ctx: Context> Drop for ReadGuard<'_, T, Ctx> {
	fn drop(&mut self) {
		self.prop.unref(false);
		self.store.dec_ref();
	}
}

pub struct MutGuard<'scope, T, Ctx: Context> {
	store: &'scope Store<Ctx>,
	prop: &'scope Prop,
	value: &'scope mut T,
}
impl<'scope, T: 'static, Ctx: Context> MutGuard<'scope, T, Ctx> {
	pub(crate) fn new(store: &'scope Store<Ctx>, prop: &'scope Prop) -> Option<Self> {
		let value = prop.get_mut()?;
		store.inc_ref();
		Some(Self { store, prop, value })
	}
}
guard_common_impls!(MutGuard);
impl<T, Ctx: Context> DerefMut for MutGuard<'_, T, Ctx> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.value
	}
}
impl<T, Ctx: Context> Drop for MutGuard<'_, T, Ctx> {
	fn drop(&mut self) {
		self.prop.unref(true);
		self.store.dec_ref();
	}
}

pub trait SignalBase<T: 'static, Ctx: Context> {
	fn store(&self) -> &Store<Ctx>;
	fn prop(&self) -> PropId<T>;
	fn status(&self) -> PropStatus {
		self.store().status_of(self.prop())
	}
}
pub trait ReadableSignal<T: 'static, Ctx: Context>: SignalBase<T, Ctx> {
	fn peek(&self) -> ReadGuard<'_, T, Ctx> {
		self.store().peek(self.prop())
	}
	fn get(&self) -> ReadGuard<'_, T, Ctx> {
		self.store().get(self.prop())
	}
	fn track_read(&self) {
		self.store().track_read(self.prop());
	}
}
pub trait WritableSignal<T: 'static, Ctx: Context>: SignalBase<T, Ctx> {
	fn set(&self, value: T) {
		self.store().set(self.prop(), value)
	}
	fn update(&self, fun: impl FnOnce(&mut T)) {
		fun(self.store().get_mut(self.prop()).deref_mut())
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
		impl<T, Ctx: Context> Debug for $type<'_, T, Ctx> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "{}({:#})", stringify!($type), self.prop)
			}
		}
		impl<T, Ctx: Context> PartialEq for $type<'_, T, Ctx> {
			fn eq(&self, other: &Self) -> bool {
				self.store == other.store && self.prop == self.prop
			}
		}
		impl<T, Ctx: Context> Eq for $type<'_, T, Ctx> {}
		impl<T, Ctx: Context> Hash for $type<'_, T, Ctx> {
			fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
				(self.store as *const Store<Ctx>).hash(state);
				self.prop.hash(state);
			}
		}
		impl<T: 'static, Ctx: Context> SignalBase<T, Ctx> for $type<'_, T, Ctx> {
			fn store(&self) -> &Store<Ctx> {
				self.store
			}
			fn prop(&self) -> PropId<T> {
				self.prop
			}
		}
	};
}

#[derive(Clone, Copy)]
pub struct Signal<'scope, T: 'static, Ctx: Context> {
	pub(crate) store: &'scope Store<Ctx>,
	pub(crate) prop: PropId<T>,
}
signal_common_impl!(Signal);
impl<T: 'static, Ctx: Context> ReadableSignal<T, Ctx> for Signal<'_, T, Ctx> {}
impl<T: 'static, Ctx: Context> WritableSignal<T, Ctx> for Signal<'_, T, Ctx> {}
impl<'scope, T: 'static, Ctx: Context> Signal<'scope, T, Ctx> {
	pub fn get_mut(&self) -> MutGuard<'scope, T, Ctx> {
		self.store.get_mut(self.prop)
	}
	pub fn as_readonly(&self) -> ROSignal<'scope, T, Ctx> {
		ROSignal { store: self.store, prop: self.prop }
	}
	pub fn as_writeonly(&self) -> WOSignal<'scope, T, Ctx> {
		WOSignal { store: self.store, prop: self.prop }
	}
}

#[derive(Clone, Copy)]
pub struct ROSignal<'scope, T: 'static, Ctx: Context> {
	pub(crate) store: &'scope Store<Ctx>,
	pub(crate) prop: PropId<T>,
}
signal_common_impl!(ROSignal);
impl<T: 'static, Ctx: Context> ReadableSignal<T, Ctx> for ROSignal<'_, T, Ctx> {}

#[derive(Clone, Copy)]
pub struct WOSignal<'scope, T: 'static, Ctx: Context> {
	pub(crate) store: &'scope Store<Ctx>,
	pub(crate) prop: PropId<T>,
}
signal_common_impl!(WOSignal);
impl<T: 'static, Ctx: Context> WritableSignal<T, Ctx> for WOSignal<'_, T, Ctx> {}
