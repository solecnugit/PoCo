use std::ops::Deref;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicI8, AtomicU8};

use chrono::TimeZone;
use poco_types::types::BlockHeight;
use poco_types::types::event::EventNonce;
use poco_types::types::round::RoundId;
use poco_types::types::task::id::TaskId;
use poco_types::types::task::OnChainTaskConfig;
use rusqlite::params;
use sea_query::{ColumnDef, Expr, Func, Iden, Query, SqliteQueryBuilder, Table};
use sea_query_rusqlite::RusqliteBinder;

use crate::config::PocoDBConfig;

pub mod config;

pub struct InnerPocoDB {
    inner: Arc<Mutex<rusqlite::Connection>>,
    // 0: flag uninitialized
    // 1: db initialized
    // 2: db uninitialized
    initialized: AtomicU8,
}

#[derive(Clone)]
pub struct PocoDB {
    inner: Arc<InnerPocoDB>,
}

impl Deref for PocoDB {
    type Target = InnerPocoDB;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Iden)]
pub enum AppTable {
    Table,
    Id,
    LastRunTime,
    LastRoundId,
    LastBlockHeight,
    LastEventOffset,
}

#[derive(Iden)]
pub enum TaskCacheTable {
    Table,
    TaskId,
    TaskConfig,
}

#[derive(Iden)]
pub enum UserProfileTable {
    Table,
    UserId,
    Field,
    Value,
}

impl InnerPocoDB {
    pub fn new(config: Arc<PocoDBConfig>) -> anyhow::Result<Self> {
        let db_path = Path::new(&config.db_path);
        let is_db_file_exists = db_path.exists();
        let connection = rusqlite::Connection::open(&config.db_path)?;

        if !is_db_file_exists {
            Self::setup_db_schemas(&connection)?;
        }

        let connection = Arc::new(Mutex::new(connection));

        Ok(Self {
            inner: connection.clone(),
            initialized: AtomicU8::new(0),
        })
    }

    fn check_db_initialized(&self) -> anyhow::Result<bool> {
        let connection = self.inner.lock().unwrap();
        let (sql, _) = Query::select()
            .expr(Func::count(Expr::col(AppTable::Id)))
            .from(AppTable::Table)
            .build(SqliteQueryBuilder);

        let mut stmt = connection.prepare(&sql)?;
        let mut rows = stmt.query(params![])?;

        rows.next()
            .unwrap()
            .map(|row| row.get::<_, i64>(0).unwrap_or(0))
            .map(|count| count > 0)
            .ok_or(anyhow::anyhow!("Failed to get count of app table"))
    }

    fn cache_db_initialized_status(&self) {
        let initialized = self.initialized.load(std::sync::atomic::Ordering::Relaxed);

        if initialized == 0 {
            let is_initialized = self.check_db_initialized().unwrap_or(false);

            let initialized = self.initialized.load(std::sync::atomic::Ordering::Relaxed);

            if initialized == 0 {
                self.initialized.store(if is_initialized { 1 } else { 2 }, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.cache_db_initialized_status();

        let initialized = self.initialized.load(std::sync::atomic::Ordering::Relaxed);

        initialized == 1
    }

    fn setup_db_schemas(connection: &rusqlite::Connection) -> anyhow::Result<()> {
        let sql = Table::create()
            .table(AppTable::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(AppTable::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(AppTable::LastRunTime).integer().not_null())
            .col(ColumnDef::new(AppTable::LastRoundId).integer().not_null())
            .col(
                ColumnDef::new(AppTable::LastBlockHeight)
                    .integer()
                    .not_null(),
            )
            .col(
                ColumnDef::new(AppTable::LastEventOffset)
                    .integer()
                    .not_null(),
            )
            .build(SqliteQueryBuilder);

        connection.execute(&sql, params![])?;

        let sql = Table::create()
            .table(TaskCacheTable::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(TaskCacheTable::TaskId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new(TaskCacheTable::TaskConfig).text().not_null())
            .build(SqliteQueryBuilder);

        connection.execute(&sql, params![])?;

        let sql = Table::create()
            .table(UserProfileTable::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(UserProfileTable::UserId)
                    .integer()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(UserProfileTable::Field)
                    .text()
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new(UserProfileTable::Value)
                    .char_len(255)
                    .not_null(),
            )
            .build(SqliteQueryBuilder);

        connection.execute(&sql, params![])?;

        Ok(())
    }

    pub fn initialize_app_metadata(&self, last_run_time: &chrono::DateTime<chrono::Local>, last_round_id: &RoundId, last_block_height: &BlockHeight, last_event_offset: &EventNonce) {
        let (sql, values) = Query::insert()
            .into_table(AppTable::Table)
            .columns(vec![
                AppTable::LastRunTime,
                AppTable::LastRoundId,
                AppTable::LastBlockHeight,
                AppTable::LastEventOffset,
            ])
            .values_panic(vec![
                last_run_time.timestamp().into(),
                (*last_round_id).into(),
                (*last_block_height).into(),
                (*last_event_offset).into(),
            ])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();

        db.execute(&sql, &*values.as_params()).unwrap();

        self.cache_db_initialized_status();
    }

    pub fn get_last_run_time(&self) -> anyhow::Result<chrono::DateTime<chrono::Local>> {
        let (sql, values) = Query::select()
            .columns(vec![AppTable::LastRunTime])
            .from(AppTable::Table)
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
        last_run_time: &chrono::DateTime<chrono::Local>,
    ) -> anyhow::Result<()> {
        let (sql, values) = Query::update()
            .table(AppTable::Table)
            .values(vec![(
                AppTable::LastRunTime,
                last_run_time.timestamp().into(),
            )])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();

        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }

    pub fn get_last_round_id(&self) -> anyhow::Result<RoundId> {
        let (sql, values) = Query::select()
            .columns(vec![AppTable::LastRoundId])
            .from(AppTable::Table)
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        let mut stmt = db.prepare(&sql)?;
        let mut rows = stmt.query(&*values.as_params())?;

        let last_round_id = rows.next()?.unwrap().get(0)?;

        Ok(last_round_id)
    }

    pub fn set_last_round_id(&self, round_id: RoundId) -> anyhow::Result<()> {
        let (sql, values) = Query::update()
            .table(AppTable::Table)
            .values(vec![(AppTable::LastRoundId, round_id.into())])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();

        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }

    pub fn get_last_block_height(&self) -> anyhow::Result<u64> {
        let (sql, values) = Query::select()
            .columns(vec![AppTable::LastBlockHeight])
            .from(AppTable::Table)
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
            .table(AppTable::Table)
            .values(vec![(AppTable::LastBlockHeight, block_height.into())])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }

    pub fn get_last_event_offset(&self) -> anyhow::Result<u32> {
        let (sql, values) = Query::select()
            .columns(vec![AppTable::LastEventOffset])
            .from(AppTable::Table)
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
            .table(AppTable::Table)
            .values(vec![(AppTable::LastEventOffset, event_offset.into())])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();
        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }

    pub fn cache_task_config(
        &self,
        task_id: &TaskId,
        task_config: &OnChainTaskConfig,
    ) -> anyhow::Result<()> {
        let task_id: u64 = task_id.into();

        let (sql, values) = Query::insert()
            .into_table(TaskCacheTable::Table)
            .columns(vec![TaskCacheTable::TaskId, TaskCacheTable::TaskConfig])
            .values_panic([task_id.into(), serde_json::to_string(task_config)?.into()])
            .build_rusqlite(SqliteQueryBuilder);

        let db = self.inner.lock().unwrap();

        db.execute(&sql, &*values.as_params())?;

        Ok(())
    }
}

impl PocoDB {
    pub fn new(config: Arc<PocoDBConfig>) -> anyhow::Result<Self> {
        let inner = Arc::new(InnerPocoDB::new(config)?);

        Ok(Self { inner })
    }
}
