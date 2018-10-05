#[macro_use]
extern crate lazy_static;

extern crate redis;
extern crate regex;
extern crate serde_json;

mod errors;
mod rules;
mod conf;

use std::time::SystemTime;
use redis::{Commands, Connection, RedisResult};
use errors::{RescResult};

fn fetch_test_value(con: &Connection) -> RedisResult<isize> {
    con.get("resc/test")
}

fn handle_global_done(con: &Connection, ruleset: &rules::Ruleset) -> RescResult<()> {
    println!("watching queue global/done...");
    while let Ok(done) = con.brpoplpush::<_, String>("global/done", "global/taken", 0) {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let now = now as f64; // fine with a timestamp in seconds because < 2^51
        println!("got task: {:?} @ {}", &done, now);
        let matching_rules = ruleset.matching_rules(&done);
        println!(" {} matching rule(s)", matching_rules.len());
        for r in &matching_rules {
            println!("  applying rule '{}'", r.name);
            let results = r.results(&done);
            for r in &results {
                println!("  -> generated task: {} for queue {}", &r.task, &r.queue);
                println!("     checking task set {}", &r.set);
                if let Ok(time) = con.zscore::<_, _, i32>(&r.set, &r.task) {
                    println!("     -> already queued @ {}", time);
                    continue;
                }
                println!("     -> not in set");
                con.lpush::<_, _, i32>(&r.queue, &r.task)?;
                println!("     task {} added to queue {}", &r.task, &r.queue);
                con.zadd::<_, f64, _, i32>(&r.set, &r.task, now)?;
                println!("     task {} added to set {}", &r.task, &r.queue);
            }
        }
        con.lrem("global/taken", 1, &done)?;
    }
    Ok(()) // unreachable but necessary for signature (and might be reached in the future)
}

fn main() {
    println!("----- starting resc scheduler -----");
    let config = conf::read_file("demo/demo.conf.json").unwrap();
    println!("got conf");
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    println!("got redis connection");

    match fetch_test_value(&con) {
        Ok(v) => println!("Got test value : {}", v),
        Err(e) => println!("Failed due to : {:?}", e),
    }

    match handle_global_done(&con, &config.global_ruleset) {
        Ok(_) => println!("Watcher unexpectedly finished"),
        Err(e) => println!("Watcher crashed: {}", e),
    }
}
