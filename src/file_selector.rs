use std::{fs, path::PathBuf};

use crate::{file_selector_printer::FileSelectorPrinter, stdin_reader::KeyboardInput};

pub struct FileSelector {
    folder: PathBuf,
    file_index: usize,
    printer: FileSelectorPrinter,
}

impl FileSelector {
    pub fn new(folder: PathBuf) -> FileSelector {
        FileSelector {
            folder,
            file_index: 0,
            printer: FileSelectorPrinter::new(),
        }
    }

    pub fn update(&self, input: KeyboardInput) {
        self.refresh_screen();
    }

    pub fn refresh_screen(&self) {
        let files = self.files();

        self.printer.print(files);
    }

    fn files(&self) -> Vec<PathBuf> {
        fs::read_dir(&self.folder)
            .expect("Cannot read source folder")
            .filter_map(|dir_entry_result| dir_entry_result.ok())
            .filter(|dir_entry| {
                dir_entry
                    .file_type()
                    .map(|ty| ty.is_file())
                    .unwrap_or(false)
            })
            .map(|dir_entry| dir_entry.path())
            .collect()
    }
}
