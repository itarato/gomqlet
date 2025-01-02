use std::{
    collections::HashMap,
    io::{self, Read},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyboardInput {
    Key(u8),
    CtrlC,
    CtrlD,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Delete,
    AltDigit(u8),
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
        let escape_combos: HashMap<Vec<u8>, KeyboardInput> = HashMap::from([
            (vec![27, 91, 65], KeyboardInput::Up),
            (vec![27, 91, 66], KeyboardInput::Down),
            (vec![27, 91, 67], KeyboardInput::Right),
            (vec![27, 91, 68], KeyboardInput::Left),
            (vec![27, 91, 72], KeyboardInput::Home),
            (vec![27, 91, 70], KeyboardInput::End),
            (vec![27, 91, 51, 126], KeyboardInput::Delete),
            (vec![27, 48], KeyboardInput::AltDigit(0)),
            (vec![27, 49], KeyboardInput::AltDigit(1)),
            (vec![27, 50], KeyboardInput::AltDigit(2)),
            (vec![27, 51], KeyboardInput::AltDigit(3)),
            (vec![27, 52], KeyboardInput::AltDigit(4)),
            (vec![27, 53], KeyboardInput::AltDigit(5)),
            (vec![27, 54], KeyboardInput::AltDigit(6)),
            (vec![27, 55], KeyboardInput::AltDigit(7)),
            (vec![27, 56], KeyboardInput::AltDigit(8)),
            (vec![27, 57], KeyboardInput::AltDigit(9)),
        ]);
        let mut i = 0usize;
        let mut out = vec![];

        debug!("{:?}", &buf[0..len]);

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
            } else {
                out.push(KeyboardInput::Key(buf[i]));
                i += 1;
            }
        }

        out
    }
}
