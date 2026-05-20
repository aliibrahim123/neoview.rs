use std::{fmt::Debug, panic::Location};

use crate::reactive::prop::ItemId;

pub struct Effect {
	pub fun: Box<dyn FnMut()>,
	pub loc: &'static Location<'static>,
	pub read: Vec<ItemId>,
	pub write: Vec<ItemId>,
}
impl Debug for Effect {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Effect")
			.field("loc", &self.loc)
			.field("read", &self.read)
			.field("write", &self.write)
			.finish()
	}
}
