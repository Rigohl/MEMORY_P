# Agent Instructions: MEMORY_P Optimization

## Overview
Este archivo define el comportamiento de Cascade para el proyecto **MEMORY_P**, optimizando el uso de créditos y la eficiencia operativa siguiendo la documentación oficial de Windsurf.

## Core Directives
- **Efficiency First**: Minimizar llamadas a herramientas costosas y reducir las preguntas al usuario. Actúa con autonomía si el contexto es suficiente.
- **Zero Technical Debt**: Prohibido dejar dead code, warnings o errores. Cada cambio debe ser limpio y seguir las mejores prácticas.
- **Rule Enforcement**: Es obligatorio consultar y seguir las reglas definidas en `.windsurf/rules/` antes de cada acción.
- **Language**: Todas las respuestas y explicaciones deben ser en **Español**.
- **Style**: Respuestas concisas, directas y altamente técnicas.

## Autonomy & Analysis
- **Analyze before acting**: Realiza un escaneo profundo de dependencias y lógica antes de editar.
- **Fix on the fly**: Corrige errores detectados proactivamente.
- **Ask Less**: Solo pide aclaración si hay ambigüedad crítica en los requisitos.

## Context Awareness
- El proyecto utiliza una estructura de reglas en `.windsurf/rules/` y flujos en `.windsurf/workflows/`.
- Priorizar el uso de `todo_list` para el seguimiento de tareas complejas.
- Utilizar `@-mentions` para referenciar archivos y conversaciones previas con precisión.

## Automation & Safety
- **Turbo Mode**: Solo para comandos seguros definidos en flujos o explícitamente aprobados.
- **Safety**: Nunca ejecutar comandos destructivos o de red sin confirmación, a menos que estén marcados como seguros en un workflow.
