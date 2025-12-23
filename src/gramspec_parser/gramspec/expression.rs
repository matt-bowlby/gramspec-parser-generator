use std::fmt;

use crate::gramspec_parser::token::Token;
#[derive(Clone)]
pub enum Expression {
    RuleName(Token),
    RegexLiteral(Token),
    StringLiteral(Token),
    Keyword(Token),
    Or(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    DelimitRepeatOne(Box<Expression>, Box<Expression>),
    DelimitRepeatZero(Box<Expression>, Box<Expression>),
    Optional(Box<Expression>),
    RepeatOne(Box<Expression>),
    RepeatZero(Box<Expression>),
    Discard(Box<Expression>),
    Meta(Box<Expression>),
}

impl fmt::Debug for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::RuleName(token) => write!(f, "{}", token.value),
            Expression::RegexLiteral(token) => write!(f, "\"{}\"", token.value),
            Expression::StringLiteral(token) => write!(f, "\'{}\'", token.value),
            Expression::Keyword(token) => write!(f, "{}", token.value),
            Expression::Or(left, right) => write!(f, "{:?} | {:?}", left, right),
            Expression::And(left, right) => write!(f, "{:?} & {:?}", left, right),
            Expression::DelimitRepeatOne(left, right) => write!(f, "({:?}),({:?})+", left, right),
            Expression::DelimitRepeatZero(left, right) => write!(f, "({:?}),({:?})*", left, right),
            Expression::Optional(expr) => write!(f, "({:?})?", expr),
            Expression::RepeatOne(expr) => write!(f, "({:?})+", expr),
            Expression::RepeatZero(expr) => write!(f, "({:?})*", expr),
            Expression::Discard(expr) => write!(f, "~({:?})", expr),
            Expression::Meta(expr) => write!(f, "$({:?})", expr),
        }
    }
}