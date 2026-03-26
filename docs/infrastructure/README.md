# Infrastructure Examples and Templates

> **Ready-to-use configurations for MEMORY_P deployments**

## 📁 Directory Structure

```
docs/
└── infrastructure/
    ├── README.md              # This overview file
    ├── examples/
    │   └── oracle-free-tier.md   # Oracle Cloud always-free deployment guide
    ├── playbooks/
    │   └── high-cpu.md        # High CPU troubleshooting playbook
    └── templates/             # (Coming soon: IaC templates)
```

## 🚀 Quick Start Examples

### 1. Oracle Cloud Free Tier (Zero Cost)

**What you get**:
- 4 OCPU ARM instance
- 24 GB RAM
- 200 GB storage
- Always free forever

**Time to deploy**: 15 minutes

See the complete guide: [Oracle Free Tier Deployment](./examples/oracle-free-tier.md)

The guide includes complete Terraform configuration examples and cloud-init automation.

---

### 2. Rocky Linux + K3s Cluster (Production-Ready)

**Coming soon**: Complete guide for Rocky Linux 9 with K3s cluster deployment.

**Stack**:
- Rocky Linux 9 (3 nodes)
- K3s lightweight Kubernetes
- PostgreSQL + Redis
- Prometheus monitoring

**Time to deploy**: 30 minutes (estimated)

---

### 3. Ubuntu Pro + MicroK8s (Developer-Friendly)

**Coming soon**: Complete guide for Ubuntu 22.04 LTS Pro with MicroK8s.

**Stack**:
- Ubuntu 22.04 LTS Pro
- MicroK8s single-node
- All MEMORY_P services
- GPU support optional

**Time to deploy**: 10 minutes (estimated)

---

## 🛠️ Templates

### Terraform Modules

- **compute**: VM instances (Oracle, AWS, Azure, GCP)
- **network**: VPC, subnets, security groups
- **storage**: Block volumes, object storage
- **database**: PostgreSQL, Redis, ClickHouse
- **kubernetes**: K8s cluster setup
- **monitoring**: Prometheus, Grafana

[Browse Templates →](./templates/)

---

### Ansible Playbooks

- **setup-node**: Initialize new server
- **deploy-memory-p**: Deploy MEMORY_P stack
- **update-system**: System updates and patches
- **backup-restore**: Automated backups
- **security-hardening**: Security best practices

[Browse Playbooks →](./templates/)

---

## 🔧 Troubleshooting Playbooks

Common issues and their solutions:

### Performance Issues

- [High CPU Usage](./playbooks/high-cpu.md)
- Memory Leaks (TODO: playbook)
- Slow Search Queries (TODO: playbook)
- Disk Space Full (TODO: playbook)

### Connectivity Issues

- Database Connection Errors (TODO: playbook)
- Redis Connection Pool (TODO: playbook)
- Qdrant Unavailable (TODO: playbook)
- Kubernetes DNS Issues (TODO: playbook)

### Deployment Issues

- Failed Docker Build (TODO: playbook)
- K8s Pod CrashLoopBackOff (TODO: playbook)
- Terraform State Lock (TODO: playbook)
- Ansible Connection Timeout (TODO: playbook)

[All Playbooks →](./playbooks/)

---

## 🎯 Use Case Guides

### By Team Size

- **Solo Developer**: Ubuntu MicroK8s (TODO: guide)
- **Small Team (2-5)**: [Oracle Free Tier](./examples/oracle-free-tier.md)
- **Startup (5-20)**: Rocky K3s Cluster (TODO: guide)
- **Enterprise**: Hybrid Deployment (TODO: guide)

### By Budget

- **$0/month**: [Oracle Free Tier](./examples/oracle-free-tier.md)
- **<$50/month**: Oracle + 2 Workers (TODO: guide)
- **<$200/month**: Multi-cloud HA (TODO: guide)
- **Enterprise**: Full Production Stack (TODO: guide)

### By Workload

- **Development**: Docker Compose (TODO: guide)
- **Staging**: Ubuntu MicroK8s (TODO: guide)
- **Production**: Rocky K3s (TODO: guide)
- **GPU Workloads**: GPU-Enabled Setup (TODO: guide)

---

## 📊 Comparison Matrix

| Example | Cost/Month | Setup Time | Complexity | Production-Ready | GPU Support |
|---------|-----------|------------|------------|------------------|-------------|
| Oracle Free Tier | $0 | 15 min | Low | ⚠️ Single node | ❌ |
| Rocky K3s | $50-100 | 30 min | Medium | ✅ Multi-node | ✅ Optional |
| Ubuntu MicroK8s | $0 | 10 min | Low | ⚠️ Single node | ✅ |
| Hybrid Deploy | $100+ | 60 min | High | ✅ HA | ✅ |
| Docker Compose | $0 | 5 min | Very Low | ❌ Dev only | ❌ |

---

## 🔗 Related Documentation

- [Infrastructure Guide](../INFRASTRUCTURE.md) - RHEL alternatives, cloud options
- [CI/CD Best Practices](../CICD_BEST_PRACTICES.md) - Automation and pipelines
- [Copilot Infrastructure](../COPILOT_INFRASTRUCTURE.md) - AI-assisted DevOps
- [Quick Start](../QUICK_START.md) - Get MEMORY_P running fast

---

**Questions?** Open an issue: https://github.com/Rigohl/MEMORY_P/issues/new
