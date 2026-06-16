use proc_macro2::{
	Delimiter::{self, Brace},
	Ident, Literal, Span, TokenStream, TokenTree,
};
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
pub enum Child {
	Element(Element),
	DoBlock(Vec<TokenTree>),
	Content(Vec<TokenTree>),
}

#[derive(Debug)]
pub enum Tag {
	Path(Path),
	Lit(Literal),
}

#[derive(Debug)]
pub struct Element {
	pub tag: Tag,
	pub attrs: Vec<(Vec<TokenTree>, Vec<TokenTree>)>,
	pub children: Vec<Child>,
}

#[derive(Debug)]
pub struct ChunkInput {
	pub build: Vec<TokenTree>,
	pub children: Vec<Child>,
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

fn parse_children(cur: &mut Cursor) -> Result<Vec<Child>, Error> {
	let mut children = Vec::new();
	while !cur.is_end() {
		if cur.try_kw("do") {
			let block = cur.group(Brace)?;
			children.push(Child::DoBlock(block.stream().into_iter().collect()));
		} else if cur.test_kw("if") {
			let mut block = Vec::new();
			loop {
				block.extend(cur.eat_until(
					|token| matches!(token, Token::Group(group) if group.delimiter() == Brace),
				));
				block.push(cur.group(Brace)?.into());
				if !cur.test_kw("else") {
					break;
				}
			}
			children.push(Child::DoBlock(block))
		} else if cur.test_kw("for") | cur.test_kw("match") {
			let mut block = cur.eat_until(
				|token| matches!(token, Token::Group(group) if group.delimiter() == Brace),
			);
			block.push(cur.group(Brace)?.into());
			children.push(Child::DoBlock(block))
		} else if let Some(el) = try_parse_el(cur)? {
			children.push(Child::Element(el));
			if !match_punct!(cur.peek(), ',') {
				continue;
			}
		} else {
			let content = cur.eat_until(|token| match_punct!(token, ','));
			if content.is_empty() {
				return err!("expected an element, expression or a do block", cur.peek().span());
			}
			children.push(Child::Content(content));
		}
		cur.try_punct(',');
	}
	Ok(children)
}

fn try_parse_el(cur: &mut Cursor) -> Result<Option<Element>, Error> {
	let start = cur.ind();
	let tag = if let Some(path) = try_parse_path(cur) {
		Tag::Path(path)
	} else if let Some(lit) = cur.try_literal() {
		Tag::Lit(lit)
	} else {
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

	Ok(ChunkInput { build: chunk, children: nodes })
}
