//! Builds a UI using a native builder pattern.
//!
//! The `apply` module provides another way to construct a UI for those who prefer a more native macroless approach using the builder pattern.
//!
//! It consists of two parts: [`Applicable`] types that modify the current element and [`apply`](ChunkBuild::apply) which applies the [`Applicable`] to the current element.
//!
//! # Example
//! ```
//! let count = build.prop(0);
//! build.apply(div((
//!     h1((id("header"), style("color", "red"), text("hello world"))),
//!     button((
//!         on("click", move |ctx, _| *ctx.read_mut(count) += 1), text("count: "), text(count)
//!    )),
//! )));
//! ```
use std::borrow::Cow;

use neoview::{PropId, StoreProv};
use web_sys::Event;

use crate::{
	bindings::{AttrValue, ClassValue, NodeValue, PropValue, StyleValue, TextValue},
	build_codes::__buildcode::add_event,
	prelude::{ChunkBuild, DomContext},
};

/// A type that modifies the current element.
///
/// It takes a [`ChunkBuild`] and performs an action such as changing an attribute, adding a child, doing nothing, or even adding entire UI chunks.
///
/// It is implemented for `FnOnce(&mut ChunkBuild)` and for tuples of `Applicable` up to 12 elements where each `Applicable` is applied in order.
///
/// # Example
/// ```
/// fn counter(build: &mut ChunkBuild) {
///     let count = build.prop(0);
///     build.apply(button((
///         on("click", move |ctx, _| *ctx.read_mut(count) += 1),
///         text(count),
///     )));
/// }
/// build.apply(counter);
/// ```
pub trait Applicable {
	/// Applies the [`Applicable`] to the current element.
	fn apply(self, build: &mut ChunkBuild<'_>);
}

impl<Fn: FnOnce(&mut ChunkBuild<'_>)> Applicable for Fn {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self(build);
	}
}

/// Returns an [`Applicable`] that creates a `tag` element, applies `applicable` to it, and appends it to the current element.
///
/// # Example
/// ```
/// build.apply(el("div", text("hello world")));
/// ```
pub fn el(tag: &str, applicable: impl Applicable) -> impl Applicable {
	move |build: &mut ChunkBuild| {
		build.build_codes.start_el(tag);
		applicable.apply(build);
		build.build_codes.end_el();
	}
}

macro_rules! define_tags {
	($($tag:ident),+) => {
		$(
			#[doc = concat!(
				"Returns an [`Applicable`] that creates a `", stringify!($tag),
				"` element, applies `applicable` to it, and appends it to the current element.\n\n",
				"# Example\n```\n",
				"build.apply(", stringify!($tag), "(id(\"my-id\")));\n",
				"```"
			)]
			pub fn $tag(applicable: impl Applicable) -> impl Applicable {
				el(stringify!($tag), applicable)
			}
		)*
	};
}

/// Specialization of [`el`] for HTML tags.
pub mod tags {
	use super::*;
	define_tags![
		a, abbr, address, area, article, aside, audio, b, base, bdi, bdo, blockquote, body, br,
		button, canvas, caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn,
		dialog, div, dl, dt, em, embed, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4,
		h5, h6, head, header, hgroup, hr, html, i, iframe, img, input, ins, kbd, label, legend, li,
		link, main, map, mark, math, menu, meta, meter, nav, noscript, object, ol, optgroup,
		option, output, p, picture, portal, pre, progress, q, rp, rt, ruby, s, samp, script,
		search, section, select, slot, small, source, span, strong, style, sub, summary, sup, svg,
		table, tbody, td, template, textarea, tfoot, th, thead, time, title, tr, track, u, ul, var,
		video, wbr
	];
}

/// Returns an [`Applicable`] that applies `value` to the `name` attribute of the current element.
///
/// The `name` can be an [`&'static str`](str) or a [`String`] and `value` can be any of the [attribute value types](crate::chunk!#attribute-values) in the [`chunk`](crate::chunk!) macro.
///
/// # Example
/// ```
/// let hidden = build.prop(true);
/// build.apply(attr("id", "my-id"));
/// build.apply(attr("hidden", hidden));
/// ```
pub fn attr<T>(name: impl Into<Cow<'static, str>>, value: impl AttrValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}

/// Specialization of [`attr`] for the `id` attribute.
///
/// # Example
/// ```
/// build.apply(id("my-id"));
/// ```
pub fn id<T>(value: impl AttrValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, "id".into())
}

/// Returns an [`Applicable`] that toggles the `name` class of the current element based on `value`.
///
/// The `name` can be an [`&'static str`](str) or a [`String`] and `value` can be any of the [`class.name` attribute types](crate::chunk!#classname) in the [`chunk`](crate::chunk!) macro.
///
/// # Example
/// ```
/// let hidden = build.prop(true);
/// build.apply(class("px-1", true));
/// build.apply(class("hidden", hidden));
/// ```
pub fn class(name: impl Into<Cow<'static, str>>, value: impl ClassValue) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}

/// Returns an [`Applicable`] that applies `value` to the `name` CSS property of the current element.
///
/// The `name` can be an [`&'static str`](str) or a [`String`] and `value` can be any of the [`style.name` attribute types](crate::chunk!#styleprop) in the [`chunk`](crate::chunk!) macro.
///
/// # Example
/// ```
/// let color = build.prop(String::from("red"));
/// build.apply(style("font-size", "16px"));
/// build.apply(style("background-color", color));
/// ```
pub fn style<T>(name: impl Into<Cow<'static, str>>, value: impl StyleValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}

/// Returns an [`Applicable`] that sets `value` to the `name` property of the current element.
///
/// The `name` can be an [`&'static str`](str) or a [`String`] and `value` can be any of the [`prop.name` attribute types](crate::chunk!#propname) in the [`chunk`](crate::chunk!) macro.
///
/// # Example
/// ```
/// let html = build.prop(JsValue::from("html"));
/// build.apply(prop("innerText", JsValue::from("hello world")));
/// build.apply(prop("innerHTML", html));
/// ```
pub fn prop(name: impl Into<Cow<'static, str>>, value: impl PropValue) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}

/// Returns an [`Applicable`] that adds an `event` event listener to the current element.
///
/// The listener takes a [`DomContext`] and an [`Event`] as arguments.
///
/// # Example
/// ```
/// build.apply(button(on("click", |ctx, _| println!("clicked"))));
/// ```
pub fn on(event: &str, fun: impl FnMut(&mut DomContext, Event) + 'static) -> impl Applicable {
	move |build: &mut ChunkBuild| add_event(build, event, Box::new(fun))
}

/// Returns an [`Applicable`] that appends `value` as a text node to the current element.
///
/// The `value` can be any of the [text content types](crate::chunk!#text-content) in the [`chunk`](crate::chunk!) macro.
///
/// # Example
/// ```
/// let text = build.prop(String::from("abc"));
/// build.apply(text("hello world"));
/// build.apply(text(text));
/// ```
pub fn text<T>(value: impl TextValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build)
}

/// Returns an [`Applicable`] that appends `value` as a node to the current element.
///
/// The `value` can be any of the [node content types](crate::chunk!#node-content) in the [`chunk`](crate::chunk!) macro.
///
/// # Example
/// ```
/// let el = document().unwrap().create_element("div").unwrap();
/// let prop = build.prop(el.clone());
/// build.apply(node(el));
/// build.apply(node(prop));
/// ```
pub fn node<T>(value: impl NodeValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build)
}

/// The [`Applicable`] implementation for [`show_if`].
#[doc(hidden)]
pub trait ShowIfValue {
	fn apply(self, build: &mut ChunkBuild);
}
impl ShowIfValue for bool {
	fn apply(self, build: &mut ChunkBuild) {
		if !self {
			build.build_codes.style("display", "none");
		}
	}
}
impl ShowIfValue for PropId<bool> {
	fn apply(self, build: &mut ChunkBuild) {
		StyleValue::apply(
			move |ctx: &mut DomContext| if ctx.get(self) { "" } else { "none" },
			build,
			"display".into(),
		);
	}
}
impl<F: FnMut(&mut DomContext) -> bool + 'static> ShowIfValue for F {
	fn apply(mut self, build: &mut ChunkBuild) {
		StyleValue::apply(
			move |ctx: &mut DomContext| if self(ctx) { "" } else { "none" },
			build,
			"display".into(),
		);
	}
}

/// Returns an [`Applicable`] that shows and hides the current element based on `value`.
///
/// The `value` can be:
/// - [`bool`]: shows the element if `value` is `true`.
/// - [`PropId<bool>`]: every time the property changes the element is hidden or shown based on its value.
/// - [`ComputedExpr<bool>`](crate::chunk!#computedexprt): the element is hidden or shown based on the evaluated value.
///
/// # Example
/// ```
/// let visible = build.prop(true);
/// build.apply(show_if(visible));
/// ```
pub fn show_if(value: impl ShowIfValue) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build)
}

impl Applicable for () {
	fn apply(self, _build: &mut ChunkBuild<'_>) {}
}
macro_rules! impl_applyable_tuple {
	($($item:ident),*) => {
		impl<$($item: Applicable),*,> Applicable for ($($item,)*) {
			fn apply(self, build: &mut ChunkBuild<'_>) {
				#[allow(non_snake_case)]
				let ($($item,)*) = self;
				$($item.apply(build);)*
			}
		}
	};
}
impl_applyable_tuple![A];
impl_applyable_tuple![A, B];
impl_applyable_tuple![A, B, C];
impl_applyable_tuple![A, B, C, D];
impl_applyable_tuple![A, B, C, D, E];
impl_applyable_tuple![A, B, C, D, E, F];
impl_applyable_tuple![A, B, C, D, E, F, G];
impl_applyable_tuple![A, B, C, D, E, F, G, H];
impl_applyable_tuple![A, B, C, D, E, F, G, H, I];
impl_applyable_tuple![A, B, C, D, E, F, G, H, I, J];
impl_applyable_tuple![A, B, C, D, E, F, G, H, I, J, K];
impl_applyable_tuple![A, B, C, D, E, F, G, H, I, J, K, L];
