use thiserror::Error;

pub type BwgResult<T> = std::result::Result<T, BwgError>;

#[derive(Error, Debug)]
pub enum BwgError {
    #[error("error with some sfml operation: {0}")]
    Sfml(#[from] sfml::SfError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
