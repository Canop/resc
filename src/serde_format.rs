use {
    crate::*,
    serde::de::DeserializeOwned,
    std::{
        fs,
        path::Path,
    },
};


/// Formats usable for reading configuration files
#[derive(Default, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SerdeFormat {
    Hjson,
    #[default]
    Json,
}

pub static FORMATS: &[SerdeFormat] = &[
    SerdeFormat::Hjson,
    SerdeFormat::Json,
];

impl SerdeFormat {
    pub fn key(self) -> &'static str {
        match self {
            Self::Hjson => "hjson",
            Self::Json => "json",
        }
    }
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "hjson" => Some(SerdeFormat::Hjson),
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
        let format = Self::from_path(path)?;
        match format {
            Self::Hjson => {
                let file_content = fs::read_to_string(path)?;
                let conf = deser_hjson::from_str(&file_content);
                if let Err(e) = &conf {
                    warn!("Error while deserializing conf: {:#?}", e);
                }
                Ok(conf?)
            }
            Self::Json => {
                Ok(serde_json::from_reader(fs::File::open(path)?)?)
            }
        }
    }
}

