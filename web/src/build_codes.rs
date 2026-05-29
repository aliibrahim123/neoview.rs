use std::borrow::Cow;

use neoview::{PropId, ScopedStoreProv, Store, StoreProv, TrackResult};
use slotmap::Key;
use wasm_bindgen::prelude::{JsCast, JsValue, wasm_bindgen};
use web_sys::{Element, HtmlElement, Node, Text, js_sys::Reflect};

use crate::{
	chunk::{ChunkBuild, ChunkId},
	context::{ContextId, DomContext},
};

#[wasm_bindgen(module = "neoview-web-binder")]
extern "C" {
	pub fn construct(
		target_el: &Element, build_codes: Vec<u8>, props: Vec<JsValue>, nodes: Vec<Node>,
	) -> Vec<Element>;

}

const COMMON_NAMES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/common_names.rs"));

#[derive(Debug)]
pub struct BuildCodes {
	codes: Vec<u8>,
	props: Vec<JsValue>,
	nodes: Vec<Node>,
	cur_el_id: u64,
	el_id_stack: Vec<Option<u64>>,
}
impl BuildCodes {
	pub fn new() -> Self {
		Self {
			codes: Vec::new(),
			props: Vec::new(),
			nodes: Vec::new(),
			cur_el_id: 1,
			el_id_stack: vec![Some(0)],
		}
	}
	pub fn push_slice(&mut self, slice: &[u8]) {
		self.codes.extend_from_slice(slice);
	}
	pub fn push_vuint(&mut self, mut value: u64) {
		let mut there_input = true;

		while there_input {
			let byte = value as u8 & 0b0111_1111;
			value >>= 7;
			self.codes.push(if value == 0 { byte } else { byte | 0b1000_0000 });
			there_input = value != 0;
		}
	}
	pub fn push_str(&mut self, str: &str) {
		self.push_vuint(str.len() as u64);
		self.push_slice(str.as_bytes());
	}
	pub fn push_name(&mut self, str: &str) {
		match COMMON_NAMES.binary_search(&str) {
			Ok(id) => self.push_vuint((id << 1) as u64 | 1),
			Err(_) => {
				self.push_vuint((str.len() << 1) as u64);
				self.push_slice(str.as_bytes());
			}
		}
	}

	const EL_START: u8 = 0;
	const EL_ID: u8 = 1;
	const ATTR: u8 = 2;
	const PROP: u8 = 3;
	const CLASS: u8 = 4;
	const STYLE: u8 = 5;
	const TEXT: u8 = 6;
	const NODE: u8 = 7;
	const EVENT: u8 = 8;
	const END: u8 = 255;

	pub fn start_el(&mut self, tag: &str) {
		self.codes.push(Self::EL_START);
		self.push_name(tag);
		self.el_id_stack.push(None);
	}
	pub fn request_id(&mut self) -> u64 {
		if let Some(id) = self.el_id_stack.last().copied().flatten() {
			return id;
		}
		let id = self.cur_el_id;
		self.cur_el_id += 1;
		let last_ind = self.el_id_stack.len() - 1;
		self.el_id_stack[last_ind] = Some(id);
		self.codes.push(Self::EL_ID);
		id
	}
	pub fn attr(&mut self, name: &str, value: &str) {
		self.codes.push(Self::ATTR);
		self.push_name(name);
		self.push_str(value);
	}
	pub fn prop(&mut self, name: &str, value: JsValue) {
		self.codes.push(Self::PROP);
		self.push_str(name);
		self.push_vuint(self.props.len() as u64);
		self.props.push(value);
	}
	pub fn class(&mut self, value: &str) {
		self.codes.push(Self::CLASS);
		self.push_str(value);
	}
	pub fn style(&mut self, name: &str, value: &str) {
		self.codes.push(Self::STYLE);
		self.push_name(name);
		self.push_str(value);
	}
	pub fn text(&mut self, value: &str) {
		self.codes.push(Self::TEXT);
		self.push_str(value);
	}
	pub fn node(&mut self, value: Node) {
		self.codes.push(Self::NODE);
		self.push_vuint(self.nodes.len() as u64);
		self.nodes.push(value);
	}
	pub fn event(&mut self, ctx_id: ContextId, chunk_id: ChunkId, name: &str, fn_id: u64) {
		self.codes.push(Self::EVENT);
		self.push_vuint(ctx_id.0);
		self.push_vuint(chunk_id.data().as_ffi());
		self.push_name(name);
		self.push_vuint(fn_id);
	}
	pub fn end_el(&mut self) {
		self.codes.push(Self::END);
		self.el_id_stack.pop();
	}
	pub fn construct(self, ctx: &mut DomContext, base_el: &Element, id: ChunkId) {
		let elements = construct(base_el, self.codes, self.props, self.nodes);
		ctx.chunks[id].elements = elements;
	}
}

#[doc(hidden)]
pub mod __buildcode {
	#[macro_export]
	#[doc(hidden)]
	macro_rules! start_chunk {
		($($t:tt)*) => {
			()
		};
	}
	#[macro_export]
	#[doc(hidden)]
	macro_rules! end_chunk {
		($($t:tt)*) => {};
	}

	#[macro_export]
	#[doc(hidden)]
	#[cfg(feature = "html-types")]
	macro_rules! start_el {
		($build:expr, $el:expr, $tag:ident) => {{
			$crate::html_types::html_tags::$tag;
			$build.build_codes.start_el(stringify!($tag));
			()
		}};
		($($t:tt)*) => {
			__buildcode::start_el_common!($($t)*)
		}
	}
	#[macro_export]
	#[doc(hidden)]
	#[cfg(not(feature = "html-types"))]
	macro_rules! start_el {
		($build:expr, $el:expr, $tag:ident) => {{
			$build.build_codes.start_el(stringify!($tag));
			()
		}};
		($($t:tt)*) => {
			__buildcode::start_el_common!($($t)*)
		}
	}

	#[macro_export]
	#[doc(hidden)]
	macro_rules! start_el_common {
		($build:expr, $el:expr, $tag:literal) => {{
			$build.build_codes.start_el($tag);
			()
		}};
		($build:expr, $el:expr, $($t:tt)*) => {
			::core::compile_error!(concat!("unknown tag: ", stringify!($($t)*)))
		}
	}

	#[macro_export]
	#[doc(hidden)]
	#[cfg(feature = "css-types")]
	macro_rules! attr {
		($build:expr, $el:expr, [style.$prop:ident], $($value:tt)*) => {{
			$crate::html_types::css_props::$prop;
			__buildcode::StyleValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($prop).into()
			);
		}};
		($($t:tt)*) => {
			__buildcode::attr_html!($($t)*)
		}
	}
	#[macro_export]
	#[doc(hidden)]
	#[cfg(all(feature = "html-types", not(feature = "css-types")))]
	macro_rules! attr {
		($($t:tt)*) => {__buildcode::attr_html!($($t)*)};
	}
	#[doc(hidden)]
	#[macro_export]
	#[cfg(feature = "html-types")]
	macro_rules! attr_html {
		($build:expr, $el:expr, [$attr:ident], $($value:tt)*) => {{
			$crate::html_types::html_attrs::$attr;
			__buildcode::AttrValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($attr).into()
			);
		}};
		($build:expr, $el:expr, [on.$event:ident], $($value:tt)*) => {{
			$crate::html_types::html_events::$event;
			__buildcode::add_event(&mut $build, stringify!($event), Box::new($($value)*));
		}};
		($($t:tt)*) => {
			__buildcode::attr_common!($($t)*)
		}
	}
	#[macro_export]
	#[doc(hidden)]
	#[cfg(not(feature = "html-types"))]
	macro_rules! attr {
		($build:expr, $el:expr, [$attr:ident], $($value:tt)*) => {{
			__buildcode::AttrValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($attr).into()
			);
		}};
		($build:expr, $el:expr, [on.$event:ident], $($value:tt)*) => {{
			__buildcode::add_event(&mut $build, stringify!($event), Box::new($($value)*));
		}};
		($($t:tt)*) => {
			__buildcode::attr_common!($($t)*)
		}
	}
	#[macro_export]
	#[doc(hidden)]
	macro_rules! attr_common {
		($build:expr, $el:expr, [$attr_start:ident $(-$attr_rest:ident)+], $($value:tt)*) => {{
			__buildcode::AttrValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($attr_start $(-$attr_rest)+).into()
			);
		}};
		($build:expr, $el:expr, [$attr:literal], $($value:tt)*) => {
			__buildcode::AttrValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, $attr.into()
			);
		};
		($build:expr, $el:expr, [class.$class_start:ident $(-$class_rest:ident)*], $($value:tt)*) => {{
			__buildcode::ClassValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($class_start $(-$class_rest)*).into()
			);
		}};
		($build:expr, $el:expr, [class.$class:literal], $($value:tt)*) => {{
			__buildcode::ClassValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, $class.into()
			);
		}};
		($build:expr, $el:expr, [style.$prop_start:ident $(-$prop_rest:ident)*], $($value:tt)*) => {{
			__buildcode::StyleValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($prop_start $(-$prop_rest)*).into()
			);
		}};
		($build:expr, $el:expr, [style.$prop:literal], $($value:tt)*) => {{
			__buildcode::ClassValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, $prop.into()
			);
		}};
		($build:expr, $el:expr, [prop.$prop:ident], $($value:tt)*) => {{
			__buildcode::PropValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, stringify!($prop).into()
			);
		}};
		($build:expr, $el:expr, [prop.$prop:literal], $($value:tt)*) => {{
			__buildcode::PropValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, $prop.into()
			);
		}};
		($build:expr, $el:expr, [on.$event:literal], $($value:tt)*) => {{
			__buildcode::add_event(&mut $build, $event, Box::new($($value)*));
		}};
		($build:expr, $el:expr, [$($attr:tt)*], $($value:tt)*) => {
			::core::compile_error!(concat!("unknown attribute: ", stringify!($($attr)*)))
		};
	}

	#[macro_export]
	#[doc(hidden)]
	macro_rules! content {
		($build:expr, $el:expr, $($t:tt)*) => {
			__buildcode::ContentValue::apply(__buildcode::refine_value!($($t)*), &mut $build)
		};
	}

	#[macro_export]
	#[doc(hidden)]
	macro_rules! end_el {
		($build:expr, $($t:tt)*) => {
			$build.build_codes.end_el()
		};
	}

	#[macro_export]
	#[doc(hidden)]
	macro_rules! start_do_block {
		($($t:tt)*) => {};
	}
	#[macro_export]
	#[doc(hidden)]
	macro_rules! end_do_block {
		($($t:tt)*) => {};
	}

	#[macro_export]
	#[doc(hidden)]
	macro_rules! refine_value {
		(move |$ctx:ident| $($t:tt)*) => { move |$ctx: &mut $crate::DomContext| $($t)* };
		($($t:tt)*) => { $($t)* };
	}

	use crate::prelude::{ChunkBuild, DomContext};

	pub fn add_event(
		build: &mut ChunkBuild, event: &str, fun: Box<dyn FnMut(&mut DomContext, Event)>,
	) {
		let events = &mut build.ctx.chunks[build.id].events;
		build.build_codes.event(build.ctx.id, build.id, event, events.len() as u64);
		events.push(fun);
	}

	pub use super::{AttrValue, ClassValue, ContentValue, PropValue, StyleValue};
	#[cfg(feature = "html-types")]
	pub use attr_html;
	pub use neoview_macro::kababify;
	use web_sys::Event;
	pub use {
		attr, attr_common, content, end_chunk, end_do_block, end_el, refine_value, start_chunk,
		start_do_block, start_el, start_el_common,
	};
}

pub struct Static;
pub struct Prop;
pub struct Computed;

fn to_html_el(el: &Element) -> &HtmlElement {
	el.dyn_ref().unwrap()
}
fn add_effect(
	build: &mut ChunkBuild, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
	fun: impl FnMut(&mut DomContext) + 'static,
) {
	match build.slab() {
		Some(slab) => Store::effect_manual_in(build.ctx, slab, read, write, fun, false).unwrap(),
		None => Store::effect_manual(build.ctx, read, write, fun, false),
	}
}
fn add_effect_with_el(
	build: &mut ChunkBuild, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
	mut fun: impl FnMut(&mut DomContext, ChunkId, usize) + 'static,
) {
	let el = build.build_codes.request_id();
	let chunk = build.id;
	let fun = move |ctx: &mut DomContext| {
		fun(ctx, chunk, el as usize);
	};
	add_effect(build, read, write, fun);
}

trait BasicAttrValue {
	fn apply_static(&self, build_codes: &mut BuildCodes, name: &str);
	fn apply_dynamic(&self, el: &Element, name: &str);
}
impl BasicAttrValue for &str {
	fn apply_static(&self, build_codes: &mut BuildCodes, name: &str) {
		build_codes.attr(name, self);
	}
	fn apply_dynamic(&self, el: &Element, name: &str) {
		el.set_attribute(name, self).unwrap();
	}
}
impl BasicAttrValue for str {
	fn apply_static(&self, build_codes: &mut BuildCodes, name: &str) {
		build_codes.attr(name, self);
	}
	fn apply_dynamic(&self, el: &Element, name: &str) {
		el.set_attribute(name, self).unwrap();
	}
}
impl BasicAttrValue for String {
	fn apply_static(&self, build_codes: &mut BuildCodes, name: &str) {
		build_codes.attr(name, &self);
	}
	fn apply_dynamic(&self, el: &Element, name: &str) {
		el.set_attribute(name, &self).unwrap();
	}
}
impl BasicAttrValue for bool {
	fn apply_static(&self, build_codes: &mut BuildCodes, name: &str) {
		if *self {
			build_codes.attr(name, "");
		}
	}
	fn apply_dynamic(&self, el: &Element, name: &str) {
		el.toggle_attribute_with_force(name, *self).unwrap();
	}
}
macro_rules! basic_attr_int {
	($($ty:ty)*) => {
		$(impl BasicAttrValue for $ty {
			fn apply_static(&self, build_codes: &mut BuildCodes, name: &str) {
				build_codes.attr(name, &self.to_string());
			}
			fn apply_dynamic(&self, el: &Element, name: &str) {
				el.set_attribute(name, &self.to_string()).unwrap();
			}
		})*
	};
}
basic_attr_int!(i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64);
impl<T: BasicAttrValue> BasicAttrValue for &T {
	fn apply_static(&self, build_codes: &mut BuildCodes, name: &str) {
		(*self).apply_static(build_codes, name);
	}
	fn apply_dynamic(&self, el: &Element, name: &str) {
		(*self).apply_dynamic(el, name);
	}
}
impl<T: BasicAttrValue> BasicAttrValue for Option<T> {
	fn apply_static(&self, build_codes: &mut BuildCodes, name: &str) {
		if let Some(v) = self {
			v.apply_static(build_codes, name);
		}
	}
	fn apply_dynamic(&self, el: &Element, name: &str) {
		if let Some(v) = self {
			v.apply_dynamic(el, name);
		} else {
			el.remove_attribute(name).unwrap();
		}
	}
}

pub trait AttrValue<Value> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>);
}
impl<T: BasicAttrValue> AttrValue<Static> for T {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		self.apply_static(&mut build.build_codes, &name);
	}
}
impl<T: BasicAttrValue> AttrValue<Prop> for PropId<T> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		BasicAttrValue::apply_static(build.ctx.read(self), &mut build.build_codes, &name);
		add_effect_with_el(build, vec![self.erase_type()], vec![], move |ctx, chunk, el| {
			let el = &ctx.chunks[chunk].elements[el];
			BasicAttrValue::apply_dynamic(ctx.read(self), el, &name);
		});
	}
}
impl<T: BasicAttrValue, F: FnMut(&mut DomContext) -> T + 'static> AttrValue<Computed> for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.store().start_track().unwrap();
		let value = self(build.ctx);
		let TrackResult { read, written: write } = build.store().end_track().unwrap();
		BasicAttrValue::apply_static(&value, &mut build.build_codes, &name);
		add_effect_with_el(build, read, write, move |ctx, chunk, el| {
			let value = self(ctx);
			let el = &ctx.chunks[chunk].elements[el];
			BasicAttrValue::apply_dynamic(&value, el, &name);
		});
	}
}

pub trait ClassValue {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>);
}
impl ClassValue for bool {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		if self {
			build.build_codes.class(&name);
		}
	}
}
impl ClassValue for PropId<bool> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		if build.ctx.get(self) {
			build.build_codes.class(&name);
		}
		add_effect_with_el(build, vec![self.erase_type()], vec![], move |ctx, chunk, el| {
			let el = &ctx.chunks[chunk].elements[el];
			el.class_list().toggle_with_force(&name, ctx.get(self)).unwrap();
		});
	}
}
impl<F: FnMut(&mut DomContext) -> bool + 'static> ClassValue for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.store().start_track().unwrap();
		let value = self(build.ctx);
		let TrackResult { read, written: write } = build.store().end_track().unwrap();
		if value {
			build.build_codes.class(&name);
		}
		add_effect_with_el(build, read, write, move |ctx, chunk, el| {
			let value = self(ctx);
			let el = &ctx.chunks[chunk].elements[el];
			el.class_list().toggle_with_force(&name, value).unwrap();
		});
	}
}

pub trait BasicStyleValue {
	fn to_str(&self) -> &str;
}
impl BasicStyleValue for str {
	fn to_str(&self) -> &str {
		self
	}
}
impl BasicStyleValue for &str {
	fn to_str(&self) -> &str {
		*self
	}
}
impl BasicStyleValue for String {
	fn to_str(&self) -> &str {
		&self
	}
}
impl<T: BasicStyleValue> BasicStyleValue for Option<T> {
	fn to_str(&self) -> &str {
		if let Some(v) = self { v.to_str() } else { "" }
	}
}
impl<T: BasicStyleValue> BasicStyleValue for &T {
	fn to_str(&self) -> &str {
		(**self).to_str()
	}
}

pub trait StyleValue<Value> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>);
}
impl<T: BasicStyleValue> StyleValue<Static> for T {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.build_codes.style(&name, self.to_str());
	}
}
impl<T: BasicStyleValue + Clone> StyleValue<Prop> for PropId<T> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.build_codes.style(&name, build.ctx.read(self).to_str());
		add_effect_with_el(build, vec![self.erase_type()], vec![], move |ctx, chunk, el| {
			let el = &ctx.chunks[chunk].elements[el];
			to_html_el(el).style().set_property(&name, ctx.read(self).to_str()).unwrap();
		});
	}
}
impl<T: BasicStyleValue, F: FnMut(&mut DomContext) -> T + 'static> StyleValue<Computed> for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.store().start_track().unwrap();
		let value = self(build.ctx);
		let TrackResult { read, written: write } = build.store().end_track().unwrap();
		build.build_codes.style(&name, value.to_str());
		add_effect_with_el(build, read, write, move |ctx, chunk, el| {
			let value = self(ctx);
			let el = &ctx.chunks[chunk].elements[el];
			to_html_el(el).style().set_property(&name, value.to_str()).unwrap();
		});
	}
}

pub trait PropValue {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>);
}
impl PropValue for JsValue {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.build_codes.prop(&name, self);
	}
}
impl PropValue for PropId<JsValue> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		let value = build.ctx().read(self).clone();
		build.build_codes.prop(&name, value);
		add_effect_with_el(build, vec![self.erase_type()], vec![], move |ctx, chunk, el| {
			let el = &ctx.chunks[chunk].elements[el];
			Reflect::set(el, &name.as_ref().into(), ctx.read(self)).unwrap();
		});
	}
}
impl<F: FnMut(&mut DomContext) -> JsValue + 'static> PropValue for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.store().start_track().unwrap();
		let value = self(build.ctx);
		let TrackResult { read, written: write } = build.store().end_track().unwrap();
		build.build_codes.prop(&name, value);
		add_effect_with_el(build, read, write, move |ctx, chunk, el| {
			let value = self(ctx);
			let el = &ctx.chunks[chunk].elements[el];
			Reflect::set(el, &name.as_ref().into(), &value).unwrap();
		});
	}
}

pub trait BasicTextValue {
	fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R;
}
impl BasicTextValue for str {
	fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R {
		fun(self)
	}
}
impl BasicTextValue for &str {
	fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R {
		fun(*self)
	}
}
impl BasicTextValue for String {
	fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R {
		fun(self)
	}
}
macro_rules! basic_text_primitive {
	($($ty:ty)+) => {
		$(impl BasicTextValue for $ty {
			fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R {
				fun(&self.to_string())
			}
		})+
	};
}
basic_text_primitive!(bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64);
impl<T: BasicTextValue> BasicTextValue for Option<T> {
	fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R {
		if let Some(value) = self { value.with(fun) } else { fun("") }
	}
}
impl<T: BasicTextValue> BasicTextValue for &T {
	fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R {
		(*self).with(fun)
	}
}
pub trait ContentValue<Value> {
	fn apply(self, build: &mut ChunkBuild<'_>);
}
impl<T: BasicTextValue> ContentValue<Static> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self.with(|value| {
			if value != "" {
				build.build_codes.text(value)
			}
		});
	}
}
impl<T: BasicTextValue> ContentValue<Prop> for PropId<T> {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		let node = build.ctx().read(self).with(|value| Text::new_with_data(value).unwrap());
		build.build_codes.node(node.clone().into());
		add_effect(build, vec![self.erase_type()], vec![], move |ctx| {
			ctx.read(self).with(|value| node.set_text_content(Some(value)));
		});
	}
}
impl<T: BasicTextValue, F: FnMut(&mut DomContext) -> T + 'static> ContentValue<Computed> for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>) {
		build.store().start_track().unwrap();
		let value = self(build.ctx);
		let TrackResult { read, written: write } = build.store().end_track().unwrap();
		let node = value.with(|value| Text::new_with_data(value).unwrap());
		build.build_codes.node(node.clone().into());
		add_effect(build, read, write, move |ctx| {
			self(ctx).with(|value| node.set_text_content(Some(value)));
		});
	}
}
impl ContentValue<Node> for Node {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		build.build_codes.node(self);
	}
}
impl ContentValue<Node> for PropId<Node> {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		let mut node = build.ctx().read(self).clone();
		build.build_codes.node(node.clone().into());
		add_effect(build, vec![self.erase_type()], vec![], move |ctx| {
			let cur = ctx.read(self).clone();
			node.parent_node().unwrap().replace_child(&cur, &node).unwrap();
			node = cur;
		});
	}
}
impl<F: FnMut(&mut DomContext) -> Node + 'static> ContentValue<Node> for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>) {
		build.store().start_track().unwrap();
		let mut node = self(build.ctx);
		let TrackResult { read, written: write } = build.store().end_track().unwrap();
		build.build_codes.node(node.clone());
		add_effect(build, read, write, move |ctx| {
			let cur = self(ctx);
			node.parent_node().unwrap().replace_child(&cur, &node).unwrap();
			node = cur;
		});
	}
}
