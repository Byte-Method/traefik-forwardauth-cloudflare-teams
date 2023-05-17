use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init()
}
