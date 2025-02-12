use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use crate::{analyzer::Suggestion, util::CoordUsize};

const TAB_SIZE: usize = 2;

enum WordDeleteMode {
    Whitespace,
    Alphanumberic,
    Other,
}

pub struct Text {
    pub lines: Vec<String>,
    pub cursor: CoordUsize,
    pub file_path: Option<PathBuf>,
    pub is_file_saved: bool,
}

impl Text {
    pub fn new(file_path: Option<PathBuf>) -> Text {
        let lines = if let Some(file_path) = &file_path {
            let mut file = File::open(file_path).expect("Cannot load source file");
            let mut source = String::new();

            file.read_to_string(&mut source)
                .expect("Failed reading content of source");

            source.lines().map(|slice| slice.to_string()).collect()
        } else {
            vec![String::new()]
        };

        Text {
            lines,
            cursor: CoordUsize { x: 0, y: 0 },
            file_path,
            is_file_saved: true,
        }
    }

    pub fn reload_from_file(&mut self, path: PathBuf) {
        let mut file = File::open(&path).expect("Cannot load source file");
        let mut content = String::new();

        file.read_to_string(&mut content)
            .expect("Failed reading content of source");

        self.lines = content.lines().map(|slice| slice.to_string()).collect();
        if self.lines.is_empty() {
            self.lines = vec![String::new()];
        }

        self.cursor = CoordUsize { x: 0, y: 0 };
        self.file_path = Some(path);
        self.is_file_saved = true;
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn save_to_file(&mut self) {
        let mut file = File::create(self.file_path.as_ref().expect("Missing file"))
            .expect("Failed opening file to save.");
        file.write_all(self.to_string().as_bytes())
            .expect("Failed writing to file");

        self.is_file_saved = true;

        info!("File has been saved to {:?}", self.file_path);
    }

    pub fn insert_new_line(&mut self) {
        let mut fragment_to_split = self.lines[self.cursor.y][self.cursor.x..].to_owned();
        self.lines
            .get_mut(self.cursor.y)
            .expect("Missing line")
            .truncate(self.cursor.x);

        let new_line_spaces_len = self.front_space_length(self.cursor.y);
        let new_prefix = String::from_utf8(vec![b' '; new_line_spaces_len]).unwrap();
        fragment_to_split.insert_str(0, &new_prefix);

        self.lines.insert(self.cursor.y + 1, fragment_to_split);

        self.cursor.x = new_line_spaces_len;
        self.cursor.y += 1;

        self.is_file_saved = false;
    }

    pub fn insert_visible_char(&mut self, ch: char) {
        if (ch as u8) < 32 || (ch as u8) > 126 {
            trace!("Command character received: {}", ch as u8);
            return;
        }

        self.lines
            .get_mut(self.cursor.y)
            .expect("Missing line")
            .insert(self.cursor.x, ch);

        self.cursor.x += 1;

        let opposite = match ch {
            '[' => Some(']'),
            '(' => Some(')'),
            '{' => Some('}'),
            _ => None,
        };
        if let Some(opposite) = opposite {
            self.lines
                .get_mut(self.cursor.y)
                .expect("Missing line")
                .insert(self.cursor.x, opposite);
        }

        self.is_file_saved = false;
    }

    pub fn insert_tab(&mut self) {
        let remaining_spaces = TAB_SIZE - (self.cursor.x % TAB_SIZE);
        for _ in 0..remaining_spaces {
            self.insert_visible_char(' ');
        }

        self.is_file_saved = false;
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

        self.is_file_saved = false;
    }

    pub fn delete_word(&mut self) {
        let mut i = self.cursor.x as i32 - 1;
        if i < 0 {
            self.backspace();
            return;
        }

        let chars = self.lines[self.cursor.y].chars().collect::<Vec<_>>();
        let delete_mode = if chars[i as usize].is_ascii_alphanumeric() {
            WordDeleteMode::Alphanumberic
        } else if chars[i as usize].is_ascii_whitespace() {
            WordDeleteMode::Whitespace
        } else {
            WordDeleteMode::Other
        };

        while i >= 0 {
            let ch = chars[i as usize];

            let should_stop = match delete_mode {
                WordDeleteMode::Alphanumberic => !ch.is_ascii_alphanumeric(),

                WordDeleteMode::Whitespace => !ch.is_ascii_whitespace(),
                WordDeleteMode::Other => ch.is_alphanumeric() || ch.is_whitespace(),
            };
            if should_stop {
                break;
            }

            self.lines[self.cursor.y].remove(i as usize);
            i -= 1;
            self.cursor.x -= 1;
        }

        self.is_file_saved = false;
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

        self.is_file_saved = false;
    }

    pub fn toggle_comment(&mut self) {
        if self.has_comment_on_current_line() {
            self.lines[self.cursor.y] = self.lines[self.cursor.y][2..].into()
        } else {
            self.lines[self.cursor.y].insert_str(0, "//");
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

    pub fn apply_suggestion(&mut self, suggestion: &Suggestion, idx: usize) {
        if idx >= suggestion.elems.len() {
            error!("Suggestion selection index out of bounds");
            return;
        }

        match &suggestion.token {
            Some(token) => {
                let token_start_cursor = self.cursor_of_absolute_position(token.pos);
                let line_len = self.lines[token_start_cursor.y].len();
                self.lines[token_start_cursor.y].replace_range(
                    token_start_cursor.x..(token_start_cursor.x + token.len).min(line_len),
                    &suggestion.elems[idx].name,
                );
                self.cursor.x = token_start_cursor.x + suggestion.elems[idx].name.len();
            }
            None => {
                self.lines[self.cursor.y].insert_str(self.cursor.x, &suggestion.elems[idx].name);
                self.cursor.x += suggestion.elems[idx].name.len();
            }
        };

        self.is_file_saved = false;
    }

    fn cursor_of_absolute_position(&self, pos: usize) -> CoordUsize {
        let mut y = 0usize;
        let mut i = 0;

        // Using `<` instead of `<=` to avoid end-of-line position (the X coord after the last char, which is out of index [~hidden new line]).
        while i + self.lines[y].len() < pos {
            i += self.lines[y].len() + 1 /* new line from tokenizer */;
            y += 1;
        }

        CoordUsize { x: pos - i, y }
    }

    fn has_comment_on_current_line(&self) -> bool {
        if self.lines[self.cursor.y].len() < 2 {
            return false;
        }

        &self.lines[self.cursor.y][0..=1] == "//"
    }
}

#[cfg(test)]
mod test {
    use super::Text;

    #[test]
    fn test_minimal_flow() {
        let mut text = Text::new(None);

        assert_eq!(vec![""], text.lines);

        text.insert_visible_char('a');
        text.insert_visible_char('b');
        text.insert_visible_char('c');

        assert_eq!(vec!["abc"], text.lines);

        text.move_cursor_left();
        text.insert_visible_char('d');

        assert_eq!(vec!["abdc"], text.lines);
    }
}
