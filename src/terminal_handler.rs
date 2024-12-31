use std::{
    io::{self},
    os::fd::AsRawFd,
};

use termios::{
    tcflush, tcgetattr, tcsetattr, Termios, BRKINT, CS8, ECHO, ICANON, ICRNL, IEXTEN, INPCK, ISIG,
    ISTRIP, IXON, OPOST, TCIOFLUSH, TCSANOW, VMIN, VTIME,
};

pub struct TerminalHandler {
    original_termios: Termios,
}

impl TerminalHandler {
    pub fn new() -> TerminalHandler {
        let fd = io::stdin().as_raw_fd();
        let mut original_termios = Termios::from_fd(fd).expect("Failed getting STDIN id");

        tcgetattr(fd, &mut original_termios).expect("Failed getting termios config");
        TerminalHandler::terminal_enable_raw_mode(original_termios.clone())
            .expect("Failed setting up raw mode");

        TerminalHandler { original_termios }
    }

    pub fn terminal_enable_raw_mode(mut termios: Termios) -> io::Result<()> {
        let fd = io::stdin().as_raw_fd();

        termios.c_iflag &= !(IXON | ICRNL | BRKINT | INPCK | ISTRIP);
        termios.c_oflag &= !(OPOST);
        termios.c_lflag &= !(ECHO | ICANON | ISIG | IEXTEN);
        termios.c_cflag |= CS8;

        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 1;

        tcsetattr(fd, TCSANOW, &termios)?;
        tcflush(fd, TCIOFLUSH)?;

        Ok(())
    }

    pub fn terminal_restore_mode(&self) -> io::Result<()> {
        let fd = io::stdin().as_raw_fd();

        tcsetattr(fd, TCSANOW, &self.original_termios)?;
        tcflush(fd, TCIOFLUSH)?;

        Ok(())
    }

    pub fn append_cursor_location(out: &mut String, row: usize, col: usize) {
        let cmd = format!("\x1b[{};{}H", col + 1, row + 1);
        out.push_str(cmd.as_str());
    }

    pub fn append_clear_screen(out: &mut String) {
        out.push_str("\x1b[2J");
    }

    pub fn append_hide_cursor(out: &mut String) {
        out.push_str("\x1b[?25l");
    }

    pub fn append_show_cursor(out: &mut String) {
        out.push_str("\x1b[?25h");
    }
}
