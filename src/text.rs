use crate::{analyzer::Suggestion, util::CoordUsize};

const TAB_SIZE: usize = 2;

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

    pub fn to_string(&self) -> String {
        self.lines.join("")
    }

    pub fn insert_new_line(&mut self) {
        let mut fragment_to_split = self.lines[self.cursor.y][self.cursor.x..].to_owned();
        self.lines
            .get_mut(self.cursor.y)
            .expect("Missing line")
            .truncate(self.cursor.x);

        let previous_line_front_space_length = self.front_space_length(self.cursor.y);
        let new_prefix = String::from_utf8(vec![b' '; previous_line_front_space_length]).unwrap();
        fragment_to_split.insert_str(0, &new_prefix);

        self.lines.insert(self.cursor.y + 1, fragment_to_split);

        self.cursor.x = previous_line_front_space_length;
        self.cursor.y += 1;
    }

    pub fn insert_visible_char(&mut self, ch: char) {
        self.lines
            .get_mut(self.cursor.y)
            .expect("Missing line")
            .insert(self.cursor.x, ch);

        self.cursor.x += 1;
    }

    pub fn insert_tab(&mut self) {
        let remaining_spaces = TAB_SIZE - (self.cursor.x % TAB_SIZE);
        for _ in 0..remaining_spaces {
            self.insert_visible_char(' ');
        }
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
                let prev_line_len = self.lines[self.cursor.y - 1].len();

                let current_line_content = self.lines[self.cursor.y].clone();
                self.lines
                    .get_mut(self.cursor.y - 1)
                    .expect("Missing line")
                    .push_str(&current_line_content);
                self.lines.remove(self.cursor.y);
                self.cursor.y -= 1;
                self.cursor.x = prev_line_len;
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

    pub fn move_cursor_to_home(&mut self) {
        self.cursor.x = 0;
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor.x = self.lines[self.cursor.y].len();
    }

    pub fn delete(&mut self) {
        if self.cursor.x < self.lines[self.cursor.y].len() {
            self.lines
                .get_mut(self.cursor.y)
                .expect("Missing line")
                .remove(self.cursor.x);
        } else {
            if self.cursor.y < self.lines.len() - 1 {
                let next_line_content = self.lines[self.cursor.y + 1].clone();
                self.lines
                    .get_mut(self.cursor.y)
                    .expect("Missing line")
                    .push_str(&next_line_content);
                self.lines.remove(self.cursor.y + 1);
            } else {
                // Noop.
            }
        }
    }

    pub fn new_line_adjusted_cursor_position(&self) -> usize {
        let mut pos = 0usize;

        for i in 0..self.cursor.y {
            pos += self.lines[i].len() + 1;
        }

        pos + self.cursor.x
    }

    fn front_space_length(&self, line_index: usize) -> usize {
        let mut n = 0;
        for c in self.lines[line_index].chars() {
            if c != ' ' {
                break;
            }

            n += 1;
        }

        n
    }

    pub fn apply_suggestion(&mut self, suggestion: Suggestion, idx: usize) {
        if idx >= suggestion.elems.len() {
            error!("Suggestion selection index out of bounds");
            return;
        }

        match suggestion.token {
            Some(token) => {
                let token_start_cursor = self.cursor_of_absolute_position(token.pos);
                self.lines[token_start_cursor.y].replace_range(
                    token_start_cursor.x..token_start_cursor.x + token.len,
                    &suggestion.elems[idx],
                );
                self.cursor.x = token_start_cursor.x + suggestion.elems[idx].len();
            }
            None => {
                self.lines[self.cursor.y].insert_str(self.cursor.x, &suggestion.elems[idx]);
                self.cursor.x += suggestion.elems[idx].len();
            }
        };
    }

    fn cursor_of_absolute_position(&self, pos: usize) -> CoordUsize {
        let mut y = 0usize;
        let mut i = 0;

        while i + self.lines[y].len() <= pos {
            i += self.lines[y].len() + 1 /* new line from tokenizer */;
            y += 1;
        }

        debug!("pos={} i={} y={}", pos, i, y);

        CoordUsize { x: pos - i, y }
    }
}
