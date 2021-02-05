use {
    lazy_static::lazy_static,
    regex::{Captures, Regex},
    serde::{Deserialize, Deserializer},
    std::collections::HashMap,
};

/// Patterns are built from strings like "bla ${some_var} ${some.otherone} bla"
/// and are expanded with HashMap<String, String>
#[derive(Debug, Clone)]
pub struct Pattern {
    pub src: String,
}

impl Pattern {
    pub fn inject(&self, props: &HashMap<String, String>) -> String {
        lazy_static! {
            static ref OUT_GROUP_REGEX: Regex = Regex::new(r"\$\{([\w.]+)\}").unwrap();
        }
        OUT_GROUP_REGEX
            .replace_all(&*self.src, |caps: &Captures| {
                match props.get(&*caps.get(1).unwrap().as_str()) {
                    Some(value) => value,
                    None => &*"-missing group!-", // we'll probably panic later on
                }
            })
            .to_string()
    }
}

impl<'de> Deserialize<'de> for Pattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let src = String::deserialize(deserializer)?;
        Ok(Self { src })
    }
}
