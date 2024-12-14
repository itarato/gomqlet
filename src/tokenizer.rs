use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    pos: usize,
    len: usize,
}

impl Token {
    fn new(kind: TokenKind, pos: usize, len: usize) -> Token {
        Token { kind, pos, len }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum TokenKind {
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Colon,
    IntNumber(i32),
    Keyword(String),
    Str(String),
}

pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize(source: &str) -> Vec<Token> {
        let mut tokens = vec![];

        let chars = source.chars().collect::<Vec<_>>();
        let mut pos = 0usize;

        while pos < chars.len() {
            match chars[pos] {
                '{' => {
                    tokens.push(Token::new(TokenKind::OpenBrace, pos, 1));
                    pos += 1;
                }
                '}' => {
                    tokens.push(Token::new(TokenKind::CloseBrace, pos, 1));
                    pos += 1;
                }
                '(' => {
                    tokens.push(Token::new(TokenKind::OpenParen, pos, 1));
                    pos += 1;
                }
                ')' => {
                    tokens.push(Token::new(TokenKind::CloseParen, pos, 1));
                    pos += 1;
                }
                ':' => {
                    tokens.push(Token::new(TokenKind::Colon, pos, 1));
                    pos += 1;
                }
                'a'..='z' => {
                    tokens.push(Tokenizer::consume_keyword(&chars, &mut pos));
                }
                ' ' | '\t' | '\r' | '\n' => {
                    pos += 1;
                }
                '0'..='9' => {
                    tokens.push(Tokenizer::consume_number(&chars, &mut pos));
                }
                '"' => {
                    tokens.push(Tokenizer::consume_string(&chars, &mut pos));
                }
                _ => {
                    error!("Tokenizer error: unexpected char: {}", chars[pos]);
                    pos += 1;
                }
            }
        }

        tokens
    }

    fn consume_keyword(chars: &Vec<char>, pos: &mut usize) -> Token {
        let mut fragment = String::new();

        while *pos < chars.len() {
            if !chars[*pos].is_ascii_alphabetic() {
                break;
            }

            fragment.push(chars[*pos]);
            *pos += 1;
        }
        let fragment_len = fragment.len();

        Token::new(
            TokenKind::Keyword(fragment),
            *pos - fragment_len,
            fragment_len,
        )
    }

    fn consume_number(chars: &Vec<char>, pos: &mut usize) -> Token {
        let mut fragment = String::new();

        while *pos < chars.len() {
            if !chars[*pos].is_ascii_digit() {
                break;
            }

            fragment.push(chars[*pos]);
            *pos += 1;
        }

        Token::new(
            TokenKind::IntNumber(i32::from_str_radix(&fragment, 10).expect("Invalid number")),
            *pos - fragment.len(),
            fragment.len(),
        )
    }

    fn consume_string(chars: &Vec<char>, pos: &mut usize) -> Token {
        let mut fragment = String::new();

        *pos += 1; // Quote.

        while *pos < chars.len() {
            if chars[*pos] == '"' {
                break;
            }

            fragment.push(chars[*pos]);
            *pos += 1;
        }

        *pos += 1; // Closing quote;
        let fragment_len = fragment.len();

        Token::new(
            TokenKind::Str(fragment),
            *pos - fragment_len - 2,
            fragment_len + 2,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::{Token, TokenKind};

    use super::Tokenizer;

    #[test]
    fn test_empty() {
        let tokens = Tokenizer::tokenize("");
        assert_eq!(0, tokens.len());
    }

    #[test]
    fn test_braces() {
        let tokens = Tokenizer::tokenize("{}");
        assert_eq!(2, tokens.len());
        assert_eq!(TokenKind::OpenBrace, tokens[0].kind);
        assert_eq!(TokenKind::CloseBrace, tokens[1].kind);
    }

    #[test]
    fn test_keyword() {
        let tokens = Tokenizer::tokenize("{user}");
        assert_eq!(3, tokens.len());
        assert_eq!(TokenKind::OpenBrace, tokens[0].kind);
        assert_eq!(TokenKind::Keyword("user".into()), tokens[1].kind);
        assert_eq!(TokenKind::CloseBrace, tokens[2].kind);
    }

    #[test]
    fn test_keyword_with_whitespaces() {
        let tokens = Tokenizer::tokenize("\t {     \n\nuser\r\n  }    ");
        assert_eq!(3, tokens.len());
        assert_eq!(TokenKind::OpenBrace, tokens[0].kind);
        assert_eq!(TokenKind::Keyword("user".into()), tokens[1].kind);
        assert_eq!(TokenKind::CloseBrace, tokens[2].kind);
    }

    #[test]
    fn test_paren_and_args() {
        let tokens = Tokenizer::tokenize("{ users(first: 1) }");
        assert_eq!(8, tokens.len());
        assert_eq!(TokenKind::OpenBrace, tokens[0].kind);
        assert_eq!(TokenKind::Keyword("users".into()), tokens[1].kind);
        assert_eq!(TokenKind::OpenParen, tokens[2].kind);
        assert_eq!(TokenKind::Keyword("first".into()), tokens[3].kind);
        assert_eq!(TokenKind::Colon, tokens[4].kind);
        assert_eq!(TokenKind::IntNumber(1), tokens[5].kind);
        assert_eq!(TokenKind::CloseParen, tokens[6].kind);
        assert_eq!(TokenKind::CloseBrace, tokens[7].kind);
    }

    #[test]
    fn test_string() {
        let tokens = Tokenizer::tokenize("{ user(id: \"gid://user/1\") }");
        assert_eq!(8, tokens.len());
        assert_eq!(TokenKind::OpenBrace, tokens[0].kind);
        assert_eq!(TokenKind::Keyword("user".into()), tokens[1].kind);
        assert_eq!(TokenKind::OpenParen, tokens[2].kind);
        assert_eq!(TokenKind::Keyword("id".into()), tokens[3].kind);
        assert_eq!(TokenKind::Colon, tokens[4].kind);
        assert_eq!(TokenKind::Str("gid://user/1".into()), tokens[5].kind);
        assert_eq!(TokenKind::CloseParen, tokens[6].kind);
        assert_eq!(TokenKind::CloseBrace, tokens[7].kind);
    }

    #[test]
    fn test_pos() {
        let tokens = Tokenizer::tokenize("   { \"hello\"\t123\n\n}");

        assert_eq!(4, tokens.len());

        assert_eq!(Token::new(TokenKind::OpenBrace, 3, 1), tokens[0]);
        assert_eq!(Token::new(TokenKind::Str("hello".into()), 5, 7), tokens[1]);
        assert_eq!(Token::new(TokenKind::IntNumber(123), 13, 3), tokens[2]);
        assert_eq!(Token::new(TokenKind::CloseBrace, 18, 1), tokens[3]);
    }
}
