use std::{fmt::Debug, panic::Location};

use crate::{context::Context, reactive::prop::ItemId};

pub struct Effect<Ctx> {
	pub fun: Box<dyn FnMut(&Ctx)>,
	pub loc: &'static Location<'static>,
	pub read: Vec<ItemId>,
	pub write: Vec<ItemId>,
}
impl<Ctx> Debug for Effect<Ctx> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Effect")
			.field("read", &self.read)
			.field("write", &self.write)
			.finish()
	}
}
