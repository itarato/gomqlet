use std::{cell::RefCell, rc::Rc};

use crate::text::Text;

pub enum EditorInput {
    Char(u8),
    Left,
    Right,
    Up,
    Down,
}

pub struct Editor {
    content: Rc<RefCell<Text>>,
}

impl Editor {
    pub fn new(content: Rc<RefCell<Text>>) -> Editor {
        Editor { content }
    }

    pub fn parse_input(&mut self, input: EditorInput) {
        match input {
            EditorInput::Char(127) => self.content.borrow_mut().backspace(),
            EditorInput::Char(13) => self.content.borrow_mut().insert_new_line(),
            EditorInput::Char(ch) => self.content.borrow_mut().insert_visible_char(ch as char),
        }
    }
}
