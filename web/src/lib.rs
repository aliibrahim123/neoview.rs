//! # NeoView Web
//! the web renderer for [`neoview`].
//!
//! `neoview-web` is the official web renderer for the [`neoview`] framework based on [`HTML`](https://developer.mozilla.org/en-US/docs/Web/HTML) and [`DOM`](https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model) technology and interfacing through the [`wasm-bindgen`] crate.
//!
//! aligned with [`neoview`] core principles, `neoview-web` supports ergonomic, fully reactive UI definitions with a strong emphasis on safety, robustness, efficiency.
//!
//! like every [`neoview`] renderer, it utilizes the effecincy of [fine grained reactivity](neoview#reactive-system), the safety of [context passing](neoview#reactive-system) and the ergonomic of [chunked templating](neoview#templating) to provide high level expressiveness with low level robustness.
//!
//! # Features
//! `neoview-web` has its own context [`DomContext`], it borrows the reactive systems from [`neoview`] and has its own flavor of [`chunk`](macro@chunk) with full html support.
//!
//! in additional to the featurefullness of [`chunk`](macro@chunk), `neoview-web` has [conditional rendering](apply::show_if), [list rendering](render_list), a [builder pattern](apply) for templating, and [tags, attributes, events and css props intelesence](#html-types).
//!
//! here is a simple example, without the init bolerplate:
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
//! for a more in depth introduction, check out [guide section](docs::guide).
//!
//! # crate features
//! ### `html-types`
//! provides intelesence for html tags, attributes, and events (autocomplete and hover discription).
//!
//! it is an optional quality of life feature with no runtime cost.
//! ### `css-types`
//! provides intelesence for css properties (autocomplete and hover discription).
//!
//! it is an optional quality of life feature with no runtime cost.
//!
//! it requires `html-types`.
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
