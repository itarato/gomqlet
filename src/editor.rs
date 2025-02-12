use std::path::PathBuf;
use std::{cell::RefCell, rc::Rc};

use crate::analyzer::{Analyzer, Suggestion};
use crate::editor_printer::EditorPrinter;
use crate::net_ops::NetOps;
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
    pub fn new(
        content: Rc<RefCell<Text>>,
        net_ops: &NetOps,
        schema_cache_file_path: &PathBuf,
        reload_schema: bool,
    ) -> Editor {
        Editor {
            content,
            analyzer: Analyzer::new(&net_ops, schema_cache_file_path, reload_schema),
            state: State::Edit,
            previous_suggestion: None,
            printer: EditorPrinter::new(),
        }
    }

    pub fn update(&mut self, input: KeyboardInput) {
        match input {
            KeyboardInput::CtrlO => {
                // CTRL-O
                self.state = State::SuggestionSelect;
            }
            KeyboardInput::CtrlR => {
                // CTRL-R
                self.printer.reload_terminal_size();
            }
            KeyboardInput::VisibleChar(code) => {
                if self.state == State::SuggestionSelect {
                    if code >= b'0' && code <= b'9' {
                        self.content.borrow_mut().apply_suggestion(
                            self.previous_suggestion.as_ref().unwrap(),
                            (code - b'0') as usize,
                        );
                    }
                    self.state = State::Edit;
                } else {
                    self.content.borrow_mut().insert_visible_char(code as char);
                }
            }
            KeyboardInput::Left => self.content.borrow_mut().move_cursor_left(),
            KeyboardInput::Right => self.content.borrow_mut().move_cursor_right(),
            KeyboardInput::Up => self.content.borrow_mut().move_cursor_up(),
            KeyboardInput::Down => self.content.borrow_mut().move_cursor_down(),
            KeyboardInput::Delete => self.content.borrow_mut().delete(),
            KeyboardInput::Home => self.content.borrow_mut().move_cursor_to_home(),
            KeyboardInput::End => self.content.borrow_mut().move_cursor_to_end(),
            KeyboardInput::AltS | KeyboardInput::CtrlS => self.content.borrow_mut().save_to_file(),
            KeyboardInput::Enter => self.content.borrow_mut().insert_new_line(),
            KeyboardInput::Backspace => self.content.borrow_mut().backspace(),
            KeyboardInput::Tab => self.content.borrow_mut().insert_tab(),
            KeyboardInput::CtrlSlash => self.content.borrow_mut().toggle_comment(),
            KeyboardInput::AltDigit(digit) => {
                self.content
                    .borrow_mut()
                    .apply_suggestion(self.previous_suggestion.as_ref().unwrap(), digit as usize);
            }
            KeyboardInput::CtrlW => self.content.borrow_mut().delete_word(),
            _ => {
                warn!("Unrecognized editor input: {:?}", input);
            }
        }

        self.refresh_screen();
    }

    pub fn refresh_screen(&mut self) {
        let tokens = self.build_tokens();
        let tokens_without_whitespace = tokens
            .clone()
            .into_iter()
            .filter(|token| match token.kind {
                TokenKind::Whitespace(_) => false,
                TokenKind::LineBreak => false,
                TokenKind::Comment => false,
                _ => true,
            })
            .collect::<Vec<_>>();

        let mut parse_error = None;
        let mut suggestions = None;
        let mut definition_error = None;
        match parser::Parser::new(tokens_without_whitespace).parse() {
            Ok(root) => {
                match self.analyzer.analyze(
                    root,
                    self.content.borrow().new_line_adjusted_cursor_position(),
                ) {
                    Ok(ok) => {
                        self.previous_suggestion = ok.clone();
                        suggestions = ok;
                    }
                    Err(err) => definition_error = Some(err.to_string()),
                };
            }
            Err(err) => parse_error = Some(err),
        }

        self.printer.print(
            tokens,
            &self.content.borrow().cursor,
            suggestions,
            parse_error,
            definition_error,
            self.state == State::SuggestionSelect,
            &self.content.borrow().file_path,
            self.content.borrow().is_file_saved,
        );
    }

    fn build_tokens(&self) -> Vec<Token> {
        Tokenizer::tokenize_lines(&self.content.borrow().lines, true)
    }
}
