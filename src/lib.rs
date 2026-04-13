pub mod config;
pub mod error;
pub mod git;
pub mod hooks;
pub mod perms;

pub type Result<T> = std::result::Result<T, error::Error>;
