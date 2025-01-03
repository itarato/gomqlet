use std::io::{self, Write};

use crate::{
    analyzer::Suggestion,
    parser::ParseError,
    terminal_handler::TerminalHandler,
    tokenizer::{Token, TokenKind},
    util::{trim_coloured_string_list, CoordUsize},
};

const POPUP_BAR_WIDTH_DIVIDER: usize = 2;

pub struct EditorPrinter {
    terminal_dimension: (usize, usize),
    // Text scroll up   = positive range
    // Text scroll down = negative range
    vscroll: usize,
}

impl EditorPrinter {
    pub fn new() -> EditorPrinter {
        EditorPrinter {
            terminal_dimension: term_size::dimensions().unwrap(),
            vscroll: 0,
        }
    }

    pub fn print(
        &mut self,
        tokens: Vec<Token>,
        cursor: CoordUsize,
        suggestions: Option<Suggestion>,
        parse_error: Option<ParseError>,
        definition_error: Option<String>,
        suggestion_selection_mode: bool,
    ) {
        let mut buf: String = String::new();
        TerminalHandler::append_hide_cursor(&mut buf);
        TerminalHandler::append_clear_screen(&mut buf);

        self.print_tokens(&mut buf, tokens, cursor.y);

        if let Some(suggestions) = suggestions {
            self.print_analyzer_result_suggestions(
                &mut buf,
                suggestions,
                suggestion_selection_mode,
            );
        }

        if let Some(parse_error) = parse_error {
            self.print_analyzer_result_parse_error(&mut buf, parse_error);
        } else if let Some(definition_error) = definition_error {
            self.print_analyzer_result_definition_error(&mut buf, definition_error);
        }

        TerminalHandler::append_cursor_location(&mut buf, cursor.x, cursor.y - self.vscroll);
        TerminalHandler::append_show_cursor(&mut buf);

        io::stdout()
            .write_all(buf.as_bytes())
            .expect("Failed writing output");

        io::stdout().flush().expect("Cannot flush STDOUT");
    }

    fn print_tokens(&mut self, buf: &mut String, tokens: Vec<Token>, cursor_y: usize) {
        TerminalHandler::append_cursor_location(buf, 0, 0);

        self.resolve_vscroll(cursor_y);
        let output = self.colorize(tokens);

        let lines = output.lines().collect::<Vec<_>>();

        let last_line_index = lines.len().min(self.terminal_height() + self.vscroll);
        for i in self.vscroll..last_line_index {
            if i > self.vscroll {
                buf.push_str("\n\r");
            }

            buf.push_str(lines[i]);
        }
    }

    fn resolve_vscroll(&mut self, cursor_y: usize) {
        debug!("WAS vscroll={} cursor_y={}", self.vscroll, cursor_y);

        let global_cursor_y = cursor_y as i32 - self.vscroll as i32;

        if global_cursor_y < 0 {
            self.vscroll = (self.vscroll as i32 + global_cursor_y) as usize;
        } else if global_cursor_y >= self.terminal_height() as i32 {
            self.vscroll += global_cursor_y as usize - self.terminal_height() + 1;
        }
        debug!("BECAME vscroll={} cursor_y={}", self.vscroll, cursor_y);
    }

    fn print_analyzer_result_suggestions(
        &self,
        buf: &mut String,
        suggestions: Suggestion,
        suggestion_selection_mode: bool,
    ) {
        let popup_width = self.terminal_dimension.0 / POPUP_BAR_WIDTH_DIVIDER;

        for i in 0..suggestions.elems.len() {
            TerminalHandler::append_cursor_location(
                buf,
                self.terminal_dimension.0 - popup_width,
                i,
            );

            let mut line_elems = vec![
                ("|".to_string(), Some(92)),
                (suggestions.elems[i].name.clone(), Some(32)),
                (String::from(" "), None),
                (suggestions.elems[i].kind.clone(), Some(90)),
            ];

            if i <= 9 {
                let number_color = if suggestion_selection_mode { 93 } else { 90 };
                line_elems.insert(1, (format!("{} ", i), Some(number_color)));
            }

            buf.push_str(&format!(
                "{: <width$}",
                trim_coloured_string_list(line_elems, popup_width),
                width = popup_width
            ));
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
        let err_lines = EditorPrinter::chop_string(error.message, self.terminal_width());
        let token_lines = EditorPrinter::chop_string(
            format!("At token: {:?}", error.token),
            self.terminal_width(),
        );
        let scope_lines =
            EditorPrinter::chop_string(format!("Scope: {:?}", error.scope), self.terminal_width());
        let title_lines =
            EditorPrinter::chop_string("PARSE ERROR".to_string(), self.terminal_width());

        let lines = title_lines
            .into_iter()
            .chain(scope_lines.into_iter())
            .chain(token_lines.into_iter())
            .chain(err_lines.into_iter())
            .collect();

        self.print_error_message(buf, lines);
    }

    fn print_analyzer_result_definition_error(&self, buf: &mut String, error: String) {
        let mut lines = EditorPrinter::chop_string(error, self.terminal_width());

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
                TokenKind::Invalid(_) => {
                    format!(
                        "\x1B[{}m\x1B[4m{}\x1B[0m",
                        token.kind.vt100_color_code(),
                        token.original
                    )
                }
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
