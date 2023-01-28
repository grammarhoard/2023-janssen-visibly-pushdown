use std::collections::HashMap;

use regex::Regex;

#[derive(Debug, Clone)]
pub struct RegexParser {
    pub regex: Regex,
    pub size: usize,
    pub captures: HashMap<String, Vec<String>>,
    pub original_strings: Vec<String>,
}

impl RegexParser {
    pub fn parse<'a>(&self, input: &'a str) -> Option<(usize, &'a str, HashMap<String, String>)> {
        let captures = self.regex.captures(input)?;
        for i in 0..=self.size {
            if let Some(capture) = captures.name(&format!("RESTRICTED_{i}")) {
                let mut returned_captures: HashMap<String, String> = HashMap::new();
                self.captures
                    .get(&format!("RESTRICTED_{i}"))
                    .unwrap()
                    .iter()
                    .for_each(|name| {
                        returned_captures.insert(
                            name.clone(),
                            captures.name(name).unwrap().as_str().to_owned(),
                        );
                    });
                return Some((i, &input[capture.end()..], returned_captures));
            }
        }
        None
    }
}

impl From<Vec<String>> for RegexParser {
    fn from(regs: Vec<String>) -> Self {
        let mut captures: HashMap<String, Vec<String>> = HashMap::new();
        regs.iter().enumerate().for_each(|(i, r)| {
            let regex = Regex::new(r).unwrap();
            let mut map = Vec::new();
            regex.capture_names().into_iter().for_each(|name| {
                if let Some(name) = name {
                    map.push(name.to_owned());
                }
            });
            // map.push(format!("a{i}"));
            captures.insert(format!("RESTRICTED_{i}"), map);
        });
        let mut res: String = format!(r"^((?P<RESTRICTED_0>{})", regs.get(0).unwrap());
        for (i, reg) in regs.iter().enumerate().skip(1) {
            res.extend(format!(r"|(?P<RESTRICTED_{i}>{reg})").chars());
        }
        res.push(')');
        Self {
            regex: Regex::new(&res).unwrap(),
            size: regs.len(),
            captures,
            original_strings: regs,
        }
    }
}