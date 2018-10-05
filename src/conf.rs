/// handle reading and parsing a JSON configuration file
/// checking the consistency
/// and building the Conf object.

use std::fs;
use serde_json;
use serde_json::{Value};
use regex::Regex;

use rules::{Rule, Ruleset};
use errors::{RescResult};

#[derive(Debug)]
pub struct Conf {
    pub global_ruleset: Ruleset
}

/// a trait defining conversions from json parsed values
trait JConv {
    fn get_l2_string(&self, c1: &str, c2: &str) -> RescResult<String>;
    fn as_conf(&self) -> RescResult<Conf>;
}

impl JConv for Value {

    fn get_l2_string(&self, c1: &str, c2: &str) -> RescResult<String> {
        match &self[c1][c2] {
            Value::String(v) => Ok(v.to_owned()),
            _ => Err(format!("Missing {}/{}", c1, c2).into()),
        }
    }

    fn as_conf(&self) -> RescResult<Conf> {
        let mut global_ruleset = Ruleset {
            rules: Vec::new()
        };

        let rules_value = match &self["watchers"][0]["rules"] {
            Value::Array(v) => v,
            _ => return Err("no global_ruleset/rules array".into()),
        };

        for rule_value in rules_value.iter() {

            let name = match &rule_value["name"] {
                Value::String(v) => v.to_owned(),
                _ => "<anonymous rule>".to_owned(),
            };

            let on_pattern = rule_value.get_l2_string("on", "done")?;
            let on_regex = match Regex::new(&on_pattern) {
                Ok(r) => r,
                Err(_) => return Err("invalid on/done pattern".into()),
            };

            let todo_task_pattern = rule_value.get_l2_string("todo", "task")?;
            let todo_queue_pattern = rule_value.get_l2_string("todo", "queue")?;
            let todo_set_pattern = rule_value.get_l2_string("todo", "set")?;

            global_ruleset.rules.push(Rule{
                name,
                on_regex,
                todo_task_pattern,
                todo_queue_pattern,
                todo_set_pattern,
            });
        }

        Ok(Conf {
            global_ruleset
        })

    }
}

pub fn read_file(filename: &str) -> RescResult<Conf> {
    let data = fs::read_to_string(filename).expect("Failed to read file");
    println!("{}", &data);
    let root: Value = serde_json::from_str(&data)?;
    println!("Json conf file parsed");
    root.as_conf()
}
