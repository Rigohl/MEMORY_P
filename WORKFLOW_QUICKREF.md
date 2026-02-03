# Quick Reference: Auto-Push & Auto-Management Workflows

## 🚀 Quick Start

### Enable Auto-Push on a PR

```bash
# 1. Create PR to authorized branch
git checkout -b feature/my-feature
git push origin feature/my-feature

# 2. Create PR targeting authorized branch (develop, staging, etc.)
gh pr create --base develop --title "My Feature"

# 3. Add auto-push label
gh pr edit {PR_NUMBER} --add-label "auto-push"

# 4. Watch it auto-merge! 🎉
```

### Manual Workflow Triggers

```bash
# Trigger auto-recovery
gh workflow run auto-recovery.yml -f recovery_mode=auto

# Run deep validation
gh workflow run nuclear-crawler-validation.yml -f validation_level=deep

# Run full tests
gh workflow run dynamic-tests.yml -f test_scope=full

# Run forensic scan
gh workflow run recurring-scan.yml -f scan_type=forensic
```

### Check Workflow Status

```bash
# List recent runs
gh run list --limit 10

# View specific run
gh run view {RUN_ID}

# View logs
gh run view {RUN_ID} --log

# Watch live
gh run watch
```

## 📋 Workflow Cheat Sheet

| Workflow | Trigger | Purpose | Duration |
|----------|---------|---------|----------|
| `auto-push` | PR + label | Auto-merge PRs | ~10 min |
| `auto-recovery` | Failures | Self-heal | ~15 min |
| `nuclear-crawler` | Push to src/ | Validate crawler | ~20 min |
| `dynamic-tests` | Any push | Adaptive tests | ~15 min |
| `recurring-scan` | Daily/Weekly | Code health | ~30 min |

## 🎯 Common Scenarios

### Scenario 1: Emergency Hotfix

```bash
# 1. Create hotfix branch
git checkout -b hotfix/critical-fix

# 2. Make changes
git commit -am "Fix critical bug"

# 3. Push and auto-merge
git push origin hotfix/critical-fix
gh pr create --base main --label auto-push

# Auto-push workflow will:
# - Validate
# - Test
# - Auto-merge in ~10 minutes
```

### Scenario 2: Build Failing Repeatedly

```bash
# Auto-recovery will automatically:
# 1. Detect repeated failures
# 2. Clear cache
# 3. Rebuild from scratch
# 4. Adjust configuration

# If still fails, check:
gh run list --workflow=auto-recovery.yml
gh run view {RUN_ID} --log

# Manual intervention:
gh workflow run auto-recovery.yml -f recovery_mode=aggressive
```

### Scenario 3: Check System Health

```bash
# View auto-manager status
cargo run --release -- --health-check

# Or check workflow outputs
gh run list --workflow=recurring-scan.yml --limit 1
gh run view {LATEST_RUN_ID}
```

## 🔍 Debugging Tips

### Auto-Push Not Working?

```bash
# Check 1: Is branch authorized?
# Authorized: develop, staging, hotfix/*, feature/auto-*, copilot/*

# Check 2: Has auto-push label?
gh pr view {PR_NUMBER} --json labels

# Check 3: Are checks passing?
gh pr checks {PR_NUMBER}

# Check 4: View workflow logs
gh run list --workflow=auto-push.yml
gh run view {RUN_ID} --log
```

### Tests Failing in CI but Pass Locally?

```bash
# Dynamic tests will adapt automatically
# But you can force specific strategy:

# Run same tests as CI locally
cargo test --all-features --verbose

# Check for flaky tests
for i in {1..3}; do cargo test; done

# Force test isolation
cargo test -- --test-threads=1
```

### Workflow Takes Too Long?

```bash
# Use minimal test scope for quick feedback
gh workflow run dynamic-tests.yml -f test_scope=minimal

# Check cache is working
# Look for "Cache restored from key" in logs

# Optimize in code:
# - Use `|| true` for non-critical steps
# - Reduce `timeout-minutes`
# - Enable `fail-fast: false` for matrix
```

## 📊 Monitoring Commands

### View All Workflows

```bash
# List all workflows
gh workflow list

# View specific workflow runs
gh run list --workflow=auto-push.yml --limit 5

# Watch current run
gh run watch
```

### Check System Metrics

```bash
# From auto-manager
cargo run -- --export-metrics

# From recurring scan
gh run list --workflow=recurring-scan.yml --limit 1
gh run view {RUN_ID} | grep "Metrics"
```

### View Issues Created by Workflows

```bash
# Auto-push failures
gh issue list --label auto-push-failed

# Recovery reports
gh issue list --label auto-recovery

# Scan reports
gh issue list --label recurring-scan
```

## 🛠️ Configuration

### Authorized Branches for Auto-Push

Edit in `.github/workflows/auto-push.yml`:

```yaml
AUTHORIZED_BRANCHES="develop staging hotfix/* feature/auto-* copilot/*"
```

### Change Scan Schedule

Edit in `.github/workflows/recurring-scan.yml`:

```yaml
on:
  schedule:
    - cron: '0 3 * * *'  # Daily at 3 AM
    - cron: '0 0 * * 0'  # Weekly on Sunday
```

### Adjust Recovery Timeouts

Edit in `.github/workflows/auto-recovery.yml`:

```yaml
env:
  MAX_RETRY_ATTEMPTS: 3  # Change this
```

## 🔐 Security Notes

### What's Automatic?

✅ Auto-push to authorized branches  
✅ Auto-recovery from failures  
✅ Auto-validation of code  
✅ Auto-scanning for vulnerabilities  

### What Requires Manual Approval?

❌ Pushes to `main` (use PR)  
❌ Changes to workflows (review required)  
❌ Security critical changes  
❌ Breaking changes  

### Permissions Used

- `contents: write` - For auto-push
- `pull-requests: write` - For auto-approval
- `issues: write` - For tracking issues
- `actions: write` - For re-running workflows

## 📚 Additional Resources

### Documentation

- [Workflow README](.github/workflows/README.md) - Complete workflow docs
- [Integration Guide](docs/WORKFLOW_INTEGRATION.md) - System integration
- [Auto-Manager Source](src/auto_manager.rs) - Core auto-management code

### GitHub Actions Docs

- [Workflow Syntax](https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions)
- [GitHub Script](https://github.com/actions/github-script)
- [Actions Toolkit](https://github.com/actions/toolkit)

### Troubleshooting

1. **Workflow not triggering**: Check branch patterns and triggers
2. **Permission denied**: Verify workflow has correct permissions
3. **Timeout**: Increase timeout in workflow or optimize code
4. **Flaky tests**: Enable test isolation or use retry logic

## 💡 Tips & Tricks

### Speed Up Workflows

```bash
# Use cache effectively
- uses: actions/cache@v4
  with:
    path: ~/.cargo
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

# Run jobs in parallel
jobs:
  job1:
    runs-on: ubuntu-latest
  job2:
    runs-on: ubuntu-latest
  # Both run simultaneously

# Skip unnecessary steps
- name: Optional Step
  if: github.event_name == 'push'
  run: ...
```

### Debug Workflows Locally

```bash
# Install act (GitHub Actions local runner)
curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# Run workflow locally
act push

# Run specific job
act -j job_name
```

### Custom Notifications

Add to workflows:

```yaml
- name: Notify on Slack
  if: failure()
  run: |
    curl -X POST ${{ secrets.SLACK_WEBHOOK }} \
      -d '{"text":"Workflow failed: ${{ github.workflow }}"}'
```

## ✅ Pre-Deployment Checklist

Before using in production:

- [ ] Review all workflow files
- [ ] Set up branch protection rules
- [ ] Test each workflow manually
- [ ] Verify permissions are correct
- [ ] Add team members to notifications
- [ ] Document any custom changes
- [ ] Monitor first few runs closely

---

**Need Help?**

- 📖 Read full docs: `.github/workflows/README.md`
- 🐛 Report issues: `gh issue create`
- 💬 Ask team: Check project discussions

**Last Updated**: Febrero 2026  
**Version**: 1.0.0
