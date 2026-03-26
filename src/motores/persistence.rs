//! Persistence layer for MEMORY_P motors.
//!
//! Each backend uses a namespaced sled tree so data is isolated per motor.
//! When external services (PostgreSQL, ClickHouse, Redis) are available,
//! these implementations can be extended to delegate to them; the sled layer
//! acts as the always-available local fallback.

use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

#[async_trait]
pub trait PersistenceLayer: Send + Sync {
    async fn save_data(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn load_data(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn Error + Send + Sync>>;
}

/// Shared sled database instance (one DB, multiple trees per backend).
fn open_sled_db() -> Result<sled::Db, sled::Error> {
    sled::open("data/memory_p_persistence")
}

// ---------------------------------------------------------------------------
// PostgresPersistence — local sled tree "postgres"
// ---------------------------------------------------------------------------

pub struct PostgresPersistence {
    tree: sled::Tree,
}

impl PostgresPersistence {
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let db = open_sled_db()?;
        let tree = db.open_tree("postgres")?;
        Ok(Self { tree })
    }

    /// Wrap in `Arc` for shared ownership across async tasks.
    pub fn shared() -> Result<Arc<Self>, Box<dyn Error + Send + Sync>> {
        Ok(Arc::new(Self::new()?))
    }
}

#[async_trait]
impl PersistenceLayer for PostgresPersistence {
    async fn save_data(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let bytes = serde_json::to_vec(value)?;
        self.tree.insert(key.as_bytes(), bytes)?;
        self.tree.flush_async().await?;
        tracing::debug!("postgres:sled persisted key={}", key);
        Ok(())
    }

    async fn load_data(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn Error + Send + Sync>> {
        match self.tree.get(key.as_bytes())? {
            Some(bytes) => {
                let val: serde_json::Value = serde_json::from_slice(&bytes)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// ClickHousePersistence — local sled tree "clickhouse"
// ---------------------------------------------------------------------------

pub struct ClickHousePersistence {
    tree: sled::Tree,
}

impl ClickHousePersistence {
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let db = open_sled_db()?;
        let tree = db.open_tree("clickhouse")?;
        Ok(Self { tree })
    }

    pub fn shared() -> Result<Arc<Self>, Box<dyn Error + Send + Sync>> {
        Ok(Arc::new(Self::new()?))
    }
}

#[async_trait]
impl PersistenceLayer for ClickHousePersistence {
    async fn save_data(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let bytes = serde_json::to_vec(value)?;
        self.tree.insert(key.as_bytes(), bytes)?;
        self.tree.flush_async().await?;
        tracing::debug!("clickhouse:sled persisted key={}", key);
        Ok(())
    }

    async fn load_data(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn Error + Send + Sync>> {
        match self.tree.get(key.as_bytes())? {
            Some(bytes) => {
                let val: serde_json::Value = serde_json::from_slice(&bytes)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// RedisPersistence — local sled tree "redis"
// ---------------------------------------------------------------------------

pub struct RedisPersistence {
    tree: sled::Tree,
}

impl RedisPersistence {
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        let db = open_sled_db()?;
        let tree = db.open_tree("redis")?;
        Ok(Self { tree })
    }

    pub fn shared() -> Result<Arc<Self>, Box<dyn Error + Send + Sync>> {
        Ok(Arc::new(Self::new()?))
    }
}

#[async_trait]
impl PersistenceLayer for RedisPersistence {
    async fn save_data(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let bytes = serde_json::to_vec(value)?;
        self.tree.insert(key.as_bytes(), bytes)?;
        self.tree.flush_async().await?;
        tracing::debug!("redis:sled cached key={}", key);
        Ok(())
    }

    async fn load_data(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn Error + Send + Sync>> {
        match self.tree.get(key.as_bytes())? {
            Some(bytes) => {
                let val: serde_json::Value = serde_json::from_slice(&bytes)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// SledPersistence — generic, any namespace
// ---------------------------------------------------------------------------

/// A generic sled-backed persistence layer with a configurable tree name.
pub struct SledPersistence {
    tree: sled::Tree,
}

impl SledPersistence {
    pub fn new(namespace: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let db = open_sled_db()?;
        let tree = db.open_tree(namespace)?;
        Ok(Self { tree })
    }

    pub fn shared(namespace: &str) -> Result<Arc<Self>, Box<dyn Error + Send + Sync>> {
        Ok(Arc::new(Self::new(namespace)?))
    }
}

#[async_trait]
impl PersistenceLayer for SledPersistence {
    async fn save_data(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let bytes = serde_json::to_vec(value)?;
        self.tree.insert(key.as_bytes(), bytes)?;
        self.tree.flush_async().await?;
        Ok(())
    }

    async fn load_data(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn Error + Send + Sync>> {
        match self.tree.get(key.as_bytes())? {
            Some(bytes) => {
                let val: serde_json::Value = serde_json::from_slice(&bytes)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }
}
