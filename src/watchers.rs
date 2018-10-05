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
            println!("got task: {:?} @ {}", &done, now);
            let matching_rules = self.ruleset.matching_rules(&done);
            println!(" {} matching rule(s)", matching_rules.len());
            for r in &matching_rules {
                println!("  applying rule '{:?}'", r.name);
                let results = r.results(&done);
                for r in &results {
                    println!("  -> generated task: {:?} for queue {:?}", &r.task, &r.queue);
                    println!("     checking task set {:?}", &r.set);
                    if let Ok(time) = con.zscore::<_, _, i32>(&r.set, &r.task) {
                        println!("     -> already queued @ {}", time);
                        continue;
                    }
                    println!("     -> not in set");
                    con.lpush::<_, _, i32>(&r.queue, &r.task)?;
                    println!("     task {:?} added to queue {:?}", &r.task, &r.queue);
                    con.zadd::<_, f64, _, i32>(&r.set, &r.task, now)?;
                    println!("     task {:?} added to set {:?}", &r.task, &r.queue);
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
