use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error{
    Webhook(String),
    Shikimoti(String),
    SerdeJson(String),
    Other(String),
    Debug,
    Unknown,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Webhook(error) => formatter.write_str(&error.to_string()),
            Error::Shikimoti(error)=> formatter.write_str(&error.to_string()),
            Error::SerdeJson(error)=> formatter.write_str(&error.to_string()),
            Error::Other(msg) => formatter.write_str(msg),
            Error::Debug =>  formatter.write_str("This is error for debugging"),
            Error::Unknown =>  formatter.write_str("Unknown error"),
        }
    }
}

impl std::error::Error for Error {}
