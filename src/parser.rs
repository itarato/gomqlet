use crate::ast;
use crate::tokenizer::{Token, TokenKind};

#[derive(Debug)]
pub enum ParseErrorScope {
    Root,
    Query,
    Mutation,
    Field,
}

#[derive(Debug)]
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

    pub fn parse(mut self) -> Result<ast::Root, ParseError> {
        Ok(ast::Root::Query(self.parse_query()?))
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

    use super::{ParseError, Parser};

    #[test]
    fn test_empty() {
        let Root::Query(query) = parse("{}").unwrap();
        assert_eq!(0, query.fields.len());
    }

    #[test]
    fn test_plan_fields() {
        let Root::Query(ast) = parse("{ user company task }").unwrap();
        assert_eq!(3, ast.fields.len());
        assert_eq!("user".to_string(), ast.fields[0].name);
        assert_eq!("company".to_string(), ast.fields[1].name);
        assert_eq!("task".to_string(), ast.fields[2].name);
    }

    fn parse(raw: &str) -> Result<Root, ParseError> {
        let tokens = Tokenizer::tokenize(raw, false);
        let parser = Parser::new(tokens);
        parser.parse()
    }
}
