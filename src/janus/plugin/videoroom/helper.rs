use super::error::VideoroomError;
use serde::Deserialize;

pub fn parse_json<'a, T>(s: &'a str) -> Result<T, VideoroomError>
where T: Deserialize<'a>
{
    serde_json::from_str(s).map_err(VideoroomError::from_json_parse_error)
}
