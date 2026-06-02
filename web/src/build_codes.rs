use slotmap::Key;
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use web_sys::{Element, Node};

use crate::{
	chunk::ChunkId,
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
		($build:expr, $el:expr, [$attr:ident], $($value:tt)*) => {
			__buildcode::AttrValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($attr).into()
			);
		};
		($build:expr, $el:expr, [on.$event:ident], $($value:tt)*) => {{
			__buildcode::colorify!($event);
			__buildcode::add_event(&mut $build, stringify!($event), Box::new($($value)*));
		}};
		($($t:tt)*) => {
			__buildcode::attr_common!($($t)*)
		}
	}
	#[macro_export]
	#[doc(hidden)]
	macro_rules! attr_common {
		($build:expr, $el:expr, [$attr_start:ident $(-$attr_rest:ident)+], $($value:tt)*) => {
			__buildcode::AttrValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($attr_start $(-$attr_rest)+).into()
			);
		};
		($build:expr, $el:expr, [$attr:literal], $($value:tt)*) => {
			__buildcode::AttrValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, $attr.into()
			);
		};
		($build:expr, $el:expr, [class.$class_start:ident $(-$class_rest:ident)*], $($value:tt)*) => {{
			__buildcode::colorify!($class_start);
			__buildcode::ClassValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($class_start $(-$class_rest)*).into()
			);
		}};
		($build:expr, $el:expr, [class.$class:literal], $($value:tt)*) => {
			__buildcode::ClassValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, $class.into()
			);
		};
		($build:expr, $el:expr, [style.$prop_start:ident $(-$prop_rest:ident)*], $($value:tt)*) => {{
			__buildcode::colorify!($prop_start);
			__buildcode::StyleValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build,
				__buildcode::kababify!($prop_start $(-$prop_rest)*).into()
			);
		}};
		($build:expr, $el:expr, [style.$prop:literal], $($value:tt)*) => {
			__buildcode::ClassValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, $prop.into()
			);
		};
		($build:expr, $el:expr, [prop.$prop:ident], $($value:tt)*) => {{
			__buildcode::colorify!($prop);
			__buildcode::PropValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, stringify!($prop).into()
			);
		}};
		($build:expr, $el:expr, [prop.$prop:literal], $($value:tt)*) => {
			__buildcode::PropValue::apply(
				__buildcode::refine_value!($($value)*), &mut $build, $prop.into()
			);
		};
		($build:expr, $el:expr, [on.$event:literal], $($value:tt)*) => {
			__buildcode::add_event(&mut $build, $event, Box::new($($value)*))
		};
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

	#[macro_export]
	#[doc(hidden)]
	macro_rules! colorify {
		($x:ident) => {{
			#[allow(unused, nonstandard_style)]
			let $x = ();
		}};
	}

	use crate::prelude::{ChunkBuild, DomContext};

	pub fn add_event(
		build: &mut ChunkBuild, event: &str, fun: Box<dyn FnMut(&mut DomContext, Event)>,
	) {
		let events = &mut build.ctx.chunks[build.id].events;
		build.build_codes.event(build.ctx.id, build.id, event, events.len() as u64);
		events.push(fun);
	}

	pub use crate::bindings::{AttrValue, ClassValue, ContentValue, PropValue, StyleValue};
	#[cfg(feature = "html-types")]
	pub use attr_html;
	pub use neoview_macro::kababify;
	use web_sys::Event;
	pub use {
		attr, attr_common, colorify, content, end_chunk, end_do_block, end_el, refine_value,
		start_chunk, start_do_block, start_el, start_el_common,
	};
}
