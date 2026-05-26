mod chunk_build;
mod context;
mod wire;
pub use chunk_build::ChunkBuild;

mod binder {
	use std::sync::atomic::{AtomicU64, Ordering};

	use wasm_bindgen::prelude::*;
	use web_sys::Element;

	static CUR_EL_ID: AtomicU64 = AtomicU64::new(1);
	pub fn next_el_id() -> u64 {
		CUR_EL_ID.fetch_add(1, Ordering::Relaxed)
	}
	static CUR_CHUNK_ID: AtomicU64 = AtomicU64::new(1);
	pub fn next_chunk_id() -> u64 {
		CUR_CHUNK_ID.fetch_add(1, Ordering::Relaxed)
	}

	#[wasm_bindgen(module = "neoview-web-binder")]
	extern "C" {
		pub fn construct(target_el: &Element, build_codes: Vec<u8>, args: Vec<JsValue>);
		pub fn remove_chunk(chunk_id: u64);
	}
}
