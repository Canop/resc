/// handle reading and parsing a JSON configuration file
/// checking the consistency
/// and building the Conf object.

use std::fs;
use serde_json;
use serde_json::{Value};
use regex::Regex;

use rules::{Rule, Ruleset};
use errors::{RescResult};
use watchers::{Watcher};

#[derive(Debug)]
pub struct Conf {
    pub watchers: Vec<Watcher>,
}

/// a trait defining conversions from json parsed values
trait JConv {
    fn get_string(&self, c: &str) -> RescResult<String>;
    fn get_l2_string(&self, c1: &str, c2: &str) -> RescResult<String>;
    fn as_rule(&self) -> RescResult<Rule>;
    fn as_watcher(&self) -> RescResult<Watcher>;
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

    fn as_rule(&self) -> RescResult<Rule> {
            let name = match &self["name"] {
                Value::String(v) => v.to_owned(),
                _ => "<anonymous rule>".to_owned(),
            };

            let on_pattern = self.get_l2_string("on", "done")?;
            let on_regex = match Regex::new(&on_pattern) {
                Ok(r) => r,
                Err(_) => return Err("invalid on/done pattern".into()),
            };

            let todo_task_pattern = self.get_l2_string("todo", "task")?;
            let todo_queue_pattern = self.get_l2_string("todo", "queue")?;
            let todo_set_pattern = self.get_l2_string("todo", "set")?;

            Ok(Rule{
                name,
                on_regex,
                todo_task_pattern,
                todo_queue_pattern,
                todo_set_pattern,
            })
    }

    fn as_watcher(&self) -> RescResult<Watcher> {
        let redis_url = self.get_l2_string("redis", "url")?;
        let input_queue = self.get_string("input_queue")?;
        let taken_queue = self.get_string("taken_queue")?;
        let mut ruleset = Ruleset {
            rules: Vec::new()
        };
        let rules_value = match &self["rules"] {
            Value::Array(v) => v,
            _ => return Err("no global_ruleset/rules array".into()),
        };
        for rule_value in rules_value.iter() {
            let rule = rule_value.as_rule()?;
            ruleset.rules.push(rule);
        }
        Ok(Watcher{
            redis_url,
            input_queue,
            taken_queue,
            ruleset,
        })
    }

    fn as_conf(&self) -> RescResult<Conf> {
        let mut watchers = Vec::new();

        let watchers_value = match &self["watchers"] {
            Value::Array(v) => v,
            _ => return Err("no watchers array".into()),
        };

        for watcher_value in watchers_value.iter() {
            let watcher = watcher_value.as_watcher()?;
            watchers.push(watcher);
        }

        Ok(Conf {
            watchers
        })
    }
}

pub fn read_file(filename: &str) -> RescResult<Conf> {
    let data = fs::read_to_string(filename)
        .expect(&*format!("Failed to read config file {}", &filename));
    let root: Value = serde_json::from_str(&data)?;
    root.as_conf()
}
