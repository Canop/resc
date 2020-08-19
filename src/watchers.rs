use {
    crate::{errors::RescResult, rules::Ruleset},
    log::*,
    redis::{self, Commands, Connection},
    std::time::SystemTime,
};

/// A watcher watches the events incoming in one specific queue
/// and applies rules to generate tasks
#[derive(Debug)]
pub struct Watcher {
    pub redis_url: String,        // same for all watchers
    pub task_set: String,         // same for all watchers
    pub listener_channel: String, // same for all watchers
    pub input_queue: String,
    pub taken_queue: String,
    pub ruleset: Ruleset,
}

impl Watcher {
    fn empty_taken_queue(&self, con: &mut Connection) {
        debug!("watcher cleans its taken queue");
        let mut n = 0;
        while let Ok(taken) = con.rpoplpush::<_, String>(&self.taken_queue, &self.input_queue) {
            debug!(
                " moving {:?} from {:?} to {:?}",
                &taken, &self.taken_queue, &self.input_queue
            );
            n = n + 1;
        }
        if n > 0 {
            warn!(
                "moved {} tasks from  {:?} to {:?}",
                n, &self.taken_queue, &self.input_queue
            );
        }
    }

    fn watch_input_queue(&self, con: &mut Connection) -> RescResult<()> {
        info!("watcher launched on queue {:?}...", &self.input_queue);
        while let Ok(done) = con.brpoplpush::<_, String>(&self.input_queue, &self.taken_queue, 0) {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let now = now as f64; // fine with a timestamp in seconds because < 2^51
            info!(
                "<- got {:?} in queue {:?} @ {}",
                &done, &self.input_queue, now
            );
            if let Ok(n) = con.zrem::<_, _, i32>(&self.task_set, &done) {
                if n > 0 {
                    debug!(" previously queued task {:?} end", &done);
                } else {
                    debug!(" external task {:?} end", &done);
                }
            }
            let matching_rules = self.ruleset.matching_rules(&done);
            debug!(" {} matching rule(s)", matching_rules.len());
            for r in &matching_rules {
                debug!(" applying rule {:?}", r.name);
                match r.results(&done) {
                    Ok(results) => {
                        for r in &results {
                            if let Ok(time) = con.zscore::<_, _, i32>(&self.task_set, &r.task) {
                                info!("  task {:?} already queued @ {}", &r.task, time);
                                continue;
                            }
                            info!("  ->  {:?} pushed to queue {:?}", &r.task, &r.queue);
                            con.lpush(&r.queue, &r.task)?;
                            con.zadd(&self.task_set, &r.task, now)?;
                            debug!(
                                "      {:?} pushed to task_set {:?} @ {}",
                                &r.task, &self.task_set, now
                            );
                            con.publish(
                                &self.listener_channel,
                                format!("{} TRIGGER {} -> {}", &self.taken_queue, &done, &r.task,),
                            )?;
                        }
                    }
                    Err(err) => error!("  Rule execution failed: {:?}", err),
                }
            }
            con.lrem(&self.taken_queue, 1, &done)?;
            con.publish(
                &self.listener_channel,
                format!("{} DONE {}", &self.taken_queue, &done),
            )?;
            debug!(" done with task {:?}", &done);
        }
        unreachable!();
    }

    pub fn run(&self) {
        let client = redis::Client::open(&*self.redis_url).unwrap();
        let mut con = client.get_connection().unwrap();
        debug!("got redis connection");
        self.empty_taken_queue(&mut con);
        match self.watch_input_queue(&mut con) {
            Ok(_) => error!("Watcher unexpectedly finished"),
            Err(e) => error!("Watcher crashed: {:?}", e),
        }
    }
}
