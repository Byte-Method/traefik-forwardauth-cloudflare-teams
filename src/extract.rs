use axum::{async_trait, extract::FromRequestParts, headers::HeaderName, http::request::Parts};

use crate::error::{InvalidTokenError, RequestError};

static CF_ACCESS_JWT_ASSERTION: HeaderName = HeaderName::from_static("cf-access-jwt-assertion");

pub struct ExtractAssertionToken(pub(crate) String);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractAssertionToken
where
    S: Send + Sync,
{
    type Rejection = RequestError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(&CF_ACCESS_JWT_ASSERTION)
            .ok_or(RequestError::MissingCredentials)
            .and_then(|value| {
                value
                    .to_str()
                    .map_err(|_| RequestError::InvalidToken(InvalidTokenError::Malformed))
            })?;

        Ok(Self(token.to_owned()))
    }
}
