# GitHub Actions - Quick Setup Guide

## 🚀 Inicio Rápido

### 1. Primer Setup (5 minutos)

#### Paso 1: Verificar que los workflows están presentes
```bash
ls -la .github/workflows/
# Deberías ver 7 archivos .yml
```

#### Paso 2: Habilitar GitHub Actions
1. Ir a `Settings > Actions > General`
2. En "Actions permissions", seleccionar:
   - ✅ "Allow all actions and reusable workflows"
3. En "Workflow permissions", seleccionar:
   - ✅ "Read and write permissions"
   - ✅ "Allow GitHub Actions to create and approve pull requests"
4. Click "Save"

#### Paso 3: Configurar Branch Protection (IMPORTANTE)
Ver guía completa en [BRANCH_PROTECTION.md](./BRANCH_PROTECTION.md)

**Configuración mínima para `main`:**
1. Ir a `Settings > Branches > Add rule`
2. Branch name pattern: `main`
3. Habilitar:
   - ✅ Require a pull request before merging (1 approval)
   - ✅ Require status checks to pass before merging
   - ✅ Require conversation resolution before merging
   - ✅ Include administrators
   - ✅ Allow auto-merge

**Status checks a requerir:**
- `lint`
- `build (ubuntu-latest)`
- `coverage`
- `docs`

#### Paso 4: Habilitar Dependabot
1. Ir a `Settings > Code security and analysis`
2. Habilitar:
   - ✅ Dependabot alerts
   - ✅ Dependabot security updates
3. El archivo `.github/dependabot.yml` ya está configurado

#### Paso 5: Configurar Secrets (Opcional)
Si quieres cobertura de código:
1. Ir a `Settings > Secrets and variables > Actions`
2. Añadir: `CODECOV_TOKEN` (obtener de codecov.io)

---

## 📋 Workflows Disponibles

### ✅ Ejecución Automática

| Workflow | Trigger | Propósito |
|----------|---------|-----------|
| **CI Pipeline** | Push, PR | Build, tests, linting |
| **Security Audit** | Push, PR, Diario | Auditoría de seguridad |
| **Auto-Merge** | PR events | Merge automático |
| **Dependency Check** | Diario, Cambios | Monitoreo de deps |
| **Code Quality** | Push, PR, Semanal | Análisis de calidad |
| **AI Analysis** | Cada 6h, Push, PR | Predicción de fallos |
| **Metrics** | Push, PR, Semanal | Métricas de rendimiento |

### 🎯 Ejecución Manual

Todos los workflows pueden ejecutarse manualmente:
1. Ir a `Actions` tab
2. Seleccionar workflow
3. Click "Run workflow"
4. Seleccionar rama
5. Click "Run workflow" verde

---

## 🏷️ Labels para Auto-Merge

### Crear Labels Requeridos

```bash
# Via GitHub CLI
gh label create "auto-merge" --color "0E8A16" --description "Enable automatic merge after approval"
gh label create "automated" --color "1D76DB" --description "Automated by GitHub Actions"
gh label create "dependencies" --color "0366D6" --description "Dependency updates"
```

O manualmente en GitHub:
1. Ir a `Issues > Labels`
2. Crear:
   - `auto-merge` (verde) - Para PRs que se deben mergear automáticamente
   - `automated` (azul) - PRs creados automáticamente
   - `dependencies` (azul) - Actualizaciones de dependencias

### Uso de Auto-Merge

```bash
# Crear PR con auto-merge
gh pr create --title "Mi feature" --body "Descripción" --label "auto-merge"

# O añadir label a PR existente
gh pr edit 123 --add-label "auto-merge"
```

---

## 🔍 Verificar que Funciona

### Test 1: CI Pipeline

```bash
# Crear rama de prueba
git checkout -b test/ci-verification
echo "# Test" >> test_file.md
git add test_file.md
git commit -m "test: verify CI pipeline"
git push origin test/ci-verification

# Crear PR y verificar que se ejecutan:
# - lint
# - build (3 OS)
# - coverage
# - docs
```

### Test 2: Security Audit

```bash
# Debería ejecutarse automáticamente en el PR anterior
# Verificar en Actions tab que "Security Audit" se ejecutó
```

### Test 3: Auto-Merge

```bash
# En el PR de prueba:
# 1. Añadir label "auto-merge"
# 2. Aprobar el PR (como otro usuario o admin)
# 3. Esperar que todos los checks pasen
# 4. El PR debería mergearse automáticamente
```

---

## 📊 Monitorear Workflows

### Ver Ejecuciones Recientes

```bash
# Via GitHub CLI
gh run list --limit 10

# Ver detalles de una ejecución
gh run view <run-id>

# Ver logs
gh run view <run-id> --log
```

### Via Web UI

1. Ir a `Actions` tab
2. Ver:
   - Ejecuciones recientes
   - Estado de cada job
   - Logs detallados
   - Artifacts generados

---

## 🚨 Troubleshooting Rápido

### Problema: Workflow no se ejecuta

**Verificar:**
```bash
# 1. Actions habilitado en Settings
# 2. Trigger correcto (push a rama correcta)
# 3. Syntax YAML válida

# Validar YAML localmente
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"
```

### Problema: Status check no aparece en PR

**Solución:**
1. Hacer un commit adicional para re-trigger
2. Verificar que el workflow tiene permiso de lectura/escritura
3. Verificar que la rama está actualizada

### Problema: Auto-merge no funciona

**Checklist:**
```
☑ Label "auto-merge" presente
☑ PR aprobado por revisor
☑ Todos los checks en verde
☑ Conversaciones resueltas
☑ Auto-merge habilitado en Settings
☑ Branch protection rules configuradas
```

### Problema: Dependabot no crea PRs

**Verificar:**
```bash
# 1. Dependabot habilitado en Settings
# 2. Archivo .github/dependabot.yml presente
# 3. Syntax correcta en dependabot.yml

# Ver logs de Dependabot
# Settings > Code security > Dependabot > View logs
```

---

## 🎯 Best Practices

### Para Desarrolladores

```bash
# ✅ HACER
- Crear feature branches desde develop
- Usar commits descriptivos (conventional commits)
- Añadir tests para nuevas features
- Resolver todos los comments en PRs
- Usar auto-merge para PRs triviales

# ❌ NO HACER
- Push directo a main o develop
- Ignorar failures de CI
- Mergear con conversaciones sin resolver
- Crear PRs gigantes (>500 líneas)
```

### Para Reviewers

```bash
# ✅ HACER
- Revisar en <24 horas
- Probar cambios localmente si es crítico
- Dar feedback constructivo
- Aprobar solo cuando esté listo

# ❌ NO HACER
- Aprobar "a ciegas"
- Ignorar warnings de seguridad
- Saltarse verificación de tests
```

---

## 📈 Métricas a Monitorear

### KPIs Semanales

```yaml
CI/CD Health:
  - Failure Rate: < 10%
  - Average Build Time: < 5 min
  - PR Merge Time: < 24h
  - Test Coverage: > 80%

Security:
  - Critical Vulnerabilities: 0
  - High Vulnerabilities: < 5
  - Outdated Dependencies: < 20%

Quality:
  - Code Duplication: < 5%
  - Unused Dependencies: 0
  - Clippy Warnings: 0
```

### Dashboards

Revisar semanalmente:
1. `Actions` - Success rate de workflows
2. `Security` - Alertas de Dependabot
3. `Insights > Pulse` - Actividad general
4. Artifacts de AI Analysis - Predicciones

---

## 🔗 Enlaces Útiles

### Documentación del Proyecto
- [workflows/README.md](./workflows/README.md) - Guía completa de workflows
- [BRANCH_PROTECTION.md](./BRANCH_PROTECTION.md) - Configuración de protección
- [dependabot.yml](./dependabot.yml) - Configuración de Dependabot

### Documentación Oficial
- [GitHub Actions Docs](https://docs.github.com/en/actions)
- [Workflow Syntax](https://docs.github.com/en/actions/reference/workflow-syntax-for-github-actions)
- [GitHub CLI](https://cli.github.com/manual/)

### Herramientas
- [act](https://github.com/nektos/act) - Test workflows localmente
- [actionlint](https://github.com/rhysd/actionlint) - Lint para workflows

---

## ✅ Checklist de Verificación Post-Setup

```
Configuración Inicial:
☐ GitHub Actions habilitado
☐ Workflow permissions configurados
☐ Branch protection para main configurado
☐ Labels creados (auto-merge, automated, dependencies)
☐ Dependabot habilitado

Verificación Funcional:
☐ CI Pipeline ejecutado exitosamente
☐ Security Audit ejecutado
☐ Auto-merge funciona con label
☐ Dependabot configurado y funcionando
☐ Métricas recolectándose

Documentación:
☐ Equipo entrenado en uso de auto-merge
☐ Process de revisión de PRs definido
☐ Responsables de seguridad asignados
☐ KPIs establecidos y monitoreados
```

---

**¿Necesitas ayuda?** Crea un issue con label `github-actions` o consulta la documentación completa en los archivos mencionados.

**Última actualización:** Febrero 2026  
**Versión:** 1.0.0
