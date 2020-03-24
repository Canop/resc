use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;

/// Patterns are built from strings like "bla ${some_var} ${some.otherone} bla"
/// and are expanded with HashMap<String, String>
///
#[derive(Debug)]
pub struct Pattern {
    pub src: String,
}

// optm: locations of groups in output patterns could be computed
//       when initializing
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
            }).to_string()
    }
}
