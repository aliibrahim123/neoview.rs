use std::{
	env,
	fs::{read_to_string, write},
	path::Path,
};

fn main() {
	println!("cargo:rerun-if-changed=src/common_names.txt");

	let file = read_to_string("./src/common_names.txt").unwrap();
	let mut names = file.lines().map(|line| line.trim()).collect::<Vec<_>>();
	names.sort();

	let mut expr = "[".to_string();
	for name in &names {
		expr += &format!("\"{name}\",");
	}
	expr += "]";

	let outfile = Path::new(&env::var("OUT_DIR").unwrap()).join("common_names.rs");
	write(outfile, expr).unwrap()
}
