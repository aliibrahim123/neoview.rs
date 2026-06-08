use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote_spanned};

#[derive(Debug)]
pub enum Token {
	Group(Group),
	Ident(Ident),
	Punct(char, Span, Spacing),
	Literal(Literal),
	End(Span),
}
impl Token {
	pub fn span(&self) -> Span {
		match self {
			Token::Group(group) => group.span(),
			Token::Ident(ident) => ident.span(),
			Token::Punct(_, span, _) => *span,
			Token::Literal(lit) => lit.span(),
			Token::End(span) => *span,
		}
	}
}
impl From<TokenTree> for Token {
	fn from(tt: TokenTree) -> Self {
		match tt {
			TokenTree::Group(g) => Token::Group(g),
			TokenTree::Ident(i) => Token::Ident(i),
			TokenTree::Punct(p) => Token::Punct(p.as_char(), p.span(), p.spacing()),
			TokenTree::Literal(l) => Token::Literal(l),
		}
	}
}
impl ToTokens for Token {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Token::Group(group) => group.to_tokens(tokens),
			Token::Ident(ident) => ident.to_tokens(tokens),
			Token::Punct(char, span, spacing) => {
				pack_punct(*char, *span, *spacing).to_tokens(tokens)
			}
			Token::Literal(lit) => lit.to_tokens(tokens),
			Token::End(_) => (),
		}
	}
}
fn pack_punct(char: char, span: Span, spacing: Spacing) -> Punct {
	let mut punct = Punct::new(char, spacing);
	punct.set_span(span);
	punct
}
impl Into<TokenTree> for Token {
	fn into(self) -> TokenTree {
		match self {
			Self::Ident(ident) => TokenTree::Ident(ident),
			Self::Literal(lit) => TokenTree::Literal(lit),
			Self::Punct(char, span, spacing) => {
				let mut punc = Punct::new(char, spacing);
				punc.set_span(span);
				TokenTree::Punct(punc)
			}
			Self::Group(group) => TokenTree::Group(group),
			_ => panic!(),
		}
	}
}

macro_rules! match_punct {
	($tok:expr, $pat:pat) => {
		matches!($tok, $crate::cursor::Token::Punct($pat, _, _))
	};
}
pub(crate) use match_punct;

#[derive(Debug)]
pub struct Cursor {
	pub tokens: Box<[Token]>,
	pub ind: usize,
}
impl Cursor {
	pub fn new(stream: TokenStream, end_span: Span) -> Self {
		let mut tokens = stream.into_iter().map(|tok| tok.into()).collect::<Vec<_>>();
		tokens.push(Token::End(end_span));
		Self { tokens: tokens.into_boxed_slice(), ind: 0 }
	}
	pub fn peek(&self) -> &Token {
		&self.tokens[self.ind]
	}
	pub fn peek_next(&self, n: usize) -> &Token {
		&self.tokens[self.ind + n]
	}
	pub fn skip(&mut self) {
		self.ind += 1;
	}
	pub fn prev(&self) -> &Token {
		&self.tokens[self.ind - 1]
	}
	pub fn is_end(&self) -> bool {
		self.ind >= self.tokens.len() - 1
	}
	pub fn ind(&self) -> usize {
		self.ind
	}
	pub fn recap(&mut self, ind: usize) {
		self.ind = ind
	}
	pub fn punct(&mut self, char: char) -> Result<Span, Error> {
		if self.try_punct(char) {
			Ok(self.prev().span())
		} else {
			err!("expected `{char}`", self.peek().span())
		}
	}
	pub fn try_punct(&mut self, char: char) -> bool {
		if let Token::Punct(ch, _, _) = self.peek()
			&& *ch == char
		{
			self.skip();
			return true;
		}
		false
	}
	pub fn multi_punct<const N: usize>(&mut self, chars: [char; N]) -> Result<[Span; N], Error> {
		if let Some(spans) = self.try_multi_punct(chars) {
			Ok(spans)
		} else {
			let chars = chars.iter().collect::<String>();
			err!("expected `{chars}`", self.peek().span())
		}
	}
	pub fn try_multi_punct<const N: usize>(&mut self, chars: [char; N]) -> Option<[Span; N]> {
		let mut spans = [Span::call_site(); N];
		for i in 0..N - 1 {
			self.peek_next(i);
			if let Token::Punct(char, span, Spacing::Joint) = self.peek_next(i)
				&& *char == chars[i]
			{
				spans[i] = *span;
				continue;
			}
			return None;
		}
		if let Token::Punct(char, span, _) = self.peek_next(N - 1)
			&& *char == chars[N - 1]
		{
			spans[N - 1] = *span;
			self.ind += N;
			Some(spans)
		} else {
			None
		}
	}
	pub fn ident(&mut self) -> Result<Ident, Error> {
		if let Some(ident) = self.try_ident() {
			Ok(ident)
		} else {
			err!("expected an identifier", self.peek().span())
		}
	}
	pub fn try_ident(&mut self) -> Option<Ident> {
		let Token::Ident(ident) = self.peek() else { return None };
		let ident = ident.clone();
		self.skip();
		Some(ident)
	}
	pub fn kw(&mut self, kw: &str) -> Result<Span, Error> {
		if self.try_kw(kw) {
			Ok(self.prev().span())
		} else {
			err!("expected `{kw}`", self.peek().span())
		}
	}
	pub fn try_kw(&mut self, kw: &str) -> bool {
		let Token::Ident(ident) = self.peek() else { return false };
		if ident == kw {
			self.skip();
			return true;
		}
		false
	}
	pub fn test_kw(&mut self, kw: &str) -> bool {
		let Token::Ident(ident) = self.peek() else { return false };
		ident == kw
	}
	pub fn literal(&mut self) -> Result<Literal, Error> {
		if let Some(lit) = self.try_literal() {
			Ok(lit)
		} else {
			err!("expected a literal", self.peek().span())
		}
	}
	pub fn try_literal(&mut self) -> Option<Literal> {
		let Token::Literal(lit) = self.peek() else { return None };
		let lit = lit.clone();
		self.skip();
		Some(lit)
	}
	pub fn group(&mut self, delim: Delimiter) -> Result<Group, Error> {
		if let Some(group) = self.try_group(delim) {
			Ok(group)
		} else {
			let bracket = match delim {
				Delimiter::Parenthesis => "(",
				Delimiter::Brace => "{",
				Delimiter::Bracket => "[",
				Delimiter::None => panic!(),
			};
			err!("expected `{bracket}`", self.peek().span())
		}
	}
	pub fn try_group(&mut self, delim: Delimiter) -> Option<Group> {
		let Token::Group(group) = self.peek() else { return None };
		if group.delimiter() == delim {
			let group = group.clone();
			self.skip();
			Some(group)
		} else {
			None
		}
	}
	pub fn enter_group(&mut self, delim: Delimiter) -> Result<Cursor, Error> {
		let group = self.group(delim)?;
		Ok(Cursor::new(group.stream(), group.span_close()))
	}
	pub fn try_enter_group(&mut self, delim: Delimiter) -> Option<Cursor> {
		let group = self.try_group(delim)?;
		Some(Cursor::new(group.stream(), group.span_close()))
	}
	pub fn eat_until(&mut self, pred: impl Fn(&Token) -> bool) -> Vec<TokenTree> {
		let mut tokens = Vec::new();
		while !(self.is_end() || pred(self.peek())) {
			tokens.push(match self.peek() {
				Token::Group(group) => TokenTree::Group(group.clone()),
				Token::Ident(ident) => TokenTree::Ident(ident.clone()),
				Token::Punct(char, span, spacing) => {
					TokenTree::Punct(pack_punct(*char, *span, *spacing))
				}
				Token::Literal(lit) => TokenTree::Literal(lit.clone()),
				_ => unreachable!(),
			});
			self.ind += 1;
		}
		tokens
	}
}

#[derive(Debug, Clone)]
pub struct Error {
	msg: String,
	span: Span,
}
impl Error {
	pub fn new(msg: String, span: Span) -> Self {
		Self { msg, span }
	}
}

impl ToTokens for Error {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let Self { msg, span } = self;
		tokens.extend(quote_spanned! { *span => ::core::compile_error!(#msg); });
	}
}

macro_rules! err {
	($msg:literal, $span:expr) => {
		Err(crate::cursor::Error::new(format!($msg), $span))
	};
}
pub(crate) use err;
