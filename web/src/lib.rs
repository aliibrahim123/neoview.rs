//! # NeoView Web
//! The web renderer for [`neoview`].
//!
//! `neoview-web` is the official web renderer for the [`neoview`] framework, based on [`HTML`](https://developer.mozilla.org/en-US/docs/Web/HTML) and [`DOM`](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model) technologies and interfacing through the [`wasm-bindgen`](::wasm_bindgen) crate.
//!
//! Aligned with [`neoview`]'s core principles, `neoview-web` supports ergonomic, fully reactive UI definitions with a strong emphasis on safety, robustness, and efficiency.
//!
//! Like every [`neoview`] renderer, it utilizes the efficiency of [fine-grained reactivity](neoview#reactive-system), the safety of [context passing](neoview#reactive-system), and the ergonomics of [chunked templating](neoview#templating) to provide high-level expressiveness with low-level robustness.
//!
//! # Features
//! `neoview-web` features its own context, [`DomContext`]; it borrows the reactive system from [`neoview`] and provides its own flavor of the [`chunk`](macro@chunk) macro with full HTML support.
//!
//! In addition to the feature-richness of [`chunk`](macro@chunk), `neoview-web` includes [conditional rendering](apply::show_if), [list rendering](render_list), a [builder pattern](apply) for templating, and [IntelliSense for tags, attributes, events, and CSS properties](#html-types).
//!
//! Here is a simple example without the initialization boilerplate:
//! ```
//! chunk!(build, div {
//!     h3 { "Hello world!" }
//!     do {
//!         let count = build.prop(0);
//!         chunk!(build, button(
//!             on.click: (move |ctx, _| ctx.update(count, |v| *v += 1))
//!         ) { "count: ", count });
//!     }
//! });
//! ```
//!
//! For a more in-depth introduction, check out the [guide section](docs::guide).
//!
//! # Crate Features
//! ### `html-types`
//! Provides IntelliSense for HTML tags, attributes, and events (autocompletion and hover descriptions).
//!
//! It is an optional, quality-of-life feature with no runtime cost.
//! ### `css-types`
//! Provides IntelliSense for CSS properties (autocompletion and hover descriptions).
//!
//! It is an optional, quality-of-life feature with no runtime cost.
//!
//! It requires the `html-types` feature.
pub mod apply;
mod bindings;
mod build_codes;
mod chunk;
mod context;
#[doc(hidden)]
#[cfg(feature = "css-types")]
pub mod css_props;
#[cfg(feature = "html-types")]
pub mod html_types;
mod list_render;

/// other documentation sections.
#[cfg(doc)]
pub mod docs {
	pub mod guide;
}

#[doc(hidden)]
pub mod __private {
	use super::*;
	use neoview::PropId;
	/// Constructs a UI chunk in an expressive, object-like syntax.
	///
	/// this `chunk` macro inherent the expressiveness of [`neoview`] [`chunk`](https://docs.rs/neoview/latest/neoview/macro.chunk.html) macro and add its html flavor over it.
	///
	/// `chunk` requires the [`__buildcodes`](prelude::__buildcode) module to be in scope.
	///
	/// `chunk` is the recommended way to construct UI, however they are the [`apply`] module if you like a native builder pattern.
	///
	/// the [`html-types`](crate#html-types) and [`css-types`](crate#css-types) crate features provide intellisense for tags, attributes, events, and CSS properties.
	///
	/// # example
	/// ```
	/// chunk!(build, div {
	///     h3 { "Hello world!" }
	///     do {
	///         let count = build.prop(0);
	///         chunk!(build, button(
	///             on.click: (move |ctx, _| ctx.update(count, |v| *v += 1))
	///         ) { "count: ", count });
	///     }
	///     for i in 0..10 {
	///         chunk!(build, div(style.color: if i % 2 == 0 { "red" } else { "blue" }) { "item ", i });
	///     }
	/// });
	/// ```
	///
	/// # syntax
	/// the `build` argument must be a [`ChunkBuild`].
	///
	/// ### `ComputedExpr<T>`
	/// computed expressions are closures of type `FnMut(&mut DomContext) -> T` passed as attribute values and content.
	///
	/// the are kind of dynamic binding where each time one of the properties they read is updated, they get reevaluated and the new value is set to the target.
	///
	/// ## elements
	/// elements are defined though the [element syntax](https://docs.rs/neoview/latest/neoview/macro.chunk.html#element) where they can have attributes, children, or both.
	///
	/// tags can be an identifier that is a valid html tag, or a string literal for custom tags.
	///
	/// ```
	/// chunk!(build,
	///     div { "content" }
	///     div(id: "my-id") { "attrs with content" }
	///     "web-component"(id: "my-id")
	/// );
	/// ```
	/// ## attributes
	/// attributes are applied statically or dynamically based on a value.
	///
	/// attributes names can be:
	/// - an identifier that is valid html attribute, the `_` in it are replaced with `-`.
	/// - kebab case name written with identifiers separated by `-`.
	/// - string literal for custom attribute names.
	///
	/// ```
	/// chunk!(build, div(
	///     id: "my-id",
	///     data-test: "kebab-case",
	///     aria_description: "snake_case",
	///     "@ns.custom[attr]": "any attr"
	/// ));
	/// ```
	/// #### attribute values
	/// attribute values are expressions that can be of type:
	/// - [`&str`](str), [`String`], [`char`]: set directly.
	/// - [`bool`]: toggle attrubute based on it.
	/// - [`i8`], [`i16`], [`i32`], [`i64`], [`i128`], [`isize`], [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], [`usize`], [`f32`], [`f64`]: stringify then set.
	/// - [`Option<T>`]: if `Some(T)` then apply T, else remove the attribute.
	/// - `&T`: apply T.
	/// - [`PropId<T>`]: everythime the property is updated, its value is applied to the attribute.
	/// - [`ComputedExpr<T>`](#computedexprt): its evaluated value is applied to the attribute.
	/// ```
	/// let value = build.prop(10);
	/// chunk!(build, progress(
	///     id: "progress",
	///     hidden: false,
	///     max: Some(&100),
	///     value,
	///     class: move |ctx| (ctx.read(value).len() <= 10).then_some("error"),
	/// ));
	/// ```
	///
	/// ## special attributes
	/// ### `class.name`
	/// the class `name` is toggled statically and dynamically based on a value.
	///
	/// the `name` can be:
	/// - an identifier where the `_` in it are replaced with `-`.
	/// - kebab case name written with identifiers separated by `-`.
	/// - string literal for custom class names.
	///
	/// the value can be:
	/// - [`bool`]: toggle the class at build time.
	/// - [`PropId<bool>`]: everytime the property is updated, the class is toggled based on its value.
	/// - [`ComputedExpr<bool>`](#computedexprt): its evaluated value is used to toggle the class.
	///
	/// ```
	/// let active = build.prop(false);
	/// chunk!(build, div(
	///     class.round-box: true,
	///     class.active: active,
	///     class."px-2": move |ctx| !ctx.get(active),
	/// ));
	/// ```
	///
	/// ### `style.prop`
	/// the css property `prop` is applied statically and dynamically based on a value.
	///
	/// the `prop` can be:
	/// - an identifier where the `_` in it are replaced with `-`.
	/// - kebab case name written with identifiers separated by `-`.
	/// - string literal for complex property names.
	///
	/// the value can be:
	/// - [`&str`](str), [`String`]: set directly.
	/// - [`Option<T>`]: if `Some(T)` then apply T, else remove the property.
	/// - `&T`: apply T.
	/// - [`PropId<T>`]: everythime the property is updated, its value is applied to the css property.
	/// - [`ComputedExpr<T>`](#computedexprt): its evaluated value is applied to the css property.
	///
	/// ```
	/// let color = build.prop(String::from("red"));
	/// chunk!(build, div(
	///     style.background_color: "red",
	///     style.font-size: Some(&"10px"),
	///     style.color: color,
	///     style."--complement-color": move |ctx| if ctx.read(color) == "red" { "blue" } else { "red" },
	/// ));
	/// ```
	///
	/// ### `prop.name`
	/// the element property `name` is binded statically and dynamically based on a value.
	///
	/// the `name` can be an identifier or a string literal.
	///
	/// the value can be:
	/// - [`JsValue`](wasm_bindgen::JsValue): set at build time.
	/// - [`PropId<JsValue>`]: everythime the property is updated, its value is set to the property.
	/// - [`ComputedExpr<JsValue>`](#computedexprt): its evaluated value is set to the property.
	///
	/// ```
	/// let html = build.prop(JsValue::from("hello <b>world</b>"));
	/// chunk!(build, div(
	///     prop.innerText: JsValue::from("hello world"),
	///     prop.innerHTML: html,
	///     prop."$data123": JsValue::from("some data"),
	/// ));
	/// ```
	///
	/// ### `on.event`
	/// add an event listener to the event `event`.
	///
	/// the listener is an `FnMut` called with `(&mut DomContext, Event)` and after it is called, updates are [flushed](neoview::Store::flush_updates).
	///
	/// `event` can be a valid html event name, or a string literal.
	///
	/// ```
	/// let count = build.prop(0);
	/// chunk!(build,
	///     button(on.click: (move |ctx, _| *ctx.read_mut(count) += 1))
	///     div(on."custom_event", (move |ctx, _| println!("custom event!")))
	/// );
	/// ```
	///
	/// ## content
	/// ### text content
	/// the text content is an expression placed as a child of an element, its evaluated value is inserted as text node in the element.
	///
	/// its value can be:
	/// - [`&str`](str), [`String`], [`char`]: inserted directly.
	/// - [`bool`], [`i8`], [`i16`], [`i32`], [`i64`], [`i128`], [`isize`], [`u8`], [`u16`], [`u32`], [`u64`], [`usize`], [`u128`], [`f32`], [`f64`]: stringified and inserted.
	/// - [`Option<T>`]: if `Some(T)` then insert T, else set the text content to `""`.
	/// - `&T`: insert T.
	/// - [`PropId<T>`]: everythime the property is updated, the text node is updated with its value.
	/// - [`ComputedExpr<T>`](#computedexprt): the text node is updated with the evaluated value.
	///
	/// ```
	/// let text = build.prop(String::from("abc"));
	/// chunk!(div {
	///     "hello world",
	///     1 + 2,
	///     Some(&true),
	///     text,
	///     move |ctx| ctx.read(text).len(),
	/// });
	/// ```
	///
	/// ### node content
	/// the node content is an expression placed as a child of an element, its evaluated value is inserted as a child of that element.
	///
	/// its value can be:
	/// - [`Into<Node>`](web_sys::Node): inserted at build time.
	/// - [`PropId<Into<Node>>`]: everythime the property is updated, the node is replaced with its value.
	/// - [`ComputedExpr<Into<Node>>`](#computedexprt): the node is replaced with the evaluated value.
	///
	/// ```
	/// let el = document().unwrap().create_element("div").unwrap();
	/// let prop = build.prop(el.clone());
	/// chunk!(build, div {
	///     el,
	///     prop,
	///     move |ctx| ctx.read(prop).first_child().unwrap()
	/// });
	/// ```
	#[macro_export]
	macro_rules! chunk {
        ($($t:tt)*) => { ::neoview_web::__private::real_chunk!($($t)*) };
    }
	pub use chunk as chunk_export;
	pub use neoview_macro::chunk as real_chunk;
}

pub use {
	chunk::{ChunkBuild, ChunkRemover, RemovableChunk},
	context::{ContextId, CtxHandle, CtxOptions, DomContext, get_ctx, use_ctx},
	list_render::{render_list, render_list_enumerated},
	neoview,
};

/// `neoview-web` prelude.
pub mod prelude {
	pub use crate::{
		__private::chunk_export as chunk, ChunkBuild, DomContext, build_codes::__buildcode,
	};
	pub use neoview::{PropId, ScopedStoreProv, StoreProv};
}
