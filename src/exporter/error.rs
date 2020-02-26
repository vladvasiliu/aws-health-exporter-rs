use crate::scraper::error::Error as ScraperError;
use prometheus::Error as PromError;
use std::fmt;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Error>;

pub enum Error {
    ScraperError(ScraperError),
    PromError(PromError),
}

impl Error {}

impl From<ScraperError> for Error {
    fn from(err: ScraperError) -> Self {
        Self::ScraperError(err)
    }
}

impl From<PromError> for Error {
    fn from(err: PromError) -> Self {
        Self::PromError(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScraperError(err) => write!(f, "{}", err),
            Self::PromError(err) => write!(f, "{}", err),
        }
    }
}
