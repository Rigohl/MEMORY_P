-- ==========================================
-- MEMORY_P PostgreSQL Initialization
-- ==========================================

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create schemas for motor isolation
CREATE SCHEMA IF NOT EXISTS qdrant_meta;
CREATE SCHEMA IF NOT EXISTS tantivy_meta;
CREATE SCHEMA IF NOT EXISTS lnx_meta;
CREATE SCHEMA IF NOT EXISTS meilisearch_meta;
CREATE SCHEMA IF NOT EXISTS memorybank_meta;
CREATE SCHEMA IF NOT EXISTS analytics;

-- ==========================================
-- Core Tables
-- ==========================================

-- Documents metadata
CREATE TABLE IF NOT EXISTS public.documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content TEXT NOT NULL,
    title VARCHAR(500),
    source VARCHAR(255),
    metadata JSONB,
    embedding vector(384),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index on embeddings for vector similarity search
CREATE INDEX IF NOT EXISTS idx_documents_embedding 
ON public.documents USING ivfflat (embedding vector_cosine_ops) 
WITH (lists = 100);

-- Create GIN index on metadata JSONB
CREATE INDEX IF NOT EXISTS idx_documents_metadata 
ON public.documents USING GIN (metadata);

-- ==========================================
-- Motor-specific tables
-- ==========================================

-- Qdrant synchronization tracking
CREATE TABLE IF NOT EXISTS qdrant_meta.sync_status (
    id UUID PRIMARY KEY,
    document_id UUID REFERENCES public.documents(id),
    qdrant_point_id UUID,
    synced_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50)
);

-- Tantivy index metadata
CREATE TABLE IF NOT EXISTS tantivy_meta.index_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID REFERENCES public.documents(id),
    indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    segment_id VARCHAR(255)
);

-- MeiliSearch sync tracking
CREATE TABLE IF NOT EXISTS meilisearch_meta.sync_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID REFERENCES public.documents(id),
    meili_doc_id VARCHAR(255),
    synced_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- LNX cluster coordination
CREATE TABLE IF NOT EXISTS lnx_meta.cluster_state (
    node_id VARCHAR(50) PRIMARY KEY,
    last_heartbeat TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(50),
    documents_count BIGINT
);

-- ==========================================
-- Analytics Tables
-- ==========================================

-- Search query logs
CREATE TABLE IF NOT EXISTS analytics.search_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query TEXT NOT NULL,
    engine VARCHAR(50),
    results_count INTEGER,
    latency_ms INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Performance metrics
CREATE TABLE IF NOT EXISTS analytics.engine_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    engine VARCHAR(50) NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value NUMERIC,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_engine_metrics_timestamp 
ON analytics.engine_metrics(timestamp DESC);

-- ==========================================
-- Agent Activity Tracking
-- ==========================================

CREATE TABLE IF NOT EXISTS public.agent_activity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_name VARCHAR(100) NOT NULL,
    action VARCHAR(100) NOT NULL,
    target VARCHAR(255),
    metadata JSONB,
    result TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_agent_activity_created 
ON public.agent_activity(created_at DESC);

-- ==========================================
-- Functions
-- ==========================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Trigger for documents table
DROP TRIGGER IF EXISTS update_documents_updated_at ON public.documents;
CREATE TRIGGER update_documents_updated_at
    BEFORE UPDATE ON public.documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ==========================================
-- Permissions
-- ==========================================

-- Grant permissions to memory_p user
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO memory_p;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA qdrant_meta TO memory_p;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA tantivy_meta TO memory_p;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA lnx_meta TO memory_p;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA meilisearch_meta TO memory_p;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA memorybank_meta TO memory_p;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA analytics TO memory_p;

-- Grant sequence permissions
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO memory_p;

-- Default privileges for future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO memory_p;
ALTER DEFAULT PRIVILEGES IN SCHEMA qdrant_meta GRANT ALL ON TABLES TO memory_p;
ALTER DEFAULT PRIVILEGES IN SCHEMA tantivy_meta GRANT ALL ON TABLES TO memory_p;
ALTER DEFAULT PRIVILEGES IN SCHEMA lnx_meta GRANT ALL ON TABLES TO memory_p;
ALTER DEFAULT PRIVILEGES IN SCHEMA meilisearch_meta GRANT ALL ON TABLES TO memory_p;
ALTER DEFAULT PRIVILEGES IN SCHEMA memorybank_meta GRANT ALL ON TABLES TO memory_p;
ALTER DEFAULT PRIVILEGES IN SCHEMA analytics GRANT ALL ON TABLES TO memory_p;
