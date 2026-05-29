use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{ToTokens, quote};

use crate::{
	build_coder::encode,
	cursor::{Cursor, Error},
	parse::parse_chunk_input,
};

mod build_coder;
mod cursor;
mod parse;

fn chunk_impl(input: TokenStream2) -> Result<TokenStream2, Error> {
	let input = parse_chunk_input(input)?;
	Ok(encode(input))
}

#[proc_macro]
pub fn chunk(input: TokenStream) -> TokenStream {
	match chunk_impl(input.into()) {
		Ok(tokens) => tokens.into(),
		Err(err) => err.into_token_stream().into(),
	}
}

fn kababify_impl(input: TokenStream2) -> Result<TokenStream2, Error> {
	let mut cur = Cursor::new(input.into(), Span::call_site());
	let mut parts = vec![cur.ident()?];
	while cur.try_punct('-') {
		parts.push(cur.ident()?);
	}
	if parts.len() == 1 {
		let snake = &parts[0];
		let mut kebab = Literal::string(&snake.to_string().replace('_', "-"));
		kebab.set_span(snake.span());
		Ok(quote! { #kebab })
	} else {
		let mut kebab = parts[0].to_string();
		for part in &parts[1..] {
			kebab.push('-');
			kebab.push_str(&part.to_string());
		}
		let kebab = Literal::string(&kebab);
		Ok(quote! { #kebab })
	}
}

#[proc_macro]
pub fn kababify(input: TokenStream) -> TokenStream {
	match kababify_impl(input.into()) {
		Ok(tokens) => tokens.into(),
		Err(err) => err.into_token_stream().into(),
	}
}
