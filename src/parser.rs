use crate::ast::{self, FieldList, ParamValue};
use crate::tokenizer::{Token, TokenKind};

#[derive(Debug)]
pub enum ParseErrorScope {
    Query,
    Field,
    ArgList,
    ArgListValue,
    ArgListValueListType,
}

#[derive(Debug)]
pub struct ParseError {
    pub token: Option<Token>,
    pub scope: ParseErrorScope,
    pub message: String,
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
        if self.peek_token().is_none() {
            return Err(self.parse_error(ParseErrorScope::Query, "Empty query"));
        }

        if self.is_next_keyword("query") {
            self.ptr += 1;
        }

        let field_list = self.parse_fields_subobject()?;

        Ok(ast::Query { field_list })
    }

    fn parse_mutation(&mut self) -> Result<ast::Mutation, ParseError> {
        if !self.is_next_keyword("mutation") {
            return Err(self.parse_error(ParseErrorScope::Query, "Empty mutation"));
        }
        self.ptr += 1;

        let field_list = self.parse_fields_subobject()?;

        Ok(ast::Mutation { field_list })
    }

    fn parse_field(&mut self) -> Result<ast::Field, ParseError> {
        match self.peek_token() {
            Some(Token {
                kind: TokenKind::Keyword(_),
                ..
            }) => self
                .parse_concrete_field()
                .map(|field| ast::Field::Concrete(field)),
            Some(Token {
                kind: TokenKind::Ellipsis,
                ..
            }) => self
                .parse_union_field()
                .map(|field| ast::Field::Union(field)),
            _ => Err(self.parse_error(ParseErrorScope::Field, "Invalid field start")),
        }
    }

    fn parse_concrete_field(&mut self) -> Result<ast::ConcreteField, ParseError> {
        let name_token = if let Some(Token {
            kind: TokenKind::Keyword(_),
            ..
        }) = self.peek_token()
        {
            self.peek_token().cloned().unwrap()
        } else {
            return Err(self.parse_error(ParseErrorScope::Field, "Missing field name"));
        };
        self.ptr += 1;

        let mut end_pos = name_token.end_pos();

        let arglist = if self.is_next_token_kind(&TokenKind::OpenParen) {
            let arglist = self.parse_arglist(&TokenKind::OpenParen, &TokenKind::CloseParen)?;
            end_pos = arglist.end_pos;
            Some(arglist)
        } else {
            None
        };

        let field_list = if self.is_next_token_kind(&TokenKind::OpenBrace) {
            let field_list = self.parse_fields_subobject()?;
            end_pos = field_list.end_pos;
            Some(field_list)
        } else {
            None
        };

        Ok(ast::ConcreteField {
            start_pos: name_token.pos,
            end_pos,
            name: name_token,
            arglist,
            field_list,
        })
    }

    fn parse_union_field(&mut self) -> Result<ast::UnionField, ParseError> {
        if !self.is_next_token_kind(&TokenKind::Ellipsis) {
            return Err(self.parse_error(
                ParseErrorScope::Field,
                "Invalid union field: missing ellipsis",
            ));
        }
        let start_pos = self.peek_token().unwrap().pos;
        self.ptr += 1;

        if !self.is_next_token_kind(&TokenKind::Keyword("on".to_string())) {
            return Err(self.parse_error(
                ParseErrorScope::Field,
                "Invalid union field: missing 'on' keyword",
            ));
        }
        self.ptr += 1;

        let type_name = if let Some(Token {
            kind: TokenKind::Keyword(_),
            ..
        }) = self.peek_token()
        {
            let type_name = self.peek_token().cloned().unwrap();
            self.ptr += 1;
            type_name
        } else {
            // Error correction: add empty keyword token to be able to autocomple on.
            let missing_token_pos = self.peek_previous_token().unwrap().end_pos() + 1; // Leave space for insertion.
            let have_space = self
                .peek_token()
                .map(|next_token| next_token.pos >= missing_token_pos)
                .unwrap_or(false);

            if have_space {
                Token::new(
                    TokenKind::Keyword("".to_string()),
                    missing_token_pos,
                    0,
                    "".to_string(),
                )
            } else {
                return Err(
                    self.parse_error(ParseErrorScope::Field, "Missing space for union type")
                );
            }
        };

        let field_list = if self.is_next_token_kind(&TokenKind::OpenBrace) {
            self.parse_fields_subobject()?
        } else {
            FieldList::new_empty(type_name.end_pos())
        };
        let end_pos = field_list.end_pos;

        Ok(ast::UnionField {
            start_pos,
            end_pos,
            type_name,
            field_list,
        })
    }

    fn parse_arglist(
        &mut self,
        open_token_kind: &TokenKind,
        close_token_kind: &TokenKind,
    ) -> Result<ast::ArgList, ParseError> {
        if !self.is_next_token_kind(open_token_kind) {
            return Err(self.parse_error(ParseErrorScope::ArgList, "Missing open paren"));
        }
        let start_pos = self.peek_token().unwrap().pos;
        self.ptr += 1;

        let mut params = vec![];
        loop {
            if self.is_next_token_kind(close_token_kind) {
                break;
            }

            let key = if let Some(Token {
                kind: TokenKind::Keyword(_),
                ..
            }) = self.peek_token()
            {
                self.peek_token().cloned().unwrap()
            } else {
                return Err(self.parse_error(ParseErrorScope::ArgList, "Missing keyword"));
            };
            self.ptr += 1;

            let value = if self.is_next_token_kind(&TokenKind::Colon) {
                self.ptr += 1;
                self.parse_arglist_value(close_token_kind)?
            } else {
                // Else is omitted due to error handling (assume it's missing for now).
                ParamValue::Missing((
                    key.end_pos(),
                    self.peek_token()
                        .map(|token| token.pos)
                        .unwrap_or(key.end_pos()),
                ))
            };

            params.push(ast::ParamKeyValuePair {
                start_pos: key.pos,
                end_pos: value.end_pos(),
                key,
                value,
            });

            if self.is_next_token_kind(&TokenKind::Comma) {
                self.ptr += 1;
                continue;
            } else {
                // Error handling: assume user is typing a new key to a non-end location.
                if self.is_next_token_keyword() {
                    continue;
                }
                break;
            }
        }

        if !self.is_next_token_kind(close_token_kind) {
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

    fn parse_arglist_value(
        &mut self,
        close_token_kind: &TokenKind,
    ) -> Result<ast::ParamValue, ParseError> {
        let token = self.peek_token().cloned();
        match token {
            Some(Token {
                kind: TokenKind::Number(_),
                ..
            })
            | Some(Token {
                kind: TokenKind::Str(_),
                ..
            })
            | Some(Token {
                kind: TokenKind::Keyword(_),
                ..
            })
            | Some(Token {
                kind: TokenKind::MagicValue(_),
                ..
            }) => {
                self.ptr += 1;
                Ok(ast::ParamValue::Simple(token.unwrap()))
            }
            Some(Token {
                kind: TokenKind::OpenBracket,
                ..
            }) => Ok(ast::ParamValue::List(
                self.parse_arglist_value_list_type(close_token_kind)?,
            )),
            Some(Token {
                kind: TokenKind::OpenBrace,
                ..
            }) => Ok(ast::ParamValue::Object(
                self.parse_arglist(&TokenKind::OpenBrace, &TokenKind::CloseBrace)?,
            )),
            // Error handling:
            Some(Token { ref kind, pos, .. }) => {
                if kind == close_token_kind || kind == &TokenKind::Comma {
                    let start_pos = self
                        .peek_previous_token()
                        .map(|token| token.end_pos())
                        .unwrap_or(0);
                    Ok(ast::ParamValue::Missing((start_pos, pos)))
                } else {
                    Err(self.parse_error(
                        ParseErrorScope::ArgListValue,
                        "Unexpected arglist value type",
                    ))
                }
            }
            None => Err(self.parse_error(
                ParseErrorScope::ArgListValue,
                "Unexpected lack of arglist value",
            )),
        }
    }

    fn parse_arglist_value_list_type(
        &mut self,
        close_token_kind: &TokenKind,
    ) -> Result<ast::ListParamValue, ParseError> {
        if !self.is_next_token_kind(&TokenKind::OpenBracket) {
            return Err(self.parse_error(
                ParseErrorScope::ArgListValueListType,
                "Expected list opening backet for list param value",
            ));
        }
        let start_pos = self.peek_token().unwrap().pos;
        self.ptr += 1;

        let mut elems = vec![];
        loop {
            if self.is_next_token_kind(&TokenKind::CloseBracket) {
                break;
            }

            elems.push(self.parse_arglist_value(close_token_kind)?);

            if !self.is_next_token_kind(&TokenKind::Comma) {
                break;
            }
            self.ptr += 1;
        }

        if !self.is_next_token_kind(&TokenKind::CloseBracket) {
            return Err(self.parse_error(
                ParseErrorScope::ArgListValueListType,
                "Expected list closing backet for list param value",
            ));
        }
        let end_pos = self.peek_token().unwrap().pos;
        self.ptr += 1;

        Ok(ast::ListParamValue {
            start_pos,
            end_pos,
            elems,
        })
    }

    fn parse_fields_subobject(&mut self) -> Result<FieldList, ParseError> {
        let start_pos = if self.is_next_token_kind(&TokenKind::OpenBrace) {
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

            if self.is_next_token_kind(&TokenKind::CloseBrace) {
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
            token: self.peek_token().cloned(),
            scope,
            message: message.into(),
        }
    }

    fn is_next_token_kind(&self, expected: &TokenKind) -> bool {
        if let Some(Token { kind, .. }) = self.peek_token() {
            kind == expected
        } else {
            false
        }
    }

    fn peek_previous_token(&self) -> Option<&Token> {
        if self.ptr == 0 {
            None
        } else {
            self.tokens.get(self.ptr - 1)
        }
    }

    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.ptr)
    }

    fn is_next_keyword(&self, value: &str) -> bool {
        if let Some(keyword) = self.peek_keyword() {
            if keyword == value {
                return true;
            }
        }

        false
    }

    fn is_next_token_keyword(&self) -> bool {
        if let Some(Token {
            kind: TokenKind::Keyword(_),
            ..
        }) = self.peek_token()
        {
            true
        } else {
            false
        }
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
        ast::{Mutation, Query, Root},
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
        assert_eq!(
            "user".to_string(),
            query.field_list.fields[0].as_concrete_field().name.original
        );
        assert_eq!(
            "company".to_string(),
            query.field_list.fields[1].as_concrete_field().name.original
        );
        assert_eq!(
            "task".to_string(),
            query.field_list.fields[2].as_concrete_field().name.original
        );
    }

    #[test]
    fn test_nested_fields() {
        let query = parse_query("{ user company { location size } }");
        assert_eq!(2, query.field_list.fields.len());
        assert_eq!(
            2,
            query.field_list.fields[1]
                .as_concrete_field()
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
            query.field_list.fields[0]
                .as_concrete_field()
                .arglist
                .as_ref()
                .unwrap()
                .params[0]
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
                .as_concrete_field()
                .arglist
                .as_ref()
                .unwrap()
                .params
                .len()
        );
        assert_eq!(
            3,
            query.field_list.fields[0]
                .as_concrete_field()
                .arglist
                .as_ref()
                .unwrap()
                .params[0]
                .value
                .as_list()
                .elems
                .len(),
        );
        assert_eq!(
            "11".to_string(),
            query.field_list.fields[0]
                .as_concrete_field()
                .arglist
                .as_ref()
                .unwrap()
                .params[0]
                .value
                .as_list()
                .elems[0]
                .as_simple()
                .original,
        );
    }

    #[test]
    fn test_empty_arglist() {
        let query = parse_query("{ users() }");

        assert_eq!(1, query.field_list.fields.len());
        assert_eq!(
            0,
            query.field_list.fields[0]
                .as_concrete_field()
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
                .as_concrete_field()
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
                .as_concrete_field()
                .arglist
                .as_ref()
                .unwrap()
                .params
                .len()
        );
    }

    #[test]
    fn test_arglist_with_object() {
        let mutation = parse_mutation(
            "mutation { createUser(address: { city: \"London\", codes: [1, 2] }) { id } }",
        );

        assert_eq!(1, mutation.field_list.fields.len());

        let args = mutation.field_list.fields[0]
            .as_concrete_field()
            .arglist
            .as_ref()
            .unwrap();
        assert_eq!(1, args.params.len());

        let first_arg_value = args.params.get(0).unwrap().value.as_object();
        assert_eq!(2, first_arg_value.params.len());
        assert_eq!("city".to_string(), first_arg_value.params[0].key.original);
        assert_eq!(
            "\"London\"".to_string(),
            first_arg_value.params[0].value.as_simple().original
        );

        assert_eq!(2, first_arg_value.params[1].value.as_list().elems.len());
    }

    #[test]
    fn test_union() {
        let query = parse_query("{ user { ... on Corporate { boss } } }");

        assert_eq!(1, query.field_list.fields.len());
        assert_eq!(
            1,
            query.field_list.fields[0]
                .as_concrete_field()
                .field_list
                .as_ref()
                .unwrap()
                .fields
                .len()
        );
        assert_eq!(
            "Corporate".to_string(),
            query.field_list.fields[0]
                .as_concrete_field()
                .field_list
                .as_ref()
                .unwrap()
                .fields[0]
                .as_union_field()
                .type_name
                .original
        );

        assert_eq!(
            9,
            query.field_list.fields[0]
                .as_concrete_field()
                .field_list
                .as_ref()
                .unwrap()
                .fields[0]
                .as_union_field()
                .start_pos
        );

        assert_eq!(
            34,
            query.field_list.fields[0]
                .as_concrete_field()
                .field_list
                .as_ref()
                .unwrap()
                .fields[0]
                .as_union_field()
                .end_pos
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

    fn parse_mutation(raw: &str) -> Mutation {
        let tokens = Tokenizer::tokenize(raw, false);
        let parser = Parser::new(tokens);

        match parser.parse().unwrap() {
            Root::Mutation(mutation) => mutation,
            _ => panic!("This must be called with a valid mutation"),
        }
    }
}
