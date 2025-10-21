use chrono::{DateTime, Utc};

use super::Request;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid URL")]
    InvalidUrl { expected: &'static str },
    #[error("Invalid body")]
    InvalidBody { expected: &'static str },
    #[error("Other")]
    Other { message: &'static str },
}

impl ParseError {
    #[must_use]
    pub fn serde<E: serde::de::Error>(self, request: &Request<'_>) -> E {
        match self {
            Self::InvalidUrl { expected } => serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(request.url.as_str()),
                &expected,
            ),
            Self::InvalidBody { expected } => serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(request.body.as_deref().unwrap_or_default()),
                &expected,
            ),
            Self::Other { message } => serde::de::Error::custom(message),
        }
    }
}

pub trait Params: Sized {
    fn build_request(&self, timestamp: Option<DateTime<Utc>>) -> Request<'_>;
    fn parse_request(request: &Request<'_>) -> Result<Self, ParseError>;
}
