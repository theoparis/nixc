use crate::lexer::Token;
use logos::Lexer;
use miette::{diagnostic, Diagnostic, NamedSource, Result, SourceSpan};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("parse error: {message}")]
#[diagnostic(
	code(nixc::parser::error),
	url(docsrs),
	help("try doing it better next time?")
)]
pub struct ParseError {
	// The Source that we're gonna be printing snippets out of.
	// This can be a String if you don't have or care about file names.
	#[source_code]
	src: NamedSource<String>,
	// Snippets and highlights can be included in the diagnostic!
	#[label("This right here")]
	bad_bit: SourceSpan,

	message: String,
}

/// Represent any valid JSON value.
#[derive(Debug, Clone)]
pub enum Value<'source> {
	Null,
	Bool(bool),
	Integer(i64),
	Float(f64),
	String(&'source str),
	List(Vec<Value<'source>>),
	AttrSet(HashMap<&'source str, Value<'source>>),
	LetIn(HashMap<&'source str, Value<'source>>, Box<Value<'source>>),
}

pub struct Parser<'a> {
	pub file_name: &'a str,
}

impl<'a> Parser<'a> {
	pub fn parse_value<'source>(
		&mut self,
		lexer: &mut Lexer<'source, Token<'source>>,
	) -> Result<Value<'source>> {
		let span = lexer.span();

		if let Some(token) = lexer.next() {
			match token {
				Ok(Token::Bool(b)) => Ok(Value::Bool(b)),
				Ok(Token::BraceOpen) => self.parse_attrset(lexer),
				Ok(Token::BracketOpen) => self.parse_list(lexer),
				Ok(Token::Null) => Ok(Value::Null),
				Ok(Token::Float(n)) => Ok(Value::Float(n)),
				Ok(Token::Integer(s)) => Ok(Value::Integer(s)),
				_ => Err(ParseError {
					src: NamedSource::new(
						self.file_name,
						lexer.source().to_string(),
					),
					bad_bit: (span.start, span.end).into(),
					message: "unexpected token (context: value)".to_owned(),
				})?,
			}
		} else {
			Err(ParseError {
				src: NamedSource::new(
					self.file_name,
					lexer.source().to_string(),
				),
				bad_bit: (span.start, span.end).into(),
				message: "empty values are not allowed".to_owned(),
			})?
		}
	}

	pub fn parse_list<'source>(
		&mut self,
		lexer: &mut Lexer<'source, Token<'source>>,
	) -> Result<Value<'source>> {
		let mut array = Vec::new();
		let span = lexer.span();
		let mut awaits_space = false;
		let mut awaits_value = false;

		while let Some(token) = lexer.next() {
			match token {
				Ok(Token::Bool(b)) if !awaits_space => {
					array.push(Value::Bool(b));
					awaits_value = false;
				}
				Ok(Token::BraceOpen) if !awaits_space => {
					let object = self.parse_attrset(lexer)?;
					array.push(object);
					awaits_value = false;
				}
				Ok(Token::BracketOpen) if !awaits_space => {
					let sub_array = self.parse_list(lexer)?;
					array.push(sub_array);
					awaits_value = false;
				}
				Ok(Token::BracketClose) if !awaits_value => {
					return Ok(Value::List(array))
				}
				Ok(Token::Space) if awaits_space => awaits_value = true,
				Ok(Token::Null) if !awaits_space => {
					array.push(Value::Null);
					awaits_value = false
				}
				Ok(Token::Float(n)) if !awaits_space => {
					array.push(Value::Float(n));
					awaits_value = false;
				}
				Ok(Token::Integer(s)) if !awaits_space => {
					array.push(Value::Integer(s));
					awaits_value = false;
				}
				_ => Err(ParseError {
					src: NamedSource::new(
						self.file_name,
						lexer.source().to_string(),
					),
					bad_bit: (span.start, span.end).into(),
					message: "unexpected token (context: list)".to_owned(),
				})?,
			}
			awaits_space = !awaits_value;
		}

		Err(ParseError {
			src: NamedSource::new(self.file_name, lexer.source().to_string()),
			bad_bit: (span.start, span.end).into(),
			message: "unmatched opening bracket defined (context: list)"
				.to_owned(),
		})?
	}

	pub fn parse_attrset<'source>(
		&mut self,
		lexer: &mut Lexer<'source, Token<'source>>,
	) -> Result<Value<'source>> {
		let mut map = HashMap::new();
		let span = lexer.span();
		let mut awaits_comma = false;
		let mut awaits_key = false;

		while let Some(token) = lexer.next() {
			match token {
				Ok(Token::BraceClose) if !awaits_key => {
					return Ok(Value::AttrSet(map))
				}
				Ok(Token::Comma) if awaits_comma => awaits_key = true,
				Ok(Token::Identifier(key)) if !awaits_comma => {
					match lexer.next() {
						Some(Ok(Token::Equals)) => (),
						_ => Err(ParseError {
							src: NamedSource::new(
								self.file_name,
								lexer.source().to_string(),
							),
							bad_bit: (span.start, span.end).into(),
							message: "expected '=' (context: attrset)"
								.to_owned(),
						})?,
					}
					let value = self.parse_value(lexer)?;
					map.insert(key, value);
					awaits_key = false;
				}
				_ => {
					return Err(ParseError {
						src: NamedSource::new(
							self.file_name,
							lexer.source().to_string(),
						),
						bad_bit: (span.start, span.end).into(),
						message: "expected '=' (context: attrset)".to_owned(),
					})?;
				}
			}
			awaits_comma = !awaits_key;
		}

		Err(ParseError {
			src: NamedSource::new(self.file_name, lexer.source().to_string()),
			bad_bit: (span.start, span.end).into(),
			message: "unmatched opening brace (context: attrset)".to_owned(),
		})?
	}
}
