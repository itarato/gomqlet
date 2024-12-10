use crate::util::CoordUsize;

pub struct Text {
    pub lines: Vec<String>,
    pub cursor: CoordUsize,
}

impl Text {
    pub fn new() -> Text {
        Text {
            lines: vec![String::new()],
            cursor: CoordUsize { x: 0, y: 0 },
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        self.lines
            .get_mut(self.cursor.y)
            .expect("Missing line")
            .insert(self.cursor.x, ch);

        self.cursor.x += 1;
    }

    pub fn erase_char(&mut self) {
        self.lines
            .get_mut(self.cursor.y)
            .expect("Missing line")
            .remove(self.cursor.x - 1);

        self.cursor.x -= 1;
    }
}
