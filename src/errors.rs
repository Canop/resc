use {
    thiserror::Error,
};


#[derive(Error, Debug)]
pub enum RescError {

    #[error("conf error")]
    Conf(#[from] ConfError),

    #[error("fetch error")]
    Reqwest(#[from] FetchError),

    #[error("redis error")]
    Redis(#[from] redis::RedisError),

}

#[derive(Error, Debug)]
pub enum ConfError {

    #[error("Unknow file extension: {0:?}")]
    UnknownFileExtension(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Invalid Hjson: {0}")]
    Hjson(#[from] deser_hjson::Error),

    #[error("Invalid JSON: {0}")]
    JSON(#[from] serde_json::Error),
}


#[derive(Error, Debug)]
pub enum FetchError {

    #[error("reqwest error")]
    Reqwest(#[from] reqwest::Error),

    #[error("fetch received an error - status: {0}")]
    ErrorStatus(u16),

    #[error("unexpected response content")]
    UnexpectedContent,

    #[error("io error")]
    IO(#[from] std::io::Error),

    #[error("invalid JSON")]
    JSON(#[from] serde_json::Error),

}

