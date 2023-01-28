use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::TimeZone;
use poco_types::types::task::OnChainTaskConfig;
use rusqlite::params;
use sea_query::{ColumnDef, Iden, Query, SqliteQueryBuilder, Table};
use sea_query_rusqlite::RusqliteBinder;

use crate::config::PocoAgentConfig;

#[derive(Clone)]
pub struct PocoDB {
    inner: Arc<Mutex<rusqlite::Connection>>,
}

unsafe impl Send for PocoDB {}

#[derive(Iden)]
pub enum AppLog {
    Table,
    Id,
    LastRunTime,
    LastBlockHeight,
    LastEventOffset,
}

#[derive(Iden)]
pub enum TaskLog {
    Table,
    TaskId,
    TaskConfig,
}

impl PocoDB {
    pub fn new(config: Arc<PocoAgentConfig>) -> anyhow::Result<Self> {
        let db_path = Path::new(&config.app.database_path);
        let is_db_file_exists = db_path.exists();
        let connection = rusqlite::Connection::open(&config.app.database_path)?;

        if !is_db_file_exists {
            Self::setup_db(&connection)?;
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn setup_db(connection: &rusqlite::Connection) -> anyhow::Result<()> {
        let sql = Table::create()
            .table(AppLog::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(AppLog::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(AppLog::LastRunTime).integer().not_null())
            .col(ColumnDef::new(AppLog::LastBlockHeight).integer().not_null())
            .col(ColumnDef::new(AppLog::LastEventOffset).integer().not_null())
            .build(SqliteQueryBuilder);

        connection.execute(&sql, params![])?;

        let (sql, value) = Query::insert()
            .into_table(AppLog::Table)
            .columns(vec![
                AppLog::LastRunTime,
                AppLog::LastBlockHeight,
                AppLog::LastEventOffset,
            ])
            .values_panic([chrono::Local::now().timestamp().into(), 0.into(), 0.into()])
            .build_rusqlite(SqliteQueryBuilder);

        connection.execute(&sql, &*value.as_params())?;

        let sql = Table::create()
            .table(TaskLog::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TaskLog::TaskId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(TaskLog::TaskConfig).text().not_null())
            .build(SqliteQueryBuilder);

        connection.execute(&sql, params![])?;

        Ok(())
    }

    pub fn get_last_run_time(&self) -> anyhow::Result<chrono::DateTime<chrono::Local>> {
        let (sql, values) = Query::select()
            .columns(vec![AppLog::LastRunTime])
            .from(AppLog::Table)
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        let mut stmt = db.prepare(&sql)?;
        let mut rows = stmt.query(&*values.as_params())?;

        let last_run_time = rows.next()?.unwrap().get(0)?;

        match chrono::Local.timestamp_opt(last_run_time, 0) {
            chrono::LocalResult::Single(dt) => Ok(dt),
            _ => Err(anyhow::anyhow!("Invalid last run time")),
        }
    }

    pub fn set_last_run_time(
        &self,
        last_run_time: chrono::DateTime<chrono::Local>,
    ) -> anyhow::Result<()> {
        let (sql, values) = Query::update()
            .table(AppLog::Table)
            .values(vec![(
                AppLog::LastRunTime,
                last_run_time.timestamp().into(),
            )])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();

        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }

    pub fn get_last_block_height(&self) -> anyhow::Result<u64> {
        let (sql, values) = Query::select()
            .columns(vec![AppLog::LastBlockHeight])
            .from(AppLog::Table)
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        let mut stmt = db.prepare(&sql)?;
        let mut rows = stmt.query(&*values.as_params())?;

        let last_block_height = rows.next()?.unwrap().get(0)?;

        Ok(last_block_height)
    }

    pub fn set_last_block_height(&self, block_height: u64) -> anyhow::Result<()> {
        let (sql, values) = Query::update()
            .table(AppLog::Table)
            .values(vec![(AppLog::LastBlockHeight, block_height.into())])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }

    pub fn get_last_event_offset(&self) -> anyhow::Result<u32> {
        let (sql, values) = Query::select()
            .columns(vec![AppLog::LastEventOffset])
            .from(AppLog::Table)
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        let mut stmt = db.prepare(&sql)?;
        let mut rows = stmt.query(&*values.as_params())?;

        let last_event_offset = rows.next()?.unwrap().get(0)?;

        Ok(last_event_offset)
    }

    pub fn set_last_event_offset(&self, event_offset: u32) -> anyhow::Result<()> {
        let (sql, values) = Query::update()
            .table(AppLog::Table)
            .values(vec![(AppLog::LastEventOffset, event_offset.into())])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }

    pub fn cache_task_config(
        &self,
        task_id: u64,
        task_config: &OnChainTaskConfig,
    ) -> anyhow::Result<()> {
        let (sql, values) = Query::insert()
            .into_table(TaskLog::Table)
            .columns(vec![TaskLog::TaskId, TaskLog::TaskConfig])
            .values_panic([task_id.into(), serde_json::to_string(task_config)?.into()])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }
}
