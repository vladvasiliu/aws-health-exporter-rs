use std::{error::Error as StdError, result::Result as StdResult, fmt};
use rusoto_signature::region::ParseRegionError;
use std::fmt::Formatter;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidRegion(ParseRegionError),
}


impl From<ParseRegionError> for Error {
    fn from(err: ParseRegionError) -> Self {
        Self::InvalidRegion(err)
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRegion(err) => write!(f, "{}", err)
        }
    }
}
