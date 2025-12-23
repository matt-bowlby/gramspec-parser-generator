use std::error::Error;
use std::io::Write;

use crate::gramspec_parser::gramspec::GramSpec;
use crate::gramspec_parser::gramspec::expression::Expression;

pub struct Generator {
    gramspec: GramSpec,
}

impl Generator {
    pub fn new(gramspec: GramSpec) -> Self {
        Generator { gramspec }
    }

    pub fn generate(&self, output_file: &str, parser_name: &str, tab_string: &str) -> Result<(), Box<dyn Error>> {
        // Read templates
        let parser_template = std::fs::read_to_string("./templates/parser.txt")?;
        let rule_cases = self.generate_rule_cases()?;
        let rule_functions = self.generate_rule_functions()?;
        let meta_rule_functions = self.generate_meta_rule_functions()?;
        let discard_rule_functions = self.generate_discard_rule_functions()?;
        let node_types = self.node_types();

        // Initialize contents
        let mut contents = String::new();

        // Add parser template to contents
        contents.push_str(&parser_template);

        // Replace placeholders
        contents = contents.replace("_PARSERNAME_", parser_name);
        contents = contents.replace("_ENTRYRULE_", &self.gramspec.config.entry_rule);
        contents = contents.replace("_RULECASES_", &rule_cases);
        contents = contents.replace("_RULEFUNCTIONS_", &rule_functions);
        contents = contents.replace("_METARULEFUNCTIONS_", &meta_rule_functions);
        contents = contents.replace("_DISCARDRULEFUNCTIONS_", &discard_rule_functions);
        contents = contents.replace("_PASCALCASERULENAMES_", &node_types);
        contents = contents.replace("_TS_", tab_string); // Replace tab spaces

        // Write to output file
        let file = std::fs::File::create(output_file)?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(contents.as_bytes())?;

        Ok(())
    }

    fn generate_rule_cases(&self) -> Result<String, Box<dyn Error>> {
        let rule_case_regular_template =
            std::fs::read_to_string("./templates/rule_case_regular.txt")?;
        let rule_case_circular_template =
            std::fs::read_to_string("./templates/rule_case_circular.txt")?;

        // Generate rule cases
        let mut rule_cases = String::new();
        for i in 0..self.gramspec.rules.keys().len() {
            let rule = self.gramspec.rules.keys().nth(i).unwrap();
            if self.gramspec.is_left_circular(rule) {
                rule_cases.push_str(&rule_case_circular_template.replace("_RULENAME_", rule));
            } else {
                rule_cases.push_str(&rule_case_regular_template.replace("_RULENAME_", rule));
            }
            rule_cases.push('\n');
        }
        for i in 0..self.gramspec.meta_rules.keys().len() {
            let rule = self.gramspec.meta_rules.keys().nth(i).unwrap();
            if self.gramspec.is_left_circular(rule) {
                rule_cases.push_str(&rule_case_circular_template.replace("_RULENAME_", rule));
            } else {
                rule_cases.push_str(&rule_case_regular_template.replace("_RULENAME_", rule));
            }
            rule_cases.push('\n');
        }
        for i in 0..self.gramspec.discard_rules.keys().len() {
            let rule = self.gramspec.discard_rules.keys().nth(i).unwrap();
            if self.gramspec.is_left_circular(rule) {
                rule_cases.push_str(&rule_case_circular_template.replace("_RULENAME_", rule));
            } else {
                rule_cases.push_str(&rule_case_regular_template.replace("_RULENAME_", rule));
            }
            rule_cases.push('\n');
        }
        Ok(rule_cases)
    }

    fn generate_rule_functions(&self) -> Result<String, Box<dyn Error>> {
        let rule_function_template = std::fs::read_to_string("./templates/rule_function.txt")?;

        // Generate rule functions
        let mut rule_functions = String::new();
        for i in 0..self.gramspec.rules.keys().len() {
            let rule = self.gramspec.rules.keys().nth(i).unwrap();
            let token_expression = self
                .gramspec
                .rules
                .get(rule)
                .or_else(|| self.gramspec.meta_rules.get(rule))
                .ok_or_else(|| format!("Rule '{}' not found", rule))?;

            let mut expressions = String::from("");
            for i in 0..token_expression.len() {
                let expression = &token_expression[i];
                expressions.push_str(&format!("_TS__TS__TS_{},", self.to_conditional(expression)?));
                if i < token_expression.len() - 1 {
                    expressions.push_str("\n");
                }
            }

            rule_functions.push_str(
                &rule_function_template
                    .replace("_RULENAME_", &format!("{}", rule))
                    .replace("_EXPRESSIONS_", &expressions)
                    .replace("_EXPRESSIONSLENGTH_", &token_expression.len().to_string())
                    .replace("_PASCALCASERULENAME_", &Self::to_pascal_case(rule)),
            );
            if i < self.gramspec.rules.keys().len() - 1 {
                rule_functions.push_str("\n\n");
            }
        }

        Ok(rule_functions)
    }

    fn generate_meta_rule_functions(&self) -> Result<String, Box<dyn Error>> {
        let rule_function_template = std::fs::read_to_string("./templates/meta_rule_function.txt")?;

        // Generate rule functions
        let mut rule_functions = String::new();
        for i in 0..self.gramspec.meta_rules.keys().len() {
            let rule = self.gramspec.meta_rules.keys().nth(i).unwrap();
            let token_expression = self
                .gramspec
                .rules
                .get(rule)
                .or_else(|| self.gramspec.meta_rules.get(rule))
                .ok_or_else(|| format!("Rule '{}' not found", rule))?;

            let mut expressions = String::from("");
            for i in 0..token_expression.len() {
                let expression = &token_expression[i];
                expressions.push_str(&format!("_TS__TS__TS_{},", self.to_conditional(expression)?));
                if i < token_expression.len() - 1 {
                    expressions.push_str("\n");
                }
            }

            rule_functions.push_str(
                &rule_function_template
                    .replace("_RULENAME_", &format!("{}", rule))
                    .replace("_EXPRESSIONS_", &expressions)
                    .replace("_EXPRESSIONSLENGTH_", &token_expression.len().to_string()),
            );
            if i < self.gramspec.meta_rules.keys().len() - 1 {
                rule_functions.push_str("\n\n");
            }
        }

        Ok(rule_functions)
    }

    fn generate_discard_rule_functions(&self) -> Result<String, Box<dyn Error>> {
        let rule_function_template = std::fs::read_to_string("./templates/discard_rule_function.txt")?;

        // Generate rule functions
        let mut rule_functions = String::new();
        for i in 0..self.gramspec.discard_rules.keys().len() {
            let rule = self.gramspec.discard_rules.keys().nth(i).unwrap();
            let token_expression = self
                .gramspec
                .rules
                .get(rule)
                .or_else(|| self.gramspec.discard_rules.get(rule))
                .ok_or_else(|| format!("Rule '{}' not found", rule))?;

            let mut expressions = String::from("");
            for i in 0..token_expression.len() {
                let expression = &token_expression[i];
                expressions.push_str(&format!("_TS__TS__TS_{},", self.to_conditional(expression)?));
                if i < token_expression.len() - 1 {
                    expressions.push_str("\n");
                }
            }

            rule_functions.push_str(
                &rule_function_template
                    .replace("_RULENAME_", &format!("{}", rule))
                    .replace("_EXPRESSIONS_", &expressions)
                    .replace("_EXPRESSIONSLENGTH_", &token_expression.len().to_string()),
            );
            if i < self.gramspec.discard_rules.keys().len() - 1 {
                rule_functions.push_str("\n\n");
            }
        }

        Ok(rule_functions)
    }

    fn to_conditional(&self, expression: &Expression) -> Result<String, Box<dyn Error>> {
        match expression {
            Expression::RuleName(name) => Ok(format!("Rule(\"{}\")", name.value)),
            Expression::Keyword(keyword) => Ok(format!("Keyword(\"{}\")", keyword.value)),
            Expression::RegexLiteral(regex) => Ok(format!("RegexLiteral(r#\"^{}\"#)", regex.value)),
            Expression::StringLiteral(string) => {
                if string.value == "\"" {
                    Ok("StringLiteral(\"\\\"\")".to_string())
                } else if string.value == "\\" {
                    Ok("StringLiteral(\"\\\\\")".to_string())
                } else if string.value == "\n" {
                    Ok("StringLiteral(\"\\n\")".to_string())
                } else if string.value == "\t" {
                    Ok("StringLiteral(\"\\t\")".to_string())
                } else {
                    Ok(format!("StringLiteral(\"{}\")", string.value))
                }
            }
            Expression::Discard(expr) => Ok(format!(
                "Expression::discard({})",
                self.to_conditional(expr)?
            )),
            Expression::Meta(expr) => Ok(format!(
                "Expression::meta({})",
                self.to_conditional(expr)?
            )),
            Expression::Or(left, right) => Ok(format!(
                "Expression::or({}, {})",
                self.to_conditional(left)?,
                self.to_conditional(right)?
            )),
            Expression::And(left, right) => Ok(format!(
                "Expression::and({}, {})",
                self.to_conditional(left)?,
                self.to_conditional(right)?
            )),
            Expression::DelimitRepeatOne(left, right) => Ok(format!(
                "Expression::delimit_repeat_one({}, {})",
                self.to_conditional(left)?,
                self.to_conditional(right)?
            )),
            Expression::DelimitRepeatZero(left, right) => Ok(format!(
                "Expression::delimit_repeat_zero({}, {})",
                self.to_conditional(left)?,
                self.to_conditional(right)?
            )),
            Expression::Optional(expr) => Ok(format!(
                "Expression::optional({})",
                self.to_conditional(expr)?
            )),
            Expression::RepeatOne(expr) => Ok(format!(
                "Expression::repeat_one({})",
                self.to_conditional(expr)?
            )),
            Expression::RepeatZero(expr) => Ok(format!(
                "Expression::repeat_zero({})",
                self.to_conditional(expr)?
            )),
        }
    }

    fn node_types(&self) -> String {
        let mut result = String::new();
        for i in 0..self.gramspec.rules.keys().len() {
            let rule = self.gramspec.rules.keys().nth(i).unwrap();
            if i < self.gramspec.rules.keys().len() - 1 {
                result.push_str(&format!("_TS__TS_{},\n", Self::to_pascal_case(rule)));
            } else {
                result.push_str(&format!("_TS__TS_{},", Self::to_pascal_case(rule)));
            }
        }
        result
    }

    fn to_pascal_case(s: &str) -> String {
        s.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
}
}
