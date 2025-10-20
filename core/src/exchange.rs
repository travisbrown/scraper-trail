use crate::{multi_value::MultiValue, request::Request};
use bounded_static::{IntoBoundedStatic, ToBoundedStatic};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("URL parse error")]
    UrlParse(#[from] url::ParseError),
    #[error("Header value error")]
    RequestHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("Header value error")]
    ResponseHeaderValue(#[from] http::header::ToStrError),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Exchange<'a, T> {
    #[serde(borrow)]
    pub request: Request<'a>,
    pub response: Response<'a, T>,
}

impl<'a, T> Exchange<'a, T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Exchange<'a, U> {
        Exchange {
            request: self.request,
            response: self.response.map(f),
        }
    }
}

impl<'a, T: IntoBoundedStatic + 'a> IntoBoundedStatic for Exchange<'a, T> {
    type Static = Exchange<'static, T::Static>;

    fn into_static(self) -> Self::Static {
        Self::Static {
            request: self.request.into_static(),
            response: self.response.into_static(),
        }
    }
}

impl<T: ToBoundedStatic> ToBoundedStatic for Exchange<'_, T> {
    type Static = Exchange<'static, T::Static>;

    fn to_static(&self) -> Self::Static {
        Self::Static {
            request: self.request.to_static(),
            response: self.response.to_static(),
        }
    }
}

impl<T: serde::ser::Serialize> Exchange<'_, T> {
    pub fn save_file<P: AsRef<Path>>(&self, base: P) -> Result<PathBuf, std::io::Error> {
        std::fs::create_dir_all(&base)?;

        let output_path = base.as_ref().join(format!(
            "{}.json",
            self.request.timestamp.timestamp_millis()
        ));

        std::fs::write(&output_path, serde_json::json!(self).to_string())?;

        Ok(output_path)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Response<'a, T> {
    #[serde(borrow)]
    pub headers: HashMap<Cow<'a, str>, MultiValue<'a>>,
    pub data: T,
}

impl<'a, T> Response<'a, T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Response<'a, U> {
        Response {
            headers: self.headers,
            data: f(self.data),
        }
    }

    pub fn and_then<U, E, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<Response<'a, U>, E> {
        f(self.data).map(|new_data| Response {
            headers: self.headers,
            data: new_data,
        })
    }
}

impl<'a, T: IntoBoundedStatic + 'a> IntoBoundedStatic for Response<'a, T> {
    type Static = Response<'static, T::Static>;

    fn into_static(self) -> Self::Static {
        Self::Static {
            headers: self
                .headers
                .into_iter()
                .map(|(key, values)| (key.into_static(), values.into_static()))
                .collect(),
            data: self.data.into_static(),
        }
    }
}

impl<T: ToBoundedStatic> ToBoundedStatic for Response<'_, T> {
    type Static = Response<'static, T::Static>;

    fn to_static(&self) -> Self::Static {
        Self::Static {
            headers: self
                .headers
                .iter()
                .map(|(key, values)| (key.to_static(), values.to_static()))
                .collect(),
            data: self.data.to_static(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Exchange;

    const APPLE_ITUNES_01_EXAMPLE: &str = include_str!("../../examples/apple-itunes-01.json");
    const GOOGLE_PLAY_01_EXAMPLE: &str = include_str!("../../examples/google-play-01.json");

    #[test]
    fn deserialize_example_apple_itunes_01() -> Result<(), Box<dyn std::error::Error>> {
        let example: Exchange<'_, serde_json::Value> =
            serde_json::from_str(APPLE_ITUNES_01_EXAMPLE)?;

        assert!(
            example
                .request
                .url
                .as_str()
                .starts_with("https://itunes.apple.com/lookup")
        );

        assert_eq!(
            example.request.timestamp,
            chrono::DateTime::from_timestamp_millis(1760252742866).unwrap()
        );

        Ok(())
    }

    #[test]
    fn deserialize_example_google_play_01() -> Result<(), Box<dyn std::error::Error>> {
        let example: Exchange<'_, serde_json::Value> =
            serde_json::from_str(GOOGLE_PLAY_01_EXAMPLE)?;

        assert!(
            example
                .request
                .url
                .as_str()
                .starts_with("https://play.google.com/_/PlayStoreUi/data/")
        );

        assert_eq!(
            example.request.timestamp,
            chrono::DateTime::from_timestamp_millis(1759391955666).unwrap()
        );

        Ok(())
    }
}
