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
| `analyze`   | 🔬 Massively parallel code analysis with security       |
| `repair`    | 🛠️ Auto-fix formatting, imports, and code style         |
| `edit`      | ✏️ Atomic bulk editing with regex support               |
| `workflow`  | 🌊 Pipeline orchestration with auto-evolution           |
| `simulate`  | 🌀 5-phase optimization simulations (25K+ sims)         |

## 📦 Tech Stack

- **Parallelism**: `rayon` 1.8 with work-stealing scheduler
- **Memory**: `mimalloc` 0.1.48 allocator + `memmap2` zero-copy I/O
- **Caching**: `scc` 2.1 lock-free HashMap
- **Serialization**: `rkyv` 0.7.42 zero-copy deserialization
- **HTTP**: `axum` 0.7 + `tokio` async runtime
- **Protocol**: MCP 2024-11-05 with JSON-RPC 2.0

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
│   └── analyzer.rs          # Code analysis
├── JULIA_BRAIN/             # Julia orchestrator
├── PAYLOAD_BANK/            # Workflows and analysis data
├── docs/                    # Documentation
├── AGENTS.md                # GitHub Copilot Agents guide
└── SKILLS.md                # GitHub Copilot Skills guide
```

## 📚 Documentation

- **[AGENTS.md](AGENTS.md)** - Guía completa de GitHub Copilot Agents
- **[SKILLS.md](SKILLS.md)** - Documentación de GitHub Copilot Agent Skills
- **[.github/README.md](.github/README.md)** - Agents & Skills personalizados del proyecto
- **[docs/](docs/)** - Documentación técnica:
  - [Tutorial de Inicio](docs/TUTORIAL_START.md)
  - [Guía de Reparación](docs/HOWTO_REPAIR.md)
  - [Referencia de Herramientas](docs/REFERENCE_TOOLS.md)

## 📄 License

MIT License - Built with 🦀 Rust
