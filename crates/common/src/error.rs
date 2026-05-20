use thiserror::Error;


#[derive(Error, Debug)]
pub enum FallError {
    #[error("Failed to parse TOML: {0}")]
    Toml(String),

    #[error("Failed to serialize to TOML: {0}")]
    TomlSerialize(String),

    #[error("Failed to deserialize from TOML: {0}")]
    TomlDeserialize(String),
}