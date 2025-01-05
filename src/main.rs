use std::{
    cell::RefCell,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
    rc::Rc,
};

use clap::Parser;
use config::Config;
use editor::Editor;
use file_selector::FileSelector;
use net_ops::NetOps;
use stdin_reader::{KeyboardInput, StdinReader};
use terminal_handler::TerminalHandler;
use text::Text;

extern crate pretty_env_logger;
extern crate termios;
#[macro_use]
extern crate log;

mod analyzer;
mod ast;
mod config;
mod editor;
mod editor_printer;
mod file_selector;
mod file_selector_printer;
mod net_ops;
mod parser;
mod schema;
mod stdin_reader;
mod terminal_handler;
mod text;
mod tokenizer;
mod util;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct CommandLineParams {
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config_file: String,

    #[arg(long, value_name = "SOURCE_FILE")]
    source_file: Option<String>,

    #[arg(long, value_name = "SOURCE_FOLDER")]
    source_folder: Option<String>,
}

impl CommandLineParams {
    fn config(&self) -> Config {
        let file = File::open(&self.config_file).expect("Cannot load config file");
        serde_json::from_reader(file).unwrap()
    }

    fn source_folder(&self) -> PathBuf {
        if let Some(ref source_folder) = self.source_folder {
            PathBuf::from(source_folder)
        } else if let Some(ref source_file) = self.source_file {
            let mut path = fs::canonicalize(PathBuf::from(source_file))
                .expect("Failed to get path of source file");
            path.pop();

            path
        } else {
            fs::canonicalize(PathBuf::from("./"))
                .expect("Failed to get absolute path for current dir")
        }
    }
}

#[derive(PartialEq)]
enum State {
    Editor,
    FileSelector,
}

struct Gomqlet {
    terminal_handler: TerminalHandler,
    editor: Editor,
    file_selector: FileSelector,
    content: Rc<RefCell<Text>>,
    net_ops: NetOps,
    state: State,
}

impl Gomqlet {
    fn new(command_line_params: CommandLineParams) -> io::Result<Gomqlet> {
        let state = if command_line_params.source_file.is_some() {
            State::Editor
        } else {
            State::FileSelector
        };
        let terminal_handler = TerminalHandler::new();
        let content = Rc::new(RefCell::new(Text::new(
            command_line_params
                .source_file
                .clone()
                .map(|file_path| PathBuf::from(file_path)),
        )));
        let config = command_line_params.config();
        let source_folder = command_line_params.source_folder();

        Ok(Gomqlet {
            terminal_handler,
            editor: Editor::new(content.clone()),
            file_selector: FileSelector::new(source_folder),
            content,
            net_ops: NetOps::new(&config),
            state,
        })
    }

    fn exec_loop(&mut self) -> io::Result<()> {
        match self.state {
            State::Editor => self.editor.refresh_screen(),
            State::FileSelector => self.file_selector.refresh_screen(),
        }

        loop {
            for cmd in StdinReader::read_commands()? {
                if cmd == KeyboardInput::CtrlC || cmd == KeyboardInput::CtrlD {
                    return Ok(());
                } else if cmd == KeyboardInput::Key(7) {
                    // CTRL-G
                    self.net_ops
                        .execute_graphql_operation(self.content.borrow().to_string_no_new_lines());
                } else if cmd == KeyboardInput::AltF || cmd == KeyboardInput::CtrlF {
                    self.state = State::FileSelector;
                    self.file_selector.refresh_screen();
                } else if self.state == State::Editor {
                    self.editor.update(cmd);
                } else if self.state == State::FileSelector {
                    match self.file_selector.update(cmd) {
                        Some(file_selector::Command::OpenFile(path)) => {
                            self.state = State::Editor;
                            self.content.borrow_mut().reload_from_file(path);
                            self.editor.refresh_screen();
                        }
                        _ => {}
                    };
                } else {
                    unreachable!("Invalid state");
                }
            }
        }
    }
}

impl Drop for Gomqlet {
    fn drop(&mut self) {
        self.terminal_handler
            .terminal_restore_mode()
            .expect("Failed reverting terminal mode.");
    }
}

fn main() -> io::Result<()> {
    pretty_env_logger::init();
    info!("Gomqlet start");

    let args = CommandLineParams::parse();

    let mut gomqlet = Gomqlet::new(args)?;
    gomqlet.exec_loop()?;

    Ok(())
}
