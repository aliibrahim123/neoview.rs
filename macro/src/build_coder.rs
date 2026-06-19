//! transform [`ChunkInput`] to [`TokenStream`]

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::parse::{Child, ChunkInput, Element, Tag};

/// transform [`ChunkInput`] to [`TokenStream`]
pub fn encode(input: ChunkInput) -> TokenStream {
	let ChunkInput { build, build_ident, children } = input;
	let mut codes = quote! {
		let mut #build_ident = &mut #(#build)*;
		let mut __el = __buildcode::start_chunk!(#build_ident);
	};

	for child in children {
		encode_child(&mut codes, child, &build_ident);
	}

	quote! { {
		#codes
		__buildcode::end_chunk!(#build_ident, __el);
	} }
}

fn encode_child(codes: &mut TokenStream, child: Child, build_ident: &Ident) {
	match child {
		Child::Content(content) => {
			codes.extend(quote! { __buildcode::content!(#build_ident, __el, #(#content)*); })
		}
		Child::Element(Element { tag, attrs, children }) => {
			let tag = match tag {
				Tag::Path(path) => quote! { #path },
				Tag::Lit(lit) => quote! { #lit },
			};
			let mut el_codes =
				quote! { let mut __el = __buildcode::start_el!(#build_ident, __el, #tag); };

			for (attr, value) in attrs {
				el_codes.extend(
					quote! { __buildcode::attr!(#build_ident, __el, [#(#attr)*], #(#value)*); },
				);
			}

			for child in children {
				encode_child(&mut el_codes, child, build_ident);
			}

			codes.extend(quote! {
				let mut __child = { #el_codes __el };
				__buildcode::end_el!(#build_ident, __el, __child, #tag);
			});
		}
		Child::DoBlock(block) => codes.extend(quote! {{
			__buildcode::start_do_block!(#build_ident, __el);
			#(#block)*;
			__buildcode::end_do_block!(#build_ident, __el);
		}}),
	}
}
