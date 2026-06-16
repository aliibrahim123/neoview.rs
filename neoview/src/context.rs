//! Defines the [`Context`] and [`StoreProv`] family.
use crate::{PropId, SlabId, Store, store::EffectDeps};

/// A type that owns the UI.
///
/// The `Context` is the root of the UI system that manages everything from rendering, reactivity, and bindings. It is the single owner that destroys the entire UI when it is dropped.
///
/// Every interaction with the UI and its reactive system requires mutable access to the `Context`. Therefore, the `Context` is borrowed and passed around for short periods of time whenever an interaction occurs.
///
/// Beyond the philosophy of context passing and ownership, `neocomp` requires `Context`s to implement [`GlobalStoreProv`], which exposes its [`Store`] through [`StoreProv::store`] and [`StoreProv::store_ref`].
///
/// # Example
/// ```
/// struct SomeContext(Store<Self>);
/// impl Context for SomeContext {}
/// impl GlobalStoreProv for SomeContext {}
/// impl StoreProv for SomeContext {
///     type Ctx = Self;
///     fn ctx(&mut self) -> &mut Self::Ctx { self }
///     fn ctx_ref(&self) -> &Self::Ctx { self }
///     fn store(&mut self) -> &mut Store<Self::Ctx> { &mut self.0 }
///     fn store_ref(&self) -> &Store<Self::Ctx> { &self.0 }
/// }
/// ```
pub trait Context: Sized + GlobalStoreProv<Ctx = Self> {}

/// A type that provides access to a [`Store`].
///
/// The `StoreProv` trait redirects the common methods of the wrapped [`Store`] (such as [`read`](Store::read), [`get`](Store::get), and [`write`](Store::write)) for better ergonomics.
///
/// ```
/// let count = provider.ctx().store().prop(1);
/// assert_eq!(provider.ctx().store().read(count), &1);
/// provider.ctx().store().write(count, 2);
/// assert_eq!(provider.ctx().store().get(count), 2);
/// provider.ctx().store().update(count, |v| *v += 1);
/// assert_eq!(provider.ctx().store().peek(count), &3);
///
/// // vs
/// let count = provider.prop(1);
/// assert_eq!(provider.read(count), &1);
/// provider.write(count, 2);
/// assert_eq!(provider.get(count), 2);
/// provider.update(count, |v| *v += 1);
/// assert_eq!(provider.peek(count), &3);
/// ```
///
/// To gain this ability, the implementer must provide access to its [`Context`] through [`ctx`](StoreProv::ctx) and [`ctx_ref`](StoreProv::ctx_ref).
///
/// ```
/// struct Provider<'a>(&'a mut SomeContext);
/// impl<'a> StoreProv for Provider<'a> {
///     type Ctx = SomeContext;
///     fn ctx(&mut self) -> &mut Self::Ctx { self.0 }
///     fn ctx_ref(&self) -> &Self::Ctx { self.0 }
/// }
/// ```
///
/// This trait redirects the [property access methods](Store#property-access). For item definition redirection, see the [`ScopedStoreProv`] trait.
pub trait StoreProv {
	/// The [`Context`] containing the provided [`Store`].
	type Ctx: Context;
	/// Returns a mutable reference to the [`Context`].
	///
	/// Generally, it is sufficient to implement this along with [`ctx_ref`](Self::ctx_ref) when implementing `StoreProv`.
	fn ctx(&mut self) -> &mut Self::Ctx;
	/// Returns a reference to the [`Context`].
	///
	/// Generally, it is sufficient to implement this along with [`ctx`](Self::ctx) when implementing `StoreProv`.
	fn ctx_ref(&self) -> &Self::Ctx;
	/// Returns a mutable reference to the provided [`Store`].
	///
	/// It has a default implementation that uses `self.ctx().store()`, override it if necessary.
	fn store(&mut self) -> &mut Store<Self::Ctx> {
		self.ctx().store()
	}
	/// Returns a reference to the provided [`Store`].
	///
	/// It has a default implementation that uses `self.ctx_ref().store_ref()`, override it if necessary.
	fn store_ref(&self) -> &Store<Self::Ctx> {
		self.ctx_ref().store_ref()
	}

	/// Returns a reference to a reactive property's value.
	///
	/// This method redirects [`Store::read`].
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// assert_eq!(provider.read(nb), &1);
	/// ```
	fn read<T: 'static>(&self, id: PropId<T>) -> &T {
		self.store_ref().read(id)
	}

	/// Returns a copy of a [`Copy`]able reactive property's value.
	///
	/// This method redirects [`Store::get`].
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// assert_eq!(provider.get(nb), 1);
	/// ```
	fn get<T: 'static + Copy>(&self, id: PropId<T>) -> T {
		self.store_ref().get(id)
	}

	/// Returns a reference to a reactive property's value without tracking it.
	///
	/// This method redirects [`Store::peek`].
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// assert_eq!(provider.peek(nb), &1);
	/// ```
	fn peek<T: 'static>(&self, id: PropId<T>) -> &T {
		self.store_ref().peek(id)
	}

	/// Returns a mutable reference to a reactive property's value.
	///
	/// This method redirects [`Store::read_mut`].
	///
	/// # Example
	/// ```
	/// let arr = provider.prop(vec![1, 2, 3]);
	/// provider.read_mut(arr)[2] = 4;
	/// assert_eq!(provider.read(arr), &[1, 2, 4]);
	/// ```
	fn read_mut<T: 'static>(&mut self, id: PropId<T>) -> &mut T {
		self.store().read_mut(id)
	}

	/// Sets a reactive property's value.
	///
	/// This method redirects [`Store::write`].
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// provider.write(nb, 2);
	/// assert_eq!(provider.get(nb), 2);
	/// ```
	fn write<T: 'static>(&mut self, id: PropId<T>, value: T) -> T {
		self.store().write(id, value)
	}

	/// Updates a reactive property using an updater function.
	///
	/// This method redirects [`Store::update`].
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// provider.update(nb, |v| *v += 1);
	/// assert_eq!(provider.get(nb), 2);
	/// ```
	fn update<T: 'static>(&mut self, id: PropId<T>, fun: impl FnOnce(&mut T)) {
		self.store().update(id, fun)
	}
}

/// A [`StoreProv`] with a specific scope.
///
/// The `ScopedStoreProv` is a trait that extends [`StoreProv`] by further redirecting the item definition methods ([`prop`](Store::prop), [`effect`](Store::effect), and [`computed`](Store::computed)) from the provided [`Store`].
///
/// ```
/// let nb = provider.ctx().store().prop_in(Some(provider.slab()), 1);
/// Store::effect(provider.ctx(), Some(provider.slab()), EffectDeps::Tracked,
///     move |ctx| println!("nb: {}", ctx.store().get(nb))
/// );
/// let doubled = Store::computed(provider.ctx(), Some(provider.slab()),
///     move |ctx| ctx.store().get(nb) * 2
/// );
///
/// // vs
/// let nb = provider.prop(1);
/// provider.effect(move |ctx| println!("nb: {}", ctx.get(nb)));
/// let doubled = provider.computed(move |ctx| ctx.get(nb) * 2);
/// ```
///
/// To gain this ability, the implementer must specify its scope through [`slab`](ScopedStoreProv::slab).
/// ```
/// struct Provider<'a>(&'a mut SomeContext, SlabId);
/// impl<'a> StoreProv for Provider<'a> {
///     type Ctx = SomeContext;
///     fn ctx(&mut self) -> &mut Self::Ctx { self.0 }
///     fn ctx_ref(&self) -> &Self::Ctx { self.0 }
/// }
/// impl<'a> ScopedStoreProv for Provider<'a> {
///     fn slab(&self) -> Option<SlabId> { Some(self.1) }
/// }
/// ```
pub trait ScopedStoreProv: StoreProv {
	/// Returns the [`SlabId`] of the slab the provider is scoped to.
	///
	/// Its return value must always be the same and must exist in the [`Store`]; it can be [`None`] for the global scope.
	fn slab(&self) -> Option<SlabId>;

	/// Defines a new reactive property in the provider's scope.
	///
	/// This method redirects [`Store::prop_in`].
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// assert_eq!(provider.get(nb), 1);
	/// ```
	fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let slab = self.slab();
		self.store().prop_in(slab, value).unwrap()
	}

	/// Defines an effect in the provider's scope.
	///
	/// This method redirects [`Store::effect`] with [`deps: EffectDeps::Tracked`](EffectDeps::Tracked).
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// provider.effect(move |ctx| println!("nb: {}", ctx.get(nb)));
	/// ```
	fn effect(&mut self, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		let slab = self.slab();
		Store::effect(self.ctx(), slab, EffectDeps::Tracked, fun).unwrap();
	}

	/// Defines an effect in the provider's scope with manual dependencies.
	///
	/// This method redirects [`Store::effect`] with [`deps: EffectDeps::Manual { init_run: true }`](EffectDeps::Manual).
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// provider.effect_manual(vec![nb.erase_type()], vec![], move |ctx| println!("nb: {}", ctx.get(nb)));
	/// ```
	fn effect_manual(
		&mut self, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Self::Ctx) + 'static,
	) {
		let slab = self.slab();
		Store::effect(self.ctx(), slab, EffectDeps::Manual { read, write, init_run: true }, fun)
			.unwrap();
	}

	/// Creates a computed property in the provider's scope.
	///
	/// This method redirects [`Store::computed`].
	///
	/// # Example
	/// ```
	/// let nb = provider.prop(1);
	/// let doubled = provider.computed(move |ctx| ctx.get(nb) * 2);
	/// assert_eq!(provider.get(doubled), 2);
	/// ```
	fn computed<T: 'static>(
		&mut self, fun: impl FnMut(&mut Self::Ctx) -> T + 'static,
	) -> PropId<T> {
		let slab = self.slab();
		Store::computed(self.ctx(), slab, fun).unwrap()
	}
}

/// A [`ScopedStoreProv`] specialized for the global scope.
///
/// `GlobalStoreProv` is a marker trait that automatically implements [`ScopedStoreProv`] for the global scope on the implementing [`StoreProv`].
///
/// ```
/// struct Provider<'a>(&'a mut SomeContext);
/// impl<'a> StoreProv for Provider<'a> {
///     type Ctx = SomeContext;
///     fn ctx(&mut self) -> &mut Self::Ctx { self.0 }
///     fn ctx_ref(&self) -> &Self::Ctx { self.0 }
/// }
/// impl<'a> GlobalStoreProv for Provider<'a> {}
/// ```
pub trait GlobalStoreProv: ScopedStoreProv {}

impl<T: GlobalStoreProv> ScopedStoreProv for T {
	fn slab(&self) -> Option<SlabId> {
		None
	}
}
