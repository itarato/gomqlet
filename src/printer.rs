use std::io::{self, Write};

use crate::{
    analyzer::AnalyzerResult,
    terminal_handler::TerminalHandler,
    tokenizer::{Token, TokenKind},
    util::CoordUsize,
};

const POPUP_BAR_WIDTH_DIVIDER: usize = 3;

pub struct Printer {
    terminal_dimension: (usize, usize),
}

impl Printer {
    pub fn new() -> Printer {
        Printer {
            terminal_dimension: term_size::dimensions().unwrap(),
        }
    }

    pub fn print(&self, tokens: Vec<Token>, cursor: CoordUsize, analyzer_result: AnalyzerResult) {
        let mut buf: String = String::new();
        TerminalHandler::append_hide_cursor(&mut buf);
        TerminalHandler::append_clear_screen(&mut buf);
        TerminalHandler::append_cursor_location(&mut buf, 0, 0);

        let output = self.colorize(tokens);
        buf.push_str(&output);

        self.print_analyzer_result(&mut buf, analyzer_result);

        TerminalHandler::append_cursor_location(&mut buf, cursor.x, cursor.y);
        TerminalHandler::append_show_cursor(&mut buf);

        io::stdout()
            .write_all(buf.as_bytes())
            .expect("Failed writing output");

        io::stdout().flush().expect("Cannot flush STDOUT");
    }

    fn print_analyzer_result(&self, buf: &mut String, analyzer_result: AnalyzerResult) {
        match analyzer_result {
            AnalyzerResult::Autocomplete(suggestions) => {
                self.print_analyzer_result_suggestions(buf, suggestions)
            }
            AnalyzerResult::DefinitionError(error) => {}
            AnalyzerResult::ParseError(error) => {}
            AnalyzerResult::Empty => {}
        }
    }

    fn print_analyzer_result_suggestions(&self, buf: &mut String, suggestions: Vec<String>) {
        let popup_width = self.terminal_dimension.0 / POPUP_BAR_WIDTH_DIVIDER;

        for i in 0..suggestions.len() {
            TerminalHandler::append_cursor_location(
                buf,
                self.terminal_dimension.0 - popup_width,
                i,
            );
            buf.push_str(&suggestions[i]);
        }
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
