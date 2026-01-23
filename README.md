# ⚡ MEMORY_P v2.0

Nuclear MCP Toolkit with 9 Search Engines

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![MCP](https://img.shields.io/badge/MCP-2025.2-blue?style=for-the-badge)
![Parallel](https://img.shields.io/badge/Parallel-Rayon-orange?style=for-the-badge)
![Search](https://img.shields.io/badge/Search-9_Motors-green?style=for-the-badge)

High-performance Model Context Protocol server with **9 specialized search engines** built in pure Rust.
Compatible with Cursor, Windsurf, Claude Desktop, and VS Code.

---

## 🚀 Features

### Core MCP Tools
| Tool        | Description                                             |
| ----------- | ------------------------------------------------------- |
| `analyze`   | 🔬 Massively parallel code analysis with security       |
| `repair`    | 🛠️ Auto-fix formatting, imports, and code style         |
| `edit`      | ✏️ Atomic bulk editing with regex support               |
| `workflow`  | 🌊 Pipeline orchestration with auto-evolution           |
| `simulate`  | 🌀 5-phase optimization simulations (25K+ sims)         |

### 🎯 9 Search Engines (NEW in v2.0)

#### Vector Search (3 engines)
- **Qdrant** - Semantic similarity (<100ms, <1M vectors)
- **FAISS-GPU** - Billions-scale GPU acceleration (<50ms)
- **SCANN** - Google trillion-scale learned indexing (<200ms)

#### Text Search (4 engines)
- **Tantivy** - Single-node BM25 champion (<10ms)
- **LNX** - Distributed Raft consensus (<150ms)
- **Toshi** - Experimental distributed (<300ms)
- **MeiliSearch** - Typo-tolerant user-friendly (<80ms)

#### Specialized (2 engines)
- **Julia NLP** - Mathematical text analysis (<500ms)
- **MemoryBank Ultra** - Multi-language FFI (Rust+Zig+Julia+Python+Mojo+Pony)

## 📦 Tech Stack

### Core Infrastructure
- **Parallelism**: `rayon` 1.11 with work-stealing scheduler
- **Memory**: `mimalloc` 0.1.48 allocator + `memmap2` zero-copy I/O
- **Caching**: `scc` 2.4 lock-free HashMap
- **HTTP**: `axum` 0.7 + `tokio` async runtime
- **Protocol**: MCP 2024-11-05 with JSON-RPC 2.0

### Search Architecture (NEW)
- **9 Specialized Engines**: Complete separation per motor
- **Intelligent Routing**: AI-powered query routing
- **Health Monitoring**: Real-time engine health checks
- **Multi-Engine Fusion**: Coordinated cross-engine searches
- **Database Isolation**: PostgreSQL schema per motor
- **Analytics**: ClickHouse for performance metrics

## 🛠️ Installation

```bash
# Clone
git clone https://github.com/Rigohl/MEMORY_P.git
cd MEMORY_P

# Build release
cargo build --release

# Run server (port 4040)
./target/release/memory_p
```

## ⚙️ MCP Configuration

### For Cursor / Windsurf

Add to your MCP settings or `mcp.json`:

```json
{
  "mcpServers": {
    "memory_p": {
      "url": "http://127.0.0.1:4040/mcp",
      "transport": "http"
    }
  }
}
```

### For Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "memory_p": {
      "command": "cargo",
      "args": ["run", "--release", "--", "--stdio"],
      "cwd": "/path/to/MEMORY_P"
    }
  }
}
```

## 📊 Benchmarks

| Phase                | Simulations | Improvement |
| -------------------- | ----------- | ----------- |
| Module Optimization  | 65K         | 89.8%       |
| Parallelism Tuning   | 200K        | 1345.6%     |
| Ecosystem Analysis   | 550K        | Optimal     |
| Repair Operations    | 10K         | Variable    |
| Edit Operations      | 10K         | Variable    |

## 📁 Project Structure

```text
MEMORY_P/
├── src/
│   ├── main.rs              # Entry point
│   ├── mcp_api.rs           # MCP handlers (5 tools)
│   ├── parallel_engine.rs   # Rayon-powered processing
│   ├── mega_simulator.rs    # 3-phase simulation engine
│   ├── analyzer.rs          # Code analysis
│   └── motores/             # 🆕 9 Search Engines
│       ├── core/            # Traits, types, routing, health
│       ├── vector_search/   # Qdrant, FAISS, SCANN
│       ├── text_search/     # Tantivy, LNX, Toshi, MeiliSearch
│       ├── specialized/     # Julia NLP, MemoryBank
│       ├── hybrid/          # Multi-engine coordination
│       └── factory/         # Engine creation
├── database/                # 🆕 SQL schemas
│   └── schemas/
│       ├── postgresql_motors.sql    # 9 motor schemas
│       └── clickhouse_analytics.sql # Performance analytics
├── .github/
│   ├── agents/              # Custom Copilot agents
│   ├── skills/              # 🆕 Motor-specific skills
│   └── copilot-instructions.md  # 🆕 Development guidelines
├── JULIA_BRAIN/             # Julia orchestrator
├── PAYLOAD_BANK/            # Workflows and analysis data
├── docs/
│   ├── NINE_MOTORS_GUIDE.md # 🆕 Complete motors guide
│   ├── TUTORIAL_START.md
│   ├── HOWTO_REPAIR.md
│   └── REFERENCE_TOOLS.md
├── AGENTS.md                # GitHub Copilot Agents guide
└── SKILLS.md                # GitHub Copilot Skills guide
```

## 📚 Documentation

- **[NINE_MOTORS_GUIDE.md](docs/NINE_MOTORS_GUIDE.md)** - 🆕 Complete guide to 9 search engines
- **[AGENTS.md](AGENTS.md)** - Guía completa de GitHub Copilot Agents
- **[SKILLS.md](SKILLS.md)** - Documentación de GitHub Copilot Agent Skills
- **[.github/README.md](.github/README.md)** - Agents & Skills personalizados del proyecto
- **[.github/copilot-instructions.md](.github/copilot-instructions.md)** - 🆕 Development guidelines
- **[docs/](docs/)** - Documentación técnica:
  - [Tutorial de Inicio](docs/TUTORIAL_START.md)
  - [Guía de Reparación](docs/HOWTO_REPAIR.md)
  - [Referencia de Herramientas](docs/REFERENCE_TOOLS.md)

## 🔍 Quick Start: Search Engines

```rust
use memory_p::motores::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create fusion engine for multi-motor search
    let fusion = Arc::new(FusionEngine::new());
    
    // Initialize engines
    for name in ["qdrant", "tantivy", "meilisearch"] {
        let config = /* create config based on engine name */ ;
        let engine = EngineFactory::create_engine(name, config)?;
        fusion.register_engine(name.to_string(), engine).await;
    }
    
    // Intelligent multi-engine search
    let query = SearchQuery {
        text: "machine learning".to_string(),
        vector: Some(embedding),
        query_type: QueryType::Hybrid,
        limit: 10,
        // ...
    };
    
    let results = fusion.search_multi(&query).await?;
    println!("Found {} results", results.len());
    
    Ok(())
}
```

See [NINE_MOTORS_GUIDE.md](docs/NINE_MOTORS_GUIDE.md) for complete documentation.

## 📄 License

MIT License - Built with 🦀 Rust
