use std::collections::HashMap;
use std::collections::HashSet;

use crate::ast;
use crate::ir;

pub type ElaborationError = String;

type ElaborationResult<T> = Result<T, ElaborationError>;

pub fn elaborate(grammar: ast::Grammar) -> ElaborationResult<ir::Grammar> {
    // Check if all used words are defined
    let grammar = check_definitions(grammar)?;

    // Check if all rules are nested or only use subsequent rules
    let grammar = check_rule_order(grammar)?;

    Ok(ir::Grammar::from(&grammar))
}

fn convert_to_hashmap(grammar: &ast::Grammar) -> HashMap<String, (usize, &ast::Nonterminal)> {
    let mut map: HashMap<String, (usize, &ast::Nonterminal)> = HashMap::new();
    for word in grammar.words.iter().enumerate() {
        map.insert(word.1.identifier.clone(), word);
    }
    map
}

fn check_definitions(grammar: ast::Grammar) -> ElaborationResult<ast::Grammar> {
    let words: HashMap<String, (usize, &ast::Nonterminal)> = convert_to_hashmap(&grammar);
    for word in &grammar.words {
        for rule in &word.rules {
            let mut extern_identifiers = rule
                .transform
                .regs
                .iter()
                .filter_map(|reg| {
                    if let ast::RuleTransformItem::Identifier(id) = reg {
                        Some(id.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
            if extern_identifiers
                .iter()
                .cloned()
                .collect::<HashSet<String>>()
                .len()
                != extern_identifiers.len()
            {
                return Err(format!(
                    "Word {} has a rule with two identicial identifiers in the transform",
                    word.identifier
                ));
            }
            for reg in &rule.source.regs {
                match reg {
                    ast::RuleSourceItem::Identifier(id) => {
                        if !words.contains_key(&id.internal) {
                            return Err(format!("Word {} is not defined", id.internal));
                        }
                        if extern_identifiers.contains(&id.external) {
                            extern_identifiers.retain(|x| x != &id.external);
                        } else {
                            return Err(format!(
                                "Word {} is not used in the transform",
                                id.external
                            ));
                        }
                    }
                    ast::RuleSourceItem::Nested(n) => {
                        if !words.contains_key(&n.rule.internal) {
                            return Err(format!("Word {} is not defined", &n.rule.internal));
                        }
                        if extern_identifiers.contains(&n.rule.external) {
                            extern_identifiers.retain(|x| x != &n.rule.external);
                        } else {
                            return Err(format!(
                                "Word {} is not used in the transform",
                                n.rule.external
                            ));
                        }
                        check_regex(&n.call_symbol, &mut extern_identifiers)?;
                        check_regex(&n.return_symbol, &mut extern_identifiers)?;
                    }
                    ast::RuleSourceItem::RegexString(s) => {
                        check_regex(s, &mut extern_identifiers)?;
                    }
                }
            }
            if !extern_identifiers.is_empty() {
                return Err(format!(
                    "Word {} is not used in the source",
                    extern_identifiers[0]
                ));
            }
        }
    }
    Ok(grammar)
}

fn check_regex(s: &str, extern_identifiers: &mut Vec<String>) -> Result<(), String> {
    let regex = regex::Regex::new(s).unwrap();
    for name in regex.capture_names().flatten() {
        if extern_identifiers.contains(&name.to_string()) {
            extern_identifiers.retain(|x| x != &name.to_string());
        } else {
            return Err(format!("Word {name} is not used in the transform"));
        }
    }
    Ok(())
}

fn check_rule_order(grammar: ast::Grammar) -> ElaborationResult<ast::Grammar> {
    let words: HashMap<String, (usize, &ast::Nonterminal)> = convert_to_hashmap(&grammar);
    for (index, word) in grammar.words.iter().enumerate() {
        for rule in &word.rules {
            for reg in &rule.source.regs {
                if let ast::RuleSourceItem::Identifier(id) = reg {
                    let rule_index = words.get(&id.internal).unwrap().0;
                    if rule_index <= index {
                        return Err(format!(
                            "Rule {} is not defined before rule {}",
                            id.internal, index
                        ));
                    }
                }
            }
        }
    }
    Ok(grammar)
}
