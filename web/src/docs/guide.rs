//! # Introduction
//! `neoview-web` apps are compiled as WebAssembly (Wasm) files. It is recommended to read the [`wasm-bindgen` introduction](https://wasm-bindgen.github.io/wasm-bindgen/) before continuing.
//!
//! Here is an example `Cargo.toml`:
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
//! Here is a minimal example `lib.rs`:
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
//! This may look like a lot of code, but it is just the initialization code, the rest of the code is much more ergonomic.
//!
//! First, let's start with:
//!
//! # [`DomContext`]
//! Unlike other web frameworks that favor magic and ergonomics by providing a very concise API, `neoview-web` and the entire `neoview` ecosystem favor a middle ground of robustness and safety.
//!
//! In `neoview-web`, the UI has a clear ownership model. All of the UI is owned by a single struct called [`DomContext`]. All interactions with the UI occur through this [`DomContext`], and the UI is dropped when the [`DomContext`] is dropped.
//!
//! This single struct will be passed by mutable reference from one place to another throughout your program. Do not be afraid, it is just one argument named `ctx` with no lifetime nightmares, as every anonymous type is [`Copy`].
//!
//! ```
//! // functions will be like
//! fn fun(ctx: &mut DomContext, prop: PropId<String>) {
//! 	println!("{}", ctx.read(prop));
//! }
//! ```
//!
//! Generally, a single [`DomContext`] will be created in the main function. It is created with [`DomContext::new`], which requires the root element and a [`CtxOptions`] (just passing `Default::default()` is fine).
//!
//! [`DomContext::new`] returns a [`CtxHandle`], which is a wrapper around [`DomContext`] that needs to be kept alive for the duration of the UI. Just call [`std::mem::forget`] on it and forget about it.
//!
//! # Reactivity
//! Reactivity is essential to every UI framework, and `neoview-web` is no different.
//!
//! `neoview-web` uses fine-grained reactivity to update only the required parts of the UI. However, there are no signals, only old-school property access and mutation.
//!
//! We all love signals, but they require heavy infrastructure and come with unregulated lifetimes. In contrast, the plain old property access method is only a few characters longer and is much more robust.
//!
//! [`prop`](Store::prop) creates a reactive property that is identified by a [`Copy`]able [`PropId`], and methods like [`read`](Store::read), [`write`](Store::write), [`get`](Store::get), and [`update`](Store::update) are used to access and mutate properties.
//!
//! - [`read`](Store::read): Returns a reference to a reactive property's value.
//! - [`write`](Store::write): Writes to a reactive property's value.
//! - [`get`](Store::get): Returns a copy of a reactive property's value.
//! - [`update`](Store::update): Updates a reactive property with an updater function.
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
//! Note that not only does the [`DomContext`] provide these methods, but most of the types you work with also provide them through the [`StoreProv`] trait.
//!
//! Reactive code that needs to be executed every time some properties change is put inside [`effect`s](Store::effect). An effect's properties are identified implicitly (if they are always the same), and the [`DomContext`] is passed to the effects, allowing you to relax and focus on your code.
//!
//! ```
//! let nb = ctx.prop(1);
//! ctx.effect(move |ctx| println!("nb: {}", ctx.read(nb)));
//! ctx.write(nb, 2); // => nb: 2
//! ```
//!
//! There are also [`computed` properties](Store::computed) that are derived from other properties based on a reactive expression; they offer the same ergonomics as effects.
//! ```
//! let nb = ctx.prop(1);
//! let doubled = ctx.computed(move |ctx| ctx.get(nb) * 2);
//! println!("doubled: {}", ctx.get(doubled)); // => doubled: 2
//! ```
//!
//! Note that effects do not execute immediately upon calls to [`write`](Store::write). Instead, they get queued and executed when [`flush_updates`](Store::flush_updates) is called, though most of the time the framework handles this for you.
//!
//! Now that you understand the reactivity primitives, let's start with the UI.
//!
//! # UI Construction
//! When you create your [`DomContext`], you start UI construction by creating a chunk with [`DomContext::root_chunk`], then using the [`chunk`](macro@chunk) macro recursively to construct the UI, and finally committing the UI with [`ChunkBuild::build`].
//!
//! The UI in `neoview-web`, like in other fine-grained reactive frameworks, is initially built once, and updates flow directly to specific elements using fine-grained reactivity.
//!
//! However, instead of having the UI as expressions defined at the end of functions, the UI in `neoview-web` is constructed in multiple chunks. The [`ChunkBuild`] uses the builder pattern, but instead of raw draw calls, you define entire chunks of UI.
//! ```
//! chunk!(build, div { "part 1" });
//! chunk!(build, div { "part 2" });
//! ```
//!
//! A result of this approach is that you can inline logic directly into the UI, and you can use functions of any size, allowing logic and UI to interleave seamlessly.
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
//! You can also use any control flow you want, including imperative patterns. There are no components, only functions that borrow the context.
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
//! Now let's recap the [`chunk`](macro@chunk) macro syntax.
//!
//! ## Chunk Syntax
//! Elements are created using an object-like syntax: a tag, followed by a list of attributes, and then a list of children.
//!
//! Attributes are written inside parentheses and separated by commas, names and values are separated by a colon.
//!
//! Attributes accept `&str` and numbers as values, which get stringified. They also accept a `bool` that specifies if the attribute is present or not, as well as an `Option<T>` where the attribute is removed if `None` and set to the value of `T` if `Some(T)`.
//!
//! Attributes can be dynamic based on a reactive property simply by passing it as a value, or based on a reactive expression by passing a `FnMut(&mut DomContext) -> T`, which has the same ergonomics as effects.
//! ```
//! let hidden = build.prop(true);
//! chunk!(build,
//! 	span(id: "an-element") { "text" }
//! 	input(hidden)
//! 	a(href: move |ctx| ctx.get(hidden).not().then_some("https://example.com")) { "link" }
//! );
//! ```
//!
//! If the attribute name is `class.name`, the class `name` will be toggled statically and dynamically based on a `bool` value.
//!
//! If the attribute name is `style.name`, the CSS style property `name` will be set statically and dynamically based on a `&str` value, an `Option<&str>` can be used as well.
//!
//! If the attribute name is `on.event`, the given event listener will be attached to the specified `event`.
//! ```
//! let hidden = build.prop(true);
//! chunk!(build,
//! 	span(class.hidden: hidden) { "text" }
//! 	div(style.color: move |ctx| if ctx.get(hidden) { Some("red") } else { None }) { "other text" }
//! 	button(on.click: (move |ctx, _| ctx.update(hidden, |v| *v = !*v))) { "toggle" }
//! );
//! ```
//!
//! The element body is a list of children enclosed inside curly braces. Children can be other elements, or `&str`, numbers, and `bool`s that get stringified and inserted as text nodes.
//! ```
//! let text = build.prop(String::from("abc"));
//! chunk!(build, div {
//! 	span { "text" },
//! 	1 + 2,
//! 	text,
//! 	move |ctx| ctx.read(text).len(),
//! });
//! ```
//! Children can also be a do block, which are code blocks that get evaluated at the point they are defined.
//!
//! Any [`chunk`](macro@chunk) call inside a do block targets the position of that do block. [`ChunkBuild`] is not a linear builder but a tree-based builder where chunks can be nested endlessly.
//!
//! There are shorthands for `if`, `for`, and `match` expressions.
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
//! ## Other Features
//! If you do not prefer the macro syntax, there is also a [builder syntax](crate::apply).
//! ```
//! let count = build.prop(0);
//! build.apply(button((
//! 	on("click", move |ctx, _| ctx.update(count, |v| *v += 1)),
//! 	text(count),
//! )));
//! ```
//!
//! Reactive conditional rendering is done using [`show_if`](apply::show_if), while list rendering is done using [`render_list`].
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
//! Element references are created by calling [`ChunkBuild::ref_el`] when inside the target element.
//! ```
//! build.ref_el(|ctx, el| el.focus());
//! ```
use crate::{neoview::Store, prelude::*, *};
