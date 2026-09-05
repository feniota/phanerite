//! Data storage for Phanerite

pub mod migration;
// Disabled while the storage-registry backend design is under discussion.
// pub mod storage_registry;

use std::path::PathBuf;
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
    async fn new_fs(path: Option<PathBuf>) -> anyhow::Result<Db> {
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

        let path = match path {
            Some(path) => path,
            None => dirs::data_local_dir()
                .ok_or(anyhow::anyhow!("Failed to get the local data dir"))?
                .join(crate::APP_ID),
        };

        if let Err(e) = async_fs::create_dir_all(&path).await
            && !matches!(e.kind(), std::io::ErrorKind::AlreadyExists)
        {
            return Err(e.into());
        }

        let path = path.join("database");
        let db = turso::Builder::new_local(&path.to_string_lossy())
            .with_io_impl(io)
            .build()
            .await?;
        Ok(db)
    }

    async fn new_memory() -> anyhow::Result<Db> {
        turso::Builder::new_local(":memory:")
            .build()
            .await
            .map_err(|e| e.into())
    }

    /// Create a new [`Database`] on the default path
    pub fn new() -> Self {
        gpui_kit::block_on(async {
            #[cfg(debug_assertions)]
            let disk_path = {
                let _ = dotenvy::dotenv();
                std::env::var_os("PHANERITE_DB_PATH").map(PathBuf::from)
            };

            #[cfg(not(debug_assertions))]
            let disk_path = None;

            let new = if cfg!(debug_assertions) && disk_path.is_none() {
                Self::new_memory().await
            } else {
                Self::new_fs(disk_path).await
            };

            let db = match new {
                Ok(d) => d,
                Err(e) => {
                    #[cfg(not(debug_assertions))]
                    panic!("Failed to initialize the on-disk Turso database: {e:#}");

                    #[cfg(debug_assertions)]
                    {
                        tracing::warn!(
                            "Failed to initialize on-disk Turso database, falling back to memory: {}",
                            e
                        );
                        Self::new_memory().await.unwrap()
                    }
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
