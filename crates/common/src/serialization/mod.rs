use bevy::ecs::error::BevyError;
use thiserror::Error;


pub trait FromToml<T> {
    fn from_toml(toml: &str) -> Result<T, BevyError>;
}

pub trait FromJson<T> {
    fn from_json(json: &str) -> Result<T, BevyError>;
}

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("Failed to parse TOML: {0}")]
    TomlParse(String),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(String),
}

