//! define [`Context`] and [`StoreProv`] family.
use crate::{PropId, SlabId, Store, store::EffectDeps};

/// a type that owns the ui.
///
/// the `Context` is the root of the ui system that manages everything from rendering, reactivity and bindings, it is the one owner that with its drop the whole ui is destroyed.
///
/// every interaction with ui and its reactive system requires a mutable access to the `Context`, for that the `Context` is borrowed and passed around for short periods of time every time an interaction occurs.
///
/// other than the phelosophy of context passing and ownership, and being real ui systems, neocomp requires from `Context`s to be a [`GlobalStoreProv`] that exposes its [`Store`] through [`StoreProv::store`] and [`StoreProv::store_ref`].
///
/// # example
/// ```
/// struct SomeContext(Store<Self>);
/// impl Context for SomeContext {}
/// impl GlobalStoreProv for SomeContext {}
/// impl StoreProv for SomeContext {
/// 	type Ctx = Self;
/// 	fn ctx(&mut self) -> &mut Self::Ctx { self }
/// 	fn ctx_ref(&self) -> &Self::Ctx { self }
/// 	fn store(&mut self) -> &mut Store<Self::Ctx> { &mut self.0 }
/// 	fn store_ref(&self) -> &Store<Self::Ctx> { &self.0 }
/// }
/// ```
pub trait Context: Sized + GlobalStoreProv<Ctx = Self> {}

/// a type that provides access to a [`Store`].
///
/// the `StoreProv`ider is a trait that redirect the common methods of the [`Store`] ([`read`](Store::read), [`get`](Store::get), [`write`](Store::write), ...) the implementer wraps.
///
/// ```
/// let count = provider.ctx.store.prop(1);
/// assert!(provider.ctx.store.read(count), 1);
/// provider.ctx.store.write(count, 2);
/// assert!(provider.ctx.store.get(count), 2);
/// provider.ctx.store.update(count, |v| *v += 1);
/// assert!(provider.ctx.store.peek(count), 3);
///
/// // vs
/// let count = provider.prop(1);
/// assert!(provider.read(count), 1);
/// provider.write(count, 2);
/// assert!(provider.get(count), 2);
/// provider.update(count, |v| *v += 1);
/// assert!(provider.peek(count), 3);
/// ```
///
/// to gain this ability, the implementer must provide access to its [`Context`] though [`ctx`](StoreProv::ctx) and [`ctx_ref`](StoreProv::ctx_ref).
///
/// ```
/// struct Provider<'a>(&mut 'a SomeContext).
/// impl<'a> StoreProv for Provider<'a> {
/// 	type Ctx = SomeContext;
/// 	fn ctx(&mut self) -> &mut Self::Ctx { self.0 }
/// 	fn ctx_ref(&self) -> &Self::Ctx { self.0 }
/// }
/// ```
///
/// this trait redirects the [property access methods](Store#property-access), for item definition redirection, see [`ScopedStoreProv`] supertrait.
pub trait StoreProv {
	/// the [`Context`] containing the provided [`Store`].
	type Ctx: Context;
	/// return a mutable reference to the [`Context`].
	///
	/// generaly it is enough to implement this along with [`ctx_ref`](Self::ctx_ref) when implementing `StoreProv`.
	fn ctx(&mut self) -> &mut Self::Ctx;
	/// return a reference to the [`Context`].
	///
	/// generaly it is enough to implement this along with [`ctx`](Self::ctx) when implementing `StoreProv`.
	fn ctx_ref(&self) -> &Self::Ctx;
	/// return a mutable reference to the provided [`Store`].
	///
	/// it has a default implementation that uses `StoreProv::store(self.ctx())`, overide it if necessary.
	fn store(&mut self) -> &mut Store<Self::Ctx> {
		self.ctx().store()
	}
	/// return a reference to the provided [`Store`].
	///
	/// it has a default implementation that uses `StoreProv::store_ref(self.ctx_ref())`, overide it if necessary.
	fn store_ref(&self) -> &Store<Self::Ctx> {
		self.ctx_ref().store_ref()
	}

	/// return a reference to a reactive property value.
	///
	/// this method redirect [`Store::read`].
	///
	/// # example
	/// ```
	/// let nb = provider.prop(1);
	/// assert_eq!(provider.read(nb), 1);
	/// ```
	fn read<T: 'static>(&self, id: PropId<T>) -> &T {
		self.store_ref().read(id)
	}

	/// return a copy of a [`Copy`]able reactive property value.
	///
	/// this method redirect [`Store::get`].
	///
	/// # example
	/// ```
	/// let nb = provider.prop(1);
	/// assert_eq!(provider.get(nb), 1);
	/// ```
	fn get<T: 'static + Copy>(&self, id: PropId<T>) -> T {
		self.store_ref().get(id)
	}

	/// return a reference to a reactive property value without being tracked.
	///
	/// this method redirect [`Store::peek`].
	///
	/// # example
	/// ```
	/// let nb = provider.prop(1);
	/// assert_eq!(provider.peek(nb), 1);
	/// ```
	fn peek<T: 'static>(&self, id: PropId<T>) -> &T {
		self.store_ref().peek(id)
	}

	/// return a mutable reference to a reactive property value.
	///
	/// this method redirect [`Store::read_mut`].
	///
	/// # example
	/// ```
	/// let arr = provider.prop(vec![1, 2, 3]);
	/// provider.get_mut(arr)[2] = 4;
	/// assert_eq!(provider.read(arr), [1, 2, 4]);
	/// ```
	fn read_mut<T: 'static>(&mut self, id: PropId<T>) -> &mut T {
		self.store().read_mut(id)
	}

	/// set a reactive property value.
	///
	/// this method redirect [`Store::write`].
	///
	/// # example
	/// ```
	/// let nb = provider.prop(1);
	/// provider.write(nb, 2);
	/// assert_eq!(provider.get(nb), 2);
	/// ```
	fn write<T: 'static>(&mut self, id: PropId<T>, value: T) -> T {
		self.store().write(id, value)
	}

	/// update a reactive property using an updater function.
	///
	/// this method redirect [`Store::update`].
	///
	/// # example
	/// ```
	/// let nb = provider.prop(1);
	/// provider.update(nb, |v| *v += 1);
	/// assert_eq!(provider.get(nb), 2);
	/// ```
	fn update<T: 'static>(&mut self, id: PropId<T>, fun: impl FnOnce(&mut T)) {
		self.store().update(id, fun)
	}
}

/// [`StoreProv`] with a specifc scope.
///
/// the `ScopedStoreProv` is a supertrait of [`StoreProv`] that further redirect the item definition methods ([`prop`](Store::prop), [`effect`](Store::effect), [`computed`](Store::computed)) from the provided [`Store`].
///
/// ```
/// let nb = provider.ctx.store.prop_in(Some(provider.slab), 1);
/// Store::effect(provider.ctx, Some(provider.slab), EffectDeps::Tracked,
/// 	move |ctx| println!("nb: ", ctx.store.get(nb))
/// );
/// let doubled = Store::computed(provider.ctx, Some(provider.slab),
/// 	move |ctx| ctx.store.get(nb) * 2
/// );
///
/// // vs
/// let nb = provider.prop(1);
/// provider.effect(move |ctx| println!("nb: ", ctx.get(nb)));
/// let doubled = provider.computed(move |ctx| ctx.get(nb) * 2);
/// ```
///
/// to gain this ability, the implementer must specifies its scope through [`slab`](ScopedStoreProv::slab).
/// ```
/// struct Provider<'a>(&mut 'a SomeContext, SlabId).
/// impl<'a> StoreProv for Provider<'a> {
/// 	type Ctx = SomeContext;
/// 	fn ctx(&mut self) -> &mut Self::Ctx { self.0 }
/// 	fn ctx_ref(&self) -> &Self::Ctx { self.0 }
/// }
/// impl<'a> ScopedStoreProv for Provider<'a> {
/// 	fn slab(&self) -> Option<SlabId> { Some(self.1) }
/// }
/// ```
pub trait ScopedStoreProv: StoreProv {
	/// return the [`SlabId`] of the slab the provider is scoped to.
	///
	/// its return value must always be the same and exist in the [`Store`], it can be [`None`] for global scope.
	fn slab(&self) -> Option<SlabId>;

	/// defines a new reactive property in the provider scope.
	///
	/// this method redirect [`Store::prop_in`].
	///
	/// # example
	/// ```
	/// let nb = provider.prop(1);
	/// assert_eq!(provider.get(nb), 1);
	/// ```
	fn prop<T: 'static>(&mut self, value: T) -> PropId<T> {
		let slab = self.slab();
		self.store().prop_in(slab, value).unwrap()
	}

	/// defines an effect in the provider scope.
	///
	/// this method redirect [`Store::effect`] with [`deps: EffectDeps::Tracked`](EffectDeps::Tracked).
	///
	/// # example
	/// ```
	/// let nb = provider.prop(1);
	/// provider.effect(move |ctx| println!("nb: ", ctx.get(nb)));
	/// ```
	fn effect(&mut self, fun: impl FnMut(&mut Self::Ctx) + 'static) {
		let slab = self.slab();
		Store::effect(self.ctx(), slab, EffectDeps::Tracked, fun).unwrap();
	}

	/// defines an effect in the provider scope, with manual dependencies.
	///
	/// this method redirect [`Store::effect`] with [`deps: EffectDeps::Manual { init_run: true }`](EffectDeps::Manual).
	///
	/// # example
	/// ```
	/// let nb = provider.prop(1);
	/// provider.effect_manual(vec![nb.erase_type()], vec![], move |ctx| println!("nb: ", ctx.get(nb)));
	/// ```
	fn effect_manual(
		&mut self, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
		fun: impl FnMut(&mut Self::Ctx) + 'static,
	) {
		let slab = self.slab();
		Store::effect(self.ctx(), slab, EffectDeps::Manual { read, write, init_run: true }, fun)
			.unwrap();
	}

	/// create a computed property in the provider scope.
	///
	/// this method redirect [`Store::computed`].
	///
	/// # example
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

/// [`ScopedStoreProv`] specilized for the global scope.
///
/// `GlobalStoreProv` is a marker trait that automatically implementes [`ScopedStoreProv`] of the global scope for the implementer [`StoreProv`].
///
/// ```
/// struct Provider<'a>(&mut 'a SomeContext, SlabId).
/// impl<'a> StoreProv for Provider<'a> {
/// 	type Ctx = SomeContext;
/// 	fn ctx(&mut self) -> &mut Self::Ctx { self.0 }
/// 	fn ctx_ref(&self) -> &Self::Ctx { self.0 }
/// }
/// impl<'a> GlobalStoreProv for Provider<'a> {}
/// ```
pub trait GlobalStoreProv: ScopedStoreProv {}

impl<T: GlobalStoreProv> ScopedStoreProv for T {
	fn slab(&self) -> Option<SlabId> {
		None
	}
}
