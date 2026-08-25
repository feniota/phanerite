//! Data storage for Phanerite

pub mod migration;
pub mod storage_registry;

#[cfg(any(not(debug_assertions), feature = "no-memory-db"))]
use std::sync::Arc;

use turso::Database as Db;

/// Handle to a Turso Database.
///
/// Can be cloned cheaply, still pointing to the same database.
#[derive(Clone)]
pub struct Database {
    /// turso::Database
    ///
    /// It wraps an Arc so can be cheaply cloned
    pub inner: Db,
}

impl Database {
    #[cfg(any(not(debug_assertions), feature = "no-memory-db"))]
    async fn new_fs() -> anyhow::Result<Db> {
        let io: Arc<dyn turso_core::IO + 'static>;
        #[cfg(target_os = "linux")]
        match turso_core::UringIO::new() {
            Ok(i) => io = Arc::new(i),
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize io_uring IO, falling back to default: {}",
                    e
                );
                io = Arc::new(turso_core::PlatformIO::new()?);
            }
        }
        #[cfg(target_os = "windows")]
        {
            io = Arc::new(turso_core::WindowsIOCP::new()?);
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            io = Arc::new(turso_core::PlatformIO::new()?);
        }

        let path = dirs::data_local_dir()
            .ok_or(anyhow::anyhow!("Failed to get the local data dir"))?
            .join(crate::APP_ID)
            .join("database");
        let db = turso::Builder::new_local(&path.to_string_lossy())
            .with_io_impl(io)
            .build()
            .await?;
        Ok(db)
    }

    #[allow(unused)]
    async fn new_memory() -> anyhow::Result<Db> {
        turso::Builder::new_local(":memory:")
            .build()
            .await
            .map_err(|e| e.into())
    }

    /// Create a new [`Database`] on the default path
    pub fn new() -> Self {
        gpui::block_on(async {
            #[cfg(any(not(debug_assertions), feature = "no-memory-db"))]
            let new = Self::new_fs().await;

            #[cfg(all(debug_assertions, not(feature = "no-memory-db")))]
            let new = Self::new_memory().await;

            let db = match new {
                Ok(d) => d,
                Err(e) => {
                    tracing::warn!(
                        "Failed to initialize on-disk Turso database, falling back to memory..."
                    );
                    tracing::warn!("{:?}", e);

                    turso::Builder::new_local(":memory:").build().await.unwrap()
                }
            };

            db.connect()
                .expect("Failed to connect to the database")
                .pragma_update("journal_mode", "mvcc")
                .await
                .unwrap();
            Self { inner: db }
        })
    }
}

impl std::ops::Deref for Database {
    type Target = Db;
    fn deref(&self) -> &Db {
        &self.inner
    }
}
