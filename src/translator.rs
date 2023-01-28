use std::error::Error;

use crate::{
    ir::*,
    recognizer::{Recognizer, ParseTree},
};

#[derive(Debug)]
pub struct Translator {
    recognizer: Recognizer,
    grammar: Grammar,
}

/// Struct that can translate an input text with a specified transformation.
///
/// This struct can translate a subset of VPLs in linear time.
impl Translator {
    /// Creates a new VPL translator based on the input grammar.
    ///
    /// This parses the grammar to an AST, checks if it holds to all requirements, and builds a translator for that grammar.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// use vpl_parser_generator::Tranlator;
    ///
    /// let string = fs::read_to_string("examples/a-lang.vpa").unwrap();
    /// let mut translator = Translator::new(&string).unwrap();
    /// ```
    pub fn new(string: &str) -> Result<Self, Box<dyn Error + '_>> {
        let (rest_string, grammar) = crate::ast::Grammar::parse(string)?;
        if !rest_string.is_empty() {
            return Err("Error, entire file not parsed".into());
        }
        let elaborated = crate::elaborator::elaborate(grammar)?;
        let recognizer = crate::recognizer::Recognizer::from(&elaborated);
        Ok(Translator {
            recognizer,
            grammar: elaborated,
        })
    }

    fn terminal_size(&self) -> usize {
        self.grammar
            .nonterminals
            .iter()
            .flat_map(|w| w.rules.iter())
            .map(|r| {
                r.transform
                    .regs
                    .iter()
                    .filter_map(|r| match r {
                        RuleTransformItem::String(s) => Some(s.len()),
                        _ => None,
                    })
                    .sum::<usize>()
            })
            .sum::<usize>()
    }

    pub fn translate(&mut self, text: &str) -> Option<String> {
        let (parse_tree, mut size) = self.recognizer.parse(text)?;
        size += self.terminal_size();
        let mut current_node = parse_tree;
        let mut result: String = String::with_capacity(size);
        let mut queue: Vec<TranslateAction> = Vec::new();
        self.add_rule_source_items(&current_node, &mut queue);
        while let Some(action) = queue.pop() {
            match action {
                TranslateAction::Parent => {
                    current_node = current_node.parent().unwrap();
                }
                TranslateAction::ParseRuleTransformItem(RuleTransformItem::Identifier(index)) => {
                    let value = current_node.child(index).unwrap();
                    match value {
                        crate::recognizer::Child::Node(n) => {
                            current_node = n.clone();
                            self.add_rule_source_items(&current_node, &mut queue);
                        }
                        crate::recognizer::Child::Leaf(l) => {
                            result.push_str(&l);
                        }
                    }
                }
                TranslateAction::ParseRuleTransformItem(RuleTransformItem::String(s)) => {
                    result.push_str(&s);
                }
            }
        }
        Some(result)
    }

    /// Adds all rule transform items of the current node to the queue
    fn add_rule_source_items(
        &mut self,
        current_node: &std::rc::Rc<std::cell::RefCell<crate::recognizer::Node>>,
        queue: &mut Vec<TranslateAction>,
    ) {
        let n = &mut *current_node.borrow_mut();
        let rule = self.grammar.nonterminals[n.identifier].rules[n.rule_nr].clone();
        queue.push(TranslateAction::Parent);
        rule.transform
            .regs
            .iter()
            .rev()
            .for_each(|r| queue.push(TranslateAction::ParseRuleTransformItem(r.clone())));
    }
}

enum TranslateAction {
    Parent,
    ParseRuleTransformItem(RuleTransformItem),
}
