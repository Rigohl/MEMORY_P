# 🚀 MEMORY_P v2.0 - DevOps Guide

Guía completa de operaciones DevOps para MEMORY_P v2.0, incluyendo despliegue, monitoreo, troubleshooting y optimización.

---

## 📋 Tabla de Contenidos

1. [Arquitectura DevOps](#arquitectura-devops)
2. [Requisitos del Sistema](#requisitos-del-sistema)
3. [Instalación y Configuración](#instalación-y-configuración)
4. [Despliegue](#despliegue)
5. [Monitoreo y Observabilidad](#monitoreo-y-observabilidad)
6. [CI/CD](#cicd)
7. [Agentes Inteligentes](#agentes-inteligentes)
8. [Troubleshooting](#troubleshooting)
9. [Optimización](#optimización)
10. [Backup y Recuperación](#backup-y-recuperación)

---

## 🏗️ Arquitectura DevOps

```
┌─────────────────────────────────────────────────────────────┐
│                   MEMORY_P DevOps Stack                     │
├─────────────────────────────────────────────────────────────┤
│  CI/CD Layer                                                │
│  ├── GitHub Actions (Build, Test, Deploy)                  │
│  ├── Docker Registry (GHCR)                                │
│  └── Security Scanning (Trivy, CodeQL)                     │
├─────────────────────────────────────────────────────────────┤
│  Application Layer                                          │
│  ├── MEMORY_P (Rust + Julia + JAX)                        │
│  └── 9 Search Engines                                      │
├─────────────────────────────────────────────────────────────┤
│  Data Layer                                                 │
│  ├── PostgreSQL (pgvector)                                 │
│  ├── Redis (Cache)                                         │
│  ├── Qdrant (Vectors)                                      │
│  └── MeiliSearch (Full-text)                               │
├─────────────────────────────────────────────────────────────┤
│  Monitoring Layer                                           │
│  ├── Prometheus (Metrics)                                  │
│  ├── Grafana (Dashboards)                                  │
│  └── Agent Monitor (Auto-alerts)                           │
└─────────────────────────────────────────────────────────────┘
```

---

## 💻 Requisitos del Sistema

### Hardware Mínimo
- **CPU**: 4 cores (8 recomendado)
- **RAM**: 16 GB (32 GB recomendado)
- **Disco**: 100 GB SSD
- **GPU**: Opcional (requerido para FAISS-GPU)

### Software
- Docker 20.10+
- Docker Compose 2.0+
- Git 2.30+
- (Opcional) NVIDIA Docker para GPU

### Sistemas Operativos Soportados
- ✅ Linux (Ubuntu 22.04+, Debian 11+)
- ✅ macOS 12+
- ⚠️  Windows (vía WSL2)

---

## 📦 Instalación y Configuración

### 1. Clonar Repositorio

```bash
git clone https://github.com/Rigohl/MEMORY_P.git
cd MEMORY_P
```

### 2. Configuración de Variables de Entorno

Crear archivo `.env`:

```bash
# MEMORY_P Configuration
RUST_LOG=info
JULIA_ENABLED=true
JULIA_THREADS=4

# Database
POSTGRES_PASSWORD=your_secure_password
REDIS_PASSWORD=your_redis_password

# MeiliSearch
MEILI_MASTER_KEY=your_meili_key

# Resource Limits
MEMORY_P_CPUS=4.0
MEMORY_P_MEMORY=8G
```

### 3. Personalizar Configuraciones

Editar archivos en `config/`:
- `docker.toml` - Configuración de MEMORY_P
- `prometheus.yml` - Targets de monitoreo
- `init.sql` - Schemas de PostgreSQL
- `lnx-node*.toml` - Cluster LNX

---

## 🚀 Despliegue

### Despliegue Local (Desarrollo)

```bash
# Iniciar servicios esenciales
docker-compose up -d postgres redis qdrant meilisearch

# Verificar salud
./scripts/diagnose.sh

# Iniciar MEMORY_P
docker-compose up -d memory-p

# Ver logs
docker-compose logs -f memory-p
```

### Despliegue Completo (Producción)

```bash
# 1. Build de imagen optimizada
docker-compose build --no-cache

# 2. Iniciar todos los servicios
docker-compose up -d

# 3. Esperar a que servicios estén saludables
sleep 30

# 4. Verificar estado
./scripts/diagnose.sh --full

# 5. Acceder a servicios
# - MEMORY_P API: http://localhost:4040
# - Grafana: http://localhost:3000
# - Prometheus: http://localhost:9090
```

### Despliegue con Julia Habilitado

```bash
# Build con Julia
docker-compose build --build-arg JULIA_VERSION=1.10.0

# Iniciar con Julia
docker-compose up -d
```

### Escalado Horizontal

```bash
# Escalar MEMORY_P a 3 instancias
docker-compose up -d --scale memory-p=3

# Verificar
docker-compose ps memory-p
```

---

## 📊 Monitoreo y Observabilidad

### Prometheus

**Endpoint**: http://localhost:9090

Métricas disponibles:
- `search_duration_seconds` - Latencia de búsquedas
- `search_total` - Total de búsquedas
- `memory_usage_bytes` - Uso de memoria
- `cpu_usage_percent` - Uso de CPU

### Grafana

**Endpoint**: http://localhost:3000  
**Usuario**: admin  
**Password**: admin

Dashboards pre-configurados:
1. **MEMORY_P Overview** - Vista general del sistema
2. **Search Engine Performance** - Métricas por motor
3. **Resource Usage** - CPU, memoria, disco

### Health Checks

```bash
# Health check manual
curl http://localhost:4040/health

# Health checks automáticos cada 15s vía Docker
docker inspect memory-p-app | jq '.[0].State.Health'
```

### Logs Centralizados

```bash
# Ver logs en tiempo real
docker-compose logs -f

# Logs de servicio específico
docker-compose logs -f memory-p

# Últimas 100 líneas
docker-compose logs --tail=100 memory-p

# Filtrar por error
docker-compose logs memory-p | grep -i error
```

---

## 🔄 CI/CD

### Workflows Disponibles

#### 1. **CI - Build and Test** (`.github/workflows/ci.yml`)

Ejecuta en cada push:
- ✅ Lint (rustfmt, clippy)
- ✅ Build (Ubuntu, macOS)
- ✅ Tests unitarios e integración
- ✅ Code coverage

#### 2. **Docker** (`.github/workflows/docker.yml`)

Build y push de imágenes:
- 🐳 Build multi-stage
- 🏷️  Tag semántico
- 🔍 Vulnerability scan
- 📤 Push a GHCR

#### 3. **Security** (`.github/workflows/security.yml`)

Escaneo diario:
- 🔐 Cargo audit
- 🔍 CodeQL analysis
- 🕵️  Secret scanning
- 🛡️  Container security

#### 4. **Dependencies** (`.github/workflows/dependencies.yml`)

Actualización semanal:
- 📦 Actualizar Rust crates
- 🐳 Actualizar imágenes Docker
- 🐍 Actualizar Python packages
- 🔄 Auto-crear PRs

### Triggers

```yaml
# Manual
workflow_dispatch

# Automático
push:
  branches: [main, develop]
pull_request:
  branches: [main]
schedule:
  - cron: '0 2 * * *'  # Daily 2 AM
```

---

## 🤖 Agentes Inteligentes

MEMORY_P incluye 3 agentes DevOps especializados:

### 1. DevOps Monitor Agent

**Ubicación**: `.github/agents/devops-monitor-agent.agent.md`

**Capacidades**:
- ✅ Monitoreo continuo de salud
- 🔍 Detección de anomalías
- 🔧 Acciones correctivas automáticas
- 📢 Alertas inteligentes

**Uso**:
```bash
@devops-monitor-agent analiza el estado actual del sistema 
y genera un reporte de salud
```

### 2. Docker Optimizer Agent

**Ubicación**: `.github/agents/docker-optimizer-agent.agent.md`

**Capacidades**:
- 🎯 Optimización de Dockerfiles
- 📉 Reducción de tamaño de imágenes
- ⚡ Mejora de performance
- 🔧 Tuning de recursos

**Uso**:
```bash
@docker-optimizer-agent optimiza la configuración de Docker Compose
```

### 3. Dependency Manager Agent

**Ubicación**: `.github/agents/dependency-manager-agent.agent.md`

**Capacidades**:
- 📦 Gestión de dependencias
- 🔒 Detección de vulnerabilidades
- 🔄 Resolución de conflictos
- 📊 Análisis de impacto

**Uso**:
```bash
@dependency-manager-agent audita las dependencias actuales
y propone actualizaciones seguras
```

---

## 🔧 Troubleshooting

### Problema: Servicios no inician

**Síntomas**: `docker-compose up` falla

**Diagnóstico**:
```bash
# Ver logs detallados
docker-compose logs

# Verificar configuración
docker-compose config
```

**Soluciones**:
1. Verificar puertos disponibles: `netstat -tuln | grep <port>`
2. Limpiar volúmenes: `docker-compose down -v`
3. Rebuild: `docker-compose build --no-cache`

### Problema: Alta latencia en búsquedas

**Síntomas**: P99 > 500ms

**Diagnóstico**:
```bash
# Ver métricas de Prometheus
curl 'http://localhost:9090/api/v1/query?query=search_duration_seconds'

# Verificar uso de CPU/memoria
docker stats
```

**Soluciones**:
1. Aumentar workers en `config/docker.toml`
2. Incrementar límites de recursos en docker-compose
3. Habilitar caché de Redis
4. Optimizar índices

### Problema: PostgreSQL no responde

**Síntomas**: Connection refused

**Diagnóstico**:
```bash
# Verificar estado
docker-compose ps postgres

# Ver logs
docker-compose logs postgres

# Test de conexión
docker exec -it postgres pg_isready -U memory_p
```

**Soluciones**:
1. Restart: `docker-compose restart postgres`
2. Verificar credenciales en `.env`
3. Verificar espacio en disco
4. Revisar init.sql por errores

### Problema: Qdrant vector search falla

**Síntomas**: 500 errors en /search

**Diagnóstico**:
```bash
# Health check
curl http://localhost:6333/health

# Ver colecciones
curl http://localhost:6333/collections
```

**Soluciones**:
1. Verificar memoria disponible
2. Recrear colección
3. Ajustar `QDRANT__STORAGE__OPTIMIZERS__INDEXING_THRESHOLD`

---

## ⚡ Optimización

### Auto-Optimización

```bash
# Ejecutar análisis
./scripts/optimize.sh

# Aplicar optimizaciones
./scripts/optimize.sh --apply
```

### Optimizaciones Manuales

#### 1. PostgreSQL

```sql
-- Vacuum y analyze
VACUUM ANALYZE;

-- Reindex
REINDEX DATABASE memory_p;

-- Tune parameters
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
```

#### 2. Redis

```bash
# Verificar fragmentación
docker exec redis redis-cli INFO memory

# Flush cache si es necesario
docker exec redis redis-cli FLUSHALL
```

#### 3. Docker

```bash
# Limpiar recursos no utilizados
docker system prune -af --volumes

# Verificar tamaño de imágenes
docker images | grep memory-p

# Optimize layer cache
docker build --cache-from memory-p:latest .
```

---

## 💾 Backup y Recuperación

### Backup Manual

```bash
# PostgreSQL
docker exec postgres pg_dump -U memory_p memory_p > backup.sql

# Qdrant (exportar colección)
curl -X POST http://localhost:6333/collections/memory_p_vectors/snapshots

# Redis
docker exec redis redis-cli SAVE

# Volúmenes
docker run --rm -v postgres_data:/data -v $(pwd):/backup \
  ubuntu tar czf /backup/postgres_backup.tar.gz /data
```

### Restauración

```bash
# PostgreSQL
cat backup.sql | docker exec -i postgres psql -U memory_p memory_p

# Redis
docker exec -i redis redis-cli --pipe < dump.rdb
```

### Backup Automatizado (Cron)

```bash
# Agregar a crontab
0 2 * * * /path/to/MEMORY_P/scripts/backup.sh
```

---

## 📚 Referencias

- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Prometheus Monitoring](https://prometheus.io/docs/introduction/overview/)
- [GitHub Actions](https://docs.github.com/en/actions)
- [MEMORY_P Agents](./AGENTS.md)
- [MEMORY_P Skills](./SKILLS.md)

---

## 🆘 Soporte

Para problemas o preguntas:
1. Revisar esta guía
2. Ejecutar `./scripts/diagnose.sh --full`
3. Consultar logs: `docker-compose logs`
4. Abrir issue en GitHub
5. Contactar al equipo DevOps

---

**Última actualización**: Febrero 2026  
**Versión**: 2.0  
**Mantenido por**: MEMORY_P Team
