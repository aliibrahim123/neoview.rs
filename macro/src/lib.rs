use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::{build_coder::encode, cursor::Error, parse::parse_chunk_input};

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
