# JAR - Intelligent DevOps CLI for MEMORY_P

🔧 **Automated validation, SQL detection, and repair for CI/CD workflows**

## Overview

JAR (Just Auto-Repair) is an intelligent DevOps command-line tool designed specifically for the MEMORY_P project. It combines code validation, SQL analysis, and automatic repair capabilities to streamline development workflows and maintain code quality.

## Features

### 🔍 **Validation**
- ✅ Project structure validation
- 🔍 Dead code detection
- 📝 TODO/FIXME/HACK scanning
- 🔌 MCP endpoint validation

### 🗄️ **SQL Detection**
- 🔎 Deep scan for SQL queries in Rust code
- ✅ Syntax validation using `sqlparser`
- ⚠️ Detection of common issues:
  - SELECT * usage
  - Missing WHERE clauses
  - Potential SQL injection risks
  - Non-parameterized queries

### 🔧 **Auto-Repair**
- 📦 Dependency conflict resolution
- ✨ Automatic code formatting
- 🔧 Clippy warning fixes
- 📋 Schema regeneration (planned)

### 🚀 **CI/CD Integration**
- ✅ GitHub Actions workflows included
- 🤖 Auto-repair on PRs
- 📊 SQL validation pipeline
- 🔒 Security audit checks

## Installation

### Prerequisites
- Rust 1.70+ with Cargo
- Git

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Rigohl/MEMORY_P.git
cd MEMORY_P

# Build the CLI
cargo build --release --bin jar

# The binary will be at target/release/jar
```

### Add to PATH (Optional)

```bash
# Linux/macOS
sudo cp target/release/jar /usr/local/bin/

# Or add to your shell profile
export PATH="$PATH:$(pwd)/target/release"
```

## Usage

### Basic Commands

```bash
# Show help
jar --help

# Show version
jar --version

# Enable verbose output (works with any command)
jar --verbose <command>
```

### Validate Project

```bash
# Basic validation
jar validate

# Full validation with all checks
jar validate --scan-todos --check-dead-code --validate-mcp

# Validate specific directory
jar validate --path /path/to/project
```

**Example Output:**
```
🔍 Validating project at: .
  📝 Scanning for TODOs/FIXMEs...

=== Validation Report ===

📝 TODOs/FIXMEs found:
  Todo (78):
    ./src/main.rs:42 - Implement caching
    ./src/analyzer.rs:15 - Add benchmarks
    ...
  
✅ Validation passed!
```

### Detect SQL Queries

```bash
# Scan for SQL queries
jar detect-sql --path .

# With syntax validation
jar detect-sql --path . --validate-syntax

# Full analysis with issue detection
jar detect-sql --path . --validate-syntax --detect-issues
```

**Example Output:**
```
🔍 Scanning for SQL queries in: .

=== SQL Analysis Report ===

📊 SQL queries found: 15
  • Select: 10
  • Insert: 3
  • Update: 2

⚠️ Potential Issues:
  Warnings (2):
    src/db.rs:45 - SELECT * detected
    src/queries.rs:89 - Non-parameterized query
    
✅ No critical issues!
```

### Auto-Repair

```bash
# Format code
jar repair --format

# Fix dependencies
jar repair --fix-deps

# Fix clippy warnings
jar repair --fix-clippy

# Dry run (show what would be fixed)
jar repair --format --fix-deps --dry-run

# Full repair (default: format + fix-deps)
jar repair
```

**Example Output:**
```
🔧 Starting auto-repair for: .

  ✨ Formatting code...
  📦 Fixing Rust dependencies...

=== Repair Report ===

🔧 Actions taken: 2 successful, 0 failed

✅ cargo fmt --all
  Code formatted successfully

✅ cargo update
  Updating crates.io index
  Updated 5 packages
```

### CI/CD Check

```bash
# Check workflow health
jar ci-check

# Check specific directory
jar ci-check --path .github/workflows
```

## GitHub Actions Integration

Three workflows are automatically configured:

### 1. **CI Workflow** (`.github/workflows/ci.yml`)
Runs on every push and PR:
- ✅ Project validation
- 🧪 Build and test
- 🔒 Security audit
- 📊 SQL analysis

### 2. **Auto-Repair Workflow** (`.github/workflows/auto-repair.yml`)
Automatically fixes issues on PRs:
- ✨ Code formatting
- 📦 Dependency updates
- 🤖 Auto-commits fixes

### 3. **SQL Check Workflow** (`.github/workflows/sql-check.yml`)
Validates SQL on code changes:
- 🔎 Detects all SQL queries
- ✅ Validates syntax
- ⚠️ Reports issues

## Configuration

JAR uses sensible defaults but can be customized:

```toml
# Add to Cargo.toml for custom settings
[package.metadata.jar]
max_todos = 100
ignore_patterns = ["target/", "node_modules/"]
sql_dialects = ["PostgreSQL", "MySQL"]
```

## Development

### Adding New Validators

```rust
// src/cli/validators.rs
pub fn my_custom_validator(path: &Path) -> Result<Vec<Issue>> {
    // Your validation logic
}
```

### Adding New Repair Actions

```rust
// src/cli/auto_repair.rs
pub fn my_repair_action(path: &Path) -> Result<RepairAction> {
    // Your repair logic
}
```

## Architecture

```
jar (binary)
├── cli/
│   ├── commands.rs      # Command definitions (Clap)
│   ├── validators.rs    # Project validation
│   ├── sql_detector.rs  # SQL analysis
│   └── auto_repair.rs   # Auto-repair logic
└── bin/
    └── jar.rs           # Main entry point
```

## Roadmap

- [x] Basic validation
- [x] SQL detection
- [x] Auto-repair (format, deps)
- [x] GitHub Actions integration
- [ ] SQL schema regeneration
- [ ] Docker health checks
- [ ] Kubernetes validation
- [ ] Custom rule engine
- [ ] Plugin system
- [ ] TUI interface

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Add tests for new features
4. Run `jar validate` before submitting
5. Submit a PR

## License

MIT License - See LICENSE file for details

## Related Projects

- [MEMORY_P](https://github.com/Rigohl/MEMORY_P) - Main project
- [nuclear-crawler-hybrid](https://github.com/Rigohl/nuclear-crawler-hybrid) - Web scraping integration

## Support

- 📖 [Documentation](https://github.com/Rigohl/MEMORY_P/blob/main/docs/)
- 🐛 [Issue Tracker](https://github.com/Rigohl/MEMORY_P/issues)
- 💬 [Discussions](https://github.com/Rigohl/MEMORY_P/discussions)

---

**Built with ❤️ for the MEMORY_P ecosystem**
