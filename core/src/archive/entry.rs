use crate::{
    archive::Archiveable,
    exchange::Exchange,
    request::{Request, params::Params},
};
use std::borrow::Cow;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
pub enum Field {
    Request,
    Response,
}

pub struct Entry<'a, T: Archiveable> {
    pub request_params: T::RequestParams<'a>,
    pub exchange: Exchange<'a, T>,
}

impl<'a, 'de: 'a, T: Archiveable + 'a> serde::de::Deserialize<'de> for Entry<'a, T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntryVisitor<'a, T>(std::marker::PhantomData<&'a T>);

        impl<'a, 'de: 'a, T: Archiveable> serde::de::Visitor<'de> for EntryVisitor<'a, T> {
            type Value = Entry<'a, T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("scraper exchange archive entry")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let request = map
                    .next_entry::<Field, Request<'_>>()?
                    .and_then(|(field, request)| {
                        if field == Field::Request {
                            Some(request)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| serde::de::Error::missing_field("request"))?;

                let request_params = T::RequestParams::parse_request(&request)
                    .map_err(|error| error.serde(&request))?;

                let response = T::deserialize_response_field(&request_params, &mut map)?
                    .and_then(|(field, data)| {
                        if field == Field::Response {
                            Some(data)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| serde::de::Error::missing_field("response"))?;

                match map.next_entry::<Cow<'_, str>, serde::de::IgnoredAny>()? {
                    Some((field, _)) => Err(serde::de::Error::unknown_field(
                        &field,
                        &["request", "response"],
                    )),
                    None => Ok(Entry {
                        request_params,
                        exchange: Exchange { request, response },
                    }),
                }
            }
        }

        deserializer.deserialize_map(EntryVisitor(std::marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::{Archiveable, Entry, Field};
    use crate::exchange::Response;
    use regex::Regex;
    use std::borrow::Cow;
    use std::sync::LazyLock;

    const GOOGLE_PLAY_01_EXAMPLE: &str = include_str!("../../../examples/google-play-01.json");

    #[test]
    fn deserialize_google_archive() -> Result<(), Box<dyn std::error::Error>> {
        let archive = serde_json::from_str::<Entry<'_, GoogleData>>(GOOGLE_PLAY_01_EXAMPLE)?;

        assert_eq!(archive.request_params.pagination.country, "us");
        assert_eq!(archive.request_params.review.app_id, "ai.chesslegends");
        assert!(matches!(
            archive.exchange.response.data,
            GoogleData::Review(serde_json::Value::Array(_))
        ));

        Ok(())
    }

    struct ReviewRequest<'a> {
        pagination: Pagination<'a>,
        review: Review,
    }

    impl<'a> crate::request::params::Params<'a> for ReviewRequest<'a> {
        fn build_request(
            &'a self,
            _timestamp: Option<chrono::DateTime<chrono::Utc>>,
        ) -> crate::request::Request<'a> {
            // Not tested here.
            todo![]
        }

        fn parse_request(
            request: &crate::request::Request<'_>,
        ) -> Result<Self, crate::request::params::ParseError> {
            let pagination = request.url.as_str().parse().map_err(|_| {
                crate::request::params::ParseError::InvalidUrl {
                    expected: "Google review pagination request",
                }
            })?;

            let review = request
                .body
                .as_ref()
                .and_then(|body| body.parse().ok())
                .ok_or_else(|| crate::request::params::ParseError::InvalidUrl {
                    expected: "Google review pagination request",
                })?;

            Ok(Self { pagination, review })
        }
    }

    enum GoogleData {
        Review(serde_json::Value),
    }

    impl Archiveable for GoogleData {
        type RequestParams<'a> = ReviewRequest<'a>;

        fn deserialize_response_field<'a, 'de: 'a, A: serde::de::MapAccess<'de>>(
            _request_params: &Self::RequestParams<'a>,
            map: &mut A,
        ) -> Result<Option<(Field, Response<'a, Self>)>, A::Error> {
            Ok(map
                .next_entry::<Field, Response<'a, serde_json::Value>>()?
                .map(|(field, response)| (field, response.map(|value| GoogleData::Review(value)))))
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Pagination<'a> {
        pub language: Cow<'a, str>,
        pub country: Cow<'a, str>,
    }

    impl std::str::FromStr for Pagination<'static> {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            static LANGUAGE_AND_COUNTRY_RE: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(r"hl=([a-z]{2}).*gl=([a-z]{2})").unwrap());

            LANGUAGE_AND_COUNTRY_RE
                .captures(s)
                .and_then(|captures| captures.get(1).zip(captures.get(2)))
                .map(|(language, country)| Self {
                    language: language.as_str().to_string().into(),
                    country: country.as_str().to_string().into(),
                })
                .ok_or_else(|| s.to_string())
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct Review {
        pub app_id: String,
        pub sort_order: u8,
        pub number: usize,
        pub token: Option<String>,
    }

    impl std::str::FromStr for Review {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            static REVIEW_RE: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r#"^f\.req=\[\[\["UsvDTd","\[null,null,\[2,(\d+),\[(\d+),null,([^\]]+)\],null,\[\]\],\[\\"([^\]]+)\\",7\]\]",null,"generic"\]\]\]$"#).unwrap()
            });

            let decoded = urlencoding::decode(s).map_err(|_| s.to_string())?;

            REVIEW_RE
                .captures(&decoded)
                .and_then(|captures| {
                    captures
                        .get(1)
                        .zip(captures.get(2))
                        .zip(captures.get(3))
                        .zip(captures.get(4))
                        .and_then(
                            |(((sort_order_match, number_match), token_match), app_id_match)| {
                                sort_order_match
                                    .as_str()
                                    .parse::<u8>()
                                    .ok()
                                    .zip(number_match.as_str().parse::<usize>().ok())
                                    .zip(match token_match.as_str() {
                                        "null" => Some(None),
                                        other
                                            if other.starts_with(r#"\""#)
                                                && other.ends_with(r#"\""#) =>
                                        {
                                            Some(Some(other[2..other.len() - 2].to_string()))
                                        }
                                        _ => None,
                                    })
                                    .map(|((sort_order, number), token)| Self {
                                        app_id: app_id_match.as_str().to_string(),
                                        sort_order,
                                        number,
                                        token,
                                    })
                            },
                        )
                })
                .ok_or_else(|| s.to_string())
        }
    }
}
