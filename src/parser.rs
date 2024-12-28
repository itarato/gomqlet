use crate::ast::{self, FieldList};
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
            Ok(ast::Root::Mutation(self.parse_mutation()?))
        } else {
            Ok(ast::Root::Query(self.parse_query()?))
        }
    }

    fn parse_query(&mut self) -> Result<ast::Query, ParseError> {
        let start_pos = if let Some(token) = self.peek_token() {
            token.pos
        } else {
            return Err(self.parse_error(ParseErrorScope::Query, "Empty query"));
        };

        if self.is_next_keyword("query") {
            self.ptr += 1;
        }

        // TODO variables.

        let field_list = self.parse_fields_subobject()?;

        Ok(ast::Query {
            start_pos,
            end_pos: field_list.end_pos,
            field_list,
        })
    }

    fn parse_mutation(&mut self) -> Result<ast::Mutation, ParseError> {
        if !self.is_next_keyword("mutation") {
            return Err(self.parse_error(ParseErrorScope::Query, "Empty mutation"));
        }
        let start_pos = self.peek_token().unwrap().pos;

        self.ptr += 1;

        // TODO variables.

        let field_list = self.parse_fields_subobject()?;

        Ok(ast::Mutation {
            start_pos,
            end_pos: field_list.end_pos,
            field_list,
        })
    }

    fn parse_field(&mut self) -> Result<ast::Field, ParseError> {
        let name_token = if let Some(Token {
            kind: TokenKind::Keyword(_),
            ..
        }) = self.peek_token()
        {
            self.peek_token_cloned().unwrap()
        } else {
            return Err(self.parse_error(ParseErrorScope::Field, "Missing field name"));
        };
        self.ptr += 1;

        let mut end_pos = name_token.end_pos();

        let arglist = if self.is_next_token_kind(TokenKind::OpenParen) {
            let arglist = self.parse_arglist()?;
            end_pos = arglist.end_pos;
            Some(arglist)
        } else {
            None
        };

        let field_list = if self.is_next_token_kind(TokenKind::OpenBrace) {
            let field_list = self.parse_fields_subobject()?;
            end_pos = field_list.end_pos;
            Some(field_list)
        } else {
            None
        };

        Ok(ast::Field {
            start_pos: name_token.pos,
            end_pos,
            name: name_token,
            arglist,
            field_list,
        })
    }

    fn parse_arglist(&mut self) -> Result<ast::ArgList, ParseError> {
        if !self.is_next_token_kind(TokenKind::OpenParen) {
            return Err(self.parse_error(ParseErrorScope::ArgList, "Missing open paren"));
        }
        let start_pos = self.peek_token().unwrap().pos;
        self.ptr += 1;

        let mut params = vec![];
        loop {
            if self.is_next_token_kind(TokenKind::CloseParen) {
                break;
            }

            let key = if let Some(Token {
                kind: TokenKind::Keyword(_),
                ..
            }) = self.peek_token()
            {
                self.peek_token_cloned().unwrap()
            } else {
                return Err(self.parse_error(ParseErrorScope::ArgList, "Missing keyword"));
            };
            self.ptr += 1;

            if self.is_next_token_kind(TokenKind::Colon) {
                self.ptr += 1;
            } // Else is omitted due to error handling (assume it's missing for now).

            let value = self.parse_arglist_value()?;

            params.push(ast::ParamKeyValuePair {
                start_pos: key.pos,
                end_pos: value.end_pos(),
                key,
                value,
            });

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
        let end_pos = self.peek_token().unwrap().end_pos();
        self.ptr += 1;

        Ok(ast::ArgList {
            start_pos,
            end_pos,
            params,
        })
    }

    fn parse_arglist_value(&mut self) -> Result<ast::ParamValue, ParseError> {
        let token = self.peek_token_cloned();
        match token {
            Some(Token {
                kind: TokenKind::IntNumber(_),
                ..
            })
            | Some(Token {
                kind: TokenKind::Str(_),
                ..
            })
            | Some(Token {
                kind: TokenKind::Keyword(_),
                ..
            }) => {
                self.ptr += 1;
                Ok(ast::ParamValue::Simple(token.unwrap()))
            }
            // Error handling:
            Some(Token {
                kind: TokenKind::CloseParen,
                pos,
                ..
            })
            | Some(Token {
                kind: TokenKind::Comma,
                pos,
                ..
            }) => Ok(ast::ParamValue::Missing(pos)),
            _ => Err(self.parse_error(
                ParseErrorScope::ArgListValue,
                "Unexpected arglist value type",
            )),
        }
    }

    fn parse_fields_subobject(&mut self) -> Result<FieldList, ParseError> {
        let start_pos = if self.is_next_token_kind(TokenKind::OpenBrace) {
            let start_pos = self.peek_token().unwrap().pos;
            self.ptr += 1;
            start_pos
        } else {
            return Err(self.parse_error(ParseErrorScope::Query, "Missing opening brace"));
        };

        let mut fields = vec![];
        loop {
            if self.peek_token().is_none() {
                return Err(self.parse_error(ParseErrorScope::Query, "Missing closing brace"));
            }

            if self.is_next_token_kind(TokenKind::CloseBrace) {
                let end_pos = self.peek_token().unwrap().end_pos();
                self.ptr += 1;
                return Ok(ast::FieldList {
                    start_pos,
                    end_pos,
                    fields,
                });
            }

            fields.push(self.parse_field()?);
        }
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
        if let Some(keyword) = self.peek_keyword() {
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
        ast::{Query, Root},
        tokenizer::Tokenizer,
    };

    use super::Parser;

    #[test]
    fn test_empty() {
        let query = parse_query("{}");
        assert_eq!(0, query.field_list.fields.len());
    }

    #[test]
    fn test_optional_query_keywors() {
        let query = parse_query("query { user }");
        assert_eq!(1, query.field_list.fields.len());
    }

    #[test]
    fn test_plan_fields() {
        let query = parse_query("{ user company task }");
        assert_eq!(3, query.field_list.fields.len());
        assert_eq!("user".to_string(), query.field_list.fields[0].name.original);
        assert_eq!(
            "company".to_string(),
            query.field_list.fields[1].name.original
        );
        assert_eq!("task".to_string(), query.field_list.fields[2].name.original);
    }

    #[test]
    fn test_nested_fields() {
        let query = parse_query("{ user company { location size } }");
        assert_eq!(2, query.field_list.fields.len());
        assert_eq!(
            2,
            query.field_list.fields[1]
                .field_list
                .as_ref()
                .unwrap()
                .fields
                .len()
        );
    }

    #[test]
    fn test_arglist() {
        let query = parse_query("{ user(id: \"gid://user/1\", order: ASC) }");

        assert_eq!(1, query.field_list.fields.len());

        assert_eq!(
            "id".to_string(),
            query.field_list.fields[0].arglist.as_ref().unwrap().params[0]
                .key
                .original,
        );
    }

    #[test]
    fn test_arglist_with_list_param() {
        let query = parse_query("{ createUser(tags: [11, 22, 33]) }");

        assert_eq!(1, query.field_list.fields.len());
        assert_eq!(
            1,
            query.field_list.fields[0]
                .arglist
                .as_ref()
                .unwrap()
                .params
                .len()
        );
        assert_eq!(
            1,
            query.field_list.fields[0].arglist.as_ref().unwrap().params[0].value
        );
    }

    #[test]
    fn test_empty_arglist() {
        let query = parse_query("{ users() }");

        assert_eq!(1, query.field_list.fields.len());
        assert_eq!(
            0,
            query.field_list.fields[0]
                .arglist
                .as_ref()
                .unwrap()
                .params
                .len()
        );
    }

    #[test]
    fn test_arglist_without_arg_value() {
        let query = parse_query("{ users(id: ) }");

        assert_eq!(1, query.field_list.fields.len());
        assert_eq!(
            1,
            query.field_list.fields[0]
                .arglist
                .as_ref()
                .unwrap()
                .params
                .len()
        );
    }

    #[test]
    fn test_arglist_without_arg_value_and_colon() {
        let query = parse_query("{ users(id ) }");

        assert_eq!(1, query.field_list.fields.len());
        assert_eq!(
            1,
            query.field_list.fields[0]
                .arglist
                .as_ref()
                .unwrap()
                .params
                .len()
        );
    }

    fn parse_query(raw: &str) -> Query {
        let tokens = Tokenizer::tokenize(raw, false);
        let parser = Parser::new(tokens);

        match parser.parse().unwrap() {
            Root::Query(query) => query,
            _ => panic!("This must be called with a valid query"),
        }
    }
}
