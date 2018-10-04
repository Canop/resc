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

pub fn read_file(filename: &str) -> RescResult<Conf> {
    let data = fs::read_to_string(filename).expect("Failed to read file");
    println!("{}", &data);
    let root: Value = serde_json::from_str(&data)?;
    println!("Json conf file parsed");

    let mut global_ruleset = Ruleset {
        rules: Vec::new()
    };

    let rules_value = match &root["global_ruleset"]["rules"] {
        Value::Array(v) => v,
        _ => return Err("no global_ruleset/rules array".into()),
    };

    for rule_value in rules_value.iter() {

        let name = match &rule_value["name"] {
            Value::String(v) => v.to_owned(),
            _ => "<anonymous rule>".to_owned(),
        };
        println!("name = {:?}", name);

        let on_pattern = match &rule_value["on"]["done"] {
            Value::String(v) => v.to_owned(),
            _ => return Err("rule without on/done pattern".into()),
        };
        let on_regex = match Regex::new(&on_pattern) {
            Ok(r) => r,
            Err(_) => return Err("invalid on/done pattern".into()),
        };
        println!("on_regex = {:?}", on_regex);

        let todo_task_pattern = match &rule_value["todo"]["task"] {
            Value::String(v) => v.to_owned(),
            _ => return Err("rule without todo/task pattern".into()),
        };

        let todo_queue_pattern = match &rule_value["todo"]["queue"] {
            Value::String(v) => v.to_owned(),
            _ => return Err("rule without todo/queue pattern".into()),
        };

        // todo factorize 2 levels string access
        let todo_set_pattern = match &rule_value["todo"]["set"] {
            Value::String(v) => v.to_owned(),
            _ => return Err("rule without todo/set pattern".into()),
        };

        println!("todo_task_pattern = {:?}", &todo_task_pattern);
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
