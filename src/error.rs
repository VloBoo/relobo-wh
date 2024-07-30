use std::fmt::{self, Display};


pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error{
    Io(std::io::Error),
    Webhook(String),
    Other(String),
    Unknown,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(error) => formatter.write_str(&error.to_string()),
            Error::Webhook(error) => formatter.write_str(&error.to_string()),
            Error::Other(msg) => formatter.write_str(msg),
            Error::Unknown =>  formatter.write_str("Unknown error"),
           // _ => !unimplemented!()
        }
    }
}

impl std::error::Error for Error {}
