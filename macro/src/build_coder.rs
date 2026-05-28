use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::{ChunkInput, Element, Node, Tag};

pub fn encode(input: ChunkInput) -> TokenStream {
	let ChunkInput { build, nodes } = input;
	let mut codes = quote! {
		let mut build = #(#build)*;
		let mut el = __buildcode::start_chunk!(build);
	};

	for node in nodes {
		encode_node(&mut codes, node);
	}

	quote! { {
		#codes
		__buildcode::end_chunk!(build, el);
	} }
}

fn encode_node(codes: &mut TokenStream, node: Node) {
	match node {
		Node::Content(content) => {
			codes.extend(quote! { __buildcode::content!(build, el, #(#content)*); })
		}
		Node::Element(Element { tag, attrs, children }) => {
			let tag = match tag {
				Tag::Path(path) => quote! { #path },
				Tag::Lit(lit) => quote! { #lit },
			};
			let mut el_codes = quote! { let mut el = __buildcode::start_el!(build, el, #tag); };

			for (attr, value) in attrs {
				el_codes.extend(quote! { __buildcode::attr!(build, el, [#(#attr)*], #(#value)*); });
			}

			for child in children {
				encode_node(&mut el_codes, child);
			}

			codes.extend(quote! {
				let mut child = { #el_codes el };
				__buildcode::end_el!(build, el, child, #tag);
			});
		}
		Node::DoBlock(block) => codes.extend(quote! {{
			__buildcode::start_do_block!(build, el);
			#block
			__buildcode::end_do_block!(build, el);
		}}),
	}
}
