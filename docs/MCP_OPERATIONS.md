# MCP Protocol 2024-11-05 Operational Guide

## Quick Reference

**Status**: ✅ **PRODUCTION-READY**  
**Version**: `2024-11-05`  
**Last Updated**: March 13, 2026  
**Compliance Score**: 100%

---

## Daily Operations

### 1. Verify Compliance Status

```bash
# Check current status
cat mcp_status.json | jq .mcp_compliance_status.overall_status

# Run validation script
./scripts/mcp_compliance_check.sh

# Generate detailed report
./scripts/mcp_compliance_check.sh --report
```

### 2. Monitor CI/CD Workflows

All 24 workflows include MCP validation:

```bash
# Check workflow status
git log --oneline .github/workflows/ | head -10

# Verify protocol version in workflows
grep -r "MCP_PROTOCOL_VERSION" .github/workflows/
```

### 3. Compilation Status

```bash
# Quick check
cargo check --all-features

# Full build
cargo build --release

# Run tests
cargo test --all
```

---

## Making Changes

### ✅ When to Update MCP Version

Update only if the official MCP specification releases a new version:

1. Update `src/mcp/` files
2. Update `Cargo.toml` version reference
3. Update `.github/workflows/` environment variables
4. Update `docs/MCP_COMPLIANCE.md`
5. Run validation script: `./scripts/mcp_compliance_check.sh --auto-fix`

### ❌ Never

- ❌ Use speculative future versions (2025, 2026, etc.)
- ❌ Create custom MCP versions
- ❌ Hardcode version strings without validation
- ❌ Skip MCP validation in CI/CD

---

## CI/CD Pipeline Overview

### Workflow Execution Order

```
1. mcp-validation (ci.yml)
   ↓
2. lint job
   ↓
3. build job
   ↓
4. test job
   ↓
✅ Pipeline Complete
```

### Validation Jobs (By Workflow)

| Workflow | Validation Job | Trigger |
|----------|---|---|
| **ci.yml** | mcp-validation | Early (before lint) |
| **memory-mcp.yml** | mcp-compliance | Early (before build) |
| **autonomous-mcp-ci.yml** | mcp-validation | Early |
| **security.yml** | mcp-validation | Early |
| **docker.yml** | mcp-validation | Early |
| **code-quality.yml** | mcp-validation | Early |
| **auto-merge.yml** | mcp-validation | Before merge |
| **mcp-compliance-check.yml** | mcp-compliance | Scheduled (1 AM UTC) |

### Validation Logic

Each validation job:

1. ✅ Checks `MCP_PROTOCOL_VERSION=2024-11-05` exists
2. ✅ Rejects `2026-*`, `2025-*`, `2023-*` versions
3. ✅ Verifies JSON-RPC 2.0 implementation
4. ✅ Confirms required methods exist
5. ✅ Blocks pipeline if validation fails

---

## Troubleshooting

### Issue: "MCP version not found"

**Solution**:
```bash
# Check source files
grep -r "2024-11-05" src/mcp/

# If missing, run auto-fix
./scripts/mcp_compliance_check.sh --auto-fix

# Recompile
cargo check --all-features
```

### Issue: "Wrong MCP version detected"

**Solution**:
```bash
# Find old versions
grep -r "2026-11-05\|2025-\|2023-" src/

# Auto-fix
./scripts/mcp_compliance_check.sh --auto-fix
```

### Issue: CI/CD workflow failed at validation

**Actions**:
1. Check `mcp_status.json` for compliance score
2. Review workflow logs for specific error
3. Run `./scripts/mcp_compliance_check.sh --report`
4. Look for "old version" or "missing reference" errors

### Issue: "Required method not found"

**Solution**:
```bash
# Verify all methods exist
grep -r "fn initialize\|fn list_resources\|fn read_resource\|fn list_tools\|fn call_tool" src/mcp/

# Ensure implementations are complete
```

---

## Performance Monitoring

### Monitor Validation Speed

All validation jobs should complete in <10 seconds:

```bash
# Check typical execution time
# Look for "Finished in X.XXs" in workflow logs
```

### Check Motor Status

All 9 motors must report MCP 2024-11-05:

```bash
# Check motor references
grep -r "2024-11-05" src/motores/

# Verify all motors are active
grep -c "impl.*SearchEngine" src/motores/*/*.rs
```

---

## Release Procedures

### When Deploying to Production

1. ✅ Ensure all workflows pass
2. ✅ Verify `cargo check --all-features` = PASS
3. ✅ Run `./scripts/mcp_compliance_check.sh --report`
4. ✅ Check `mcp_status.json` shows 100% compliance
5. ✅ Tag release with MCP version in description

### Release Checklist

```bash
# Before Release
✅ All CI/CD workflows passing
✅ cargo test --all passing
✅ No old MCP versions detected
✅ All 5 required methods implemented
✅ mcp_status.json shows 100%

# During Release
✅ Create GitHub release
✅ Tag: v2.0-mcp-2024-11-05
✅ Update CHANGELOG.md

# After Release
✅ Verify production deployment
✅ Monitor first 24 hours
✅ Check compliance dashboard
```

---

## Dashboard & Monitoring

### Compliance Dashboard

View current status: [mcp_status.json](../mcp_status.json)

**Key Metrics**:
- Protocol Version: `2024-11-05` ✅
- Compliance Score: `100%` ✅
- Workflows Compliant: `24/24` ✅
- Motors Active: `9/9` ✅

### GitHub Actions Status

All workflows visible at: `.github/workflows/`

Recent runs: Check GitHub Actions tab in repository

---

## Documentation

### For Users
- [MCP Protocol Compliance](MCP_COMPLIANCE.md) - Full specification
- [API Reference](API_REFERENCE.md) - Endpoint documentation
- [Getting Started](GETTING_STARTED.md) - Quick start guide

### For Developers
- [Architecture Guide](ARCHITECTURE.md) - System design
- [Motor Implementation](MOTOR_ARCHITECTURE.md) - 9-motor guide
- [FFI Integration](../FFI/) - Foreign function interface

### Runbooks
1. **Daily Operations** - This guide
2. **Incident Response** - See troubleshooting section
3. **Release Procedures** - Above section

---

## Support

### Getting Help

1. **Check logs**: GitHub Actions workflow logs
2. **Read guide**: This operational guide
3. **Run diagnostic**: `./scripts/mcp_compliance_check.sh --report`
4. **File issue**: GitHub Issues with compliance report attached

### Reporting Issues

When filing issues, include:
- Output of `./scripts/mcp_compliance_check.sh --report`
- Relevant workflow logs (from GitHub Actions)
- Content of `mcp_status.json`
- Rust version: `rustc --version`

---

## FAQ

### Q: Can I use MCP 2025-11-05?
**A**: No, always use `2024-11-05`. Future versions will not be supported until officially released.

### Q: What if a workflow fails MCP validation?
**A**: The pipeline is blocked. This is intentional to prevent deployment of non-compliant code.

### Q: How often should I run the compliance check?
**A**: Automatically (via GitHub Actions daily). Manually before significant changes.

### Q: Can I modify the validation logic?
**A**: No. The validation ensures protocol compliance. Contact the team to propose changes.

### Q: What's the maximum allowed latency for validation?
**A**: Each validation job has a 10-second timeout. Should complete in <5 seconds.

---

## Automation & Self-Healing

### Automatic Validation
- ✅ Daily compliance check (1 AM UTC)
- ✅ On every push to `src/mcp/**`
- ✅ On every workflow modification
- ✅ Before every auto-merge

### Self-Healing
```bash
# Auto-fix violations
./scripts/mcp_compliance_check.sh --auto-fix

# This will:
# - Correct wrong versions
# - Add missing references
# - Update workflow environments
# - Preserve working code
```

### Backup & Recovery
```bash
# Auto-creates backups
# Original files saved as file.rs.bak

# Restore if needed
cp file.rs.bak file.rs
```

---

## Compliance Score Interpretation

| Score | Status | Action |
|-------|--------|--------|
| 100% | ✅ Excellent | Continue normal operations |
| 99%+ | ✅ Good | Monitor, run validation weekly |
| 95%+ | ⚠️ Fair | Run auto-fix, investigate |
| <95% | ❌ Critical | Stop, run auto-fix, escalate |

---

## Next Steps

- ✅ Review [MCP_COMPLIANCE.md](MCP_COMPLIANCE.md) for technical details
- ✅ Set up monitoring dashboard
- ✅ Schedule regular compliance checks
- ✅ Train team on validation procedures

---

**Version**: 1.0  
**Last Updated**: March 13, 2026  
**Status**: ✅ Production-Ready

For questions or updates, refer to the main [README.md](../README.md).
