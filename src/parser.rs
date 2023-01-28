use nom::{
    bytes::complete::{escaped, is_not, tag},
    character::complete::{anychar, one_of},
    combinator::opt,
    sequence::delimited,
    IResult,
};

use crate::ast::{
    Grammar, Identifier, Nested, Nonterminal, Rule, RuleSource, RuleSourceItem, RuleTransform,
    RuleTransformItem,
};

fn alphanumeric(inp: &str) -> IResult<&str, &str> {
    let (inp, string) = nom::bytes::complete::take_while1(char::is_alphanumeric)(inp)?;
    Ok((inp, string))
}

fn parse_regex(inp: &str) -> IResult<&str, &str> {
    let (inp, _) = tag("\"")(inp)?;
    // let (inp, regex) = many0(alt((preceded(tag("\\"), tag("\"")), is_not(r#"""#))))(inp)?;
    let (inp, regex) = escaped(is_not("\"\\"), '\\', one_of("\"\\"))(inp)?;
    let (inp, _) = tag("\"")(inp)?;
    Ok((inp, regex))
}

fn skip_whitespace(inp: &str) -> IResult<&str, &str> {
    let (inp, whitespace) = nom::bytes::complete::take_while1(char::is_whitespace)(inp)?;
    Ok((inp, whitespace))
}

fn skip_space(inp: &str) -> IResult<&str, &str> {
    let (inp, whitespace) =
        nom::bytes::complete::take_while1(|c: char| c.is_whitespace() && c != '\n')(inp)?;
    Ok((inp, whitespace))
}

impl Grammar {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        let (inp, words) = nom::multi::many1(Nonterminal::parse)(inp)?;
        Ok((inp, Self { words }))
    }
}

impl Nonterminal {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        let (inp, name) = alphanumeric(inp)?;
        let (inp, _) = nom::bytes::complete::tag(":")(inp)?;
        let (inp, _) = skip_whitespace(inp)?;
        let (inp, rules) = nom::multi::separated_list1(skip_whitespace, Rule::parse)(inp)?;
        Ok((
            inp,
            Self {
                identifier: name.to_string(),
                rules,
            },
        ))
    }
}

impl Rule {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        let (inp, source) = RuleSource::parse(inp)?;
        let (inp, _) = skip_space(inp)?;
        let (inp, transform) = RuleTransform::parse(inp)?;
        Ok((inp, Self { source, transform }))
    }
}

impl RuleTransform {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        let (inp, _) = tag("->")(inp)?;
        let (inp, _) = skip_whitespace(inp)?;
        let (inp, regs) = nom::multi::separated_list1(skip_space, RuleTransformItem::parse)(inp)?;
        let (inp, _) = opt(tag("\n"))(inp)?;
        Ok((inp, Self { regs }))
    }
}

impl RuleTransformItem {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        if inp.starts_with('\"') {
            let (inp, regex) = parse_regex(inp)?;
            Ok((inp, Self::String(regex.to_string())))
        } else {
            let (inp, string) = alphanumeric(inp)?;
            Ok((inp, Self::Identifier(string.to_string())))
        }
    }
}

impl Nested {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        let (inp, _) = tag("[")(inp)?;
        // let (inp, _) = skip_space(inp)?;
        let (inp, call_symbol) = delimited(
            tag("\""),
            escaped(is_not(r#"\""#), '\\', anychar),
            tag("\""),
        )(inp)?;
        let (inp, _) = skip_space(inp)?;
        let (inp, id) = Identifier::parse(inp)?;
        let (inp, _) = skip_space(inp)?;
        let (inp, return_symbol) = delimited(
            tag("\""),
            escaped(is_not(r#"\""#), '\\', anychar),
            tag("\""),
        )(inp)?;
        // let (inp, _) = skip_space(inp)?;
        let (inp, _) = tag("]")(inp)?;
        Ok((
            inp,
            Self {
                call_symbol: call_symbol.to_string(),
                rule: id,
                return_symbol: return_symbol.to_string(),
            },
        ))
    }
}

impl RuleSource {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        let (inp, regs) = nom::multi::separated_list1(skip_space, RuleSourceItem::parse)(inp)?;
        Ok((inp, Self { regs }))
    }
}

impl RuleSourceItem {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        if let Ok((inp, nested)) = Nested::parse(inp) {
            return Ok((inp, Self::Nested(nested)));
        }
        if let Ok((inp, regex_string)) = parse_regex(inp) {
            return Ok((inp, Self::RegexString(regex_string.to_string())));
        }
        let (inp, id) = Identifier::parse(inp)?;
        Ok((inp, Self::Identifier(id)))
    }
}

impl Identifier {
    pub fn parse(inp: &str) -> IResult<&str, Self> {
        let (inp, internal) = alphanumeric(inp)?;
        let (inp, _) = nom::character::complete::char('=')(inp)?;
        let (inp, external) = alphanumeric(inp)?;
        Ok((
            inp,
            Self {
                external: external.to_string(),
                internal: internal.to_string(),
            },
        ))
    }
}