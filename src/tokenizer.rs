use std::ops::RangeInclusive;

pub const COLOR_INVALID: u8 = 91;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
    pub len: usize,
    pub original: String,
}

impl Token {
    pub fn new(kind: TokenKind, pos: usize, len: usize, original: String) -> Token {
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

    pub fn range_inclusive(&self) -> RangeInclusive<usize> {
        self.pos..=self.pos + self.len
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
    Colon,        // :
    Comma,        // ,
    Ellipsis,     // ...
    LineBreak,
    Number(String),
    Keyword(String),
    Str(String),
    Whitespace(String),
    Invalid(String),
    MagicValue(String),
    Comment,
}

impl TokenKind {
    pub fn vt100_color_code(&self) -> u8 {
        match self {
            TokenKind::CloseBrace | TokenKind::OpenBrace => 92,
            TokenKind::CloseParen | TokenKind::OpenParen => 96,
            TokenKind::Colon | TokenKind::Comma => 97,
            TokenKind::Keyword(_) => 93,
            TokenKind::Number(_) => 95,
            TokenKind::Str(_) => 94,
            TokenKind::Invalid(_) => COLOR_INVALID,
            TokenKind::MagicValue(_) => 44,
            TokenKind::Comment => 90,
            _ => 0,
        }
    }
}

pub struct Tokenizer;

impl Tokenizer {
    pub fn tokenize_lines(source: &Vec<String>, record_whitespace: bool) -> Vec<Token> {
        Tokenizer::tokenize(&source.join("\n"), record_whitespace)
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
                ' ' | '\t' => {
                    if record_whitespace {
                        tokens.push(Tokenizer::consume_whitespace(&chars, &mut pos));
                    } else {
                        pos += 1;
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    tokens.push(Tokenizer::consume_keyword(&chars, &mut pos))
                }
                '0'..='9' | '-' => tokens.push(Tokenizer::consume_number(&chars, &mut pos)),
                '"' => tokens.push(Tokenizer::consume_string(&chars, &mut pos)),
                '<' => tokens.push(Tokenizer::consume_magic_value(&chars, &mut pos)),
                '.' => tokens.push(Tokenizer::consume_ellipsis(&chars, &mut pos)),
                '/' => match Tokenizer::consume_comment(&chars, &mut pos) {
                    token @ Token {
                        kind: TokenKind::Comment,
                        ..
                    } => {
                        if record_whitespace {
                            tokens.push(token);
                        }
                    }
                    token @ _ => tokens.push(token),
                },
                _ => {
                    tokens.push(Token::new(
                        TokenKind::Invalid("Invalid character".into()),
                        pos,
                        1,
                        chars[pos].to_string(),
                    ));
                    pos += 1;
                }
            }
        }

        tokens
    }

    fn consume_ellipsis(chars: &Vec<char>, pos: &mut usize) -> Token {
        if chars.len() < *pos + 3 {
            *pos += 1;
            return Token::new(
                TokenKind::Invalid("Invalid ellipsis lenght".to_string()),
                *pos - 1,
                1,
                chars[*pos].to_string(),
            );
        }

        if chars[*pos] != '.' || chars[*pos + 1] != '.' || chars[*pos + 2] != '.' {
            *pos += 1;
            return Token::new(
                TokenKind::Invalid("Invalid ellipsis chars".to_string()),
                *pos - 1,
                1,
                chars[*pos].to_string(),
            );
        }

        *pos += 3;
        Token::new(TokenKind::Ellipsis, *pos - 3, 3, "...".to_string())
    }

    fn consume_comment(chars: &Vec<char>, pos: &mut usize) -> Token {
        let pos_orig = *pos;

        if chars.len() < *pos + 2 || chars[*pos] != '/' || chars[*pos + 1] != '/' {
            *pos += 1;
            return Token::new(
                TokenKind::Invalid("Invalid comment start".to_string()),
                *pos - 1,
                1,
                chars[*pos - 1].to_string(),
            );
        }

        loop {
            if *pos >= chars.len() {
                break;
            }
            if chars[*pos] == '\n' {
                break;
            }

            *pos += 1;
        }

        Token::new(
            TokenKind::Comment,
            pos_orig,
            *pos - pos_orig,
            chars[pos_orig..*pos].iter().cloned().collect(),
        )
    }

    fn consume_keyword(chars: &Vec<char>, pos: &mut usize) -> Token {
        let mut fragment = String::new();

        while *pos < chars.len() {
            if !chars[*pos].is_ascii_alphanumeric() && chars[*pos] != '_' {
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
            if !(chars[*pos].is_ascii_digit() || chars[*pos] == '.' || chars[*pos] == '-') {
                break;
            }

            fragment.push(chars[*pos]);
            *pos += 1;
        }

        let fragment_len = fragment.len();

        Token::new(
            TokenKind::Number(fragment.clone()),
            *pos - fragment_len,
            fragment_len,
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

    fn consume_magic_value(chars: &Vec<char>, pos: &mut usize) -> Token {
        let mut original = String::new();
        let mut fragment = String::new();

        original.push(chars[*pos]);

        *pos += 1; // < sign.

        let mut has_closing_angle_quote = false;

        while *pos < chars.len() {
            if chars[*pos] == '>' {
                has_closing_angle_quote = true;
                break;
            }
            if chars[*pos] == '\n' {
                break;
            }

            fragment.push(chars[*pos]);
            original.push(chars[*pos]);
            *pos += 1;
        }

        if has_closing_angle_quote {
            original.push(chars[*pos]);
            *pos += 1; // Closing quote;
            let fragment_len = fragment.len();

            Token::new(
                TokenKind::MagicValue(fragment.clone()),
                *pos - fragment_len - 2,
                fragment_len + 2,
                original,
            )
        } else {
            let original_len = original.len();
            Token::new(
                TokenKind::Invalid("Invalid magic value token".into()),
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
        assert_eq!(TokenKind::Number("1".to_string()), tokens[5].kind);
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
            Token::new(TokenKind::Number("123".to_string()), 13, 3, "123".into()),
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

    #[test]
    fn test_float() {
        let tokens = Tokenizer::tokenize("12.12 0.23", false);
        assert_eq!(2, tokens.len());
        assert_eq!(TokenKind::Number("12.12".to_string()), tokens[0].kind);
        assert_eq!(TokenKind::Number("0.23".to_string()), tokens[1].kind);
    }

    #[test]
    fn test_negative() {
        let tokens = Tokenizer::tokenize("-0.12 -3", false);
        assert_eq!(2, tokens.len());
        assert_eq!(TokenKind::Number("-0.12".to_string()), tokens[0].kind);
        assert_eq!(TokenKind::Number("-3".to_string()), tokens[1].kind);
    }

    #[test]
    fn test_magic_value() {
        let tokens = Tokenizer::tokenize("input(name: <command:params>)", false);
        assert_eq!(6, tokens.len());
        assert_eq!(
            TokenKind::MagicValue("command:params".to_string()),
            tokens[4].kind
        );
    }

    #[test]
    fn test_ellipsis() {
        let tokens = Tokenizer::tokenize("... on {}", false);
        assert_eq!(4, tokens.len());
        assert_eq!(TokenKind::Ellipsis, tokens[0].kind);
    }

    #[test]
    fn test_comment() {
        let tokens = Tokenizer::tokenize("foo\n// comment\nbar", true);
        assert_eq!(5, tokens.len());
        assert_eq!(TokenKind::Comment, tokens[2].kind);
    }

    #[test]
    fn test_comment_is_skipped_when_no_whitespace() {
        let tokens = Tokenizer::tokenize("foo\n// comment\nbar", false);
        assert_eq!(2, tokens.len());
        assert_eq!(TokenKind::Keyword("foo".to_string()), tokens[0].kind);
        assert_eq!(TokenKind::Keyword("bar".to_string()), tokens[1].kind);
    }
}
