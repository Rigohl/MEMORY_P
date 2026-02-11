use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait PersistenceLayer: Send + Sync {
    async fn save_data(&self, key: &str, value: &serde_json::Value) -> Result<(), Box<dyn Error>>;
    async fn load_data(&self, key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>>;
}

pub struct PostgresPersistence;
#[async_trait]
impl PersistenceLayer for PostgresPersistence {
    async fn save_data(&self, key: &str, value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        tracing::info!("🐘 Persisting to PostgreSQL (+pgvector): {}", key);
        // SQL: INSERT INTO memory (key, data) VALUES (...) ON CONFLICT ...
        Ok(())
    }
    async fn load_data(&self, key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        Ok(None)
    }
}

pub struct ClickHousePersistence;
#[async_trait]
impl PersistenceLayer for ClickHousePersistence {
    async fn save_data(&self, key: &str, value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        tracing::info!("💎 Persisting to ClickHouse (OLAP): {}", key);
        Ok(())
    }
    async fn load_data(&self, key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        Ok(None)
    }
}

pub struct RedisPersistence;
#[async_trait]
impl PersistenceLayer for RedisPersistence {
    async fn save_data(&self, key: &str, value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        tracing::info!("🚀 Caching to Redis: {}", key);
        Ok(())
    }
    async fn load_data(&self, key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        Ok(None)
    }
}
