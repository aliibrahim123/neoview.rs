//! build ui in a native builder pattern.
//!
//! the `apply` module provides another way to construct ui for whom who like a more native macroless approuch using the builder pattern.
//!
//! it consists of 2 parts: [`Applicable`] types that do something to the current element, and [`apply`](ChunkBuild::apply) that apply the [`Applicable`] to the current element.
//!
//! # example
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

/// a type that does something to the current element.
///
/// it takes a [`ChunkBuild`] and does something, this something can be anything: an attribute changed, a child added, nothing and even adding entire ui chunks.
///
/// it is implemented for `FnOnce(&mut ChunkBuild)`, and for tuples of `Applicable` upto 12 elements where each `Applicable` is applied in order.
///
/// # example
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
	/// apply the [`Applicable`] to the current element
	fn apply(self, build: &mut ChunkBuild<'_>);
}

impl<Fn: FnOnce(&mut ChunkBuild<'_>)> Applicable for Fn {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self(build);
	}
}

/// returns an [`Applicable`] that create a `tag` element, apply `applicable` to it and append it to the current element.
///
/// # example
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
				"returns an [`Applicable`] that create a `", stringify!($tag),
				"` element, apply `applicable` to it and append it to the current element.\n\n",
				"# example\n```\n",
				"build.apply(", stringify!($tag), "(id(\"my-id\")));\n",
				"```"
			)]
			pub fn $tag(applicable: impl Applicable) -> impl Applicable {
				el(stringify!($tag), applicable)
			}
		)*
	};
}

/// specilization of [`el`] for html tags.
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

/// returns an [`Applicable`] that apply `value` to the `name` attribute of the current element.
///
/// the `name` can be [`&'static str`](str) or a [`String`], and `value` can be any of [attribute values types](crate::chunk!#attribute-values) in [`chunk`](crate::chunk!) macro.
///
/// # example
/// ```
/// let hidden = build.prop(true);
/// build.apply(attr("id", "my-id"));
/// build.apply(attr("hidden", hidden));
/// ```
pub fn attr<T>(name: impl Into<Cow<'static, str>>, value: impl AttrValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}

/// specialization of [`attr`] for `id` attribute
///
/// # example
/// ```
/// build.apply(id("my-id"));
/// ```
pub fn id<T>(value: impl AttrValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, "id".into())
}

/// returns an [`Applicable`] that toggles the `name` class of the current element based on `value`.
///
/// the `name` can be [`&'static str`](str) or a [`String`], and `value` can be any of [`class.name` attributes types](crate::chunk!#classname) in [`chunk`](crate::chunk!) macro.
///
/// # example
/// ```
/// let hidden = build.prop(true);
/// build.apply(class("px-1", true));
/// build.apply(class("hidden", hidden));
/// ```
pub fn class(name: impl Into<Cow<'static, str>>, value: impl ClassValue) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}

/// returns an [`Applicable`] that apply `value` to the `name` css property of the current element.
///
/// the `name` can be [`&'static str`](str) or a [`String`], and `value` can be any of [`style.name` attributes types](crate::chunk!#styleprop) in [`chunk`](crate::chunk!) macro.
///
/// # example
/// ```
/// let color = build.prop(String::from("red"));
/// build.apply(style("font-size", "16px"));
/// build.apply(style("background-color", color));
/// ```
pub fn style<T>(name: impl Into<Cow<'static, str>>, value: impl StyleValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}

/// returns an [`Applicable`] that apply set `value` to the `name` property of the current element.
///
/// the `name` can be [`&'static str`](str) or a [`String`], and `value` can be any of [`prop.name` attributes types](crate::chunk!#propname) in [`chunk`](crate::chunk!) macro.
///
/// # example
/// ```
/// let html = build.prop(JsValue::from("html"));
/// build.apply(prop("innerText", JsValue::from("hello world")));
/// build.apply(prop("innerHTML", html));
/// ```
pub fn prop(name: impl Into<Cow<'static, str>>, value: impl PropValue) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}

/// returns an [`Applicable`] that add a `event` event listener to the current element.
///
/// the listener takes a [`DomContext`] and [`Event`] as arguments.
///
/// # example
/// ```
/// build.apply(button(on("click", |ctx, _| println!("clicked"))));
/// ```
pub fn on(event: &str, fun: impl FnMut(&mut DomContext, Event) + 'static) -> impl Applicable {
	move |build: &mut ChunkBuild| add_event(build, event, Box::new(fun))
}

/// returns an [`Applicable`] that append `value` as text node to the current element.
///
/// `value` can be any of [text content types](crate::chunk!#text-content) in [`chunk`](crate::chunk!) macro.
///
/// # example
/// ```
/// let text = build.prop(String::from("abc"));
/// build.apply(text("hello world"));
/// build.apply(text(text));
/// ```
pub fn text<T>(value: impl TextValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build)
}

/// returns an [`Applicable`] that append `value` as node to the current element.
///
/// `value` can be any of [node content types](crate::chunk!#node-content) in [`chunk`](crate::chunk!) macro.
///
/// # example
/// ```
/// let el = document().unwrap().create_element("div").unwrap();
/// let prop = build.prop(el.clone());
/// build.apply(node(el));
/// build.apply(node(prop));
/// ```
pub fn node<T>(value: impl NodeValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build)
}

/// the [`Applicable`] of [`show_if`]
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

/// returns an [`Applicable`] that show and hides the current element based on `value`.
///
/// `value` can be:
/// - [`bool`]: show the element if `value` is `true`.
/// - [`PropId<bool>`]: everytime the property changes, the element is hidden/showen based on its value.
/// - [`ComputedExpr<bool>`](crate::chunk!#computedexprt): the element is hidden/showen based on the evaluated value.
///
/// # example
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
