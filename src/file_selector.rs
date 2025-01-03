use std::{fs, path::PathBuf};

use crate::{file_selector_printer::FileSelectorPrinter, stdin_reader::KeyboardInput};

pub enum Command {
    OpenFile(PathBuf),
}

pub struct FileSelector {
    folder: PathBuf,
    file_index: usize,
    printer: FileSelectorPrinter,
    files: Vec<PathBuf>,
}

impl FileSelector {
    pub fn new(folder: PathBuf) -> FileSelector {
        let files = FileSelector::files(&folder);

        FileSelector {
            folder,
            file_index: 0,
            printer: FileSelectorPrinter::new(),
            files,
        }
    }

    pub fn update(&mut self, input: KeyboardInput) -> Option<Command> {
        match input {
            KeyboardInput::AltDigit(value) => {
                self.file_index = value as usize % self.files.len();
                return Some(Command::OpenFile(self.files[self.file_index].clone()));
            }
            KeyboardInput::Up => {
                self.file_index = (self.file_index + self.files.len() - 1) % self.files.len();
            }
            KeyboardInput::Down => {
                self.file_index = (self.file_index + 1) % self.files.len();
            }
            KeyboardInput::Key(13) => {
                return Some(Command::OpenFile(self.files[self.file_index].clone()));
            }
            _ => {}
        }

        self.refresh_screen();

        None
    }

    pub fn refresh_screen(&self) {
        let files = FileSelector::files(&self.folder);

        self.printer.print(files, self.file_index);
    }

    fn files(folder: &PathBuf) -> Vec<PathBuf> {
        fs::read_dir(folder)
            .expect("Cannot read source folder")
            .filter_map(|dir_entry_result| dir_entry_result.ok())
            .filter(|dir_entry| {
                dir_entry
                    .file_type()
                    .map(|ty| ty.is_file())
                    .unwrap_or(false)
            })
            .map(|dir_entry| dir_entry.path())
            .filter(|path| {
                path.extension()
                    .map(|ext| ext == "graphql")
                    .unwrap_or(false)
            })
            .collect()
    }
}
