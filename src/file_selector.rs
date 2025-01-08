use std::{
    fs::{self, File},
    path::PathBuf,
};

use crate::{file_selector_printer::FileSelectorPrinter, stdin_reader::KeyboardInput};

enum State {
    Selection,
    NewFileNameTyping,
}

pub enum Command {
    OpenFile(PathBuf),
}

pub struct FileSelector {
    folder: PathBuf,
    selection_index: usize,
    printer: FileSelectorPrinter,
    files: Vec<PathBuf>,
    state: State,
    new_file_name: Option<String>,
}

impl FileSelector {
    pub fn new(folder: PathBuf) -> FileSelector {
        let files = FileSelector::files(&folder);

        FileSelector {
            folder,
            selection_index: 0,
            printer: FileSelectorPrinter::new(),
            files,
            state: State::Selection,
            new_file_name: None,
        }
    }

    fn elem_len(&self) -> usize {
        self.files.len() + 2 // files + create new + parent dir
    }

    pub fn update(&mut self, input: KeyboardInput) -> Option<Command> {
        let cmd = match self.state {
            State::Selection => self.update_selection_state(input),
            State::NewFileNameTyping => self.update_new_file_name_typing_state(input),
        };
        if cmd.is_some() {
            return cmd;
        }

        self.refresh_screen();

        None
    }

    fn update_selection_state(&mut self, input: KeyboardInput) -> Option<Command> {
        match input {
            KeyboardInput::Up => {
                self.selection_index =
                    (self.selection_index + self.elem_len() - 1) % self.elem_len();
            }
            KeyboardInput::Down => {
                self.selection_index = (self.selection_index + 1) % self.elem_len();
            }
            KeyboardInput::Enter => {
                if self.selection_index == 0 {
                    self.folder.pop();
                    self.selection_index = 0;
                } else if self.selection_index == self.files.len() + 1 {
                    self.state_change_to_new_file_name_typing();
                } else {
                    if self.files[self.selection_index - 1].is_dir() {
                        self.folder = self.files[self.selection_index - 1].clone();
                        self.selection_index = 0;
                    } else {
                        return Some(Command::OpenFile(
                            self.files[self.selection_index - 1].clone(),
                        ));
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn update_new_file_name_typing_state(&mut self, input: KeyboardInput) -> Option<Command> {
        match input {
            KeyboardInput::Enter => {
                let mut new_file_path = self.folder.clone();
                new_file_path.push(format!("{}.graphql", self.new_file_name.as_ref().unwrap()));

                File::create(&new_file_path).expect("Failed creating new file");

                self.state_selection();

                return Some(Command::OpenFile(new_file_path));
            }
            KeyboardInput::Backspace => {
                self.new_file_name.as_mut().unwrap().pop();
            }
            KeyboardInput::VisibleChar(ch) => {
                if ch.is_ascii_alphanumeric() || ch == b'.' || ch == b'_' {
                    self.new_file_name.as_mut().unwrap().push(ch as char);
                }
            }
            KeyboardInput::Escape => {
                self.state_selection();
            }
            _ => {}
        }
        None
    }

    pub fn refresh_screen(&mut self) {
        self.files = FileSelector::files(&self.folder);

        self.printer.print(
            &self.folder,
            &self.files,
            self.selection_index,
            &self.new_file_name,
        );
    }

    fn state_change_to_new_file_name_typing(&mut self) {
        self.state = State::NewFileNameTyping;
        self.new_file_name = Some(String::new());
    }

    fn state_selection(&mut self) {
        self.state = State::Selection;
        self.new_file_name = None;
    }

    fn files(folder: &PathBuf) -> Vec<PathBuf> {
        fs::read_dir(folder)
            .expect("Cannot read source folder")
            .filter_map(|dir_entry_result| dir_entry_result.ok())
            .map(|dir_entry| dir_entry.path())
            .filter(|path| {
                path.is_dir()
                    || path
                        .extension()
                        .map(|ext| ext == "graphql")
                        .unwrap_or(false)
            })
            .collect()
    }
}
