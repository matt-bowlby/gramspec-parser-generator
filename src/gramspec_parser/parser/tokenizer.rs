use super::token::token_type::TokenType;
use super::token::Token;

/// A tokenizer for the grammar specification.
pub struct Tokenizer {
	input: String,
	position: usize
}

impl Tokenizer {

	/// Creates a new Tokenizer with the given input string.
	pub fn new(input: String) -> Self {
		Tokenizer { input, position: 0 }
	}

	/// Tokenizes the input string and returns a vector of tokens.
	pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
		// Initialize a vector to hold the tokens
		let mut tokens = Vec::new();
		// Loop until we reach the end of the input string
		while let Some(token) = self.next_token()? {
			// Skip whitespace and comment tokens
			if token.token_type == TokenType::Whitespace || token.token_type == TokenType::Comment {
				// If the next token after this is a newline, skip this token
				if let Some(next_token) = self.peek_token()? {
					// If the next token is a newline, skip this token
					if next_token.token_type == TokenType::Newline {
						self.next_token()?; // Consume the Newline token

					}
				}
				// Continue to the next iteration of the loop
				continue;
			}
			// Add the token to the vector
			tokens.push(token);
		}
		// Return the vector of tokens
		Ok(tokens)
	}

	/// Gets the current line and column based on the current position.
	pub fn get_line_column(&self, position: usize) -> (usize, usize) {
		let mut line = 1;
		let mut column = 1;
		for (i, c) in self.input.chars().enumerate() {
			if i >= position {
				break;
			}
			if c == '\n' {
				line += 1;
				column = 1;
			} else {
				column += 1;
			}
		}
		(line, column)
	}

	/// Peeks at the next token without consuming it.
	fn peek_token(&self) -> Result<Option<Token>, String> {
		// Make sure we don't go out of bounds
		if self.position >= self.input.len() {
			return Ok(None);
		}

		// Get slize of input from current position
		let input_slice = &self.input[self.position..];

		// Initialize variables to track the longest match
		let mut longest_length = 0;
		let mut best_match: Option<(TokenType, String)> = None;

		// Iterate over all token types and find the longest match
		for token_type in TokenType::all() {
			// Get the regex match for the current token type
			let regex = token_type.get_regex();
			// If there is a match, check if it's the longest one
			if let Some(result) = regex.captures(input_slice) {
				let length = result[0].chars().count();
				// If it is the longest match, update the variables
				if length > longest_length {
					longest_length = length;
					best_match = Some((token_type, String::from(&result[0])));
				}
			}
		}

		// Get the current line and column
		let (line, column) = self.get_line_column(self.position);

		// If no match was found, return an error
		if longest_length == 0 {
			let next_char = self.input[self.position..].chars().next().unwrap();
			return Err(format!("Unexpected character '{}' at line {}, column {}", next_char, line, column));
		}

		// If a match was found, create a new token and update the position
		if let Some((ltype, lmatch)) = best_match {
			// Create the token
			let token = Token::new(
				&ltype,
				&ltype.transform(&lmatch),
				self.position,
				line,
				column
			);
			// Return the token
			return Ok(Some(token));
		}

		// If we reach here, something went wrong
		Ok(None)
	}

	/// Gets the next token from the current position in the input string while consuming it.
	fn next_token(&mut self) -> Result<Option<Token>, String> {

		// Make sure we don't go out of bounds
		if self.position >= self.input.len() {
			return Ok(None);
		}

		// Get slize of input from current position
		let input_slice = &self.input[self.position..];

		// Initialize variables to track the longest match
		let mut longest_length = 0;
		let mut best_match: Option<(TokenType, String)> = None;

		// Iterate over all token types and find the longest match
		for token_type in TokenType::all() {
			// Get the regex match for the current token type
			let regex = token_type.get_regex();
			// If there is a match, check if it's the longest one
			if let Some(result) = regex.captures(input_slice) {
				let length = result[0].chars().count();
				// If it is the longest match, update the variables
				if length > longest_length {
					longest_length = length;
					best_match = Some((token_type, String::from(&result[0])));
				}
			}
		}

		// Get the current line and column
		let (line, column) = self.get_line_column(self.position);

		// If no match was found, return an error
		if longest_length == 0 {
			let next_char = self.input[self.position..].chars().next().unwrap();
			return Err(format!("Unexpected character '{}' at line {}, column {}", next_char, line, column));
		}

		// If a match was found, create a new token and update the position
		if let Some((ltype, lmatch)) = best_match {
			// Length of the match
			let match_length = lmatch.chars().count();
			// Create the token
			let token = Token::new(
				&ltype,
				&ltype.transform(&lmatch),
				self.position,
				line,
				column
			);
			// Update the position
			self.position += match_length;
			// Return the token
			return Ok(Some(token));
		}

		// If we reach here, something went wrong
		Ok(None)
	}

}