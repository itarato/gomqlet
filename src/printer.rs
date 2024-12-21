use std::io::{self, Write};

use crate::{terminal_handler::TerminalHandler, tokenizer::Token, util::CoordUsize};

pub struct Printer;

impl Printer {
    pub fn new() -> Printer {
        Printer
    }

    pub fn print(&self, lines_tokens: Vec<Vec<Token>>, cursor: CoordUsize) {
        let mut buf: String = String::new();
        TerminalHandler::append_hide_cursor(&mut buf);
        TerminalHandler::append_clear_screen(&mut buf);
        TerminalHandler::append_cursor_location(&mut buf, 0, 0);

        let lines = self.lines_with_coloring(lines_tokens);
        let lines_len = lines.len();
        for i in 0..lines_len {
            buf.push_str(lines[i].as_str());
            buf.push_str("\n\r");
        }

        TerminalHandler::append_cursor_location(&mut buf, cursor.x, cursor.y);
        TerminalHandler::append_show_cursor(&mut buf);

        io::stdout()
            .write_all(buf.as_bytes())
            .expect("Failed writing output");

        io::stdout().flush().expect("Cannot flush STDOUT");
    }

    fn lines_with_coloring(&self, lines_tokens: Vec<Vec<Token>>) -> Vec<String> {
        lines_tokens
            .into_iter()
            .map(|tokens| {
                tokens
                    .into_iter()
                    .map(|token| {
                        format!(
                            "\x1B[{}m{}\x1B[0m",
                            token.kind.vt100_color_code(),
                            token.original
                        )
                    })
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
    }
}
