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
//! `neoview-web` features its own context, [`DomContext`], it borrows the reactive system from [`neoview`] and provides its own flavor of the [`chunk`](macro@chunk) macro with full HTML support.
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
mod utility;

/// other documentation sections.
#[cfg(doc)]
pub mod docs {
	pub mod guide;
}

#[doc(hidden)]
#[allow(unused_imports)]
pub mod __private {
	use super::*;
	use neoview::PropId;
	/// Constructs a UI chunk in an expressive, object-like syntax.
	///
	/// This `chunk` macro inherits the expressiveness of the [`neoview`] [`chunk`](https://docs.rs/neoview/latest/neoview/macro.chunk.html) macro and adds an HTML flavor to it.
	///
	/// The `chunk` macro requires the [`__buildcodes`](prelude::__buildcode) module to be in scope.
	///
	/// The `chunk` macro is the recommended way to construct UIs, however, you can use the [`apply`] module if you prefer a native builder pattern.
	///
	/// The [`html-types`](crate#html-types) and [`css-types`](crate#css-types) crate features provide IntelliSense for tags, attributes, events, and CSS properties.
	///
	/// # Example
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
	///         div(style.color: if i % 2 == 0 { "red" } else { "blue" }) { "item ", i }
	///     }
	/// });
	/// ```
	///
	/// # Syntax
	/// The `build` argument must be a [`ChunkBuild`].
	///
	/// ### `ComputedExpr<T>`
	/// Computed expressions are closures of type `FnMut(&mut DomContext) -> T` passed as attribute values and content.
	///
	/// They act as a kind of dynamic binding: each time one of the properties they read is updated, they are re-evaluated and the new value is applied to the target.
	///
	/// ## Elements
	/// Elements are defined using the [element syntax](https://docs.rs/neoview/latest/neoview/macro.chunk.html#element), allowing them to have attributes, children, or both.
	///
	/// Tags can be an identifier representing a valid HTML tag, or a string literal for custom tags.
	///
	/// ```
	/// chunk!(build,
	///     div { "content" }
	///     div(id: "my-id") { "attrs with content" }
	///     "web-component"(id: "my-id")
	/// );
	/// ```
	/// ## Attributes
	/// Attributes are applied statically or dynamically based on a value.
	///
	/// Attribute names can be:
	/// - An identifier representing a valid HTML attribute, any `_` characters are replaced with `-`.
	/// - A kebab-case name composed of identifiers separated by `-`.
	/// - A string literal for custom attribute names.
	///
	/// ```
	/// chunk!(build, div(
	///     id: "my-id",
	///     data-test: "kebab-case",
	///     aria_description: "snake_case",
	///     "@ns.custom[attr]": "any attr"
	/// ));
	/// ```
	/// #### Attribute Values
	/// Attribute values are expressions that can be of the following types:
	/// - [`&str`](str), [`String`], [`char`]: Set directly.
	/// - [`bool`]: Toggles the attribute based on its boolean value.
	/// - [`i8`], [`i16`], [`i32`], [`i64`], [`i128`], [`isize`], [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], [`usize`], [`f32`], [`f64`]: Stringified and then set.
	/// - [`Option<T>`]: If `Some(T)`, applies `T`, otherwise, removes the attribute.
	/// - `&T`: Applies `T`.
	/// - [`PropId<T>`]: Every time the property is updated, its new value is applied to the attribute.
	/// - [`ComputedExpr<T>`](#computedexprt): Its evaluated value is applied to the attribute.
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
	/// ## Special Attributes
	/// ### `class.name`
	/// The class `name` is toggled statically or dynamically based on a value.
	///
	/// The `name` can be:
	/// - An identifier where any `_` characters are replaced with `-`.
	/// - A kebab-case name composed of identifiers separated by `-`.
	/// - A string literal for custom class names.
	///
	/// The value can be:
	/// - [`bool`]: Toggles the class at build time.
	/// - [`PropId<bool>`]: Every time the property is updated, the class is toggled based on its new value.
	/// - [`ComputedExpr<bool>`](#computedexprt): Its evaluated value is used to toggle the class.
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
	/// The CSS property `prop` is applied statically or dynamically based on a value.
	///
	/// The `prop` can be:
	/// - An identifier where any `_` characters are replaced with `-`.
	/// - A kebab-case name composed of identifiers separated by `-`.
	/// - A string literal for complex property names.
	///
	/// The value can be:
	/// - [`&str`](str), [`String`]: Set directly.
	/// - [`Option<T>`]: If `Some(T)`, applies `T`, otherwise, removes the CSS property.
	/// - `&T`: Applies `T`.
	/// - [`PropId<T>`]: Every time the property is updated, its new value is applied to the CSS property.
	/// - [`ComputedExpr<T>`](#computedexprt): Its evaluated value is applied to the CSS property.
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
	/// The element property `name` is bound statically or dynamically based on a value.
	///
	/// The `name` can be an identifier or a string literal.
	///
	/// The value can be:
	/// - [`JsValue`](wasm_bindgen::JsValue): Set at build time.
	/// - [`PropId<JsValue>`]: Every time the property is updated, its new value is assigned to the element property.
	/// - [`ComputedExpr<JsValue>`](#computedexprt): Its evaluated value is assigned to the element property.
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
	/// Adds an event listener to the specified `event`.
	///
	/// The listener is an `FnMut` called with `(&mut DomContext, Event)`, after it is called, updates are [flushed](neoview::Store::flush_updates).
	///
	/// The `event` can be a valid HTML event name or a string literal.
	///
	/// ```
	/// let count = build.prop(0);
	/// chunk!(build,
	///     button(on.click: (move |ctx, _| *ctx.read_mut(count) += 1))
	///     div(on."custom_event", (move |ctx, _| println!("custom event!")))
	/// );
	/// ```
	///
	/// ## Content
	/// ### Text Content
	/// Text content is an expression placed as a child of an element; its evaluated value is inserted as a text node within the element.
	///
	/// Its value can be:
	/// - [`&str`](str), [`String`], [`char`]: Inserted directly.
	/// - [`bool`], [`i8`], [`i16`], [`i32`], [`i64`], [`i128`], [`isize`], [`u8`], [`u16`], [`u32`], [`u64`], [`usize`], [`u128`], [`f32`], [`f64`]: Stringified and inserted.
	/// - [`Option<T>`]: If `Some(T)`, inserts `T`, otherwise, sets the text content to `""`.
	/// - `&T`: Inserts `T`.
	/// - [`PropId<T>`]: Every time the property is updated, the text node is updated with its new value.
	/// - [`ComputedExpr<T>`](#computedexprt): The text node is updated with the evaluated value.
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
	/// ### Node Content
	/// Node content is an expression placed as a child of an element, its evaluated value is inserted as a child node within that element.
	///
	/// Its value can be:
	/// - [`Into<Node>`](web_sys::Node): Inserted at build time.
	/// - [`PropId<Into<Node>>`]: Every time the property is updated, the node is replaced with its new value.
	/// - [`ComputedExpr<Into<Node>>`](#computedexprt): The node is replaced with the evaluated value.
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
	context::{ContextId, CtxHandle, CtxOptions, DomContext, get_ctx, new_ctx, use_ctx},
	list_render::{render_list, render_list_enumerated},
	neoview,
	utility::show_if,
};

/// `neoview-web` prelude.
pub mod prelude {
	pub use crate::{
		__private::chunk_export as chunk, ChunkBuild, DomContext, build_codes::__buildcode,
	};
	pub use neoview::{PropId, ScopedStoreProv, StoreProv};
}
