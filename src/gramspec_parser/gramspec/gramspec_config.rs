use std::error::Error;

pub struct GramSpecConfig {
	pub entry_rule: String,
	pub ignore_spaces: bool,
	pub ignore_newlines: bool,
	pub ignore_between_tokens: Vec<String>,
}

impl GramSpecConfig {
	pub fn new() -> Self {
		GramSpecConfig {
			entry_rule: String::from("file"),
			ignore_spaces: false,
			ignore_newlines: false,
			ignore_between_tokens: Vec::new(),
		}
	}

	pub fn set(&mut self, config: String, value: String) -> Result<(), Box<dyn Error>> {
		match config.as_str() {
			"entry_rule" => self.entry_rule = value.to_string(),
			"ignore_spaces" => {
				self.ignore_spaces = value.parse::<bool>()?;
			},
			"ignore_newlines" => self.ignore_newlines = value.parse::<bool>()?,
			"ignore_between_tokens" => {
				self.ignore_between_tokens = value
					.split(',')
					.map(|s| s.trim().to_string())
					.collect();
			},
			_ => return Err(format!("Unknown configuration option: {}", config).into()),
		}
		Ok(())
	}
}