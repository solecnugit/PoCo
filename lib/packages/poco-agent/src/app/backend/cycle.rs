use std::sync::Arc;
use std::time::Duration;

use poco_types::types::event::{Events, IndexedEvent};
use tokio::sync::Mutex;

use crate::agent::agent::PocoAgent;
use crate::app::backend::db::PocoDB;
use crate::app::ui::event::UIActionEvent;
use crate::app::ui::util::log_string;
use crate::config::PocoAgentConfig;

pub async fn event_cycle(config: Arc<PocoAgentConfig>,
                         db: PocoDB,
                         agent: Arc<PocoAgent>,
                         ui_sender: crossbeam_channel::Sender<UIActionEvent>,
                         runtime: Arc<tokio::runtime::Runtime>) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(config.poco.event_cycle_in_ms));

    let mut offset = db.get_last_event_offset()?;

    loop {
        interval.tick().await;

        match agent.query_events(offset, 10).await {
            Ok(events) => {
                offset += events.len() as u32;

                events
                    .into_iter()
                    .for_each(|e| {
                        let sender = ui_sender.clone();

                        runtime.spawn(async move {
                            match e.payload {
                                Events::NewRoundEvent { round_id } => log_string(&sender, format!("New round: {round_id}")),
                                Events::NewTaskEvent { task_id, .. } => log_string(&sender, format!("New task: {task_id}")),
                                Events::UserProfileFieldUpdateEvent { .. } => {}
                            }
                        });
                    });
            }
            Err(error) =>
                log_string(&ui_sender, format!("Error while querying events: {error}"))
        }

        db.set_last_event_offset(offset)?;
    }
}