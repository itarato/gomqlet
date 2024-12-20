use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Read, Write},
    os::fd::AsRawFd,
    rc::Rc,
};

use editor::{Editor, EditorInput};
use graphql_parser::parse_schema;
use printer::Printer;
use terminal_handler::TerminalHandler;
use text::Text;

extern crate pretty_env_logger;
extern crate termios;
#[macro_use]
extern crate log;

mod ast;
mod editor;
mod parser;
mod printer;
mod terminal_handler;
mod text;
mod tokenizer;
mod util;

#[derive(Clone)]
enum KeyboardInput {
    Key(u8),
    CtrlC,
    CtrlD,
    Left,
    Right,
    Up,
    Down,
}

struct Gomqlet {
    terminal_handler: TerminalHandler,
    editor: Editor,
    printer: Printer,
}

impl Gomqlet {
    fn new() -> io::Result<Gomqlet> {
        let terminal_handler = TerminalHandler::new();
        let content = Rc::new(RefCell::new(Text::new()));
        Ok(Gomqlet {
            terminal_handler,
            editor: Editor::new(content.clone()),
            printer: Printer::new(content),
        })
    }

    fn exec_loop(&mut self) -> io::Result<()> {
        let mut stdin = io::stdin();
        let mut stdout = io::stdout();
        dbg!(stdin.as_raw_fd());
        let mut buf: [u8; 8] = [0; 8];

        TerminalHandler::clear_screen()?;
        TerminalHandler::set_cursor_location(0, 0);
        stdout.flush()?;

        loop {
            let read_len = stdin.read(&mut buf)?;
            if read_len == 0 {
                continue;
            }
            let cmds = parse_stdin_bytes(&mut buf, read_len);
            // debug!("Read len: {}", read_len);

            for cmd in cmds {
                match cmd {
                    KeyboardInput::CtrlC | KeyboardInput::CtrlD => return Ok(()),
                    KeyboardInput::Key(code) => {
                        self.editor.parse_input(EditorInput::Char(code));
                        self.printer.print();
                    }
                    KeyboardInput::Left => {
                        self.editor.parse_input(EditorInput::Left);
                        self.printer.print();
                    }
                    KeyboardInput::Right => {
                        self.editor.parse_input(EditorInput::Right);
                        self.printer.print();
                    }
                    KeyboardInput::Up => {
                        self.editor.parse_input(EditorInput::Up);
                        self.printer.print();
                    }
                    KeyboardInput::Down => {
                        self.editor.parse_input(EditorInput::Down);
                        self.printer.print();
                    }
                };
            }

            // debug!("Key hit: {}", buf[0]);
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

fn parse_stdin_bytes(buf: &[u8], len: usize) -> Vec<KeyboardInput> {
    /*
     * Up:    27 91 65
     * Down:  27 91 66
     * Right: 27 91 67
     * Left:  27 91 68
     */
    let escape_combos: HashMap<Vec<u8>, KeyboardInput> = HashMap::from([
        (vec![27, 91, 65], KeyboardInput::Up),
        (vec![27, 91, 66], KeyboardInput::Down),
        (vec![27, 91, 67], KeyboardInput::Right),
        (vec![27, 91, 68], KeyboardInput::Left),
    ]);
    let mut i = 0usize;
    let mut out = vec![];

    while i < len {
        if buf[i] == 27 {
            for (seq, ki) in &escape_combos {
                if i + seq.len() <= len {
                    if seq.as_slice() == &buf[i..i + seq.len()] {
                        out.push(ki.clone());
                        i += seq.len();
                        break;
                    }
                }
            }
            i += 1;
        } else if buf[i] == 3 {
            out.push(KeyboardInput::CtrlC);
            i += 1;
        } else if buf[i] == 4 {
            out.push(KeyboardInput::CtrlD);
            i += 1;
        } else {
            out.push(KeyboardInput::Key(buf[i]));
            i += 1;
        }
    }

    out
}

fn main() -> io::Result<()> {
    pretty_env_logger::init();

    info!("Gomqlet start");

    let mut gomqlet = Gomqlet::new()?;
    gomqlet.exec_loop()?;

    Ok(())
}
