use std::path::PathBuf;
use std::{cell::RefCell, rc::Rc};

use crate::analyzer::{Analyzer, Suggestion};
use crate::editor_printer::EditorPrinter;
use crate::parser;
use crate::tokenizer::{Token, TokenKind, Tokenizer};
use crate::{stdin_reader::KeyboardInput, text::Text};

#[derive(Debug, PartialEq)]
enum State {
    Edit,
    SuggestionSelect,
}

pub struct Editor {
    content: Rc<RefCell<Text>>,
    analyzer: Analyzer,
    state: State,
    previous_suggestion: Option<Suggestion>,
    printer: EditorPrinter,
}

impl Editor {
    pub fn new(content: Rc<RefCell<Text>>) -> Editor {
        Editor {
            content,
            analyzer: Analyzer::new(),
            state: State::Edit,
            previous_suggestion: None,
            printer: EditorPrinter::new(),
        }
    }

    pub fn update(&mut self, input: KeyboardInput) {
        match input {
            KeyboardInput::Key(15) => {
                // CTRL-O
                self.state = State::SuggestionSelect;
            }
            KeyboardInput::Key(code) => {
                if self.state == State::SuggestionSelect {
                    if code >= b'0' && code <= b'9' {
                        self.content.borrow_mut().apply_suggestion(
                            self.previous_suggestion.clone().unwrap(),
                            (code - b'0') as usize,
                        );
                    }
                    self.state = State::Edit;
                } else {
                    self.handle_char_input(code);
                }
            }
            KeyboardInput::Left => self.content.borrow_mut().move_cursor_left(),
            KeyboardInput::Right => self.content.borrow_mut().move_cursor_right(),
            KeyboardInput::Up => self.content.borrow_mut().move_cursor_up(),
            KeyboardInput::Down => self.content.borrow_mut().move_cursor_down(),
            KeyboardInput::Delete => self.content.borrow_mut().delete(),
            KeyboardInput::Home => self.content.borrow_mut().move_cursor_to_home(),
            KeyboardInput::End => self.content.borrow_mut().move_cursor_to_end(),
            KeyboardInput::AltDigit(digit) => {
                self.content
                    .borrow_mut()
                    .apply_suggestion(self.previous_suggestion.clone().unwrap(), digit as usize);
            }
            _ => {
                debug!("Unrecognized editor input: {:?}", input);
            }
        }

        self.refresh_screen();
    }

    fn handle_char_input(&mut self, ch: u8) {
        match ch {
            127 => self.content.borrow_mut().backspace(),
            13 => self.content.borrow_mut().insert_new_line(),
            9 => self.content.borrow_mut().insert_tab(),
            ch => self.content.borrow_mut().insert_visible_char(ch as char),
        }
    }

    pub fn refresh_screen(&mut self) {
        let tokens = self.build_tokens();
        let tokens_without_whitecpace = tokens
            .clone()
            .into_iter()
            .filter(|token| match token.kind {
                TokenKind::Whitespace(_) => false,
                TokenKind::LineBreak => false,
                _ => true,
            })
            .collect::<Vec<_>>();

        let mut parse_error = None;
        let mut suggestions = None;
        let mut definition_error = None;
        match parser::Parser::new(tokens_without_whitecpace).parse() {
            Ok(root) => {
                match self.analyzer.analyze(
                    root,
                    self.content.borrow().new_line_adjusted_cursor_position(),
                ) {
                    Ok(ok) => {
                        self.previous_suggestion = ok.clone();
                        suggestions = ok;
                    }
                    Err(err) => definition_error = Some(err),
                };
            }
            Err(err) => parse_error = Some(err),
        }

        self.printer.print(
            tokens,
            self.content.borrow().cursor.clone(),
            suggestions,
            parse_error,
            definition_error,
            self.state == State::SuggestionSelect,
        );
    }

    fn build_tokens(&self) -> Vec<Token> {
        Tokenizer::tokenize_lines(&self.content.borrow().lines, true)
    }
}
