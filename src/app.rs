use crate::{jwks, error::RequestError, extract::ExtractAssertionToken};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn router() -> Router<Arc<jwks::JwkSet>> {
    Router::new()
        .route("/auth/:audience", get(handler))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

pub async fn handler(
    ExtractAssertionToken(token): ExtractAssertionToken,
    Path(audience): Path<String>,
    State(jwks): State<Arc<jwks::JwkSet>>,
) -> Result<impl IntoResponse, RequestError> {
    let token = jwks.decode(&token, &audience).await?;

    Ok(([("x-auth-user", token.claims.email)], ()))
}
