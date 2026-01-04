use std::error::Error;

pub struct GramSpecConfig {
    pub entry_rule: String,
    pub ignore_between: Vec<String>,
}

impl GramSpecConfig {
    pub fn new() -> Self {
        GramSpecConfig {
            entry_rule: String::from("file"),
            ignore_between: Vec::new(),
        }
    }

    pub fn set(&mut self, config: String, value: String) -> Result<(), Box<dyn Error>> {
        match config.as_str() {
            "entry_rule" => self.entry_rule = value.to_string(),
            "ignore_between" => {
                self.ignore_between.push(value.to_string());
            },
            _ => return Err(format!("Unknown configuration option: {}", config).into()),
        }
        Ok(())
    }
}