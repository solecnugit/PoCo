use std::sync::Arc;
use std::time::Duration;

use poco_types::types::event::{Events, IndexedEvent};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::agent::agent::PocoAgent;
use crate::app::backend::db::PocoDB;
use crate::app::ui::event::UIActionEvent;
use crate::app::ui::util::log_string;
use crate::config::{PocoAgentConfig, PocoTaskPolicy};

pub async fn event_cycle(
    config: Arc<PocoAgentConfig>,
    db: PocoDB,
    agent: Arc<PocoAgent>,
    ui_sender: crossbeam_channel::Sender<UIActionEvent>,
    runtime: Arc<tokio::runtime::Runtime>,
) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(config.poco.event_cycle_in_ms));

    let mut offset = db.get_last_event_offset()?;

    loop {
        interval.tick().await;

        match agent.query_events(offset, 10).await {
            Ok(events) => {
                offset += events.len() as u32;

                events
                    .into_iter()
                    .map(|e| {
                        let sender = ui_sender.clone();
                        let policy = config.poco.task_policy.clone();
                        let db = db.clone();

                        runtime.spawn(async move {
                            match e.payload {
                                Events::NewRoundEvent { round_id } => {
                                    log_string(&sender, format!("New round: {round_id}"))
                                }
                                Events::NewTaskEvent {
                                    task_id,
                                    task_config,
                                } => {
                                    log_string(&sender, format!("New task: {task_id}"));

                                    let task_id_u64: u64 = task_id.into();

                                    db.cache_task_config(task_id_u64, &task_config)?;

                                    match policy {
                                        PocoTaskPolicy::AlwaysTaken => {
                                            // let _ = agent.take_task(task_id).await;
                                        }
                                        PocoTaskPolicy::AlwaysIgnore => {}
                                    }
                                }
                                Events::UserProfileFieldUpdateEvent { .. } => {}
                            }

                            Ok(())
                        })
                    })
                    .collect::<Vec<JoinHandle<anyhow::Result<()>>>>();
            }
            Err(error) => log_string(&ui_sender, format!("Error while querying events: {error}")),
        }

        db.set_last_event_offset(offset)?;
    }
}
