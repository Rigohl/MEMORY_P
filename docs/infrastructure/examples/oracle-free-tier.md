# Oracle Cloud Free Tier - MEMORY_P Deployment

> **Deploy MEMORY_P on Oracle Cloud's always-free tier (4 ARM cores, 24GB RAM) at $0/month**

## 📋 What You Get

### Always-Free Resources
- ✅ **Compute**: 4 OCPU ARM Ampere A1 + 24 GB RAM
- ✅ **Storage**: 200 GB block storage
- ✅ **Network**: 10 TB outbound traffic/month
- ✅ **Load Balancer**: 1 instance
- ✅ **Database**: 2 Autonomous Database instances (optional)

**Cost**: $0/month forever (no credit card charges after trial)

---

## 🚀 Quick Start (15 minutes)

### Prerequisites

```bash
# Install required tools (modern method without deprecated apt-key)
wget -O- https://apt.releases.hashicorp.com/gpg | \
  gpg --dearmor | \
  sudo tee /usr/share/keyrings/hashicorp-archive-keyring.gpg > /dev/null
echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] \
  https://apt.releases.hashicorp.com $(lsb_release -cs) main" | \
  sudo tee /etc/apt/sources.list.d/hashicorp.list
sudo apt-get update && sudo apt-get install terraform

# Install OCI CLI
bash -c "$(curl -L https://raw.githubusercontent.com/oracle/oci-cli/master/scripts/install/install.sh)"
```

### Step 1: Oracle Cloud Account Setup

1. Sign up at https://cloud.oracle.com/
2. Complete identity verification
3. Navigate to: Profile → User Settings → API Keys
4. Generate API key pair (save private key as `~/.oci/oci_api_key.pem`)

### Step 2: Configure OCI CLI

```bash
oci setup config

# Enter when prompted:
# - Tenancy OCID (from Console → Tenancy)
# - User OCID (from Console → Profile)
# - Region (e.g., us-ashburn-1)
# - Key file: ~/.oci/oci_api_key.pem
```

### Step 3: Deploy with Terraform

**Note**: The Terraform configuration examples below should be created in your own infrastructure directory (e.g., `infra/oracle-free-tier/`). The complete configuration includes `main.tf`, `free-tier.tfvars`, and `cloud-init.yaml` as shown in the sections below.

```bash
# Create your infrastructure directory
mkdir -p infra/oracle-free-tier
cd infra/oracle-free-tier

# Create the configuration files shown below (main.tf, free-tier.tfvars, cloud-init.yaml)

# Initialize Terraform
terraform init

# Review plan
terraform plan -var-file="free-tier.tfvars"

# Deploy
terraform apply -var-file="free-tier.tfvars"

# Get instance IP
terraform output instance_public_ip
```

---

## 📁 Configuration Files

### free-tier.tfvars

```hcl
# Oracle Cloud always-free tier configuration

# Your tenancy OCID (from OCI Console)
tenancy_ocid = "ocid1.tenancy.oc1..aaaaaaaxxxxxx"

# Your user OCID
user_ocid = "ocid1.user.oc1..aaaaaaaxxxxxx"

# Your compartment OCID (use root compartment for free tier)
compartment_ocid = "ocid1.compartment.oc1..aaaaaaaxxxxxx"

# Region (choose closest to you)
region = "us-ashburn-1"

# SSH public key for instance access
ssh_public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQ..."

# Instance configuration (always-free maximums)
instance_shape = "VM.Standard.A1.Flex"
instance_ocpus = 4
instance_memory_gb = 24
boot_volume_size_gb = 200

# Availability domain (get from: oci iam availability-domain list)
availability_domain = "zInd:US-ASHBURN-AD-1"
```

### main.tf

```hcl
terraform {
  required_version = ">= 1.6.0"
  
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 5.0"
    }
  }
}

provider "oci" {
  tenancy_ocid     = var.tenancy_ocid
  user_ocid        = var.user_ocid
  private_key_path = "~/.oci/oci_api_key.pem"
  fingerprint      = var.fingerprint
  region           = var.region
}

# Variables
variable "tenancy_ocid" {}
variable "user_ocid" {}
variable "compartment_ocid" {}
variable "region" {}
variable "ssh_public_key" {}
variable "fingerprint" {}
variable "availability_domain" {}

# VCN (Virtual Cloud Network)
resource "oci_core_vcn" "memory_p_vcn" {
  compartment_id = var.compartment_ocid
  cidr_block     = "10.0.0.0/16"
  display_name   = "memory-p-vcn"
  dns_label      = "memoryp"
}

# Internet Gateway
resource "oci_core_internet_gateway" "memory_p_igw" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.memory_p_vcn.id
  display_name   = "memory-p-igw"
  enabled        = true
}

# Route Table
resource "oci_core_route_table" "memory_p_rt" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.memory_p_vcn.id
  display_name   = "memory-p-rt"
  
  route_rules {
    destination       = "0.0.0.0/0"
    network_entity_id = oci_core_internet_gateway.memory_p_igw.id
  }
}

# Security List
resource "oci_core_security_list" "memory_p_sl" {
  compartment_id = var.compartment_ocid
  vcn_id         = oci_core_vcn.memory_p_vcn.id
  display_name   = "memory-p-sl"
  
  # Egress: Allow all
  egress_security_rules {
    destination = "0.0.0.0/0"
    protocol    = "all"
  }
  
  # Ingress: SSH
  ingress_security_rules {
    protocol = "6"  # TCP
    source   = "0.0.0.0/0"
    tcp_options {
      min = 22
      max = 22
    }
  }
  
  # Ingress: MEMORY_P HTTP
  ingress_security_rules {
    protocol = "6"  # TCP
    source   = "0.0.0.0/0"
    tcp_options {
      min = 4040
      max = 4040
    }
  }
  
  # Ingress: Qdrant
  ingress_security_rules {
    protocol = "6"  # TCP
    source   = "10.0.0.0/16"  # VCN only
    tcp_options {
      min = 6333
      max = 6333
    }
  }
  
  # Ingress: PostgreSQL
  ingress_security_rules {
    protocol = "6"  # TCP
    source   = "10.0.0.0/16"  # VCN only
    tcp_options {
      min = 5432
      max = 5432
    }
  }
  
  # Ingress: Redis
  ingress_security_rules {
    protocol = "6"  # TCP
    source   = "10.0.0.0/16"  # VCN only
    tcp_options {
      min = 6379
      max = 6379
    }
  }
}

# Subnet
resource "oci_core_subnet" "memory_p_subnet" {
  compartment_id    = var.compartment_ocid
  vcn_id            = oci_core_vcn.memory_p_vcn.id
  cidr_block        = "10.0.1.0/24"
  display_name      = "memory-p-subnet"
  route_table_id    = oci_core_route_table.memory_p_rt.id
  security_list_ids = [oci_core_security_list.memory_p_sl.id]
  dns_label         = "memorypsubnet"
}

# Get latest Ubuntu 22.04 ARM image
data "oci_core_images" "ubuntu_arm" {
  compartment_id           = var.compartment_ocid
  operating_system         = "Canonical Ubuntu"
  operating_system_version = "22.04"
  shape                    = "VM.Standard.A1.Flex"
  sort_by                  = "TIMECREATED"
  sort_order               = "DESC"
}

# ARM Instance (Always-Free)
resource "oci_core_instance" "memory_p" {
  compartment_id      = var.compartment_ocid
  availability_domain = var.availability_domain
  display_name        = "memory-p-primary"
  shape               = "VM.Standard.A1.Flex"
  
  shape_config {
    ocpus         = 4
    memory_in_gbs = 24
  }
  
  source_details {
    source_type             = "image"
    source_id               = data.oci_core_images.ubuntu_arm.images[0].id
    boot_volume_size_in_gbs = 200
  }
  
  create_vnic_details {
    subnet_id        = oci_core_subnet.memory_p_subnet.id
    assign_public_ip = true
    display_name     = "memory-p-vnic"
  }
  
  metadata = {
    ssh_authorized_keys = var.ssh_public_key
    user_data = base64encode(templatefile("${path.module}/cloud-init.yaml", {
      memory_p_version = "v2.0.1"
    }))
  }
  
  freeform_tags = {
    Project     = "MEMORY_P"
    Environment = "production"
    ManagedBy   = "Terraform"
    AlwaysFree  = "true"
  }
}

# Outputs
output "instance_public_ip" {
  value = oci_core_instance.memory_p.public_ip
}

output "instance_private_ip" {
  value = oci_core_instance.memory_p.private_ip
}

output "memory_p_url" {
  value = "http://${oci_core_instance.memory_p.public_ip}:4040"
}
```

### cloud-init.yaml

```yaml
#cloud-config

# MEMORY_P automated deployment on Oracle Cloud ARM

hostname: memory-p-primary

package_update: true
package_upgrade: true

packages:
  - git
  - curl
  - build-essential
  - pkg-config
  - libssl-dev
  - ca-certificates
  - iptables-persistent  # For netfilter-persistent

runcmd:
  # Configure firewall for Oracle Cloud
  - iptables -I INPUT 6 -m state --state NEW -p tcp --dport 22 -j ACCEPT
  - iptables -I INPUT 6 -m state --state NEW -p tcp --dport 4040 -j ACCEPT
  - iptables -I INPUT 6 -m state --state NEW -p tcp --dport 6333 -j ACCEPT
  - netfilter-persistent save
  
  # Install Docker
  - curl -fsSL https://get.docker.com -o /tmp/get-docker.sh
  - sh /tmp/get-docker.sh
  - usermod -aG docker ubuntu
  
  # Install Docker Compose
  - curl -L "https://github.com/docker/compose/releases/download/v2.24.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
  - chmod +x /usr/local/bin/docker-compose
  
  # Install Rust (ARM)
  - sudo -u ubuntu curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sudo -u ubuntu sh -s -- -y
  
  # Install Julia (ARM)
  - curl -fsSL https://julialang-s3.julialang.org/bin/linux/aarch64/1.10/julia-1.10.0-linux-aarch64.tar.gz -o /tmp/julia.tar.gz
  - tar -xzf /tmp/julia.tar.gz -C /opt/
  - ln -s /opt/julia-1.10.0/bin/julia /usr/local/bin/julia
  
  # Clone MEMORY_P (fix directory conflict)
  - cd /home/ubuntu
  - sudo -u ubuntu git clone https://github.com/Rigohl/MEMORY_P.git
  - cd MEMORY_P
  - sudo -u ubuntu git checkout ${memory_p_version}
  
  # Configure for ARM
  - cp config/oracle-cloud-arm.toml config/production.toml
  
  # Start services (run as root during cloud-init, group membership applies on next login)
  - cd /home/ubuntu/MEMORY_P
  - docker-compose up -d
  
  # Setup systemd service
  - |
    cat > /etc/systemd/system/memory-p.service <<EOF
    [Unit]
    Description=MEMORY_P MCP Server
    After=docker.service
    Requires=docker.service
    
    [Service]
    Type=simple
    User=ubuntu
    WorkingDirectory=/home/ubuntu/MEMORY_P
    ExecStart=/usr/local/bin/docker-compose up
    ExecStop=/usr/local/bin/docker-compose down
    Restart=always
    RestartSec=10
    
    [Install]
    WantedBy=multi-user.target
    EOF
  - systemctl daemon-reload
  - systemctl enable memory-p.service

write_files:
  - path: /home/ubuntu/MEMORY_P/config/oracle-cloud-arm.toml
    owner: ubuntu:ubuntu
    permissions: '0644'
    content: |
      [server]
      host = "0.0.0.0"
      port = 4040
      
      [search]
      # Lightweight engines for ARM
      tantivy_enabled = true
      tantivy_path = "/app/indices/tantivy"
      qdrant_enabled = true
      qdrant_url = "http://qdrant:6333"
      
      # Disable heavy engines
      faiss_enabled = false
      scann_enabled = false
      
      [julia]
      enabled = true
      threads = 4
      
      [storage]
      # NOTE: Replace <CHANGE_ME> with a strong password or configure this via an environment variable/secret manager.
      postgres_url = "postgresql://memory_p:<CHANGE_ME>@postgres:5432/memory_p"
      redis_url = "redis://redis:6379"
      
      [monitoring]
      metrics_port = 9091
      health_check_interval = 30

final_message: |
  MEMORY_P deployment complete!
  Access at: http://$(hostname -I | awk '{print $1}'):4040
  Check logs: sudo journalctl -u memory-p -f
```

---

## 🔧 Post-Deployment

### Verify Deployment

```bash
# Get instance IP
INSTANCE_IP=$(terraform output -raw instance_public_ip)

# SSH to instance
ssh ubuntu@$INSTANCE_IP

# Check services
docker-compose ps

# Test MEMORY_P
curl http://localhost:4040/health
```

### Setup Monitoring

```bash
# On your local machine
ssh -L 9090:localhost:9090 ubuntu@$INSTANCE_IP

# Access Prometheus at http://localhost:9090
```

---

## 📊 Performance Tuning for ARM

### Optimize Docker

```bash
# Edit /etc/docker/daemon.json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "default-address-pools": [
    {
      "base": "172.17.0.0/16",
      "size": 24
    }
  ]
}

sudo systemctl restart docker
```

### Julia Precompilation

```bash
# Precompile Julia packages (faster startup)
cd /home/ubuntu/MEMORY_P
julia --project=FFI/JULIA_BRAIN -e 'using Pkg; Pkg.precompile()'
```

---

## 💰 Cost Monitoring

```bash
# Check always-free usage
oci limits resource-availability list \
  --compartment-id $COMPARTMENT_ID \
  --service-name compute \
  --limit-name standard-a1-core-count

# Should show: available = 4, used = 4 (free tier max)
```

---

## 🔄 Scaling Options

### Option 1: Add AMD Instance (Also Free)

Oracle provides 2 AMD OCPU always-free as well:

```hcl
# Add to main.tf
resource "oci_core_instance" "memory_p_amd" {
  # ... similar config ...
  shape = "VM.Standard.E2.1.Micro"  # 1 OCPU, 1 GB RAM (free)
}
```

### Option 2: Use Trial Credits ($300)

Add GPU instance during trial:

```bash
terraform apply -var-file="with-gpu.tfvars"
# Uses A10 GPU for FAISS/SCANN testing
```

---

## 🛠️ Troubleshooting

### Issue: Instance won't start

```bash
# Check quota
oci limits resource-availability list --compartment-id $COMPARTMENT_ID

# If out of quota, delete other instances or request increase
```

### Issue: Can't connect via SSH

```bash
# Check security list
oci network security-list get --security-list-id $SL_ID

# Add ingress rule for your IP
oci network security-list update \
  --security-list-id $SL_ID \
  --ingress-security-rules file://rules.json
```

### Issue: Services out of memory

```bash
# Adjust memory limits in docker-compose.yml
services:
  qdrant:
    mem_limit: 8g  # Reduce from 16g
  memory-p:
    mem_limit: 12g  # Reduce from 16g
```

---

## 📚 Next Steps

- [Add monitoring](../playbooks/setup-monitoring.md)
- [Setup backups](../playbooks/automated-backups.md)
- [Configure SSL](../playbooks/ssl-setup.md)
- [Optimize performance](../playbooks/performance-tuning.md)

---

**Cost**: $0/month forever ✅

**Questions?** Open an issue: https://github.com/Rigohl/MEMORY_P/issues/new
