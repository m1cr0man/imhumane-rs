use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to parse provided challenge ID"))]
    ParseUuid { source: uuid::Error },
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = self.to_string();
        match self {
            Error::ParseUuid { source: _ } => (StatusCode::BAD_REQUEST, message).into_response(),
        }
    }
}
