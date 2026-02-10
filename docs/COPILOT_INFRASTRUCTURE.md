# 🤖 Copilot Infrastructure Guide

> **Maximize GitHub Copilot effectiveness for infrastructure management, AI-assisted DevOps, and automated improvements**

## 📋 Table of Contents

- [Documentation Patterns for Copilot](#-documentation-patterns-for-copilot)
- [Version Control Best Practices](#-version-control-best-practices)
- [Audit and Monitoring](#-audit-and-monitoring)
- [AI-Assisted Workflows](#-ai-assisted-workflows)
- [Automated Improvement Pipelines](#-automated-improvement-pipelines)
- [Troubleshooting with AI](#-troubleshooting-with-ai)
- [Knowledge Graph Integration](#-knowledge-graph-integration)

---

## 📝 Documentation Patterns for Copilot

### Why Documentation Matters for AI

GitHub Copilot and AI assistants learn from:
1. **Comments in code** - Inline explanations
2. **README files** - High-level architecture
3. **Documentation blocks** - Structured metadata
4. **Commit messages** - Historical context
5. **Issue/PR descriptions** - Problem-solution patterns

**Principle**: *Well-documented infrastructure → Better AI suggestions → Faster development*

### Documentation Template for Infrastructure Code

```hcl
# infrastructure/terraform/modules/compute/main.tf

# ============================================
# COPILOT CONTEXT BLOCK
# ============================================
# Module: Oracle Cloud Compute Instance
# Purpose: Deploy MEMORY_P MCP server on ARM
# Dependencies: VCN module, security lists
# Cost: Always-free tier (4 OCPU, 24 GB RAM)
# Managed by: Terraform Cloud
# Owner: @devops-team
# Last updated: 2026-02-10
# ============================================

terraform {
  # COPILOT: Use these exact versions for reproducibility
  required_version = ">= 1.6.0"
  
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 5.0"
      # WHY: Version 5.0+ includes ARM shape support
    }
  }
}

# COPILOT: This variable represents the Oracle Cloud compartment
# It's required because Oracle uses compartments for resource isolation
variable "compartment_id" {
  description = "Oracle Cloud compartment ID (always-free account)"
  type        = string
  
  # COPILOT TIP: Get this from OCI Console → Identity → Compartments
  # Format: ocid1.compartment.oc1..aaaaaaaaxxx
  
  validation {
    condition     = can(regex("^ocid1\\.compartment\\.oc1\\.\\..*", var.compartment_id))
    error_message = "Compartment ID must be a valid OCID starting with 'ocid1.compartment.oc1..'"
  }
}

# COPILOT: Get latest Ubuntu 22.04 ARM image
# This ensures we always use the most up-to-date OS with security patches
data "oci_core_images" "ubuntu_arm" {
  compartment_id           = var.compartment_id
  operating_system         = "Canonical Ubuntu"
  operating_system_version = "22.04"
  shape                    = "VM.Standard.A1.Flex"
  
  # COPILOT: Sort by creation time to get latest image
  sort_by    = "TIMECREATED"
  sort_order = "DESC"
}

# COPILOT: Create ARM instance using always-free tier
# This is the main MEMORY_P server that runs 24/7 at no cost
resource "oci_core_instance" "memory_p_primary" {
  display_name        = "memory-p-primary-arm"
  compartment_id      = var.compartment_id
  availability_domain = var.availability_domain
  
  # COPILOT: ARM shape with max free tier resources
  # 4 OCPU and 24 GB RAM is the maximum for always-free
  shape = "VM.Standard.A1.Flex"
  
  shape_config {
    ocpus         = 4   # Max for always-free
    memory_in_gbs = 24  # Max for always-free (6 GB per OCPU)
  }
  
  # COPILOT: Use latest Ubuntu image with max free storage
  source_details {
    source_type             = "image"
    source_id               = data.oci_core_images.ubuntu_arm.images[0].id
    boot_volume_size_in_gbs = 200  # Max for always-free
  }
  
  # COPILOT: Network configuration for public access
  create_vnic_details {
    subnet_id        = var.subnet_id
    display_name     = "memory-p-vnic"
    assign_public_ip = true
    hostname_label   = "memory-p"
    
    # COPILOT: NSG for fine-grained security rules
    nsg_ids = [var.nsg_id]
  }
  
  # COPILOT: Cloud-init for automated MEMORY_P setup
  metadata = {
    ssh_authorized_keys = var.ssh_public_key
    user_data           = base64encode(file("${path.module}/cloud-init.yaml"))
  }
  
  # COPILOT: Tags for cost tracking and automation
  freeform_tags = {
    Project      = "MEMORY_P"
    Environment  = "production"
    ManagedBy    = "Terraform"
    AlwaysFree   = "true"
    CostCenter   = "engineering"
    AutoShutdown = "false"  # Never shutdown
  }
  
  # COPILOT: Prevent accidental deletion
  lifecycle {
    prevent_destroy = true
    
    # Ignore changes to metadata (cloud-init runs once)
    ignore_changes = [
      metadata["user_data"]
    ]
  }
}

# COPILOT: Output values for use in other modules
output "public_ip" {
  description = "Public IP address for MEMORY_P MCP server"
  value       = oci_core_instance.memory_p_primary.public_ip
  
  # COPILOT: Mark as sensitive to hide in logs
  sensitive = false  # Public IP is not sensitive
}

output "private_ip" {
  description = "Private IP for internal communication"
  value       = oci_core_instance.memory_p_primary.private_ip
}

output "instance_id" {
  description = "OCID of the compute instance"
  value       = oci_core_instance.memory_p_primary.id
}
```

### YAML Documentation Pattern

```yaml
# .github/workflows/deploy-production.yml

# ============================================
# COPILOT CONTEXT BLOCK
# ============================================
# Workflow: Production Deployment
# Trigger: Manual dispatch (protected)
# Environment: Production (requires approval)
# Duration: ~15 minutes
# Rollback: Automatic on failure
# Notifications: Slack #deployments channel
# ============================================

name: 🚀 Deploy to Production

on:
  workflow_dispatch:
    # COPILOT: Manual trigger only (no automatic deployments)
    inputs:
      version:
        description: 'Version to deploy (e.g., v2.0.1)'
        required: true
        type: string
      environment:
        description: 'Target environment'
        required: true
        type: choice
        options:
          - production
          - staging
      dry_run:
        description: 'Dry run (no actual deployment)'
        required: false
        type: boolean
        default: false

# COPILOT: Use production environment with approvals
environment:
  name: ${{ github.event.inputs.environment }}
  # COPILOT: This URL will be displayed after deployment
  url: https://memory-p.production.example.com

# COPILOT: Global environment variables
env:
  # COPILOT: Rust build configuration
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  
  # COPILOT: Deployment configuration
  DEPLOY_TIMEOUT: 600  # 10 minutes
  HEALTH_CHECK_RETRIES: 30
  HEALTH_CHECK_INTERVAL: 10

jobs:
  # ==========================================
  # Pre-deployment validation
  # ==========================================
  validate:
    name: ✅ Validate Deployment
    runs-on: ubuntu-latest
    
    steps:
      - name: 📥 Checkout code
        uses: actions/checkout@v4
        with:
          ref: ${{ github.event.inputs.version }}
          
      # COPILOT: Validate version tag exists
      - name: 🏷️ Verify version tag
        run: |
          if ! git rev-parse "${{ github.event.inputs.version }}" >/dev/null 2>&1; then
            echo "❌ Version tag not found: ${{ github.event.inputs.version }}"
            exit 1
          fi
          echo "✅ Version tag verified"
          
      # COPILOT: Check if version is already deployed
      - name: 🔍 Check current deployment
        run: |
          CURRENT_VERSION=$(curl -sf https://memory-p.production.example.com/version | jq -r '.version')
          if [ "$CURRENT_VERSION" = "${{ github.event.inputs.version }}" ]; then
            echo "⚠️ Version already deployed: $CURRENT_VERSION"
            echo "ALREADY_DEPLOYED=true" >> $GITHUB_ENV
          else
            echo "📌 Current version: $CURRENT_VERSION"
            echo "📌 New version: ${{ github.event.inputs.version }}"
          fi
          
      # COPILOT: Run pre-deployment tests
      - name: 🧪 Run smoke tests
        run: |
          cargo test --release --test smoke_tests
          
  # ==========================================
  # Build Docker image
  # ==========================================
  build:
    name: 🐳 Build Image
    needs: validate
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.inputs.version }}
          
      # COPILOT: Setup Docker Buildx for multi-platform builds
      - name: 🔧 Setup Docker Buildx
        uses: docker/setup-buildx-action@v3
        
      # COPILOT: Login to container registry
      - name: 🔐 Login to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
          
      # COPILOT: Build and push image
      - name: 🐳 Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: ${{ github.event.inputs.dry_run == false }}
          tags: |
            ghcr.io/${{ github.repository }}:${{ github.event.inputs.version }}
            ghcr.io/${{ github.repository }}:latest
          # COPILOT: Cache layers for faster builds
          cache-from: type=gha
          cache-to: type=gha,mode=max
          # COPILOT: Add build metadata
          labels: |
            org.opencontainers.image.title=MEMORY_P
            org.opencontainers.image.version=${{ github.event.inputs.version }}
            org.opencontainers.image.source=${{ github.repositoryUrl }}
            org.opencontainers.image.revision=${{ github.sha }}
            
  # ==========================================
  # Deploy to Kubernetes
  # ==========================================
  deploy:
    name: 🚀 Deploy
    needs: build
    runs-on: ubuntu-latest
    if: github.event.inputs.dry_run == false
    
    steps:
      # COPILOT: Setup kubectl with cluster credentials
      - name: ⚙️ Configure kubectl
        uses: azure/k8s-set-context@v3
        with:
          method: kubeconfig
          kubeconfig: ${{ secrets.KUBECONFIG }}
          
      # COPILOT: Update deployment image
      - name: 🔄 Update deployment
        run: |
          kubectl set image deployment/memory-p \
            memory-p=ghcr.io/${{ github.repository }}:${{ github.event.inputs.version }} \
            --namespace=memory-p
            
      # COPILOT: Wait for rollout to complete
      - name: ⏳ Wait for rollout
        run: |
          kubectl rollout status deployment/memory-p \
            --namespace=memory-p \
            --timeout=${DEPLOY_TIMEOUT}s
            
      # COPILOT: Verify deployment health
      - name: 🏥 Health check
        run: |
          for i in $(seq 1 $HEALTH_CHECK_RETRIES); do
            if curl -sf https://memory-p.production.example.com/health; then
              echo "✅ Health check passed"
              exit 0
            fi
            echo "⏳ Waiting for health check... ($i/$HEALTH_CHECK_RETRIES)"
            sleep $HEALTH_CHECK_INTERVAL
          done
          echo "❌ Health check failed"
          exit 1
          
      # COPILOT: Run post-deployment validation
      - name: 🧪 Validate deployment
        run: |
          ./scripts/validate_deployment.sh
          
  # ==========================================
  # Rollback on failure
  # ==========================================
  rollback:
    name: ↩️ Rollback
    needs: deploy
    runs-on: ubuntu-latest
    if: failure() && github.event.inputs.dry_run == false
    
    steps:
      - name: ⚙️ Configure kubectl
        uses: azure/k8s-set-context@v3
        with:
          method: kubeconfig
          kubeconfig: ${{ secrets.KUBECONFIG }}
          
      # COPILOT: Rollback to previous version
      - name: ↩️ Rollback deployment
        run: |
          kubectl rollout undo deployment/memory-p --namespace=memory-p
          kubectl rollout status deployment/memory-p --namespace=memory-p
          
      # COPILOT: Notify team of rollback
      - name: 📢 Notify rollback
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "❌ Production deployment FAILED and was rolled back",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Deployment Failed*\nVersion: ${{ github.event.inputs.version }}\nEnvironment: ${{ github.event.inputs.environment }}\nAction: Automatic rollback completed"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

---

## 📋 Version Control Best Practices

### Git Commit Messages for AI Context

```bash
# ❌ Bad commit message (no context for AI)
git commit -m "fix"

# ✅ Good commit message (rich context)
git commit -m "fix(terraform): increase ARM instance memory to 24GB

Oracle Cloud allows 24GB (not 16GB) on 4 OCPU ARM instances.
This maximizes the always-free tier resources.

Related to: MEMORY_P deployment optimization
Impact: 50% more memory for vector search cache
Testing: Validated on staging environment

Closes #123"
```

**Template**:
```
<type>(<scope>): <short description>

<detailed description>
- Why this change was needed
- What problem it solves
- How it was tested

<related information>
- Related issue/PR numbers
- Breaking changes
- Migration steps (if any)

<copilot context>
Copilot: This change affects [X]. Future modifications should consider [Y].
```

### Semantic Commit Types

```bash
# Infrastructure
terraform: Terraform configuration changes
ansible: Ansible playbook changes
k8s: Kubernetes manifests
docker: Dockerfile or docker-compose changes

# CI/CD
ci: GitHub Actions workflow changes
cd: Deployment automation
pipeline: General pipeline modifications

# Documentation
docs: Documentation updates
readme: README.md changes
guide: Tutorial or guide updates

# Code
feat: New feature
fix: Bug fix
perf: Performance improvement
refactor: Code refactoring
test: Test additions/modifications
```

### Branch Naming Convention

```bash
# Feature branches
feature/oracle-cloud-arm-support
feature/k3s-multi-node-cluster
feature/automated-backups

# Infrastructure branches
infra/production-kubernetes
infra/staging-environment
infra/monitoring-stack

# Documentation branches
docs/cicd-best-practices
docs/infrastructure-guide
docs/troubleshooting-playbook

# Hotfix branches
hotfix/production-disk-space
hotfix/ssl-certificate-renewal
```

### Tags for Infrastructure Releases

```bash
# Semantic versioning for infrastructure
git tag -a infra-v1.0.0 -m "Initial production infrastructure

- Oracle Cloud ARM instances (always-free)
- K3s cluster (1 master, 2 workers)
- PostgreSQL + Redis stack
- Prometheus monitoring

Copilot: This is the baseline infrastructure. All future changes
should maintain backward compatibility with this configuration."

# Release tags with metadata
git tag -a release-2026.02.10 -m "Monthly release - February 2026

Infrastructure changes:
- Upgraded K3s to v1.29
- Added GPU support to worker nodes
- Implemented blue-green deployments

Application changes:
- MEMORY_P v2.0.1
- Julia 1.10.0 mathematical brain
- New SCANN search engine

Performance improvements:
- 40% faster vector search
- 60% less memory usage
- Sub-100ms P99 latency

Copilot: Use this as reference for stable production configuration."
```

---

## 🔍 Audit and Monitoring

### Infrastructure State Audit

```bash
#!/bin/bash
# scripts/audit_infrastructure.sh

# COPILOT CONTEXT: Comprehensive infrastructure audit script
# Purpose: Generate audit report for compliance/security review
# Output: JSON and Markdown reports
# Frequency: Daily (automated via cron)

set -euo pipefail

OUTPUT_DIR="${1:-./audit-reports}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="$OUTPUT_DIR/audit_$TIMESTAMP.json"

mkdir -p "$OUTPUT_DIR"

echo "🔍 Starting infrastructure audit..."

# COPILOT: Collect Terraform state
echo "📊 Analyzing Terraform state..."
terraform show -json > "$OUTPUT_DIR/terraform_state_$TIMESTAMP.json"

# COPILOT: Extract key metrics
TOTAL_RESOURCES=$(terraform state list | wc -l)
COMPUTE_INSTANCES=$(terraform state list | grep -c 'oci_core_instance' || true)
STORAGE_VOLUMES=$(terraform state list | grep -c 'oci_core_volume' || true)

# COPILOT: Check for drift
echo "🔄 Checking for infrastructure drift..."
terraform plan -detailed-exitcode > /dev/null 2>&1
DRIFT_STATUS=$?

if [ $DRIFT_STATUS -eq 2 ]; then
  DRIFT="detected"
  DRIFT_DETAILS=$(terraform plan -no-color)
else
  DRIFT="none"
  DRIFT_DETAILS="No drift detected"
fi

# COPILOT: Security audit
echo "🔒 Running security checks..."

# Check for unencrypted resources
UNENCRYPTED=$(terraform state list | grep -E 'oci_core_volume|oci_database' | \
  while read resource; do
    ENCRYPTED=$(terraform state show "$resource" | grep -c 'kms_key_id' || true)
    if [ $ENCRYPTED -eq 0 ]; then
      echo "$resource"
    fi
  done)

# Check for public IPs
PUBLIC_IPS=$(terraform state list | grep 'oci_core_instance' | \
  while read instance; do
    terraform state show "$instance" | grep 'public_ip' | awk '{print $3}' | tr -d '"'
  done)

# COPILOT: Cost estimation
echo "💰 Estimating costs..."
# Note: Oracle Cloud free tier doesn't have direct cost API
# This estimates based on resource usage

COST_ESTIMATE=$(cat <<EOF
{
  "always_free": {
    "compute": "4 OCPU ARM (free)",
    "storage": "200 GB (free)",
    "network": "10 TB egress (free)"
  },
  "paid_resources": $(terraform state list | grep -v -E 'free|always_free' | wc -l)
}
EOF
)

# COPILOT: Generate JSON report
cat > "$REPORT_FILE" <<EOF
{
  "timestamp": "$TIMESTAMP",
  "audit_version": "1.0",
  "summary": {
    "total_resources": $TOTAL_RESOURCES,
    "compute_instances": $COMPUTE_INSTANCES,
    "storage_volumes": $STORAGE_VOLUMES,
    "drift_status": "$DRIFT",
    "security_issues": $(echo "$UNENCRYPTED" | wc -l)
  },
  "drift": {
    "status": "$DRIFT",
    "details": $(echo "$DRIFT_DETAILS" | jq -Rs .)
  },
  "security": {
    "unencrypted_resources": $(echo "$UNENCRYPTED" | jq -R . | jq -s .),
    "public_ips": $(echo "$PUBLIC_IPS" | jq -R . | jq -s .)
  },
  "cost": $COST_ESTIMATE
}
EOF

# COPILOT: Generate Markdown report
cat > "$OUTPUT_DIR/audit_$TIMESTAMP.md" <<EOF
# Infrastructure Audit Report

**Date**: $(date)
**Auditor**: Automated (infrastructure-audit.sh)

## Summary

- **Total Resources**: $TOTAL_RESOURCES
- **Compute Instances**: $COMPUTE_INSTANCES
- **Storage Volumes**: $STORAGE_VOLUMES
- **Drift Status**: $DRIFT

## Drift Analysis

\`\`\`
$DRIFT_DETAILS
\`\`\`

## Security Findings

### Unencrypted Resources
$(if [ -z "$UNENCRYPTED" ]; then echo "✅ All resources are encrypted"; else echo "$UNENCRYPTED"; fi)

### Public IP Addresses
$(echo "$PUBLIC_IPS" | while read ip; do echo "- $ip"; done)

## Cost Estimate

Always-Free Resources:
- 4 OCPU ARM compute (free)
- 200 GB block storage (free)
- 10 TB network egress (free)

## Recommendations

1. Review unencrypted resources and enable KMS encryption
2. Verify public IPs are necessary and properly secured
3. Check for unused resources to optimize costs
4. Update tags for better resource tracking

---

**Copilot Context**: This audit provides baseline for infrastructure compliance.
Use this as reference when making changes. All modifications should maintain
security posture and cost efficiency.
EOF

echo "✅ Audit complete!"
echo "📄 JSON report: $REPORT_FILE"
echo "📄 Markdown report: $OUTPUT_DIR/audit_$TIMESTAMP.md"

# COPILOT: Send notifications if issues found
if [ "$DRIFT" = "detected" ] || [ -n "$UNENCRYPTED" ]; then
  echo "⚠️ Issues detected! Review the report."
  # Send to Slack/email (implement based on your setup)
fi
```

### Monitoring Configuration

```yaml
# monitoring/prometheus/memory-p-rules.yml

# COPILOT CONTEXT: Prometheus alerting rules for MEMORY_P
# Purpose: Proactive monitoring and auto-remediation triggers
# Review: Monthly or after incidents
# Integration: AlertManager → Slack/PagerDuty

groups:
  - name: memory_p_infrastructure
    interval: 30s
    
    rules:
      # COPILOT: Alert on high CPU usage
      - alert: HighCPUUsage
        # WHY: ARM instances have limited CPU, need to detect overload early
        expr: node_cpu_seconds_total{mode="idle"} < 20
        for: 5m
        labels:
          severity: warning
          component: compute
          # COPILOT: Auto-remediation possible
          auto_remediate: "true"
        annotations:
          summary: "High CPU usage on {{ $labels.instance }}"
          description: "CPU idle time below 20% for 5 minutes"
          remediation: "Consider scaling horizontally or optimizing workload"
          
      # COPILOT: Alert on memory pressure
      - alert: MemoryPressure
        # WHY: 24GB RAM is max on free tier, must prevent OOM
        expr: (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) < 0.1
        for: 2m
        labels:
          severity: critical
          component: memory
          auto_remediate: "true"
        annotations:
          summary: "Memory pressure on {{ $labels.instance }}"
          description: "Less than 10% memory available"
          remediation: "Restart memory-hungry services or clear caches"
          
      # COPILOT: Alert on disk space
      - alert: DiskSpaceLow
        # WHY: 200GB is max free tier, can't expand easily
        expr: (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes) < 0.15
        for: 5m
        labels:
          severity: warning
          component: storage
          auto_remediate: "true"
        annotations:
          summary: "Low disk space on {{ $labels.instance }}"
          description: "Less than 15% disk space available"
          remediation: "Run log rotation, clear Docker images, or expand volume"
          
      # COPILOT: Alert on search latency
      - alert: SearchLatencyHigh
        # WHY: MEMORY_P SLA is <100ms P99 latency
        expr: histogram_quantile(0.99, rate(memory_p_search_duration_seconds_bucket[5m])) > 0.1
        for: 5m
        labels:
          severity: warning
          component: application
          sla_breach: "true"
        annotations:
          summary: "High search latency detected"
          description: "P99 search latency above 100ms SLA"
          remediation: "Check engine health, review slow queries, consider cache tuning"
          
      # COPILOT: Alert on engine failures
      - alert: SearchEngineDown
        # WHY: Hybrid search requires multiple engines operational
        expr: memory_p_engine_health{status="healthy"} == 0
        for: 1m
        labels:
          severity: critical
          component: application
          auto_remediate: "true"
        annotations:
          summary: "Search engine {{ $labels.engine }} is down"
          description: "Engine health check failing"
          remediation: "Restart engine container, check logs, failover to backup"
```

---

## 🤖 AI-Assisted Workflows

### Copilot Chat Commands

Create a `.github/copilot-commands.md` file:

```markdown
# Copilot Custom Commands for MEMORY_P Infrastructure

## Deployment Commands

### /deploy-staging
Deploy current branch to staging environment
```bash
# 1. Build Docker image
docker build -t memory-p:staging .

# 2. Push to registry
docker push ghcr.io/rigohl/memory-p:staging

# 3. Update K8s deployment
kubectl set image deployment/memory-p memory-p=ghcr.io/rigohl/memory-p:staging -n staging

# 4. Wait for rollout
kubectl rollout status deployment/memory-p -n staging
```

### /check-health
Check health of all infrastructure components
```bash
# MEMORY_P application
curl -f http://localhost:4040/health

# Qdrant
curl -f http://localhost:6333/health

# PostgreSQL
pg_isready -h localhost -p 5432

# Redis
redis-cli ping

# K8s cluster
kubectl get nodes
kubectl get pods --all-namespaces
```

### /estimate-costs
Estimate monthly infrastructure costs
```bash
# Run cost estimation script
python scripts/estimate_costs.py --environment production

# Output example:
# Oracle Cloud: $0 (always-free tier)
# GitHub Actions: ~$10/month (estimate)
# Monitoring: $0 (Prometheus/Grafana open source)
# Total: ~$10/month
```

## Troubleshooting Commands

### /debug-slow-search
Debug slow search queries
```bash
# 1. Check engine latencies
curl http://localhost:9091/metrics | grep memory_p_search_duration

# 2. Analyze slow query log
tail -f /var/log/memory-p/slow-queries.log

# 3. Check resource usage
docker stats memory-p

# 4. Profile with flamegraph
cargo flamegraph --bench search_benchmark
```

### /fix-oom
Fix out-of-memory issues
```bash
# 1. Check memory usage
free -h
docker stats --no-stream

# 2. Identify memory hogs
ps aux --sort=-%mem | head -n 10

# 3. Clear caches
sync; echo 3 > /proc/sys/vm/drop_caches

# 4. Restart heavy services
docker-compose restart qdrant

# 5. Adjust OOM settings
echo 100 > /proc/sys/vm/oom_kill_allocating_task
```

## Code Generation Commands

### /generate-terraform
Generate Terraform module for new service
```hcl
# Template: New service module
module "new_service" {
  source = "./modules/service"
  
  name        = "new-service"
  environment = var.environment
  
  # Compute
  instance_shape = "VM.Standard.A1.Flex"
  ocpus          = 2
  memory_gb      = 12
  
  # Storage
  boot_volume_size_gb = 50
  
  # Network
  subnet_id       = var.subnet_id
  security_groups = [var.default_sg]
  
  # Tags
  tags = {
    Project     = "MEMORY_P"
    ManagedBy   = "Terraform"
    Environment = var.environment
  }
}
```

### /generate-k8s-manifest
Generate Kubernetes manifest for service
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: new-service
  namespace: memory-p
spec:
  replicas: 3
  selector:
    matchLabels:
      app: new-service
  template:
    metadata:
      labels:
        app: new-service
    spec:
      containers:
      - name: new-service
        image: ghcr.io/rigohl/new-service:latest
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: new-service
  namespace: memory-p
spec:
  selector:
    app: new-service
  ports:
  - port: 80
    targetPort: 8080
  type: ClusterIP
```
```

---

## 🔄 Automated Improvement Pipelines

### Self-Improving Infrastructure

```yaml
# .github/workflows/infrastructure-optimization.yml

name: 🔧 Infrastructure Optimization

# COPILOT CONTEXT: AI-driven infrastructure optimization
# Purpose: Automatically detect and suggest improvements
# Frequency: Weekly
# Approval: Required for production changes

on:
  schedule:
    - cron: '0 3 * * 1'  # Every Monday at 3 AM
  workflow_dispatch:

jobs:
  analyze:
    name: 📊 Analyze Infrastructure
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      # COPILOT: Analyze Terraform for improvements
      - name: 🔍 Terraform analysis
        run: |
          # Use tfsec for security scanning
          docker run --rm -v $PWD:/src aquasec/tfsec /src --format json > tfsec-results.json
          
          # Use checkov for best practices
          pip install checkov
          checkov -d infrastructure/terraform --output json > checkov-results.json
          
      # COPILOT: Cost optimization suggestions
      - name: 💰 Cost analysis
        run: |
          # Analyze resource usage vs allocation
          python scripts/cost_optimization.py > cost-recommendations.md
          
      # COPILOT: Performance recommendations
      - name: ⚡ Performance analysis
        run: |
          # Analyze metrics for optimization opportunities
          python scripts/performance_recommendations.py > perf-recommendations.md
          
      # COPILOT: Generate improvement PR
      - name: 📝 Create improvement PR
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: |
            chore(infra): automated infrastructure improvements
            
            AI-generated recommendations based on analysis:
            - Security: $(cat tfsec-results.json | jq '.results | length') issues
            - Best practices: $(cat checkov-results.json | jq '.summary.failed') checks
            - Cost: Potential savings identified
            - Performance: Optimization opportunities detected
            
            Copilot: Review these suggestions and apply as needed.
          branch: automation/infrastructure-improvements
          title: '🤖 Automated Infrastructure Improvements'
          body-path: improvement-summary.md
          labels: |
            infrastructure
            automated
            needs-review
```

### AI-Powered Documentation Updates

```python
# scripts/update_documentation.py

"""
COPILOT CONTEXT: Automatic documentation updater
Purpose: Keep documentation in sync with code changes
Triggers: On code commits to main branch
AI Model: GitHub Copilot + GPT-4 for analysis
"""

import os
import re
from pathlib import Path
from typing import List, Dict
import anthropic  # or openai

def analyze_code_changes(git_diff: str) -> Dict[str, any]:
    """
    COPILOT: Analyze git diff and identify documentation updates needed
    
    Returns:
        Dict with:
        - files_changed: List of files
        - doc_updates: Suggested documentation changes
        - examples_needed: New code examples to add
    """
    client = anthropic.Client(api_key=os.environ["ANTHROPIC_API_KEY"])
    
    prompt = f"""
    Analyze this git diff and suggest documentation updates:
    
    {git_diff}
    
    Provide:
    1. Which documentation files need updating
    2. What changes should be made
    3. New examples to add
    4. Outdated content to remove
    
    Format as JSON.
    """
    
    response = client.messages.create(
        model="claude-3-opus-20240229",
        max_tokens=4000,
        messages=[{"role": "user", "content": prompt}]
    )
    
    return parse_ai_response(response.content)

def update_readme(changes: Dict) -> None:
    """
    COPILOT: Update README.md with new features/changes
    """
    readme_path = Path("README.md")
    content = readme_path.read_text()
    
    # Add new features section if needed
    if changes.get("new_features"):
        features_section = generate_features_section(changes["new_features"])
        content = inject_section(content, "## Features", features_section)
    
    # Update version numbers
    if changes.get("version"):
        content = re.sub(r"v\d+\.\d+\.\d+", changes["version"], content)
    
    readme_path.write_text(content)

def generate_architecture_diagram(code_structure: Dict) -> str:
    """
    COPILOT: Generate Mermaid diagram from code structure
    """
    nodes = []
    edges = []
    
    for module, dependencies in code_structure.items():
        nodes.append(f"    {module}")
        for dep in dependencies:
            edges.append(f"    {module} --> {dep}")
    
    diagram = f"""
```mermaid
graph TD
{chr(10).join(nodes)}
{chr(10).join(edges)}
```
"""
    return diagram

if __name__ == "__main__":
    # Get git diff from last commit
    import subprocess
    
    diff = subprocess.check_output(["git", "diff", "HEAD~1", "HEAD"]).decode()
    
    # Analyze changes
    changes = analyze_code_changes(diff)
    
    # Update documentation
    update_readme(changes)
    update_api_docs(changes)
    update_architecture_docs(changes)
    
    print("✅ Documentation updated successfully!")
    print(f"📝 Updated: {changes['files_updated']}")
```

---

## 🐛 Troubleshooting with AI

### AI-Powered Debugging Assistant

```bash
#!/bin/bash
# scripts/ai_debug.sh

# COPILOT CONTEXT: AI-powered debugging assistant
# Purpose: Analyze errors and suggest fixes
# Usage: ./ai_debug.sh <error_log_file>

set -euo pipefail

ERROR_LOG="${1:-/var/log/memory-p/error.log}"
AI_MODEL="${2:-claude-3-opus}"

echo "🔍 Analyzing errors with AI..."

# COPILOT: Extract recent errors
ERRORS=$(tail -n 100 "$ERROR_LOG" | grep -E "ERROR|FATAL|CRITICAL" || true)

if [ -z "$ERRORS" ]; then
  echo "✅ No recent errors found!"
  exit 0
fi

# COPILOT: Get system context
SYSTEM_INFO=$(cat <<EOF
{
  "hostname": "$(hostname)",
  "os": "$(uname -a)",
  "memory": "$(free -h | grep Mem | awk '{print $3"/"$2}')",
  "disk": "$(df -h / | tail -1 | awk '{print $3"/"$2}')",
  "uptime": "$(uptime -p)",
  "load": "$(cat /proc/loadavg)"
}
EOF
)

# COPILOT: Analyze with Claude
ANALYSIS=$(claude --model "$AI_MODEL" <<EOF
I have the following errors in my MEMORY_P infrastructure:

\`\`\`
$ERRORS
\`\`\`

System context:
\`\`\`json
$SYSTEM_INFO
\`\`\`

Please:
1. Identify the root cause
2. Suggest immediate fix
3. Recommend long-term prevention
4. Provide exact commands to resolve

Format as Markdown with sections for each point.
EOF
)

# COPILOT: Save analysis
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
REPORT_FILE="/tmp/ai_debug_$TIMESTAMP.md"

cat > "$REPORT_FILE" <<EOF
# AI Debug Report - $TIMESTAMP

## Errors Analyzed

\`\`\`
$ERRORS
\`\`\`

## System Context

\`\`\`json
$SYSTEM_INFO
\`\`\`

## AI Analysis

$ANALYSIS

---

**Generated by**: ai_debug.sh
**Model**: $AI_MODEL
**Timestamp**: $(date)

**Copilot Context**: This analysis is AI-generated. Always verify suggestions
before applying to production. Document the resolution for future reference.
EOF

echo "📄 Analysis saved to: $REPORT_FILE"
cat "$REPORT_FILE"

# COPILOT: Offer to apply fix if safe
if echo "$ANALYSIS" | grep -q "Safe to auto-apply"; then
  read -p "Apply suggested fix automatically? (y/N) " -n 1 -r
  echo
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    # Extract and execute commands (implement safety checks!)
    echo "🔧 Applying fix..."
  fi
fi
```

---

## 🧠 Knowledge Graph Integration

### Building Infrastructure Knowledge Graph

```python
# scripts/build_knowledge_graph.py

"""
COPILOT CONTEXT: Infrastructure knowledge graph builder
Purpose: Create searchable knowledge base from docs/code/logs
Output: Neo4j graph database + vector embeddings
Usage: Enables AI to understand infrastructure relationships
"""

from neo4j import GraphDatabase
from sentence_transformers import SentenceTransformer
import re
from pathlib import Path

class InfrastructureKnowledgeGraph:
    def __init__(self, neo4j_uri, user, password):
        self.driver = GraphDatabase.driver(neo4j_uri, auth=(user, password))
        self.embedder = SentenceTransformer('all-MiniLM-L6-v2')
        
    def ingest_terraform(self, terraform_dir: Path):
        """
        COPILOT: Parse Terraform files and create graph nodes
        
        Creates:
        - Resource nodes (compute, storage, network)
        - Dependency edges (depends_on, references)
        - Metadata properties (cost, region, tags)
        """
        with self.driver.session() as session:
            for tf_file in terraform_dir.rglob("*.tf"):
                content = tf_file.read_text()
                
                # Extract resources
                resources = re.findall(
                    r'resource\s+"([^"]+)"\s+"([^"]+)"\s*{([^}]+)}',
                    content,
                    re.DOTALL
                )
                
                for resource_type, resource_name, resource_body in resources:
                    # Create node
                    session.run("""
                        MERGE (r:Resource {
                            type: $type,
                            name: $name,
                            file: $file
                        })
                        SET r.embedding = $embedding
                    """, {
                        "type": resource_type,
                        "name": resource_name,
                        "file": str(tf_file),
                        "embedding": self.embedder.encode(
                            f"{resource_type} {resource_name} {resource_body}"
                        ).tolist()
                    })
                    
                    # Extract dependencies
                    deps = re.findall(r'(\w+\.\w+\.\w+)', resource_body)
                    for dep in deps:
                        session.run("""
                            MATCH (r:Resource {name: $name})
                            MERGE (d:Resource {name: $dep})
                            MERGE (r)-[:DEPENDS_ON]->(d)
                        """, {"name": resource_name, "dep": dep})
    
    def ingest_documentation(self, docs_dir: Path):
        """
        COPILOT: Parse documentation and create searchable nodes
        """
        with self.driver.session() as session:
            for doc_file in docs_dir.rglob("*.md"):
                content = doc_file.read_text()
                
                # Create document node
                session.run("""
                    MERGE (d:Documentation {
                        title: $title,
                        path: $path
                    })
                    SET d.content = $content,
                        d.embedding = $embedding
                """, {
                    "title": doc_file.stem,
                    "path": str(doc_file),
                    "content": content,
                    "embedding": self.embedder.encode(content).tolist()
                })
                
                # Extract code examples
                code_blocks = re.findall(r'```(\w+)\n(.*?)```', content, re.DOTALL)
                for lang, code in code_blocks:
                    session.run("""
                        MATCH (d:Documentation {path: $path})
                        CREATE (e:Example {
                            language: $lang,
                            code: $code,
                            embedding: $embedding
                        })
                        CREATE (d)-[:CONTAINS_EXAMPLE]->(e)
                    """, {
                        "path": str(doc_file),
                        "lang": lang,
                        "code": code,
                        "embedding": self.embedder.encode(code).tolist()
                    })
    
    def semantic_search(self, query: str, top_k: int = 5):
        """
        COPILOT: Vector similarity search across knowledge graph
        """
        query_embedding = self.embedder.encode(query).tolist()
        
        with self.driver.session() as session:
            results = session.run("""
                MATCH (n)
                WHERE n.embedding IS NOT NULL
                WITH n, gds.similarity.cosine(n.embedding, $query_embedding) AS similarity
                WHERE similarity > 0.5
                RETURN n, similarity
                ORDER BY similarity DESC
                LIMIT $top_k
            """, {"query_embedding": query_embedding, "top_k": top_k})
            
            return [{"node": record["n"], "similarity": record["similarity"]} 
                    for record in results]

# COPILOT: Usage example
if __name__ == "__main__":
    kg = InfrastructureKnowledgeGraph(
        "bolt://localhost:7687",
        "neo4j",
        "password"
    )
    
    # Build graph from infrastructure
    kg.ingest_terraform(Path("infrastructure/terraform"))
    kg.ingest_documentation(Path("docs"))
    
    # Query
    results = kg.semantic_search("How do I deploy to Oracle Cloud?")
    for r in results:
        print(f"Found: {r['node']['title']} (similarity: {r['similarity']:.2f})")
```

---

## 📚 Additional Resources

- [GitHub Copilot Documentation](https://docs.github.com/en/copilot)
- [AI-Powered DevOps](https://learn.microsoft.com/en-us/azure/devops/ai/)
- [MLOps Best Practices](https://ml-ops.org/)
- [Infrastructure as Code with AI](https://www.hashicorp.com/resources/ai-iac)

**Related MEMORY_P Docs**:
- [Infrastructure Guide](./INFRASTRUCTURE.md) - OS and cloud setup
- [CI/CD Best Practices](./CICD_BEST_PRACTICES.md) - Pipeline automation
- [Quick Start](./QUICK_START.md) - Get running in 5 minutes

---

**Questions?** Open an issue: https://github.com/Rigohl/MEMORY_P/issues/new
