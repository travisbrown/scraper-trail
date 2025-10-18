use crate::multi_value::MultiValue;
use crate::{
    exchange::{Exchange, Response},
    request::Request,
};
use http::{StatusCode, header::HeaderMap};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP client error")]
    Http(#[from] reqwest::Error),
    #[error("Invalid header")]
    Header(#[from] crate::request::HeaderError),
    #[error("Header value serialization error")]
    HeaderValueToStr(#[from] http::header::ToStrError),
    #[error("Unexpected status")]
    UnexpectedStatus {
        status_code: http::StatusCode,
        body: Option<String>,
    },
}

pub async fn json_send<'a>(
    request: Request<'a>,
    client: &reqwest::Client,
) -> Result<crate::exchange::Exchange<'a, serde_json::Value>, Error> {
    let builder = build_request(&request, client)?;
    let response = builder.send().await?;
    let status_code = response.status();
    let headers = response.headers();
    let headers = response_headers_to_index_map(headers)?;

    if status_code == StatusCode::OK {
        let json = response.json().await?;

        Ok(Exchange {
            request,
            response: Response {
                headers,
                data: json,
            },
        })
    } else {
        // We attempt to retrieve the body for better error messages, but ignore any failure here.
        let body = response.text().await.ok();

        Err(Error::UnexpectedStatus { status_code, body })
    }
}

fn build_request<'a>(
    request: &'a Request<'a>,
    client: &reqwest::Client,
) -> Result<reqwest::RequestBuilder, crate::request::HeaderError> {
    let builder = client
        .request(request.method.clone(), request.url.clone())
        .headers(request.header_map()?);

    Ok(if let Some(body) = request.body.as_ref() {
        builder.body(body.to_string())
    } else {
        builder
    })
}

fn response_headers_to_index_map(
    response_headers: &HeaderMap,
) -> Result<HashMap<Cow<'static, str>, MultiValue<'static>>, http::header::ToStrError> {
    let mut result: HashMap<Cow<'static, str>, MultiValue<'static>> = HashMap::new();

    for (name, value) in response_headers {
        let value = value.to_str()?;

        match result.entry(name.as_str().to_string().into()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let multi_value = entry.get_mut();
                multi_value.push(value.to_string());
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(MultiValue::new(value.to_string()));
            }
        }
    }

    Ok(result)
}
