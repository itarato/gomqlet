use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

use crate::{
    terminal_handler::TerminalHandler,
    text::Text,
    tokenizer::{Token, Tokenizer},
};

pub struct Printer {
    content: Rc<RefCell<Text>>,
}

impl Printer {
    pub fn new(content: Rc<RefCell<Text>>) -> Printer {
        Printer { content }
    }

    pub fn print(&self) {
        let mut buf: String = String::new();
        TerminalHandler::append_hide_cursor(&mut buf);
        TerminalHandler::append_clear_screen(&mut buf);
        TerminalHandler::append_cursor_location(&mut buf, 0, 0);

        let lines = self.lines_with_coloring();
        let lines_len = lines.len();
        for i in 0..lines_len {
            buf.push_str(lines[i].as_str());
            buf.push_str("\n\r");
        }

        TerminalHandler::append_cursor_location(
            &mut buf,
            self.content.borrow().cursor.x,
            self.content.borrow().cursor.y,
        );
        TerminalHandler::append_show_cursor(&mut buf);

        io::stdout()
            .write_all(buf.as_bytes())
            .expect("Failed writing output");

        io::stdout().flush().expect("Cannot flush STDOUT");
    }

    fn build_tokens(&self) -> Vec<Vec<Token>> {
        self.content
            .borrow()
            .lines
            .clone()
            .into_iter()
            .map(|line| Tokenizer::tokenize(&*line, true))
            .collect()
    }

    fn lines_with_coloring(&self) -> Vec<String> {
        self.build_tokens()
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
