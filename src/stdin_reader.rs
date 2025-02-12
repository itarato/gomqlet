use std::io::{self, Read};

const ESCAPE_COMBOS: &[(&[u8], KeyboardInput)] = &[
    (&[27, 91, 65], KeyboardInput::Up),
    (&[27, 91, 66], KeyboardInput::Down),
    (&[27, 91, 67], KeyboardInput::Right),
    (&[27, 91, 68], KeyboardInput::Left),
    (&[27, 91, 72], KeyboardInput::Home),
    (&[27, 91, 70], KeyboardInput::End),
    (&[27, 91, 51, 126], KeyboardInput::Delete),
    (&[27, 48], KeyboardInput::AltDigit(0)),
    (&[27, 49], KeyboardInput::AltDigit(1)),
    (&[27, 50], KeyboardInput::AltDigit(2)),
    (&[27, 51], KeyboardInput::AltDigit(3)),
    (&[27, 52], KeyboardInput::AltDigit(4)),
    (&[27, 53], KeyboardInput::AltDigit(5)),
    (&[27, 54], KeyboardInput::AltDigit(6)),
    (&[27, 55], KeyboardInput::AltDigit(7)),
    (&[27, 56], KeyboardInput::AltDigit(8)),
    (&[27, 57], KeyboardInput::AltDigit(9)),
    (&[27, 102], KeyboardInput::AltF),
    (&[27, 115], KeyboardInput::AltS),
    // MacOS
    (&[194, 186], KeyboardInput::AltDigit(0)),
    (&[194, 161], KeyboardInput::AltDigit(1)),
    (&[226, 132, 162], KeyboardInput::AltDigit(2)),
    (&[194, 163], KeyboardInput::AltDigit(3)),
    (&[194, 162], KeyboardInput::AltDigit(4)),
    (&[226, 136, 158], KeyboardInput::AltDigit(5)),
    (&[194, 167], KeyboardInput::AltDigit(6)),
    (&[194, 182], KeyboardInput::AltDigit(7)),
    (&[226, 128, 162], KeyboardInput::AltDigit(8)),
    (&[194, 170], KeyboardInput::AltDigit(9)),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardInput {
    VisibleChar(u8),
    ControlChar(u8),

    CtrlC,
    CtrlD,
    CtrlF,
    CtrlW,
    CtrlS,
    CtrlR,
    CtrlG,
    CtrlO,
    CtrlSlash,

    AltDigit(u8),
    AltF,
    AltS,

    Left,
    Right,
    Up,
    Down,

    Home,
    End,
    Delete,
    Enter,
    Escape,
    Backspace,
    Tab,
}

pub struct StdinReader;

impl StdinReader {
    pub fn read_commands() -> io::Result<Vec<KeyboardInput>> {
        let mut stdin = io::stdin();
        let mut buf: [u8; 8] = [0; 8];

        let read_len = stdin.read(&mut buf)?;
        if read_len == 0 {
            return Ok(vec![]);
        }

        Ok(StdinReader::parse_stdin_bytes(&mut buf, read_len))
    }

    pub fn parse_stdin_bytes(buf: &[u8], len: usize) -> Vec<KeyboardInput> {
        let mut i = 0usize;
        let mut out = vec![];

        trace!("Input: {:?}", &buf[0..len]);

        while i < len {
            //                  MacOS option key starters:
            //                  |----------------------------|
            if (buf[i] == 27 || buf[i] == 194 || buf[i] == 226) && len > 1 {
                for (seq, ki) in ESCAPE_COMBOS {
                    if i + seq.len() <= len {
                        if *seq == &buf[i..i + seq.len()] {
                            out.push(ki.clone());
                            i += seq.len();
                            break;
                        }
                    }
                }

                // We didn't hit the combo - possibly unmapped one.
                if i != len {
                    warn!("Unmapped combo {:?}", &buf[i..len]);
                }

                // We might ignore real multi key input - however that's a price to pay as long as we're not mapping
                // all escape sequenced combos.
                i = len;
            } else if buf[i] == 3 {
                out.push(KeyboardInput::CtrlC);
                i += 1;
            } else if buf[i] == 4 {
                out.push(KeyboardInput::CtrlD);
                i += 1;
            } else if buf[i] == 6 {
                out.push(KeyboardInput::CtrlF);
                i += 1;
            } else if buf[i] == 7 {
                out.push(KeyboardInput::CtrlG);
                i += 1;
            } else if buf[i] == 9 {
                out.push(KeyboardInput::Tab);
                i += 1;
            } else if buf[i] == 13 {
                out.push(KeyboardInput::Enter);
                i += 1;
            } else if buf[i] == 15 {
                out.push(KeyboardInput::CtrlO);
                i += 1;
            } else if buf[i] == 18 {
                out.push(KeyboardInput::CtrlR);
                i += 1;
            } else if buf[i] == 19 {
                out.push(KeyboardInput::CtrlS);
                i += 1;
            } else if buf[i] == 23 {
                out.push(KeyboardInput::CtrlW);
                i += 1;
            } else if buf[i] == 27 {
                out.push(KeyboardInput::Escape);
                i += 1;
            } else if buf[i] == 31 {
                out.push(KeyboardInput::CtrlSlash);
                i += 1;
            } else if buf[i] == 127 {
                out.push(KeyboardInput::Backspace);
                i += 1;
            } else if buf[i] >= 32 && buf[i] <= 126 {
                out.push(KeyboardInput::VisibleChar(buf[i]));
                i += 1;
            } else {
                out.push(KeyboardInput::ControlChar(buf[i]));
                i += 1;
            }
        }

        out
    }
}
