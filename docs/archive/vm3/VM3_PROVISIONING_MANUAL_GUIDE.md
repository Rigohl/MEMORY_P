# VM3 Provisioning - Current Status & Manual Steps

**Date**: 2025-01-14  
**Status**: ⏳ BLOCKED - OCI CLI Required Manual Intervention

## Summary

✅ **Completed**:
- OCI file permissions repaired
- VCN and Subnet IDs obtained
- PowerShell provisioning script created (provision_vm3_direct.ps1)
- SSH key loaded and verified
- Cloud-init script prepared

🔴 **Blocked**:
- OCI compute instance launch command failing with exit code 2
- Likely issue: Shape availability, image validation, or parameter format

## Current Infrastructure

```
Tenancy:      ocid1.tenancy.oc1..aaaaaaaalcfbf563uf7mg66seouila2xdnpdwke3wb3jvkmqf7ofdyldieua
Region:       us-ashburn-1
VCN ID:       ocid1.vcn.oc1.iad.amaaaaaaawwr6qoia732u3mrqamt6qvxcdwwytnddaes2iueflx3vtaor6s2q
Subnet ID:    ocid1.subnet.oc1.iad.aaaaaaaasswgwftnm3hqkulwjjjyoxmwmrzqvpm633jsv
AD:           us-ashburn-1-ad-1

Target VM:
  Name:       vm3-qdrant-rust-arm
  Shape:      VM.Standard.A1.Flex
  vCPU:       4 (ARM Ampere)
  Memory:     24 GB
  OS:         Oracle Linux 10 (ARM)
  Cost:       $0/month (Always Free)
```

## Manual Provisioning Steps

### Option 1: Oracle Cloud Console (Easiest)

1. **Login to Oracle Cloud Console**:
   - URL: https://cloud.oracle.com/
   - Tenancy: Your account
   - Region: us-ashburn-1

2. **Create Instance**:
   - Go to: **Compute** → **Instances**
   - Click: **Create Instance**
   - Fill fields:
     ```
     Name:              vm3-qdrant-rust-arm
     Availability Domain: us-ashburn-1-ad-1
     Image:             Oracle Linux 10 (ARM/Ampere)
     Shape:             VM.Standard.A1.Flex
     vCPUs:             4
     Memory:            24 GB
     VCN:               (Select from dropdown)
     Subnet:            (Select from dropdown)
     SSH Key:           (Paste public key from ~/.ssh/id_rsa.pub)
     ```
   - Click: **Create Instance**
   - Wait: 2-3 minutes
   - Copy: Public IP address

3. **Update Code**:
   ```bash
   # Replace 0.0.0.0 with actual public IP in src/oracle_vm_bridge.rs
   sed -i 's/ip: "0\.0\.0\.0"/ip: "YOUR_PUBLIC_IP"/' src/oracle_vm_bridge.rs
   
   # Commit changes
   git add src/oracle_vm_bridge.rs
   git commit -m "chore: VM3 provisioned with public IP YOUR_PUBLIC_IP"
   ```

### Option 2: OCI CLI Manual Commands

```bash
# Set variables
TENANCY="ocid1.tenancy.oc1..aaaaaaaalcfbf563uf7mg66seouila2xdnpdwke3wb3jvkmqf7ofdyldieua"
VCN_ID="ocid1.vcn.oc1.iad.amaaaaaaawwr6qoia732u3mrqamt6qvxcdwwytnddaes2iueflx3vtaor6s2q"
SUBNET_ID="ocid1.subnet.oc1.iad.aaaaaaaasswgwftnm3hqkulwjjjyoxmwmrzqvpm633jsv"
AD="us-ashburn-1-ad-1"
REGION="us-ashburn-1"

# Get latest Oracle Linux 10 ARM image
IMAGE=$(oci compute image list \
  --compartment-id $TENANCY \
  --operating-system "Oracle Linux" \
  --operating-system-version "10" \
  --shape "VM.Standard.A1.Flex" \
  --region $REGION \
  --query 'data[0].id' \
  --raw-output)

echo "Using image: $IMAGE"

# Launch instance
oci compute instance launch \
  --availability-domain "$AD" \
  --compartment-id "$TENANCY" \
  --image-id "$IMAGE" \
  --shape "VM.Standard.A1.Flex" \
  --shape-config "ocpus=4,memory-in-gbs=24" \
  --subnet-id "$SUBNET_ID" \
  --display-name "vm3-qdrant-rust-arm" \
  --region "$REGION" \
  --wait

# Get instance OCID
INSTANCE_OCID=$(oci compute instance list \
  --compartment-id $TENANCY \
  --filter "display-name=vm3-qdrant-rust-arm" \
  --region $REGION \
  --query 'data[0].id' \
  --raw-output)

echo "Instance created: $INSTANCE_OCID"

# Wait for IP assignment (5-10 minutes)
sleep 60

# Get public IP
PUBLIC_IP=$(oci compute instance list-vnic-attachments \
  --instance-id "$INSTANCE_OCID" \
  --compartment-id "$TENANCY" \
  --region $REGION \
  --query 'data[0].vnic-id' \
  --raw-output | \
  xargs -I {} oci network vnic get \
    --vnic-id {} \
    --region $REGION \
    --query 'data."public-ip"' \
    --raw-output)

echo "Public IP: $PUBLIC_IP"
```

### Option 3: Terraform (Infrastructure as Code)

See `terraform/vm3.tf` for complete IaC approach.

## Verification & Testing

### 1. Verify Instance Creation (Oracle Console)
```
Compute → Instances → vm3-qdrant-rust-arm
- State: RUNNING
- Public IP: <assigned>
```

### 2. SSH Access
```bash
ssh opc@<PUBLIC_IP>
# Should succeed without password if SSH key was added

# Check system info
uname -m                    # Should show "aarch64"
nproc                       # Should show "4"
free -h                     # Should show "~24G"
```

### 3. Check Cloud-Init
```bash
# On VM3:
tail -f /var/log/cloud-init-output.log

# Should show Rust, Qdrant, MEMORY_P installation complete
```

### 4. Verify Qdrant (after cloud-init complete)
```bash
# On VM3:
curl http://localhost:6333/health

# Should return:
# {"status":"ok"}
```

## Update MEMORY_P Code

Once VM3 public IP is available:

```bash
# Edit src/oracle_vm_bridge.rs
# Replace line ~72:
# OLD: ip: "0.0.0.0".to_string(),
# NEW: ip: "<PUBLIC_IP>".to_string(),

# Then:
git add src/oracle_vm_bridge.rs
git commit -m "chore: VM3 provisioned ($PUBLIC_IP)"

# Recompile
cargo build --release

# Deploy to VM3
scp target/release/memory_p opc@$PUBLIC_IP:/opt/memory_p
ssh opc@$PUBLIC_IP /opt/memory_p
```

## Troubleshooting

### Shape Not Available
```
Error: Shape VM.Standard.A1.Flex not available in AD

Solution: 
- List available shapes: oci compute shape list --compartment-id $TENANCY
- Use VM.Standard.E2.1.Micro (x86 Flex) instead if A1 unavailable
- Or check different AD
```

### Image Not Found
```
Error: Image not found for Oracle Linux 10 ARM

Solution:
- List available images: oci compute image list --operating-system "Oracle Linux"
- Verify filtering by shape: --shape VM.Standard.A1.Flex
- Or use a standard Ubuntu 22.04 ARM image
```

### Cloud-Init Not Running
```
SSH to VM: ssh opc@<IP>
Check logs: tail -100 /var/log/cloud-init-output.log
           tail -100 /var/log/cloud-init.log

Manually install:
- curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
- yum install -y gcc pkg-config openssl-devel  
- cd /opt && wget https://github.com/qdrant/qdrant/releases/download/v1.7.0/qdrant-x86_64-pc-linux-gnu.zip && unzip qdrant*.zip
```

## Files to Check

- **Code**: `src/oracle_vm_bridge.rs` (contains VM3 IP reference)
- **Docs**: `docs/VM3_CREATION_CONFIGURATION_GUIDE.md` (detailed guide)
- **Scripts**:
  - `scripts/provision_vm3_direct.ps1` (PowerShell provisioning)
  - `scripts/simple_vm3_launch.sh` (Bash launch)
  - `scripts/create_vm3_oci.sh` (original provisioning script)

## Next Steps

1. **Choose provisioning method** (Console, CLI, or Terraform)
2. **Execute provisioning** (expected time: 12-17 minutes total)
3. **Get public IP** from Oracle Console or CLI
4. **Update `src/oracle_vm_bridge.rs`** with actual IP
5. **Test SSH access** to verify
6. **Verify Qdrant health** (curl http://<IP>:6333/health)
7. **Commit code changes** to git
8. **Recompile and deploy** MEMORY_P

## Cost Analysis

```
Current (vm1 + vm2):        $0/month (Always Free)
After VM3 (vm1+vm2+vm3):    $0/month (Always Free)

Specifications:
  vm1: 1 vCPU, 1 GB RAM (x86)     → $0
  vm2: 1 vCPU, 1 GB RAM (x86)     → $0  
  vm3: 4 vCPU, 24 GB RAM (ARM)    → $0
  ─────────────────────────────────
  Total: 6 vCPU, 26 GB RAM        → $0/month
```

---

**Status**: Awaiting manual provisioning via Oracle Console or OCI CLI  
**Time to Complete**: ~15 minutes after starting provisioning  
**Expected Result**: VM3 fully operational with Qdrant, Rust, and MEMORY_P
