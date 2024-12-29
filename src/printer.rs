use std::io::{self, Write};

use crate::{
    analyzer::AnalyzerResult,
    parser::ParseError,
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
            AnalyzerResult::DefinitionError(error) => {
                self.print_analyzer_result_definition_error(buf, error)
            }

            AnalyzerResult::ParseError(error) => self.print_analyzer_result_parse_error(buf, error),
            AnalyzerResult::Empty => {}
        }
    }

    fn print_analyzer_result_suggestions(&self, buf: &mut String, suggestions: Vec<String>) {
        let popup_width = self.terminal_dimension.0 / POPUP_BAR_WIDTH_DIVIDER;
        let text_width = popup_width - 2;

        for i in 0..suggestions.len() {
            TerminalHandler::append_cursor_location(
                buf,
                self.terminal_dimension.0 - popup_width,
                i,
            );
            buf.push_str("\x1B[44m ");
            buf.push_str(&format!(
                "{: <width$}",
                &suggestions[i][0..text_width.min(suggestions[i].len())],
                width = text_width
            ));
            buf.push_str(" \x1B[0m");
        }
    }

    fn chop_string(s: String, width: usize) -> Vec<String> {
        let mut i = 0usize;
        let mut lines = vec![];

        while i < s.len() {
            let line_width = (s.len() - i).min(width);

            lines.push(format!("{: <width$}", &s[0..line_width], width = width));

            i += width;
        }

        lines
    }

    fn print_analyzer_result_parse_error(&self, buf: &mut String, error: ParseError) {
        let err_lines = Printer::chop_string(error.message, self.terminal_width());
        let token_lines = Printer::chop_string(
            format!("At token: {:?}", error.token),
            self.terminal_width(),
        );
        let scope_lines =
            Printer::chop_string(format!("Scope: {:?}", error.scope), self.terminal_width());
        let title_lines = Printer::chop_string("PARSE ERROR".to_string(), self.terminal_width());

        let lines = title_lines
            .into_iter()
            .chain(scope_lines.into_iter())
            .chain(token_lines.into_iter())
            .chain(err_lines.into_iter())
            .collect();

        self.print_error_message(buf, lines);
    }

    fn print_analyzer_result_definition_error(&self, buf: &mut String, error: String) {
        let mut lines = Printer::chop_string(error, self.terminal_width());

        lines.insert(
            0,
            format!(
                "{: <width$}",
                "ANALYZER ERROR",
                width = self.terminal_width()
            ),
        );

        self.print_error_message(buf, lines);
    }

    fn print_error_message(&self, buf: &mut String, lines: Vec<String>) {
        for i in 0..lines.len() {
            debug!("MOVE: {}", self.terminal_height() - i);
            TerminalHandler::append_cursor_location(
                buf,
                0,
                self.terminal_height() - lines.len() + i,
            );

            if i == 0 {
                buf.push_str("\x1B[1m");
            }

            buf.push_str("\x1B[48;5;52m");
            buf.push_str(&lines[i]);
            buf.push_str("\x1B[0m");
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

    fn terminal_width(&self) -> usize {
        self.terminal_dimension.0
    }

    fn terminal_height(&self) -> usize {
        self.terminal_dimension.1
    }
}
