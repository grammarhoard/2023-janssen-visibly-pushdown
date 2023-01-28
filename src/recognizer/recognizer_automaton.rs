use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    rc::{Rc},
};

use crate::{ir::*};

use super::{builder::RecognizerBuilder, NextMap, Tree, Node, ParseTree};

type State = usize;
type Identifier = usize;
#[derive(Debug, Clone)]
pub(crate) enum Action {
    Call(State, State, Vec<(Identifier, usize)>),
    Next(State, Vec<(Identifier, usize)>),
    Return(HashMap<State, State>),
}

#[derive(Debug, Clone)]
pub(crate) enum ActionType {
    Call(Vec<(usize, usize)>),
    Return(),
    Next(Vec<(usize, usize)>),
}

/// Struct that can recognize and parse an input text to a specific parse tree.
/// 
/// This struct can recognize or parse a subset of VPLs in linear time.
#[derive(Debug)]
pub struct Recognizer {
    pub(crate) stack: RefCell<Vec<State>>,
    pub(crate) state: State,
    pub(crate) next_state: NextMap,
    pub(crate) nonterminals_length: usize,
}


impl From<&Grammar> for Recognizer {
    fn from(grammar: &Grammar) -> Self {
        let builder = RecognizerBuilder::new(grammar);
        builder.build()
    }
}


impl Recognizer {
    /// Creates a new VPL recognizer based on the input grammar.
    ///
    /// This parses the grammar to an AST, checks if it holds to all requirements, and builds a recognizer for that grammar
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// use vpl_parser_generator::Recognizer;
    ///
    /// let string = fs::read_to_string("examples/a-lang.vpa").unwrap();
    /// let mut correct_recognizer = Recognizer::new(&string);
    /// assert!(correct_recognizer.is_ok());
    /// let string = fs::read_to_string("examples/invalid-lang.vpa").unwrap();
    /// let mut incorrect_recognizer = Recognizer::new(&string);
    /// assert!(incorrect_recognizer.is_err());
    /// ```
    pub fn new(string: &str) -> Result<Self, Box<dyn Error + '_>> {
        let (rest_string, grammar) = crate::ast::Grammar::parse(string)?;
        if !rest_string.is_empty() {
            return Err("Error, entire file not parsed".into());
        }
        let elaborated = crate::elaborator::elaborate(grammar)?;
        Ok(crate::recognizer::Recognizer::from(&elaborated))
    }

    pub(crate) fn push(&self, state: State) {
        self.stack.borrow_mut().push(state);
    }

    pub(crate) fn pop(&self) -> Option<State> {
        self.stack.borrow_mut().pop()
    }

    pub(crate) fn next_state<'a>(
        &mut self,
        text: &'a str,
    ) -> Option<(&'a str, HashMap<String, String>, ActionType, usize)> {
        let (regex, actions) = self.next_state.get(&self.state)?;
        let (matches, rest_text, captures) = regex.parse(text)?;
        let action = actions.get(matches)?;
        match action {
            Action::Call(state, next, _) => {
                self.push(*state);
                self.state = *next;
            }
            Action::Next(next, _) => {
                self.state = *next;
            }
            Action::Return(map) => {
                let orig_state = &self.pop()?;
                let next = map.get(orig_state)?;
                self.state = *next;
            }
        }
        let id = match action {
            Action::Call(_, _, id) => ActionType::Call(id.clone()),
            Action::Next(_, id) => ActionType::Next(id.clone()),
            Action::Return(_) => ActionType::Return(),
        };
        Some((rest_text, captures, id, matches))
    }

    /// Parses an input text to an AST, and gives the total size of the strings of the nonterminals.
    /// 
    /// This parses an input text to an abstract syntax Tree, with a size_hint for the resulting translation.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::fs;
    /// use vpl_parser_generator::Recognizer;
    ///
    /// let string = fs::read_to_string("examples/a-lang.vpa").unwrap();
    /// let mut recognizer = Recognizer::new(&string).unwrap();
    /// assert!(recognizer.parse("aaaaa").is_some());
    /// assert!(recognizer.parse("b").is_none());
    /// ```
    pub fn parse(&mut self, text: &str) -> Option<(Tree, usize)> {
        let mut rest_text = text;
        let mut size = 0;
        let root = Rc::new(RefCell::new(Node {
            children: Vec::new(),
            identifier: 0,
            rule_nr: 0,
            parent: None,
        }));
        let mut current_tree = root.clone();
        let mut call_stack = Vec::new();
        let mut previous_state = 1;
        while let Some((text, captures, action, matches)) = self.next_state(rest_text) {
            rest_text = text;
            if previous_state <= self.nonterminals_length && previous_state > 0 {
                current_tree.borrow_mut().rule_nr = matches;
            }
            match action {
                ActionType::Call(i) => {
                    i.iter()
                        .take(i.iter().len() - 1)
                        .for_each(|(identifier, rule_nr)| {
                            call_stack.push(Call::Identifier);
                            current_tree =
                                current_tree.add_node(*identifier, *rule_nr).unwrap();
                        });
                    call_stack.push(Call::Nested);
                    current_tree = current_tree.add_node(
                        i.last().unwrap().0,
                        i.last().unwrap().1,
                    )
                    .unwrap();
                }
                ActionType::Next(i) => {
                    i.iter().rev().for_each(|(identifier, rule_nr)| {
                        call_stack.push(Call::Identifier);
                        current_tree =
                            current_tree.add_node(*identifier, *rule_nr).unwrap();
                    });
                }
                ActionType::Return() => {
                    call_stack.pop();
                    current_tree = current_tree.parent()?;
                }
            }
            captures.into_iter().for_each(|(_k, v)| {
                current_tree.add_leaf(&v);
                size += v.len();
            });
            if self.state == 0 {
                while !call_stack.is_empty() && *call_stack.last()? == Call::Identifier {
                    call_stack.pop();
                    current_tree = current_tree.parent()?;
                }
            }

            if rest_text.is_empty() {
                break;
            }
            previous_state = self.state;
        }
        let result = self.accepting_state() && rest_text.is_empty();
        self.reset();
        if result {
            Some((root, size))
        } else {
            None
        }
    }

    pub fn recognize(&mut self, text: &str) -> Option<()> {
        let mut rest_text = text;
        while let Some((text, _, _, _)) = self.next_state(rest_text) {
            rest_text = text;
            if rest_text.is_empty() {
                break;
            }
        }
        let result = self.accepting_state() && rest_text.is_empty();
        self.reset();
        if result {
            Some(())
        } else {
            None
        }
    }

    pub(crate) fn reset(&mut self) {
        self.stack.borrow_mut().clear();
        self.state = 1;
    }

    pub(crate) fn accepting_state(&self) -> bool {
        self.state == 0 && self.stack.borrow().is_empty()
    }
}

#[derive(Debug, PartialEq)]
enum Call {
    Nested,
    Identifier,
}
