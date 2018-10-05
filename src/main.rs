#[macro_use]
extern crate lazy_static;

extern crate redis;
extern crate regex;
extern crate serde_json;

mod errors;
mod rules;
mod conf;
mod watchers;

use std::thread;

fn main() {
    println!("----- starting resc scheduler -----");
    let config = conf::read_file("demo/demo.conf.json").unwrap();
    println!("got configuration");

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
