mod build_codes;
mod chunk;
mod context;
#[doc(hidden)]
#[cfg(feature = "css-types")]
pub mod css_props;
#[cfg(feature = "html-types")]
pub mod html_types;

pub use {
	chunk::{ChunkBuild, RemovableChunk},
	context::{CtxHandle, CtxOptions, DomContext},
	neoview,
	neoview::chunk,
};
pub mod prelude {
	pub use crate::{ChunkBuild, DomContext, build_codes::__buildcode};
	pub use neoview::{GlobalStoreProv, LocalStoreProv, PropId, ScopedStoreProv, StoreProv, chunk};
}
