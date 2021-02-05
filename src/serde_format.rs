use {
    crate::*,
    serde::de::DeserializeOwned,
    std::{
        fs,
        path::Path,
    },
    serde_json,
};


/// Formats usable for reading configuration files
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SerdeFormat {
    Json,
}

pub static FORMATS: &[SerdeFormat] = &[
    SerdeFormat::Json,
];

impl SerdeFormat {
    pub fn key(self) -> &'static str {
        match self {
            Self::Json => "json",
        }
    }
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "json" => Some(SerdeFormat::Json),
            _ => None,
        }
    }
    pub fn from_path(path: &Path) -> Result<Self, ConfError> {
        path.extension()
            .and_then(|os| os.to_str())
            .map(|ext| ext.to_lowercase())
            .and_then(|key| Self::from_key(&key))
            .ok_or_else(|| ConfError::UnknownFileExtension(path.to_string_lossy().to_string()))
    }
    pub fn read_file<T>(path: &Path) -> Result<T, ConfError>
        where T: DeserializeOwned
    {
        let format = Self::from_path(&path)?;
        match format {
            Self::Json => {
                Ok(serde_json::from_reader(fs::File::open(path)?)?)
            }
        }
    }
}

impl Default for SerdeFormat {
    fn default() -> Self {
        SerdeFormat::Json
    }
}
