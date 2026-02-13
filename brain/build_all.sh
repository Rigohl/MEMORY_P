#!/bin/bash
# brain/build_all.sh - Compila todos los lóbulos del cerebro multilingüe

echo "🧠 Compilando lóbulos cerebrales..."

# 1. Zig Bridge (Core Link)
echo "🔗 Compilando Zig Bridge..."
cd zig
zig build-lib ffi_bridge.zig -dynamic -lc
mv libffi_bridge.so ../../target/release/ 2>/dev/null
cd ..

# 2. Julia Core
echo "🧮 Julia core no requiere compilación previa (JIT), pero se verifican dependencias..."
julia -e 'using Pkg; Pkg.add(["Optim", "LinearAlgebra", "Statistics"])'

# 3. Mojo Kernels
echo "🔥 Compilando Mojo Kernels..."
if command -v mojo &> /dev/null; then
    mojo build mojo/kernels.mojo -o ../target/release/libmojo_kernels.so --release
else
    echo "⚠️ Mojo no encontrado, saltando..."
fi

# 4. Pony Actors
echo "🐎 Compilando Pony Actors..."
if command -v ponyc &> /dev/null; then
    ponyc pony -o ../target/release/
else
    echo "⚠️ Ponyc no encontrado, saltando..."
fi

echo "✅ Proceso de construcción de lóbulos finalizado."
