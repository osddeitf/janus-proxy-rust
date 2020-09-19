use serde::{Serialize, Deserialize};
use crate::janus::error::{JanusError};

#[allow(non_camel_case_types)]
pub type JSON_STRING = String;

#[allow(non_camel_case_types)]
pub type JSON_STRING_SLICE<'a> = &'a str;

#[allow(non_camel_case_types)]
pub type JSON_OBJECT = serde_json::Value;

// #[allow(non_camel_case_types)]
// pub type JSON_INTEGER = i64;

#[allow(non_camel_case_types)]
pub type JSON_POSITIVE_INTEGER = u64;

#[allow(non_camel_case_types)]
pub type JSON_BOOL = bool;

#[allow(non_camel_case_types)]
pub type JSON_ARRAY<T> = Vec<T>;


pub fn is_zero(n: &JSON_POSITIVE_INTEGER) -> bool {
    *n == JSON_POSITIVE_INTEGER::MIN
}

pub fn parse<'a, T>(s: &'a str) -> Result<T, JanusError>
where T: Deserialize<'a>
{
    serde_json::from_str(s).map_err(JanusError::from_json_parse_error)
}

pub fn stringify<T>(value: &T) -> Result<String, JanusError>
where T: Serialize
{
    serde_json::to_string(value).map_err(JanusError::from_json_stringify_error)
}
