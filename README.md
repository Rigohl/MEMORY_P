# ⚡ MEMORY_P

Nuclear MCP Toolkit for Massive Parallel Processing

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![MCP](https://img.shields.io/badge/MCP-2025.2-blue?style=for-the-badge)
![Parallel](https://img.shields.io/badge/Parallel-Rayon-orange?style=for-the-badge)

High-performance Model Context Protocol server built in pure Rust.
Compatible with Cursor, Windsurf, Claude Desktop, and VS Code.

---

## 🚀 Features

| Tool        | Description                                             |
| ----------- | ------------------------------------------------------- |
| `analyze`   | 🔬 Análisis masivo: deep/quick/overview/**optimize** (Amdahl's Law) |
| `repair`    | 🛠️ Auto-fix: imports duplicados, formato, espacios     |
| `edit`      | ✏️ Edición atómica masiva: replace/regex/append/delete |
| `workflow`  | 🌊 Pipeline: Scan→Filter→Analyze→Edit→Repair→Evolve    |
| `simulate`  | 🌀 Mega simulaciones 3-phase: 15K/150K/500K iterations |

## 📦 Tech Stack

- **Parallelism**: `rayon 1.11` with work-stealing
- **Memory**: `mimalloc` allocator + `memmap2` zero-copy I/O
- **Caching**: `scc 2.4` lock-free HashMap
- **Serialization**: `rkyv 0.8` zero-copy deserialization
- **HTTP**: `axum 0.7` + `tokio 1.49`
- **MCP**: `mcp-sdk-rs 0.3` + `mcpkit-core 0.5`

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

Add to your `mcp.json`:

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

## 📊 Benchmarks

| Phase                | Simulations | Improvement |
| -------------------- | ----------- | ----------- |
| Module Optimization  | 65K         | 89.8%       |
| Parallelism Tuning   | 200K        | 1345.6%     |
| Ecosystem Analysis   | 550K        | Optimal     |

## 📁 Project Structure

```text
MEMORY_P/
├── src/
│   ├── main.rs              # Entry point + HTTP/stdio modes
│   ├── mcp_api.rs           # MCP handlers (5 tools)
│   ├── parallel_engine.rs   # Rayon-powered processing
│   ├── mega_simulator.rs    # 3-phase simulation engine
│   ├── analyzer.rs          # Code analysis with security
│   ├── workspace.rs         # Workspace management
│   └── config.rs            # Configuration
├── PAYLOAD_BANK/            # Workflows and analysis data
└── docs/                    # Documentation
```

## 📄 License

MIT License - Built with 🦀 Rust
