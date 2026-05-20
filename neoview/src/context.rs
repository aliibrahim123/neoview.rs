use crate::reactive::Store;

pub trait Context: Sized {
	fn store(&mut self) -> &mut Store<Self>;
	fn store_ref(&self) -> &Store<Self>;
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct VoidContext {
	store: Store<Self>,
}
impl Context for VoidContext {
	fn store(&mut self) -> &mut Store<Self> {
		&mut self.store
	}
	fn store_ref(&self) -> &Store<Self> {
		&self.store
	}
}
