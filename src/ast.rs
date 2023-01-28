#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grammar {
    pub words: Vec<Nonterminal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nonterminal {
    pub identifier: String,
    pub rules: Vec<Rule>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub source: RuleSource,
    pub transform: RuleTransform,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleTransform {
    pub regs: Vec<RuleTransformItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleTransformItem {
    String(String),
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSource {
    pub regs: Vec<RuleSourceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleSourceItem {
    RegexString(String),
    Identifier(Identifier),
    Nested(Nested),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nested {
    pub call_symbol: String,
    pub return_symbol: String,
    pub rule: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub internal: String,
    pub external: String,
}
