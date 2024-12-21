use crate::{parser::Parser, tokenizer::Token};

pub struct Analyzer;

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer
    }

    pub fn analyze(&self, tokens: Vec<Token>) {
        let ast = Parser::new(tokens).parse();
    }
}
