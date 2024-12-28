#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
    pub len: usize,
    pub original: String,
}

impl Token {
    fn new(kind: TokenKind, pos: usize, len: usize, original: String) -> Token {
        Token {
            kind,
            pos,
            len,
            original,
        }
    }

    // The position after the token (aka non inclusive).
    pub fn end_pos(&self) -> usize {
        self.pos + self.len
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenKind {
    OpenBrace,    // {
    CloseBrace,   // }
    OpenParen,    // (
    CloseParen,   // )
    OpenBracket,  // [
    CloseBracket, // ]
    Colon,
    Comma,
    IntNumber(i32),
    Keyword(String),
    Str(String),
    Whitespace(String),
    LineBreak,
    Invalid(String),
}

impl TokenKind {
    pub fn vt100_color_code(&self) -> u8 {
        match self {
            TokenKind::CloseBrace | TokenKind::OpenBrace => 92,
            TokenKind::CloseParen | TokenKind::OpenParen => 96,
            TokenKind::Colon | TokenKind::Comma => 97,
            TokenKind::Keyword(_) => 93,
            TokenKind::IntNumber(_) => 95,
            TokenKind::Str(_) => 94,
            TokenKind::Invalid(_) => 91,
            _ => 0,
        }
    }
}

pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize_lines(source: &Vec<String>, record_whitespace: bool) -> Vec<Token> {
        Tokenizer::tokenize(&source.clone().join("\n"), record_whitespace)
    }

    pub fn tokenize(source: &str, record_whitespace: bool) -> Vec<Token> {
        let mut tokens = vec![];

        let chars = source.chars().collect::<Vec<_>>();
        let mut pos = 0usize;

        while pos < chars.len() {
            match chars[pos] {
                '{' => {
                    tokens.push(Token::new(TokenKind::OpenBrace, pos, 1, "{".into()));
                    pos += 1;
                }
                '}' => {
                    tokens.push(Token::new(TokenKind::CloseBrace, pos, 1, "}".into()));
                    pos += 1;
                }
                '(' => {
                    tokens.push(Token::new(TokenKind::OpenParen, pos, 1, "(".into()));
                    pos += 1;
                }
                ')' => {
                    tokens.push(Token::new(TokenKind::CloseParen, pos, 1, ")".into()));
                    pos += 1;
                }
                '[' => {
                    tokens.push(Token::new(TokenKind::OpenBracket, pos, 1, "[".into()));
                    pos += 1;
                }
                ']' => {
                    tokens.push(Token::new(TokenKind::CloseBracket, pos, 1, "]".into()));
                    pos += 1;
                }
                ':' => {
                    tokens.push(Token::new(TokenKind::Colon, pos, 1, ":".into()));
                    pos += 1;
                }
                ',' => {
                    tokens.push(Token::new(TokenKind::Comma, pos, 1, ",".into()));
                    pos += 1;
                }
                '\n' => {
                    if record_whitespace {
                        tokens.push(Token::new(TokenKind::LineBreak, pos, 1, "\n".into()));
                    }
                    pos += 1;
                }
                'a'..='z' | 'A'..='Z' => {
                    tokens.push(Tokenizer::consume_keyword(&chars, &mut pos));
                }
                ' ' | '\t' => {
                    if record_whitespace {
                        tokens.push(Tokenizer::consume_whitespace(&chars, &mut pos));
                    } else {
                        pos += 1;
                    }
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
            if !chars[*pos].is_ascii_alphabetic() && chars[*pos] != '_' {
                break;
            }

            fragment.push(chars[*pos]);
            *pos += 1;
        }
        let fragment_len = fragment.len();

        Token::new(
            TokenKind::Keyword(fragment.clone()),
            *pos - fragment_len,
            fragment_len,
            fragment,
        )
    }

    fn consume_whitespace(chars: &Vec<char>, pos: &mut usize) -> Token {
        let mut fragment = String::new();

        while *pos < chars.len() {
            if !(chars[*pos] == ' ' || chars[*pos] == '\t') {
                break;
            }

            fragment.push(chars[*pos]);
            *pos += 1;
        }

        let fragment_len = fragment.len();

        Token::new(
            TokenKind::Whitespace(fragment.clone()),
            *pos - fragment_len,
            fragment_len,
            fragment,
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
            fragment,
        )
    }

    fn consume_string(chars: &Vec<char>, pos: &mut usize) -> Token {
        let mut original = String::new();
        let mut fragment = String::new();

        original.push(chars[*pos]);

        *pos += 1; // Quote.

        let mut has_closing_quote = false;

        while *pos < chars.len() {
            if chars[*pos] == '"' {
                has_closing_quote = true;
                break;
            }
            if chars[*pos] == '\n' {
                break;
            }

            fragment.push(chars[*pos]);
            original.push(chars[*pos]);
            *pos += 1;
        }

        if has_closing_quote {
            original.push(chars[*pos]);
            *pos += 1; // Closing quote;
            let fragment_len = fragment.len();

            Token::new(
                TokenKind::Str(fragment.clone()),
                *pos - fragment_len - 2,
                fragment_len + 2,
                original,
            )
        } else {
            let original_len = original.len();
            Token::new(
                TokenKind::Invalid("Invalid string token".into()),
                *pos - original_len,
                original_len,
                original,
            )
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::{Token, TokenKind};

    use super::Tokenizer;

    #[test]
    fn test_empty() {
        let tokens = Tokenizer::tokenize("", false);
        assert_eq!(0, tokens.len());
    }

    #[test]
    fn test_braces() {
        let tokens = Tokenizer::tokenize("{}", false);
        assert_eq!(2, tokens.len());
        assert_eq!(TokenKind::OpenBrace, tokens[0].kind);
        assert_eq!(TokenKind::CloseBrace, tokens[1].kind);
    }

    #[test]
    fn test_brackets() {
        let tokens = Tokenizer::tokenize("[]", false);
        assert_eq!(2, tokens.len());
        assert_eq!(TokenKind::OpenBracket, tokens[0].kind);
        assert_eq!(TokenKind::CloseBracket, tokens[1].kind);
    }

    #[test]
    fn test_keyword() {
        let tokens = Tokenizer::tokenize("{user}", false);
        assert_eq!(3, tokens.len());
        assert_eq!(TokenKind::OpenBrace, tokens[0].kind);
        assert_eq!(TokenKind::Keyword("user".into()), tokens[1].kind);
        assert_eq!(TokenKind::CloseBrace, tokens[2].kind);
    }

    #[test]
    fn test_keyword_with_whitespaces() {
        let tokens = Tokenizer::tokenize("\t {     user  }    ", false);
        assert_eq!(3, tokens.len());
        assert_eq!(TokenKind::OpenBrace, tokens[0].kind);
        assert_eq!(TokenKind::Keyword("user".into()), tokens[1].kind);
        assert_eq!(TokenKind::CloseBrace, tokens[2].kind);
    }

    #[test]
    fn test_paren_and_args() {
        let tokens = Tokenizer::tokenize("{ users(first: 1) }", false);
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
        let tokens = Tokenizer::tokenize("{ user(id: \"gid://user/1\") }", false);
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
        let tokens = Tokenizer::tokenize("   { \"hello\"\t123\n\n}", false);

        assert_eq!(4, tokens.len());

        assert_eq!(
            Token::new(TokenKind::OpenBrace, 3, 1, "{".into()),
            tokens[0]
        );
        assert_eq!(
            Token::new(TokenKind::Str("hello".into()), 5, 7, "\"hello\"".into()),
            tokens[1]
        );
        assert_eq!(
            Token::new(TokenKind::IntNumber(123), 13, 3, "123".into()),
            tokens[2]
        );
        assert_eq!(
            Token::new(TokenKind::CloseBrace, 18, 1, "}".into()),
            tokens[3]
        );
    }

    #[test]
    fn test_comma() {
        let tokens = Tokenizer::tokenize("foo: \"bar\", bar: 123", false);

        assert_eq!(7, tokens.len());
        assert_eq!(TokenKind::Comma, tokens[3].kind);
    }

    #[test]
    fn test_invalid_string() {
        let tokens = Tokenizer::tokenize("\"hello  ", true);

        assert_eq!(1, tokens.len());
        assert_eq!(
            TokenKind::Invalid("Invalid string token".into()),
            tokens[0].kind
        );
        assert_eq!("\"hello  ".to_string(), tokens[0].original);
    }

    #[test]
    fn test_capital_and_snake_keyword() {
        let tokens = Tokenizer::tokenize("HELLO_WORLD", false);

        assert_eq!(1, tokens.len());
        assert_eq!(TokenKind::Keyword("HELLO_WORLD".into()), tokens[0].kind);
    }

    #[test]
    fn test_line_break() {
        let tokens = Tokenizer::tokenize("  \n  ", true);
        assert_eq!(3, tokens.len());
        assert_eq!(TokenKind::LineBreak, tokens[1].kind);
    }

    #[test]
    fn test_string_does_not_consume_newline() {
        let tokens = Tokenizer::tokenize("abc\"\ndef", true);
        assert_eq!(4, tokens.len());
        assert_eq!(TokenKind::LineBreak, tokens[2].kind);
    }
}
