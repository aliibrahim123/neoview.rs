use neoview::{PropId, StoreProv};

use crate::{
	bindings::StyleValue,
	prelude::{ChunkBuild, DomContext},
};

/// The [`Applicable`] implementation for [`show_if`].
#[doc(hidden)]
pub trait ShowIfValue {
	fn apply(self, build: &mut ChunkBuild);
}
impl ShowIfValue for bool {
	fn apply(self, build: &mut ChunkBuild) {
		if !self {
			build.build_codes.style("display", "none");
		}
	}
}
impl ShowIfValue for PropId<bool> {
	fn apply(self, build: &mut ChunkBuild) {
		StyleValue::apply(
			move |ctx: &mut DomContext| if ctx.get(self) { "" } else { "none" },
			build,
			"display".into(),
		);
	}
}
impl<F: FnMut(&mut DomContext) -> bool + 'static> ShowIfValue for F {
	fn apply(mut self, build: &mut ChunkBuild) {
		StyleValue::apply(
			move |ctx: &mut DomContext| if self(ctx) { "" } else { "none" },
			build,
			"display".into(),
		);
	}
}

/// shows and hides the current element based on `value`.
///
/// The `value` can be:
/// - [`bool`]: shows the element if `value` is `true`.
/// - [`PropId<bool>`]: every time the property changes the element is hidden or shown based on its value.
/// - [`ComputedExpr<bool>`](crate::chunk!#computedexprt): the element is hidden or shown based on the evaluated value.
///
/// # Example
/// ```
/// let visible = build.prop(true);
/// show_if(build, visible);
/// ```
pub fn show_if(build: &mut ChunkBuild, value: impl ShowIfValue) {
	value.apply(build);
}
