pub mod expression;

use std::collections::HashMap;
use crate::gramspec_parser::gramspec::expression::Expression;
pub struct GramSpec {
	pub rules: HashMap<String, Expression>,
	pub config_directives: HashMap<String, String>,
	pub meta_rules: HashMap<String, Expression>,
}