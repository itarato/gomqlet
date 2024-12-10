use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Read, Write},
    os::fd::AsRawFd,
    rc::Rc,
};

use editor::{Editor, EditorInput};
use printer::Printer;
use terminal_handler::TerminalHandler;
use text::Text;

extern crate pretty_env_logger;
extern crate termios;
#[macro_use]
extern crate log;

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
        TerminalHandler::set_cursor_location(0, 0);
        stdout.flush()?;

        /*
         * Up:    27 91 65
         * Down:  27 91 66
         * Right: 27 91 67
         * Left:  27 91 68
         */
        let escape_combos: HashMap<Vec<u8>, EditorInput> = HashMap::from([
            (vec![91, 65], EditorInput::Up),
            (vec![91, 66], EditorInput::Down),
            (vec![91, 67], EditorInput::Right),
            (vec![91, 68], EditorInput::Left),
        ]);

        loop {
            let read_len = stdin.read(&mut buf)?;
            if read_len == 0 {
                continue;
            }

            if buf[0] == 27 {}

            // CTRL + C       CTRL + D
            if buf[0] == 3 || buf[0] == 4 {
                return Ok(());
            }

            self.editor.parse_input(EditorInput::Char(buf[0]));
            self.printer.print();

            debug!("Key hit: {}", buf[0]);
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

    let mut gomqlet = Gomqlet::new()?;
    gomqlet.exec_loop()?;

    Ok(())
}
