use std::fmt;

use axum::{response::{IntoResponse, Response}, http::StatusCode};

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum InvalidTokenError {
	Malformed,
	Decode(jsonwebtoken::errors::Error),
	/// JWT missing the `kid` claim.
	KeyIdMissing,
	/// Signing key lookup using `kid` claim failed for JWT.
	UnknownSigner,
	NoSigners,
}

impl fmt::Display for InvalidTokenError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			InvalidTokenError::Malformed => write!(f, "Malformed"),
			InvalidTokenError::Decode(err) => write!(f, "Decode error: {}", err),
			InvalidTokenError::KeyIdMissing => write!(f, "Missing 'kid'"),
			InvalidTokenError::UnknownSigner => write!(f, "Unknown signer"),
			InvalidTokenError::NoSigners => write!(f, "No signer keys"),
		}
	}
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum RequestError {
	/// The `CF-Access-JWT-Assertion` is missing.
    MissingCredentials,
	/// The token is not valid.
    InvalidToken(InvalidTokenError),
}

impl From<InvalidTokenError> for RequestError {
	fn from(error: InvalidTokenError) -> Self {
		Self::InvalidToken(error)
	}
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            RequestError::MissingCredentials => (StatusCode::BAD_REQUEST, String::from("Missing credentials")),
            RequestError::InvalidToken(err) => match err {
				InvalidTokenError::NoSigners => (StatusCode::SERVICE_UNAVAILABLE, format!("Please try again later")),
				err => (StatusCode::BAD_REQUEST, format!("Invalid token: {}", err))
			},
        };
        let body = error_message;
        (status, body).into_response()
    }
}
