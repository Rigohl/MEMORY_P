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

> **Estado actual**: 5 Skills operativas + 6 Skills multi-lenguaje ubicadas en `.github/skills/`

### Skills Core (Implementadas)

El proyecto MEMORY_P cuenta con las siguientes skills especializadas:

#### 1. **rust-parallel-testing** ([Ver skill](.github/skills/rust-parallel-testing/SKILL.md))
- Analiza código Rust con énfasis en paralelismo
- Detecta oportunidades de optimización con rayon
- Verifica uso correcto de `mimalloc` y `memmap2`
- Genera tests con assertions específicas de concurrencia

#### 2. **memory-p-analyzer** ([Ver skill](.github/skills/memory-p-analyzer/SKILL.md))
- Análisis profundo de código con métricas avanzadas
- Detección de complejidad ciclomática
- Identificación de código inseguro (`unsafe`, `unwrap()`)
- Integración con Julia para análisis de caos

#### 3. **mcp-validator** ([Ver skill](.github/skills/mcp-validator/SKILL.md))
- Valida endpoints MCP contra especificación 2024-11-05
- Genera tests de integración para API
- Verifica compatibilidad con Cursor/Windsurf/Claude Desktop
- Validación de schemas JSON-RPC 2.0

#### 4. **rust-documentation** ([Ver skill](.github/skills/rust-documentation/SKILL.md))
- Genera documentación Rust con ejemplos y benchmarks
- Doc comments con secciones `# Examples`, `# Panics`, `# Errors`
- Links a documentación oficial
- Integración con rustdoc para generación HTML

#### 5. **performance-benchmark** ([Ver skill](.github/skills/performance-benchmark/SKILL.md))
- Genera benchmarks con `criterion`
- Compara rendimiento pre/post cambios
- Análisis estadístico de resultados
- Documenta mejoras en formato README

### Skills Multi-Lenguaje (v2.0)

Nuevas skills que integran el stack multi-lenguaje:

#### 6. **julia-math-optimization** (Nueva)
```markdown
---
name: "julia-math-optimization"
description: "Optimización matemática con Julia + Optim.jl"
version: "2.0.0"
tags: ["julia", "optimization", "chaos", "differential-equations"]
---

# Julia Mathematical Optimization Skill

## Descripción
Aplica algoritmos de optimización matemática usando Julia para mejorar
parámetros de código, detectar patrones caóticos y resolver ecuaciones
diferenciales.

## Cuándo Usar
- Optimización de parámetros numéricos
- Análisis de estabilidad de sistemas
- Detección de comportamiento caótico
- Modelado de sistemas dinámicos

## Instrucciones
1. Identificar parámetros a optimizar
2. Definir función objetivo en Julia
3. Aplicar Optim.jl (LBFGS, NelderMead, etc.)
4. Analizar resultados con ChaosTools.jl
5. Validar mejoras con tests

## Ejemplo
\`\`\`julia
using Optim, ChaosTools

# Optimizar pesos de búsqueda híbrida
function search_quality(weights)
    precision = evaluate_search(weights)
    return -precision  # Minimizar negativo = maximizar
end

result = optimize(search_quality, [0.33, 0.33, 0.34], LBFGS())
optimal_weights = result.minimizer
# [0.41, 0.29, 0.30]
\`\`\`
```

#### 7. **jax-ml-inference** (Nueva)
```markdown
---
name: "jax-ml-inference"
description: "ML inference y embeddings con JAX"
version: "2.0.0"
tags: ["jax", "ml", "embeddings", "inference"]
---

# JAX ML Inference Skill

## Descripción
Genera embeddings semánticos y ejecuta inference de modelos ML
usando JAX con aceleración GPU/TPU.

## Cuándo Usar
- Generación de embeddings para búsqueda vectorial
- Clasificación de código
- Semantic similarity
- Ranking ML-powered

## Instrucciones
1. Cargar modelo (sentence-transformers)
2. Compilar con @jax.jit para máximo rendimiento
3. Procesar en batches para eficiencia
4. Almacenar en Qdrant o PostgreSQL+pgvector

## Ejemplo
\`\`\`python
import jax.numpy as jnp
from sentence_transformers import SentenceTransformer

model = SentenceTransformer('all-MiniLM-L6-v2')

@jax.jit
def batch_embed(texts: list) -> jnp.ndarray:
    return jnp.array([model.encode(t) for t in texts])

embeddings = batch_embed(["query 1", "query 2"])
\`\`\`
```

#### 8. **mojo-simd-kernels** (Nueva)
```markdown
---
name: "mojo-simd-kernels"
description: "Kernels ultra-optimizados con Mojo SIMD"
version: "2.0.0"
tags: ["mojo", "simd", "performance", "vectorization"]
---

# Mojo SIMD Kernels Skill

## Descripción
Optimiza operaciones numéricas críticas usando SIMD intrinsics
de Mojo para rendimiento 35000x superior a Python.

## Cuándo Usar
- Dot products de vectores grandes
- Matrix operations
- Operaciones sobre arrays numéricos
- Hotspots identificados por profiling

## Instrucciones
1. Identificar operación crítica (profile)
2. Implementar en Mojo con @vectorize
3. Compilar con --release
4. Integrar vía FFI desde Rust
5. Benchmark y comparar

## Ejemplo
\`\`\`mojo
@export("mojo_dot_product")
fn dot_product(a: Pointer[Float64], b: Pointer[Float64], n: Int32) -> Float64:
    alias simd_width = simdwidthof[DType.float64]()
    var result: Float64 = 0.0
    
    @parameter
    fn compute[width: Int](i: Int):
        result += (a.simd_load[width](i) * b.simd_load[width](i)).reduce_add()
    
    vectorize[simd_width, compute](int(n))
    return result
\`\`\`
```

#### 9. **zig-ffi-bridge** (Nueva)
```markdown
---
name: "zig-ffi-bridge"
description: "Puente FFI entre Rust y otros lenguajes con Zig"
version: "2.0.0"
tags: ["zig", "ffi", "c-abi", "interop"]
---

# Zig FFI Bridge Skill

## Descripción
Crea puentes FFI seguros y eficientes entre Rust y Julia/Mojo/Pony
usando Zig como capa de abstracción C ABI.

## Cuándo Usar
- Llamar funciones Julia desde Rust
- Integrar kernels Mojo
- Interop con Pony actors
- Cualquier FFI multi-lenguaje

## Instrucciones
1. Definir structs C-compatible (@repr(C) en Rust, extern struct en Zig)
2. Crear funciones export en Zig
3. Compilar con zig build-lib
4. Enlazar desde Rust con extern "C"
5. Validar memory safety

## Ejemplo
\`\`\`zig
// Zig bridge
export fn zig_call_julia(data: [*]f64, len: usize) callconv(.C) [*]f64 {
    const julia_fn = @extern(*fn([*]f64, usize) [*]f64, "julia_optimize");
    return julia_fn(data, len);
}

// Rust side
extern "C" {
    fn zig_call_julia(data: *const f64, len: usize) -> *mut f64;
}
\`\`\`
```

#### 10. **pony-actor-system** (Nueva)
```markdown
---
name: "pony-actor-system"
description: "Concurrencia segura con Pony actors"
version: "2.0.0"
tags: ["pony", "actors", "concurrency", "distributed"]
---

# Pony Actor System Skill

## Descripción
Implementa procesamiento concurrente y distribuido usando el modelo
de actores de Pony, con garantías de seguridad en compile-time.

## Cuándo Usar
- Búsqueda distribuida en múltiples índices
- Procesamiento paralelo sin locks
- Sistemas de mensajería
- Evitar data races

## Instrucciones
1. Definir actors con comportamientos (be)
2. Implementar message passing
3. Usar reference capabilities para seguridad
4. Integrar vía C FFI desde Rust
5. Validar ausencia de data races

## Ejemplo
\`\`\`pony
actor SearchWorker
  let _index: Index val
  
  new create(index: Index val) =>
    _index = index
  
  be search(query: String, respond: SearchResponder) =>
    let results = _index.search(query)
    respond.receive(results)
\`\`\`
```

#### 11. **hybrid-search-fusion** (Nueva)
```markdown
---
name: "hybrid-search-fusion"
description: "Fusión de resultados de múltiples motores de búsqueda"
version: "2.0.0"
tags: ["search", "fusion", "ranking", "hybrid"]
---

# Hybrid Search Fusion Skill

## Descripción
Combina resultados de vector search (Qdrant), full-text (Tantivy)
y heurísticas (MemoryBank) usando Reciprocal Rank Fusion.

## Cuándo Usar
- Búsquedas que requieren precisión máxima
- Combinar semántica + keywords + heurísticas
- Ranking multi-criterio
- Optimización de relevancia

## Instrucciones
1. Ejecutar búsquedas en paralelo (Rayon)
2. Normalizar scores de cada motor
3. Aplicar RRF con pesos configurables
4. Agregar heurísticas matemáticas (Julia)
5. Retornar top-k final

## Ejemplo
\`\`\`rust
async fn hybrid_search(query: &str) -> Vec<Document> {
    let (vector_res, text_res, heuristic_res) = tokio::join!(
        qdrant_search(query),
        tantivy_search(query),
        memorybank_search(query)
    );
    
    rrf_fusion(
        vector_res, 0.4,
        text_res, 0.35,
        heuristic_res, 0.25
    ).top_k(10)
}
\`\`\`
```

### Uso de las Skills v2.0

Las skills se activan automáticamente en contextos relevantes o pueden invocarse manualmente:

```bash
# En chat de GitHub Copilot
@workspace Aplica la skill julia-math-optimization para optimizar
los pesos de búsqueda híbrida

# En Cursor/Windsurf
# Las skills se cargan automáticamente desde .github/skills/

# Invocación explícita
Use the jax-ml-inference skill to generate embeddings for the new documents

# Combinación de skills
Apply rust-parallel-testing and performance-benchmark skills to validate
the new SIMD kernels
```

### Estructura de Carpetas v2.0

```bash
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
├── julia-math-optimization/      # Nueva v2.0
│   ├── SKILL.md
│   └── examples/
│       └── optimize_weights.jl
├── jax-ml-inference/             # Nueva v2.0
│   ├── SKILL.md
│   └── examples/
│       └── batch_embed.py
├── mojo-simd-kernels/            # Nueva v2.0
│   ├── SKILL.md
│   └── examples/
│       └── dot_product.mojo
├── zig-ffi-bridge/               # Nueva v2.0
│   ├── SKILL.md
│   └── examples/
│       └── bridge.zig
├── pony-actor-system/            # Nueva v2.0
│   ├── SKILL.md
│   └── examples/
│       └── search_worker.pony
└── hybrid-search-fusion/         # Nueva v2.0
    ├── SKILL.md
    └── examples/
        └── rrf_fusion.rs
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

### Para MEMORY_P v2.0
1. ✅ Skills core implementadas y documentadas
2. ✅ Agents personalizados configurados
3. ✅ Skills multi-lenguaje especificadas (6 nuevas)
4. 🔲 Implementar skills Julia/JAX/Mojo/Zig/Pony
5. 🔲 Crear CI/CD para validar skills
6. 🔲 Añadir ejemplos adicionales en cada skill
7. 🔲 Documentar casos de uso avanzados
8. 🔲 Integración completa con MemoryBank FFI

---

**Última actualización**: Enero 2026  
**Basado en**: Documentación oficial de GitHub Copilot Agent Skills  
**Proyecto**: MEMORY_P v2.0 - Always-On MCP Toolkit with Multi-Language Brain  
**Compatibilidad**: VS Code, Copilot CLI, Coding Agent, Cursor, Windsurf
