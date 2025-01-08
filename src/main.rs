use std::{cell::RefCell, io, path::PathBuf, rc::Rc};

use clap::Parser;
use command_line_params::CommandLineParams;
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
mod command_line_params;
mod config;
mod editor;
mod editor_printer;
mod file_selector;
mod file_selector_printer;
mod json_path;
mod magic_command;
mod net_ops;
mod parser;
mod schema;
mod stdin_reader;
mod terminal_handler;
mod text;
mod tokenizer;
mod util;

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
        let net_ops = NetOps::new(&config);
        let editor = Editor::new(
            content.clone(),
            &net_ops,
            &PathBuf::from(config.schema_cache),
            command_line_params.reload_schema,
        );

        Ok(Gomqlet {
            terminal_handler,
            editor,
            file_selector: FileSelector::new(source_folder),
            content,
            net_ops,
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
                } else if cmd == KeyboardInput::CtrlG {
                    // CTRL-G
                    self.net_ops
                        .execute_graphql_operation(&self.content.borrow().to_string_no_new_lines());
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
