#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(clippy::missing_errors_doc)]
#![forbid(unsafe_code)]
pub mod archive;
pub mod client;
pub mod exchange;
pub mod multi_value;
pub mod request;
