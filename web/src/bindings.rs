//! define the bindings to the dom (+ static)
// sorry trait solver for this madness
use std::borrow::Cow;

use neoview::{EffectDeps::Manual, PropId, Store, StoreProv, TrackResult};
use wasm_bindgen::prelude::{JsCast, JsValue};
use web_sys::{Element, HtmlElement, Node, Text, js_sys::Reflect};

use crate::{
	chunk::ChunkId,
	prelude::{ChunkBuild, DomContext},
};

// marker types, implementaion overloading.
pub struct Static;
pub struct Prop;
pub struct Computed;
pub struct Static2;
pub struct Prop2;
pub struct Computed2;

/// because typing `el.dyn_ref::<HtmlElement>().unwrap()` is too long
fn to_html_el(el: &Element) -> &HtmlElement {
	el.dyn_ref().unwrap()
}
/// add effect in the build scope, manual dependencies and `init_run: false`
fn add_effect(
	build: &mut ChunkBuild, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
	fun: impl FnMut(&mut DomContext) + 'static,
) {
	Store::effect(build.ctx, build.slab, Manual { read, write, init_run: false }, fun).unwrap();
}
/// add effect in the build scope, manual dependencies and `init_run: false`, the effect get passed the chunk and el ids.
fn add_effect_with_el(
	build: &mut ChunkBuild, read: Vec<PropId<()>>, write: Vec<PropId<()>>,
	mut fun: impl FnMut(&mut DomContext, ChunkId, usize) + 'static,
) {
	let el = build.build_codes.request_id();
	let chunk = build.id;
	let fun = move |ctx: &mut DomContext| {
		fun(ctx, chunk, el as usize);
	};
	Store::effect(build.ctx, build.slab, Manual { read, write, init_run: false }, fun).unwrap();
}

fn call_with_track<T>(
	build: &mut ChunkBuild, fun: &mut impl FnMut(&mut DomContext) -> T,
) -> (T, TrackResult) {
	build.store().start_track().unwrap();
	let value = fun(build.ctx);
	let result = build.store().end_track().unwrap();
	(value, result)
}

/// convert a value into an attribute value, set `Some<str>` and remove on `None`
trait BasicAttrValue {
	/// convert the value into attribute value and pass it to `fun`
	// since and we cant return a ref to a local var.
	fn with(&self, fun: impl FnOnce(Option<&str>));
}
impl BasicAttrValue for &str {
	fn with(&self, fun: impl FnOnce(Option<&str>)) {
		fun(Some(*self))
	}
}
impl BasicAttrValue for str {
	fn with(&self, fun: impl FnOnce(Option<&str>)) {
		fun(Some(self))
	}
}
impl BasicAttrValue for char {
	fn with(&self, fun: impl FnOnce(Option<&str>)) {
		fun(Some(self.encode_utf8(&mut [0; 4])))
	}
}
impl BasicAttrValue for String {
	fn with(&self, fun: impl FnOnce(Option<&str>)) {
		fun(Some(&self))
	}
}
impl BasicAttrValue for bool {
	fn with(&self, fun: impl FnOnce(Option<&str>)) {
		fun(self.then_some(""))
	}
}
macro_rules! basic_attr_int {
	($($ty:ty),*) => {
		$(impl BasicAttrValue for $ty {
			fn with(&self, fun: impl FnOnce(Option<&str>)) {
				fun(Some(&self.to_string()))
			}
		})*
	};
}
basic_attr_int!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64);
impl<T: BasicAttrValue> BasicAttrValue for &T {
	fn with(&self, fun: impl FnOnce(Option<&str>)) {
		(*self).with(fun)
	}
}
impl<T: BasicAttrValue> BasicAttrValue for Option<T> {
	fn with(&self, fun: impl FnOnce(Option<&str>)) {
		if let Some(v) = self {
			v.with(fun);
		} else {
			fun(None)
		}
	}
}

/// apply an attribute, static or dynamic
pub trait AttrValue<Value> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>);
}
impl<T: BasicAttrValue> AttrValue<Static> for T {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		self.with(|value| {
			value.map(|value| build.build_codes.attr(&name, value));
		});
	}
}
impl<T: BasicAttrValue> AttrValue<Prop> for PropId<T> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.ctx.read(self).with(|value| {
			value.map(|value| build.build_codes.attr(&name, value));
		});

		add_effect_with_el(build, vec![self.erase_type()], vec![], move |ctx, chunk, el| {
			ctx.read(self).with(|value| {
				let el = &ctx.chunks[chunk].elements[el];
				if let Some(value) = value {
					el.set_attribute(&name, value).unwrap();
				} else {
					el.remove_attribute(&name).unwrap();
				}
			})
		});
	}
}
impl<T: BasicAttrValue, F: FnMut(&mut DomContext) -> T + 'static> AttrValue<Computed> for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		let (value, TrackResult { read, written }) = call_with_track(build, &mut self);
		value.with(|value| {
			value.map(|value| build.build_codes.attr(&name, value));
		});

		add_effect_with_el(build, read, written, move |ctx, chunk, el| {
			let value = self(ctx);
			let el = &ctx.chunks[chunk].elements[el];
			value.with(|value| {
				if let Some(value) = value {
					el.set_attribute(&name, value).unwrap();
				} else {
					el.remove_attribute(&name).unwrap();
				}
			});
		});
	}
}

/// apply a class, static or dynamic
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
		let (value, TrackResult { read, written }) = call_with_track(build, &mut self);
		if value {
			build.build_codes.class(&name);
		}

		add_effect_with_el(build, read, written, move |ctx, chunk, el| {
			let value = self(ctx);
			let el = &ctx.chunks[chunk].elements[el];
			el.class_list().toggle_with_force(&name, value).unwrap();
		});
	}
}

/// convert value to style value
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

/// apply a css style, static or dynamic
pub trait StyleValue<Value> {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>);
}
impl<T: BasicStyleValue> StyleValue<Static> for T {
	fn apply(self, build: &mut ChunkBuild<'_>, name: Cow<'static, str>) {
		build.build_codes.style(&name, self.to_str());
	}
}
impl<T: BasicStyleValue> StyleValue<Prop> for PropId<T> {
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
		let (value, TrackResult { read, written }) = call_with_track(build, &mut self);
		build.build_codes.style(&name, value.to_str());

		add_effect_with_el(build, read, written, move |ctx, chunk, el| {
			let value = self(ctx);
			let el = &ctx.chunks[chunk].elements[el];
			to_html_el(el).style().set_property(&name, value.to_str()).unwrap();
		});
	}
}

/// set an element property
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
		let (value, TrackResult { read, written }) = call_with_track(build, &mut self);
		build.build_codes.prop(&name, value);

		add_effect_with_el(build, read, written, move |ctx, chunk, el| {
			let value = self(ctx);
			let el = &ctx.chunks[chunk].elements[el];
			Reflect::set(el, &name.as_ref().into(), &value).unwrap();
		});
	}
}

/// convert value to text
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
impl BasicTextValue for char {
	fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R {
		fun(self.encode_utf8(&mut [0; 4]))
	}
}
macro_rules! basic_text_primitive {
	($($ty:ty),+) => {
		$(impl BasicTextValue for $ty {
			fn with<R>(&self, fun: impl FnOnce(&str) -> R) -> R {
				fun(&self.to_string())
			}
		})+
	};
}
basic_text_primitive!(
	bool, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, usize, u128, f32, f64
);
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

/// text binding
pub trait TextValue<Value> {
	fn apply(self, build: &mut ChunkBuild<'_>);
}
impl<T: BasicTextValue> TextValue<Static> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self.with(|value| {
			(value != "").then(|| build.build_codes.text(value));
		});
	}
}
impl<T: BasicTextValue> TextValue<Prop> for PropId<T> {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		let node = build.ctx().read(self).with(|value| Text::new_with_data(value).unwrap());
		build.build_codes.node(node.clone().into());

		add_effect(build, vec![self.erase_type()], vec![], move |ctx| {
			ctx.read(self).with(|value| node.set_text_content(Some(value)));
		});
	}
}
impl<T: BasicTextValue, F: FnMut(&mut DomContext) -> T + 'static> TextValue<Computed> for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>) {
		let (value, TrackResult { read, written }) = call_with_track(build, &mut self);
		let node = value.with(|value| Text::new_with_data(value).unwrap());
		build.build_codes.node(node.clone().into());

		add_effect(build, read, written, move |ctx| {
			self(ctx).with(|value| node.set_text_content(Some(value)));
		});
	}
}

/// node binding
pub trait NodeValue<Value> {
	fn apply(self, build: &mut ChunkBuild<'_>);
}
impl<T: Into<Node>> NodeValue<Static> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		build.build_codes.node(self.into());
	}
}
impl<T: Into<Node> + Clone> NodeValue<Prop> for PropId<T> {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		let mut node = build.ctx().read(self).clone().into();
		build.build_codes.node(node.clone());

		add_effect(build, vec![self.erase_type()], vec![], move |ctx| {
			let cur = ctx.read(self).clone().into();
			node.parent_node().unwrap().replace_child(&cur, &node).unwrap();
			node = cur;
		});
	}
}
impl<T: Into<Node>, F: FnMut(&mut DomContext) -> T + 'static> NodeValue<Computed> for F {
	fn apply(mut self, build: &mut ChunkBuild<'_>) {
		let (value, TrackResult { read, written }) = call_with_track(build, &mut self);
		let mut node = value.into();
		build.build_codes.node(node.clone());

		add_effect(build, read, written, move |ctx| {
			let cur = self(ctx).into();
			node.parent_node().unwrap().replace_child(&cur, &node).unwrap();
			node = cur;
		});
	}
}

/// content binding
pub trait ContentValue<Value> {
	fn apply(self, build: &mut ChunkBuild<'_>);
}

impl<T: TextValue<Static>> ContentValue<Static> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self.apply(build);
	}
}
impl<T: TextValue<Prop>> ContentValue<Prop> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self.apply(build);
	}
}
impl<T: TextValue<Computed>> ContentValue<Computed> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self.apply(build);
	}
}
impl<T: NodeValue<Static>> ContentValue<Static2> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self.apply(build);
	}
}
impl<T: NodeValue<Prop>> ContentValue<Prop2> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self.apply(build);
	}
}
impl<T: NodeValue<Computed>> ContentValue<Computed2> for T {
	fn apply(self, build: &mut ChunkBuild<'_>) {
		self.apply(build);
	}
}
