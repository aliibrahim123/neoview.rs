use std::{any::Any, cell::RefCell, fmt::Debug, ops::DerefMut, ptr};

use rustc_hash::FxHashMap;
use slotmap::SlotMap;

use crate::{
	Error, PropId, SlabId,
	context::Context,
	prop::ItemId,
	updater::{Updater, start_track_panicing},
};

/// stores items owned by a slab
pub struct SlabData<Ctx> {
	pub props: Vec<ItemId>,
	pub effects: Vec<ItemId>,
	pub cleaner: Vec<Box<dyn FnOnce(&mut Ctx)>>,
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

/// the result of a tracking operation.
///
/// produced by when the tracking operation is ended with [`Store::end_track`].
///
/// # example
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
	/// the properties read.
	pub read: Vec<PropId<()>>,
	/// the properties written to.
	pub written: Vec<PropId<()>>,
}
impl TrackResult {
	/// destruct the `TrackResult` into a `(read, written)` pair
	pub(crate) fn destruct(self) -> (Vec<PropId<()>>, Vec<PropId<()>>) {
		(self.read, self.written)
	}
}

/// the container of the reactivity system.
///
/// the `Store` is the structure that owns and manages the entire reactivity system with its [properties](#property-managment) and [effects](#effects).
///
/// it is tightly copouled to a specific [`Context`] that owns it, and its lifetime is identical to it.
///
/// every interaction with the reactive system requires a mutable access to the `Store`, however all the common operations are redirected through the family of [`StoreProv`](crate::StoreProv)iders traits.
///
/// ## example
/// ```
/// let count = store.prop(0);
/// assert_eq!(store.get(count), 0);
/// Store::effect(ctx, move |ctx| println!("count: {}", ctx.get(count)))
/// store.set(count, 1);
/// assert_eq!(store.get(count), 1);
///
/// let doubled = Store::computed(ctx, move |ctx| ctx.get(count) * 2);
/// store.update(count, |cnt| *cnt += 1);
/// assert_eq!(store.get(doubled), 4);
/// Store::flush_updates(ctx); // => count: 2
/// ```
///
/// # sections
/// due to the large api surface exposed by the `Store`, its documentations has been splitted into multiple parts.
///
/// they are:
/// - [property managment](#property-managment).
/// - [property access](#property-access).
/// - [safe property access](#safe-property-access).
/// - [Effects](#effects).
/// - [Slab Managment](#slab-managment).
/// - [Updating](#updating).
/// - [Tracking](#tracking).
/// - [Store Managment](#store-managment).
pub struct Store<Ctx> {
	pub(crate) props: SlotMap<ItemId, Box<dyn Any>>,

	pub(crate) slabs: FxHashMap<SlabId, SlabData<Ctx>>,
	/// the `SlabId` of the next slab to be added
	next_slab: SlabId,
	/// slabs removed during an update to be deleated at the end of that update
	slabs_to_remove: Vec<SlabId>,

	pub(crate) updater: Updater<Ctx>,

	global_cleaners: Vec<Box<dyn FnOnce(&mut Ctx)>>,
	is_dropped: bool,

	// `RefCell` to not make `read` take mutable reference
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
			.field("tarcking", &self.tracking)
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

/// <h2 id=property-managment>Property Managment</h2>
///
/// reactive properties managment is the sole purpose of the `Store`.
///
/// a reactive property is any value that is used inside the reactivity system, it can be of any type not containing references (`'static` is allowed) and it is owned by the `Store`.
///
/// a property is created by [`prop`](Store::prop), identified by a [`PropId`], accesed by the [property access methods](#property-access), and can be binded to and by multiple [effects](#effects).
///
/// individual properties can not be removed, they can only be removed with the store or their owner slab.
impl<Ctx: Context> Store<Ctx> {
	/// defines a new reactive property in the global scope.
	///
	/// it accepts the property initial `value`, and returns its [`PropId`],
	///
	/// # example
	/// ```
	/// let count = store.prop(0);
	/// let text = store.prop("hello".to_string());
	/// struct Value { a: i32, b: f64, c: String, d: Vec<u8> }
	/// let value = store.prop(Value {
	/// 	a: 1, b: 1.5, c: "abc".to_string(), d: vec![1, 2, 3],
	/// });
	/// ```
	pub fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let id = self.props.insert(Box::new(value));
		PropId::new(id)
	}

	/// defines a new reactive property in a specific scope.
	///
	/// it accepts the target `slab` and the property initial `value`, and returns its [`PropId`].
	///
	/// if `slab` is [`None`], the property is defined in the global scope.
	///
	/// # example
	/// ```
	/// let slab = store.create_slab();
	/// let count = store.prop_in(Some(slab), 0);
	/// let text = store.prop_in(Some(slab), "hello".to_string());
	///
	/// // the same
	/// let nb = store.prop(1.5);
	/// let nb = store.prop_in(None, 1.5);
	/// ```
	pub fn prop_in<T: 'static>(
		&mut self, slab: Option<SlabId>, value: T,
	) -> Result<PropId<T>, Error> {
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

	/// check whether a reactive property is inside the `Store`.
	///
	/// # example
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
/// the property access methods are functions that read and mutate the reactive properties defined inside the `Store`.
///
/// they are of 2 kinds:
/// - **reading methods**: [`read`](Store::read), [`get`](Store::get) and [`peek`](Store::peek).
/// - **mutating methods**: [`write`](Store::write), [`read_mut`](Store::read_mut) and [`update`](Store::update).
///
/// ## example
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
/// these methods will trigger [updates](#updating), and they might be [tracked](#tracking).
///
/// these methods are redirected by every [`StoreProv`](crate::StoreProv)ider.
///
/// these methods are designed to be ergonomic, they will panic on errors, for a safe version see [safe property access](#safe-property-access).
impl<Ctx: Context> Store<Ctx> {
	/// return a reference to a reactive property value.
	///
	/// `read` is the property reading primitive, it triggers a read signal while tracking.
	///
	/// it will panic if the given property is removed, for a safe version see [`try_read`](Store::try_read).
	///
	/// # example
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

	/// return a copy of a [`Copy`]able reactive property value.
	///
	/// it triggers a read signal while tracking, and will panic if the given property is removed, for a safe version see [`try_get`](Store::try_get).
	///
	/// # example
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

	/// return a reference to a reactive property value without being tracked.
	///
	/// this method is identical to [`read`](Store::read), except it doesnt trigger a read signal.
	///
	/// it will panic if the given property is removed, for a safe version see [`try_peek`](Store::try_peek).
	///
	/// # example
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

	/// set a reactive property value.
	///
	/// this method trigger an update, trigger a write signal while tracking and return the previous value of the property.
	///
	/// it will panic if the given property is removed, for a safe version see [`try_write`](Store::try_write).
	///
	/// # example
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

	/// return a mutable reference to a reactive property value.
	///
	/// this method is the mutating property primitive, it triggers an update, and trigger a write signal while tracking.
	///
	/// it will panic if the given property is removed, for a safe version see [`try_read_mut`](Store::try_read_mut).
	///
	/// # example
	/// ```
	/// let arr = store.prop(vec![1, 2, 3]);
	/// store.read_mut(arr)[2] = 4;
	/// assert_eq!(store.read(arr), [1, 2, 4]);
	/// ```
	pub fn read_mut<T: 'static>(&mut self, prop: PropId<T>) -> &mut T {
		self.try_read_mut(prop).expect("mutating removed property")
	}

	/// update a reactive property using an updater function.
	///
	/// the `fun`tion is called with a mutable reference to the property value, the method triggers an update and triggers a write signal while tracking.
	///
	/// it will panic if the given property is removed, for a safe version see [`try_update`](Store::try_update).
	///
	/// # example
	/// ```
	/// let nb = store.prop(1);
	/// store.update(nb, |v| *v += 1);
	/// assert_eq!(store.get(nb), 2);
	/// ```
	pub fn update<T: 'static>(&mut self, prop: PropId<T>, fun: impl FnOnce(&mut T)) {
		self.try_update(prop, fun).expect("updating removed property");
	}
}

/// <h2 id=safe-property-access>Safe Property Access</h2>
///
/// these methods are the safe version of the [property access methods](#property-access).
///
/// they behave exactly like their conterparts, except they return an [`Error`] for removed properties.
///
/// ## example
/// ```
/// let nb = store.prop_in(Some(slab), 1);
/// assert_eq!(store.try_read(nb), Some(1));
/// store.try_write(nb, 2);
/// assert_eq!(store.try_get(nb), Some(2));
///
/// Store::remove_slab(ctx, slab);
/// assert!(store.try_peek(nb), None);
/// store.update(nb, |_| println!("will not run"));
/// ```
impl<Ctx: Context> Store<Ctx> {
	/// the safe version of [`read`](Store::read).
	///
	/// it returns a reference to the given property if it exists, and return [`None`] otherwise without triggering a read signal.
	///
	/// # example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert_eq!(store.try_read(nb), Some(1));
	///
	/// Store::remove_slab(ctx, slab);
	/// assert_eq!(store.try_read(nb), None);
	/// ```
	pub fn try_read<T: 'static>(&self, prop: PropId<T>) -> Option<&T> {
		let value = self.try_peek(prop)?;
		self.track_read(prop);
		Some(value)
	}

	/// the safe version of [`get`](Store::get).
	///
	/// it returns a copy of the given property if it exists, and return [`None`] otherwise without triggering a read signal.
	///
	/// # example
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

	/// the safe version of [`peek`](Store::peek).
	///
	/// it returns a reference of the given property if it exists, and return [`None`] otherwise, it doesnt trigger a read signal in both cases.
	///
	/// # example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert_eq!(store.try_peek(nb), Some(1));
	///
	/// Store::remove_slab(ctx, slab);
	/// store.start_track();
	/// assert_eq!(store.try_peek(nb), None);
	/// assert_eq!(store.end_track().unwrap().read, []);
	/// ```
	pub fn try_peek<T: 'static>(&self, prop: PropId<T>) -> Option<&T> {
		self.props.get(prop.0)?.downcast_ref()
	}

	/// the safe version of [`write`](Store::write).
	///
	/// it sets the value of the given property if it exists, and return [`Error::Removed`] otherwise without triggering any thing.
	///
	/// # example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert_eq!(store.try_write(nb, 2), Ok(2));
	///
	/// Store::remove_slab(ctx, slab);
	/// assert_eq!(store.try_write(nb), Err(Error::Removed));
	/// ```
	pub fn try_write<T: 'static>(&mut self, prop: PropId<T>, value: T) -> Result<T, Error> {
		let prop = self.try_read_mut(prop).ok_or(Error::Removed)?;
		Ok(std::mem::replace(prop, value))
	}

	/// the safe version of [`read_mut`](Store::read_mut).
	///
	/// it returns a mutable reference for the given property if it exists, and return [`None`] otherwise without triggering any thing.
	///
	/// # example
	/// ```
	/// let arr = store.prop_in(Some(slab), vec![1, 2, 3]);
	/// store.try_read_mut(arr).unwrap()[2] = 4;
	///
	/// Store::remove_slab(ctx, slab);
	/// assert_eq!(store.try_read_mut(nb), None);
	/// ```
	pub fn try_read_mut<T: 'static>(&mut self, prop: PropId<T>) -> Option<&mut T> {
		let value = self.props.get_mut(prop.0)?.downcast_mut()?;
		Self::_track_write(&self.tracking, prop);
		self.updater.push_update(prop.0);
		Some(value)
	}

	/// the safe version of [`update`](Store::update).
	///
	/// it updates the given property with the updater `fun`ction if it exists, and return [`Error::Removed`] otherwise without triggering any thing.
	///
	/// # example
	/// ```
	/// let nb = store.prop_in(Some(slab), 1);
	/// assert!(store.try_update(nb, |v| *v += 1).is_some);
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
}

/// an effect dependencies.
///
/// this enum specifies if the effect dependencies implicitly identified through [tracking](Store#tracking), or manually specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDeps {
	/// depednencies are identified implicitly by [tracking](Store#tracking).
	Tracked,
	/// dependencies are manually specified.
	Manual {
		/// the properties the effect read and reevaluate on.
		read: Vec<PropId<()>>,
		/// the properties the effect write to.
		write: Vec<PropId<()>>,
		/// whether to run the effect initialy at definition.
		init_run: bool,
	},
}

/// <h2 id=effects>Effects</h2>
///
/// effects are where the reactivity occurs.
///
/// they are functions that depends on specific properties, when the target properties change they rerun, they are passed the owner [`Context`], and must be of `'static` lifetime.
///
/// to know the effects run order, read the [updating section](#updating).
impl<Ctx: Context> Store<Ctx> {
	/// define an effect.
	///
	/// `effect` register the `fun`ction as an effect in a specific scope (global if `slab` is [`None`]), with the dependencies specified.
	///
	/// the dependencies can be implicitly identified through [tracking](#tracking) if `deps` is [`EffectDeps::Tracked`], otherwise they are manually specifed through [`EffectDeps::Manual`].
	///
	/// it takes the context and call the `fun` on definition to identify the dependencies if needed.
	///
	/// it returns [`Error::Removed`] if the `slab` is removed.
	///
	/// this method is redirected by every [`ScopedStoreProv`](crate::ScopedStoreProv)ider in a more egornomic format.
	///
	/// # example
	/// ```
	/// use EffectDeps::*;
	/// let count = store.prop(1);
	/// let doubled = store.prop(1);
	///
	/// Store::effect(ctx, None, Tracked, move |ctx| println!("doubled: {}", ctx.get(doubled))); // => doubled: 2
	/// // noop afterward
	/// Store::effect(ctx, None, Tracked, move |ctx| println!("doubled: {}", ctx.peek(doubled))); // => doubled: 2
	/// // same as the first
	/// let deps = Manual { read: vec![doubled.erase_type()], write: Vec::new(), init_run: true };
	/// Store::effect(ctx, None, deps, move |ctx| println!("doubled: {}", ctx.get(doubled))); // => doubled: 2
	/// // in slab
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

	/// the core fun that create computed properties, it returns (prop, effect_id)
	pub(crate) fn computed_core<T: 'static>(
		ctx: &mut Ctx, mut fun: impl FnMut(&mut Ctx) -> T + 'static,
	) -> (PropId<T>, ItemId) {
		start_track_panicing(ctx.store_ref());
		let value = fun(ctx);
		let store = ctx.store();
		let TrackResult { read, written } = store.end_track().unwrap();

		if !written.is_empty() {
			panic!("computed properties can not write any properties");
		}

		let id = store.prop(value);

		let fun = move |ctx: &mut Ctx| {
			let value = fun(ctx);
			ctx.store().write(id, value);
		};
		let effect = Updater::add_effect(ctx, fun, Some((read, vec![id.erase_type()])), false);

		(id, effect)
	}

	/// create a computed property.
	///
	/// a computed property is a reactive property that is derived from a `fun`ction taking the [`Context`] and get reevaluated when its dependencies change.
	///
	/// `computed` defines the property in a specific scope (global if `slab` is [`None`]), and return the property [`PropId`].
	///
	/// it takes the context and call `fun` on definition to set the property initial value, and to identifies the dependencies using [tracking](#tracking)
	///
	/// it returns [`Error::Removed`] if the `slab` is removed.
	///
	/// this method is redirected by every [`ScopedStoreProv`](crate::ScopedStoreProv)ider in a more egornomic format.
	///
	/// # example
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
		ctx: &mut Ctx, slab: Option<SlabId>, fun: impl FnMut(&mut Ctx) -> T + 'static,
	) -> Result<PropId<T>, Error> {
		if let Some(slab) = slab
			&& !ctx.store().has_slab(slab)
		{
			return Err(Error::Removed);
		}
		let (id, effect) = Self::computed_core(ctx, fun);

		if let Some(slab) = slab {
			let slab = ctx.store().slab(slab);
			slab.effects.push(effect);
			slab.props.push(id.0);
		}
		Ok(id)
	}
}

/// <h2 id=slab-managment>Slab Managment</h2>
///
/// the `Store` is composed of multiple independent scopes, each scope have its own lifetime shared accross its own items (properties and effects).
///
/// the default scope the global scope that spans the entire `Store` lifetime.
///
/// the other kinds are slabs that are scopes identified with [`SlabId`] and can be removed when needed.
///
/// all effects dependent on properties inside slabs, they themself should be inside the shortest life slab, this because they would run while their dependencies are removed leading to panics.
///
/// ## example
/// ```
/// let slab = store.create_slab();
/// let count = store.prop_in(slab, 1);
///
/// Store::remove_slab(ctx, slab);
/// assert!(!store.has_slab(slab));
/// assert!(!store.contains(count));
/// ```
impl<Ctx: Context> Store<Ctx> {
	/// creates a slab, returning its [`SlabId`].
	///
	/// # example
	/// ```
	/// let slab = store.create_slab();
	/// ```
	pub fn create_slab(&mut self) -> SlabId {
		let id = self.next_slab;
		self.slabs.insert(id, SlabData::default());
		self.next_slab = SlabId(id.0 + 1);
		id
	}

	/// return a mutable reference to `SlabData`
	fn slab(&mut self, slab: SlabId) -> &mut SlabData<Ctx> {
		self.slabs.get_mut(&slab).unwrap()
	}

	/// check whether a slab is inside the `Store`.
	///
	/// in some [circumstances](#remove-slab-note), a slab may be marked removed but its items are not.
	///
	/// # example
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

	/// remove the given slab.
	///
	/// this methods will drop all the slab items and return [`Error::Removed`] if the given `slab` was previously removed.
	///
	/// <h4 id=remove-slab-note>note</h4>
	///
	/// during updating inside effects, removed slabs will not be dropped instantly, instead they will be marked removed and will be removed when the updating end.
	///
	/// # example
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
			// since we would need to clean the current effect queue, and it is ineffecient and rarely necessary.
			store.slabs_to_remove.push(id);
		} else {
			Store::drop_slab(ctx, id);
		}
		Ok(())
	}
	/// remove the slab for real
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
impl<Ctx: Context> Store<Ctx> {}

/// <h2 id=tracking>Tracking</h2>
///
/// tracking is a mechanism that allow to identify the properties used in a chunk of normal code without any extra syntax.
///
/// it starts with [`start_track`](Store::start_track), then every call to a [property access method](#property-access) record the target property as read or write, and at the end a call to [`end_track`](Store::end_track) retuns the recorded properties in a [`TrackResult`].
///
/// # example
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
	/// check whether tracking is activated.
	///
	/// # example
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

	/// activate tracking.
	///
	/// returns [`Error::Tracking`] if tracking was activated before.
	pub fn start_track(&self) -> Result<(), Error> {
		if self.is_tracking() {
			return Err(Error::Tracking);
		}
		self.tracking.replace(Some(TrackResult::default()));
		Ok(())
	}

	/// stop tracking and return the [`TrackResult`].
	///
	/// returns [`Error::NotTracking`] if tracking was not activated before.
	pub fn end_track(&self) -> Result<TrackResult, Error> {
		let mut result = self.tracking.take().ok_or(Error::NotTracking)?;

		// deduping at the end is faster than on every call
		result.read.sort_unstable();
		result.read.dedup();
		result.written.sort_unstable();
		result.written.dedup();

		Ok(result)
	}

	/// record a property as read while tracking.
	///
	/// it is noop while not tracking.
	///
	/// # example
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

	/// record a property as written while tracking.
	// like that because in `get_mut` requires partial borrow
	fn _track_write<T: 'static>(tracking: &RefCell<Option<TrackResult>>, id: PropId<T>) {
		if let Some(tracking) = tracking.borrow_mut().deref_mut() {
			tracking.written.push(id.erase_type());
		}
	}

	/// record a property as written while tracking.
	///
	/// it is noop while not tracking.
	///
	/// # example
	/// ```
	/// let a = store.prop(1);
	/// store.start_track();
	/// store.track_write(a);
	/// assert_eq!(store.end_track().unwrap().written, [a.erase_type()]);
	/// ```
	pub fn track_write<T: 'static>(&self, id: PropId<T>) {
		Self::_track_write(&self.tracking, id);
	}
}

/// <h2 id=store-managment>Store Managment</h2>
impl<Ctx: Context> Store<Ctx> {}

impl<Ctx: Context> Store<Ctx> {
	pub fn add_global_cleaner(&mut self, fun: impl FnOnce(&mut Ctx) + 'static) {
		self.global_cleaners.push(Box::new(fun))
	}
	pub fn add_cleaner_in(
		&mut self, slab: Option<SlabId>, fun: impl FnOnce(&mut Ctx) + 'static,
	) -> Result<(), Error> {
		let Some(slab) = slab else {
			self.add_global_cleaner(fun);
			return Ok(());
		};
		if !self.has_slab(slab) {
			return Err(Error::Removed);
		}
		self.slab(slab).cleaner.push(Box::new(fun));
		Ok(())
	}

	pub fn is_updating(&self) -> bool {
		self.updater.is_updating
	}

	pub fn force_update<T: 'static>(&mut self, id: PropId<T>) {
		if !self.updater.dirty_props.contains(&id.0) {
			self.updater.dirty_props.push(id.0);
		}
	}
	pub fn flush_updates(ctx: &mut Ctx) {
		Updater::update(ctx);

		while let Some(slab) = ctx.store().slabs_to_remove.pop() {
			Store::drop_slab(ctx, slab);
		}
	}

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
