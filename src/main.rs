#[macro_use]
extern crate lazy_static;

extern crate redis;
extern crate regex;
extern crate serde_json;

mod errors;
mod rules;
mod conf;
mod watchers;

use std::env;
use std::thread;

fn main() {
    println!("----- starting resc scheduler -----");

    let args: Vec<String> = env::args().collect();
    if args.len()<2 {
        panic!("no configuration file provided");
    }
    let config_filename = &args[1];
    let config = conf::read_file(&config_filename).unwrap();
    println!("configuration read from {}", &config_filename);

    let handles: Vec<thread::JoinHandle<_>> = config.watchers.into_iter().map(move |watcher| {
        thread::spawn(move || {
            watcher.run()
        })
    }).collect();

    println!("all watchers started");

    for h in handles {
        h.join().unwrap();
    }

}
