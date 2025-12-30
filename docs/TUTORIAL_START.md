# MEMORY_P: Tutorial de Inicio Rápido

Bienvenido a **MEMORY_P**, el motor de procesamiento masivo nativo diseñado para integrarse con Cursor, Windsurf y VS Code. En este tutorial, aprenderás a configurar tu entorno y realizar tu primer análisis de código.

## 1. Requisitos Previos
- **Rust Stable**: Instalado mediante `rustup`.
- **Cargo**: El gestor de paquetes de Rust.
- **MCP Host**: Cursor o Windsurf configurados para aceptar servidores MCP HTTP.

## 2. Ejecución del Servidor
Para iniciar el servidor en modo HTTP (puerto 4040 por defecto):
```powershell
cargo run
```

Si prefieres el modo stdio para integración directa con hosts compatibles:
```powershell
cargo run -- --stdio
```

## 3. Tu Primera Herramienta: `scan_project`
Una vez conectado, puedes pedirle a tu asistente (Cursor/Windsurf) que use la herramienta `scan_project`. Esto le dará una visión rápida de los archivos disponibles.

**Ejemplo de solicitud:**
> "Analiza los archivos .rs en la carpeta ./src usando scan_project"

## 4. Análisis Profundo
Usa `analyze_project` para obtener métricas de complejidad y detectar vulnerabilidades potenciales como el uso de `unsafe` o `.unwrap()`.

---
*Próximos pasos: Consulta la [Guía de Reparación Inteligente](./HOWTO_REPAIR.md) para aprender a optimizar tu código automáticamente.*
