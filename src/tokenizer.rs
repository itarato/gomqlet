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
                    let (token, keyword_len) = Tokenizer::consume_keyword(source_iter.clone());
                    tokens.push(token);
                    source_iter.skip(keyword_len);
                }
                _ => unimplemented!(),
            }
        }

        tokens
    }

    fn consume_keyword(source_iter: Peekable<Chars<'_>>) -> (Token, usize) {
        let fragment = source_iter
            .take_while(|ch| ch.is_ascii_alphabetic())
            .collect::<String>();

        let len = fragment.len();
        (Token::Keyword(fragment), len)
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
        let tokens = Tokenizer::tokenize("{user}{}");
        dbg!(&tokens);
        assert_eq!(3, tokens.len());
        assert_eq!(Token::OpenBrace, tokens[0]);
        assert_eq!(Token::Keyword("user".into()), tokens[1]);
        assert_eq!(Token::CloseBrace, tokens[2]);
    }
}
