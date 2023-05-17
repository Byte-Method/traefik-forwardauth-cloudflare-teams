use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    println,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    jwks::{Claims, JwkSet},
    app,
};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use tower::{Service, ServiceExt};

fn read_jwks_from_file<P>(path: P) -> Result<jsonwebtoken::jwk::JwkSet, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
}

fn read_pem_from_file<P>(path: P) -> Result<Vec<u8>, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn get_encoding_key() -> jsonwebtoken::EncodingKey {
    jsonwebtoken::EncodingKey::from_rsa_pem(
        read_pem_from_file("src/tests/keys/private.pem")
            .unwrap()
            .as_slice(),
    )
    .unwrap()
}

fn build_request(jwt: &str) -> Request<Body> {
    Request::builder()
        .method(http::Method::GET)
        .uri("/auth/test")
        .header("cf-access-jwt-assertion", jwt)
        .body(Body::empty())
        .unwrap()
}

async fn preload_jwks(issuer: &str) -> JwkSet {
    let jwks = JwkSet::new(issuer);

    let set = read_jwks_from_file("src/tests/keys/jwks.json").unwrap();

    jwks.update_keys(set.keys).await;

    jwks
}

#[tokio::test]
async fn test_normal() {
    let teams_domain = "localhost";

    let key = get_encoding_key();

    let issuer = format!("https://{teams_domain}");

    let jwks = preload_jwks(&issuer).await;

    let mut app = app::router().with_state(Arc::new(jwks));

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let header = jsonwebtoken::Header {
        alg: jsonwebtoken::Algorithm::RS256,
        kid: Some(String::from("_eMZ_eHZcJjsjn2LvNqZi6vdoOm9w0LRUwPe5C7mwBI")),
        ..Default::default()
    };

    let claims = Claims {
        aud: String::from("test"),
        email: String::from("user@example.com"),
        exp: now + 60,
        iat: now,
        nbf: now,
        iss: issuer,
        r#type: String::from("app"),
        identity_nonce: String::from("1234"),
        sub: String::from("abc"),
        country: String::from("CA"),
    };

    let jwt = jsonwebtoken::encode(&header, &claims, &key).unwrap();

    let request = build_request(&jwt);
    let response = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("x-auth-user").unwrap(),
        &"user@example.com"
    );
}

#[tokio::test]
async fn test_errors() {
    let teams_domain = "localhost";

    let key = get_encoding_key();

    let issuer = format!("https://{teams_domain}");

    let jwks = preload_jwks(&issuer).await;

    let mut app = app::router().with_state(Arc::new(jwks));

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let header = jsonwebtoken::Header {
        alg: jsonwebtoken::Algorithm::RS256,
        kid: Some(String::from("_eMZ_eHZcJjsjn2LvNqZi6vdoOm9w0LRUwPe5C7mwBI")),
        ..Default::default()
    };

    let claims = Claims {
        aud: String::from("test"),
        email: String::from("user@example.com"),
        exp: now + 60,
        iat: now,
        nbf: now,
        iss: issuer,
        r#type: String::from("app"),
        identity_nonce: String::from("1234"),
        sub: String::from("abc"),
        country: String::from("CA"),
    };

    let cases = [
        (
            "testing wrong 'kid' header claim",
            jsonwebtoken::Header {
                kid: Some(String::from("wrong")),
                ..(header.clone())
            },
            claims.clone(),
            StatusCode::BAD_REQUEST,
            "Invalid token: Unknown signer",
        ),
        (
            "testing wrong 'alg' header claim",
            jsonwebtoken::Header {
                alg: jsonwebtoken::Algorithm::RS384,
                ..(header.clone())
            },
            claims.clone(),
            StatusCode::BAD_REQUEST,
            "Invalid token: Decode error: InvalidAlgorithm",
        ),
        (
            "testing wrong 'aud' claim",
            header.clone(),
            Claims {
                aud: String::from("wrong"),
                ..(claims.clone())
            },
            StatusCode::BAD_REQUEST,
            "Invalid token: Decode error: InvalidAudience",
        ),
        (
            "testing bad 'exp' claim",
            header.clone(),
            Claims {
                iat: now - 3600 - 60,
                nbf: now - 3600 - 60,
                exp: now - 3600,
                ..(claims.clone())
            },
            StatusCode::BAD_REQUEST,
            "Invalid token: Decode error: ExpiredSignature",
        ),
        (
            "testing bad 'nbf' claim",
            header.clone(),
            Claims {
                iat: now,
                nbf: now + 120,
                exp: now + 3600,
                ..(claims.clone())
            },
            StatusCode::BAD_REQUEST,
            "Invalid token: Decode error: ImmatureSignature",
        ),
        (
            "testing bad 'iss' claim",
            header.clone(),
            Claims {
                iss: String::from("https://example.net"),
                ..(claims.clone())
            },
            StatusCode::BAD_REQUEST,
            "Invalid token: Decode error: InvalidIssuer",
        ),
    ];

    for item in cases.into_iter() {
        let (msg, header, claims, status, result) = item;

        let jwt = jsonwebtoken::encode(&header, &claims, &key).unwrap();

        let request = build_request(&jwt);
        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        assert_eq!(response.status(), status, "{}", msg);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        println!("{:?}", body);
        assert_eq!(&body[..], result.as_bytes(), "{}", msg);
    }

    {
        let issuer = format!("https://{teams_domain}");
        let header = header.clone();
        let claims = serde_json::json!({
            "aud": String::from("test"),
            "exp": now + 60,
            "iat": now,
            "nbf": now,
            "iss": issuer,
            "type": String::from("app"),
            "identity_nonce": String::from("1234"),
            "sub": String::from("abc"),
            "country": String::from("CA"),
        });

        let jwt = jsonwebtoken::encode(&header, &claims, &key).unwrap();

        let request = build_request(&jwt);
        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        println!("{:?}", body);
    }

    {
        let request = Request::builder()
            .method(http::Method::GET)
            .uri("/auth/test")
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		println!("{:?}", body);
        assert_eq!(&body[..], b"Missing credentials", "testing missing header");
    }

	{
        let request = Request::builder()
            .method(http::Method::GET)
            .uri("/auth/test")
			.header("cf-access-jwt-assertion", "∑")
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut app)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
		println!("{:?}", body);
        assert_eq!(&body[..], b"Invalid token: Malformed", "testing malformed header");
    }
}
