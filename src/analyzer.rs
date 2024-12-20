use std::{cell::RefCell, rc::Rc};

use crate::text::Text;

pub struct Analyzer {
    content: Rc<RefCell<Text>>,
}

impl Analyzer {
    pub fn new(content: Rc<RefCell<Text>>) -> Analyzer {
        Analyzer { content }
    }

    pub fn analyze(&self) {}
}
