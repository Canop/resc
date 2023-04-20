use {
    crate::*,
    serde::Deserialize,
    std::collections::HashMap,
};


#[derive(Debug, Clone, Deserialize)]
pub struct Maker {

    /// an optional name, for logs and for documentation in formats
    /// not allowing comments
    pub name: Option<String>,

    /// the output task generation pattern, defined with token
    /// found with on_regex or a fetcher
    #[serde(default = "Pattern::default_task")]
    pub task: Pattern,

    /// the queue where the generated tasks must be written
    pub queue: Pattern,

    /// the optional task set used for deduplicating
    pub set: Option<Pattern>,

}
impl Maker {
    pub fn make(
        &self,
        props: &HashMap<String, String>,
        results: &mut Vec<RuleResult>,
    ) {
        results.push(RuleResult {
            task: self.task.inject(props),
            queue: self.queue.inject(props),
            set: self.set.as_ref().map(|pattern| pattern.inject(props)),
        });
    }
}

/// This mimics the configuration structure where Make
/// elements can be given in an array or just single.
/// For now there's no difference and a single works
/// just as a 1 element array.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Makers {

    Single(Maker),

    Multiple(Vec<Maker>),

}

impl Makers {
    pub fn make(
        &self,
        props: &HashMap<String, String>,
        results: &mut Vec<RuleResult>,
    ) {
        match self {
            Self::Single(maker) => {
                maker.make(props, results);
            }
            Self::Multiple(vec) => {
                for maker in vec {
                    maker.make(props, results);
                }
            }
        }
    }
}
