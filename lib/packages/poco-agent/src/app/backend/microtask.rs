use std::sync::Arc;
use std::time::Duration;

use poco_types::types::event::Events;
use tokio::task::JoinHandle;
use crate::app::backend::event::ContractEventHandler;
use poco_agent::agent::PocoAgent;
use poco_db::PocoDB;

use crate::app::ui::event::UIActionEvent;
use crate::app::ui::util::log_string;
use crate::config::{PocoClientConfig, PocoTaskPolicy};

use super::Backend;

pub async fn event_microtask(
    // config: Arc<PocoClientConfig>,
    // db: PocoDB,
    // agent: Arc<PocoAgent>,
    // ui_sender: crossbeam_channel::Sender<UIActionEvent>,
    // runtime: Arc<tokio::runtime::Runtime>,
    it: Backend
) -> anyhow::Result<()> {
    // 创建间隔定时器
    let mut interval =
        tokio::time::interval(Duration::from_millis(it.config.app.microtask_interval_in_ms));

    let mut offset = it.db.get_last_event_offset()?;

    loop {
        // 等待定时器的下一个事件
        interval.tick().await;

        match it.agent.query_events(offset, 10).await {
            Ok(events) => {
                offset += events.len() as u32;

                events.into_iter().for_each(|e| it.handle_event(e));

            }
            Err(error) => log_string(&it.ui_sender, format!("Error while querying events: {error}")),
        }

        it.db.set_last_event_offset(offset)?;
    }
}
