use crate::ast;
use crate::tokenizer::{Token, TokenKind};

#[derive(Debug)]
pub enum ParseErrorScope {
    Query,
    Field,
    ArgList,
    ArgListValue,
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
            return Err(self.parse_error(ParseErrorScope::Field, "Missing field name"));
        };
        self.ptr += 1;

        let arglist = if self.is_next_token_kind(TokenKind::OpenParen) {
            Some(self.parse_arglist()?)
        } else {
            None
        };

        let fields = if self.is_next_token_kind(TokenKind::OpenBrace) {
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
        if !self.is_next_token_kind(TokenKind::OpenParen) {
            return Err(self.parse_error(ParseErrorScope::ArgList, "Missing open paren"));
        }
        self.ptr += 1;

        let mut params = vec![];
        loop {
            let key = if let Some(Token {
                kind: TokenKind::Keyword(key),
                ..
            }) = self.peek_token()
            {
                key.into()
            } else {
                return Err(self.parse_error(ParseErrorScope::ArgList, "Missing keyword"));
            };
            self.ptr += 1;

            if !self.is_next_token_kind(TokenKind::Colon) {
                return Err(self.parse_error(ParseErrorScope::ArgList, "Missing colon"));
            }
            self.ptr += 1;

            let value = self.parse_arglist_value()?;

            params.push(ast::ParamKeyValuePair { key, value });

            if self.is_next_token_kind(TokenKind::Comma) {
                self.ptr += 1;
                continue;
            } else {
                break;
            }
        }

        if !self.is_next_token_kind(TokenKind::CloseParen) {
            return Err(self.parse_error(ParseErrorScope::ArgList, "Missing close paren"));
        }
        self.ptr += 1;

        Ok(ast::ArgList { params })
    }

    fn parse_arglist_value(&mut self) -> Result<ast::ParamValue, ParseError> {
        match self.peek_token_cloned() {
            Some(Token {
                kind: TokenKind::IntNumber(value),
                ..
            }) => {
                self.ptr += 1;
                Ok(ast::ParamValue::Int(value))
            }
            Some(Token {
                kind: TokenKind::Str(value),
                ..
            }) => {
                self.ptr += 1;
                Ok(ast::ParamValue::Str(value.clone()))
            }
            Some(Token {
                kind: TokenKind::Keyword(value),
                ..
            }) => {
                self.ptr += 1;
                Ok(ast::ParamValue::Keyword(value.clone()))
            }
            _ => Err(self.parse_error(
                ParseErrorScope::ArgListValue,
                "Unexpected arglist value type",
            )),
        }
    }

    fn parse_fields_subobject(&mut self) -> Result<Vec<ast::Field>, ParseError> {
        if self.is_next_token_kind(TokenKind::OpenBrace) {
            self.ptr += 1;
        } else {
            return Err(self.parse_error(ParseErrorScope::Query, "Missing opening brace"));
        }

        let mut fields = vec![];
        loop {
            if self.peek_token().is_none() {
                return Err(self.parse_error(ParseErrorScope::Query, "Missing closing brace"));
            }

            if self.is_next_token_kind(TokenKind::CloseBrace) {
                self.ptr += 1;
                break;
            }

            fields.push(self.parse_field()?);
        }

        Ok(fields)
    }

    fn parse_error(&self, scope: ParseErrorScope, message: &str) -> ParseError {
        ParseError {
            token: self.peek_token_cloned(),
            scope,
            message: message.into(),
        }
    }

    fn is_next_token_kind(&self, expected: TokenKind) -> bool {
        if let Some(Token { kind, .. }) = self.peek_token() {
            kind == &expected
        } else {
            false
        }
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

    fn peek_keyword(&self) -> Option<&String> {
        if let Some(Token {
            kind: TokenKind::Keyword(keyword),
            ..
        }) = self.peek_token()
        {
            Some(keyword)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{self, Root},
        tokenizer::Tokenizer,
    };

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

    #[test]
    fn test_arglist() {
        let Root::Query(query) = parse("{ user(id: \"gid://user/1\", order: ASC) }").unwrap();

        assert_eq!(1, query.fields.len());

        assert_eq!(
            ast::ParamKeyValuePair {
                key: "id".into(),
                value: ast::ParamValue::Str("gid://user/1".into()),
            },
            query.fields[0].arglist.as_ref().unwrap().params[0]
        );
    }

    fn parse(raw: &str) -> Result<Root, ParseError> {
        let tokens = Tokenizer::tokenize(raw, false);
        let parser = Parser::new(tokens);
        parser.parse()
    }
}
