use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait PersistenceLayer: Send + Sync {
    async fn save_data(&self, key: &str, value: &serde_json::Value) -> Result<(), Box<dyn Error>>;
    async fn load_data(&self, key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>>;
}

pub struct PostgresPersistence {
    #[cfg(feature = "sqlx")]
    pool: Option<sqlx::PgPool>,
}

impl PostgresPersistence {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "sqlx")]
            pool: None,
        }
    }

    #[cfg(feature = "sqlx")]
    pub fn with_pool(pool: sqlx::PgPool) -> Self {
        Self { pool: Some(pool) }
    }
}

#[async_trait]
impl PersistenceLayer for PostgresPersistence {
    async fn save_data(
        &self,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        tracing::info!("🐘 Persisting to PostgreSQL (+pgvector): {}", key);

        #[cfg(feature = "sqlx")]
        if let Some(pool) = &self.pool {
            sqlx::query(
                "INSERT INTO public.memory (key, data) \
                 VALUES ($1, $2) \
                 ON CONFLICT (key) DO UPDATE SET \
                 data = EXCLUDED.data, \
                 updated_at = CURRENT_TIMESTAMP"
            )
            .bind(key)
            .bind(value)
            .execute(pool)
            .await?;
            return Ok(());
        }

        Ok(())
    }
    async fn load_data(&self, key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        #[cfg(feature = "sqlx")]
        if let Some(pool) = &self.pool {
            let row: Option<(serde_json::Value,)> = sqlx::query_as(
                "SELECT data FROM public.memory WHERE key = $1"
            )
            .bind(key)
            .fetch_optional(pool)
            .await?;

            return Ok(row.map(|r| r.0));
        }

        Ok(None)
    }
}

pub struct ClickHousePersistence;
#[async_trait]
impl PersistenceLayer for ClickHousePersistence {
    async fn save_data(
        &self,
        _key: &str,
        _value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        tracing::info!("💎 Persisting to ClickHouse (OLAP): {}", _key);
        // TODO: HTTP POST to ClickHouse with INSERT INTO analytics ...
        Ok(())
    }
    async fn load_data(&self, _key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        Ok(None)
    }
}

pub struct RedisPersistence;
#[async_trait]
impl PersistenceLayer for RedisPersistence {
    async fn save_data(
        &self,
        _key: &str,
        _value: &serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        tracing::info!("🚀 Caching to Redis: {}", _key);
        // TODO: redis::cmd("SET").arg(key).arg(value).query_async(&mut conn)
        Ok(())
    }
    async fn load_data(&self, _key: &str) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        Ok(None)
    }
}
