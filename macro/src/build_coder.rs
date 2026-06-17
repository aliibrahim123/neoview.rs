//! transform [`ChunkInput`] to [`TokenStream`]

use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::{Child, ChunkInput, Element, Tag};

/// transform [`ChunkInput`] to [`TokenStream`]
pub fn encode(input: ChunkInput) -> TokenStream {
	let ChunkInput { build, children } = input;
	let mut codes = quote! {
		let mut build = &mut #(#build)*;
		let mut __el = __buildcode::start_chunk!(build);
	};

	for child in children {
		encode_child(&mut codes, child);
	}

	quote! { {
		#codes
		__buildcode::end_chunk!(build, __el);
	} }
}

fn encode_child(codes: &mut TokenStream, child: Child) {
	match child {
		Child::Content(content) => {
			codes.extend(quote! { __buildcode::content!(build, __el, #(#content)*); })
		}
		Child::Element(Element { tag, attrs, children }) => {
			let tag = match tag {
				Tag::Path(path) => quote! { #path },
				Tag::Lit(lit) => quote! { #lit },
			};
			let mut el_codes = quote! { let mut __el = __buildcode::start_el!(build, __el, #tag); };

			for (attr, value) in attrs {
				el_codes
					.extend(quote! { __buildcode::attr!(build, __el, [#(#attr)*], #(#value)*); });
			}

			for child in children {
				encode_child(&mut el_codes, child);
			}

			codes.extend(quote! {
				let mut __child = { #el_codes __el };
				__buildcode::end_el!(build, __el, __child, #tag);
			});
		}
		Child::DoBlock(block) => codes.extend(quote! {{
			__buildcode::start_do_block!(build, __el);
			#(#block)*;
			__buildcode::end_do_block!(build, __el);
		}}),
	}
}
