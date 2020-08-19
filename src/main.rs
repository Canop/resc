extern crate env_logger;
extern crate lazy_static;
extern crate log;
extern crate chrono;
extern crate redis;
extern crate regex;
extern crate reqwest;
extern crate serde_json;

mod conf;
mod errors;
mod fetchers;
mod patterns;
mod rules;
mod watchers;

use log::*;
use std::env;
use std::thread;
use std::io::Write;
use chrono::Local;

fn configure_logger() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn");
    let mut builder = env_logger::Builder::from_env(env);
    builder.default_format_module_path(false);
    // log format with millisecond for better understanding of concurrency issues
    builder.format(|buf, record| {
        writeln!(buf,
                 "{} [{}] - {}",
                 Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                 record.level(),
                 record.args()
        )
    });
    builder.init();
}

fn main() {
    configure_logger();

    info!("----- starting resc scheduler -----");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("no configuration file provided");
    }
    let config_filename = &args[1];
    let config = conf::read_file(&config_filename).unwrap();
    info!("configuration read from {}", &config_filename);

    let handles: Vec<thread::JoinHandle<_>> = config
        .watchers
        .into_iter()
        .map(move |watcher| thread::spawn(move || watcher.run()))
        .collect();

    debug!("all watchers started");

    for h in handles {
        h.join().unwrap();
    }
}
