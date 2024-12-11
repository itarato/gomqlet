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

    pub fn insert_new_line(&mut self) {
        self.lines.push(String::new());
        self.cursor.x = 0;
        self.cursor.y += 1;
    }

    pub fn insert_visible_char(&mut self, ch: char) {
        self.lines
            .get_mut(self.cursor.y)
            .expect("Missing line")
            .insert(self.cursor.x, ch);

        self.cursor.x += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor.x > 0 {
            self.lines
                .get_mut(self.cursor.y)
                .expect("Missing line")
                .remove(self.cursor.x - 1);

            self.cursor.x -= 1;
        } else {
            if self.cursor.y > 0 {
                let current_line_content = self.lines[self.cursor.y].clone();
                self.lines
                    .get_mut(self.cursor.y - 1)
                    .expect("Missing line")
                    .push_str(&current_line_content);
                self.lines.remove(self.cursor.y);
                self.cursor.y -= 1;
                self.cursor.x = self.lines[self.cursor.y].len();
            } else {
                // Noop.
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1;
        } else if self.cursor.y > 0 {
            self.cursor.y -= 1;
            self.cursor.x = self.lines[self.cursor.y].len();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor.x < self.lines[self.cursor.y].len() {
            self.cursor.x += 1;
        } else if self.cursor.y < self.lines.len() - 1 {
            self.cursor.y += 1;
            self.cursor.x = 0;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor.y > 0 {
            self.cursor.y -= 1;

            if self.cursor.x > self.lines[self.cursor.y].len() {
                self.cursor.x = self.lines[self.cursor.y].len();
            }
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor.y < self.lines.len() - 1 {
            self.cursor.y += 1;

            if self.cursor.x > self.lines[self.cursor.y].len() {
                self.cursor.x = self.lines[self.cursor.y].len();
            }
        }
    }
}
