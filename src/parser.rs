use std::error::Error;

use crate::parser::lang::Lang;

mod lang;

/// Generated Grammar Specification
struct TestLanguage {
    lang: Lang,
}

impl TestLanguage {

    pub fn new() -> Self {
        TestLanguage { lang: Lang::new("TestLanguage", "".to_string()) }
    }

    pub fn parse(&mut self, input: String) -> Result<Option<()>, Box<dyn Error>> {
        self.lang.set_content(input);
        self.file()?;
        Ok(None)
    }

    fn file(&mut self) -> Result<Option<()>, Box<dyn Error>> {
        Ok(None)
    }

}

