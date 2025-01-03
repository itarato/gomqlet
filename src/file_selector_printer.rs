use std::{
    io::{self, Write},
    path::PathBuf,
};

use crate::terminal_handler::TerminalHandler;

pub struct FileSelectorPrinter;

impl FileSelectorPrinter {
    pub fn new() -> FileSelectorPrinter {
        FileSelectorPrinter
    }

    pub fn print(&self, file_paths: Vec<PathBuf>, selected_index: usize) {
        let mut buf: String = String::new();
        TerminalHandler::append_hide_cursor(&mut buf);
        TerminalHandler::append_clear_screen(&mut buf);
        TerminalHandler::append_cursor_location(&mut buf, 0, 0);

        for (i, path) in file_paths.iter().enumerate() {
            if selected_index == i {
                buf.push_str("> ");
            }

            buf.push_str(&format!("#{}: {}", i, path.to_str().unwrap()));
            buf.push_str("\n\r");
        }

        TerminalHandler::append_show_cursor(&mut buf);

        io::stdout()
            .write_all(buf.as_bytes())
            .expect("Failed writing output");

        io::stdout().flush().expect("Cannot flush STDOUT");
    }
}
