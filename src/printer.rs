use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

use crate::{terminal_handler::TerminalHandler, text::Text};

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
}
