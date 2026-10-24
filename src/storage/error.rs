/// Recoverable database, file or validation failure with its original cause.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
