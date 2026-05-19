pub trait Context: 'static {
	type Id: Copy;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VoidContext;
impl Context for VoidContext {
	type Id = ();
}
impl VoidContext {
	pub fn get_by_id(id: (), _life_marker: &()) -> &VoidContext {
		&VoidContext
	}
}
