use logos::Logos;
use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("lexer error")]
#[diagnostic(
	code(nixc::lexer::error),
	url(docsrs),
	help("try doing it better next time?")
)]
pub struct LexError {
	#[source_code]
	src: NamedSource<String>,

	#[label("This bit here")]
	bad_bit: SourceSpan,
}

#[derive(Logos, Debug, PartialEq)]
#[logos(subpattern decimal = r"[0-9][_0-9]*")]
#[logos(subpattern hex = r"[0-9a-fA-F][_0-9a-fA-F]*")]
#[logos(subpattern octal = r"[0-7][_0-7]*")]
#[logos(subpattern binary = r"[0-1][_0-1]*")]
#[logos(subpattern exp = r"[eE][+-]?[0-9][_0-9]*")]
pub enum Token<'a> {
	#[regex(r"[ ]", priority = 3)]
	Space,

	#[regex(r"//.*\n?", logos::skip)]
	#[regex(r"[ \t\n\f]+", logos::skip)]
	Error,

	#[regex("(?&decimal)", |lex| lex.slice().parse::<i64>().unwrap())]
	Integer(i64),

	#[regex("0[xX](?&hex)")]
	HexInteger,

	#[regex("0[oO](?&octal)")]
	OctalInteger,

	#[regex("0[bB](?&binary)")]
	BinaryInteger,

	#[regex(r#"[+-]?(((?&decimal)\.(?&decimal)?(?&exp)?[fFdD]?)|(\.(?&decimal)(?&exp)?[fFdD]?)|((?&decimal)(?&exp)[fFdD]?)|((?&decimal)(?&exp)?[fFdD]))"#, |lex| lex.slice().parse::<f64>().unwrap())]
	Float(f64),

	#[regex(r"0[xX](((?&hex))|((?&hex)\.)|((?&hex)?\.(?&hex)))[pP][+-]?(?&decimal)[fFdD]?")]
	HexFloat(&'a str),

	#[token("false", |_| false)]
	#[token("true", |_| true)]
	Bool(bool),

	#[token("{")]
	BraceOpen,

	#[token("}")]
	BraceClose,

	#[token("[")]
	BracketOpen,

	#[token("]")]
	BracketClose,

	#[token(":")]
	Colon,

	#[token(",")]
	Comma,

	#[token("null")]
	Null,

	#[token("let")]
	Let,

	#[token("in")]
	In,

	#[token("=")]
	Equals,

	#[token(";")]
	SemiColon,

	#[regex(r"(\p{XID_Start}|_)\p{XID_Continue}*")]
	Identifier(&'a str),
}
