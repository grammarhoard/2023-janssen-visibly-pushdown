use std::collections::HashMap;

use regex::Regex;

use crate::ast::{self};

type Id = usize;
type IRFrom<'a, T> = (&'a ast::Grammar, &'a T);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grammar {
    pub nonterminals: Vec<Nonterminal>,
}

impl From<&ast::Grammar> for Grammar {
    fn from(grammar: &ast::Grammar) -> Self {
        let mut nonterminals: Vec<Nonterminal> = Vec::new();
        for word in &grammar.words {
            nonterminals.push(Nonterminal::from((grammar, word)));
        }
        Self { nonterminals }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nonterminal {
    pub identifier: Id,
    pub rules: Vec<Rule>,
}

impl From<IRFrom<'_, ast::Nonterminal>> for Nonterminal {
    fn from((grammar, word): IRFrom<ast::Nonterminal>) -> Self {
        let mut rules: Vec<Rule> = Vec::new();
        for rule in &word.rules {
            rules.push(Rule::from((grammar, rule)));
        }
        Self {
            identifier: grammar
                .words
                .iter()
                .position(|w| w.identifier == word.identifier)
                .unwrap()
                + 1,
            rules,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub source: RuleSource,
    pub transform: RuleTransform,
}

impl From<IRFrom<'_, ast::Rule>> for Rule {
    fn from((grammar, rule): IRFrom<ast::Rule>) -> Self {
        let external_map = transform_map(rule);
        Self {
            source: RuleSource::from((grammar, &rule.source, &external_map)),
            transform: RuleTransform::from((grammar, &rule.transform, &external_map)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleTransform {
    pub regs: Vec<RuleTransformItem>,
}

impl From<(&ast::Grammar, &ast::RuleTransform, &TransformMap)> for RuleTransform {
    fn from(
        (grammar, rule_transform, external_map): (&ast::Grammar, &ast::RuleTransform, &TransformMap),
    ) -> Self {
        let mut regs: Vec<RuleTransformItem> = Vec::new();
        for reg in &rule_transform.regs {
            regs.push(RuleTransformItem::from((grammar, reg, external_map)));
        }
        Self { regs }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleTransformItem {
    String(String),
    Identifier(Id),
}

impl From<(&ast::Grammar, &ast::RuleTransformItem, &TransformMap)> for RuleTransformItem {
    fn from(
        (_grammar, rule_transform_item, external_map): (
            &ast::Grammar,
            &ast::RuleTransformItem,
            &TransformMap,
        ),
    ) -> Self {
        match rule_transform_item {
            ast::RuleTransformItem::String(string) => Self::String(string.to_string()),
            ast::RuleTransformItem::Identifier(identifier) => {
                Self::Identifier(*external_map.get(identifier).unwrap())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSource {
    pub items: Vec<RuleSourceItem>,
}

impl From<(&ast::Grammar, &ast::RuleSource, &TransformMap)> for RuleSource {
    fn from(
        (grammar, regular, external_map): (&ast::Grammar, &ast::RuleSource, &TransformMap),
    ) -> Self {
        let mut regs: Vec<RuleSourceItem> = Vec::new();
        for reg in &regular.regs {
            regs.push(RuleSourceItem::from((grammar, reg, external_map)));
        }
        Self { items: regs }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleSourceItem {
    RegexString(String),
    Identifier(Identifier),
    Nested(Nested),
}

impl From<(&ast::Grammar, &ast::RuleSourceItem, &TransformMap)> for RuleSourceItem {
    fn from(
        (grammar, reg, external_map): (&ast::Grammar, &ast::RuleSourceItem, &TransformMap),
    ) -> Self {
        match reg {
            ast::RuleSourceItem::RegexString(regex_string) => {
                Self::RegexString(regex_string.clone())
            }
            ast::RuleSourceItem::Identifier(identifier) => {
                Self::Identifier(Identifier::from((grammar, identifier, external_map)))
            }
            ast::RuleSourceItem::Nested(nested) => {
                Self::Nested(Nested::from((grammar, nested, external_map)))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nested {
    pub call_symbol: String,
    pub return_symbol: String,
    pub nonterminal: Identifier,
}
impl From<(&ast::Grammar, &ast::Nested, &TransformMap)> for Nested {
    fn from((grammar, nested, external_map): (&ast::Grammar, &ast::Nested, &TransformMap)) -> Self {
        Self {
            call_symbol: nested.call_symbol.clone(),
            return_symbol: nested.return_symbol.clone(),
            nonterminal: Identifier::from((grammar, &nested.rule, external_map)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub source: Id,
    pub transform: Id,
}

impl From<(&ast::Grammar, &ast::Identifier, &TransformMap)> for Identifier {
    fn from(
        (grammar, identifier, external_map): (&ast::Grammar, &ast::Identifier, &TransformMap),
    ) -> Self {
        let external = *external_map.get(&identifier.external).unwrap();
        Self {
            source: grammar
                .words
                .iter()
                .position(|w| w.identifier == identifier.internal)
                .unwrap()
                + 1,
            transform: external,
        }
    }
}

type TransformMap = HashMap<String, usize>;

fn transform_map(rule: &ast::Rule) -> TransformMap {
    let mut map = HashMap::new();
    let mut i = 0;
    rule.source.regs.iter().for_each(|reg| match reg {
        ast::RuleSourceItem::Identifier(identifier) => {
            map.insert(identifier.external.clone(), i);
            i += 1;
        }
        ast::RuleSourceItem::RegexString(s) => {
            let regex = Regex::new(s).unwrap();
            regex.capture_names().for_each(|name| {
                if let Some(name) = name {
                    map.insert(name.to_string(), i);
                    i += 1;
                }
            });
        }
        ast::RuleSourceItem::Nested(nested) => {
            let regex = Regex::new(&nested.call_symbol).unwrap();
            regex.capture_names().for_each(|name| {
                if let Some(name) = name {
                    map.insert(name.to_string(), i);
                    i += 1;
                }
            });
            map.insert(nested.rule.external.clone(), i);
            let regex = Regex::new(&nested.return_symbol).unwrap();
            regex.capture_names().for_each(|name| {
                if let Some(name) = name {
                    map.insert(name.to_string(), i);
                    i += 1;
                }
            });
        }
    });
    map
}
