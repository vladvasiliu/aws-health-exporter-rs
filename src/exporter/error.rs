use crate::scraper::error::Error as ScraperError;
use std::fmt;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

pub enum Error {
    ScraperError(ScraperError),
}

impl Error {}

impl From<ScraperError> for Error {
    fn from(err: ScraperError) -> Self {
        Self::ScraperError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScraperError(err) => write!(f, "{}", err),
        }
    }
}
