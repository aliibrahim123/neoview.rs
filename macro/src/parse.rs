use proc_macro2::{Delimiter, Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt, quote};

use crate::cursor::{Cursor, Error, Token, err, match_punct};

#[derive(Debug)]
pub struct Path {
	pub leading_colon: bool,
	pub segments: Vec<Ident>,
}
impl ToTokens for Path {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		if self.leading_colon {
			tokens.extend(quote! { :: });
		}
		for seg in &self.segments[..self.segments.len() - 1] {
			tokens.extend(quote! { #seg:: });
		}
		tokens.append(self.segments.last().unwrap().clone());
	}
}

#[derive(Debug)]
pub enum Node {
	Element(Element),
	DoBlock(TokenStream),
	Content(Vec<TokenTree>),
}

#[derive(Debug)]
pub struct Element {
	pub tag: Path,
	pub attrs: Vec<(Vec<TokenTree>, Vec<TokenTree>)>,
	pub children: Vec<Node>,
}

#[derive(Debug)]
pub struct ChunkInput {
	pub build: Vec<TokenTree>,
	pub nodes: Vec<Node>,
}

pub fn try_parse_path(cur: &mut Cursor) -> Option<Path> {
	let leading_colon = cur.try_multi_punct([':', ':']).is_some();
	let mut segments = Vec::new();
	while let Some(ident) = cur.try_ident() {
		segments.push(ident);
		if cur.try_multi_punct([':', ':']).is_none() {
			break;
		}
	}
	if segments.is_empty() {
		return None;
	}
	Some(Path { leading_colon, segments })
}

fn parse_children(cur: &mut Cursor) -> Result<Vec<Node>, Error> {
	let mut children = Vec::new();
	while !cur.is_end() {
		if cur.try_kw("do") {
			let block = cur.group(Delimiter::Brace)?;
			children.push(Node::DoBlock(block.stream()));
		} else if let Some(el) = try_parse_el(cur)? {
			children.push(Node::Element(el));
			if !match_punct!(cur.peek(), ',') {
				continue;
			}
		} else {
			let content = cur.eat_until(|token| match_punct!(token, ','));
			if content.is_empty() {
				return err!("expected an element, expression or a do block", cur.peek().span());
			}
			children.push(Node::Content(content));
		}
		cur.try_punct(',');
	}
	Ok(children)
}

fn try_parse_el(cur: &mut Cursor) -> Result<Option<Element>, Error> {
	let start = cur.ind();
	let Some(tag) = try_parse_path(cur) else {
		cur.recap(start);
		return Ok(None);
	};

	let mut has_body = false;
	let mut attrs = Vec::new();
	if let Some(mut cur) = cur.try_enter_group(Delimiter::Parenthesis) {
		has_body = true;
		while !cur.is_end() {
			let attr = cur.eat_until(|token| match_punct!(token, ',' | ':'));
			if attr.is_empty() {
				return err!("expected an attribute", cur.peek().span());
			}
			if (cur.is_end() || match_punct!(cur.peek(), ','))
				&& matches!(&attr[..], [TokenTree::Ident(_)])
			{
				attrs.push((attr.clone(), attr));
				continue;
			}
			cur.punct(':')?;

			let value = cur.eat_until(|token| match_punct!(token, ','));
			if value.is_empty() {
				return err!("expected an attribute value", cur.peek().span());
			}
			attrs.push((attr, value));

			if cur.is_end() {
				break;
			}
			cur.punct(',')?;
		}
	}

	let mut children = Vec::new();
	if let Some(mut cur) = cur.try_enter_group(Delimiter::Brace) {
		has_body = true;
		children = parse_children(&mut cur)?;
	}

	if has_body {
		Ok(Some(Element { tag, attrs, children }))
	} else {
		cur.recap(start);
		Ok(None)
	}
}

pub fn parse_chunk_input(input: TokenStream) -> Result<ChunkInput, Error> {
	let mut cur = Cursor::new(input.into(), Span::call_site());

	let chunk = cur.eat_until(|token| match_punct!(token, ','));
	if chunk.is_empty() {
		return err!("expected an expression", cur.peek().span());
	}
	cur.punct(',')?;

	let nodes = parse_children(&mut cur)?;
	if nodes.is_empty() {
		return err!("expected an element, expression or a do block", cur.peek().span());
	}

	Ok(ChunkInput { build: chunk, nodes })
}
