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

pub fn read_file(filename: &str) -> Result<Conf, ConfError> {
    let start = std::time::Instant::now();
    let conf = SerdeFormat::read_file(&PathBuf::from(&filename));
    debug!("Conf read in {:?}", start.elapsed());
    conf
}
