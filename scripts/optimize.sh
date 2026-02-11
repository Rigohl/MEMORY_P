#!/bin/bash
# ==========================================
# MEMORY_P v2.0 - Auto-Optimization Script
# ==========================================
# Usage: ./scripts/optimize.sh [--apply]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

APPLY_CHANGES=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --apply) APPLY_CHANGES=true; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  MEMORY_P v2.0 Auto-Optimizer         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# ==========================================
# 1. Analyze Current Resource Usage
# ==========================================
echo -e "${BLUE}[1/5]${NC} Analyzing current resource usage..."

if docker ps | grep -q "memory-p-app"; then
    echo "Getting metrics from running containers..."
    docker stats --no-stream --format "{{.Name}}: CPU={{.CPUPerc}} MEM={{.MemUsage}}"
else
    echo -e "${YELLOW}⚠ Services not running. Start them first.${NC}"
    exit 1
fi

# ==========================================
# 2. Check for Unused Docker Resources
# ==========================================
echo ""
echo -e "${BLUE}[2/5]${NC} Checking for unused resources..."

UNUSED_IMAGES=$(docker images -f "dangling=true" -q | wc -l)
UNUSED_VOLUMES=$(docker volume ls -qf dangling=true | wc -l)
UNUSED_CONTAINERS=$(docker ps -aq -f status=exited | wc -l)

echo "Found:"
echo "  - $UNUSED_IMAGES unused images"
echo "  - $UNUSED_VOLUMES unused volumes"
echo "  - $UNUSED_CONTAINERS stopped containers"

if [ "$APPLY_CHANGES" = true ]; then
    echo ""
    echo "Cleaning up..."
    docker system prune -f > /dev/null 2>&1
    echo -e "${GREEN}✓ Cleanup complete${NC}"
fi

# ==========================================
# 3. Optimize Docker Compose Configuration
# ==========================================
echo ""
echo -e "${BLUE}[3/5]${NC} Analyzing docker-compose configuration..."

# Check if resource limits are set
if grep -q "deploy:" "$PROJECT_ROOT/docker-compose.yml"; then
    echo -e "${GREEN}✓ Resource limits configured${NC}"
else
    echo -e "${YELLOW}⚠ No resource limits found${NC}"
    echo "  Recommendation: Add deploy.resources to services"
fi

# Check health checks
SERVICES_WITH_HEALTH=$(grep -c "healthcheck:" "$PROJECT_ROOT/docker-compose.yml" || echo 0)
echo "  - $SERVICES_WITH_HEALTH services have health checks"

# ==========================================
# 4. Analyze Logs for Optimization Hints
# ==========================================
echo ""
echo -e "${BLUE}[4/5]${NC} Analyzing logs for optimization opportunities..."

# Check for memory warnings
MEM_WARNINGS=$(docker logs memory-p-app 2>&1 | grep -i "memory" | grep -i "warn\|error" | wc -l || echo 0)
if [ "$MEM_WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}⚠ Found $MEM_WARNINGS memory-related warnings${NC}"
    echo "  Recommendation: Increase memory limits"
fi

# Check for connection pool issues
CONN_ISSUES=$(docker logs memory-p-app 2>&1 | grep -i "connection pool" | wc -l || echo 0)
if [ "$CONN_ISSUES" -gt 0 ]; then
    echo -e "${YELLOW}⚠ Found $CONN_ISSUES connection pool warnings${NC}"
    echo "  Recommendation: Increase max_connections in config"
fi

# ==========================================
# 5. Generate Optimization Report
# ==========================================
echo ""
echo -e "${BLUE}[5/5]${NC} Generating optimization report..."

cat > "$PROJECT_ROOT/optimization-report.txt" << EOF
MEMORY_P v2.0 - Optimization Report
Generated: $(date)

═══════════════════════════════════════════════

RESOURCE USAGE:
$(docker stats --no-stream --format "  {{.Name}}: CPU={{.CPUPerc}} MEM={{.MemPerc}}")

RECOMMENDATIONS:
EOF

# Add recommendations based on analysis
if [ "$UNUSED_IMAGES" -gt 5 ]; then
    echo "  - Clean up $UNUSED_IMAGES unused Docker images" >> "$PROJECT_ROOT/optimization-report.txt"
fi

if [ "$MEM_WARNINGS" -gt 0 ]; then
    echo "  - Investigate memory warnings in memory-p-app" >> "$PROJECT_ROOT/optimization-report.txt"
fi

echo -e "${GREEN}✓ Report saved to optimization-report.txt${NC}"

# ==========================================
# Summary
# ==========================================
echo ""
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Optimization Complete                 ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

if [ "$APPLY_CHANGES" = false ]; then
    echo "This was a dry-run. To apply changes, run:"
    echo "  ./scripts/optimize.sh --apply"
fi
