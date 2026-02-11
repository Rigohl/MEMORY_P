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
    async fn save_data(&self, _key: &str, _value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        tracing::info!("🐘 Persisting to PostgreSQL (+pgvector): {}", _key);
        // TODO: SQL: INSERT INTO memory (key, data) VALUES (...) ON CONFLICT ...
        Ok(())
    }
    async fn load_data(&self, _key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        // TODO: SQL: SELECT data FROM memory WHERE key = $1
        Ok(None)
    }
}

pub struct ClickHousePersistence;
#[async_trait]
impl PersistenceLayer for ClickHousePersistence {
    async fn save_data(&self, _key: &str, _value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        tracing::info!("💎 Persisting to ClickHouse (OLAP): {}", _key);
        // TODO: HTTP POST to ClickHouse with INSERT INTO analytics ...
        Ok(())
    }
    async fn load_data(&self, _key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        // TODO: HTTP POST with SELECT * FROM analytics WHERE key = '{}'
        Ok(None)
    }
}

pub struct RedisPersistence;
#[async_trait]
impl PersistenceLayer for RedisPersistence {
    async fn save_data(&self, _key: &str, _value: &serde_json::Value) -> Result<(), Box<dyn Error>> {
        tracing::info!("🚀 Caching to Redis: {}", _key);
        // TODO: redis::cmd("SET").arg(key).arg(value).query_async(&mut conn)
        Ok(())
    }
    async fn load_data(&self, _key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        // TODO: redis::cmd("GET").arg(key).query_async(&mut conn)
        Ok(None)
    }
}
