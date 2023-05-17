use clap::Parser;
use reqwest::Url;
use std::sync::Arc;

mod app;
mod cli;
mod error;
mod extract;
mod jwks;
#[cfg(test)]
mod tests;
mod tracing;

#[tokio::main]
async fn main() {
    tracing::setup();

    let args = cli::Args::parse();

    let jwks = Arc::new(jwks::JwkSet::new(&format!(
        "https://{}",
        &args.teams_domain
    )));

    let app = app::router().with_state(Arc::clone(&jwks));
    let http_server = axum::Server::bind(&args.bind).serve(app.into_make_service());
    let http_handle = tokio::spawn(http_server);

    let url: Url = format!("https://{}/cdn-cgi/access/certs", &args.teams_domain)
        .parse()
        .unwrap();

    //let poll_handle = jwks::poll_jwks(reqwest::Client::new(), certs_url, Arc::clone(&jwks));
    let mut poll = jwks::Poll::new(reqwest::Client::new(), url, Arc::clone(&jwks));

    let _ = tokio::join!(poll.run(), http_handle);
}
