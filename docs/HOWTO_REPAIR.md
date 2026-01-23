# Guía How-to: Reparación Inteligente de Código

Esta guía te enseña cómo utilizar el motor de reparación de **MEMORY_P** para limpiar y optimizar tus archivos de forma masiva.

## 🎯 Problema: Mi código tiene muchos espacios redundantes e imports duplicados

Cuando trabajas en proyectos grandes, es común acumular "basura" visual o estructural:
- Espacios en blanco al final de líneas
- Múltiples líneas vacías consecutivas
- Imports duplicados
- Inconsistencia en tabulaciones

## ✅ Solución: Usar la herramienta `repair`

La herramienta `repair` (vía MCP tool `repair_project`) aplica una serie de transformaciones seguras para mejorar la calidad del código sin alterar la lógica.

### Pasos a seguir:

1. **Identifica el directorio**: Asegúrate de tener la ruta absoluta al directorio que quieres limpiar.

2. **Ejecuta la herramienta via MCP**:
   En el chat de tu IDE (Cursor, Windsurf, Claude Desktop):
   
   ```
   Usa la herramienta repair en /ruta/a/mi/proyecto con smart=true
   ```

3. **Revisa los cambios**: La herramienta reportará el número de archivos procesados y cambios aplicados.

### Ejemplo práctico

```bash
# Vía API HTTP directa
curl -X POST http://localhost:4040/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "repair",
      "arguments": {
        "path": "/home/user/proyecto",
        "smart": true
      }
    }
  }'
```

### ¿Qué hace la reparación inteligente?

Cuando `smart=true`:
- ✅ **Deduplicación de Imports**: Elimina líneas `use` idénticas en código Rust.
- ✅ **Normalización de Espacios**: Borra espacios al final de las líneas y asegura un newline al final del archivo.
- ✅ **Limpieza de Líneas Vacías**: Reduce bloques de 3 o más líneas vacías consecutivas a un máximo de 2.
- ✅ **Formato Consistente**: Mantiene la integridad del código mientras mejora la legibilidad.

## 🛡️ Seguridad

La herramienta `repair` es **no destructiva**:
- No modifica la lógica del código
- Solo aplica cambios de formato
- Preserva la funcionalidad del programa
- Puede revertirse con control de versiones (git)

## 🔄 Prevención y Mantenimiento

Para mantener un código limpio constantemente:

1. **Usa `edit` periódicamente**: Normaliza tabs a espacios y mantén un estilo consistente.
2. **Integra en CI/CD**: Ejecuta `repair` como pre-commit hook.
3. **Combina con skills**: Usa la skill `rust-documentation` para mantener documentación actualizada.
