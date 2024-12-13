use std::{iter::Peekable, str::Chars};

#[derive(PartialEq, Eq, Debug)]
enum Token {
    OpenBrace,
    CloseBrace,
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
                'a'..='z' => {
                    tokens.push(Tokenizer::consume_keyword(&mut source_iter));
                }
                ' ' | '\t' | '\r' | '\n' => {
                    source_iter.next().expect("Bad iteration");
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
        dbg!(&tokens);
        assert_eq!(3, tokens.len());
        assert_eq!(Token::OpenBrace, tokens[0]);
        assert_eq!(Token::Keyword("user".into()), tokens[1]);
        assert_eq!(Token::CloseBrace, tokens[2]);
    }

    #[test]
    fn test_keyword_with_whitespaces() {
        let tokens = Tokenizer::tokenize("\t {     \n\nuser\r\n  }    ");
        dbg!(&tokens);
        assert_eq!(3, tokens.len());
        assert_eq!(Token::OpenBrace, tokens[0]);
        assert_eq!(Token::Keyword("user".into()), tokens[1]);
        assert_eq!(Token::CloseBrace, tokens[2]);
    }
}
