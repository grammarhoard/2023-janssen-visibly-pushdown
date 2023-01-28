mod builder;
mod recognizer_automaton;
mod parse_tree;

use std::collections::HashMap;

use crate::regex::RegexParser;

use self::recognizer_automaton::Action;

pub(crate) type State = usize;
pub(crate) type NextMap = HashMap<State, (RegexParser, Vec<Action>)>;

pub use recognizer_automaton::Recognizer;
pub use parse_tree::{Child, Node, ParseTree, Tree};