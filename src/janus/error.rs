use tokio_tungstenite::tungstenite;
use http::StatusCode;

pub enum JsonError {
    ParseError(serde_json::error::Error),
    SerialError(serde_json::error::Error)
}

impl From<JsonError> for tungstenite::Error {
    fn from(e: JsonError) -> Self {
        match e {
            JsonError::ParseError(_) => tungstenite::Error::Http(StatusCode::BAD_REQUEST),
            JsonError::SerialError(_) => tungstenite::Error::Http(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
