# CI/CD DevOps Improvements - GitHub Actions + GitLab CI/CD
**Date**: 2026-03-29  
**Version**: v2.0 Enterprise-Grade  
**Status**: Implementation Guide

---

## EXECUTIVE SUMMARY

Best practices from GitHub & GitLab docs consolidated for MEMORY_P:
- ✅ GitHub Actions: Workflow optimization, caching, matrix builds, deployments
- ✅ GitLab CI/CD: Variables, pipeline inputs, advanced rules, security
- ✅ Unified DevOps: Single source of truth for both platforms

---

## PART 1: GITHUB ACTIONS IMPROVEMENTS

### 1.1 Current State
**File**: `.github/workflows/ci.yml`

**Current Jobs**:
1. `check` - Format & Clippy (✅ good)
2. `test` - Unit tests (✅ good)
3. Building blocks present but not optimized

**Issues to Fix**:
- [ ] No matrix builds for multi-platform testing
- [ ] Runner selection not optimized
- [ ] Dependency caching could be better
- [ ] No environment-based deployments configured
- [ ] Missing workflow_dispatch inputs for flexibility

### 1.2 Recommended Improvements

#### A. Add Matrix Builds
```yaml
jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]
        exclude:
          # Nightly on Windows causes issues
          - os: windows-latest
            rust: nightly
```

**Why**: Tests on all platforms where your binary will run. Catches platform-specific bugs early.

#### B. Improve Caching Strategy
```yaml
- name: Restore cache
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: |
      . -> target
      ffi -> target_ffi
    cache-targets: true  # Cache compiled artifacts
    cache-directories: |
      ~/.cargo/registry
      ~/.cargo/git
```

**Why**: Rust-cache action is specifically optimized for Cargo. Reduces CI time by 40-60%.

#### C. Add Workflow Dispatch Inputs
```yaml
on:
  workflow_dispatch:
    inputs:
      build_target:
        type: choice
        description: 'Build target'
        options:
          - x86_64
          - aarch64
          - all
        default: x86_64
      
      enable_benchmarks:
        type: boolean
        description: 'Run benchmarks'
        default: false
      
      deploy_env:
        type: choice
        description: 'Deploy to environment'
        options:
          - none
          - staging
          - production
        default: none
```

**Why**: Allows manual control without changing code. Matches GitLab inputs approach.

#### D. Add Deployment Job
```yaml
deploy:
  name: Deploy
  needs: [check, test, build]
  runs-on: ubuntu-latest
  if: github.ref == 'refs/heads/main' && github.event_name == 'push'
  environment:
    name: ${{ needs.build.outputs.deploy_env }}
    url: https://memory-p.example.com
  concurrency:
    group: deployment
    cancel-in-progress: false  # Don't cancel in-progress deployments
  steps:
    - name: Deploy to environment
      run: |
        echo "Deploying to ${{ environment.name }}"
        # Add actual deployment commands
```

**Why**: Environments provide:
- Required approvals
- Secrets management
- Concurrency control
- Deployment tracking

#### E. Add Code Quality Jobs
```yaml
security:
  name: Security Audit
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: cargo audit
    - run: cargo deny check
    
coverage:
  name: Code Coverage
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: taiki-e/install-action@cargo-tarpaulin
    - run: cargo tarpaulin --out Xml --timeout 300
    - uses: codecov/codecov-action@v3
```

**Why**: Automated security + coverage tracking prevents regressions.

### 1.3 GitHub Actions Secrets Management
Use environment-scoped secrets:

```yaml
env:
  # Non-secret: Use in all environments
  CARGO_INCREMENTAL: 1
  RUST_BACKTRACE: full

jobs:
  deploy:
    environment: production
    env:
      # Secret: Only available in production deploys
      DEPLOY_KEY: ${{ secrets.DEPLOY_KEY }}  # Production-scoped secret
      API_TOKEN: ${{ secrets.API_TOKEN }}    # Production-scoped secret
```

**Best Practice**: Secrets defined at environment level are only available when deploying to that environment.

---

## PART 2: GITLAB CI/CD IMPROVEMENTS

### 2.1 Current State
**File**: `.gitlab-ci.yml`

**Current Stages**:
- ✅ lint, test, ffi, build, security, analyze, benchmark, repair, cleanup, pages, deploy

**Issues to Fix**:
- [ ] Variables not fully leveraged
- [ ] Conditional execution could be more precise
- [ ] Artifact retention policies missing
- [ ] Performance optimization (parallel jobs)
- [ ] Runner selection not optimized

### 2.2 Recommended Improvements

#### A. Optimized Variable Strategy
```yaml
variables:
  # Global defaults
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  CARGO_INCREMENTAL: 1
  RUSTFLAGS: "-C target-cpu=native"
  FF_USE_FASTZIP: "true"
  TRANSFER_METER_FREQUENCY: "5s"
  
workflow:
  rules:
    # Protected branches: strict mode
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH && $CI_COMMIT_REF_PROTECTED == "true"
      variables:
        SAST_STRICT_MODE: "true"
        RUST_LTO: "fat"  # Slower, better optimization
        ENABLE_BENCHMARKS: "true"
    
    # MRs: fast mode
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
      variables:
        RUST_LTO: "thin"  # Faster compilation
        ENABLE_BENCHMARKS: "false"
        SAST_STRICT_MODE: "false"
    
    # Pull from origin: normal mode
    - when: always
      variables:
        RUST_LTO: "thin"
        SAST_STRICT_MODE: "false"
```

**Why**: Different strategies for different branches saves time on MRs, strict on main.

#### B. Fine-Grained Job Dependencies
```yaml
build:linux-x86_64:
  stage: build
  needs:
    - ffi:check-deps
    - ffi:build-zig
    - lint:format
    - lint:clippy
  rules:
    - if: $BUILD_TARGET =~ /x86_64|all/
  script:
    - cargo build --release --target x86_64-unknown-linux-gnu

build:linux-aarch64:
  stage: build
  needs:
    - ffi:check-deps
    - ffi:build-zig
    - lint:format
    - lint:clippy
  rules:
    - if: $BUILD_TARGET =~ /aarch64|all/
  script:
    - cargo build --release --target aarch64-unknown-linux-gnu
```

**Why**: Only run what's needed. `needs:` specifies exact dependencies (not stage order).

#### C. Artifact Management
```yaml
build:linux-x86_64:
  artifacts:
    paths:
      - target/x86_64-unknown-linux-gnu/release/memory_p
      - target/x86_64-unknown-linux-gnu/release/mcp_server
    expire_in: 30 days
    name: "$CI_COMMIT_SHORT_SHA-x86_64"
  
  # Report metrics to GitLab
  reports:
    dotenv: build.env
    junit: target/test-results.xml
    coverage_report:
      coverage_format: cobertura
      path: coverage.xml
```

**Why**: Artifacts with expiration save storage. Reports integrate with GitLab UI.

#### C. Parallel Job Optimization
```yaml
test:unit:
  parallel:
    matrix:
      - TEST_SUITE: ["motores::vector", "motores::text", "core_traits"]
  script:
    - cargo test --lib $TEST_SUITE --no-default-features

# Runs 3 parallel jobs: test:unit[1], test:unit[2], test:unit[3]
```

**Why**: Run tests in parallel. Each gets its own runner - 3x faster.

### 2.3 GitLab CI/CD Security Best Practices

#### A. Protected Variables
```yaml
# In GitLab UI: Settings > CI/CD > Variables
# Set "Protect variable" for sensitive data

variables:
  # Non-sensitive: visible, unprotected
  REGISTRY_URL: registry.example.com
  
  # Protected: masked, only on protected branches
  # DOCKER_REGISTRY_TOKEN: (set in UI, marked protected + masked)
```

**Why**: Protected variables prevent accidental leaks in MRs.

#### B. File Type Variables
```yaml
# In GitLab UI: Create variable with type=File
# Example: SSH key, Docker config

jobs:
  deploy:
    script:
      # Variable contains path to file, not the contents
      - ssh-keyscan -t ed25519 $DEPLOY_HOST >> ~/.ssh/known_hosts
      - scp -i $DEPLOY_KEY /local/file $DEPLOY_HOST:/remote/
```

**Why**: File-type variables prevent secrets from appearing in logs.

#### C. External Secrets Providers
```yaml
# When credentials are highly sensitive, use external provider:
# - HashiCorp Vault
# - Azure Key Vault
# - AWS Secrets Manager

deploy:
  script:
    - |
      TOKEN=$(curl -s -X GET https://vault.example.com/v1/secret/data/deploy \
        -H "X-Vault-Token: $VAULT_TOKEN" | jq .data.data.token)
      curl -X POST -H "Authorization: Bearer $TOKEN" ...
```

**Why**: Secrets never stored in GitLab. Audit trail in external system.

---

## PART 3: UNIFIED CI/CD PATTERNS

### 3.1 Equivalent Workflows

| Concept | GitHub Actions | GitLab CI/CD |
|---------|---|---|
| **Stages** | Jobs that run in sequence (via `needs:`) | Explicit `stages:` keyword |
| **Matrix Builds** | `strategy.matrix` | `parallel.matrix` |
| **Inputs** | `workflow_dispatch.inputs` | `spec.inputs` in pipeline |
| **Caching** | `actions/cache@v4` | `cache:` keyword |
| **Secrets** | `secrets.NAME` (environment-scoped) | Variables UI (protected + masked) |
| **Artifacts** | `artifacts:` with retention | `artifacts:` with `expire_in` |
| **Concurrency** | `concurrency:` group | `resource_group:` (premium) |
| **Conditionals** | `if:` conditions | `rules:` with expressions |
| **Approvals** | Environment protection rules | Manual job `when: manual` + protection |

### 3.2 Best Practices (Both Platforms)

#### Rule 1: Cache Aggressively
```
GitHub:
  - Use Swatinem/rust-cache for Rust
  - Key strategy: ${{ hashFiles('Cargo.lock') }}

GitLab:
  - Use native cache keyword
  - Include both target/ and ~/.cargo
```
**Impact**: 40-60% time savings

#### Rule 2: Matrix Build for Coverage
```
Test on:
  - Multiple OS (ubuntu, macos, windows)
  - Multiple Rust versions (stable, nightly)
  - Multiple feature combinations (default, minimal, all)

Example: 2 OS × 2 versions × 3 features = 12 parallel jobs
```
**Impact**: Catches platform bugs early

#### Rule 3: Fail Fast
```
GitHub:
  - Run lint/check first (fast)
  - Test after (slower)
  - Build last (slowest)
  
GitLab:
  - Use `needs:` to skip unnecessary stages
  - Lint stage blocks all others
```
**Impact**: 15-20 minute feedback loops instead of 45 minutes

#### Rule 4: Artifact Retention
```
GitHub:
  - Artifacts auto-delete after default retention
  - Set via repository settings OR workflow `retention-days`

GitLab:
  - Use `expire_in: 7 days` per job
  - Large artifacts (30 days) only for releases
```
**Impact**: Storage costs controlled

#### Rule 5: Security
```
Both platforms:
  ✅ Secrets in environment/UI, never in code
  ✅ Mask sensitive variables in logs
  ✅ Use file-type variables for credentials
  ✅ Restrict secrets to protected branches
  ✅ Regular `cargo audit` in CI
  ✅ SAST scanning enabled
```

---

## PART 4: IMPLEMENTATION ROADMAP

### Phase    4A: GitHub Actions (This Week)
```yaml
Tasks:
  1. [ ] Add matrix builds for OS/Rust versions
  2. [ ] Replace manual cache with Swatinem/rust-cache
  3. [ ] Add security audit job
  4. [ ] Add deployment job with environments
  5. [ ] Test workflow_dispatch inputs
  
Timeline: 30 minutes
Files: .github/workflows/ci.yml
```

### Phase 4B: GitLab CI/CD (This Week)
```yaml
Tasks:
  1. [ ] Optimize variables (per-stage)
  2. [ ] Add parallel matrix for tests
  3. [ ] Improve artifact retention policies
  4. [ ] Add security scanning
  5. [ ] Test protected variable setup
  
Timeline: 30 minutes
Files: .gitlab-ci.yml
```

### Phase 4C: Monitoring & Documentation (Next Week)
```yaml
Tasks:
  1. [ ] Enable GitHub deployment tracking
  2. [ ] Set up GitLab Pages reports
  3. [ ] Create CI/CD runbook
  4. [ ] Document secrets management
  5. [ ] Set up alerts for failures
  
Timeline: 1 hour
```

---

## PART 5: QUICK REFERENCE

### GitHub Actions Syntax
```yaml
# Trigger from main and develop
on:
  push:
    branches: [main, develop]
  workflow_dispatch:

# Define matrix
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable, nightly]

# Cache
- uses: Swatinem/rust-cache@v2

# Conditional
- if: github.ref == 'refs/heads/main'

# Artifact
- uses: actions/upload-artifact@v3
  with:
    name: binaries-${{ matrix.os }}
    path: target/release/memory_p
```

### GitLab CI/CD Syntax
```yaml
# Trigger rules
rules:
  - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
  - if: $CI_PIPELINE_SOURCE == "merge_request_event"

# Define matrix
parallel:
  matrix:
    - OS: [linux, macos]
      RUST: [stable, nightly]

# Cache
cache:
  key: ${{ CI_COMMIT_REF_SLUG }}-${{ hashFiles('Cargo.lock') }}
  paths:
    - target/
    - .cargo/

# Artifact
artifacts:
  paths:
    - target/release/memory_p
  expire_in: 30 days
```

---

## PART 6: MONITORING & ALERTS

### GitHub: Enable Status Checks
```yaml
# In repository settings:
Settings > Branches > Branch protection rules > Add rule
  - Require status checks before merging
  - Require: check, test, build, security
  - Require branches to be up to date
  - Require approval from code owners
```

### GitLab: Enable Protected Branches
```yaml
Settings > Repository > Protected branches > Add rule
  - Branch name: main
  - Allowed to merge: Maintainers
  - Allowed to push: Nobody
  - Require approvals: 2
```

---

## References

**GitHub Docs**:
- https://docs.github.com/en/actions/learn-github-actions
- https://docs.github.com/en/actions/deployment/deploying-with-github-actions

**GitLab Docs**:
- https://docs.gitlab.com/ee/ci/
- https://docs.gitlab.com/ee/ci/yaml/
- https://docs.gitlab.com/ee/ci/variables/

**Rust CI/CD**:
- https://rust-lang.github.io/rustup/
- https://github.com/Swatinem/rust-cache
- https://doc.rust-lang.org/cargo/

---

**Last Updated**: 2026-03-29  
**Maintainer**: MEMORY_P DevOps  
**Status**: Implementation Guide (Ready to Apply)
