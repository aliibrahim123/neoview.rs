use std::borrow::Cow;

use neoview::{PropId, ScopedStoreProv, SlabId, Store, StoreProv, TrackResult};
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use web_sys::Element;

use crate::{
	chunk_build::{ChunkBuild, ChunkId},
	context::DomContext,
};

#[wasm_bindgen(module = "neoview-web-binder")]
extern "C" {
	pub fn construct(target_el: &Element, build_codes: Vec<u8>, args: Vec<JsValue>)
	-> Vec<Element>;

}

const COMMON_NAMES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/common_names.rs"));

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Buf(pub Vec<u8>);
impl Buf {}

#[derive(Debug)]
pub struct BuildCodes {
	codes: Vec<u8>,
	args: Vec<JsValue>,
	cur_el_id: u64,
	el_id_stack: Vec<Option<u64>>,
}
impl BuildCodes {
	pub fn new() -> Self {
		Self { codes: Vec::new(), args: Vec::new(), cur_el_id: 1, el_id_stack: vec![Some(0)] }
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
		self.push_vuint(self.args.len() as u64);
		self.args.push(value);
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
	pub fn node(&mut self, value: JsValue) {
		self.codes.push(Self::NODE);
		self.push_vuint(self.args.len() as u64);
		self.args.push(value);
	}
	pub fn end_el(&mut self) {
		self.codes.push(Self::END);
		self.el_id_stack.pop();
	}
	pub fn construct(self, ctx: &mut DomContext, base_el: &Element, id: ChunkId) {
		let els = construct(base_el, self.codes, self.args);
		ctx.chunk_el_map[id] = els;
	}
}

#[doc(hidden)]
pub mod __buildcode {
	#[macro_export]
	macro_rules! refine_value {
		(move |$ctx:ident| $($t:tt)*) => { move |$ctx: &mut $crate::DomContext| $($t)* };
		($($t:tt)*) => { $($t)* };
	}

	#[macro_export]
	macro_rules! start_chunk {
		($($t:tt)*) => {
			()
		};
	}
	#[macro_export]
	macro_rules! end_chunk {
		($($t:tt)*) => {};
	}

	#[macro_export]
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
	#[cfg(feature = "html-types")]
	macro_rules! attr {
		($build:expr, $el:expr, [$attr:ident], $value:expr) => {{
			$crate::html_types::html_attrs::$attr;
			__buildcode::AttrValue::apply(
				__buildcode::refine_value!($value), &mut $build, stringify!($attr).into()
			);
		}};
		($($t:tt)*) => {
			__buildcode::attr_common!($($t)*)
		}
	}
	#[macro_export]
	#[cfg(not(feature = "html-types"))]
	macro_rules! attr {
		($build:expr, $el:expr, $tag:ident) => {

		};
		($($t:tt)*) => {
			__buildcode::attr_common!($($t)*)
		}
	}

	#[macro_export]
	macro_rules! attr_common {
		($build:expr, $el:expr, [$($attr:tt)*], $value:expr) => {
			::core::compile_error!(concat!("unknown attribute: ", stringify!($($attr)*)))
		};
	}

	#[macro_export]
	macro_rules! content {
		($build:expr, $el:expr, $($t:tt)*) => {};
	}

	#[macro_export]
	macro_rules! end_el {
		($build:expr, $($t:tt)*) => {
			$build.build_codes.end_el()
		};
	}

	#[macro_export]
	macro_rules! start_do_block {
		($($t:tt)*) => {};
	}
	#[macro_export]
	macro_rules! end_do_block {
		($($t:tt)*) => {};
	}

	use crate::prelude::DomContext;

	pub use super::AttrValue;
	pub use {
		attr, attr_common, content, end_chunk, end_do_block, end_el, refine_value, start_chunk,
		start_do_block, start_el, start_el_common,
	};
}

pub struct StaticValue;
pub struct PropValue;
pub struct ComputedValue;

trait BasicAttrValue {
	fn apply_static(self, build_codes: &mut BuildCodes, name: &str);
	fn apply_dynamic(self, el: &Element, name: &str);
}
impl<'a> BasicAttrValue for &'a str {
	fn apply_static(self, build_codes: &mut BuildCodes, name: &str) {
		build_codes.attr(name, self);
	}
	fn apply_dynamic(self, el: &Element, name: &str) {
		el.set_attribute(name, self).unwrap();
	}
}
impl BasicAttrValue for String {
	fn apply_static(self, build_codes: &mut BuildCodes, name: &str) {
		build_codes.attr(name, &self);
	}
	fn apply_dynamic(self, el: &Element, name: &str) {
		el.set_attribute(name, &self).unwrap();
	}
}
impl BasicAttrValue for bool {
	fn apply_static(self, build_codes: &mut BuildCodes, name: &str) {
		if self {
			build_codes.attr(name, "");
		}
	}
	fn apply_dynamic(self, el: &Element, name: &str) {
		el.toggle_attribute_with_force(name, self).unwrap();
	}
}
macro_rules! basic_attr_to_string {
	($($ty:ty)*) => {
		$(impl BasicAttrValue for $ty {
			fn apply_static(self, build_codes: &mut BuildCodes, name: &str) {
				build_codes.attr(name, &self.to_string());
			}
			fn apply_dynamic(self, el: &Element, name: &str) {
				el.set_attribute(name, &self.to_string()).unwrap();
			}
		})*
	};
}
basic_attr_to_string!(i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64);
impl<T: BasicAttrValue + Clone> BasicAttrValue for &T {
	fn apply_static(self, build_codes: &mut BuildCodes, name: &str) {
		self.clone().apply_static(build_codes, name);
	}
	fn apply_dynamic(self, el: &Element, name: &str) {
		self.clone().apply_dynamic(el, name);
	}
}
impl<T: BasicAttrValue> BasicAttrValue for Option<T> {
	fn apply_static(self, build_codes: &mut BuildCodes, name: &str) {
		if let Some(v) = self {
			v.apply_static(build_codes, name);
		}
	}
	fn apply_dynamic(self, el: &Element, name: &str) {
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
impl<T: BasicAttrValue> AttrValue<StaticValue> for T {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		self.apply_static(&mut build.build_codes, &name);
	}
}
fn add_effect(
	build: &mut ChunkBuild, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
	mut fun: impl FnMut(&mut DomContext, ChunkId, usize) + 'static,
) {
	let el = build.build_codes.request_id();
	let chunk = build.id;
	let fun = move |ctx: &mut DomContext| {
		fun(ctx, chunk, el as usize);
	};
	match build.slab() {
		Some(slab) => Store::effect_manual_in(build.ctx, slab, read, write, fun, false).unwrap(),
		None => Store::effect_manual(build.ctx, read, write, fun, false),
	}
}
impl<T: BasicAttrValue + Clone> AttrValue<PropValue> for PropId<T> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		BasicAttrValue::apply_static(build.ctx.read(self), &mut build.build_codes, &name);
		add_effect(build, vec![self.erase_type()], vec![], move |ctx, chunk, el| {
			BasicAttrValue::apply_dynamic(ctx.read(self), &ctx.chunk_el_map[chunk][el], &name);
		});
	}
}
impl<T: BasicAttrValue, F: FnMut(&mut DomContext) -> T + 'static> AttrValue<ComputedValue> for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.store().start_track().unwrap();
		let value = self(build.ctx);
		let TrackResult { read, written: write } = build.store().end_track().unwrap();
		BasicAttrValue::apply_static(value, &mut build.build_codes, &name);
		add_effect(build, read, write, move |ctx, chunk, el| {
			BasicAttrValue::apply_dynamic(self(ctx), &ctx.chunk_el_map[chunk][el], &name);
		});
	}
}
