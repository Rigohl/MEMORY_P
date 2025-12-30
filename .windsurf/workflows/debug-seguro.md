---
description: Depuración Avanzada y Resolución de Causa Raíz
---

1. **Auditoría de Errores y Contexto**:
   - Examinar logs de error, trazas de stack y mensajes de consola sin generar ruido adicional.
   - Localizar el archivo y la línea exacta del fallo usando @-mentions.
   - Identificar dependencias o estados que puedan estar influyendo.

2. **Análisis de Causa Raíz (RCA)**:
   - Formular una hipótesis clara sobre por qué ocurre el error.
   - Explicar la lógica fallida de manera amplia para asegurar la comprensión.
   - Proponer una solución técnica sólida antes de editar nada.

3. **Implementación y Verificación**:
   - Aplicar el fix mínimo necesario para no romper otras funcionalidades.
   - Crear un test pequeño o script de verificación para confirmar la resolución.
   - Documentar brevemente la lección aprendida para evitar regresiones.
