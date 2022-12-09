#![feature(async_closure)]
#![feature(fn_traits)]

pub mod agent;
pub mod app;
pub mod config;
pub mod ipfs;

use std::io;

use time::{format_description, UtcOffset};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::trace::TracingCategory;
use crate::app::App;

fn main() -> Result<(), io::Error> {
    let config = config::parse().get_config().expect("Failed to load config");
    let log_file_appender = tracing_appender::rolling::daily(
        config.log.directory.to_string(),
        config.log.prefix.to_string(),
    );
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(log_file_appender);
    let format = Box::leak(Box::new(config.log.time_format.to_string()));

    let mut app = App::new(config);
    // Init Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_ansi(false)
                .with_writer(non_blocking_appender)
                .with_timer(OffsetTime::new(
                    UtcOffset::current_local_offset().unwrap(),
                    format_description::parse(format.as_str()).unwrap(),
                )),
        )
        // .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(app.get_tracing_layer())
        .init();

    tracing::event!(
        Level::INFO,
        message = "start loading poco-agent config",
        category = TracingCategory::Agent.to_string()
    );

    tracing::event!(
        Level::INFO,
        message = "finish loading poco-agent config",
        category = TracingCategory::Agent.to_string()
    );

    app.run()?;
    app.join();

    Ok(())
}