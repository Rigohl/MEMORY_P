# Guía How-to: Reparación Inteligente de Código

Esta guía te enseña cómo utilizar el motor de reparación de **MEMORY_P** para limpiar y optimizar tus archivos de forma masiva.

## Problema: Mi código tiene muchos espacios redundantes e imports duplicados
Cuando trabajas en proyectos grandes, es común acumular "basura" visual o estructural.

## Solución: Usar `repair_project`
La herramienta `repair_project` aplica una serie de transformaciones seguras para mejorar la calidad del código sin alterar la lógica.

### Pasos a seguir:

1. **Identifica el directorio**: Asegúrate de tener la ruta absoluta al directorio que quieres limpiar.
2. **Ejecuta la herramienta**:
   En el chat de tu IDE, solicita:
   > "Usa repair_project en C:/MiProyecto/src con smart=true"

### ¿Qué hace la reparación inteligente?
- **Deduplicación de Imports**: Elimina líneas `use` idénticas.
- **Normalización de Espacios**: Borra espacios al final de las líneas y asegura un newline al final del archivo.
- **Limpieza de Líneas Vacías**: Reduce bloques de 3 o más líneas vacías consecutivas a un máximo de 2.

## Prevención
Puedes ejecutar `edit_project` periódicamente para normalizar tabs a espacios y mantener un estilo consistente en todo el equipo.
