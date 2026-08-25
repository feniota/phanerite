//! [`StorageRegistry`] implementation based on the Turso database

use super::Database;
use concurrent_queue::{ConcurrentQueue, PopError};
use phanerite_core::download::dedup::PathBufWrapper;
use phanerite_core::{download::dedup::StorageRegistry, storage::StorageIdent, utils::Hash};
use std::sync::Arc;
use std::{ops::Deref, path::PathBuf};
use turso::{Connection, Database as Db};

const HELD_CONNECTIONS: usize = 8_usize;
const INSERT_RETRIES: usize = 8_usize;

fn is_retryable_write_error(error: &turso::Error) -> bool {
    match error {
        turso::Error::Busy(_) | turso::Error::BusySnapshot(_) => true,
        turso::Error::Error(message) => message.to_ascii_lowercase().contains("conflict"),
        _ => false,
    }
}

pub struct StorageReg {
    pool: Pool,
}

impl StorageReg {
    pub async fn new(db: Database) -> Self {
        let db = db.inner.clone();
        let queue = ConcurrentQueue::bounded(HELD_CONNECTIONS);
        for _ in 0_usize..HELD_CONNECTIONS {
            let conn = db
                .connect()
                .expect("Failed to establish connection to the DB");
            queue.push(conn).unwrap();
        }
        Self {
            pool: Pool {
                conns: Arc::new(queue),
                db,
            },
        }
    }
}

impl StorageRegistry for StorageReg {
    async fn query(&self, key: &(StorageIdent, Hash)) -> Option<impl Deref<Target = PathBuf>> {
        let conn = self.pool.fetch().ok()?;
        let stmt = conn
            .prepare_cached(
                r#"SELECT path
                    FROM storage_registry
                    WHERE storage = ?1 AND hash = ?2
                "#,
            )
            .await;
        if let Err(e) = stmt {
            if !matches!(e, turso::Error::QueryReturnedNoRows) {
                tracing::error!("Preparing query statement failed: {}", e);
            }
            return None;
        }
        let mut stmt = stmt.unwrap();

        let storage = key.0.root_dir.to_string_lossy();
        let hash = key.1.as_bytes();
        let row = stmt.query_row((storage.as_ref(), hash)).await;
        if let Err(e) = row {
            tracing::error!("Querying for storage item failed: {}", e);
            return None;
        }
        let row = row.unwrap();
        let path = row.get::<String>(0).ok()?;

        Some(PathBufWrapper::from(path))
    }

    async fn insert(&self, key: (StorageIdent, Hash), val: PathBuf) {
        let ret: anyhow::Result<()> = async {
            let conn = self.pool.fetch()?;
            let mut begin = conn.prepare_cached("BEGIN CONCURRENT").await?;
            let mut insert = conn
                .prepare_cached(
                    r#"INSERT INTO storage_registry
                        (storage, hash, path)
                        VALUES (?1, ?2, ?3)
                        ON CONFLICT DO NOTHING"#,
                )
                .await?;
            let mut commit = conn.prepare_cached("COMMIT").await?;
            let mut rollback = conn.prepare_cached("ROLLBACK").await?;

            let storage = key.0.root_dir.to_string_lossy();
            let hash = key.1.as_bytes();
            let path = val.to_string_lossy();

            for attempt in 0..INSERT_RETRIES {
                if let Err(error) = begin.execute(()).await {
                    if is_retryable_write_error(&error) && attempt + 1 < INSERT_RETRIES {
                        std::thread::yield_now();
                        continue;
                    }
                    return Err(error.into());
                }

                let result = async {
                    insert
                        .execute((storage.as_ref(), hash, path.as_ref()))
                        .await?;
                    commit.execute(()).await?;
                    Ok::<_, turso::Error>(())
                }
                .await;

                match result {
                    Ok(()) => return Ok(()),
                    Err(error) => {
                        let _ = rollback.execute(()).await;
                        if is_retryable_write_error(&error) && attempt + 1 < INSERT_RETRIES {
                            std::thread::yield_now();
                            continue;
                        }
                        return Err(error.into());
                    }
                }
            }

            unreachable!("the insert retry loop always returns")
        }
        .await;
        if let Err(e) = ret {
            tracing::error!(
                "Inserting into storage registry database failed, skipping: {}",
                e
            );
        }
    }
}

pub struct ConnGuard {
    inner: Connection,
    queue: Option<Arc<ConcurrentQueue<Connection>>>,
}

impl Deref for ConnGuard {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for ConnGuard {
    fn drop(&mut self) {
        if self.queue.is_some() {
            let queue = self.queue.take().unwrap();
            // turso::Connection is also based on several `Arc`s, so multiple clones
            // of the same Connection don't actually create multiple connections.
            let r = queue.push(self.inner.clone());
            if r.is_err() {
                // Drop implements should not panic
                // however this is still a considerable error
                tracing::error!("Pushing a connection back to the pool failed: {:?}", r);
            }
        }
    }
}

#[derive(Clone)]
pub struct Pool {
    pub(super) conns: Arc<ConcurrentQueue<Connection>>,
    pub(super) db: Db,
}

impl Pool {
    /// Fetch a connection from the pool
    ///
    /// If the pool is empty (all connections held are in use),
    /// a new connection is fetched from the database and a
    /// warning is yielded.
    pub(super) fn fetch(&self) -> anyhow::Result<ConnGuard> {
        let i = self.conns.pop();
        match i {
            Ok(i) => Ok(ConnGuard {
                inner: i,
                queue: Some(self.conns.clone()),
            }),
            Err(PopError::Empty) => {
                tracing::warn!(
                    pool_size = HELD_CONNECTIONS,
                    "Storage registry connection pool exhausted; using a temporary connection"
                );

                Ok(ConnGuard {
                    inner: self.db.connect()?,
                    queue: None,
                })
            }
            Err(PopError::Closed) => {
                panic!("The connection queue shall not be closed")
            }
        }
    }
}
