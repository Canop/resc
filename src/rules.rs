use regex::{Captures, Regex};
use std::collections::HashMap;
use std::borrow::Cow;


#[derive(Debug)]
pub struct RuleResult {
    pub task: String,
    pub queue: String,
    pub set: String,
}

#[derive(Debug)]
pub struct Rule {
    pub name: String,
    pub on_regex: Regex,
    pub todo_task_pattern: String,
    pub todo_queue_pattern: String,
    pub todo_set_pattern: String,
}

fn inject_groups<'t>(pat: &'t str, props: &HashMap<&str, &str>) -> Cow<'t, str> {
        // optm: locations of groups in output patterns could be computed
        //       when reading configuration and we wouldn't need the name
        //       of the group, only their index
        lazy_static! {
            static ref out_group_regex: Regex = Regex::new(r"\$\{(\w+)\}").unwrap();
        }
        out_group_regex.replace_all(
            pat,
            |caps: &Captures| {
                match props.get(&caps.get(1).unwrap().as_str()) {
                    Some(value) => value,
                    None => &*"-missing group!-"
                }
            }
        )
}

impl Rule {
    fn is_match(&self, task: &String) -> bool {
        self.on_regex.is_match(task)
    }
    // Assumes the rule matches.
    // (there will be more than one result in the future (for example due to genealogy))
    // This could be heavily optimized by analyzing everything when reading the config.
    pub fn results(&self, task: &String) -> Vec<RuleResult> {
        let mut props = HashMap::new();
        let caps = self.on_regex.captures(task).unwrap();
        let mut results = Vec::new();
        for groupname in self.on_regex.capture_names() {
            if let Some(name) = groupname {
                if let Some(value) = caps.name(name) {
                    props.insert(name, value.as_str());
                }
            }
        }
        results.push(RuleResult{
            task: inject_groups(&self.todo_task_pattern, &props).to_string(),
            queue: inject_groups(&self.todo_queue_pattern, &props).to_string(),
            set: inject_groups(&self.todo_set_pattern, &props).to_string(),
        });
        results
    }
}

#[derive(Debug)]
pub struct Ruleset {
    pub rules: Vec<Rule>
}

impl Ruleset {
    pub fn matching_rules(&self, task: &String) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| r.is_match(&task))
            .collect()
    }
}

