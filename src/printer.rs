use std::io::{self, Write};

use crate::{
    analyzer::AnalyzerResult,
    terminal_handler::TerminalHandler,
    tokenizer::{Token, TokenKind},
    util::CoordUsize,
};

pub struct Printer;

impl Printer {
    pub fn new() -> Printer {
        Printer
    }

    pub fn print(&self, tokens: Vec<Token>, cursor: CoordUsize, analyzer_result: AnalyzerResult) {
        let mut buf: String = String::new();
        TerminalHandler::append_hide_cursor(&mut buf);
        TerminalHandler::append_clear_screen(&mut buf);
        TerminalHandler::append_cursor_location(&mut buf, 0, 0);

        let output = self.colorize(tokens);
        buf.push_str(&output);

        buf.push_str("\n\r---\n\r");
        buf.push_str(&format!("{:?}", analyzer_result));

        TerminalHandler::append_cursor_location(&mut buf, cursor.x, cursor.y);
        TerminalHandler::append_show_cursor(&mut buf);

        io::stdout()
            .write_all(buf.as_bytes())
            .expect("Failed writing output");

        io::stdout().flush().expect("Cannot flush STDOUT");
    }

    fn colorize(&self, tokens: Vec<Token>) -> String {
        tokens
            .into_iter()
            .map(|token| match token.kind {
                TokenKind::LineBreak => "\r\n".into(),
                _ => format!(
                    "\x1B[{}m{}\x1B[0m",
                    token.kind.vt100_color_code(),
                    token.original
                ),
            })
            .collect::<String>()
    }
}
