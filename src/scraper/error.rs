use prometheus::Error as PromError;
use rusoto_core::{request::TlsError, RusotoError};
use rusoto_credential::CredentialsError;
use rusoto_health::DescribeEventsError;
use rusoto_signature::region::ParseRegionError;
use std::{fmt, result::Result as StdResult};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    DescribeEventsError(RusotoError<DescribeEventsError>),
    InvalidRegion(ParseRegionError),
    InvalidCredentials(CredentialsError),
    TlsError(TlsError),
    PromError(PromError),
}

impl From<ParseRegionError> for Error {
    fn from(err: ParseRegionError) -> Self {
        Self::InvalidRegion(err)
    }
}

impl From<RusotoError<DescribeEventsError>> for Error {
    fn from(err: RusotoError<DescribeEventsError>) -> Self {
        Self::DescribeEventsError(err)
    }
}

impl From<CredentialsError> for Error {
    fn from(err: CredentialsError) -> Self {
        Self::InvalidCredentials(err)
    }
}

impl From<TlsError> for Error {
    fn from(err: TlsError) -> Self {
        Self::TlsError(err)
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
            Self::DescribeEventsError(err) => write!(f, "{}", err),
            Self::InvalidRegion(err) => write!(f, "{}", err),
            Self::PromError(err) => write!(f, "{}", err),
            Self::TlsError(err) => write!(f, "{}", err),
            Self::InvalidCredentials(err) => write!(f, "{}", err),
        }
    }
}
