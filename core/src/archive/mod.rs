use crate::{archive::entry::Field, exchange::Response};

pub mod entry;
pub mod store;

pub trait Archiveable: Sized {
    type RequestParams<'a>: crate::request::params::Params<'a>;

    fn deserialize_response_field<'a, 'de: 'a, A: serde::de::MapAccess<'de>>(
        request_params: &Self::RequestParams<'a>,
        map: &mut A,
    ) -> Result<Option<(Field, Response<'a, Self>)>, A::Error>;
}
