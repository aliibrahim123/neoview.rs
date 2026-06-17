//! integrate common names into rust and binder.js
use std::{
	env,
	fs::{read_to_string, write},
	path::Path,
};

fn main() {
	println!("cargo:rerun-if-changed=src/common_names.txt");
	println!("cargo:rerun-if-changed=src/binder.js");

	// name1\nname2\nname3 -> ["name1", "name2", "name3"]
	let file = read_to_string("./src/common_names.txt").unwrap();
	let mut names = file.lines().map(|line| line.trim()).collect::<Vec<_>>();
	names.sort();

	let mut common_names = "[".to_string();
	for name in &names {
		common_names += &format!("\"{name}\",");
	}
	common_names += "]";

	// generate `common_names.rs`
	let common_names_path = Path::new(&env::var("OUT_DIR").unwrap()).join("common_names.rs");
	write(common_names_path, &common_names).unwrap();

	// prepend `common_names` into `binder.js`
	let binder = read_to_string("./src/binder.js").unwrap();
	let binder = format!("const common_names = {common_names};\n {binder}");
	let binder_path = Path::new(&env::var("OUT_DIR").unwrap()).join("binder.js");
	write(binder_path, binder).unwrap();
}
