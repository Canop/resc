

/// result of applying a rule to a task
#[derive(Debug)]
pub struct RuleResult {

    /// the task to generate
    pub task: String,

    /// the queue where to write the task
    pub queue: String,

    /// the sorted set where to check the task
    /// isn't yet in the queue
    pub set: Option<String>,

}
