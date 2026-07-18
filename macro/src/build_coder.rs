//! transform [`ChunkInput`] to [`TokenStream`]

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::parse::{Child, Children, ChunkInput, Element, IfArm, MatchArm, Tag};

/// transform [`ChunkInput`] to [`TokenStream`]
pub fn encode(input: ChunkInput) -> TokenStream {
	let ChunkInput { build, build_ident, children } = input;
	let mut codes = quote! {
		let mut #build_ident = &mut #(#build)*;
		let mut __el = __buildcode::start_chunk!(#build_ident);
	};

	encode_children(&mut codes, children, &build_ident);

	quote! { {
		#codes
		__buildcode::end_chunk!(#build_ident, __el);
	} }
}

fn encode_children(codes: &mut TokenStream, children: Children, build_ident: &Ident) {
	for child in children {
		encode_child(codes, child, build_ident);
	}
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

			encode_children(&mut el_codes, children, build_ident);

			codes.extend(quote! {
				let mut __child = { #el_codes __el };
				__buildcode::end_el!(#build_ident, __el, __child, #tag);
			});
		}
		Child::DoBlock(block) => codes.extend(quote! {{
			__buildcode::start_op!(#build_ident, do, __el);
			#(#block)*;
			__buildcode::end_op!(#build_ident, do, __el);
		}}),
		Child::If(arms) => {
			for (ind, IfArm { cond, children }) in arms.into_iter().enumerate() {
				let prefix = if ind != 0 { Some(format_ident!("else")) } else { None };
				let op = format_ident!("{}", if cond.is_none() { "else" } else { "if" });
				let if_cond = cond.map(|cond| quote! { if #(#cond)* });

				let mut child_codes = quote! {};
				encode_children(&mut child_codes, children, build_ident);

				codes.extend(quote! {
					#prefix #if_cond {
						__buildcode::start_op!(#build_ident, #op, __el);
						#child_codes
						__buildcode::end_op!(#build_ident, #op, __el);
					}
				});
			}
		}
		Child::For { arg, children } => {
			let mut child_codes = quote! {};
			encode_children(&mut child_codes, children, build_ident);
			codes.extend(quote! {
				for #(#arg)* {
					__buildcode::start_op!(#build_ident, for, __el);
					#child_codes
					__buildcode::end_op!(#build_ident, for, __el);
				}
			});
		}
		Child::Match { arg, arms } => {
			let mut arms_codes = quote! {};
			for MatchArm { pat, children } in arms {
				let mut child_codes = quote! {};
				encode_children(&mut child_codes, children, build_ident);
				arms_codes.extend(quote! {
					#(#pat)* => {
						__buildcode::start_op!(#build_ident, match, __el);
						#child_codes;
						__buildcode::end_op!(#build_ident, match, __el);
					}
				})
			}

			codes.extend(quote! {
				match #(#arg)* { #arms_codes }
			})
		}
	}
}
