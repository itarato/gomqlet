use crate::ast;
use crate::tokenizer::{Token, TokenKind};

pub enum ParseErrorScope {
    Root,
    Query,
    Mutation,
    Field,
}

pub struct ParseError {
    token: Option<Token>,
    scope: ParseErrorScope,
    message: String,
}

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

    fn parse_query(&mut self) -> Result<ast::Query, ParseError> {
        if let Some(Token {
            kind: TokenKind::OpenBrace,
            ..
        }) = self.peek_token()
        {
            // Noop.
        } else {
            return Err(ParseError {
                token: self.peek_token_cloned(),
                scope: ParseErrorScope::Query,
                message: "Missing opening brace".into(),
            });
        }

        self.ptr += 1;

        let mut fields = vec![];
        loop {
            if self.peek_token().is_none() {
                return Err(ParseError {
                    token: None,
                    scope: ParseErrorScope::Query,
                    message: "Missing closing brace".into(),
                });
            }

            if let TokenKind::CloseBrace = self.peek_token().unwrap().kind {
                self.ptr += 1;
                break;
            }

            fields.push(self.parse_field()?);
        }

        Ok(ast::Query { fields })
    }

    fn parse_field(&mut self) -> Result<ast::Field, ParseError> {
        let name = if let Some(Token {
            kind: TokenKind::Keyword(keyword),
            ..
        }) = self.peek_token()
        {
            keyword.clone()
        } else {
            return Err(ParseError {
                message: "Missing field name".into(),
                token: self.peek_token_cloned(),
                scope: ParseErrorScope::Field,
            });
        };

        self.ptr += 1;

        let arglist = if let Some(Token {
            kind: TokenKind::OpenParen,
            ..
        }) = self.peek_token()
        {
            Some(self.parse_arglist()?)
        } else {
            None
        };

        let fields = if let Some(Token {
            kind: TokenKind::OpenBrace,
            ..
        }) = self.peek_token()
        {
            self.parse_fields_subobject()?
        } else {
            vec![]
        };

        Ok(ast::Field {
            name,
            arglist,
            fields,
        })
    }

    fn parse_arglist(&mut self) -> Result<ast::ArgList, ParseError> {
        todo!()
    }

    fn parse_fields_subobject(&mut self) -> Result<Vec<ast::Field>, ParseError> {
        todo!()
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.ptr)
    }

    fn peek_token_cloned(&self) -> Option<Token> {
        self.peek_token().map(|token| token.clone())
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::Root, tokenizer::Tokenizer};

    use super::Parser;

    #[test]
    fn test_empty() {
        let ast = parse("{}");
        assert_eq!(0, ast.len());
    }

    #[test]
    fn test_plan_fields() {}

    fn parse(raw: &str) -> Vec<Root> {
        let tokens = Tokenizer::tokenize(raw, false);
        let parser = Parser::new(tokens);
        parser.parse()
    }
}
