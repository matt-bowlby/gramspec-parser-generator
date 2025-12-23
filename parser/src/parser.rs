use regex::Regex;
use std::collections::HashMap;
use std::error::Error;

use crate::expression::Expression::{self, *};
use crate::node::{Node, NodeType};

const KEYWORDS: &[(&str, &str)] = &[("ENDMARKER", "0")];

#[allow(dead_code)]
pub struct GramspecParser {
    pub position: usize,
    pub debug: bool,

    content: std::string::String,
    memos: HashMap<usize, HashMap<std::string::String, Box<Option<Vec<Node>>>>>,
}

#[allow(dead_code)]
impl GramspecParser {
    pub fn new() -> Self {
        GramspecParser {
            content: std::string::String::new(),
            position: 0,
            memos: HashMap::new(),
            debug: false,
        }
    }

    pub fn enable_debug(mut self) -> Self {
        self.debug = true;
        self
    }

    fn debug_log(&self, message: &str) {
        if self.debug {
            println!(
                "DEBUG: {} | Position: {} | Next chars: {:?}",
                message,
                self.position,
                self.content
                    .chars()
                    .skip(self.position)
                    .take(20)
                    .collect::<std::string::String>()
            );
            let mut input = std::string::String::new();
            std::io::stdin().read_line(&mut input).unwrap();
        }
    }

    pub fn parse(&mut self, input: &str) -> Result<Option<Node>, Box<dyn Error>> {
        self.position = 0;
        self.content = input.to_string();
        if let Some(nodes) = self._file()? {
            return Ok(Some(nodes[0].clone()));
        }
        Ok(None)
    }

    pub fn parse_file(&mut self, file_path: &str) -> Result<Option<Node>, Box<dyn Error>> {
        self.parse(&std::fs::read_to_string(file_path)?)
    }

    fn circular_wrapper(
        &mut self,
        rule_name: std::string::String,
    ) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let pos = self.position;

        if let Some(cached_result_box) = self.memos.get(&pos).and_then(|memo| memo.get(&rule_name))
        {
            let cached_result = *cached_result_box.clone();

            let end_pos = cached_result
                .as_ref()
                .and_then(|nodes| nodes.iter().map(|n| n.get_end_pos()).max())
                .unwrap_or(pos);
            self.position = end_pos;

            return Ok(cached_result);
        }

        self.memos
            .entry(pos)
            .or_insert_with(HashMap::new)
            .insert(rule_name.clone(), Box::new(None));

        let mut last_result = None;
        let mut last_pos = pos;

        loop {
            self.position = pos;

            let result = self.call_rule(&rule_name, false)?;
            let end_pos = self.position;

            if end_pos <= last_pos {
                break;
            }

            last_result = result;
            last_pos = end_pos;

            if let Some(memo) = self.memos.get_mut(&pos) {
                memo.insert(rule_name.clone(), Box::new(last_result.clone()));
            }
        }

        // If the result was a failure, remove it from the cache to prevent poisoning
        if last_result.is_none() {
            if let Some(memo) = self.memos.get_mut(&pos) {
                memo.remove(&rule_name);
            }
        }

        self.position = last_pos;
        Ok(last_result)
    }

    fn expect_string(&mut self, string: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        self.debug_log(&format!("Expect string: '{}'", string));
        let mut start_pos = self.position;
        loop {
            if self.content[self.position..].starts_with(string) {
                self.position += string.len();
                return Ok(Some(vec![Node::new_with_position(
                    NodeType::_String,
                    vec![],
                    Some(string.to_string()),
                    start_pos,
                    self.position,
                )]));
            } else {
                if let Some(whitespace) = Regex::new(r"^[^\S\r\n]+")
                    .unwrap()
                    .find(&self.content[self.position..])
                {
                    start_pos += whitespace.end();
                    self.position = start_pos;
                } else {
                    break;
                }
            }
        }
        self.position = start_pos;
        Ok(None)
    }

    fn expect_regex(&mut self, regex: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        self.debug_log(&format!("Expect regex: '{}'", regex));
        let mut start_pos = self.position;
        loop {
            if let Some(captures) = Regex::new(regex)
                .unwrap()
                .captures(&self.content[self.position..])
            {
                self.position += captures.get(0).unwrap().end();
                return Ok(Some(vec![Node::new_with_position(
                    NodeType::_String,
                    vec![],
                    Some(captures.get(0).unwrap().as_str().to_string()),
                    start_pos,
                    self.position,
                )]));
            } else {
                if let Some(whitespace) = Regex::new(r"^[^\S\r\n]+")
                    .unwrap()
                    .find(&self.content[self.position..])
                {
                    start_pos += whitespace.end();
                    self.position = start_pos;
                } else {
                    break;
                }
            }
        }
        self.position = start_pos;
        Ok(None)
    }

    fn get_keywords_map(&self) -> HashMap<std::string::String, std::string::String> {
        KEYWORDS
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let keyword_value = self
            .get_keywords_map()
            .get(keyword)
            .ok_or_else(|| format!("Unknown keyword: {}", keyword))?
            .to_owned();
        if self.content[self.position..].starts_with(&keyword_value) {
            self.position += keyword_value.len();
            return Ok(Some(vec![Node::new_with_position(
                NodeType::_String,
                vec![],
                Some(keyword.to_string()),
                start_pos,
                self.position,
            )]));
        }
        self.position = start_pos;
        Ok(None)
    }

    fn eval(&mut self, expression: &Expression) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        match expression {
            Expression::Rule(rule) => {
                if let Some(nodes) = self.call_rule(rule, true)? {
                    Ok(Some(nodes))
                } else {
                    Ok(None)
                }
            }
            Expression::RegexLiteral(regex) => self.expect_regex(regex),
            Expression::StringLiteral(string) => self.expect_string(string),
            Expression::Keyword(keyword) => self.expect_keyword(keyword),
            Expression::Or(left, right) => {
                let start_pos = self.position;
                let left_nodes = self.eval(&*left)?;
                let left_end = self.position;
                self.position = start_pos;
                let right_nodes = self.eval(&*right)?;
                let right_end = self.position;

                if left_end > right_end {
                    self.position = left_end;
                    return Ok(left_nodes);
                } else if right_end > left_end {
                    self.position = right_end;
                    return Ok(right_nodes);
                } else {
                    self.position = start_pos;
                    return Ok(None);
                }
            }
            Expression::And(left, right) => {
                let left_nodes = self.eval(&*left)?;
                if left_nodes.is_none() {
                    return Ok(None);
                }
                let right_nodes = self.eval(&*right)?;
                if right_nodes.is_none() {
                    return Ok(None);
                }
                let mut final_nodes = left_nodes.unwrap();
                final_nodes.extend(right_nodes.unwrap());
                Ok(Some(final_nodes))
            }
            Expression::DelimitRepeatOne(expression, delimiter) => {
                // Attempt to parse the first expression
                let nodes = self.eval(&*expression)?;
                // If the first expression fails, return an empty vector
                if nodes.is_none() {
                    return Ok(None);
                }

                let mut nodes = nodes.unwrap();

                // Attempt to parse subsequent expressions with delimiters
                loop {
                    let start = self.position;
                    // Attempt to parse the delimiter
                    let delimiter_nodes = self.eval(&*delimiter)?;
                    // If it fails, break the loop
                    if delimiter_nodes.is_none() {
                        self.position = start; // Technically unnecessary as a failure would leave position unchanged, but just to be consistent
                        break;
                    }
                    // Attempt to parse the next expression
                    let expression_nodes = self.eval(&*expression)?;
                    // If the next expression fails, break the loop
                    if expression_nodes.is_none() {
                        self.position = start;
                        break;
                    }

                    // Only if both delimiter and expression are successful, append them to the nodes
                    nodes.extend(delimiter_nodes.unwrap());
                    nodes.extend(expression_nodes.unwrap());

                    // Prevent infinite loops by checking if position has advanced
                    if self.position <= start {
                        break;
                    }
                }

                // Return the nodes collected so far
                Ok(Some(nodes))
            }
            Expression::DelimitRepeatZero(left, right) => {
                // Attempt to parse the first expression
                let nodes = self.eval(&*left)?;
                // If the first expression fails, return an empty vector
                if nodes.is_none() {
                    return Ok(Some(vec![]));
                }

                let mut nodes = nodes.unwrap();

                // Attempt to parse subsequent expressions with delimiters
                loop {
                    let start = self.position;
                    // Attempt to parse the delimiter
                    let delimiter_nodes = self.eval(&*right)?;
                    // If it fails, break the loop
                    if delimiter_nodes.is_none() {
                        self.position = start; // Technically unnecessary as a failure would leave position unchanged, but just to be consistent
                        break;
                    }
                    // Attempt to parse the next expression
                    let expression_nodes = self.eval(&*left)?;
                    // If the next expression fails, break the loop
                    if expression_nodes.is_none() {
                        self.position = start;
                        break;
                    }

                    // Only if both delimiter and expression are successful, append them to the nodes
                    nodes.extend(delimiter_nodes.unwrap());
                    nodes.extend(expression_nodes.unwrap());

                    // Prevent infinite loops by checking if position has advanced
                    if self.position <= start {
                        break;
                    }
                }

                // Return the nodes collected so far
                Ok(Some(nodes))
            }
            Expression::RepeatOne(expr) => {
                let mut nodes = self.eval(&*expr)?;
                if nodes.is_none() {
                    return Ok(None);
                }

                let mut last_pos = self.position;
                while let Some(new_nodes) = self.eval(&*expr)? {
                    nodes.as_mut().unwrap().extend(new_nodes);
                    if self.position == last_pos {
                        break;
                    }
                    last_pos = self.position;
                }

                Ok(nodes)
            }
            Expression::RepeatZero(expr) => {
                let mut nodes = self.eval(&*expr)?;
                if nodes.is_none() {
                    return Ok(Some(vec![]));
                }

                let mut last_pos = self.position;
                while let Some(new_nodes) = self.eval(&*expr)? {
                    nodes.as_mut().unwrap().extend(new_nodes);
                    if self.position == last_pos {
                        break;
                    }
                    last_pos = self.position;
                }

                Ok(nodes)
            }
            Expression::Optional(expr) => {
                let mut nodes = self.eval(&*expr)?;

                if nodes.is_none() {
                    nodes = Some(vec![]);
                }

                Ok(nodes)
            }
            Expression::Discard(expr) => {
                let nodes = self.eval(&*expr)?;
                if nodes.is_none() {
                    return Ok(None);
                }
                let nodes = nodes.unwrap();
                let last_node = nodes[nodes.len() - 1].clone();
                let node = Node::new_with_position(
                    NodeType::_Discard,
                    vec![],
                    None,
                    self.position,
                    last_node.get_end_pos(),
                );
                Ok(Some(vec![node]))
            }
            Expression::Meta(expr) => {
                let nodes = self.eval(&*expr)?;
                if nodes.is_none() {
                    return Ok(None);
                }
                // Assume that the length of nodes is 1 for Meta, and is a Rule Node
                let nodes = nodes.unwrap()[0].children.clone();
                Ok(Some(nodes))
            }
        }
    }

    fn get_longest_expression_match(
        &mut self,
        expressions: &[Expression],
    ) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let mut longest_end = start_pos;
        let mut longest_nodes = None;

        for expr in expressions.iter() {
            let result = self.eval(&expr)?;
            let new_end_pos = self.position;
            self.position = start_pos; // Reset position to start for each expression evaluation
            if new_end_pos > longest_end || longest_nodes.is_none() {
                longest_end = new_end_pos;
                longest_nodes = result;
            }
        }
        if longest_nodes.is_none() {
            self.position = start_pos; // Reset position if no matches found
        } else {
            self.position = longest_end; // Update position to the end of the longest match
        }
        Ok(longest_nodes)
    }

    fn call_rule(
        &mut self,
        rule_name: &str,
        protected: bool,
    ) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        self.debug_log(&format!("Calling rule: {}", rule_name));
        match rule_name {
            "delimit_repeat_zero" => {
                if protected {
                    self.circular_wrapper("delimit_repeat_zero".to_string())
                } else {
                    self._delimit_repeat_zero()
                }
            }
            "group" => self._group(),
            "identifier" => self._identifier(),
            "optional" => {
                if protected {
                    self.circular_wrapper("optional".to_string())
                } else {
                    self._optional()
                }
            }
            "new_lines" => self._new_lines(),
            "delimit_repeat_one" => {
                if protected {
                    self.circular_wrapper("delimit_repeat_one".to_string())
                } else {
                    self._delimit_repeat_one()
                }
            }
            "meta_rule_definition" => self._meta_rule_definition(),
            "repeat_one" => {
                if protected {
                    self.circular_wrapper("repeat_one".to_string())
                } else {
                    self._repeat_one()
                }
            }
            "discarded_rule_definition" => self._discarded_rule_definition(),
            "discarded_value" => self._discarded_value(),
            "white_space" => self._white_space(),
            "comment" => self._comment(),
            "repeat_zero" => {
                if protected {
                    self.circular_wrapper("repeat_zero".to_string())
                } else {
                    self._repeat_zero()
                }
            }
            "and" => {
                if protected {
                    self.circular_wrapper("and".to_string())
                } else {
                    self._and()
                }
            }
            "rule_definition" => self._rule_definition(),
            "string" => self._string(),
            "config_directive" => self._config_directive(),
            "file" => self._file(),
            "or" => {
                if protected {
                    self.circular_wrapper("or".to_string())
                } else {
                    self._or()
                }
            }
            "regex" => self._regex(),
            "expressions" => self._expressions(),
            "meta_value" => self._meta_value(),
            "alternative" => {
                if protected {
                    self.circular_wrapper("alternative".to_string())
                } else {
                    self._alternative()
                }
            }
            "line" => self._line(),
            "value" => self._value(),
            "member" => self._member(),
            _ => Err(format!("Unknown rule: {}", rule_name).into()),
        }
    }

    fn _delimit_repeat_zero(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::and(Rule("alternative"), StringLiteral(",")),
            Rule("repeat_zero"),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::DelimitRepeatZero,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _group(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::and(
                StringLiteral("("),
                Expression::or(Rule("alternative"), Rule("or")),
            ),
            StringLiteral(")"),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node =
                Node::new_with_position(NodeType::Group, matches, None, start_pos, self.position);
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _identifier(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [RegexLiteral(r#"^[a-z][a-z0-9_]*"#)];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::Identifier,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _optional(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] =
            [Expression::and(Rule("alternative"), StringLiteral("?"))];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::Optional,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _new_lines(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [RegexLiteral(r#"^(\r?\n)+"#)];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::NewLines,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _delimit_repeat_one(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::and(Rule("alternative"), StringLiteral(",")),
            Rule("repeat_one"),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::DelimitRepeatOne,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _meta_rule_definition(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::and(
                Expression::and(StringLiteral("$"), Rule("identifier")),
                StringLiteral(":"),
            ),
            Rule("expressions"),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::MetaRuleDefinition,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _repeat_one(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] =
            [Expression::and(Rule("alternative"), StringLiteral("+"))];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::RepeatOne,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _discarded_rule_definition(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::and(
                Expression::and(StringLiteral("~"), Rule("identifier")),
                StringLiteral(":"),
            ),
            Rule("expressions"),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::DiscardRuleDefinition,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _discarded_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            StringLiteral("~"),
            Expression::or(
                Expression::or(Rule("string"), Rule("regex")),
                Rule("identifier"),
            ),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::DiscardValue,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _white_space(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [RegexLiteral(r#"^\s+"#)];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::WhiteSpace,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _comment(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [RegexLiteral(r#"^#[^\r\n]*"#)];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node =
                Node::new_with_position(NodeType::Comment, matches, None, start_pos, self.position);
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _repeat_zero(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] =
            [Expression::and(Rule("alternative"), StringLiteral("*"))];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::RepeatZero,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _and(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] =
            [Expression::and(Rule("alternative"), Rule("alternative"))];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node =
                Node::new_with_position(NodeType::And, matches, None, start_pos, self.position);
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _rule_definition(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::and(Rule("identifier"), StringLiteral(":")),
            Rule("expressions"),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::RuleDefinition,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _string(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [RegexLiteral(r#"^\'(?:\\.|[^\'\\])*\'"#)];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node =
                Node::new_with_position(NodeType::String, matches, None, start_pos, self.position);
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _config_directive(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::and(
                Expression::and(
                    Expression::and(StringLiteral("@"), Rule("identifier")),
                    StringLiteral(":"),
                ),
                Rule("string"),
            ),
            Rule("white_space"),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::ConfigDirective,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _file(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::and(
                Expression::optional(Rule("white_space")),
                Expression::delimit_repeat_zero(Rule("line"), Rule("white_space")),
            ),
            Expression::optional(Rule("white_space")),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node =
                Node::new_with_position(NodeType::File, matches, None, start_pos, self.position);
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _or(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Rule("alternative"),
            Expression::repeat_one(Expression::and(StringLiteral("|"), Rule("alternative"))),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node =
                Node::new_with_position(NodeType::Or, matches, None, start_pos, self.position);
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _regex(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            StringLiteral("r"),
            RegexLiteral(r#"^\'(?:\\.|[^\'\\])*\'"#),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node =
                Node::new_with_position(NodeType::Regex, matches, None, start_pos, self.position);
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _expressions(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            Expression::optional(Expression::and(
                Expression::and(Rule("new_lines"), Rule("white_space")),
                StringLiteral("|"),
            )),
            Expression::delimit_repeat_one(Rule("alternative"), StringLiteral("|")),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::Expressions,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _meta_value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let start_pos = self.position;
        let expressions: [Expression; 1] = [Expression::and(
            StringLiteral("$"),
            Expression::or(
                Expression::or(Rule("string"), Rule("regex")),
                Rule("identifier"),
            ),
        )];

        if let Some(matches) = self.get_longest_expression_match(&expressions)? {
            let node = Node::new_with_position(
                NodeType::MetaValue,
                matches,
                None,
                start_pos,
                self.position,
            );
            return Ok(Some(vec![node]));
        }

        Ok(None)
    }

    fn _alternative(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let expressions: [Expression; 8] = [
            Rule("value"),
            Rule("and"),
            Rule("group"),
            Rule("optional"),
            Rule("repeat_zero"),
            Rule("repeat_one"),
            Rule("delimit_repeat_zero"),
            Rule("delimit_repeat_one"),
        ];

        self.get_longest_expression_match(&expressions)
    }

    fn _line(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let expressions: [Expression; 2] = [
            Expression::and(Rule("member"), Expression::optional(Rule("comment"))),
            Rule("comment"),
        ];

        self.get_longest_expression_match(&expressions)
    }

    fn _value(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let expressions: [Expression; 5] = [
            Rule("discarded_value"),
            Rule("meta_value"),
            Rule("string"),
            Rule("regex"),
            Rule("identifier"),
        ];

        self.get_longest_expression_match(&expressions)
    }

    fn _member(&mut self) -> Result<Option<Vec<Node>>, Box<dyn Error>> {
        let expressions: [Expression; 4] = [
            Rule("config_directive"),
            Rule("rule_definition"),
            Rule("meta_rule_definition"),
            Rule("discarded_rule_definition"),
        ];

        self.get_longest_expression_match(&expressions)
    }
}
