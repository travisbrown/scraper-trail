use bounded_static::{IntoBoundedStatic, ToBoundedStatic};
use chrono::{DateTime, Utc};
use http::{
    Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use indexmap::IndexMap;
use serde_field_attributes::{represented_as_str, timestamp_millis_str};
use std::borrow::Cow;
use url::Url;

pub mod params;

#[derive(Debug, thiserror::Error)]
pub enum HeaderError {
    #[error("Invalid header name")]
    Name(#[from] http::header::InvalidHeaderName),
    #[error("Invalid header value")]
    Value(#[from] http::header::InvalidHeaderValue),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Request<'a> {
    pub url: Url,
    #[serde(rename = "timestamp_ms", with = "timestamp_millis_str")]
    pub timestamp: DateTime<Utc>,
    #[serde(
        with = "represented_as_str",
        default,
        skip_serializing_if = "is_method_get"
    )]
    pub method: Method,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub headers: IndexMap<Cow<'a, str>, Cow<'a, str>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Cow<'a, str>>,
}

impl<'a> Request<'a> {
    pub fn new<
        U: AsRef<str>,
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
        I: IntoIterator<Item = (K, V)>,
        B: Into<Cow<'a, str>>,
    >(
        url: U,
        timestamp: Option<DateTime<Utc>>,
        method: Option<Method>,
        headers: Option<I>,
        body: Option<B>,
    ) -> Result<Self, url::ParseError> {
        Ok(Self {
            url: url.as_ref().parse()?,
            timestamp: timestamp.unwrap_or_else(Utc::now),
            method: method.unwrap_or_default(),
            headers: headers
                .map(|headers| {
                    headers
                        .into_iter()
                        .map(|(key, value)| (key.into(), value.into()))
                        .collect()
                })
                .unwrap_or_default(),
            body: body.map(std::convert::Into::into),
        })
    }

    pub fn header_map(&self) -> Result<HeaderMap, HeaderError> {
        self.headers
            .iter()
            .map(|(name, value)| {
                Ok((
                    HeaderName::try_from(name.as_ref())?,
                    HeaderValue::try_from(value.as_ref())?,
                ))
            })
            .collect()
    }
}

impl IntoBoundedStatic for Request<'_> {
    type Static = Request<'static>;

    fn into_static(self) -> Self::Static {
        Self::Static {
            url: self.url,
            timestamp: self.timestamp,
            method: self.method,
            headers: self
                .headers
                .into_iter()
                .map(|(key, value)| (key.into_static(), value.into_static()))
                .collect(),
            body: self
                .body
                .map(bounded_static::IntoBoundedStatic::into_static),
        }
    }
}

impl ToBoundedStatic for Request<'_> {
    type Static = Request<'static>;

    fn to_static(&self) -> Self::Static {
        Self::Static {
            url: self.url.clone(),
            timestamp: self.timestamp,
            method: self.method.clone(),
            headers: self
                .headers
                .iter()
                .map(|(key, value)| (key.to_static(), value.to_static()))
                .collect(),
            body: self
                .body
                .as_ref()
                .map(bounded_static::ToBoundedStatic::to_static),
        }
    }
}

fn is_method_get(method: &Method) -> bool {
    method == Method::GET
}
