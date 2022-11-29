pub mod app;
pub mod config;

use std::io;

use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::trace::TracingCategory;

use crate::app::App;

fn main() -> Result<(), io::Error> {
    let mut app = App::new();

    // Init Tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        // .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(app.get_tracing_layer())
        .init();

    tracing::event!(
        Level::INFO,
        message = "start loading poco-agent config",
        category = format!("{:?}", TracingCategory::Agent)
    );

    let config = config::parse().get_config().expect("Failed to load config");

    tracing::event!(
        Level::INFO,
        message = "finish loading poco-agent config",
        category = format!("{:?}", TracingCategory::Agent)
    );

    app.run(config.near.rpc_endpoint)?;
    app.join();

    Ok(())
}
