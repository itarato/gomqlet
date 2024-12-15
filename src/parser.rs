use crate::ast;
use crate::tokenizer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    ptr: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, ptr: 0 }
    }

    pub fn parse(self) -> Vec<ast::Root> {
        let mut roots = vec![];

        roots
    }

    fn parse_query(&mut self) -> ast::Query {
        let mut fields = vec![];

        ast::Query { fields }
    }
}
