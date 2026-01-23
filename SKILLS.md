# GitHub Copilot Agent Skills - Documentación Oficial

> **Actualizado post-merge PR #4**: Documentación de las 5 Skills implementadas en MEMORY_P

## 📋 Índice
- [¿Qué son las Agent Skills?](#qué-son-las-agent-skills)
- [Diferencias: Skills vs Custom Instructions](#diferencias-skills-vs-custom-instructions)
- [Estructura de una Skill](#estructura-de-una-skill)
- [Creación de Skills](#creación-de-skills)
- [Ubicaciones y Alcance](#ubicaciones-y-alcance)
- [Mejores Prácticas](#mejores-prácticas)
- [Ejemplos de Skills](#ejemplos-de-skills)
- [Skills en MEMORY_P](#skills-en-memory_p)
- [Enlaces Oficiales](#enlaces-oficiales)

---

## ¿Qué son las Agent Skills?

Las **Agent Skills** (anteriormente llamadas "skills") son capacidades especializadas que permiten enseñar a GitHub Copilot y otros agents a realizar tareas específicas y repetibles.

### Características Clave
- 📁 **Portables**: Se definen como carpetas con instrucciones, scripts y recursos
- 🔄 **Reutilizables**: Compartibles entre diferentes agents (CLI, Coding Agent, VS Code)
- 🎯 **Contextuales**: Se activan automáticamente cuando el contexto es relevante
- 🛠️ **Ejecutables**: Pueden incluir scripts y ejemplos concretos

**Diferencia Principal**: A diferencia de las custom instructions, las skills pueden incluir código ejecutable y son más estructuradas.

**Documentación oficial**: 
- [About Agent Skills - GitHub Docs](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills)
- [Use Agent Skills in VS Code](https://code.visualstudio.com/docs/copilot/customization/agent-skills)

---

## Diferencias: Skills vs Custom Instructions

| Aspecto | Agent Skills | Custom Instructions |
|---------|--------------|-------------------|
| **Formato** | Carpeta con `SKILL.md` + recursos | Archivo `.md` simple |
| **Contenido** | Instrucciones + scripts + ejemplos | Solo texto/instrucciones |
| **Portabilidad** | Alta (cross-agent) | Media (agent-specific) |
| **Complejidad** | Workflows complejos | Directivas simples |
| **Ejecución** | Puede incluir código | Solo guías textuales |
| **Uso típico** | Automatizaciones, pipelines | Preferencias de estilo |

---

## Estructura de una Skill

### Anatomía de `SKILL.md`

```markdown
---
name: "nombre-skill"
description: "Descripción breve de lo que hace"
version: "1.0.0"
author: "Tu nombre"
tags: ["rust", "testing", "automation"]
---

# Nombre de la Skill

## Descripción
Explicación detallada de la funcionalidad.

## Cuándo Usar
- Contexto 1
- Contexto 2

## Instrucciones
1. Paso 1
2. Paso 2

## Ejemplos
\`\`\`rust
// Código de ejemplo
\`\`\`

## Recursos Adicionales
- archivo-helper.sh
- template.json
```

### Estructura de Carpeta

```
.github/skills/mi-skill/
├── SKILL.md           # Definición principal
├── examples/          # Ejemplos de uso
│   ├── example1.rs
│   └── example2.rs
├── scripts/           # Scripts auxiliares
│   └── helper.sh
└── templates/         # Plantillas
    └── template.toml
```

---

## Creación de Skills

### 1. Skills a Nivel de Proyecto

```bash
# Crear estructura
mkdir -p .github/skills/mi-skill
cd .github/skills/mi-skill

# Crear SKILL.md
cat > SKILL.md << 'EOF'
---
name: "rust-testing"
description: "Genera tests unitarios para módulos Rust"
---

# Rust Testing Skill

Genera tests completos con rayon para procesamiento paralelo.
EOF
```

### 2. Skills a Nivel de Usuario

```bash
# Ubicación global
mkdir -p ~/.copilot/skills/global-skill

# Crear skill
cat > ~/.copilot/skills/global-skill/SKILL.md << 'EOF'
---
name: "git-workflow"
description: "Automatiza flujo git con convenciones"
---

# Git Workflow Skill

Automatiza commits siguiendo Conventional Commits.
EOF
```

### 3. Activación Automática

Las skills se activan cuando:
- El contexto del proyecto coincide con tags
- El usuario invoca explícitamente la skill
- Copilot detecta que la tarea requiere esa skill

---

## Ubicaciones y Alcance

### Prioridad de Carga
1. **Proyecto**: `.github/skills/` (máxima prioridad)
2. **Usuario**: `~/.copilot/skills/` (global)
3. **Organización**: Skills compartidas (si configurado)

### Alcance de Aplicación

| Ubicación | Alcance | Uso Típico |
|-----------|---------|------------|
| `.github/skills/` | Solo este repo | Skills específicas del proyecto |
| `~/.copilot/skills/` | Todos tus proyectos | Skills personales |
| Org/Enterprise | Repos de la org | Estándares corporativos |

---

## Mejores Prácticas

### ✅ DO's

1. **Nombres Descriptivos**
   ```markdown
   ---
   name: "rust-parallel-testing"
   description: "Genera tests con rayon para procesamiento paralelo"
   ---
   ```

2. **Ejemplos Concretos**
   - Incluye código funcional completo
   - Muestra casos de éxito y edge cases
   - Documenta dependencias necesarias

3. **Tags Precisos**
   ```yaml
   tags: ["rust", "testing", "rayon", "parallel", "memory_p"]
   ```

4. **Versionado**
   - Usa semantic versioning
   - Documenta breaking changes
   - Mantén CHANGELOG

5. **Modularidad**
   - Una skill = una responsabilidad
   - Combina skills para workflows complejos

### ❌ DON'Ts

1. No mezclar múltiples funcionalidades en una skill
2. No incluir credenciales o datos sensibles
3. No crear skills muy genéricas (ineficientes)
4. No olvidar la documentación de uso
5. No hardcodear paths absolutos

---

## Ejemplos de Skills

### Ejemplo 1: Rust Testing Skill

```markdown
---
name: "memory-p-testing"
description: "Genera tests para MEMORY_P con rayon y assertions avanzadas"
version: "1.0.0"
tags: ["rust", "testing", "mcp", "parallel"]
---

# MEMORY_P Testing Skill

## Descripción
Genera tests unitarios y de integración para el proyecto MEMORY_P.

## Instrucciones
1. Analiza el módulo a testear
2. Genera tests con `#[test]` y `#[cfg(test)]`
3. Incluye tests paralelos con rayon si aplica
4. Añade assertions específicas de MCP

## Template
\`\`\`rust
#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;

    #[test]
    fn test_parallel_processing() {
        let data: Vec<u32> = (0..1000).collect();
        let results: Vec<_> = data
            .par_iter()
            .map(|x| x * 2)
            .collect();
        
        assert_eq!(results.len(), 1000);
    }
}
\`\`\`
```

### Ejemplo 2: Documentation Skill

```markdown
---
name: "memory-p-docs"
description: "Genera documentación Rust con ejemplos y links a docs oficiales"
version: "1.0.0"
tags: ["rust", "documentation", "rustdoc"]
---

# MEMORY_P Documentation Skill

## Instrucciones
1. Añade doc comments (`///`) a funciones públicas
2. Incluye sección `# Examples` con código funcional
3. Documenta `# Panics`, `# Errors`, `# Safety` si aplica
4. Links a documentación oficial cuando sea relevante

## Ejemplo
\`\`\`rust
/// Procesa datos en paralelo usando rayon.
///
/// # Arguments
/// * `data` - Vector de elementos a procesar
///
/// # Examples
/// \`\`\`
/// let data = vec![1, 2, 3];
/// let results = process_parallel(data);
/// \`\`\`
///
/// # Errors
/// Retorna error si el vector está vacío.
pub fn process_parallel(data: Vec<u32>) -> Result<Vec<u32>, Error> {
    // ...
}
\`\`\`
```

### Ejemplo 3: Git Workflow Skill

```markdown
---
name: "conventional-commits"
description: "Genera commits siguiendo Conventional Commits"
version: "1.0.0"
tags: ["git", "workflow", "commits"]
---

# Conventional Commits Skill

## Formato
\`\`\`
<type>(<scope>): <description>

[optional body]

[optional footer]
\`\`\`

## Types
- `feat`: Nueva funcionalidad
- `fix`: Corrección de bug
- `docs`: Solo documentación
- `refactor`: Refactorización sin cambios funcionales
- `test`: Añade tests
- `chore`: Mantenimiento

## Ejemplo
\`\`\`bash
git commit -m "feat(mcp-api): add parallel processing endpoint

Implements new /mcp/parallel endpoint using rayon.
Improves throughput by 1345%.

Closes #42"
\`\`\`
```

---

## Skills en MEMORY_P v2.0

> **Estado actual**: 9 Skills operativas ubicadas en `.github/skills/` - 5 Core + 4 Advanced

### Skills Core (v1.0)

El proyecto MEMORY_P cuenta con las siguientes skills especializadas fundamentales:

#### 1. **rust-parallel-testing** ([Ver skill](.github/skills/rust-parallel-testing/SKILL.md))
- Analiza código Rust con énfasis en paralelismo
- Detecta oportunidades de optimización con rayon
- Verifica uso correcto de `mimalloc` y `memmap2`

**Tags**: `rust`, `testing`, `rayon`, `parallel`, `memory_p`

#### 2. **memory-p-analyzer** ([Ver skill](.github/skills/memory-p-analyzer/SKILL.md))
- Análisis profundo de código MEMORY_P
- Detecta anti-patterns y optimizaciones potenciales
- Valida cumplimiento de arquitectura

**Tags**: `rust`, `analysis`, `architecture`, `memory_p`

#### 3. **mcp-validator** ([Ver skill](.github/skills/mcp-validator/SKILL.md))
- Valida endpoints MCP contra especificación 2024-11-05
- Genera tests de integración para API
- Verifica compatibilidad con Cursor/Windsurf/Claude

**Tags**: `mcp`, `validation`, `api`, `testing`

#### 4. **rust-documentation** ([Ver skill](.github/skills/rust-documentation/SKILL.md))
- Genera documentación Rust con rustdoc
- Incluye ejemplos ejecutables
- Links a documentación oficial

**Tags**: `rust`, `documentation`, `rustdoc`

#### 5. **performance-benchmark** ([Ver skill](.github/skills/performance-benchmark/SKILL.md))
- Genera benchmarks con `criterion`
- Compara rendimiento pre/post cambios
- Documenta mejoras en formato README

**Tags**: `rust`, `performance`, `benchmarking`, `criterion`

### Skills Avanzadas (v2.0)

Nuevas skills multi-lenguaje para capacidades avanzadas:

#### 6. **rust-mcp-optimization** ([Ver skill](.github/skills/rust-mcp-optimization/SKILL.md))
- Optimización avanzada de servidores MCP
- Análisis de latencia y throughput
- Tuning de parámetros Axum/Tokio
- Integración con Rayon para paralelismo

**Tecnologías**: Rust, Tokio, Axum, Rayon  
**Tags**: `rust`, `mcp`, `optimization`, `performance`, `axum`

**Casos de Uso**:
- Reducir latencia p99 de endpoints MCP
- Aumentar throughput de requests concurrentes
- Optimizar uso de memoria en servidores always-on

#### 7. **julia-chaos-analysis** ([Ver skill](.github/skills/julia-chaos-analysis/SKILL.md))
- Análisis de teoría del caos en codebases
- Cálculo de exponentes de Lyapunov
- Detección de inestabilidad y bifurcaciones
- Predicción de puntos críticos

**Tecnologías**: Julia, DynamicalSystems.jl, DifferentialEquations.jl  
**Tags**: `julia`, `chaos`, `mathematics`, `prediction`, `analysis`

**Casos de Uso**:
- Detectar complejidad creciente en proyectos
- Predecir momentos óptimos para refactorización
- Analizar estabilidad de arquitectura

#### 8. **jax-ml-inference** ([Ver skill](.github/skills/jax-ml-inference/SKILL.md))
- Generación de embeddings semánticos con JAX
- Predicción de intención de usuario
- Reinforcement learning para optimización
- Inference con XLA compilation

**Tecnologías**: JAX, Flax, Optax, Transformers  
**Tags**: `jax`, `ml`, `embeddings`, `inference`, `transformers`

**Casos de Uso**:
- Generar embeddings para búsqueda semántica
- Predecir próximas acciones del usuario
- Optimizar parámetros con RL

#### 9. **hybrid-search-engine** ([Ver skill](.github/skills/hybrid-search-engine/SKILL.md))
- Coordinación de 4 motores de búsqueda (Qdrant, Tantivy, MemoryBank, Híbrido)
- Fusión matemática con Julia (Reciprocal Rank Fusion)
- Optimización de weights por motor
- Benchmarking de precisión y recall

**Tecnologías**: Rust, Julia, Qdrant, Tantivy, Zig  
**Tags**: `search`, `hybrid`, `qdrant`, `tantivy`, `fusion`

**Casos de Uso**:
- Implementar búsqueda multi-modal
- Optimizar fusión de resultados
- Evaluar performance de motores

### Multi-Language FFI Skills

Las skills v2.0 incorporan capacidades FFI para integración multi-lenguaje:

| Skill | Lenguajes | Propósito FFI |
|-------|-----------|---------------|
| **rust-mcp-optimization** | Rust + Mojo | SIMD optimization via FFI |
| **julia-chaos-analysis** | Rust + Julia | Mathematical analysis via C API |
| **jax-ml-inference** | Rust + Python/JAX | ML inference via PyO3 |
| **hybrid-search-engine** | Rust + Julia + Zig | Zero-copy search fusion |

### Uso de las Skills

Las skills se activan automáticamente en contextos relevantes o pueden invocarse manualmente:

```bash
# En chat de GitHub Copilot
@workspace Aplica la skill rust-parallel-testing para este módulo
@workspace Usa julia-chaos-analysis para detectar inestabilidad
@workspace Ejecuta jax-ml-inference para generar embeddings

# En Cursor/Windsurf
# Las skills se cargan automáticamente desde .github/skills/
# Se activan según contexto (lenguaje, archivos abiertos, etc.)

# En CLI
copilot skill rust-mcp-optimization --analyze server.rs
```

### Estructura de Carpetas Completa

```bash
# Estructura actualizada v2.0
.github/skills/
├── rust-parallel-testing/
│   └── SKILL.md
├── memory-p-analyzer/
│   └── SKILL.md
├── mcp-validator/
│   └── SKILL.md
├── rust-documentation/
│   └── SKILL.md
├── performance-benchmark/
│   └── SKILL.md
├── rust-mcp-optimization/         # NEW v2.0
│   ├── SKILL.md
│   ├── scripts/
│   │   └── optimize_mcp.rs
│   ├── references/
│   │   └── axum_best_practices.md
│   └── templates/
│       └── mcp_server_template.rs
├── julia-chaos-analysis/           # NEW v2.0
│   ├── SKILL.md
│   ├── scripts/
│   │   ├── chaos_detector.jl
│   │   └── lyapunov_calculator.jl
│   ├── references/
│   │   └── chaos_theory_guide.md
│   └── templates/
│       └── differential_system.jl
├── jax-ml-inference/               # NEW v2.0
│   ├── SKILL.md
│   ├── scripts/
│   │   ├── embedding_generator.py
│   │   └── intent_predictor.py
│   ├── references/
│   │   └── jax_optimization.md
│   └── templates/
│       └── neural_network.py
└── hybrid-search-engine/           # NEW v2.0
    ├── SKILL.md
    ├── scripts/
    │   ├── fusion_algorithm.rs
    │   └── weight_optimizer.jl
    ├── references/
    │   └── search_theory.md
    └── templates/
        └── search_integration.rs
```

### Compatibilidad Cross-Agent

Las skills de MEMORY_P v2.0 son compatibles con múltiples agents:

| Skill | Coding Agent | CLI Agent | VS Code | Cursor | Windsurf |
|-------|--------------|-----------|---------|--------|----------|
| rust-parallel-testing | ✅ | ✅ | ✅ | ✅ | ✅ |
| memory-p-analyzer | ✅ | ✅ | ✅ | ✅ | ✅ |
| mcp-validator | ✅ | ✅ | ✅ | ✅ | ✅ |
| rust-documentation | ✅ | ✅ | ✅ | ✅ | ✅ |
| performance-benchmark | ✅ | ✅ | ✅ | ✅ | ✅ |
| rust-mcp-optimization | ✅ | ✅ | ✅ | ✅ | ✅ |
| julia-chaos-analysis | ✅ | ✅ | ⚠️ | ✅ | ✅ |
| jax-ml-inference | ✅ | ✅ | ⚠️ | ✅ | ✅ |
| hybrid-search-engine | ✅ | ✅ | ✅ | ✅ | ✅ |

⚠️ = Requiere Julia/Python instalado en el entorno

### Matemáticas en Skills v2.0

Las nuevas skills incorporan capacidades matemáticas avanzadas:

#### Teoría del Caos (julia-chaos-analysis)
```julia
# Ejemplo: Análisis de Lyapunov
using DynamicalSystems

function analyze_codebase_chaos(metrics::Vector{Metric})
    # Reconstruir sistema dinámico
    ds = reconstruct_system(metrics)
    
    # Calcular exponentes de Lyapunov
    λs = lyapunovspectrum(ds, 10000)
    
    # Clasificar estabilidad
    stability = classify_stability(λs)
    
    return (λs, stability)
end
```

#### Fusión Híbrida (hybrid-search-engine)
```julia
# Reciprocal Rank Fusion matemático
function reciprocal_rank_fusion(
    results_lists::Vector{Vector{Result}},
    k::Float64 = 60.0
)::Vector{Result}
    scores = Dict{ResultId, Float64}()
    
    for results in results_lists
        for (rank, result) in enumerate(results)
            scores[result.id] = get(scores, result.id, 0.0) + 1.0 / (k + rank)
        end
    end
    
    return sort(collect(scores), by=x->x[2], rev=true)
end
```

#### ML Inference (jax-ml-inference)
```python
import jax
import jax.numpy as jnp

@jax.jit
def generate_embedding(input_ids: jnp.ndarray) -> jnp.ndarray:
    """JIT-compiled embedding generation"""
    outputs = model(input_ids)
    return jnp.mean(outputs.last_hidden_state, axis=1)
```

---

## Enlaces Oficiales

### Documentación Principal
- [About Agent Skills - GitHub Docs](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills)
- [Use Agent Skills in VS Code](https://code.visualstudio.com/docs/copilot/customization/agent-skills)
- [GitHub Copilot Features](https://docs.github.com/en/copilot/get-started/features)

### Recursos de Aprendizaje
- [Getting Started with GitHub Copilot](https://github.com/skills/getting-started-with-github-copilot) - Curso interactivo
- [GitHub Copilot Changelog](https://github.blog/changelog/2025-12-18-github-copilot-now-supports-agent-skills/)

### Repositorios de Ejemplo
- [awesome-copilot](https://github.com/github/awesome-copilot) - Colección comunitaria
- [anthropics/skills](https://github.com/anthropics/skills) - Skills de referencia

### Documentación Técnica
- [GitHub Copilot Documentation Hub](https://docs.github.com/en/copilot)
- [VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/customization)

---

## Próximos Pasos

### Para Desarrolladores
1. ✅ Leer esta documentación
2. ✅ Revisar ejemplos de skills
3. ✅ Explorar skills implementadas en `.github/skills/`
4. 🔲 Usar skills en desarrollo diario
5. 🔲 Proponer mejoras o nuevas skills

### Para MEMORY_P
1. ✅ Skills implementadas y documentadas
2. ✅ Agents personalizados configurados
3. 🔲 Crear CI/CD para validar skills
4. 🔲 Añadir ejemplos adicionales en cada skill
5. 🔲 Documentar casos de uso avanzados

---

**Última actualización**: Enero 2026  
**Basado en**: Documentación oficial de GitHub Copilot Agent Skills  
**Proyecto**: MEMORY_P - Nuclear MCP Toolkit  
**Compatibilidad**: VS Code, Copilot CLI, Coding Agent, Cursor, Windsurf
