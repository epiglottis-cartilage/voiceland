pub type Error = Box<dyn ::core::error::Error + Send + Sync>;
pub type Result<T> = ::core::result::Result<T, Error>;