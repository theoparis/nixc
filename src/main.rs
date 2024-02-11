use logos::Logos;
use miette::Result;
use nixc::{lexer::Token, parser::Parser};

fn main() -> Result<()> {
	let filename = std::env::args().nth(1).expect("Expected file argument");
	let src = std::fs::read_to_string(&filename).expect("Failed to read file");

	let mut lexer = Token::lexer(src.as_str());
	let mut parser = Parser {
		file_name: &filename,
	};

	let value = parser.parse_value(&mut lexer)?;
	println!("{:#?}", value);

	Ok(())
}
