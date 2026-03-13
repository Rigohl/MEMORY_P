#!/bin/bash
set -e

# Script para compilar kernels.mojo REAL con Mojo 0.26.1 en WSL2
# Uso: wsl bash scripts/compile_mojo_real.sh

MOJO=/home/kimbe/memory_p_mojo/.pixi/envs/default/bin/mojo
KERNEL_SRC=/mnt/d/REPOSITORIOS/memory_p/brain/mojo/kernels.mojo
OUTPUT_DIR=/tmp/mojo_build
OUTPUT_SO=$OUTPUT_DIR/libmojo_kernels.so

# Crear directorio de salida
mkdir -p $OUTPUT_DIR
cd $OUTPUT_DIR

echo "═══════════════════════════════════════════════════════════════"
echo "  MOJO REAL COMPILATION - MEMORY_P v3.0"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "📦 Mojo Runtime:"
$MOJO --version
echo ""
echo "📂 Fuente: $KERNEL_SRC"
echo "📤 Salida: $OUTPUT_SO"
echo ""
echo "🔨 Compilando con --emit shared-lib..."
echo ""

# Compilación real con Mojo
$MOJO build $KERNEL_SRC \
  --emit shared-lib \
  --target x86_64-unknown-linux-gnu \
  -o libmojo_kernels.so \
  2>&1

echo ""
echo "═══════════════════════════════════════════════════════════════"

if [ -f "$OUTPUT_SO" ]; then
    echo "✅ ÉXITO - libmojo_kernels.so generado"
    echo ""
    echo "📊 Características del binario:"
    ls -lh "$OUTPUT_SO"
    file "$OUTPUT_SO" 2>/dev/null || echo "(info no disponible)"
    echo ""
    echo "🔗 Símbolos exportados:"
    nm -D "$OUTPUT_SO" 2>/dev/null | grep mojo_ || echo "(nm no disponible)"
    echo ""
    echo "📋 Próximos pasos:"
    echo "  1. Verificar que contiene: mojo_dot_product, mojo_cosine_similarity"
    echo "  2. Copiar a: D:\REPOSITORIOS\memory_p\FFI\lib\libmojo_kernels.so"
    echo "  3. Ejecutar: cargo build --lib --release"
else
    echo "❌ FALLO - No se encontró libmojo_kernels.so"
    exit 1
fi
