use thiserror::Error;

#[derive(Debug, Error)] // Removed Clone from here as anyhow::Error is not Clone
pub enum InputError {
    #[error("Device disconnected")]
    Disconnected,
    #[error("Input source error: {0}")]
    Other(#[from] anyhow::Error),
}

// Since anyhow::Error is not Clone, we cannot derive Clone for InputError easily.
// For testing purposes, we might need a workaround or just return new errors.
