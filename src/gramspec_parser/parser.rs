use std::collections::HashMap;
use std::error::Error;

use crate::gramspec_parser::token;
use crate::gramspec_parser::gramspec;
use token::{Token, token_type::TokenType};
use gramspec::{expression::Expression, GramSpec, gramspec_config::GramSpecConfig};

mod tokenizer;

pub struct Parser {
	tokenizer: tokenizer::Tokenizer,
	structures: Vec<Structure>,
	tokens: Vec<Token>,
	position: usize,
}

impl Parser {
	/// Creates a new Parser with the given tokens.
	pub fn new(input: String) -> Self {
		Parser {
			tokenizer: tokenizer::Tokenizer::new(input),
			structures: Vec::new(),
			tokens: Vec::new(),
			position: 0,
		}
	}

	pub fn parse(&mut self) -> Result<GramSpec, Box<dyn Error>> {
		self.tokens = self.tokenizer.tokenize()?;
		self.structures = self.structurize()?;
		let mut rules: Vec<Structure> = Vec::new();
		let mut config_directives: Vec<Structure> = Vec::new();
		let mut meta_rules: Vec<Structure> = Vec::new();

		for structure in &self.structures {
			match structure.structure_type {
				StructureType::RuleDefinition => rules.push(structure.clone()),
				StructureType::ConfigDirective => config_directives.push(structure.clone()),
				StructureType::MetaRuleDefinition => meta_rules.push(structure.clone()),
			}
		}

		let mut gramspec = GramSpec {
			rules: HashMap::new(),
			config: GramSpecConfig::new(),
			meta_rules: HashMap::new(),
		};

		// TODO: Check for rule duplicates and config directive duplicates

		for rule in &rules {
			let phrase = &rule.tokens[1..];
			let and_phrase = &self.add_implict_ands(&phrase.to_vec());
			let expression = self.to_expression(and_phrase.to_vec())?;
			let alternatives = self.split_into_alternatives(&expression);
			gramspec.rules.insert(
				rule.tokens[0].value.clone(),
				alternatives
			);
		}

		for meta_rule in &meta_rules {
			let phrase = &meta_rule.tokens[1..];
			let and_phrase = &self.add_implict_ands(&phrase.to_vec());
			let expression = self.to_expression(and_phrase.to_vec())?;
			let alternatives = self.split_into_alternatives(&expression);
			gramspec.meta_rules.insert(
				meta_rule.tokens[0].value.clone(),
				alternatives
			);
		}

		for config_directive in &mut config_directives {
			let directive_name = &config_directive.tokens[0].value;
			let directive_value = &config_directive.tokens[1].value;
			gramspec.config.set(
				directive_name.clone(),
				directive_value.clone()
			)?;
		}

		Ok(gramspec)
	}

	fn structurize(&mut self) -> Result<Vec<Structure>, Box<dyn Error>> {
		let mut structures = Vec::new();
		while self.position < self.tokens.len() {
			let mut structure = Structure::new(
				Vec::new(),
				StructureType::RuleDefinition
			);
			let initial_pos = self.position;
			let mut longest_pos = 0;

			// Try to parse a config directive
			if let Some(new_structure) = self.expect_config_directive()? {
				if self.position > longest_pos {
					structure = new_structure;
					longest_pos = self.position;
				}
			}
			// Reset position
			self.position = initial_pos;
			// Try to parse a rule definition
			if let Some(new_structure) = self.expect_rule_definition()? {
				if self.position > longest_pos {
					structure = new_structure;
					longest_pos = self.position;
				}
			}
			// Reset position
			self.position = initial_pos;
			// Try to parse a meta rule definition
			if let Some(new_structure) = self.expect_meta_rule_definition()? {
				if self.position > longest_pos {
					structure = new_structure;
					longest_pos = self.position;
				}
			}

			if longest_pos == 0 {
				let (line, column) = self.tokenizer.get_line_column(self.tokens[self.position].position);
				return Err(format!(
					"Unexpected token at position {}:{}: {:?}",
					line,
					column,
					self.tokens[self.position].token_type
				).into());
			}

			// Set position to the end of the structure
			self.position = longest_pos;

			structures.push(structure);
		}
		Ok(structures)
	}

	fn expect_config_directive(&mut self) -> Result<Option<Structure>, Box<dyn Error>> {
		let mut structure = Structure::new(
			Vec::new(),
			StructureType::ConfigDirective
		);

		// Read the config directive token, don't bother adding it to the structure
		if self.tokens[self.position].token_type == TokenType::ConfigDirective {
			self.position += 1;
		}else {
			return Ok(None);
		}

		// Read the config directive name token
		if self.tokens[self.position].token_type == TokenType::RuleName {
			structure.tokens.push(self.tokens[self.position].clone());
			self.position += 1;
		} else {
			return Err(format!(
				"Expected config directive name at position {}, found {:?}",
				self.tokens[self.position].position,
				self.tokens[self.position].token_type
			).into());
		}

		// Read open paren token, don't bother adding it to the structure
		if self.tokens[self.position].token_type == TokenType::OpenParen {
			self.position += 1;
		} else {
			return Err(format!(
				"Expected '(' at position {}, found {:?}",
				self.tokens[self.position].position,
				self.tokens[self.position].token_type
			).into());
		}

		// Read the config directive value token
		if self.tokens[self.position].token_type == TokenType::StringLiteral {
			structure.tokens.push(self.tokens[self.position].clone());
			self.position += 1;
		} else {
			return Err(format!(
				"Expected config directive value at position {}, found {:?}",
				self.tokens[self.position].position,
				self.tokens[self.position].token_type
			).into());
		}

		// Read close paren token, don't bother adding it to the structure
		if self.tokens[self.position].token_type == TokenType::CloseParen {
			self.position += 1;
		} else {
			return Err(format!(
				"Expected ')' at position {}, found {:?}",
				self.tokens[self.position].position,
				self.tokens[self.position].token_type
			).into());
		}

		// Read endline token, don't bother adding it to the structure
		if self.tokens[self.position].token_type == TokenType::Newline {
			self.position += 1;
		} else {
			return Err(format!(
				"Expected endline at position {}, found {:?}",
				self.tokens[self.position].position,
				self.tokens[self.position].token_type
			).into());
		}

		Ok(Some(structure))
	}

	fn expect_rule_definition(&mut self) -> Result<Option<Structure>, Box<dyn Error>> {
		let mut structure = Structure::new(
			Vec::new(),
			StructureType::RuleDefinition
		);

		// Read the rule name token
		if self.tokens[self.position].token_type == TokenType::RuleName {
			structure.tokens.push(self.tokens[self.position].clone());
			self.position += 1;
		} else {
			return Ok(None);
		}

		// Read the rule definition token, don't bother adding it to the structure
		if self.tokens[self.position].token_type == TokenType::RuleDefinition {
			self.position += 1;
		} else {
			return Err(format!(
				"Expected ':' at position {}, found {:?}",
				self.tokens[self.position].position,
				self.tokens[self.position].token_type
			).into());
		}

		// Read tokens until we reach a newline or end of input
		while self.position < self.tokens.len() {
			if self.tokens[self.position].token_type == TokenType::Newline {
				self.position += 1;
				break;
			}
			structure.tokens.push(self.tokens[self.position].clone());
			self.position += 1;
		}

		Ok(Some(structure))
	}

	fn expect_meta_rule_definition(&mut self) -> Result<Option<Structure>, Box<dyn Error>> {
		let mut structure = Structure::new(
			Vec::new(),
			StructureType::MetaRuleDefinition
		);

		// Read the meta rule token, don't bother adding it to the structure
		if self.tokens[self.position].token_type == TokenType::MetaRule {
			self.position += 1;
		} else {
			return Ok(None);
		}

		// Read the meta rule name token
		if self.tokens[self.position].token_type == TokenType::RuleName {
			structure.tokens.push(self.tokens[self.position].clone());
			self.position += 1;
		} else {
			return Err(format!(
				"Expected meta rule name at position {}, found {:?}",
				self.tokens[self.position].position,
				self.tokens[self.position].token_type
			).into());
		}

		// Read the rule definition token, don't bother adding it to the structure
		if self.tokens[self.position].token_type == TokenType::RuleDefinition {
			self.position += 1;
		} else {
			return Err(format!(
				"Expected ':' at position {}, found {:?}",
				self.tokens[self.position].position,
				self.tokens[self.position].token_type
			).into());
		}

		// Read tokens until we reach a newline or end of input
		while self.position < self.tokens.len() {
			if self.tokens[self.position].token_type == TokenType::Newline {
				self.position += 1;
				break;
			}
			structure.tokens.push(self.tokens[self.position].clone());
			self.position += 1;
		}

		Ok(Some(structure))
	}

	fn add_implict_ands(&self, tokens: &Vec<Token>) -> Vec<Token> {
		let mut final_ = Vec::new();
		for i in 0..(tokens.len() - 1) {
			let token = &tokens[i];
			let next_token = &tokens[i + 1];

			final_.push(token.clone());

			if token.token_type == TokenType::OpenParen {
				continue;
			}
			if token.token_type == TokenType::DelimitRepeat {
				continue;
			}
			if token.token_type == TokenType::Or {
				continue;
			}

			if next_token.token_type == TokenType::CloseParen {
				continue;
			}
			if next_token.token_type.is_operator() {
				continue;
			}

			let (line, column) = self.tokenizer.get_line_column(token.position);

			final_.push(Token {
				value: String::from("&"),
				token_type: TokenType::And,
				position: token.position + 1,
				line,
				column: column + 1,
			});
		}

		final_.push(tokens[tokens.len() - 1].clone());

		final_
	}

	fn to_expression(&self, tokens: Vec<Token>) -> Result<Expression, Box<dyn Error>> {

		// Conversion from infix to postfix notation

		let mut postfix: Vec<Token> = Vec::new();
		let mut stack: Vec<Token> = Vec::new();

		for token in tokens {
			if token.token_type.is_unary_operator() {
				if
					!stack.is_empty()
					&& stack.last().unwrap().token_type == TokenType::DelimitRepeat
					&& (token.token_type == TokenType::RepeatOne || token.token_type == TokenType::RepeatZero)
				{
					postfix.push(stack.pop().unwrap());
				}
				postfix.push(token);
			} else if token.token_type.is_operator() {
				if stack.is_empty() {
					stack.push(token);
				} else {
					while let Some(top) = stack.last() {
						if top.token_type.is_operator() && top.token_type.get_precedence() >= token.token_type.get_precedence() {
							postfix.push(stack.pop().unwrap());
						} else {
							break;
						}
					}
					stack.push(token);
				}
			} else if token.token_type == TokenType::OpenParen {
				stack.push(token);
			} else if token.token_type == TokenType::CloseParen {
				while let Some(top) = stack.pop() {
					if top.token_type == TokenType::OpenParen {
						break;
					}
					postfix.push(top);
				}
			} else {
				postfix.push(token);
			}
		}

		while stack.len() > 0 {
			let top = stack.pop().unwrap();
			if top.token_type == TokenType::OpenParen {
				return Err(format!(
					"Unmatched '(' at position {}",
					top.position
				).into());
			}
			postfix.push(top);
		}

		// Conversion from postfix to expression

		if postfix.len() == 1 {
			let token = postfix[0].clone();
			match token.token_type {
				TokenType::RuleName => return Ok(Expression::RuleName(token)),
				TokenType::RegexLiteral => return Ok(Expression::RegexLiteral(token)),
				TokenType::StringLiteral => return Ok(Expression::StringLiteral(token)),
				TokenType::Keyword => return Ok(Expression::Keyword(token)),
				_ => {}
			}
		}


		let mut operands: Vec<Expression> = Vec::new();

		let mut i = 0;
		while i < postfix.len() {
			let token = &postfix[i];
			let expression = match token.token_type {
				// Literal/Identifier tokens
				TokenType::RuleName => Expression::RuleName(token.clone()),
				TokenType::RegexLiteral => Expression::RegexLiteral(token.clone()),
				TokenType::StringLiteral => Expression::StringLiteral(token.clone()),
				TokenType::Keyword => Expression::Keyword(token.clone()),

				// Unary operators
				TokenType::RepeatOne => {
					Expression::RepeatOne(Box::new(operands.pop().unwrap()))
				},
				TokenType::RepeatZero => {
					Expression::RepeatZero(Box::new(operands.pop().unwrap()))
				},
				TokenType::Optional => {
					Expression::Optional(Box::new(operands.pop().unwrap()))
				}

				// Binary operators
				TokenType::And => {
					let right = operands.pop().unwrap();
					let left = operands.pop().unwrap();
					Expression::And(Box::new(left), Box::new(right))
				},
				TokenType::Or => {
					let right = operands.pop().unwrap();
					let left = operands.pop().unwrap();
					Expression::Or(Box::new(left), Box::new(right))
				},
				TokenType::DelimitRepeat => {
					if postfix[i + 1].token_type == TokenType::RepeatOne {
						i += 1; // Skip the RepeatOne token
						let right = operands.pop().unwrap();
						let left = operands.pop().unwrap();
						Expression::DelimitRepeatOne(Box::new(left), Box::new(right))
					} else if postfix[i + 1].token_type == TokenType::RepeatZero {
						i += 1; // Skip the RepeatZero token
						let right = operands.pop().unwrap();
						let left = operands.pop().unwrap();
						Expression::DelimitRepeatZero(Box::new(left), Box::new(right))
					} else {
						return Err(format!(
							"Expected RepeatOne or RepeatZero after DelimitRepeat at position {}",
							token.position
						).into());
					}
				}

				_ => {
					return Err(format!(
						"Unexpected token {:?} at position {}",
						token.token_type,
						token.position
					).into());
				}
			};

			operands.push(expression);

			i += 1;
		}

		Ok(operands.pop().unwrap())
	}

	fn split_into_alternatives(&self, expression: &Expression) -> Vec<Expression> {
		match expression {
			Expression::Or(left, right) => {
				let mut alternatives = self.split_into_alternatives(left);
				alternatives.extend(self.split_into_alternatives(right));
				alternatives
			},
			_ => vec![expression.clone()],
		}
	}

}

#[derive(Debug, Clone)]
pub enum StructureType {
	ConfigDirective,
	RuleDefinition,
	MetaRuleDefinition,
}

#[derive(Debug, Clone)]
pub struct Structure {
	tokens: Vec<Token>,
	structure_type: StructureType,
}

impl Structure {
	pub fn new(tokens: Vec<Token>, structure_type: StructureType) -> Self {
		Structure { tokens, structure_type }
	}
}

