use std::sync::Arc;
use std::time::Duration;

use crate::app::backend::event::ContractEventHandler;
use crate::app::backend::Backend;
use poco_agent::agent::PocoAgent;
use poco_db::PocoDB;

use crate::app::ui::event::{UIActionEvent, UIActionSender};
use crate::config::PocoClientConfig;

pub async fn event_microtask(it: Backend) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(
        it.config.app.microtask_interval_in_ms,
    ));

    let mut offset = it.db.get_last_event_offset()?;

    loop {
        interval.tick().await;

        match it.agent.query_events(offset, 10).await {
            Ok(events) => {
                offset += events.len() as u32;

                events.into_iter().for_each(|event| it.handle_event(event));
            }
            Err(error) => it
                .log_string(format!("Error while querying events: {error}"))
                .unwrap(),
        }

        it.db.set_last_event_offset(offset)?;
    }
}
