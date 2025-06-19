pub mod token_type;

use self::token_type::TokenType;

#[derive(Debug, Clone, PartialEq)]
/// A token produced by the tokenizer.
pub struct Token {
	/// The type of the token.
	pub token_type: TokenType,
	/// The value of the token.
	pub value: String,
	/// The position of the token in the input string.
	pub position: usize,
	/// The line number of the token in the input string.
	pub line: usize,
	/// The column number of the token in the input string.
	pub column: usize,
}

impl Token {
	/// Creates a new Token with the given type, value, position, line, and column.
	pub fn new(
		token_type: &TokenType,
		value: &String,
		position: usize,
		line: usize,
		column: usize
	) -> Token {
		Token {
			token_type: token_type.clone(),
			value: value.clone(),
			position,
			line,
			column
		}
	}
}