extern crate redis;
extern crate regex;

use redis::{Commands, Connection, RedisResult};

mod rules;


fn fetch_test_value(con: &Connection) -> RedisResult<isize> {
    con.get("resc/test")
}

fn handle_global_done(con: &Connection, ruleset: rules::Ruleset) {
    while let Ok(done) = con.brpoplpush::<_, String>("global/done", "global/taken", 0) {
        println!("got task: {:?}", done);
        let matching_rules = ruleset.matching_rules(&done);
        println!(" {} matching rule(s)", matching_rules.len());
        for r in &matching_rules {
            println!(" applying rule {}", r.name);
        }
    }
}

fn main() {
    println!("----- starting resc scheduler -----");
    let global_ruleset = rules::init_global_ruleset();
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();
    println!("got connection");

    match fetch_test_value(&con) {
        Ok(v) => println!("Got test value : {}", v),
        Err(e) => println!("Failed due to : {:?}", e),
    }

    handle_global_done(&con, global_ruleset);
}
