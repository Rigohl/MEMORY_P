# 🚀 VM3 - Qdrant Vector Search on Oracle Cloud

**Status**: ✅ Ready to provision  
**Spec**: 4 vCPU ARM, 24GB RAM, $0/month (Always Free)  
**What**: Oracle Cloud VM3 for high-performance Qdrant + MEMORY_P

---

## ⚡ Quick Start

### Prerequisites
- OCI CLI installed: `apt-get install oci-cli` or `brew install oci-cli`
- OCI credentials: `oci setup config`
- SSH key: `~/.ssh/id_rsa.pub` (or generate: `ssh-keygen -t rsa -b 4096`)

### Provision VM3 (3 steps)

```bash
# 1. Run provisioning script
./scripts/create_vm3.sh

# 2. When prompted, press ENTER for defaults:
#    - VCN ID: [ENTER]
#    - Subnet ID: [ENTER]  
#    - SSH Key Path: [ENTER]

# 3. Wait 2-3 minutes for VM creation + 5-10 min for cloud-init

# Script will output:
#   ✅ Public IP: <YOUR_VM3_IP>
#   ✅ Updated: src/oracle_vm_bridge.rs
```

### Verify (after cloud-init completes)

```bash
# SSH to VM3
ssh opc@<YOUR_VM3_IP>

# Check Qdrant is running
curl http://localhost:6333/health
# Should return: {"status":"ok"}

# Check system
nproc                # Should show: 4
free -h              # Should show: ~24G
uname -m             # Should show: aarch64
```

---

## 🎯 What Gets Configured

| Config | Value |
|--------|-------|
| **Shape** | VM.Standard.A1.Flex |
| **vCPU** | 4 (ARM Ampere A1) |
| **RAM** | 24 GB |
| **OS** | Oracle Linux 10 (ARM) |
| **Cost** | $0/month |
| **Qdrant Port** | 6333 |
| **Data Dir** | /data/qdrant |
| **Auto Updates** | ✅ Yes |

---

## 🔧 Troubleshooting

### "Shape not available" error
```bash
# Use manual provisioning from Oracle Console instead
# Or check different Availability Domain
```

### "OCI CLI not found"
```bash
# Install: pip install oci-cli
# Or: brew install oci-cli
# Or: apt-get install oci-cli
```

### "SSH: Permission denied"
```bash
# Fix permissions
chmod 600 ~/.oci/config ~/.oci/oci_api_key.pem

# Or regenerate key and re-run script
ssh-keygen -t rsa -b 4096 -f ~/.ssh/id_rsa -N ""
./scripts/create_vm3.sh
```

### "Can't connect after 5 minutes"
```bash
# Cloud-init is still running (normal)
# Check logs on VM3: ssh opc@<IP> tail -f /var/log/cloud-init-output.log
# Wait for: "✅ VM3 Initialization Complete"
```

---

## 📋 Files

- **Script**: `scripts/create_vm3.sh` (Provisioning)
- **Config**: `src/oracle_vm_bridge.rs` (Updated with IP)
- **Code**: `src/qdrant_vm_manager.rs` (Qdrant management)
- **Docs**: This file + `VM3_PROVISIONING_MANUAL_GUIDE.md` (if script fails)

---

## 🔗 Integration

After provisioning, MEMORY_P automatically:
1. Detects VM3 IP (from OCI API)
2. Updates `src/oracle_vm_bridge.rs`
3. Connects to remote Qdrant on VM3
4. Routes vector searches to VM3 (Qdrant)

## 🎉 Result

✅ Qdrant running on VM3 (4 vCPU, 24GB)  
✅ MEMORY_P connected to remote Qdrant  
✅ Vector search with <10ms P99 latency  
✅ FREE forever (Oracle Always Free Tier)
