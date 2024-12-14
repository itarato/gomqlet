use std::{iter::Peekable, str::Chars};

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
                    tokens.push(Tokenizer::consume_keyword(chars, &mut pos));
                }
                ' ' | '\t' | '\r' | '\n' => {
                    pos += 1;
                }
                '0'..='9' => {
                    tokens.push(Tokenizer::consume_number(chars, &mut pos));
                }
                '"' => {
                    tokens.push(Tokenizer::consume_string(chars, &mut pos));
                }
                _ => {
                    error!("Tokenizer error: unexpected char: {}", chars[pos]);
                    pos += 1;
                }
            }
        }

        tokens
    }

    fn consume_keyword(source_iter: &mut Peekable<Chars<'_>>) -> Token {
        let mut fragment = String::new();

        while let Some(ch) = source_iter.peek() {
            if !ch.is_ascii_alphabetic() {
                break;
            }

            fragment.push(*ch);
            source_iter.next().unwrap();
        }

        Token::Keyword(fragment)
    }

    fn consume_number(source_iter: &mut Peekable<Chars<'_>>) -> Token {
        let mut fragment = String::new();

        while let Some(ch) = source_iter.peek() {
            if !ch.is_ascii_digit() {
                break;
            }

            fragment.push(*ch);
            source_iter.next().unwrap();
        }

        Token::IntNumber(i32::from_str_radix(&fragment, 10).expect("Invalid number"))
    }

    fn consume_string(source_iter: &mut Peekable<Chars<'_>>) -> Token {
        let mut fragment = String::new();

        source_iter.next().unwrap();

        while let Some(ch) = source_iter.next() {
            if ch == '"' {
                break;
            }

            fragment.push(ch);
        }

        Token::Str(fragment)
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::Token;

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
        assert_eq!(Token::OpenBrace, tokens[0]);
        assert_eq!(Token::CloseBrace, tokens[1]);
    }

    #[test]
    fn test_keyword() {
        let tokens = Tokenizer::tokenize("{user}");
        assert_eq!(3, tokens.len());
        assert_eq!(Token::OpenBrace, tokens[0]);
        assert_eq!(Token::Keyword("user".into()), tokens[1]);
        assert_eq!(Token::CloseBrace, tokens[2]);
    }

    #[test]
    fn test_keyword_with_whitespaces() {
        let tokens = Tokenizer::tokenize("\t {     \n\nuser\r\n  }    ");
        assert_eq!(3, tokens.len());
        assert_eq!(Token::OpenBrace, tokens[0]);
        assert_eq!(Token::Keyword("user".into()), tokens[1]);
        assert_eq!(Token::CloseBrace, tokens[2]);
    }

    #[test]
    fn test_paren_and_args() {
        let tokens = Tokenizer::tokenize("{ users(first: 1) }");
        assert_eq!(8, tokens.len());
        assert_eq!(Token::OpenBrace, tokens[0]);
        assert_eq!(Token::Keyword("users".into()), tokens[1]);
        assert_eq!(Token::OpenParen, tokens[2]);
        assert_eq!(Token::Keyword("first".into()), tokens[3]);
        assert_eq!(Token::Colon, tokens[4]);
        assert_eq!(Token::IntNumber(1), tokens[5]);
        assert_eq!(Token::CloseParen, tokens[6]);
        assert_eq!(Token::CloseBrace, tokens[7]);
    }

    #[test]
    fn test_string() {
        let tokens = Tokenizer::tokenize("{ user(id: \"gid://user/1\") }");
        assert_eq!(8, tokens.len());
        assert_eq!(Token::OpenBrace, tokens[0]);
        assert_eq!(Token::Keyword("user".into()), tokens[1]);
        assert_eq!(Token::OpenParen, tokens[2]);
        assert_eq!(Token::Keyword("id".into()), tokens[3]);
        assert_eq!(Token::Colon, tokens[4]);
        assert_eq!(Token::Str("gid://user/1".into()), tokens[5]);
        assert_eq!(Token::CloseParen, tokens[6]);
        assert_eq!(Token::CloseBrace, tokens[7]);
    }
}
