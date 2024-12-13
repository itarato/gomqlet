use std::{iter::Peekable, str::Chars};

#[derive(PartialEq, Eq, Debug)]
enum Token {
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Colon,
    IntNumber(i32),
    Keyword(String),
}

pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize(source: &str) -> Vec<Token> {
        let mut tokens = vec![];

        let mut source_iter = source.chars().peekable();

        while let Some(next_ch) = source_iter.peek() {
            match *next_ch {
                '{' => {
                    tokens.push(Token::OpenBrace);
                    source_iter.next().expect("Bad iteration");
                }
                '}' => {
                    tokens.push(Token::CloseBrace);
                    source_iter.next().expect("Bad iteration");
                }
                '(' => {
                    tokens.push(Token::OpenParen);
                    source_iter.next().expect("Bad iteration");
                }
                ')' => {
                    tokens.push(Token::CloseParen);
                    source_iter.next().expect("Bad iteration");
                }
                ':' => {
                    tokens.push(Token::Colon);
                    source_iter.next().expect("Bad iteration");
                }
                'a'..='z' => {
                    tokens.push(Tokenizer::consume_keyword(&mut source_iter));
                }
                ' ' | '\t' | '\r' | '\n' => {
                    source_iter.next().expect("Bad iteration");
                }
                '0'..='9' => {
                    tokens.push(Tokenizer::consume_number(&mut source_iter));
                }
                _ => unimplemented!(),
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
}
