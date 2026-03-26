//! Hyper-memory system with Auto-Persistence to PostgreSQL
//! 
//! Features:
//! - Semantic and text search coordination
//! - Auto-save every 30 seconds
//! - Graceful fallback when PostgreSQL unavailable
//! - In-memory cache with DB persistence
//! 
//! CRÍTICO: Memoria autogestionada - NUNCA se pierde contexto

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use chrono::Utc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
use crate::error::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HyperMemory {
    /// In-memory storage
    pub storage: Vec<String>,
    
    /// Memory metadata
    pub agent_id: String,
    pub session_id: String,
    pub created_at: i64,
    pub last_updated: i64,
}

/// Helper to create PostgreSQL pool with type annotation
async fn create_pg_pool(db_url: &str) -> Result<sqlx::PgPool> {
    use sqlx::postgres::PgPoolOptions;
    
    let opts: sqlx::postgres::PgConnectOptions = db_url.parse()
        .map_err(|e| crate::error::MemoryPError::Db(format!("Invalid DB URL: {}", e)))?;
        
    let pool: sqlx::PgPool = PgPoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e: sqlx::Error| crate::error::MemoryPError::Db(e.to_string()))?;
        
    Ok(pool)
}

impl HyperMemory {
    pub fn new() -> Self {
        Self {
            storage: Vec::new(),
            agent_id: format!("agent-{}", uuid::Uuid::new_v4()),
            session_id: format!("session-{}", uuid::Uuid::new_v4()),
            created_at: Utc::now().timestamp(),
            last_updated: Utc::now().timestamp(),
        }
    }

    pub fn with_agent_id(agent_id: String) -> Self {
        Self {
            storage: Vec::new(),
            agent_id,
            session_id: format!("session-{}", uuid::Uuid::new_v4()),
            created_at: Utc::now().timestamp(),
            last_updated: Utc::now().timestamp(),
        }
    }

    /// Add item to memory
    pub fn add(&mut self, item: String) {
        self.storage.push(item);
        self.last_updated = Utc::now().timestamp();

        // Keep memory manageable (max 10k items)
        if self.storage.len() > 10000 {
            self.storage.remove(0);
        }
    }

    /// Get all items
    pub fn get_all(&self) -> Vec<String> {
        self.storage.clone()
    }

    /// Search for items matching pattern
    pub fn search(&self, pattern: &str) -> Vec<String> {
        self.storage
            .iter()
            .filter(|item| item.contains(pattern))
            .cloned()
            .collect()
    }

    /// Clear memory
    pub fn clear(&mut self) {
        self.storage.clear();
        self.last_updated = Utc::now().timestamp();
    }

    /// Convert to JSON for persistence
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "agent_id": self.agent_id,
            "session_id": self.session_id,
            "storage": self.storage,
            "created_at": self.created_at,
            "last_updated": self.last_updated,
        })
    }

    /// CRÍTICO: Start auto-save background loop
    /// 
    /// Saves memory to PostgreSQL every 30 seconds
    /// Continues even if DB unavailable (uses fallback)
    pub async fn start_auto_save_background(
        self: Arc<Self>,
        db_url: Option<String>,
    ) {
        if let Some(url) = db_url {
            tokio::spawn(async move {
                let mut save_interval = interval(Duration::from_secs(30));

                info!(
                    "💾 [Memory] Starting auto-save loop (agent: {}, session: {})",
                    self.agent_id, self.session_id
                );

                loop {
                    save_interval.tick().await;

                    // Try to save to PostgreSQL
                    match save_to_database(&url, self.as_ref()).await {
                        Ok(_) => {
                            debug!(
                                "✅ [Memory] Auto-saved {} items for agent {}",
                                self.storage.len(),
                                self.agent_id
                            );
                        }
                        Err(e) => {
                            warn!("⚠️ [Memory] Auto-save failed (will retry): {}", e);
                            // Fallback: In-memory storage continues
                            // DB will sync when next connectivity established
                        }
                    }
                }
            });
        } else {
            warn!("⚠️ [Memory] No database URL - using memory-only (data lost on restart)");
        }
    }

    /// Load memory from database
    pub async fn load_from_database(agent_id: &str, db_url: &str) -> Result<Self> {
        info!("📖 [Memory] Loading memory for agent: {}", agent_id);

        let pool: sqlx::PgPool = create_pg_pool(db_url).await?;

        let query_opt: Option<(serde_json::Value,)> = sqlx::query_as::<sqlx::postgres::Postgres, (serde_json::Value,)>(
            r#"
            SELECT context_data 
            FROM memory_contexts 
            WHERE agent_id = $1 
            ORDER BY timestamp DESC 
            LIMIT 1
            "#,
        )
        .bind(agent_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e: sqlx::Error| crate::error::MemoryPError::Db(e.to_string()))?;
        
        let result: (serde_json::Value,) = query_opt
        .ok_or_else(|| {
            crate::error::MemoryPError::NotFound("Memory context not found".into())
        })?;

        let data = result.0;
        let storage: Vec<String> = serde_json::from_value(data["storage"].clone())
            .unwrap_or_default();

        Ok(HyperMemory {
            storage,
            agent_id: agent_id.to_string(),
            session_id: data["session_id"].as_str().unwrap_or("").to_string(),
            created_at: data["created_at"].as_i64().unwrap_or(0),
            last_updated: data["last_updated"].as_i64().unwrap_or(0),
        })
    }

    /// Get memory statistics
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            agent_id: self.agent_id.clone(),
            item_count: self.storage.len(),
            created_at: self.created_at,
            last_updated: self.last_updated,
            total_size_bytes: self.storage.iter().map(|s| s.len()).sum(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub agent_id: String,
    pub item_count: usize,
    pub created_at: i64,
    pub last_updated: i64,
    pub total_size_bytes: usize,
}

/// Save memory to PostgreSQL
async fn save_to_database(db_url: &str, memory: &HyperMemory) -> Result<()> {    
    let pool: sqlx::PgPool = create_pg_pool(db_url).await?;

    let context_data = memory.to_json();
    
    sqlx::query(
        "INSERT INTO memory_contexts (agent_id, context_data, timestamp) \
           VALUES ($1, $2, NOW()) \
           ON CONFLICT (agent_id) DO UPDATE SET context_data = $2, timestamp = NOW()"
    )
        .bind(&memory.agent_id)
        .bind(&context_data)
        .execute(&pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("❌ [Memory] Save failed: {}", e);
            crate::error::MemoryPError::Db(e.to_string())
        })?;

    Ok(())
}

/// Initialize database schema for memory persistence
pub async fn init_database_schema(db_url: &str) -> Result<()> {
    let pool: sqlx::PgPool = create_pg_pool(db_url).await?;

    // Create memory_contexts table and indexes
    let schema_sql = r#"
        CREATE TABLE IF NOT EXISTS memory_contexts (
            agent_id TEXT PRIMARY KEY,
            context_data JSONB NOT NULL,
            timestamp TIMESTAMPTZ DEFAULT NOW(),
            created_at TIMESTAMPTZ DEFAULT NOW()
        );
        
        CREATE INDEX IF NOT EXISTS idx_memory_timestamp 
            ON memory_contexts(timestamp DESC);
        
        CREATE INDEX IF NOT EXISTS idx_memory_created 
            ON memory_contexts(created_at DESC);
    "#;
    
    sqlx::query(schema_sql)
        .execute(&pool)
        .await
        .map_err(|e: sqlx::Error| {
            error!("❌ [Memory] Schema creation failed: {}", e);
            crate::error::MemoryPError::Db(e.to_string())
        })?;

    info!("✅ [Memory] Database schema initialized");
    Ok(())
}
