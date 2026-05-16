use std::{
	fmt::{Debug, Display},
	ops::{Deref, DerefMut},
};

use crate::reactive::{Store, prop::Prop};

macro_rules! common_guard_impls {
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
common_guard_impls!(ReadGuard);
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
common_guard_impls!(MutGuard);
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
