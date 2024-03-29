use {
    crate::*,
    log::*,
    serde::Deserialize,
    serde_json::{self, Value},
    std::{collections::HashMap, io::Read},
};

/// the data the fetcher got
#[derive(Debug)]
pub struct FetchResult {
    pub props: HashMap<String, String>,
}

/// A Fetcher is responsible for synchronously fetching some data
/// (for use in handling a rule)
#[derive(Debug, Clone, Deserialize)]
pub struct Fetcher {
    pub url: Pattern,
    pub returns: String,
}

impl Fetcher {
    fn returned_key(&self, key: &str) -> String {
        format!("{}.{}", self.returns, key)
    }

    fn get_fetch_result(&self, object_value: &serde_json::Map<String, Value>) -> FetchResult {
        let mut props = HashMap::new();
        for (key, value) in object_value {
            match value {
                Value::String(string_value) => {
                    props.insert(self.returned_key(key), string_value.to_owned());
                }
                Value::Number(number_value) => {
                    props.insert(self.returned_key(key), number_value.to_string());
                }
                _ => {
                    debug!(" ignoring property {:#?}={:#?}", key, value);
                }
            }
        }
        FetchResult { props }
    }

    pub fn results(&self, props: &HashMap<String, String>) -> Result<Vec<FetchResult>, FetchError> {
        let url = self.url.inject(props);
        info!("  querying url: {:#?}", url);
        let mut response = reqwest::get(&url)?;
        if !response.status().is_success() {
            return Err(FetchError::ErrorStatus(response.status().into()));
        }
        // TODO use derive for response deserialization
        let mut json = String::new();
        response.read_to_string(&mut json)?;
        let mut results = Vec::new();
        let value: Value = serde_json::from_str(&json)?;
        // we accept either a simple object, or an array of objects
        match value {
            Value::Array(returned_values) => {
                for returned_value in &returned_values {
                    match returned_value {
                        Value::Object(object_value) => {
                            results.push(self.get_fetch_result(object_value));
                        }
                        _ => {
                            return Err(FetchError::UnexpectedContent);
                        }
                    }
                }
            }
            Value::Object(returned_value) => {
                results.push(self.get_fetch_result(&returned_value));
            }
            _ => {
                return Err(FetchError::UnexpectedContent);
            }
        }
        Ok(results)
    }
}
