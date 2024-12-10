use std::{
    cell::RefCell,
    io::{self, Read, Write},
    os::fd::AsRawFd,
    rc::Rc,
};

use editor::{Editor, EditorInput};
use printer::Printer;
use terminal_handler::TerminalHandler;
use text::Text;

extern crate termios;

mod editor;
mod printer;
mod terminal_handler;
mod text;
mod util;

struct Gomqlet {
    terminal_handler: TerminalHandler,
    editor: Editor,
    printer: Printer,
}

impl Gomqlet {
    fn new() -> io::Result<Gomqlet> {
        let terminal_handler = TerminalHandler::try_new()?;
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
        let mut buf: [u8; 1] = [0; 1];

        TerminalHandler::clear_screen()?;
        stdout.flush()?;

        loop {
            let read_len = stdin.read(&mut buf)?;
            if read_len == 0 {
                continue;
            }

            if buf[0] == 27 {
                return Ok(());
            }

            self.editor.parse_input(EditorInput::Char(buf[0]));
            self.printer.print();

            // print!("{} ", buf[0]);
            // stdout.flush()?;
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
    let mut gomqlet = Gomqlet::new()?;
    gomqlet.exec_loop()?;

    Ok(())
}
