use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not implemented")]
    NotImplemented,
}
