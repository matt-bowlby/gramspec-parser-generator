use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RULE_NAME_REGEX: Regex = Regex::new(r"^[a-z_][a-z0-9_]*").unwrap();
    static ref KEYWORD_REGEX: Regex = Regex::new(r"^[A-Z_][A-Z0-9_]*").unwrap();
    static ref REGEX_LITERAL_REGEX: Regex = Regex::new(r#"^"[^"\r\n]*""#).unwrap();
    static ref STRING_LITERAL_REGEX: Regex = Regex::new(r"^'[^'\r\n]*'").unwrap();
    static ref OR_REGEX: Regex = Regex::new(r"^\|").unwrap();
    static ref AND_REGEX: Regex = Regex::new(r"^&").unwrap();
    static ref DELIMIT_REPEAT_REGEX: Regex = Regex::new(r"^,").unwrap();
    static ref REPEAT_ONE_REGEX: Regex = Regex::new(r"^\+").unwrap();
    static ref REPEAT_ZERO_REGEX: Regex = Regex::new(r"^\*").unwrap();
    static ref OPTIONAL_REGEX: Regex = Regex::new(r"^\?").unwrap();
    static ref OPEN_PAREN_REGEX: Regex = Regex::new(r"^\(").unwrap();
    static ref CLOSE_PAREN_REGEX: Regex = Regex::new(r"^\)").unwrap();
    static ref RULE_DEF_REGEX: Regex = Regex::new(r"^:").unwrap();
    static ref CONFIG_DIRECTIVE_REGEX: Regex = Regex::new(r"^@").unwrap();
    static ref META_RULE_REGEX: Regex = Regex::new(r"^\$").unwrap();
    static ref COMMENT_REGEX: Regex = Regex::new(r"^#[^\r\n]*").unwrap();
    static ref WHITESPACE_REGEX: Regex = Regex::new(r"^[ \t\f\v]+").unwrap();
    static ref ENDLINE_REGEX: Regex = Regex::new(r"^(\r\n|\n|\r)+").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
	// Identifiers and Literals
	RuleName,
	Keyword,
	RegexLiteral,
	StringLiteral,

	// Operators
	Or,
	And,
	DelimitRepeat,
	RepeatOne,
	RepeatZero,
	Optional,
	OpenParen,
	CloseParen,

	// Special Characters
	RuleDefinition,
	ConfigDirective,
	MetaRule,
	Comment,

	// Miscellaneous
	Whitespace,
	Newline,
}

impl TokenType {
	/// Returns true if the token type is an operator.
	pub fn is_operator(&self) -> bool {
		matches!(
			self,
			TokenType::Or
				| TokenType::And
				| TokenType::DelimitRepeat
				| TokenType::RepeatOne
				| TokenType::RepeatZero
				| TokenType::Optional
		)
	}

	/// Returns true if the token type is a unary operator.
	pub fn is_unary_operator(&self) -> bool {
		matches!(self, TokenType::Optional | TokenType::RepeatOne | TokenType::RepeatZero)
	}

	/// Returns true if the token type is a binary operator.
	pub fn is_binary_operator(&self) -> bool {
		matches!(self, TokenType::Or | TokenType::And | TokenType::DelimitRepeat)
	}

	/// Returns the precedence of the operator. Higher values indicate higher precedence.
	pub fn get_precedence(&self) -> u8 {
		match self {
			TokenType::DelimitRepeat => 5,
			TokenType::RepeatOne => 4,
			TokenType::RepeatZero => 4,
			TokenType::Optional => 3,
			TokenType::And => 2,
			TokenType::Or => 1,
			_ => 0, // Non-operator tokens have no precedence
		}
	}

	/// Returns the compiled regex pattern for the token type.
	pub fn get_regex(&self) -> &'static Regex {
        match self {
            TokenType::RuleName => &RULE_NAME_REGEX,
            TokenType::Keyword => &KEYWORD_REGEX,
            TokenType::RegexLiteral => &REGEX_LITERAL_REGEX,
            TokenType::StringLiteral => &STRING_LITERAL_REGEX,
            TokenType::Or => &OR_REGEX,
            TokenType::And => &AND_REGEX,
            TokenType::DelimitRepeat => &DELIMIT_REPEAT_REGEX,
            TokenType::RepeatOne => &REPEAT_ONE_REGEX,
            TokenType::RepeatZero => &REPEAT_ZERO_REGEX,
            TokenType::Optional => &OPTIONAL_REGEX,
            TokenType::OpenParen => &OPEN_PAREN_REGEX,
            TokenType::CloseParen => &CLOSE_PAREN_REGEX,
            TokenType::RuleDefinition => &RULE_DEF_REGEX,
            TokenType::ConfigDirective => &CONFIG_DIRECTIVE_REGEX,
            TokenType::MetaRule => &META_RULE_REGEX,
            TokenType::Comment => &COMMENT_REGEX,
            TokenType::Whitespace => &WHITESPACE_REGEX,
            TokenType::Newline => &ENDLINE_REGEX,
        }
    }

    /// Transforms the value of the token based on its type. For example, it removes surrounding quotes from string literals.
    pub fn transform(&self, value: &String) -> String {
		match self {
			TokenType::RegexLiteral => {
				value.trim_matches('\"').to_string()
			}
			TokenType::StringLiteral => {
				value.trim_matches('\'').to_string()
			}
			// No transformation needed for other token types
			_ => value.clone(),
		}
	}

	pub fn all() -> Vec<TokenType> {
        vec![
            TokenType::RuleName,
			TokenType::Keyword,
			TokenType::RegexLiteral,
			TokenType::StringLiteral,

			// Operators
			TokenType::Or,
			TokenType::And,
			TokenType::DelimitRepeat,
			TokenType::RepeatOne,
			TokenType::RepeatZero,
			TokenType::Optional,
			TokenType::OpenParen,
			TokenType::CloseParen,

			// Special Characters
			TokenType::RuleDefinition,
			TokenType::ConfigDirective,
			TokenType::MetaRule,
			TokenType::Comment,

			// Miscellaneous
			TokenType::Whitespace,
			TokenType::Newline,
            // Add any other token types here
        ]
    }
}