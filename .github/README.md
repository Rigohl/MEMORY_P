# GitHub Copilot Agents & Skills para MEMORY_P

Este directorio contiene la configuración de Agents y Skills personalizados para el proyecto MEMORY_P, optimizados para desarrollo Rust de alto rendimiento con MCP.

## 📁 Estructura

```
.github/
├── agents/                          # Custom Agents
│   ├── memory-p-optimizer.agent.md   # Optimización de rendimiento
│   ├── memory-p-mcp-expert.agent.md  # Especialista en protocolo MCP
│   └── memory-p-refactor.agent.md    # Refactorización de código
│
└── skills/                          # Agent Skills
    ├── rust-parallel-testing/       # Tests con Rayon
    ├── memory-p-analyzer/           # Análisis de código
    ├── mcp-validator/               # Validación MCP
    ├── rust-documentation/          # Documentación Rust
    └── performance-benchmark/       # Benchmarks con Criterion
```

## 🤖 Custom Agents

### 1. memory-p-optimizer
**Propósito**: Maximizar rendimiento del código Rust

**Capacidades**:
- Detecta oportunidades de paralelización con Rayon
- Elimina allocations innecesarias
- Optimiza uso de memoria con mimalloc
- Sugiere mejoras de rendimiento específicas

**Usar cuando**:
- Optimizando módulos críticos
- Mejorando throughput
- Reduciendo latencia de operaciones MCP

### 2. memory-p-mcp-expert
**Propósito**: Implementar y validar protocolo MCP 2024-11-05

**Capacidades**:
- Valida JSON-RPC 2.0 compliance
- Implementa endpoints MCP correctamente
- Verifica schemas de tools
- Optimiza handlers asíncronos

**Usar cuando**:
- Creando nuevos endpoints MCP
- Depurando problemas de compatibilidad
- Validando requests/responses

### 3. memory-p-refactor
**Propósito**: Mejorar calidad del código sin cambiar funcionalidad

**Capacidades**:
- Detecta code smells (long functions, duplicación, etc.)
- Aplica patrones de refactorización
- Simplifica estructuras complejas
- Mantiene cobertura de tests

**Usar cuando**:
- Limpiando código legacy
- Mejorando mantenibilidad
- Reduciendo complejidad

## 🎯 Agent Skills

### 1. rust-parallel-testing
Genera tests unitarios optimizados con Rayon para procesamiento paralelo.

**Templates incluidos**:
- Tests paralelos con `par_iter`
- Tests asíncronos con `tokio::test`
- Tests de error handling

### 2. memory-p-analyzer
Analiza código identificando oportunidades de optimización.

**Detecta**:
- Loops secuenciales que pueden ser paralelos
- Clones innecesarios
- Allocations evitables
- Funciones sin tests

### 3. mcp-validator
Valida endpoints contra especificación MCP 2024-11-05.

**Verifica**:
- JSON-RPC 2.0 compliance
- Schemas de tools
- Error codes correctos
- Compatibilidad con clientes (Cursor, Windsurf, Claude)

### 4. rust-documentation
Genera documentación siguiendo convenciones de rustdoc.

**Incluye**:
- Templates para funciones, structs, módulos
- Secciones estándar (Arguments, Returns, Examples, Errors)
- Links intra-doc
- Ejemplos ejecutables

### 5. performance-benchmark
Crea benchmarks con Criterion para validar optimizaciones.

**Features**:
- Templates para benchmarks básicos y parametrizados
- Comparación con baselines
- Métricas objetivo del proyecto
- Integración con CI/CD

## 🚀 Cómo Usar

### Con GitHub Copilot
Los agents y skills se activan automáticamente en:
- VS Code con GitHub Copilot
- GitHub Copilot CLI
- GitHub Copilot Workspace

### Activación Manual
En chat de Copilot:
```
@workspace Usa el agent memory-p-optimizer para optimizar src/analyzer.rs
```

O con skills:
```
@workspace Aplica la skill rust-parallel-testing para crear tests en este módulo
```

### En Cursor/Windsurf
Los archivos `.agent.md` y `SKILL.md` son reconocidos automáticamente como contexto del proyecto.

## 📊 Métricas de Éxito

### Performance
- **Throughput**: >1000 archivos/segundo en análisis
- **Latencia MCP**: <100ms por operación
- **Memoria**: <500MB para repos de 10K archivos
- **Paralelismo**: Escalar linealmente hasta 8 cores

### Calidad
- **Cobertura**: >80% de líneas cubiertas
- **Warnings**: 0 warnings en `cargo clippy`
- **Documentación**: 100% de APIs públicas documentadas
- **Tests**: Todos pasan en `cargo test`

## 🔧 Mantenimiento

### Actualizar Agent
1. Editar archivo `.agent.md` correspondiente
2. Actualizar version en YAML frontmatter
3. Testear cambios con casos reales
4. Documentar cambios en commit

### Crear Nuevo Skill
1. Crear directorio en `.github/skills/nombre-skill/`
2. Crear `SKILL.md` con YAML frontmatter
3. Incluir templates y ejemplos
4. Añadir a esta documentación
5. Probar con Copilot

## 📚 Referencias

### Documentación Oficial
- [GitHub Copilot Agents](https://docs.github.com/en/copilot/how-tos/use-copilot-agents)
- [Agent Skills](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills)
- [MCP Specification](https://spec.modelcontextprotocol.io)

### Documentación del Proyecto
- [AGENTS.md](../../AGENTS.md) - Guía completa de agents
- [SKILLS.md](../../SKILLS.md) - Guía completa de skills
- [README.md](../../README.md) - Overview del proyecto

## 🤝 Contribuir

Para añadir nuevos agents o skills:
1. Seguir estructura existente
2. Incluir ejemplos prácticos
3. Documentar casos de uso
4. Alinear con métricas del proyecto
5. Testear antes de PR

---

**Última actualización**: Enero 2026  
**Proyecto**: MEMORY_P - Nuclear MCP Toolkit  
**Compatibilidad**: GitHub Copilot, Cursor, Windsurf, Claude Desktop
