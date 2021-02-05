use {
    crate::*,
    log::*,
    redis::{self, Commands, Connection},
    serde::Deserialize,
    std::{
        time::SystemTime,
    },
};

#[derive(Debug, Deserialize)]
pub struct WatcherConf {
    pub input_queue: String,
    pub taken_queue: Option<String>,
    pub rules: Vec<Rule>,
}

/// A watcher watches the events incoming in one specific queue
/// and applies rules to generate tasks
pub struct Watcher {
    con: Connection,
    listener_channel: String,
    input_queue: String,
    taken_queue: String, // can't be shared between watchers
    ruleset: Ruleset,
}

impl Watcher {

    pub fn new(
        watcher_conf: &WatcherConf,
        global_conf: &Conf,
    ) -> Result<Self, RescError> {
        let listener_channel = global_conf.listener_channel.clone();
        let input_queue = watcher_conf.input_queue.clone();
        let taken_queue = match watcher_conf.taken_queue.as_ref() {
            Some(queue) => queue.clone(),
            None => format!("{}/taken", &input_queue),
        };
        let ruleset = Ruleset {
            rules: watcher_conf.rules.clone(),
        };
        let client = redis::Client::open(&*global_conf.redis.url)?;
        let con = client.get_connection()?;
        debug!("got redis connection");
        Ok(Self {
            con,
            listener_channel,
            input_queue,
            taken_queue,
            ruleset,
        })
    }

    pub fn run(&mut self) -> Result<(), RescError> {
        self.empty_taken_queue();
        self.watch_input_queue()
    }

    /// move tasks from the taken queue to the input queue
    ///
    /// This is done on start to reschedule the tasks that
    /// weren't completely handled.
    fn empty_taken_queue(&mut self) {
        debug!("watcher cleans its taken queue");
        let mut n = 0;
        while let Ok(taken) = self.con.rpoplpush::<_, String>(&self.taken_queue, &self.input_queue) {
            debug!(
                " moving {:?} from {:?} to {:?}",
                &taken, &self.taken_queue, &self.input_queue
            );
            n += 1;
        }
        if n > 0 {
            warn!(
                "moved {} tasks from  {:?} to {:?}",
                n, &self.taken_queue, &self.input_queue
            );
        }
    }

    /// completely handle one event received on the input queue
    fn handle_input_event(&mut self, done: String) -> Result<(), RescError> {
        let now = now_secs();
        info!(
            "<- got {:?} in queue {:?} @ {}",
            &done, &self.input_queue, now
        );

        // we first compute all the rule results
        let mut results = Vec::new();
        for rule in self.ruleset.matching_rules(&done) {
            debug!(" applying rule {:?}", rule.name);
            match rule.results(&done) {
                Ok(mut rule_results) => {
                    results.append(&mut rule_results);
                }
                Err(e) => {
                    // A possible failure reason is a fetch not possible because of
                    // network or server condition.
                    // TODO should we do something better ? Requeue ?
                    error!("  Rule execution failed: {:?}", e);
                }
            }
        }
        debug!(" {} result(s)", results.len());

        // we now apply the rule results, that is we push the tasks
        for r in results {
            // if the rule specifies a task_set, we check the task isn't
            // already present in the set
            let in_set_time: Option<i32> = r.set.as_ref()
                .and_then(|s| self.con.zscore(s, &r.task).ok());
            if let Some(time) = in_set_time {
                info!("  task {:?} already queued @ {}", &r.task, time);
                continue;
            }
            info!("  ->  {:?} pushed to queue {:?}", &r.task, &r.queue);
            if let Some(task_set) = r.set.as_ref() {
                // we push first to the task set, to avoid a race condition:
                // a worker not finding the task in the set
                self.con.zadd(task_set, &r.task, now)?;
                debug!(
                    "      {:?} pushed to task_set {:?} @ {}",
                    &r.task, task_set, now
                );
            }
            self.con.lpush(&r.queue, &r.task)?;
            self.con.publish(
                &self.listener_channel,
                format!("{} TRIGGER {} -> {}", &self.taken_queue, &done, &r.task),
            )?;
        }

        // the event can now be removed from the taken queue
        self.con.lrem(&self.taken_queue, 1, &done)?;
        self.con.publish(
            &self.listener_channel,
            format!("{} DONE {}", &self.taken_queue, &done),
        )?;
        debug!(" done with task {:?}", &done);
        Ok(())
    }

    /// continuously watch the input queue an apply rules on the events
    /// it takes in the queue
    fn watch_input_queue(&mut self) -> Result<(), RescError> {
        info!("watcher launched on queue {:?}...", &self.input_queue);
        loop {
            match self.con.brpoplpush(&self.input_queue, &self.taken_queue, 0) {
                Ok(done) => {
                    self.handle_input_event(done)?
                }
                Err(e) => {
                    error!("BRPOPLPUSH on {:?} failed : {}", &self.input_queue, e);
                }
            }
        }
    }

}

/// build the Epoch related timestamp, in seconds as f64
/// because we want to use in in JSON and JS. Precision
/// in f64 is not lost because this number is smaller than 2^51.
fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        as f64
}
