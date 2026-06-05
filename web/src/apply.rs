use std::borrow::Cow;

use neoview::{PropId, StoreProv};
use web_sys::Event;

use crate::{
	bindings::{AttrValue, ClassValue, NodeValue, PropValue, StyleValue, TextValue},
	build_codes::__buildcode::add_event,
	prelude::{ChunkBuild, DomContext},
};

pub trait Applicable {
	fn apply(self, build: &mut ChunkBuild<'_>);
}

impl<Fn: FnOnce(&mut ChunkBuild<'_>)> Applicable for Fn {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self(build);
	}
}

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
			pub fn $tag(applicable: impl Applicable) -> impl Applicable {
				el(stringify!($tag), applicable)
			}
		)*
	};
}

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

pub fn attr<T>(name: impl Into<Cow<'static, str>>, value: impl AttrValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}
pub fn class(name: impl Into<Cow<'static, str>>, value: impl ClassValue) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}
pub fn style<T>(name: impl Into<Cow<'static, str>>, value: impl StyleValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}
pub fn prop(name: impl Into<Cow<'static, str>>, value: impl PropValue) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build, name.into())
}
pub fn on(event: &str, fun: impl FnMut(&mut DomContext, Event) + 'static) -> impl Applicable {
	move |build: &mut ChunkBuild| add_event(build, event, Box::new(fun))
}
pub fn text<T>(value: impl TextValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build)
}
pub fn node<T>(value: impl NodeValue<T>) -> impl Applicable {
	move |build: &mut ChunkBuild| value.apply(build)
}

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
