//! # NeoView Web
//! The web renderer for [`neoview`].
//!
//! `neoview-web` is the official web renderer for the [`neoview`] framework, based on [`HTML`](https://developer.mozilla.org/en-US/docs/Web/HTML) and [`DOM`](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model) technologies and interfacing through the [`wasm-bindgen`](::wasm_bindgen) crate.
//!
//! Aligned with [`neoview`]'s core principles, `neoview-web` supports ergonomic, fully reactive UI definitions with a strong emphasis on safety, robustness, and efficiency.
//!
//! Like every [`neoview`] renderer, it utilizes the efficiency of [fine-grained reactivity](neoview#reactive-system), the safety of [context passing](neoview#reactive-system), and the ergonomics of [chunked templating](neoview#templating) to provide high-level expressiveness with low-level robustness.
//!
//! # Features
//! `neoview-web` features its own context, [`DomContext`]; it borrows the reactive system from [`neoview`] and provides its own flavor of the [`chunk`](macro@chunk) macro with full HTML support.
//!
//! In addition to the feature-richness of [`chunk`](macro@chunk), `neoview-web` includes [conditional rendering](apply::show_if), [list rendering](render_list), a [builder pattern](apply) for templating, and [IntelliSense for tags, attributes, events, and CSS properties](#html-types).
//!
//! Here is a simple example without the initialization boilerplate:
//! ```
//! chunk!(build, div {
//! 	h3 { "Hello world!" }
//! 	do {
//! 		let count = build.prop(0);
//! 		chunk!(build, button(
//! 			on.click: (move |ctx, _| ctx.update(count, |v| *v += 1))
//! 		) { "count: ", count });
//! 	}
//! });
//! ```
//!
//! For a more in-depth introduction, check out the [guide section](docs::guide).
//!
//! # Crate Features
//! ### `html-types`
//! Provides IntelliSense for HTML tags, attributes, and events (autocompletion and hover descriptions).
//!
//! It is an optional, quality-of-life feature with no runtime cost.
//! ### `css-types`
//! Provides IntelliSense for CSS properties (autocompletion and hover descriptions).
//!
//! It is an optional, quality-of-life feature with no runtime cost.
//!
//! It requires the `html-types` feature.
pub mod apply;
mod bindings;
mod build_codes;
mod chunk;
mod context;
#[doc(hidden)]
#[cfg(feature = "css-types")]
pub mod css_props;
#[cfg(feature = "html-types")]
pub mod html_types;
mod list_render;

#[cfg(doc)]
pub mod docs {
	pub mod guide;
}

pub use {
	chunk::{ChunkBuild, RemovableChunk},
	context::{CtxHandle, CtxOptions, DomContext, get_ctx, use_ctx},
	list_render::{render_list, render_list_enumerated},
	neoview,
	neoview::chunk,
};
pub mod prelude {
	pub use crate::{ChunkBuild, DomContext, build_codes::__buildcode};
	pub use neoview::{PropId, ScopedStoreProv, StoreProv, chunk};
}
