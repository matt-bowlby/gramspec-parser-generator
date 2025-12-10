use std::fmt;

#[derive(Clone)]
#[allow(dead_code)]
pub enum Expression {
	Rule(&'static str),
	RegexLiteral(&'static str),
	StringLiteral(&'static str),
	Keyword(&'static str),
	Or(Box<Expression>, Box<Expression>),
	And(Box<Expression>, Box<Expression>),
	DelimitRepeatOne(Box<Expression>, Box<Expression>),
	DelimitRepeatZero(Box<Expression>, Box<Expression>),
	Optional(Box<Expression>),
	RepeatOne(Box<Expression>),
	RepeatZero(Box<Expression>),
}

impl fmt::Debug for Expression {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Expression::Rule(rule) => write!(f, "{}", rule),
			Expression::RegexLiteral(regex) => write!(f, "{}", regex),
			Expression::StringLiteral(string) => write!(f, "{}", string),
			Expression::Keyword(keyword) => write!(f, "{}", keyword),
			Expression::Or(left, right) => write!(f, "{:?} | {:?}", left, right),
			Expression::And(left, right) => write!(f, "{:?} & {:?}", left, right),
			Expression::DelimitRepeatOne(left, right) => write!(f, "({:?}),({:?})+", left, right),
			Expression::DelimitRepeatZero(left, right) => write!(f, "({:?}),({:?})*", left, right),
			Expression::Optional(expr) => write!(f, "({:?})?", expr),
			Expression::RepeatOne(expr) => write!(f, "({:?})+", expr),
			Expression::RepeatZero(expr) => write!(f, "({:?})*", expr),
		}
	}
}

#[allow(dead_code)]
impl Expression {
	pub fn or(left: Expression, right: Expression) -> Self {
		Expression::Or(Box::new(left), Box::new(right))
	}
	pub fn and(left: Expression, right: Expression) -> Self {
		Expression::And(Box::new(left), Box::new(right))
	}
	pub fn delimit_repeat_one(left: Expression, right: Expression) -> Self {
		Expression::DelimitRepeatOne(Box::new(left), Box::new(right))
	}
	pub fn delimit_repeat_zero(left: Expression, right: Expression) -> Self {
		Expression::DelimitRepeatZero(Box::new(left), Box::new(right))
	}
	pub fn optional(expr: Expression) -> Self {
		Expression::Optional(Box::new(expr))
	}
	pub fn repeat_one(expr: Expression) -> Self {
		Expression::RepeatOne(Box::new(expr))
	}
	pub fn repeat_zero(expr: Expression) -> Self {
		Expression::RepeatZero(Box::new(expr))
	}
}