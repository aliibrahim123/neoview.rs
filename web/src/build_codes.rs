//! defines the build codes
use neoview_web_macro::wasm_bindgen_from_build;
use slotmap::{Key, KeyData};
use wasm_bindgen::prelude::{Closure, JsCast, JsValue, wasm_bindgen};
use web_sys::{Element, Event, Node, js_sys::Function};

use crate::{chunk::ChunkId, context::ContextId, use_ctx};

#[wasm_bindgen_from_build("binder.js")]
extern "C" {
	/// contruct an element from build codes, returning the requested elements.
	pub fn construct(
		target_el: &Element, build_codes: Vec<u8>, props: Vec<JsValue>, nodes: Vec<Node>,
	) -> Vec<Element>;
	pub fn register_event_callback(fun: &Function);
}

/// called by `binder.js` on events.
pub fn recieve_event(ctx: u32, chunk: u32, fun_id: u32, event: Event) {
	use_ctx(ContextId(ctx as u64), |ctx| {
		let chunk = ChunkId::from(KeyData::from_ffi(chunk as u64));
		let mut fun = ctx.chunks[chunk].events[fun_id as usize].take().unwrap();
		fun(ctx, event);
		ctx.chunks[chunk].events[fun_id as usize] = Some(fun);
	})
	.unwrap();
}

#[wasm_bindgen(start)]
pub fn neoview_init_binder() {
	let closure = Closure::<dyn Fn(u32, u32, u32, Event)>::new(|ctx, chunk, fun_id, event| {
		recieve_event(ctx, chunk, fun_id, event)
	});
	register_event_callback(closure.as_ref().unchecked_ref());
	closure.forget();
}

/// list of common names that get passed by index not directly encoded.
const COMMON_NAMES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/common_names.rs"));

/// bytecode of ui build instructions generated while building a chunk.
///
/// executed by `binder.js` in one batch per `ChunkBuild` to make initial construction efficient.
///
/// # encoding
/// one opcode byte and n oprand bytes, it is composed of a list of build codes targeting the base element.
///
/// `vuint`: LEB128 unsigned integer.
///
/// `str`: utf8 string composed of a vuint encoding the length followed by the bytes.
///
/// `name`: `str` with short encoding for common names, composed of a vuint tag, if first bit of tag is `1` the rest of the tag encode an index to `COMMON_NAMES`, else the rest of the tag encode the length and it is followed by the bytes.
///
/// buildcode:
/// - `0 tag:name buildcode* 0xff`: construct an element of `tag` and apply `buildcode`s to it.
/// - `1`: requrest the element and push it into a buffer that gets returned at the end.
/// - `2 attr:name value:str`: apply attribute `attr` with value `value`.
/// - `3 prop:name val_ind:vuint`: set element property `prop` to value stored in `props[val_ind]`.
/// - `4 class:str`: add class `class` to element.
/// - `5 prop:name value:str`: set css property `prop` to value `value`.
/// - `6 text:str`: append text `text` to the element.
/// - `7 node_ind:vuint`: append dom node stored in `nodes[node_ind]` to the element.
/// - `8 ctx:vuint chunk:vuint event:name fun:vuint`: add listener to the event `event` on the element, the function is identified by `(ctx, chunk, fun)`.
#[derive(Debug)]
pub struct BuildCodes {
	/// the main bytecode
	codes: Vec<u8>,
	/// elements properties
	props: Vec<JsValue>,
	/// nodes to be appended
	nodes: Vec<Node>,
	/// the next element id
	next_el_id: u64,
	/// a stack of element ids, can be `None` is not required
	el_id_stack: Vec<Option<u64>>,
}
impl BuildCodes {
	pub fn new() -> Self {
		Self {
			codes: Vec::new(),
			props: Vec::new(),
			nodes: Vec::new(),
			// the base element is always ided 0
			next_el_id: 1,
			el_id_stack: vec![Some(0)],
		}
	}
	/// push a slice of bytes.
	pub fn push_slice(&mut self, slice: &[u8]) {
		self.codes.extend_from_slice(slice);
	}
	/// push a LEB128 unsigned integer.
	///
	/// a LEB128 number is a variable length integer composed of little endian bytes where the 8th bit acts as a continuation bit.
	pub fn push_vuint(&mut self, mut value: u64) {
		let mut there_input = true;

		while there_input {
			let byte = value as u8 & 0b0111_1111;
			value >>= 7;
			self.codes.push(if value == 0 { byte } else { byte | 0b1000_0000 });
			there_input = value != 0;
		}
	}
	/// push a utf8 string.
	pub fn push_str(&mut self, str: &str) {
		self.push_vuint(str.len() as u64);
		self.push_slice(str.as_bytes());
	}
	/// push a utf8 string with short encoding for common names.
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

	/// start an element
	pub fn start_el(&mut self, tag: &str) {
		self.codes.push(Self::EL_START);
		self.push_name(tag);
		self.el_id_stack.push(None);
	}
	/// request the current element so it can be used in bindings
	pub fn request_id(&mut self) -> u64 {
		if let Some(id) = self.el_id_stack.last().copied().flatten() {
			return id;
		}

		let id = self.next_el_id;
		self.next_el_id += 1;
		let last_ind = self.el_id_stack.len() - 1;
		self.el_id_stack[last_ind] = Some(id);

		self.codes.push(Self::EL_ID);
		id
	}
	/// add an attribute
	pub fn attr(&mut self, name: &str, value: &str) {
		self.codes.push(Self::ATTR);
		self.push_name(name);
		self.push_str(value);
	}
	/// set a property
	pub fn prop(&mut self, name: &str, value: JsValue) {
		self.codes.push(Self::PROP);
		self.push_str(name);
		self.push_vuint(self.props.len() as u64);
		self.props.push(value);
	}
	/// add a class
	pub fn class(&mut self, value: &str) {
		self.codes.push(Self::CLASS);
		self.push_str(value);
	}
	/// set a css property
	pub fn style(&mut self, name: &str, value: &str) {
		self.codes.push(Self::STYLE);
		self.push_name(name);
		self.push_str(value);
	}
	/// append text
	pub fn text(&mut self, value: &str) {
		self.codes.push(Self::TEXT);
		self.push_str(value);
	}
	/// append a dom node
	pub fn node(&mut self, value: Node) {
		self.codes.push(Self::NODE);
		self.push_vuint(self.nodes.len() as u64);
		self.nodes.push(value);
	}
	/// add an event listener
	pub fn event(&mut self, ctx_id: ContextId, chunk_id: ChunkId, name: &str, fn_id: u64) {
		self.codes.push(Self::EVENT);
		self.push_vuint(ctx_id.0);
		self.push_vuint(chunk_id.data().as_ffi());
		self.push_name(name);
		self.push_vuint(fn_id);
	}
	/// terminate the buildcodes of the current element
	pub fn end_el(&mut self) {
		self.codes.push(Self::END);
		self.el_id_stack.pop();
	}
	/// construct the chunk, returning the requested elements
	pub fn construct(mut self, base_el: &Element) -> Vec<Element> {
		self.codes.push(Self::END);
		// submit the batch to the `binder.js`
		construct(base_el, self.codes, self.props, self.nodes)
	}
}

/// the [buildcodes module](neoview::chunk#buildcodes) for the `neoview-web` renderer.
///
/// this module contains the buildcodes macro required for the [`chunk`](crate::chunk!) macro.
///
/// it is exposed since if you dont like to import the [`prelude`](crate::prelude) module.
pub mod __buildcode {
	// the element type is `()` since it is unused
	// items in `html_types` crate are inserted to provide intellisense and syntax highlighting.
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
		($($t:tt)*) => {
			($build:expr, $el:expr, $tag:ident) => {{
				$build.build_codes.start_el(stringify!($tag));
				()
			}};
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
		($($t:tt)*) => { __buildcode::attr_html!($($t)*) };
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
			__buildcode::StyleValue::apply(
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

	// refine the value, for now it just give type inference for computed expressions.
	#[macro_export]
	#[doc(hidden)]
	macro_rules! refine_value {
		(move |$ctx:ident| $($t:tt)*) => { move |$ctx: &mut $crate::DomContext| $($t)* };
		($($t:tt)*) => { $($t)* };
	}

	// colorify idents givinf them color of variables.
	#[macro_export]
	#[doc(hidden)]
	macro_rules! colorify {
		($x:ident) => {{
			#[allow(unused, nonstandard_style)]
			let $x = ();
		}};
	}

	use crate::prelude::{ChunkBuild, DomContext};

	#[doc(hidden)]
	pub fn add_event(
		build: &mut ChunkBuild, event: &str, fun: Box<dyn FnMut(&mut DomContext, Event)>,
	) {
		let events = &mut build.ctx.chunks[build.id].events;
		build.build_codes.event(build.ctx.id, build.id, event, events.len() as u64);
		events.push(Some(fun));
	}

	#[doc(hidden)]
	pub use crate::bindings::{AttrValue, ClassValue, ContentValue, PropValue, StyleValue};
	#[doc(hidden)]
	#[cfg(feature = "html-types")]
	pub use attr_html;
	#[doc(hidden)]
	pub use neoview_web_macro::kababify;
	use web_sys::Event;
	#[doc(hidden)]
	pub use {
		attr, attr_common, colorify, content, end_chunk, end_do_block, end_el, refine_value,
		start_chunk, start_do_block, start_el, start_el_common,
	};
}
