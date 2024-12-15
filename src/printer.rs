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

        let lines_len = self.content.borrow().lines.len();
        for i in 0..lines_len {
            buf.push_str(self.content.borrow().lines[i].as_str());
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

    fn add_coloring(&self) -> Vec<String> {
        let mut lines = self.content.borrow().lines.clone();
        // let tokens = self.build_tokens();

        // let mut offset = 0usize;
        // let mut current_line = 0usize;
        // let mut current_line_start = 0usize;

        // for token in tokens {
        //     let pos_start = token.pos;

        //     // Find start position's line.
        //     while current_line_start + lines[current_line].len() <= pos_start {
        //         current_line_start += lines[current_line].len();
        //         current_line += 1;
        //         offset = 0;
        //     }

        //     let line_rel_pos = pos_start - current_line_start;
        //     let color_tag = format!("<color>");

        //     lines[current_line].insert_str(line_rel_pos + offset, &color_tag);
        // }

        lines
    }
}
