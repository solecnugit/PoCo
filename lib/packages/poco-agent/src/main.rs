#![feature(async_closure)]
#![feature(fn_traits)]
#![feature(box_syntax)]

use time::{format_description, UtcOffset};
use tracing::Level;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::app::App;
use crate::app::trace::TracingCategory;

pub mod agent;
pub mod app;
pub mod config;
pub mod ipfs;
pub mod util;

fn main() -> anyhow::Result<()> {
    let app_run_config = config::parse();
    let config = app_run_config.get_config().expect("Failed to load config");
    let log_file_appender =
        tracing_appender::rolling::daily(&config.log.directory, &config.log.prefix);
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(log_file_appender);
    let format = Box::leak(Box::new(config.log.time_format.to_string()));

    let app = App::new(config);
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
                    UtcOffset::current_local_offset()
                        .unwrap_or(UtcOffset::from_hms(8, 0, 0).unwrap()),
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

    let direct_command_flag = !app_run_config.in_ui_mode;

    app.run(direct_command_flag)
}
