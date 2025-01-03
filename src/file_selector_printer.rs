use std::{
    io::{self, Write},
    path::PathBuf,
};

/*

| /home/path/other
+-- few
+-- few
*/

use crate::terminal_handler::TerminalHandler;

pub struct FileSelectorPrinter;

impl FileSelectorPrinter {
    pub fn new() -> FileSelectorPrinter {
        FileSelectorPrinter
    }

    pub fn print(&self, folder: &PathBuf, file_paths: Vec<PathBuf>, selected_index: usize) {
        let mut buf: String = String::new();
        TerminalHandler::append_hide_cursor(&mut buf);
        TerminalHandler::append_clear_screen(&mut buf);
        TerminalHandler::append_cursor_location(&mut buf, 0, 0);

        buf.push_str(&format!(
            "| {}\n\r",
            folder.to_str().expect("Failed stringifying path")
        ));

        for (i, path) in file_paths.iter().enumerate() {
            if selected_index == i {
                buf.push_str(&format!(
                    "+-- \x1B[104m#{}: {}\x1B[0m",
                    i,
                    path.to_str().unwrap()
                ));
            } else {
                buf.push_str(&format!(
                    "+-- \x1B[34m#{}: {}\x1B[0m",
                    i,
                    path.to_str().unwrap()
                ));
            }
            buf.push_str("\n\r");
        }

        TerminalHandler::append_show_cursor(&mut buf);

        io::stdout()
            .write_all(buf.as_bytes())
            .expect("Failed writing output");

        io::stdout().flush().expect("Cannot flush STDOUT");
    }
}
