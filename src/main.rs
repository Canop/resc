#[macro_use]
extern crate lazy_static;

extern crate redis;
extern crate regex;
extern crate serde_json;

use std::time::SystemTime;
use redis::{Commands, Connection, RedisResult};

mod errors;
mod rules;
mod conf;

fn fetch_test_value(con: &Connection) -> RedisResult<isize> {
    con.get("resc/test")
}

fn handle_global_done(con: &Connection, ruleset: &rules::Ruleset) {
    println!("watching queue global/done...");
    while let Ok(done) = con.brpoplpush::<_, String>("global/done", "global/taken", 0) {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        println!("got task: {:?} @ {}", done, now);
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
                //if let Ok(time) = redis::cmd("zscore").arg(&r.set).arg(&r.task).query::<i32>(con) {
                //    println!("     -> already queued @ {}", time);
                //    continue;
                //}
                println!("     -> not in set");
            }
        }
    }
}

fn main() {
    println!("----- starting resc scheduler -----");
    //let global_ruleset = rules::init_global_ruleset();
    let config = conf::read_file("demo/demo.conf.json").unwrap();
    println!("got conf");
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    println!("got connection");

    match fetch_test_value(&con) {
        Ok(v) => println!("Got test value : {}", v),
        Err(e) => println!("Failed due to : {:?}", e),
    }

    handle_global_done(&con, &config.global_ruleset);
}
