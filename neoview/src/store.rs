//! Defines the [`Store`]

use std::{cell::RefCell, fmt::Debug, ops::DerefMut, ptr};

use rustc_hash::FxHashMap;
use slotmap::SlotMap;

use crate::{
	Error, PropId, SlabId,
	context::Context,
	prop::{ItemId, PropValue},
	updater::{Updater, start_track_panicing},
};

type Cleaner<Ctx> = Box<dyn FnOnce(&mut Ctx)>;

/// Stores items owned by a slab.
pub struct SlabData<Ctx> {
	pub props: Vec<ItemId>,
	pub effects: Vec<ItemId>,
	pub cleaner: Vec<Cleaner<Ctx>>,
}
impl<Ctx> Default for SlabData<Ctx> {
	fn default() -> Self {
		Self { props: Vec::new(), effects: Vec::new(), cleaner: Vec::new() }
	}
}
impl<Ctx> Debug for SlabData<Ctx> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SlabData")
			.field("props", &self.props)
			.field("effects", &self.effects)
			.finish()
	}
}

/// The result of a tracking operation.
///
/// Produced when the tracking operation is ended with [`Store::end_track`].
///
/// # Example
/// ```
/// let a = store.prop(1);
/// let b = store.prop(2);
/// store.start_track();
/// store.set(b, store.get(a) + 2);
/// let result = store.end_track();
/// assert_eq!(result.read, [a.erase_type()]);
/// assert_eq!(result.written, [b.erase_type()]);
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct TrackResult {
	/// The properties read.
	pub read: Vec<PropId<()>>,
	/// The properties written to.
	pub written: Vec<PropId<()>>,
}
impl TrackResult {
	/// Destructures the `TrackResult` into a `(read, written)` pair.
	pub(crate) fn destruct(self) -> (Vec<PropId<()>>, Vec<PropId<()>>) {
		(self.read, self.written)
	}
}

/// The container of the reactivity system.
///
/// The `Store` is the structure that owns and manages the entire reactivity system, including its [properties](#property-management) and [effects](#effects).
///
/// It is tightly coupled to a specific owning [`Context`], and its lifetime is identical to it.
///
/// Every interaction with the reactive system requires mutable access to the `Store`, however, all common operations are redirected through the family of [`StoreProv`](crate::StoreProv)ider traits.
///
/// ## Example
/// ```
/// let count = store.prop(0);
/// assert_eq!(store.get(count), 0);
/// Store::effect(ctx, move |ctx| println!("count: {}", ctx.get(count)));
/// store.set(count, 1);
/// assert_eq!(store.get(count), 1);
///
/// let doubled = Store::computed(ctx, move |ctx| ctx.get(count) * 2);
/// store.update(count, |cnt| *cnt += 1);
/// assert_eq!(store.get(doubled), 4);
/// Store::flush_updates(ctx); // => count: 2
/// ```
///
/// # Sections
/// Due to the large API surface exposed by the `Store`, its documentation has been split into multiple parts:
///
/// - [Property Management](#property-management)
/// - [Property Access](#property-access)
/// - [Safe Property Access](#safe-property-access)
/// - [Effects](#effects)
/// - [Slab Management](#slab-management)
/// - [Updating](#updating)
/// - [Tracking](#tracking)
/// - [Store Management](#store-management)
pub struct Store<Ctx> {
	pub(crate) props: SlotMap<ItemId, PropValue>,

	pub(crate) slabs: FxHashMap<SlabId, SlabData<Ctx>>,
	/// The `SlabId` of the next slab to be added.
	next_slab: SlabId,
	/// Slabs removed during an update to be deleted at the end of that update.
	slabs_to_remove: Vec<SlabId>,

	pub(crate) updater: Updater<Ctx>,

	global_cleaners: Vec<Cleaner<Ctx>>,
	is_dropped: bool,

	// `RefCell` so that `read` doesn't require a mutable reference.
	tracking: RefCell<Option<TrackResult>>,
}
impl<Ctx: Context> Default for Store<Ctx> {
	fn default() -> Self {
		Store {
			props: SlotMap::default(),
			slabs: FxHashMap::default(),
			next_slab: SlabId(0),
			slabs_to_remove: Vec::new(),
			updater: Updater::default(),
			global_cleaners: Vec::new(),
			is_dropped: false,
			tracking: RefCell::new(None),
		}
	}
}
impl<Ctx> Debug for Store<Ctx> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Store")
			.field("props", &self.props)
			.field("slabs", &self.slabs)
			.field("slabs_to_remove", &self.slabs_to_remove)
			.field("effects", &self.updater.effects)
			.field("tracking", &self.tracking)
			.field("is_dropped", &self.is_dropped)
			.finish()
	}
}
impl<Ctx> PartialEq for Store<Ctx> {
	fn eq(&self, other: &Self) -> bool {
		ptr::eq(self, other)
	}
}
impl<Ctx> Eq for Store<Ctx> {}

/// <h2 id=property-management>Property Management</h2>
///
/// Reactive property management is the primary purpose of the `Store`.
///
/// A reactive property is any value used within the reactivity system. It can be of any type that does not contain non-`'static` references, and it is owned by the `Store`.
///
/// A property is created by [`prop`](Store::prop), identified by a [`PropId`], accessed by the [property access methods](#property-access), and can be bound to and by multiple [effects](#effects).
///
/// Individual properties cannot be removed directly, they can only be removed along with the `Store` or their owning slab.
impl<Ctx: Context> Store<Ctx> {
	/// Defines a new reactive property in the global scope.
	///
	/// It accepts the property's initial `value` and returns its [`PropId`].
	///
	/// # Example
	/// ```
	/// let count = store.prop(0);
	/// let text = store.prop("hello".to_string());
	/// struct Value { a: i32, b: f64, c: String, d: Vec<u8> }
	/// let value = store.prop(Value {
	///     a: 1, b: 1.5, c: "abc".to_string(), d: vec![1, 2, 3],
	/// });
	/// ```
	pub fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let id = self.props.insert(PropValue::new(value));
		PropId::new(id)
	}

	/// Defines a new reactive property in a specific scope.
	///
	/// It accepts the target `slab` and the property's initial `value`, and returns its [`PropId`].
	///
	/// If `slab` is [`None`], the property is defined in the global scope.
	///
	/// # Example
	/// ```
	/// let slab = store.create_slab();
	/// let count = store.prop_in(Some(slab), 0);
	/// let text = store.prop_in(Some(slab), "hello".to_string());
	///
	/// // The following two are equivalent:
	/// let nb = store.prop(1.5);
	/// let nb = store.prop_in(None, 1.5);
	/// ```
	pub fn prop_in<T: 'static>(
		&mut self, slab: Option<SlabId>, value: T,
	) -> Result<PropId<T>, Error> {
		// global prop
		let Some(slab) = slab else {
			return Ok(self.prop(value));
		};
		if !self.has_slab(slab) {
			return Err(Error::Removed);
		}
		let id = self.prop(value);
		self.slab(slab).props.push(id.0);
		Ok(id)
	}

	/// Checks whether a reactive property exists inside the `Store`.
	///
	/// # Example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert!(store.contains(nb));
	/// Store::remove_slab(ctx, slab);
	/// assert!(!store.contains(nb));
	/// ```
	pub fn contains<T>(&self, id: PropId<T>) -> bool {
		self.props.contains_key(id.0)
	}
}

/// <h2 id=property-access>Property Access</h2>
///
/// The property access methods are functions that read and mutate the reactive properties defined inside the `Store`.
///
/// They are of two kinds:
/// - **Reading methods**: [`read`](Store::read), [`get`](Store::get), and [`peek`](Store::peek).
/// - **Mutating methods**: [`write`](Store::write), [`read_mut`](Store::read_mut), and [`update`](Store::update).
///
/// ## Example
/// ```
/// let nb = store.prop(1);
/// assert_eq!(store.read(nb), 1);
///
/// store.write(nb, 2);
/// assert_eq!(store.get(nb), 2);
///
/// *store.get_mut(nb) += 1;
/// store.update(nb, |v| *v += 1);
/// assert_eq!(store.peek(nb), 4);
/// ```
///
/// These methods trigger [updates](#updating) and can be [tracked](#tracking).
///
/// These methods are redirected by every [`StoreProv`](crate::StoreProv)ider.
///
/// These methods are designed to be ergonomic and will panic on errors. For a non-panicking, fallible version, see [safe property access](#safe-property-access).
impl<Ctx: Context> Store<Ctx> {
	/// Returns a reference to a reactive property's value.
	///
	/// `read` is the primitive for reading properties, it triggers a read signal while tracking.
	///
	/// It will panic if the given property is removed. For a safe version, see [`try_read`](Store::try_read).
	///
	/// # Example
	/// ```
	/// let nb = store.prop(1);
	/// let text = store.prop(String::from("abc"));
	/// let arr = store.prop(vec![1, 2, 3]);
	///
	/// assert_eq!(store.read(nb), 1);
	/// assert_eq!(store.read(text), "abc");
	/// assert_eq!(store.read(arr)[1], 2);
	/// ```
	pub fn read<T: 'static>(&self, prop: PropId<T>) -> &T {
		self.track_read(prop);
		self.peek(prop)
	}

	/// Returns a copy of a [`Copy`]able reactive property's value.
	///
	/// It triggers a read signal while tracking and will panic if the given property is removed. For a safe version, see [`try_get`](Store::try_get).
	///
	/// # Example
	/// ```
	/// let nb = store.prop(1);
	/// let cond = store.prop(true);
	///
	/// assert_eq!(store.get(nb), 1);
	/// assert_eq!(store.get(cond), true);
	/// ```
	pub fn get<T: 'static + Copy>(&self, prop: PropId<T>) -> T {
		*self.read(prop)
	}

	/// Returns a reference to a reactive property's value without tracking it.
	///
	/// This method is identical to [`read`](Store::read), except it doesn't trigger a read signal.
	///
	/// It will panic if the given property is removed. For a safe version, see [`try_peek`](Store::try_peek).
	///
	/// # Example
	/// ```
	/// let a = store.prop(1);
	/// let b = store.prop(2);
	///
	/// store.start_track();
	/// store.read(a);
	/// store.peek(b);
	///
	/// assert_eq!(store.end_track().unwrap().read, [a.erase_type()]);
	/// ```
	pub fn peek<T: 'static>(&self, prop: PropId<T>) -> &T {
		self.try_peek(prop).expect("reading removed property")
	}

	/// Sets a reactive property's value.
	///
	/// This method triggers an update, triggers a write signal while tracking, and returns the previous value of the property.
	///
	/// It will panic if the given property is removed. For a safe version, see [`try_write`](Store::try_write).
	///
	/// # Example
	/// ```
	/// let nb = store.prop(1);
	/// let text = store.prop(String::from("abc"));
	///
	/// store.write(nb, 2);
	/// assert_eq!(store.get(nb), 2);
	///
	/// assert_eq!(store.write(text, String::from("abcd")), "abc");
	/// ```
	pub fn write<T: 'static>(&mut self, prop: PropId<T>, value: T) -> T {
		self.try_write(prop, value).expect("writing removed property")
	}

	/// Returns a mutable reference to a reactive property's value.
	///
	/// This method is the primitive for mutating properties. It triggers an update and triggers a write signal while tracking.
	///
	/// It will panic if the given property is removed. For a safe version, see [`try_read_mut`](Store::try_read_mut).
	///
	/// # Example
	/// ```
	/// let arr = store.prop(vec![1, 2, 3]);
	/// store.read_mut(arr)[2] = 4;
	/// assert_eq!(store.read(arr), [1, 2, 4]);
	/// ```
	pub fn read_mut<T: 'static>(&mut self, prop: PropId<T>) -> &mut T {
		self.try_read_mut(prop).expect("mutating removed property")
	}

	/// Updates a reactive property using an updater function.
	///
	/// The `fun`ction is called with a mutable reference to the property's value. The method triggers an update and triggers a write signal while tracking.
	///
	/// It will panic if the given property is removed. For a safe version, see [`try_update`](Store::try_update).
	///
	/// # Example
	/// ```
	/// let nb = store.prop(1);
	/// store.update(nb, |v| *v += 1);
	/// assert_eq!(store.get(nb), 2);
	/// ```
	pub fn update<T: 'static>(&mut self, prop: PropId<T>, fun: impl FnOnce(&mut T)) {
		self.try_update(prop, fun).expect("updating removed property");
	}

	/// Returns mutable references to multiple reactive property's values at once.
	///
	/// It triggers an update and triggers a write signal while tracking for all properties.
	///
	/// It will panic if any given property is removed, or if the properties are not disjoint (duplicated [`PropId`]s where given). For a safe version, see [`try_read_disjoint_mut`](Store::try_read_disjoint_mut).
	///
	/// # Example
	/// ```
	/// let a = store.prop(1);
	/// let b = store.prop(2);
	/// let c = store.prop(3);
	///
	/// let (a_val, b_val, c_val) = store.read_disjoint_mut((a, b, c));
	/// *a_val += 1;
	/// *b_val += 1;
	/// *c_val += 1;
	///
	/// assert_eq!(store.get(a), 2);
	/// assert_eq!(store.get(b), 3);
	/// assert_eq!(store.get(c), 4);
	/// ```
	pub fn read_disjoint_mut<'a, Props: PropsTuple>(
		&'a mut self, props: Props,
	) -> Props::ResultMut<'a> {
		match self.try_read_disjoint_mut(props) {
			Ok(props) => props,
			Err(Error::Removed) => panic!("mutating removed property"),
			Err(Error::NotDisjoint) => panic!("mutating non-disjoint properties"),
			_ => unreachable!(),
		}
	}
}

/// <h2 id=safe-property-access>Safe Property Access</h2>
///
/// These methods are the safe, non-panicking versions of the [property access methods](#property-access).
///
/// They behave exactly like their counterparts, except they return an [`Error`] (or `None`) for removed properties.
///
/// ## Example
/// ```
/// let nb = store.prop_in(Some(slab), 1);
/// assert_eq!(store.try_read(nb), Some(&1));
/// store.try_write(nb, 2);
/// assert_eq!(store.try_get(nb), Some(2));
///
/// Store::remove_slab(ctx, slab);
/// assert_eq!(store.try_peek(nb), None);
/// store.try_update(nb, |_| println!("will not run"));
/// ```
impl<Ctx: Context> Store<Ctx> {
	/// The safe version of [`read`](Store::read).
	///
	/// It returns a reference to the given property if it exists, and returns [`None`] otherwise, without triggering a read signal.
	///
	/// # Example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert_eq!(store.try_read(nb), Some(&1));
	///
	/// Store::remove_slab(ctx, slab);
	/// assert_eq!(store.try_read(nb), None);
	/// ```
	pub fn try_read<T: 'static>(&self, prop: PropId<T>) -> Option<&T> {
		let value = self.try_peek(prop)?;
		self.track_read(prop);
		Some(value)
	}

	/// The safe version of [`get`](Store::get).
	///
	/// It returns a copy of the given property if it exists, and returns [`None`] otherwise, without triggering a read signal.
	///
	/// # Example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert_eq!(store.try_get(nb), Some(1));
	///
	/// Store::remove_slab(ctx, slab);
	/// assert_eq!(store.try_get(nb), None);
	/// ```
	pub fn try_get<T: 'static + Copy>(&self, prop: PropId<T>) -> Option<T> {
		self.try_read(prop).copied()
	}

	/// The safe version of [`peek`](Store::peek).
	///
	/// It returns a reference to the given property if it exists, and returns [`None`] otherwise. It doesn't trigger a read signal in either case.
	///
	/// # Example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert_eq!(store.try_peek(nb), Some(&1));
	///
	/// Store::remove_slab(ctx, slab);
	/// store.start_track();
	/// assert_eq!(store.try_peek(nb), None);
	/// assert_eq!(store.end_track().unwrap().read, []);
	/// ```
	pub fn try_peek<T: 'static>(&self, prop: PropId<T>) -> Option<&T> {
		self.props.get(prop.0).map(|p| p.get())
	}

	/// The safe version of [`write`](Store::write).
	///
	/// It sets the value of the given property if it exists, and returns [`Error::Removed`] otherwise, without triggering anything.
	///
	/// # Example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert_eq!(store.try_write(nb, 2), Ok(1));
	///
	/// Store::remove_slab(ctx, slab);
	/// assert_eq!(store.try_write(nb, 3), Err(Error::Removed));
	/// ```
	pub fn try_write<T: 'static>(&mut self, prop: PropId<T>, value: T) -> Result<T, Error> {
		let prop = self.try_read_mut(prop).ok_or(Error::Removed)?;
		Ok(std::mem::replace(prop, value))
	}

	/// The safe version of [`read_mut`](Store::read_mut).
	///
	/// It returns a mutable reference to the given property if it exists, and returns [`None`] otherwise, without triggering anything.
	///
	/// # Example
	/// ```
	/// let arr = store.prop_in(Some(slab), vec![1, 2, 3]);
	/// store.try_read_mut(arr).unwrap()[2] = 4;
	///
	/// Store::remove_slab(ctx, slab);
	/// assert_eq!(store.try_read_mut(nb), None);
	/// ```
	pub fn try_read_mut<T: 'static>(&mut self, prop: PropId<T>) -> Option<&mut T> {
		let value = self.props.get_mut(prop.0)?.get_mut();
		Self::_track_write(&self.tracking, prop.0);
		self.updater.push_update(prop.0);
		Some(value)
	}

	/// The safe version of [`update`](Store::update).
	///
	/// It updates the given property with the updater `fun`ction if it exists, and returns [`Error::Removed`] otherwise, without triggering anything.
	///
	/// # Example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert!(store.try_update(nb, |v| *v += 1).is_ok());
	///
	/// Store::remove_slab(ctx, slab);
	/// assert_eq!(store.try_update(nb, |_| println!("will not run")), Err(Error::Removed));
	/// ```
	pub fn try_update<T: 'static>(
		&mut self, prop: PropId<T>, fun: impl FnOnce(&mut T),
	) -> Result<(), Error> {
		let prop = self.try_read_mut(prop).ok_or(Error::Removed)?;
		fun(prop);
		Ok(())
	}

	/// The safe version of [`read_disjoint_mut`](Store::read_disjoint_mut).
	///
	/// It returns multiple mutable references to the given properties.
	///
	/// it returns [`Error::Removed`] if any property is removed, and returns [`Error::NotDisjoint`] if the properties are not disjoint without triggering anything.
	///
	/// # Example
	/// ```
	/// let a = store.prop(1);
	/// let b = store.prop_in(Some(slab), 2);
	/// let c = store.prop(3);
	///
	/// assert_eq!(store.try_read_disjoint_mut((a, b, c)), Ok((1, 2, 3)));
	///
	/// Store::remove(ctx, slab);
	/// assert_eq!(store.try_read_disjoint_mut((a, b, c)), Err(Error::Removed));
	///
	/// assert_eq!(store.try_read_disjoint_mut((a, c, a)), Err(Error::NotDisjoint));
	/// ```
	pub fn try_read_disjoint_mut<'a, Props: PropsTuple>(
		&'a mut self, props: Props,
	) -> Result<Props::ResultMut<'a>, Error> {
		Props::read_disjoint_mut(self, props)
	}
}

/// An effect's dependencies.
///
/// This enum specifies whether the effect's dependencies are implicitly identified through [tracking](Store#tracking) or manually specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDeps {
	/// Dependencies are identified implicitly via [tracking](Store#tracking).
	Tracked,
	/// Dependencies are manually specified.
	Manual {
		/// The properties the effect reads and reevaluates upon.
		read: Vec<PropId<()>>,
		/// The properties the effect writes to.
		write: Vec<PropId<()>>,
		/// Whether to run the effect initially upon definition.
		init_run: bool,
	},
}

/// <h2 id=effects>Effects</h2>
///
/// Effects are where the reactivity occurs.
///
/// They are functions that depend on specific properties. When the target properties change, they rerun. They are passed the owning [`Context`] and must have a `'static` lifetime.
///
/// To understand the effect execution order, see the [updating section](#updating).
impl<Ctx: Context> Store<Ctx> {
	/// Defines an effect.
	///
	/// `effect` registers the `fun`ction as an effect in a specific scope (global if `slab` is [`None`]), with the specified dependencies.
	///
	/// The dependencies can be implicitly identified through [tracking](#tracking) if `dep` is [`EffectDeps::Tracked`]; otherwise, they are manually specified via [`EffectDeps::Manual`].
	///
	/// It takes the context and calls the `fun`ction upon definition to identify the dependencies if needed.
	///
	/// It returns [`Error::Removed`] if the `slab` is removed.
	///
	/// This method is redirected by every [`ScopedStoreProv`](crate::ScopedStoreProv)ider into a more ergonomic format.
	///
	/// # Example
	/// ```
	/// use EffectDeps::*;
	/// let count = store.prop(1);
	/// let doubled = store.prop(1);
	///
	/// Store::effect(ctx, None, Tracked, move |ctx| println!("doubled: {}", ctx.get(doubled))); // => doubled: 2
	/// // No-op afterward
	/// Store::effect(ctx, None, Tracked, move |ctx| println!("doubled: {}", ctx.peek(doubled))); // => doubled: 2
	/// // Same as the first
	/// let deps = Manual { read: vec![doubled.erase_type()], write: Vec::new(), init_run: true };
	/// Store::effect(ctx, None, deps, move |ctx| println!("doubled: {}", ctx.get(doubled))); // => doubled: 2
	/// // In slab
	/// let deps = Manual { read: vec![doubled.erase_type()], write: Vec::new(), init_run: false };
	/// Store::effect(ctx, Some(slab), deps, move |ctx| println!("doubled: {}", ctx.get(doubled))); // => doubled: 2
	///
	/// Store::effect(ctx, None, Tracked, move |ctx| ctx.write(doubled, ctx.get(count) * 2));
	///
	/// store.write(count, 2);
	/// Store::flush_updates(ctx); // => doubled: 4 x3
	///
	/// Store::remove_slab(ctx, slab);
	/// store.write(count, 3);
	/// Store::flush_updates(ctx); // => doubled: 6 x2
	/// ```
	pub fn effect(
		ctx: &mut Ctx, slab: Option<SlabId>, dep: EffectDeps, fun: impl FnMut(&mut Ctx) + 'static,
	) -> Result<(), Error> {
		if let Some(slab) = slab
			&& !ctx.store().has_slab(slab)
		{
			return Err(Error::Removed);
		}

		let (deps, init_run) = match dep {
			EffectDeps::Tracked => (None, true),
			EffectDeps::Manual { read, write, init_run } => (Some((read, write)), init_run),
		};
		let id = Updater::add_effect(ctx, fun, deps, init_run);

		if let Some(slab) = slab {
			ctx.store().slab(slab).effects.push(id);
		}

		Ok(())
	}

	/// Creates a computed property.
	///
	/// A computed property is a reactive property derived from a `fun`ction that takes the [`Context`] and gets reevaluated whenever its dependencies change.
	///
	/// `computed` defines the property in a specific scope (global if `slab` is [`None`]) and returns the property's [`PropId`].
	///
	/// It takes the context and calls `fun` upon definition to set the property's initial value and to identify its dependencies using [tracking](#tracking).
	///
	/// It returns [`Error::Removed`] if the `slab` is removed.
	///
	/// This method is redirected by every [`ScopedStoreProv`](crate::ScopedStoreProv)ider into a more ergonomic format.
	///
	/// # Example
	/// ```
	/// let count = store.prop(1);
	/// let doubled = Store::computed(ctx, Some(slab), move |ctx| ctx.get(count) * 2);
	/// assert_eq!(store.get(doubled), 2);
	///
	/// store.write(count, 2);
	/// Store::flush_updates(ctx);
	/// assert_eq!(store.get(doubled), 4);
	///
	/// Store::remove_slab(ctx, slab);
	/// assert!(!store.contains(doubled));
	/// ```
	pub fn computed<T: 'static>(
		ctx: &mut Ctx, slab: Option<SlabId>, mut fun: impl FnMut(&mut Ctx) -> T + 'static,
	) -> Result<PropId<T>, Error> {
		if let Some(slab) = slab
			&& !ctx.store().has_slab(slab)
		{
			return Err(Error::Removed);
		}

		// init value
		start_track_panicing(ctx.store_ref());
		let value = fun(ctx);
		let store = ctx.store();
		let TrackResult { read, written } = store.end_track().unwrap();

		if !written.is_empty() {
			panic!("computed properties cannot write to any properties");
		}

		let id = store.prop(value);

		// effect
		let fun = move |ctx: &mut Ctx| {
			let value = fun(ctx);
			ctx.store().write(id, value);
		};
		let effect = Updater::add_effect(ctx, fun, Some((read, vec![id.erase_type()])), false);

		if let Some(slab) = slab {
			let slab = ctx.store().slab(slab);
			slab.effects.push(effect);
			slab.props.push(id.0);
		}
		Ok(id)
	}
}

/// <h2 id=slab-management>Slab Management</h2>
///
/// The `Store` is composed of multiple independent scopes. Each scope has its own lifetime, which is shared across its items (properties and effects).
///
/// The default scope is the global scope, which spans the entire `Store`'s lifetime.
///
/// The other kind consists of slabs, which are scopes identified by a [`SlabId`] and can be removed when needed.
///
/// All effects that depend on properties inside slabs should themselves be placed inside the slab with the shortest lifetime. This prevents them from running after their dependencies have been removed, which would lead to panics.
///
/// ## Example
/// ```
/// let slab = store.create_slab();
/// let count = store.prop_in(slab, 1);
///
/// Store::remove_slab(ctx, slab);
/// assert!(!store.has_slab(slab));
/// assert!(!store.contains(count));
/// ```
impl<Ctx: Context> Store<Ctx> {
	/// Creates a slab, returning its [`SlabId`].
	///
	/// # Example
	/// ```
	/// let slab = store.create_slab();
	/// ```
	pub fn create_slab(&mut self) -> SlabId {
		let id = self.next_slab;
		self.slabs.insert(id, SlabData::default());
		self.next_slab = SlabId(id.0 + 1);
		id
	}

	/// Returns a mutable reference to `SlabData`.
	fn slab(&mut self, slab: SlabId) -> &mut SlabData<Ctx> {
		self.slabs.get_mut(&slab).unwrap()
	}

	/// Checks whether a slab exists inside the `Store`.
	///
	/// In some [circumstances](#remove-slab-note), a slab may be marked as removed, but its items are not dropped yet.
	///
	/// # Example
	/// ```
	/// let slab = store.create_slab();
	/// assert!(store.has_slab(slab));
	///
	/// Store::remove_slab(ctx, slab);
	/// assert!(!store.has_slab(slab));
	/// ```
	pub fn has_slab(&self, slab: SlabId) -> bool {
		self.slabs.contains_key(&slab) && !self.slabs_to_remove.contains(&slab)
	}

	/// Removes the given slab.
	///
	/// This method will drop all of the slab's items and returns [`Error::Removed`] if the given `slab` was already removed.
	///
	/// <h4 id=remove-slab-note>Note</h4>
	///
	/// During updates inside effects, removed slabs will not be dropped instantly. Instead, they will be marked as removed and will be dropped when the update ends.
	///
	/// # Example
	/// ```
	/// let slab = store.create_slab();
	/// let count = store.prop_in(slab, 1);
	///
	/// Store::remove_slab(ctx, slab);
	/// assert!(!store.has_slab(slab));
	/// ```
	pub fn remove_slab(ctx: &mut Ctx, id: SlabId) -> Result<(), Error> {
		let store = ctx.store();
		if !store.has_slab(id) {
			return Err(Error::Removed);
		}

		if store.updater.is_updating {
			// since we would need to clean the current effect queue, which is inefficient and rarely necessary.
			store.slabs_to_remove.push(id);
		} else {
			Store::drop_slab(ctx, id);
		}
		Ok(())
	}

	/// Removes the slab for real.
	fn drop_slab(ctx: &mut Ctx, id: SlabId) {
		while let Some(cleaner) = ctx.store().slab(id).cleaner.pop() {
			cleaner(ctx)
		}

		let store = ctx.store();
		let slab = &store.slabs.remove(&id).unwrap();
		for id in &slab.props {
			store.props.remove(*id);
		}
		store.updater.remove_items(&slab.effects, &slab.props);
	}
}

/// <h2 id=updating>Updating</h2>
///
/// The `Store` uses fine-grained reactivity to update only the required parts and runs the minimal number of effects.
///
/// Mutations to properties through [`write`](Store::write), [`update`](Store::update), and [`read_mut`](Store::read_mut) mark the properties as dirty and queue them. A subsequent call to [`flush_updates`](Store::flush_updates) takes the owning [`Context`] and passes it to the effects.
///
/// Effects are gathered and executed in an undefined order. However, it is guaranteed that each affected effect gets executed exactly once, and only after all of its read dependencies have been finalized (i.e., after the last effect writing to them has executed).
///
/// Effects can affect other effects through their write dependencies. The effect graph can be any acyclic graph.
///
/// # Example
/// ```
/// let a = store.prop(1);
/// let b = store.prop(1);
/// let c = store.prop(1);
/// let d = store.prop(1);
///
/// Store::effect(ctx, None, EffectDeps::Tracked, move |ctx| {
///     ctx.write(b, ctx.get(a));
/// });
/// Store::effect(ctx, None, EffectDeps::Tracked, move |ctx| {
///     ctx.write(c, ctx.get(b) + 1);
/// });
/// Store::effect(ctx, None, EffectDeps::Tracked, move |ctx| {
///     ctx.write(c, ctx.get(b) + 2);
/// });
/// Store::effect(ctx, None, EffectDeps::Tracked, move |ctx| {
///     ctx.write(d, ctx.get(c) + ctx.get(a));
/// });
///
/// store.write(a, 2);
/// Store::flush_updates(ctx); // All run exactly once
/// assert!(matches!(store.get(d), 5 | 6));
/// ```
///
/// <h3 id=conditional-updates>Conditional Updates</h3>
///
/// The effect graph is static: the same effects will run even if the written value hasn't changed, the specified write dependencies weren't actually mutated, or other unspecified properties are mutated.
///
/// For conditional mutations inside effects, use [`force_update`](Store::force_update). This queues the given properties at the end of the current effect batch. The `Store` will then regather and run the required effects, repeating this process until there are no more dirty properties.
///
/// This can cause effects to run more than once in rare cases, and can lead to endless loops in extreme ones.
///
/// # Example
/// ```
///    let a = store.prop(1);
/// let b = store.prop(1);
///
/// let mut old = store.get(a);
/// let deps = EffectDeps::Manual { read: Vec::new(), write: Vec::new(), init_run: true };
/// Store::effect(ctx, None, deps, move |ctx| {
///     if ctx.peek(a) != &old {
///         old = ctx.get(a);
///         ctx.write(b, old + 1);
///         ctx.store().force_update(b);
///     }
/// });
/// ```
impl<Ctx: Context> Store<Ctx> {
	/// Checks whether the `Store` is currently executing effects.
	///
	/// # Example
	/// ```
	/// assert!(!store.is_updating());
	/// Store::effect(ctx, None, EffectDeps::Tracked, |ctx| assert!(ctx.store().is_updating()));
	/// ```
	pub fn is_updating(&self) -> bool {
		self.updater.is_updating
	}

	/// Marks a property as dirty so it will be updated.
	///
	/// This method marks the property as dirty regardless of whether the `Store` is currently updating.
	///
	/// This method is used in [conditional updates](#conditional-updates).
	///
	/// # Example
	/// ```
	///    let a = store.prop(1);
	/// let b = store.prop(1);
	///
	/// let mut old = store.get(a);
	/// let deps = EffectDeps::Manual { read: Vec::new(), write: Vec::new(), init_run: true };
	/// Store::effect(ctx, None, deps, move |ctx| {
	///     if ctx.peek(a) != &old {
	///         old = ctx.get(a);
	///         ctx.write(b, old + 1);
	///         ctx.store().force_update(b);
	///     }
	/// });
	/// ```
	pub fn force_update<T: 'static>(&mut self, id: PropId<T>) {
		if !self.updater.dirty_props.contains(&id.0) {
			self.updater.dirty_props.push(id.0);
		}
	}

	/// Responds to the currently pending updates.
	///
	/// This method takes the owning [`Context`] and executes all effects that depend on the queued dirty properties.
	///
	/// This method is safe to call at any time, but calling it when you have finished using the [`Context`] is usually sufficient.
	///
	/// # Example
	/// ```
	/// fn event_handler(input: u64) {
	///     let ctx = get_ctx();
	///     ctx.write(some_prop, input);
	///     ctx.write(some_other_prop, input + 1);
	///     Store::flush_updates(ctx);
	/// }
	/// ```
	pub fn flush_updates(ctx: &mut Ctx) {
		if ctx.store().updater.is_updating {
			return;
		}
		Updater::update(ctx);

		// slabs deleted inside effects are removed at the end of an update to simplify logic
		while let Some(slab) = ctx.store().slabs_to_remove.pop() {
			Store::drop_slab(ctx, slab);
		}
	}
}

/// <h2 id=tracking>Tracking</h2>
///
/// Tracking is a mechanism that allows identifying the properties used in a chunk of normal code without any extra syntax.
///
/// It starts with [`start_track`](Store::start_track). Following that, every call to a [property access method](#property-access) records the target property as read or written. Finally, a call to [`end_track`](Store::end_track) returns the recorded properties in a [`TrackResult`].
///
/// # Example
/// ```
/// let a = store.prop(1);
/// let b = store.prop(2);
/// let c = store.prop(3);
/// let d = store.prop(4);
/// store.start_track();
///
/// store.read(a);
/// store.peek(b);
/// store.write(c, 5);
/// store.write(d, 6);
/// store.read(a);
///
/// let TrackResult { read, written } = store.end_track().unwrap();
/// assert_eq!(read, [a.erase_type()]);
/// assert_eq!(written, [c.erase_type(), d.erase_type()]);
/// ```
impl<Ctx: Context> Store<Ctx> {
	/// Checks whether tracking is activated.
	///
	/// # Example
	/// ```
	/// assert!(!store.is_tracking());
	/// store.start_track();
	/// assert!(store.is_tracking());
	/// store.end_track();
	/// assert!(!store.is_tracking());
	/// ```
	pub fn is_tracking(&self) -> bool {
		self.tracking.borrow().is_some()
	}

	/// Activates tracking.
	///
	/// Returns [`Error::Tracking`] if tracking was already activated.
	pub fn start_track(&self) -> Result<(), Error> {
		if self.is_tracking() {
			return Err(Error::Tracking);
		}
		self.tracking.replace(Some(TrackResult::default()));
		Ok(())
	}

	/// Stops tracking and returns the [`TrackResult`].
	///
	/// Returns [`Error::NotTracking`] if tracking was not previously activated.
	pub fn end_track(&self) -> Result<TrackResult, Error> {
		let mut result = self.tracking.take().ok_or(Error::NotTracking)?;

		// deduping at the end is faster than on every call
		result.read.sort_unstable();
		result.read.dedup();
		result.written.sort_unstable();
		result.written.dedup();

		Ok(result)
	}

	/// Records a property as read while tracking.
	///
	/// It is a no-op if tracking is not active.
	///
	/// # Example
	/// ```
	/// let a = store.prop(1);
	/// store.start_track();
	/// store.track_read(a);
	/// assert_eq!(store.end_track().unwrap().read, [a.erase_type()]);
	/// ```
	pub fn track_read<T: 'static>(&self, id: PropId<T>) {
		if let Some(tracking) = self.tracking.borrow_mut().deref_mut() {
			tracking.read.push(id.erase_type());
		}
	}

	/// Records a property as written while tracking.
	// done like this because `get_mut` requires a partial borrow.
	fn _track_write(tracking: &RefCell<Option<TrackResult>>, id: ItemId) {
		if let Some(tracking) = tracking.borrow_mut().deref_mut() {
			tracking.written.push(PropId::new(id));
		}
	}

	/// Records a property as written while tracking.
	///
	/// It is a no-op if tracking is not active.
	///
	/// # Example
	/// ```
	/// let a = store.prop(1);
	/// store.start_track();
	/// store.track_write(a);
	/// assert_eq!(store.end_track().unwrap().written, [a.erase_type()]);
	/// ```
	pub fn track_write<T: 'static>(&self, id: PropId<T>) {
		Self::_track_write(&self.tracking, id.0);
	}
}

/// <h2 id=store-management>Store Management</h2>
impl<Ctx: Context> Store<Ctx> {
	/// Adds a cleaner function to a specific scope.
	///
	/// A cleaner `fun`ction is called with the owning [`Context`] when that scope is being dropped, right before dropping its items.
	///
	/// It adds the cleaner to the global scope when `slab` is [`None`], and returns [`Error::Removed`] if the target `slab` has been removed.
	///
	/// # Example
	/// ```
	/// let slab = store.create_slab();
	/// let child_slab = store.create_slab();
	/// let count = store.prop_in(Some(slab), 1);
	/// store.add_cleaner(Some(slab), move |ctx| {
	///     println!("count was: {}", ctx.get(count));
	///     Store::remove_slab(ctx, child_slab);
	/// })
	/// ```
	pub fn add_cleaner(
		&mut self, slab: Option<SlabId>, fun: impl FnOnce(&mut Ctx) + 'static,
	) -> Result<(), Error> {
		let Some(slab) = slab else {
			self.global_cleaners.push(Box::new(fun));
			return Ok(());
		};

		if !self.has_slab(slab) {
			return Err(Error::Removed);
		}
		self.slab(slab).cleaner.push(Box::new(fun));
		Ok(())
	}

	/// A hook for dropping the [`Context`].
	///
	/// `pre_drop` is a function that must be called when the owning [`Context`] is being dropped.
	///
	/// It safely drops the `Store` and calls the [`cleaners`](Store::add_cleaner) for all scopes, starting with the slabs and ending with the global scope.
	pub fn pre_drop(ctx: &mut Ctx) {
		let store = ctx.store();
		if store.is_dropped {
			panic!("calling `Store::pre_drop` twice")
		}
		store.is_dropped = true;

		while let Some(&slab) = ctx.store().slabs.keys().next() {
			Store::remove_slab(ctx, slab).unwrap()
		}

		while let Some(cleaner) = ctx.store().global_cleaners.pop() {
			cleaner(ctx)
		}
	}
}

impl<Ctx> Drop for Store<Ctx> {
	fn drop(&mut self) {
		if !self.is_dropped {
			panic!("dropped without calling `Store::pre_drop`")
		}
	}
}

pub trait PropsTuple {
	type ResultMut<'a>;
	fn read_disjoint_mut<'a, Ctx: Context>(
		store: &'a mut Store<Ctx>, props: Self,
	) -> Result<Self::ResultMut<'a>, Error>;
}
macro_rules! impl_props_tuple {
	[$($prop:ident),+] => {
		#[allow(non_snake_case)]
		impl<$($prop: 'static),+> PropsTuple for ($(PropId<$prop>),+,) {
			type ResultMut<'a> = ($(&'a mut $prop),+,);
			fn read_disjoint_mut<Ctx: Context>(
				store: &mut Store<Ctx>, props: Self,
			) -> Result<($(&mut $prop),+,), Error> {
				let ($($prop),+,) = props;
				let props_ids = [$($prop.0),+];
				for prop in props_ids {
					if !store.props.contains_key(prop) {
						return Err(Error::Removed);
					}
				}
				let Some(props) = store.props.get_disjoint_mut(props_ids) else {
					return Err(Error::NotDisjoint);
				};
				for prop in props_ids {
					Store::<Ctx>::_track_write(&store.tracking, prop);
					store.updater.push_update(prop);
				}
				let [$($prop),+] = props;
				Ok(($($prop.get_mut()),+,))
			}
		}
	};
}
impl_props_tuple![A];
impl_props_tuple![A, B];
impl_props_tuple![A, B, C];
impl_props_tuple![A, B, C, D];
impl_props_tuple![A, B, C, D, E];
impl_props_tuple![A, B, C, D, E, F];
impl_props_tuple![A, B, C, D, E, F, G];
impl_props_tuple![A, B, C, D, E, F, G, H];
impl_props_tuple![A, B, C, D, E, F, G, H, I];
impl_props_tuple![A, B, C, D, E, F, G, H, I, J];
impl_props_tuple![A, B, C, D, E, F, G, H, I, J, K];
impl_props_tuple![A, B, C, D, E, F, G, H, I, J, K, L];
