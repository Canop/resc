/// handle reading and parsing a JSON configuration file
/// checking the consistency
/// and building the Conf object.
use regex::Regex;
use serde_json::{self, Value};
use std::fs;

use errors::RescResult;
use fetchers::Fetcher;
use patterns::Pattern;
use rules::{Rule, Ruleset};
use watchers::Watcher;

#[derive(Debug)]
pub struct Conf {
    pub watchers: Vec<Watcher>,
}

/// a trait defining conversions from json parsed values
trait JConv {
    fn get_string(&self, c: &str) -> RescResult<String>;
    fn get_l2_string(&self, c1: &str, c2: &str) -> RescResult<String>;
    fn as_fetcher(&self) -> RescResult<Fetcher>;
    fn as_rule(&self) -> RescResult<Rule>;
    fn as_watcher(&self, redis_url: String, task_set: String, listener_channel: String) -> RescResult<Watcher>;
    fn as_conf(&self) -> RescResult<Conf>;
}

impl JConv for Value {
    fn get_string(&self, c: &str) -> RescResult<String> {
        match &self[c] {
            Value::String(v) => Ok(v.to_owned()),
            _ => Err(format!("Missing {}", c).into()),
        }
    }

    fn get_l2_string(&self, c1: &str, c2: &str) -> RescResult<String> {
        match &self[c1][c2] {
            Value::String(v) => Ok(v.to_owned()),
            _ => Err(format!("Missing {}/{}", c1, c2).into()),
        }
    }

    fn as_fetcher(&self) -> RescResult<Fetcher> {
        let url_pattern = self.get_string("url")?;
        let returns = self.get_string("returns")?;
        Ok(Fetcher {
            url: Pattern { src: url_pattern },
            returns,
        })
    }

    fn as_rule(&self) -> RescResult<Rule> {
        let name = match &self["name"] {
            Value::String(v) => v.to_owned(),
            _ => "<anonymous rule>".to_owned(),
        };

        let on_pattern = self.get_string("on")?;
        let on_regex = match Regex::new(&on_pattern) {
            Ok(r) => r,
            Err(_) => return Err("invalid on/done pattern".into()),
        };

        let mut fetchers = Vec::new();
        if let Value::Array(fetchers_value) = &self["fetch"] {
            for fetcher_value in fetchers_value.iter() {
                let fetcher = fetcher_value.as_fetcher()?;
                fetchers.push(fetcher);
            }
        }

        let make_task = Pattern {
            src: match &self["make"]["task"] {
                Value::String(src) => src.to_owned(),
                _ => "${input_task}".to_owned(),
            },
        };

        let make_queue = match &self["make"]["queue"] {
            Value::String(src) => Pattern {
                src: src.to_owned(),
            },
            _ => return Err("missing make/queue string in rule".into()),
        };

        Ok(Rule {
            name,
            on_regex,
            fetchers,
            make_task,
            make_queue,
        })
    }

    fn as_watcher(&self, redis_url: String, task_set: String, listener_channel: String) -> RescResult<Watcher> {
        let input_queue = self.get_string("input_queue")?;
        let taken_queue = match &self["taken_queue"] {
            Value::String(s) => s.to_owned(),
            _ => format!("{}/taken", &input_queue).to_owned(),
        };
        let mut ruleset = Ruleset { rules: Vec::new() };
        let rules_value = match &self["rules"] {
            Value::Array(v) => v,
            _ => return Err("no global_ruleset/rules array".into()),
        };
        for rule_value in rules_value.iter() {
            let rule = rule_value.as_rule()?;
            ruleset.rules.push(rule);
        }
        Ok(Watcher {
            redis_url,
            task_set,
            listener_channel,
            input_queue,
            taken_queue,
            ruleset,
        })
    }

    fn as_conf(&self) -> RescResult<Conf> {
        let redis_url = self.get_l2_string("redis", "url")?;
        let task_set = self.get_string("task_set")?;
        let listener_channel = self.get_string("listener_channel")?;
        let mut watchers = Vec::new();

        let watchers_value = match &self["watchers"] {
            Value::Array(v) => v,
            _ => return Err("no watchers array".into()),
        };

        for watcher_value in watchers_value.iter() {
            let watcher = watcher_value.as_watcher(
                redis_url.to_owned(),
                task_set.to_owned(),
                listener_channel.to_owned())?;
            watchers.push(watcher);
        }

        Ok(Conf { watchers })
    }
}

pub fn read_file(filename: &str) -> RescResult<Conf> {
    let data =
        fs::read_to_string(filename).expect(&*format!("Failed to read config file {}", &filename));
    let root: Value = serde_json::from_str(&data)?;
    root.as_conf()
}
