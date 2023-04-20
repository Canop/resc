//! Resc is a task orchestrator for distributed systems
//! It's based on Rust and ensures in a safe way the
//! generation of deduced tasks and their availability
//! for external workers
//!
//! Introduction and complete description in the [README](https://github.com/Canop/resc)

mod conf;
mod errors;
mod fetcher;
mod make;
mod pattern;
mod rule;
mod ruleset;
mod rule_result;
mod serde_format;
mod watcher;

use {
    chrono::Local,
    log::*,
    std::{env, io::Write, thread},
};

pub use {
    conf::*,
    errors::*,
    fetcher::*,
    make::*,
    pattern::*,
    rule::*,
    ruleset::*,
    rule_result::*,
    serde_format::*,
    watcher::*,
};

fn configure_logger() {
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "warn");
    let mut builder = env_logger::Builder::from_env(env);
    builder.default_format_module_path(false);
    // log format with millisecond for better understanding of concurrency issues
    builder.format(|buf, record| {
        writeln!(
            buf,
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
    info!("configuration read from {}", config_filename);
    let conf = match conf::read_file(config_filename) {
        Ok(conf) => conf,
        Err(e) => {
            error!("Error reading configuration: {}", &e);
            eprintln!("{}", e);
            return;
        }
    };

    let mut handles = Vec::new();
    for watcher_conf in &conf.watchers {
        let mut watcher = Watcher::new(watcher_conf, &conf).unwrap();
        handles.push(thread::spawn(move || {
            watcher.run().unwrap();
        }));
    }

    debug!("all watchers started");

    for h in handles {
        h.join().unwrap();
    }
}
