#!/bin/bash
# ==========================================
# MEMORY_P v2.0 - System Diagnostic Script
# ==========================================
# Usage: ./scripts/diagnose.sh [--full] [--json]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
FULL_CHECK=false
JSON_OUTPUT=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --full) FULL_CHECK=true; shift ;;
        --json) JSON_OUTPUT=true; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  MEMORY_P v2.0 System Diagnostics     ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# ==========================================
# 1. Docker Environment Check
# ==========================================
echo -e "${BLUE}[1/8]${NC} Checking Docker environment..."

if ! command -v docker &> /dev/null; then
    echo -e "${RED}✗ Docker not installed${NC}"
    exit 1
else
    DOCKER_VERSION=$(docker --version | cut -d ' ' -f3 | cut -d ',' -f1)
    echo -e "${GREEN}✓ Docker installed${NC} (v${DOCKER_VERSION})"
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}✗ Docker Compose not installed${NC}"
    exit 1
else
    COMPOSE_VERSION=$(docker-compose --version | cut -d ' ' -f4 | cut -d ',' -f1)
    echo -e "${GREEN}✓ Docker Compose installed${NC} (v${COMPOSE_VERSION})"
fi

# ==========================================
# 2. Container Status Check
# ==========================================
echo ""
echo -e "${BLUE}[2/8]${NC} Checking container status..."

EXPECTED_SERVICES=("memory-p-app" "postgres" "redis" "qdrant" "meilisearch")
RUNNING_COUNT=0

for service in "${EXPECTED_SERVICES[@]}"; do
    if docker ps --format '{{.Names}}' | grep -q "^${service}$"; then
        STATUS=$(docker inspect --format='{{.State.Status}}' "$service" 2>/dev/null)
        echo -e "${GREEN}✓ ${service}${NC} - running"
        ((RUNNING_COUNT++))
    else
        echo -e "${RED}✗ ${service}${NC} - not running"
    fi
done

echo ""
echo -e "Services: ${RUNNING_COUNT}/${#EXPECTED_SERVICES[@]} running"

# ==========================================
# 3. Configuration Validation
# ==========================================
echo ""
echo -e "${BLUE}[3/8]${NC} Validating configurations..."

# Docker Compose
if docker-compose -f "$PROJECT_ROOT/docker-compose.yml" config > /dev/null 2>&1; then
    echo -e "${GREEN}✓ docker-compose.yml${NC} - valid"
else
    echo -e "${RED}✗ docker-compose.yml${NC} - invalid syntax"
fi

# Config files
CONFIG_FILES=("docker.toml" "prometheus.yml" "init.sql")
for file in "${CONFIG_FILES[@]}"; do
    if [ -f "$PROJECT_ROOT/config/$file" ]; then
        echo -e "${GREEN}✓ config/$file${NC} - exists"
    else
        echo -e "${RED}✗ config/$file${NC} - missing"
    fi
done

# ==========================================
# Summary
# ==========================================
echo ""
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Diagnostic Complete                   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

if [ "$RUNNING_COUNT" -eq "${#EXPECTED_SERVICES[@]}" ]; then
    echo -e "${GREEN}✓ All services operational${NC}"
else
    echo -e "${YELLOW}⚠ Some services need attention${NC}"
fi
