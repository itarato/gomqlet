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

    pub fn print(
        &self,
        folder: &PathBuf,
        file_paths: Vec<PathBuf>,
        selected_index: usize,
        new_file_name: &Option<String>,
    ) {
        let mut buf: String = String::new();
        TerminalHandler::append_hide_cursor(&mut buf);
        TerminalHandler::append_clear_screen(&mut buf);
        TerminalHandler::append_cursor_location(&mut buf, 0, 0);

        buf.push_str(&format!(
            "| {}\n\r",
            folder.to_str().expect("Failed stringifying path")
        ));

        let mut elems = file_paths
            .iter()
            .map(|e| e.to_str().unwrap())
            .collect::<Vec<_>>();

        let first_line = match new_file_name {
            Some(file_name) => {
                format!(
                    "New file: {}/{}.graphql",
                    folder.to_str().unwrap(),
                    file_name
                )
            }
            None => "Create new file".to_string(),
        };
        elems.insert(0, &first_line);

        for (i, elem) in elems.iter().enumerate() {
            if selected_index == i {
                buf.push_str(&format!("+-- \x1B[44m#{}: {}\x1B[0m", i, elem));
            } else {
                buf.push_str(&format!("+-- \x1B[34m#{}: {}\x1B[0m", i, elem));
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
