use std::{fs, path::PathBuf};

use crate::{file_selector_printer::FileSelectorPrinter, stdin_reader::KeyboardInput};

pub enum Command {
    OpenFile(PathBuf),
}

pub struct FileSelector {
    folder: PathBuf,
    selection_index: usize,
    printer: FileSelectorPrinter,
    files: Vec<PathBuf>,
}

impl FileSelector {
    pub fn new(folder: PathBuf) -> FileSelector {
        let files = FileSelector::files(&folder);

        FileSelector {
            folder,
            selection_index: 0,
            printer: FileSelectorPrinter::new(),
            files,
        }
    }

    fn elem_len(&self) -> usize {
        self.files.len() + 1
    }

    pub fn update(&mut self, input: KeyboardInput) -> Option<Command> {
        match input {
            KeyboardInput::AltDigit(value) => {
                self.selection_index = value as usize % self.elem_len();
                if self.selection_index == 0 {
                    unimplemented!()
                } else {
                    return Some(Command::OpenFile(
                        self.files[self.selection_index - 1].clone(),
                    ));
                }
            }
            KeyboardInput::Up => {
                self.selection_index =
                    (self.selection_index + self.elem_len() - 1) % self.elem_len();
            }
            KeyboardInput::Down => {
                self.selection_index = (self.selection_index + 1) % self.elem_len();
            }
            KeyboardInput::Key(13) => {
                if self.selection_index == 0 {
                    unimplemented!()
                } else {
                    return Some(Command::OpenFile(
                        self.files[self.selection_index - 1].clone(),
                    ));
                }
            }
            _ => {}
        }

        self.refresh_screen();

        None
    }

    pub fn refresh_screen(&self) {
        let files = FileSelector::files(&self.folder);

        self.printer
            .print(&self.folder, files, self.selection_index);
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
