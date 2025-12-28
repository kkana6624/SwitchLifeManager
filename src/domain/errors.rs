use thiserror::Error;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Device disconnected")]
    Disconnected,
    #[error("Input source error: {0}")]
    Other(#[from] anyhow::Error),
}
