use std::{env::var_os, fs::read_to_string, path::PathBuf};

use proc_macro::{TokenStream, TokenTree};
use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::quote;

#[proc_macro]
pub fn kababify(input: TokenStream) -> TokenStream {
	let mut input = input.into_iter();
	let TokenTree::Ident(start_ident) = input.next().unwrap() else { panic!() };
	let mut parts = vec![start_ident];
	while input.next().is_some() {
		let TokenTree::Ident(ident) = input.next().unwrap() else { panic!() };
		parts.push(ident);
	}

	if parts.len() == 1 {
		let snake = &parts[0];
		let mut kebab = Literal::string(&snake.to_string().replace('_', "-"));
		kebab.set_span(snake.span().into());
		quote! { #kebab }.into()
	} else {
		let mut kebab = parts[0].to_string();
		for part in &parts[1..] {
			kebab.push('-');
			kebab.push_str(&part.to_string());
		}
		let kebab = Literal::string(&kebab);
		quote! { #kebab }.into()
	}
}

#[proc_macro_attribute]
pub fn wasm_bindgen_from_build(attr: TokenStream, item: TokenStream) -> TokenStream {
	let mut attr = attr.into_iter();
	let TokenTree::Literal(lit) = attr.next().unwrap() else { panic!() };
	let lit = lit.to_string();

	let mut path = PathBuf::from(var_os("OUT_DIR").unwrap());
	path.push(&lit[1..lit.len() - 1]);
	let content = read_to_string(path).unwrap();

	let item: TokenStream2 = item.into();
	quote! {
		#[wasm_bindgen(inline_js = #content)]
		#item
	}
	.into()
}
