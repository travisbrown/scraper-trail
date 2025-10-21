use crate::{archive::entry::Field, exchange::Response};

pub mod entry;
pub mod store;

pub trait Archiveable: Sized {
    type RequestParams: crate::request::params::Params;

    fn deserialize_response_field<'de, A: serde::de::MapAccess<'de>>(
        request_params: &Self::RequestParams,
        map: &mut A,
    ) -> Result<Option<(Field, Response<'de, Self>)>, A::Error>;
}
