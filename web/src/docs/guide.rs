//! # introduction
//! `neoview-web` apps are compiled as wasm files, it is required to see the [`wasm-bindgen` introduction](https://wasm-bindgen.github.io/wasm-bindgen/) before continuing.
//!
//! here is an example `Cargo.toml`:
//! ```toml
//! [package]
//! name = "example"
//! version = "0.1.0"
//! edition = "2024"
//!
//! [lib]
//! crate-type = ["cdylib"]
//!
//! [dependencies]
//! neoview-web = { version = "0.1.0", features = ["html-types", "css-types"] }
//! wasm-bindgen = "0.2.122"
//! web-sys = {
//!     version = "0.3.99",
//!     features = ["Element", "Document", "Window" ]
//! }
//! ```
//!
//! here is an minimal example `lib.rs`:
//! ```
//! use neoview_web::prelude::*;
//! use wasm_bindgen::prelude::wasm_bindgen;
//! use web_sys::window;
//!
//! // main entry point
//! #[wasm_bindgen(start)]
//! pub fn main_js() {
//!     // get main element
//!     let el = window().unwrap().document().unwrap().query_selector("#main").unwrap().unwrap();
//!     // create context
//!     let handle = DomContext::new(el, Default::default());
//!     {
//!         let mut ctx = handle.borrow_mut();
//!         // create root chunk
//!         let mut build = ctx.root_chunk();
//!         chunk!(build, div { "Hello world!" });
//!         build.build();
//!     }
//!     // forget context, keeping it alive for the duration of the webpage
//!     std::mem::forget(handle);
//! }
//! ```
//!
//! this may look like a lot of code, but it is just the init code, the rest of the code is more ergonomic.
//!
//! first let starts with.
//!
//! # [`DomContext`]
//! unlike other web frameworks that favor magic and ergonomics and provide bery unverbose api, `neoview-web` and every `neoview` framework favors middle ground robustness and safety.
//!
//! in `neoview-web`, ui has clear owenership model, all the ui is owned by one struct called [`DomContext`], all interactions with the ui is through the [`DomContext`], and the ui is dropped when the [`DomContext`] is dropped.
//!
//! this one struct will be passed by mutable reference from one place to another through out your program, dont be afraid, it is just one argument named `ctx` with no lifetime nightmare as every ananymous type is [`Copy`].
//!
//! ```
//! // functions will be like
//! fn fun(ctx: &mut DomContext, prop: PropId<String>) {
//! 	println!("{}", ctx.read(prop));
//! }
//! ```
//!
//! generally, one [`DomContext`] will be created at the main function, it is created with [`DomContext::new`] that requires the root element and a [`CtxOptions`] (just pass `Default::default()` and you are fine).
//!
//! [`DomContext::new`] returns a [`CtxHandle`] that is a wrapper over [`DomContext`] that needs to be kept alive for the duration of the ui, just call [`std::mem::forget`] on it and forget like it.
//!
//! # reactivity
//! reactivity is essintial to every ui framework, and `neoview-web` is no different.
//!
//! `neoview-web` uses fine grained reactivity to update only the required parts of the ui, however they are no signals, only old school property access and mutation.
//!
//! we all love signals, however they requires heavy infrastructure and comes with unregulated lifetimes, however plain old property access method is only few character longer and it is so much more robust.
//!
//! [`prop`](Store::prop) creates reactive property that are identified by a [`Copy`]able [`PropId`], and methods like [`read`](Store::read), [`write`](Store::write), [`get`](Store::get), and [`update`](Store::update) are used to access and mutate properties.
//!
//! - [`read`](Store::read): returns a reference to a reactive property's value.
//! - [`write`](Store::write): writes to a reactive property's value.
//! - [`get`](Store::get): returns a copy of a reactive property's value.
//! - [`update`](Store::update): updates a reactive property with an updater function.
//!
//! ```
//! let nb = ctx.prop(1);
//! println!("nb: {}", ctx.read(nb)); // => nb: 1
//! ctx.write(nb, 2);
//! println!("nb: {}", ctx.get(nb)); // => nb: 2
//! ctx.update(nb, |v| *v += 1);
//! println!("nb: {}", ctx.read(nb)); // => nb: 3
//! ```
//!
//! note that not only the [`DomContext`] provides these methods, but most the types you work with also provides these methods through the [`StoreProv`] trait.
//!
//! reactive code that need to be executed everytime some properties change are put inside [`effect`s](Store::effect), effects properties are known implicitly (if they are always the same), and the [`DomContext`] is passed to the effects, letting you relax and focus on your code.
//!
//! ```
//! let nb = ctx.prop(1);
//! ctx.effect(move |ctx| println!("nb: {}", ctx.read(nb)));
//! ctx.write(nb, 2); // => nb: 2
//! ```
//!
//! there are also [`computed` properties](Store::computed) that are derived from other properties based on a reactive expression, they also have the same ergonomics like effects.
//! ```
//! let nb = ctx.prop(1);
//! let doubled = ctx.computed(move |ctx| ctx.get(nb) * 2);
//! println!("doubled: {}", ctx.get(doubled)); // => doubled: 2
//! ```
//!
//! note that effects dont get executed quickly on calls to [`write`](Store::write), instead they get queued and executed when [`flush_updates`](Store::flush_updates) is called, though most of the times the framework does it for you.
//!
//! and now after you had understand the reactivity primitives, let starts with the ui.
//!
//! # ui contruction
//! when you create your [`DomContext`], you starts ui construction by creating a chunk with [`DomContext::root_chunk`], then using [`chunk`](macro@chunk) macro recusively to contruct the ui, finally commit the ui with [`ChunkBuild::build`].
//!
//! ui in `neoview-web` like other fine grained reactivity framework is builted initialy once and updates flow directly to specific elements using fine-grained reactivity.
//!
//! however istead of having ui as expressions defined at the end of the functions, the ui in `neoview-web` is constructed in multiple chunks, where the [`ChunkBuild`] uses the builder pattern but instead of raw draw calls, you define entire chunks of ui.
//! ```
//! chunk!(build, div { "part 1" });
//! chunk!(build, div { "part 2" });
//! ```
//!
//! a result of this approuch is that you can inline the logic directly to the ui, and you can use functions of whatever sizes as logic and ui can interleave.
//! ```
//! // section 1
//! let count = ctx.prop(0);
//! chunk!(build, button(on.click: (move |ctx, _| ctx.update(count, |v| *v += 1))) { count });
//!
//! // section 2
//! let clock = ctx.prop(0);
//! set_interval(move |ctx| ctx.update(clock, |v| *v += 1), 1000);
//! chunk!(build, div { clock });
//! ```
//!
//! also you can use any control flow you want, and even imperative patterns, there are no components, only functions that borrow the context.
//! ```
//! fn counter(mut build: &mut ChunkBuild, name: &str) {
//! 	let count = build.prop(0);
//! 	chunk!(build, button(
//! 		on.click: (move |ctx, _| ctx.update(count, |v| *v += 1))
//! 	) { name, ": ", count });
//! }
//! // control flow is static not dynamic
//! for i in 0..10 {
//! 	counter(&mut build, &format!("counter {i}"));
//! 	match i % 3 {
//! 		0 | 1 => chunk!(build, br()),
//! 		_ => chunk!(build, hr()),
//! 	}
//! }
//! ```
//!
//! not lets recap the [`chunk`](macro@chunk) macro syntax.
//!
//! ## chunk syntax
//! elements are created in an object like syntax, a tag then a list of attributes, then a list of children.
//!
//! attributes are written inside parenthesis and separated by commas, name and value are separated by a colon.
//!
//! attributes can accept `&str` and numbers as values that get stringified, they also accepts `bool` that specifis if the attribute is present or not, and also `Option<T>` where the attribute is removed if `None` and set to the value of `T` if `Some(T)`.
//!
//! attributes can be dynamic based on a reactive property by just passing it as value, or based on a reactive expression by passing a `FnMut(&mut DomContext) -> T` that have the same ergonomics as effects.
//! ```
//! let hidden = build.prop(true);
//! chunk!(build,
//! 	span(id: "an-element") { "text" }
//! 	input(hidden)
//! 	a(href: move |ctx| ctx.get(hidden).not().then_some("https://example.com")) { "link" }
//! );
//! ```
//!
//! if attribute name is `class.name`, the class `name` will be toggled staticaly and dynamically based on a `bool` value.
//!
//! if attribute name is `style.name`, the style prop `name` will be set staticaly and dynamically based on a `&str` value, `Option<&str>` can be used as well.
//!
//! if attribute name is `on.event`, the given event listener will be attacked the `event` event.
//! ```
//! let hidden = build.prop(true);
//! chunk!(build,
//! 	span(class.hidden: hidden) { "text" }
//! 	div(style.color: move |ctx| if ctx.get(hidden) { Some("red") } else { None }) { "other text" }
//! 	button(on.click: (move |ctx, _| ctx.update(hidden, |v| *v = !*v))) { "toggle" }
//! );
//! ```
//!
//! element body is a list of children enclosed inside curly braces, children can be other elements or `&str`, numbers and `bool`s that get stringified and inserted as text nodes.
//! ```
//! let text = build.prop(String::from("abc"));
//! chunk!(build, div {
//! 	span { "text" },
//! 	1 + 2,
//! 	text,
//! 	move |ctx| ctx.read(text).len(),
//! });
//! ```
//! childrens can be also be a do block, they are code blocks that get eveluated at the point they are defined in.
//!
//! any [`chunk`](macro@chunk) call inside a do block target the position of the do block, [`ChunkBuild`] is not a uniform builder but a tree based builder where chunks can be nested endlessly.
//!
//! they are shorthand for `if`, `for` and `match` expressions.
//! ```
//! chunk!(build, div {
//! 	"part 1, "
//! 	do {
//! 		chunk!(build, "part 2, " )
//! 	}
//! 	for i in 0..10 {
//! 		chunk!(build, "part ", i + 2, ", ")
//! 	}
//! 	"part 13"
//! });
//! ```
//!
//! ## other features
//! if you dont love the macro syntax, they are also a builder syntax.
//! ```
//! let count = build.prop(0);
//! build.apply(button((
//! 	on("click", move |ctx, _| ctx.update(count, |v| *v += 1)),
//! 	text(count),
//! )));
//! ```
//!
//! reactive conditional rendering is done by [`show_if`](apply::show_if), while list rendering is done by [`render_list`].
//! ```
//! chunk!(build, div {
//! 	do {
//! 		let list = build.prop(vec![1, 2, 3]);
//! 		build.apply(show_if(move |ctx: &mut DomContext| ctx.read(list).len() > 0));
//! 		render_list(build, list, |v| *v, "div", |mut build, v| chunk!(build, v));
//! 	}
//! });
//! ```
//!
//! element references are done by calling [`ChunkBuild::ref_el`] when inside the target element.
//! ```
//! build.ref_el(|ctx, el| el.focus());
//! ```
use crate::{neoview::Store, prelude::*, *};
