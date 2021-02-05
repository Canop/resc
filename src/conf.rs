use {
    crate::*,
    serde::Deserialize,
    std::{
        path::PathBuf,
    },
};


/// Redis access configuration
#[derive(Debug, Deserialize)]
pub struct RedisConf {
    pub url: String,
}

/// The configuration of Resc, as read from a JSON file
#[derive(Debug, Deserialize)]
pub struct Conf {
    pub redis: RedisConf,
    pub listener_channel: String,
    pub watchers: Vec<WatcherConf>,
}

// /// a trait defining conversions from json parsed values
// trait JConv {
//     fn get_string(&self, c: &str) -> Result<String, ConfError>;
//     fn get_l2_string(&self, c1: &str, c2: &str) -> Result<String, ConfError>;
//     fn as_fetcher(&self) -> Result<Fetcher, ConfError>;
//     fn as_rule(&self) -> Result<Rule, ConfError>;
//     fn as_watcher(
//         &self,
//         redis_url: String,
//         listener_channel: String,
//     ) -> Result<Watcher, ConfError>;
//     fn as_conf(&self) -> Result<Conf, ConfError>;
// }
//
// impl JConv for Value {
//     fn get_string(&self, c: &str) -> Result<String, ConfError> {
//         match &self[c] {
//             Value::String(v) => Ok(v.to_owned()),
//             _ => Err(format!("Missing {}", c).into()),
//         }
//     }
//
//     fn get_l2_string(&self, c1: &str, c2: &str) -> Result<String, ConfError> {
//         match &self[c1][c2] {
//             Value::String(v) => Ok(v.to_owned()),
//             _ => Err(format!("Missing {}/{}", c1, c2).into()),
//         }
//     }
//
//     fn as_fetcher(&self) -> Result<Fetcher, ConfError> {
//         let url_pattern = self.get_string("url")?;
//         let returns = self.get_string("returns")?;
//         Ok(Fetcher {
//             url: Pattern { src: url_pattern },
//             returns,
//         })
//     }
//
//     fn as_rule(&self) -> Result<Rule, ConfError> {
//         let name = match &self["name"] {
//             Value::String(v) => v.to_owned(),
//             _ => "<anonymous rule>".to_owned(),
//         };
//
//         let on_pattern = self.get_string("on")?;
//         let on_regex = match Regex::new(&on_pattern) {
//             Ok(r) => r,
//             Err(_) => return Err("invalid on/done pattern".into()),
//         };
//
//         let mut fetchers = Vec::new();
//         if let Value::Array(fetchers_value) = &self["fetch"] {
//             for fetcher_value in fetchers_value.iter() {
//                 let fetcher = fetcher_value.as_fetcher()?;
//                 fetchers.push(fetcher);
//             }
//         }
//
//         let make_task = Pattern {
//             src: match &self["make"]["task"] {
//                 Value::String(src) => src.to_owned(),
//                 _ => "${input_task}".to_owned(),
//             },
//         };
//
//         let make_queue = match &self["make"]["queue"] {
//             Value::String(src) => Pattern {
//                 src: src.to_owned(),
//             },
//             _ => return Err("missing make/queue string in rule".into()),
//         };
//
//         let make_set = match &self["make"]["set"] {
//             Value::String(src) => Some(Pattern {
//                 src: src.to_owned(),
//             }),
//             Value::Null => None,
//             _ => return Err("invalid make/set in rule".into()),
//         };
//
//         Ok(Rule {
//             name,
//             on_regex,
//             fetchers,
//             make_task,
//             make_queue,
//             make_set,
//         })
//     }
//
//     fn as_watcher(
//         &self,
//         redis_url: String,
//         listener_channel: String,
//     ) -> Result<Watcher, ConfError> {
//         let input_queue = self.get_string("input_queue")?;
//         let taken_queue = match &self["taken_queue"] {
//             Value::String(s) => s.to_owned(),
//             _ => format!("{}/taken", &input_queue),
//         };
//         let mut ruleset = Ruleset { rules: Vec::new() };
//         let rules_value = match &self["rules"] {
//             Value::Array(v) => v,
//             _ => return Err("no global_ruleset/rules array".into()),
//         };
//         for rule_value in rules_value.iter() {
//             let rule = rule_value.as_rule()?;
//             ruleset.rules.push(rule);
//         }
//         Ok(Watcher {
//             redis_url,
//             listener_channel,
//             input_queue,
//             taken_queue,
//             ruleset,
//         })
//     }
//
//     fn as_conf(&self) -> Result<Conf, ConfError> {
//         let redis_url = self.get_l2_string("redis", "url")?;
//         if let Value::String(s) = &self["task_set"] {
//             log::warn!("Ignoring {:?}:{:?} because global task_set isn't supported anymore", "task_set", s);
//         }
//         let listener_channel = self.get_string("listener_channel")?;
//         let mut watchers = Vec::new();
//
//         let watchers_value = match &self["watchers"] {
//             Value::Array(v) => v,
//             _ => return Err("no watchers array".into()),
//         };
//
//         for watcher_value in watchers_value.iter() {
//             let watcher = watcher_value.as_watcher(
//                 redis_url.to_owned(),
//                 listener_channel.to_owned(),
//             )?;
//             watchers.push(watcher);
//         }
//
//         Ok(Conf { watchers })
//     }
// }

pub fn read_file(filename: &str) -> Result<Conf, ConfError> {
    SerdeFormat::read_file(&PathBuf::from(&filename))
}
