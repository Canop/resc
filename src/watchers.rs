/// A watcher watches the events incoming in one specific queue
/// and applies rules to generate tasks

use std::time::SystemTime;
use errors::{RescResult};
use redis::{self, Commands, Connection};
use rules::{Ruleset};

# [derive(Debug)]
pub struct Watcher {
    pub redis_url: String,
    pub input_queue: String,
    pub taken_queue: String,
    pub ruleset: Ruleset,
}

impl Watcher {

    fn watch_input_queue(&self, con: &Connection) -> RescResult<()> {
        println!("watcher launched on queue {:?}...", &self.input_queue);
        while let Ok(done) = con.brpoplpush::<_, String>(&self.input_queue, &self.taken_queue, 0) {
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let now = now as f64; // fine with a timestamp in seconds because < 2^51
            println!("<- task {:?} in queue {:?} @ {}", &done, &self.input_queue, now);
            let matching_rules = self.ruleset.matching_rules(&done);
            println!(" {} matching rule(s)", matching_rules.len());
            for r in &matching_rules {
                println!(" applying rule {:?}", r.name);
                match r.results(&done) {
                    Ok(results) => {
                        for r in &results {
                            if let Ok(time) = con.zscore::<_, _, i32>(&r.set, &r.task) {
                                println!("  task {:?} already queued @ {}", &r.task, time);
                                continue;
                            }
                            println!("  ->  {:?} pushed to queue {:?} and set {:?}", &r.task, &r.queue, &r.set);
                            con.lpush::<_, _, i32>(&r.queue, &r.task)?;
                            con.zadd::<_, f64, _, i32>(&r.set, &r.task, now)?;
                        }
                    },
                    Err(err) => {
                        println!("  Rule execution crashed: {:?}", err)
                    }
                }
            }
            con.lrem(&self.taken_queue, 1, &done)?;
        }
        Ok(()) // unreachable but necessary for signature (and might be reached in the future)
    }

    pub fn run(&self) {
        let client = redis::Client::open(&*self.redis_url).unwrap();
        let con = client.get_connection().unwrap();
        println!("got redis connection");
        match self.watch_input_queue(&con) {
            Ok(_) => println!("Watcher unexpectedly finished"),
            Err(e) => println!("Watcher crashed: {:?}", e),
        }
    }

}
