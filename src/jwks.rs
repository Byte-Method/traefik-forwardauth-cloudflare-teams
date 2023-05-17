use crate::error::InvalidTokenError;
use axum::{
    headers::{CacheControl, Header},
    http::header::CACHE_CONTROL,
    http::HeaderMap,
};
use jsonwebtoken::{
    decode, decode_header, jwk::Jwk, Algorithm, DecodingKey, TokenData, Validation,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tracing::error;

/// A JWK set
#[derive(Debug, Clone)]
pub struct JwkSet {
    keys: std::sync::Arc<tokio::sync::RwLock<Vec<Jwk>>>,
    issuer: String,
}

impl JwkSet {
    pub fn new(issuer: &str) -> Self {
        Self {
            keys: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            issuer: issuer.to_owned(),
        }
    }

    pub async fn decode(
        &self,
        token: &str,
        audience: &str,
    ) -> Result<TokenData<Claims>, InvalidTokenError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_nbf = true;
        validation.set_audience(&[audience]);
        validation.set_issuer(&[&self.issuer]);
        validation.set_required_spec_claims(&["exp", "nbf", "aud", "iss"]);

        let key = self.get_key_from_jwt(token).await?;

        decode::<Claims>(token, &key, &validation).map_err(|e| InvalidTokenError::Decode(e))
    }

    /// Replace any cached keys with new keys
    pub async fn update_keys(&self, new_keys: Vec<Jwk>) {
        let mut keys = self.keys.write().await;
        keys.clear();
        keys.extend(new_keys);
    }

    pub async fn get_key_from_jwt(&self, token: &str) -> Result<DecodingKey, InvalidTokenError> {
        let header = decode_header(token).map_err(|e| InvalidTokenError::Decode(e))?;
        let Some(kid) = header.kid else {
			return Err(InvalidTokenError::KeyIdMissing);
		};

        self.get_key(&kid).await
    }

    async fn get_key(&self, kid: &str) -> Result<DecodingKey, InvalidTokenError> {
        let Some(jwk) = self.find(&kid).await? else {
			return Err(InvalidTokenError::UnknownSigner)
		};

        jsonwebtoken::DecodingKey::from_jwk(&jwk).map_err(InvalidTokenError::Decode)
    }

    /// Find the key in the set that matches the given key id, if any.
    async fn find(&self, kid: &str) -> Result<Option<Jwk>, InvalidTokenError> {
        let keys = self.keys.read().await;

        if keys.is_empty() {
            return Err(InvalidTokenError::NoSigners);
        }

        Ok(keys
            .iter()
            .find(|jwk| jwk.common.key_id.is_some() && jwk.common.key_id.as_ref().unwrap() == kid)
            .map(|key| key.clone()))
    }
}

/// 10 minutes
const DEFAULT_JWK_TTL_SECS: u64 = 600;

const DEFAULT_JWK_RETRY_SECS: u64 = 6;

// pub async fn poll_jwks(client: reqwest::Client, certs_url: Url, jwks: Arc<JwkSet>) {
//     loop {
//         let jwk_ttl = match client.get(certs_url.clone()).send().await {
//             Ok(response) => {
//                 let max_age = get_max_age(response.headers()).unwrap_or_else(|err| {
//                     error!(
//                         "Unable to determine cache TTL (will invalidate in {} seconds): {:?}",
//                         DEFAULT_JWK_TTL_SECS, err
//                     );
//                     Duration::from_secs(DEFAULT_JWK_TTL_SECS)
//                 });

//                 match response.json::<jsonwebtoken::jwk::JwkSet>().await {
//                     Ok(jwk_set) => {
//                         jwks.update_keys(jwk_set.keys).await;
//                     }
//                     Err(err) => {
//                         error!("Failed to parse JWK set: {}", err);
//                     }
//                 }

//                 max_age
//             }
//             Err(err) => {
//                 error!(
//                     "Failed to fetch JWK set (will try again in {} seconds): {}",
//                     DEFAULT_JWK_RETRY_SECS, err
//                 );
//                 Duration::from_secs(DEFAULT_JWK_RETRY_SECS)
//             }
//         };

//         tokio::time::sleep(jwk_ttl).await
//     }
// }

pub struct Poll {
    client: reqwest::Client,
    uri: Url,
    jwks: Arc<JwkSet>,
    jwk_ttl: Duration,
}

impl Poll {
    pub fn new(client: reqwest::Client, uri: Url, jwks: Arc<JwkSet>) -> Self {
        Poll {
            client,
            uri,
            jwks,
            jwk_ttl: Duration::ZERO,
        }
    }

    pub async fn run(&mut self) {
        loop {
            self.update().await;

            tokio::time::sleep(self.jwk_ttl).await;
        }
    }

    async fn update(&mut self) {
        match self.client.get(self.uri.clone()).send().await {
            Ok(response) => {
                self.set_ttl(response.headers());

                match response.json::<jsonwebtoken::jwk::JwkSet>().await {
                    Ok(jwk_set) => self.jwks.update_keys(jwk_set.keys).await,
                    Err(err) => error!("Failed to parse JWK set: {}", err),
                };
            }
            Err(err) => {
                error!(
                    "Failed to fetch JWK set (will try again in {} seconds): {}",
                    DEFAULT_JWK_RETRY_SECS, err
                );
                self.jwk_ttl = Duration::from_secs(DEFAULT_JWK_RETRY_SECS);
            }
        }
    }

    fn get_max_age(headers: &HeaderMap) -> Result<Duration, anyhow::Error> {
        let cache_control = headers.get_all(CACHE_CONTROL);
        let cache_control =
            CacheControl::decode(&mut cache_control.iter()).map_err(anyhow::Error::msg)?;
        cache_control
            .max_age()
            .ok_or(anyhow::Error::msg("missing max-age directive"))
    }

    fn set_ttl(&mut self, headers: &HeaderMap) {
        self.jwk_ttl = Self::get_max_age(headers).unwrap_or_else(|err| {
            error!(
                "Unable to determine cache TTL (will invalidate in {} seconds): {:?}",
                DEFAULT_JWK_TTL_SECS, err
            );
            Duration::from_secs(DEFAULT_JWK_TTL_SECS)
        });
    }
}

/// Application token payload
///
/// See <https://developers.cloudflare.com/cloudflare-one/identity/authorization-cookie/application-token/#payload>
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
	#[cfg(test)]
    pub aud: String,
    #[cfg(test)]
	pub exp: usize,
    #[cfg(test)]
	pub iat: usize,
    #[cfg(test)]
	pub nbf: usize,
    #[cfg(test)]
	pub iss: String,
	pub email: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub identity_nonce: String,
    pub sub: String,
    pub country: String,
}
