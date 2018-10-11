use errors::RescResult;
use fetchers::Fetcher;
use patterns::Pattern;
use regex::Regex;
use std::collections::HashMap;

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
    pub fetchers: Vec<Fetcher>,
    pub make_task: Pattern,
    pub make_queue: Pattern,
    pub make_set: Pattern,
}

impl Rule {
    fn is_match(&self, task: &str) -> bool {
        self.on_regex.is_match(task)
    }
    fn result(&self, props: &HashMap<String, String>) -> RuleResult {
        RuleResult {
            task: self.make_task.inject(&props),
            queue: self.make_queue.inject(&props),
            set: self.make_set.inject(&props),
        }
    }
    // Assumes the rule matches.
    pub fn results(&self, task: &str) -> RescResult<Vec<RuleResult>> {
        let mut props: HashMap<String, String> = HashMap::new();
        props.insert("input_task".to_owned(), task.to_owned());
        let caps = self.on_regex.captures(task).unwrap();
        let mut results = Vec::new();
        for groupname in self.on_regex.capture_names() {
            if let Some(name) = groupname {
                if let Some(value) = caps.name(name) {
                    props.insert(name.to_string(), value.as_str().to_string());
                }
            }
        }
        if !self.fetchers.is_empty() {
            // if there are fetchers, we'll fetch all the possible results
            // and generate a ruleresult per fetchresult
            for fetcher in &self.fetchers {
                let mut fetch_results = fetcher.results(&props)?;
                //println!("    -> fetch results {:#?}", &fetch_results);
                for mut fetch_result in fetch_results {
                    // we inject the parent properties
                    // This is heavy but makes the whole simpler
                    for (key, value) in &props {
                        // is there a shortcut ?
                        fetch_result.props.insert(key.clone(), value.clone());
                    }
                    //println!(" merged: {:#?}", &fetch_result.props);
                    results.push(self.result(&fetch_result.props));
                }
            }
        } else {
            results.push(self.result(&props));
        }
        Ok(results)
    }
}

#[derive(Debug)]
pub struct Ruleset {
    pub rules: Vec<Rule>,
}

impl Ruleset {
    pub fn matching_rules(&self, task: &str) -> Vec<&Rule> {
        self.rules.iter().filter(|r| r.is_match(&task)).collect()
    }
}
