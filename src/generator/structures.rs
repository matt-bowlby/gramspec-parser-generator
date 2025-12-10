pub fn use_statements(indent_level: usize, use_statements: &[&str]) -> String {
	let mut uses = String::from("");
	for use_statement in use_statements {
		uses.push_str(&format!("{}{}\n", "  ".repeat(indent_level), use_statement));
	}
	uses
}

pub fn keywords(indent_level: usize) -> String {
	format!(
		"{}const KEYWORDS: &[(&str, &str)] = &[
		{}  (\"ENDMARKER\", \"0\"),
		{}];",
		"  ".repeat(indent_level),
		"  ".repeat(indent_level),
		"  ".repeat(indent_level)
	)
}