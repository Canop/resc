/// Patterns are built from strings like "bla ${some_var} ${some.otherone} bla"
/// and are expanded with HashMap<String, String>
///
use regex::{Captures, Regex};
use std::collections::HashMap;

// optm: locations of groups in output patterns could be computed
//       when initializing
#[derive(Debug)]
pub struct Pattern {
    pub src: String,
}

impl Pattern {
    pub fn inject<'a>(&self, props: &HashMap<String, String>) -> String {
        lazy_static! {
            static ref out_group_regex: Regex = Regex::new(r"\$\{([\w.]+)\}").unwrap();
        }
        out_group_regex
            .replace_all(&*self.src, |caps: &Captures| {
                match props.get(&*caps.get(1).unwrap().as_str()) {
                    Some(value) => value,
                    None => &*"-missing group!-", // we'll probably panic later on
                }
            }).to_string()
    }
}
