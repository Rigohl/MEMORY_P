# GitHub Actions Workflows - MEMORY_P

## 📋 Descripción General

Este directorio contiene workflows automatizados de GitHub Actions para el proyecto MEMORY_P v2.0, implementando:

1. **Auto-Gestión y Auto-Push** - Merge automático y gestión de ramas
2. **Auditoría y Reparación Proactiva** - Monitoreo continuo de seguridad y calidad
3. **Integración de Agentes Inteligentes** - Análisis predictivo y optimización basada en IA

---

## 🔄 Workflows Implementados

### 1. CI Pipeline (`ci.yml`)

**Trigger:** Push, Pull Request, Manual  
**Propósito:** Pipeline principal de integración continua

**Jobs:**
- **lint** - Verificación de formato (rustfmt) y linting (clippy)
- **build** - Compilación multi-plataforma (Ubuntu, macOS, Windows)
- **coverage** - Generación de cobertura de tests con tarpaulin
- **benchmark** - Benchmarks de rendimiento (solo en PRs)
- **docs** - Validación de documentación
- **success** - Gate de validación final

**Características:**
- ✅ Caché inteligente de dependencias
- ✅ Tests paralelos en múltiples OS
- ✅ Cobertura de código automática
- ✅ Validación de documentación

---

### 2. Security Audit (`security-audit.yml`)

**Trigger:** Push, Pull Request, Diario (00:00 UTC), Manual  
**Propósito:** Auditoría de seguridad continua

**Jobs:**
- **audit** - Escaneo de vulnerabilidades con cargo-audit
- **cargo-deny** - Verificación de licencias y seguridad
- **secret-scan** - Detección de secretos con Gitleaks
- **dependency-review** - Revisión de dependencias en PRs

**Características:**
- 🔒 Escaneo diario automático
- 🔒 Detección de secretos en commits
- 🔒 Revisión de licencias
- 🔒 Alertas automáticas de vulnerabilidades

---

### 3. Auto-Merge & Auto-Push (`auto-merge.yml`)

**Trigger:** Pull Request events, Check completions, Manual  
**Propósito:** Automatización de merges seguros

**Jobs:**
- **auto-merge** - Merge automático de PRs aprobados con label `auto-merge`
- **auto-push-stable** - Creación automática de PRs a `main` desde `develop`

**Características:**
- 🤖 Merge automático tras validación completa
- 🤖 Requiere aprobaciones de revisores
- 🤖 Verificación de checks obligatorios
- 🤖 Protección de ramas estables

**Uso:**
1. Agregar label `auto-merge` al PR
2. Obtener aprobación de revisor
3. Esperar a que pasen todos los checks
4. El workflow hace merge automáticamente

---

### 4. Dependency Check (`dependency-check.yml`)

**Trigger:** Diario (02:00 UTC), Cambios en Cargo.toml, Manual  
**Propósito:** Monitoreo y actualización de dependencias

**Jobs:**
- **check-dependencies** - Detecta dependencias desactualizadas
- **auto-update-dependencies** - Actualiza versiones patch automáticamente
- **check-critical-vulnerabilities** - Detecta vulnerabilidades críticas

**Características:**
- 📦 Escaneo diario de actualizaciones
- 📦 Auto-actualización de patches seguros
- 📦 Creación automática de PRs con actualizaciones
- 📦 Alertas de vulnerabilidades críticas

---

### 5. Code Quality Analysis (`code-quality.yml`)

**Trigger:** Push, Pull Request, Semanal (lunes 03:00 UTC), Manual  
**Propósito:** Análisis de calidad de código

**Jobs:**
- **complexity-analysis** - Análisis de código unsafe con cargo-geiger
- **code-metrics** - Métricas de código con tokei
- **unused-dependencies** - Detección de dependencias no usadas
- **duplicate-code** - Detección de código duplicado

**Características:**
- 📊 Métricas detalladas de código
- 📊 Detección de código unsafe
- 📊 Identificación de dependencias innecesarias
- 📊 Análisis de duplicación

---

### 6. AI Analysis & Predictive Monitoring (`ai-analysis.yml`)

**Trigger:** Push, Pull Request, Cada 6 horas, Manual  
**Propósito:** Análisis predictivo con agentes inteligentes

**Jobs:**
- **collect-ci-metrics** - Recolección de métricas de CI/CD
- **predict-failures** - Predicción de fallos usando ML
- **agent-chaos-analysis** - Análisis de estabilidad con teoría del caos
- **agent-predictive-optimization** - Optimización predictiva de tendencias
- **summary-report** - Reporte consolidado

**Características:**
- 🤖 Predicción de fallos basada en históricos
- 🤖 Análisis de patrones con ML
- 🤖 Integración con agentes custom de MEMORY_P
- 🤖 Alertas proactivas de alto riesgo

**Integración con Agentes:**
- **memory-p-chaos-analyzer** - Análisis de teoría del caos
- **memory-p-predictive-optimizer** - Optimización matemática
- **memory-p-learning-coordinator** - Aprendizaje continuo

---

### 7. Metrics & Performance Tracking (`metrics.yml`)

**Trigger:** Push, Pull Request, Semanal (domingo), Manual  
**Propósito:** Recolección de métricas de rendimiento

**Jobs:**
- **performance-benchmarks** - Benchmarks con Criterion
- **build-time-tracking** - Medición de tiempos de compilación
- **test-execution-time** - Métricas de ejecución de tests
- **code-quality-metrics** - Estadísticas de código
- **dependency-metrics** - Análisis de dependencias

**Características:**
- ⏱️ Tracking de tiempos de build
- ⏱️ Métricas de tests
- ⏱️ Estadísticas de código
- ⏱️ Dashboard consolidado

---

## 🔐 Branch Protection Rules

### Configuración Recomendada

Para habilitar la protección de ramas en GitHub:

1. **Settings → Branches → Add rule**

2. **Branch name pattern:** `main`

**Configuración:**
```yaml
Protección para rama: main
├─ Require pull request reviews before merging
│  ├─ Required approving reviews: 1
│  └─ Dismiss stale reviews on new commits: ✓
├─ Require status checks to pass
│  ├─ Require branches to be up to date: ✓
│  └─ Status checks:
│     ├─ lint
│     ├─ build (ubuntu-latest)
│     ├─ coverage
│     └─ docs
├─ Require conversation resolution before merging: ✓
├─ Require linear history: ✓
├─ Include administrators: ✓
└─ Allow auto-merge: ✓
```

3. **Branch name pattern:** `develop`

**Configuración:**
```yaml
Protección para rama: develop
├─ Require pull request reviews before merging
│  └─ Required approving reviews: 1
├─ Require status checks to pass
│  ├─ Status checks:
│  │  ├─ lint
│  │  └─ build (ubuntu-latest)
│  └─ Require branches to be up to date: ✓
└─ Allow auto-merge: ✓
```

---

## 🤖 Dependabot Configuration

Configuración en `.github/dependabot.yml`:

**Características:**
- 📦 Actualizaciones semanales de Cargo
- 🔄 Actualizaciones semanales de GitHub Actions
- 🏷️ Auto-labeling de PRs
- 👥 Auto-assignment de reviewers
- 📝 Mensajes de commit estructurados
- 🔢 Límite de 10 PRs abiertos simultáneos

**Grupos de Dependencias:**
- **rust-core** - Dependencias críticas (tokio, axum, serde)
- **development** - Herramientas de desarrollo

---

## 📊 Métricas y Reportes

### Métricas Recolectadas

1. **CI/CD Metrics:**
   - Tasa de fallos
   - Duración promedio de workflows
   - Tendencias de estabilidad

2. **Performance Metrics:**
   - Tiempos de compilación
   - Tiempos de ejecución de tests
   - Benchmarks de rendimiento

3. **Quality Metrics:**
   - Líneas de código
   - Cobertura de tests
   - Código duplicado
   - Dependencias no usadas

4. **Security Metrics:**
   - Vulnerabilidades detectadas
   - Dependencias desactualizadas
   - Código unsafe

### Artifacts Generados

- `ci-metrics.json` - Métricas de CI/CD
- `predictions.json` - Predicciones de fallos
- `tokei.json` - Estadísticas de código
- `benchmark-results.txt` - Resultados de benchmarks

---

## 🚀 Uso y Best Practices

### Para Desarrolladores

1. **Crear Feature Branch:**
   ```bash
   git checkout -b feature/nueva-funcionalidad
   ```

2. **Push y Crear PR:**
   - Los workflows de CI se ejecutan automáticamente
   - Revisar resultados en la pestaña "Actions"
   - Esperar aprobación de revisores

3. **Auto-Merge (opcional):**
   - Agregar label `auto-merge` al PR
   - El merge ocurrirá automáticamente tras aprobación y checks

### Para Maintainers

1. **Revisar PRs de Dependabot:**
   - Revisar cambios en changelog
   - Verificar que tests pasen
   - Aprobar para auto-merge

2. **Monitorear Alertas Predictivas:**
   - Revisar issues creados por AI Analysis
   - Tomar acción en alertas de alto riesgo

3. **Revisar Métricas Semanales:**
   - Dashboard en Actions summary
   - Artifacts con reportes detallados

---

## 🔧 Troubleshooting

### Workflow Failures

**Problema:** Lint failures  
**Solución:**
```bash
cargo fmt --all
cargo clippy --fix --all-targets --all-features
```

**Problema:** Test failures  
**Solución:**
```bash
cargo test --all-features
# Revisar logs específicos
```

**Problema:** Security audit failures  
**Solución:**
```bash
cargo audit
cargo audit fix --dry-run
```

### Auto-Merge No Funciona

**Verificar:**
1. Label `auto-merge` presente
2. PR aprobado por revisor
3. Todos los checks en verde
4. Branch protection rules configuradas
5. Auto-merge habilitado en settings

---

## 📚 Referencias

### Documentación Oficial
- [GitHub Actions](https://docs.github.com/en/actions)
- [Dependabot](https://docs.github.com/en/code-security/dependabot)
- [Branch Protection](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/about-protected-branches)

### Documentación del Proyecto
- [AGENTS.md](../../AGENTS.md) - Documentación de agentes
- [README.md](../../README.md) - Overview del proyecto
- [BLUEPRINT.md](../../BLUEPRINT.md) - Arquitectura técnica

---

## 🤝 Contribuir

Para modificar workflows:

1. Crear branch de feature
2. Modificar workflow en `.github/workflows/`
3. Testear localmente con [act](https://github.com/nektos/act) (opcional)
4. Crear PR con descripción detallada
5. Esperar revisión y aprobación

---

**Última actualización:** Febrero 2026  
**Versión:** 1.0.0  
**Mantenedor:** Rigohl  
**Proyecto:** MEMORY_P v2.0
