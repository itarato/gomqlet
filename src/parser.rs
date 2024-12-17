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
        if self.is_next_keyword("mutation") {
            todo!("Handle mutation");
        }

        Ok(ast::Root::Query(self.parse_query()?))
    }

    fn parse_query(&mut self) -> Result<ast::Query, ParseError> {
        if self.is_next_keyword("query") {
            self.ptr += 1;
        }

        // TODO variables.

        let fields = self.parse_fields_subobject()?;

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

        Ok(fields)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.ptr)
    }

    fn peek_token_cloned(&self) -> Option<Token> {
        self.peek_token().map(|token| token.clone())
    }

    fn is_next_keyword(&self, value: &str) -> bool {
        if let Some(Token {
            kind: TokenKind::Keyword(keyword),
            ..
        }) = self.peek_token()
        {
            if keyword == value {
                return true;
            }
        }

        false
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
    fn test_optional_query_keywors() {
        let Root::Query(query) = parse("query { user }").unwrap();
        assert_eq!(1, query.fields.len());
    }

    #[test]
    fn test_plan_fields() {
        let Root::Query(query) = parse("{ user company task }").unwrap();
        assert_eq!(3, query.fields.len());
        assert_eq!("user".to_string(), query.fields[0].name);
        assert_eq!("company".to_string(), query.fields[1].name);
        assert_eq!("task".to_string(), query.fields[2].name);
    }

    #[test]
    fn test_nested_fields() {
        let Root::Query(query) = parse("{ user company { location size } }").unwrap();
        assert_eq!(2, query.fields.len());
        assert_eq!(2, query.fields[1].fields.len());
    }

    fn parse(raw: &str) -> Result<Root, ParseError> {
        let tokens = Tokenizer::tokenize(raw, false);
        let parser = Parser::new(tokens);
        parser.parse()
    }
}
