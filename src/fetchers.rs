use errors::RescResult;
use patterns::Pattern;
use reqwest;
use serde_json::{self, Value};
/// A Fetcher is responsible for synchronously fetching some data
/// (for use in handling a rule)
use std::collections::HashMap;
use std::io::Read;

#[derive(Debug)]
pub struct FetchResult {
    pub props: HashMap<String, String>,
}

// will probably be an enum later, with various other
//  strategies including graphql and direct DB queries
#[derive(Debug)]
pub struct Fetcher {
    pub url: Pattern,
    pub returns: String,
}

impl Fetcher {
    fn returned_key(&self, key: &str) -> String {
        format!("{}.{}", self.returns, key)
    }

    fn get_fetch_result(&self, value: &Value) -> RescResult<FetchResult> {
        match value {
            Value::Object(object_value) => {
                let mut props = HashMap::new();
                for (key, value) in object_value.iter() {
                    match value {
                        Value::String(string_value) => {
                            props.insert(self.returned_key(key), string_value.to_owned());
                        }
                        Value::Number(number_value) => {
                            props.insert(self.returned_key(key), number_value.to_string());
                        }
                        _ => {
                            println!(" ignoring property {:#?}={:#?}", key, value);
                        }
                    }
                }
                Ok(FetchResult { props })
            }
            _ => Err("unexpected json value type".into()),
        }
    }

    pub fn results(&self, props: &HashMap<String, String>) -> RescResult<Vec<FetchResult>> {
        let url = self.url.inject(&props).to_string();
        println!("  querying url: {:#?}", url);
        let mut response = reqwest::get(&url)?;
        if !response.status().is_success() {
            return Err(format!("     -> request answered an error : {}", response.status()).into());
        }
        let mut json = String::new();
        response.read_to_string(&mut json)?;
        let mut results = Vec::new();
        let value: Value = serde_json::from_str(&json)?;
        match value {
            Value::Array(returned_values) => {
                for returned_value in &returned_values {
                    results.push(self.get_fetch_result(returned_value)?);
                }
            }
            _ => return Err("unexpected content".into()),
        }
        Ok(results)
    }
}
