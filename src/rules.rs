
use regex::Regex;

pub struct Rule {
    pub name: &'static str,
    on_regex: Regex,
}

impl Rule {
    fn is_match(&self, task: &String) -> bool {
        self.on_regex.is_match(task)
    }
}


fn build_rule(name: &'static str, on_pattern: &'static str) -> Rule {
    Rule {
        name,
        on_regex: Regex::new(&on_pattern).unwrap()
    }
}

pub struct Ruleset {
    rules: Vec<Rule>
}

impl Ruleset {
    pub fn matching_rules(&self, task: &String) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| r.is_match(&task))
            .collect()
    }
}

pub fn init_global_ruleset() -> Ruleset {
    let mut rules: Vec<Rule> = Vec::new();

    rules.push(build_rule("test tasks", r"^tests?/(?P<name>.*)$"));

    Ruleset {
        rules
    }
}

