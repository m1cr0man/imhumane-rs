use std::sync::Arc;

use axum::{Router, extract::Json, Extension, routing::get, http::{StatusCode, header}, response::IntoResponse};
use snafu::prelude::*;
use crate::imhumane::ImHumane;
use super::{error::{Error, ParseUuidSnafu}, constants::{HEADER_ID, HEADER_TOPIC}};

#[derive(serde::Deserialize)]
pub struct PostPayload {
    challenge_id: String,
    answer: u32,
}

#[axum::debug_handler]
pub async fn challenge_post(Extension(imhumane): Extension<Arc<ImHumane>>, Json(payload): Json<PostPayload>) -> Result<impl IntoResponse, Error> {
    let challenge_id = uuid::Uuid::try_parse(&payload.challenge_id).context(ParseUuidSnafu)?;

    let answer = payload.answer;

    match imhumane.check_answer(challenge_id.to_string(), answer) {
        true => Ok(StatusCode::NO_CONTENT),
        false => Ok(StatusCode::UNAUTHORIZED),
    }
}

pub async fn challenge_get(imhumane: Extension<Arc<ImHumane>>) -> impl IntoResponse {
    let challenge = imhumane.get_challenge().await;
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE.as_str(), "image/jpeg".to_string()),
            (HEADER_ID, challenge.id),
            (HEADER_TOPIC, challenge.topic),
        ],
        challenge.image,
    )
}

pub fn get_router(service: Arc<ImHumane>) -> Router {
    Router::new()
        .route("/",
            get(challenge_get).post(challenge_post)
        )
        .layer(Extension(service))
}
