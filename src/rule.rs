use {
    crate::*,
    log::*,
    regex::Regex,
    serde::Deserialize,
    std::collections::HashMap,
};


/// a rule, defined by a condition (the "on" pattern)
/// and what to do with the matching tasks
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {

    /// the name, unused for now, but having it in the JSON
    /// file helps making it clearer and it could be used in
    /// logging in the future, so it's mandatory
    #[serde(default = "Rule::default_name")]
    pub name: String,

    /// the input task parser. It checks the rule applies to
    /// the task and it extracts the token which will be used
    /// to generate the output task
    #[serde(with = "serde_regex", alias = "on")]
    pub on_regex: Regex,

    /// The optional fetchers which may query some additional
    /// token for generation of the output task
    #[serde(default, alias = "fetch")]
    pub fetchers: Vec<Fetcher>,

    /// The recipe for building the output tasks when the rule
    /// is verified and the fetchers did their job
    #[serde(alias = "make")]
    pub makers: Makers,

}

impl Rule {
    pub fn default_name() -> String {
        "<anonymous rule>".into()
    }
    pub fn is_match(&self, task: &str) -> bool {
        self.on_regex.is_match(task)
    }
    /// Assuming the rule matches, computes the rule results
    /// (there's only one RuleResult when no fetcher is involved)
    pub fn results(&self, task: &str) -> Result<Vec<RuleResult>, RescError> {
        // props will contain the token usable for generating
        // the task name, output queue and output set
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
                let fetch_results = fetcher.results(&props)?;
                debug!("    -> fetch results {:#?}", &fetch_results);
                for mut fetch_result in fetch_results {
                    // we inject the parent properties
                    // This is heavy but makes the whole simpler
                    for (key, value) in &props {
                        fetch_result.props.insert(key.clone(), value.clone());
                    }
                    trace!(" merged: {:#?}", &fetch_result.props);
                    self.makers.make(&fetch_result.props, &mut results);
                }
            }
        } else {
            self.makers.make(&props, &mut results);
        }
        Ok(results)
    }
}
