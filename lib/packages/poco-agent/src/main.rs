pub mod agent;
pub mod app;
pub mod config;

#[macro_use]
extern crate lazy_static;

use std::io;

use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::trace::TracingCategory;

use crate::app::App;

fn main() -> Result<(), io::Error> {
    let config = config::parse().get_config().expect("Failed to load config");
    let log_file_appender =
        tracing_appender::rolling::daily(config.app.log_dir, config.app.log_prefix);
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(log_file_appender);

    let mut app = App::new();
    // Init Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_ansi(false)
                .with_writer(non_blocking_appender),
        )
        // .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(app.get_tracing_layer())
        .init();

    tracing::event!(
        Level::INFO,
        message = "start loading poco-agent config",
        category = format!("{:?}", TracingCategory::Agent)
    );

    tracing::event!(
        Level::INFO,
        message = "finish loading poco-agent config",
        category = format!("{:?}", TracingCategory::Agent)
    );

    app.run(config.near.rpc_endpoint)?;
    app.join();

    Ok(())
}
