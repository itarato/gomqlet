use std::{
    cell::RefCell,
    fs::File,
    io::{self, Read},
    rc::Rc,
};

use clap::Parser;
use config::Config;
use editor::Editor;
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
mod net_ops;
mod parser;
mod printer;
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

    #[arg(short, long, value_name = "SOURCE_FILE")]
    source_file: Option<String>,
}

impl CommandLineParams {
    fn config(&self) -> Config {
        let file = File::open(&self.config_file).expect("Cannot load config file");
        serde_json::from_reader(file).unwrap()
    }

    fn source(&self) -> Option<String> {
        match &self.source_file {
            Some(source) => {
                let mut file = File::open(source).expect("Cannot load source file");
                let mut content = String::new();

                file.read_to_string(&mut content)
                    .expect("Failed reading content of source");

                Some(content)
            }
            None => None,
        }
    }
}

#[derive(PartialEq)]
enum State {
    Edit,
    FileSelector,
}

struct Gomqlet {
    terminal_handler: TerminalHandler,
    editor: Editor,
    content: Rc<RefCell<Text>>,
    net_ops: NetOps,
    state: State,
}

impl Gomqlet {
    fn new(command_line_params: CommandLineParams) -> io::Result<Gomqlet> {
        let terminal_handler = TerminalHandler::new();
        let content = Rc::new(RefCell::new(Text::new(command_line_params.source())));

        let config = command_line_params.config();

        Ok(Gomqlet {
            terminal_handler,
            editor: Editor::new(content.clone()),
            content,
            net_ops: NetOps::new(&config),
            state: State::Edit,
        })
    }

    fn exec_loop(&mut self) -> io::Result<()> {
        self.editor.refresh_screen();

        loop {
            for cmd in StdinReader::read_commands()? {
                if cmd == KeyboardInput::CtrlC || cmd == KeyboardInput::CtrlD {
                    return Ok(());
                } else if cmd == KeyboardInput::Key(7) {
                    // CTRL-G
                    self.net_ops
                        .execute_graphql_operation(self.content.borrow().to_string());
                } else if cmd == KeyboardInput::AltF {
                    self.state = State::FileSelector;
                } else if self.state == State::Edit {
                    self.editor.update(cmd);
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
