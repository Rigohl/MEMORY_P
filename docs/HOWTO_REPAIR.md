# Guía How-to: Reparación Inteligente de Código

Esta guía te enseña cómo utilizar la herramienta **`repair`** de MEMORY_P para limpiar y optimizar archivos de forma masiva y paralela.

## Problema: Código con imports duplicados, espacios inconsistentes y formato irregular

Cuando trabajas en proyectos grandes con múltiples contribuidores, es común acumular:
- Imports duplicados o redundantes
- Espacios en blanco al final de líneas
- Bloques grandes de líneas vacías
- Inconsistencias en EOL (End of Line)

## Solución: Herramienta `repair`

La herramienta `repair` aplica transformaciones seguras en paralelo usando el motor Rayon, mejorando la calidad del código sin alterar la lógica.

### Modo de Uso Básico

**En tu IDE con MCP (Cursor/Windsurf):**
```
Usa repair en ./src para limpiar el código
```

**Con dry_run para previsualizar:**
```
Usa repair en ./src con dry_run=true para ver qué cambios se aplicarían
```

**Para aplicar cambios:**
```
Usa repair en ./src con dry_run=false
```

### ¿Qué hace la reparación?

La tool `repair` ejecuta las siguientes transformaciones en paralelo:

1. **Deduplicación de Imports**
   - Elimina líneas `use` idénticas en archivos Rust
   - Identifica y remueve imports redundantes

2. **Normalización de Espacios**
   - Borra espacios y tabs al final de cada línea
   - Asegura un newline al final del archivo

3. **Limpieza de Líneas Vacías**
   - Reduce bloques de 3+ líneas vacías consecutivas a máximo 2
   - Mejora la legibilidad sin alterar la estructura

4. **EOL Consistency**
   - Normaliza saltos de línea (LF vs CRLF)
   - Mantiene consistencia en todo el proyecto

### Parámetros Avanzados

```json
{
  "name": "repair",
  "arguments": {
    "path": "./src",
    "extension": "rs",
    "dry_run": false
  }
}
```

**Parámetros:**
- `path`: Ruta al directorio raíz del proyecto
- `extension`: Filtrar por extensión (default: "rs")
- `dry_run`: `true` para simular, `false` para aplicar (default: false)

### Ejemplo Práctico

**Antes de `repair`:**
```rust
use std::fs;
use std::io;
use std::fs;  // duplicado
use regex::Regex;



fn main() {   
    println!("Hello");
}


```

**Después de `repair`:**
```rust
use std::fs;
use std::io;
use regex::Regex;

fn main() {
    println!("Hello");
}
```

## Integración con Workflow

Puedes combinar `repair` con otras herramientas:

```
Crea un workflow: Scan → Analyze → Repair en ./src
```

Esto ejecutará un pipeline completo:
1. **Scan**: Lista todos los archivos
2. **Analyze**: Identifica problemas
3. **Repair**: Aplica correcciones automáticas

## Prevención

Para mantener código limpio desde el inicio:
1. Ejecuta `repair` periódicamente (ej: pre-commit hook)
2. Usa `edit mode=replace` para normalizar tabs a espacios
3. Integra con CI/CD para validación automática

## Rendimiento

Gracias al procesamiento paralelo con Rayon:
- **1000 archivos**: ~2-3 segundos
- **10,000 archivos**: ~15-20 segundos
- Escala linealmente con CPU cores disponibles

---
*Siguiente: Consulta [REFERENCE_TOOLS.md](./REFERENCE_TOOLS.md) para ver todas las herramientas disponibles.*
