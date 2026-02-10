# Infrastructure Examples and Templates

> **Ready-to-use configurations for MEMORY_P deployments**

## 📁 Directory Structure

```
infrastructure/
├── examples/
│   ├── oracle-free-tier/      # Oracle Cloud always-free deployment
│   ├── rocky-k3s-cluster/     # Rocky Linux + K3s cluster
│   ├── ubuntu-microk8s/       # Ubuntu Pro + MicroK8s
│   ├── docker-compose/        # Docker Compose stacks
│   └── hybrid-deployment/     # Hybrid cloud + on-premise
├── templates/
│   ├── terraform/             # Terraform modules
│   ├── ansible/               # Ansible playbooks
│   └── kubernetes/            # K8s manifests
└── playbooks/
    ├── troubleshooting/       # Common issue fixes
    ├── optimization/          # Performance tuning
    └── migration/             # Migration guides
```

## 🚀 Quick Start Examples

### 1. Oracle Cloud Free Tier (Zero Cost)

**What you get**:
- 4 OCPU ARM instance
- 24 GB RAM
- 200 GB storage
- Always free forever

**Time to deploy**: 15 minutes

```bash
cd examples/oracle-free-tier
terraform init
terraform apply -var-file="free-tier.tfvars"
```

[📖 Full Guide →](./examples/oracle-free-tier.md)

---

### 2. Rocky Linux + K3s Cluster (Production-Ready)

**Stack**:
- Rocky Linux 9 (3 nodes)
- K3s lightweight Kubernetes
- PostgreSQL + Redis
- Prometheus monitoring

**Time to deploy**: 30 minutes

```bash
cd examples/rocky-k3s-cluster
ansible-playbook -i inventory.yml deploy.yml
```

[📖 Full Guide →](./examples/rocky-k3s-cluster.md)

---

### 3. Ubuntu Pro + MicroK8s (Developer-Friendly)

**Stack**:
- Ubuntu 22.04 LTS Pro
- MicroK8s single-node
- All MEMORY_P services
- GPU support optional

**Time to deploy**: 10 minutes

```bash
cd examples/ubuntu-microk8s
./quick-setup.sh
```

[📖 Full Guide →](./examples/ubuntu-microk8s.md)

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
- [Memory Leaks](./playbooks/memory-leaks.md)
- [Slow Search Queries](./playbooks/slow-search.md)
- [Disk Space Full](./playbooks/disk-space.md)

### Connectivity Issues

- [Database Connection Errors](./playbooks/db-connection.md)
- [Redis Connection Pool](./playbooks/redis-pool.md)
- [Qdrant Unavailable](./playbooks/qdrant-down.md)
- [Kubernetes DNS Issues](./playbooks/k8s-dns.md)

### Deployment Issues

- [Failed Docker Build](./playbooks/docker-build.md)
- [K8s Pod CrashLoopBackOff](./playbooks/crashloop.md)
- [Terraform State Lock](./playbooks/tf-state-lock.md)
- [Ansible Connection Timeout](./playbooks/ansible-timeout.md)

[All Playbooks →](./playbooks/)

---

## 🎯 Use Case Guides

### By Team Size

- **Solo Developer**: [Ubuntu MicroK8s](./examples/ubuntu-microk8s.md)
- **Small Team (2-5)**: [Oracle Free Tier](./examples/oracle-free-tier.md)
- **Startup (5-20)**: [Rocky K3s Cluster](./examples/rocky-k3s-cluster.md)
- **Enterprise**: [Hybrid Deployment](./examples/hybrid-deployment.md)

### By Budget

- **$0/month**: [Oracle Free Tier](./examples/oracle-free-tier.md)
- **<$50/month**: [Oracle + 2 Workers](./examples/oracle-scale-out.md)
- **<$200/month**: [Multi-cloud HA](./examples/hybrid-deployment.md)
- **Enterprise**: [Full Production Stack](./examples/enterprise-stack.md)

### By Workload

- **Development**: [Docker Compose](./examples/docker-compose-dev.md)
- **Staging**: [Ubuntu MicroK8s](./examples/ubuntu-microk8s.md)
- **Production**: [Rocky K3s](./examples/rocky-k3s-cluster.md)
- **GPU Workloads**: [GPU-Enabled Setup](./examples/gpu-deployment.md)

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
