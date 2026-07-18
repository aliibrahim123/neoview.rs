//! parse `chunk` input
use proc_macro2::{
	Delimiter::{self, Brace},
	Ident, Literal, Span, TokenStream, TokenTree,
};
use quote::{ToTokens, TokenStreamExt, quote};

use crate::cursor::{Cursor, Error, Token, err, match_punct};

/// rust simple path
///
/// grammer: `"::"? list<ident, "::">`
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

/// a list of [`Child`]
pub type Children = Vec<Child>;
/// a list of [`TokenTree`]
pub type Tokens = Vec<TokenTree>;

/// item that can be exist inside the chunk ui tree
#[derive(Debug)]
pub enum Child {
	Element(Element),
	/// grammer: `_+`
	Content(Tokens),
	/// grammer:  `"do" "{" _* "}"
	DoBlock(Tokens),
	/// grammer: `"if" _+ "{" children "}" ("else" _* "{" children "}")*`
	If(Vec<IfArm>),
	/// grammer: `"for" _+ in _* "{" children "}"
	For {
		arg: Tokens,
		children: Children,
	},
	/// grammer: `"match" _+ "{" (_+ "=>" (child | "{" children "}") ","?)* "}`
	Match {
		arg: Tokens,
		arms: Vec<MatchArm>,
	},
}

/// an [`Element`] tag.
///
/// grammer: `path | str_lit`
#[derive(Debug)]
pub enum Tag {
	Path(Path),
	Lit(Literal),
}

/// a ui element
///
/// grammer: `(tag = path | str_lit) (attrs | body | attrs body)`
#[derive(Debug)]
pub struct Element {
	pub tag: Tag,
	/// grammer: `(` list<ident | _+ ":" _+, ",">? ","? `)`
	pub attrs: Vec<(Tokens, Tokens)>,
	/// grammer: `{" children? "}`
	pub children: Children,
}

/// an if chain arm
#[derive(Debug)]
pub struct IfArm {
	pub cond: Option<Tokens>,
	pub children: Children,
}

#[derive(Debug)]
pub struct MatchArm {
	pub pat: Tokens,
	pub children: Children,
}

/// `chunk` input
#[derive(Debug)]
pub struct ChunkInput {
	pub build: Tokens,
	/// the build variable identifier.
	pub build_ident: Ident,
	pub children: Children,
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

fn eat_until_comma(cur: &mut Cursor) -> Tokens {
	cur.eat_until(|token| match_punct!(token, ','))
}
fn eat_until_brace(cur: &mut Cursor) -> Tokens {
	cur.eat_until(|token| matches!(token, Token::Group(group) if group.delimiter() == Brace))
}

fn expected_child<T>(cur: &Cursor) -> Result<T, Error> {
	err!("expected an element, expression, do block or a control flow", cur.peek().span())
}

fn parse_child(cur: &mut Cursor) -> Result<Child, Error> {
	// do block
	if cur.try_kw("do") {
		let block = cur.group(Brace)?;
		Ok(Child::DoBlock(block.stream().into_iter().collect()))
	}
	// if flow
	else if cur.test_kw("if") {
		let mut arms = Vec::new();
		loop {
			let cond = if cur.try_kw("if") { Some(eat_until_brace(cur)) } else { None };
			let children = parse_children(&mut cur.enter_group(Brace)?, false)?;
			arms.push(IfArm { cond, children });

			if !cur.try_kw("else") {
				break;
			}
		}
		Ok(Child::If(arms))
	}
	// for flow
	else if cur.try_kw("for") {
		let mut arg = cur.eat_until(|token| matches!(token, Token::Ident(ident) if ident == "in"));
		arg.extend(eat_until_brace(cur));
		let children = parse_children(&mut cur.enter_group(Brace)?, false)?;
		Ok(Child::For { arg, children })
	}
	// match flow
	else if cur.try_kw("match") {
		let arg = eat_until_brace(cur);
		let mut arms = Vec::new();
		let mut arms_cur = cur.enter_group(Brace)?;
		while !arms_cur.is_end() {
			let mut pat = Vec::new();
			while arms_cur.try_multi_punct(['=', '>']).is_none() {
				if arms_cur.is_end() {
					return err!("expected `=>`", arms_cur.peek().span());
				}
				pat.push(arms_cur.peek().clone().into());
				arms_cur.skip();
			}

			let children = if let Some(mut cur) = arms_cur.try_enter_group(Brace) {
				parse_children(&mut cur, false)?
			} else {
				vec![parse_child(&mut arms_cur)?]
			};

			arms.push(MatchArm { pat, children });
			arms_cur.try_punct(',');
		}
		Ok(Child::Match { arg, arms })
	}
	// element
	else if let Some(el) = try_parse_el(cur)? {
		Ok(Child::Element(el))
	}
	// content
	else {
		let content = eat_until_comma(cur);
		if content.is_empty() {
			return expected_child(cur);
		}
		Ok(Child::Content(content))
	}
}

/// grammer:
/// ```text
/// (child_opt_comma | child_req_comma) (","? child_opt_comma | "," child_req_comma)* ","?;
/// let child_opt_comma = element | do_block | if_flow | for_flow | match_flow;
/// let child_req_comma = content;
/// ```
fn parse_children(cur: &mut Cursor, is_optional: bool) -> Result<Children, Error> {
	let mut children = Vec::new();
	while !cur.is_end() {
		children.push(parse_child(cur)?);
		cur.try_punct(',');
	}
	if !is_optional && children.is_empty() {
		return expected_child(cur);
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
			// single ident shorthand
			if (cur.is_end() || match_punct!(cur.peek(), ','))
				&& matches!(&attr[..], [TokenTree::Ident(_)])
			{
				attrs.push((attr.clone(), attr));
				cur.try_punct(',');
				continue;
			}
			cur.punct(':')?;

			let value = eat_until_comma(&mut cur);
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
		children = parse_children(&mut cur, true)?;
	}

	if has_body {
		Ok(Some(Element { tag, attrs, children }))
	} else {
		cur.recap(start);
		Ok(None)
	}
}

pub fn parse_chunk_input(input: TokenStream) -> Result<ChunkInput, Error> {
	let mut cur = Cursor::new(input, Span::call_site());

	let build = eat_until_comma(&mut cur);
	if build.is_empty() {
		return err!("expected an expression", cur.peek().span());
	}
	let build_ident = match &build[..] {
		[TokenTree::Ident(ident)] => ident.clone(),
		_ => Ident::new("__build", Span::call_site()),
	};
	cur.punct(',')?;

	let children = parse_children(&mut cur, false)?;

	Ok(ChunkInput { build, build_ident, children })
}
