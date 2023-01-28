use std::{collections::HashMap, cell::RefCell};

use crate::{ir::*, regex::RegexParser, Recognizer};

use super::{recognizer_automaton::Action, State, NextMap};

pub(crate) struct RecognizerBuilder<'grammar> {
    epsilon_rules: HashMap<(String, State), State>,
    state: usize,
    state_to_rule: HashMap<State, (usize, usize)>,
    grammar: &'grammar Grammar,
    next_state: NextMap,
}

type Indexed<T> = (T, usize);

/// Builds a recognizer based on the given Grammar
impl<'grammar> RecognizerBuilder<'grammar> {
    pub(crate) fn new(grammar: &'grammar Grammar) -> Self {
        Self {
            grammar,
            epsilon_rules: HashMap::default(),
            state: usize::default(),
            state_to_rule: HashMap::default(),
            next_state: NextMap::default(),
        }
    }

    pub(crate) fn build(mut self) -> Recognizer {
        self.state += 1;
        for (index, nonterminal) in self.grammar.nonterminals.iter().enumerate().rev() {
            self.build_nonterminal(index, nonterminal);
        }
        self.build_epsilon_state();
        Recognizer {
            stack: RefCell::new(Vec::new()),
            state: 1,
            next_state: self.next_state,
            nonterminals_length: self.grammar.nonterminals.iter().len(),
        }
    }

    fn collect_starting_rules(&self, identifier: usize, rule_nr: usize) -> Vec<(String, Action)> {
        // Get the index of the word in the grammar
        let actions = &self.next_state[&(identifier + 1)];
        actions
            .1
            .iter()
            .enumerate()
            .map(|(i, a)| {
                (
                    actions.0.original_strings.get(i).unwrap().clone(),
                    add_id_to_action(a.clone(), identifier, rule_nr),
                )
            })
            .collect()
    }

    /// Builds the epsilon state, which has all nested returns.
    fn build_epsilon_state(&mut self) {
        if !self.epsilon_rules.is_empty() {
            let mapped_returns = self.epsilon_rules.iter().fold(
                HashMap::new(),
                |mut map, ((return_symbol, original_state), next_state)| {
                    map.entry(return_symbol)
                        .or_insert(HashMap::new())
                        .insert(*original_state, *next_state);
                    map
                },
            );
            self.next_state.insert(
                0,
                (
                    RegexParser::from(
                        mapped_returns
                            .keys()
                            .map(|&s| s.clone())
                            .collect::<Vec<String>>(),
                    ),
                    mapped_returns
                        .values()
                        .map(|map| Action::Return(map.clone()))
                        .collect::<Vec<Action>>(),
                ),
            );
        }
    }

    /// Builds a rule of a gramamr
    fn build_rule(
        &mut self,
        (nonterminal, nt_index): Indexed<&Nonterminal>,
        (rule, rule_index): Indexed<&Rule>,
    ) -> usize {
        let (mut next_state, mut skip_one) = match rule.source.items.last().unwrap() {
            RuleSourceItem::Identifier(i) => {
                if rule.source.items.len() == 1 {
                    // If the first and only item of a rule is an identifier, skip building that rule.
                    (0, true)
                } else {
                    // If the last rule is an identifier, the next_state of the preceding item will be the starting state of that rule.
                    (i.source, true)
                }
            }
            // If it is Nested or Regular, don't skip it.
            _ => (0, false),
        };

        for (item_index, item) in rule.source.items.iter().enumerate().skip(1).rev() {
            if skip_one {
                skip_one = false;
                continue;
            }
            next_state = self.build_rule_item(
                next_state,
                (nonterminal, nt_index),
                (rule, rule_index),
                (item, item_index),
            );
        }
        next_state
    }

    /// Builds a grammar rule item.
    fn build_rule_item(
        &mut self,
        next_state: usize,
        (nonterminal, nt_index): Indexed<&Nonterminal>,
        (_, rule_index): Indexed<&Rule>,
        (item, item_index): Indexed<&RuleSourceItem>,
    ) -> usize {
        let mut next_state = next_state;
        match item {
            RuleSourceItem::RegexString(s) => {
                self.next_state.insert(
                    self.state,
                    (
                        RegexParser::from(vec![s.clone()]),
                        vec![Action::Next(
                            next_state,
                            if next_state <= self.grammar.nonterminals.len() {
                                vec![(next_state - 1, 0)]
                            } else {
                                vec![]
                            },
                        )],
                    ),
                );
                next_state = self.state;
            }
            RuleSourceItem::Nested(n) => {
                let test = if item_index == 0 {
                    nt_index + 1
                } else {
                    self.state
                };
                self.epsilon_rules
                    .insert((n.return_symbol.clone(), test), next_state);
                if item_index == 0 {
                    next_state = n.nonterminal.source;
                } else {
                    next_state = self.state;
                    let index = n.nonterminal.source;
                    self.next_state.insert(
                        self.state,
                        (
                            RegexParser::from(vec![n.call_symbol.clone()]),
                            vec![Action::Call(test, index, vec![(index - 1, 0)])],
                        ),
                    );
                }
            }
            RuleSourceItem::Identifier(_) => unreachable!("Identifiers are not allowed to be any rule other than the last"),
        }
        self.state_to_rule
            .insert(self.state, (nonterminal.identifier, rule_index));
        self.state += 1;
        next_state
    }

    /// Build all rules of a nonterminal
    fn build_nonterminal(&mut self, nt_index: usize, nonterminal: &Nonterminal) {
        let mut next_states: Vec<State> = Vec::new();
        for (rule_index, rule) in nonterminal.rules.iter().enumerate() {
            next_states.push(self.build_rule((nonterminal, nt_index), (rule, rule_index)))
        }
        let starting_regexes: Vec<(String, Action)> = nonterminal
            .rules
            .iter()
            .map(|rule| rule.source.items.get(0).unwrap())
            .enumerate()
            .flat_map(|(rule_index, reg)| match reg {
                RuleSourceItem::RegexString(s) => {
                    let next_state = next_states[rule_index];
                    if next_state <= self.grammar.nonterminals.len() && next_state != 0 {
                        vec![(
                            s.clone(),
                            Action::Next(next_state, vec![(next_state - 1, 0)]),
                        )]
                    } else {
                        vec![(s.clone(), Action::Next(next_state, vec![]))]
                    }
                }
                RuleSourceItem::Nested(n) => {
                    let index = n.nonterminal.source;
                    vec![(
                        n.call_symbol.clone(),
                        Action::Call(index + 1, index, vec![(index - 1, 0)]),
                    )]
                }
                RuleSourceItem::Identifier(id) => {
                    let index = id.source;
                    self.collect_starting_rules(index - 1, rule_index)
                }
            })
            .collect();
        self.next_state.insert(
            nonterminal.identifier,
            (
                RegexParser::from(
                    starting_regexes
                        .iter()
                        .map(|x| x.0.clone())
                        .collect::<Vec<String>>(),
                ),
                starting_regexes.iter().map(|x| x.1.clone()).collect(),
            ),
        );
    }
}

fn add_id_to_action(action: Action, identifier: usize, rule_nr: usize) -> Action {
    let edited_action = match action {
        Action::Call(a, b, id) => {
            let mut new_id = id;
            new_id.push((identifier, rule_nr));
            Action::Call(a, b, new_id)
        }
        Action::Next(a, id) => {
            let mut new_id = id;
            new_id.push((identifier, rule_nr));
            Action::Next(a, new_id)
        }
        Action::Return(_) => {
            unreachable!("Return action should not be present in the first rule of a word")
        }
    };
    edited_action
}