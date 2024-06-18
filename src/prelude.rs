use crate::errors;

pub use errors::Error;

pub type Result<T> = std::result::Result<T, errors::Error>;
