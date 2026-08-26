//! [`MultiRegistry`] implementation based on the Turso database.

use super::Database;
use anyhow::anyhow;
use async_trait::async_trait;
use concurrent_queue::{ConcurrentQueue, PopError};
use phanerite_core::storage::shared::{Error, MultiRegistry, Path, Result};
use phanerite_core::{storage::StorageIdent, utils::Hash};
use std::sync::Arc;
use std::{ops::Deref, path::PathBuf};
use turso::{Connection, Database as Db};

const HELD_CONNECTIONS: usize = 8_usize;

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

#[async_trait]
impl MultiRegistry for StorageReg {
    async fn query(&self, key: (&StorageIdent, &Hash)) -> Option<Path<'_>> {
        let conn = self.pool.fetch().ok()?;
        let mut rows = conn
            .query(
                r#"SELECT path
                    FROM storage_registry
                    WHERE storage = ?1 AND hash = ?2
                "#,
                (key.0.root_dir.to_string_lossy().as_ref(), key.1.as_bytes()),
            )
            .await
            .ok()?;
        let row = rows.next().await.ok()??;
        Some(Path::new_owned(row.get::<String>(0).ok()?.into()))
    }

    async fn query_and_increase(&self, key: (&StorageIdent, &Hash)) -> Result<Option<Path<'_>>> {
        let conn = self.pool.fetch().map_err(|e| anyhow!(e))?;
        let mut begin = conn
            .prepare_cached("BEGIN CONCURRENT")
            .await
            .map_err(|e| anyhow!(e))?;
        let mut update = conn
            .prepare_cached(
                "UPDATE storage_registry
                 SET ref_count = ref_count + 1
                 WHERE storage = ?1 AND hash = ?2",
            )
            .await
            .map_err(|e| anyhow!(e))?;
        let mut select = conn
            .prepare_cached(
                "SELECT path FROM storage_registry
                 WHERE storage = ?1 AND hash = ?2",
            )
            .await
            .map_err(|e| anyhow!(e))?;
        let mut commit = conn
            .prepare_cached("COMMIT")
            .await
            .map_err(|e| anyhow!(e))?;
        let mut rollback = conn
            .prepare_cached("ROLLBACK")
            .await
            .map_err(|e| anyhow!(e))?;

        begin.execute(()).await.map_err(|e| anyhow!(e))?;
        let storage = key.0.root_dir.to_string_lossy();
        let params = (storage.as_ref(), key.1.as_bytes());
        update.execute(params).await.map_err(|e| anyhow!(e))?;
        let mut rows = select.query(params).await.map_err(|e| anyhow!(e))?;
        let Some(row) = rows.next().await.map_err(|e| anyhow!(e))? else {
            let _ = rollback.execute(()).await;
            return Ok(None);
        };
        let path = row.get::<String>(0).map_err(|e| anyhow!(e))?;
        commit.execute(()).await.map_err(|e| anyhow!(e))?;
        Ok(Some(Path::new_owned(path.into())))
    }

    async fn insert(&self, key: (&StorageIdent, Hash), val: PathBuf) -> Result<()> {
        let conn = self.pool.fetch().map_err(|e| anyhow!(e))?;
        let mut begin = conn
            .prepare_cached("BEGIN CONCURRENT")
            .await
            .map_err(|e| anyhow!(e))?;
        let mut insert = conn
            .prepare_cached(
                "INSERT INTO storage_registry (storage, hash, path, ref_count)
                 VALUES (?1, ?2, ?3, 1)",
            )
            .await
            .map_err(|e| anyhow!(e))?;
        let mut commit = conn
            .prepare_cached("COMMIT")
            .await
            .map_err(|e| anyhow!(e))?;
        let mut rollback = conn
            .prepare_cached("ROLLBACK")
            .await
            .map_err(|e| anyhow!(e))?;

        begin.execute(()).await.map_err(|e| anyhow!(e))?;
        let storage = key.0.root_dir.to_string_lossy();
        let result = insert
            .execute((
                storage.as_ref(),
                key.1.as_bytes(),
                val.to_string_lossy().as_ref(),
            ))
            .await;
        if let Err(error) = result {
            let _ = rollback.execute(()).await;
            return Err(anyhow!(error).into());
        }
        commit.execute(()).await.map_err(|e| anyhow!(e))?;
        Ok(())
    }

    async fn decrease(&self, key: (&StorageIdent, &Hash)) -> Result<u32> {
        let conn = self.pool.fetch().map_err(|e| anyhow!(e))?;
        let mut begin = conn
            .prepare_cached("BEGIN CONCURRENT")
            .await
            .map_err(|e| anyhow!(e))?;
        let mut update = conn
            .prepare_cached(
                "UPDATE storage_registry
                 SET ref_count = ref_count - 1
                 WHERE storage = ?1 AND hash = ?2 AND ref_count > 0",
            )
            .await
            .map_err(|e| anyhow!(e))?;
        let mut select = conn
            .prepare_cached(
                "SELECT ref_count FROM storage_registry
                 WHERE storage = ?1 AND hash = ?2",
            )
            .await
            .map_err(|e| anyhow!(e))?;
        let mut delete = conn
            .prepare_cached(
                "DELETE FROM storage_registry
                 WHERE storage = ?1 AND hash = ?2 AND ref_count = 0",
            )
            .await
            .map_err(|e| anyhow!(e))?;
        let mut commit = conn
            .prepare_cached("COMMIT")
            .await
            .map_err(|e| anyhow!(e))?;
        let mut rollback = conn
            .prepare_cached("ROLLBACK")
            .await
            .map_err(|e| anyhow!(e))?;

        begin.execute(()).await.map_err(|e| anyhow!(e))?;
        let storage = key.0.root_dir.to_string_lossy();
        let params = (storage.as_ref(), key.1.as_bytes());
        update.execute(params).await.map_err(|e| anyhow!(e))?;
        let mut rows = select.query(params).await.map_err(|e| anyhow!(e))?;
        let Some(row) = rows.next().await.map_err(|e| anyhow!(e))? else {
            let _ = rollback.execute(()).await;
            return Err(Error::EntryNotFound);
        };
        let count = row.get::<u32>(0).map_err(|e| anyhow!(e))?;
        if count == 0 {
            delete.execute(params).await.map_err(|e| anyhow!(e))?;
        }
        commit.execute(()).await.map_err(|e| anyhow!(e))?;
        Ok(count)
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
