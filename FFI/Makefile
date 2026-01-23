# Makefile para compilación FFI multi-lenguaje
# MEMORY_P v2.0

.PHONY: all julia jax mojo pony zig clean help

# Directorios
SRC_DIR := src
LIB_DIR := lib
BUILD_DIR := build

# Crear directorios si no existen
$(shell mkdir -p $(LIB_DIR) $(BUILD_DIR))

all: help

help:
	@echo "🦀 MEMORY_P FFI Build System v2.0"
	@echo ""
	@echo "Targets disponibles:"
	@echo "  make julia    - Compila motor matemático Julia"
	@echo "  make jax      - Compila extensión ML con JAX"
	@echo "  make mojo     - Compila kernels SIMD Mojo"
	@echo "  make pony     - Compila sistema de actores Pony"
	@echo "  make zig      - Compila FFI bridge Zig"
	@echo "  make all-ffi  - Compila todos los componentes FFI"
	@echo "  make clean    - Limpia archivos compilados"
	@echo "  make test     - Ejecuta tests de integración FFI"
	@echo ""
	@echo "Nota: Asegúrate de tener instalados los compiladores necesarios"

all-ffi: zig julia jax mojo pony
	@echo "✅ Todos los componentes FFI compilados"

# Zig FFI Bridge (core)
zig:
	@echo "🌉 Compilando Zig FFI bridge..."
	@if command -v zig >/dev/null 2>&1; then \
		zig build-lib $(SRC_DIR)/ffi_bridge.zig \
			-dynamic \
			-O ReleaseFast \
			-femit-bin=$(LIB_DIR)/libzig_bridge.so; \
		echo "✅ Zig bridge compilado: $(LIB_DIR)/libzig_bridge.so"; \
	else \
		echo "⚠️  Zig no encontrado. Instalalo desde https://ziglang.org/"; \
		echo "   Creando stub..."; \
		touch $(LIB_DIR)/libzig_bridge.so.stub; \
	fi

# Julia Mathematical Core
julia:
	@echo "🧮 Compilando Julia mathematical core..."
	@if command -v julia >/dev/null 2>&1; then \
		julia --project -e 'using PackageCompiler; \
			create_sysimage(["Optim", "ChaosTools", "DifferentialEquations"]; \
			sysimage_path="$(LIB_DIR)/libjulia_ffi.so", \
			precompile_execution_file="$(SRC_DIR)/julia_math.jl")'; \
		echo "✅ Julia core compilado: $(LIB_DIR)/libjulia_ffi.so"; \
	else \
		echo "⚠️  Julia no encontrado. Instalalo desde https://julialang.org/"; \
		echo "   MEMORY_P funcionará sin optimización matemática"; \
		touch $(LIB_DIR)/libjulia_ffi.so.stub; \
	fi

# JAX ML Inference
jax:
	@echo "🤖 Compilando JAX ML inference..."
	@if command -v python3 >/dev/null 2>&1; then \
		python3 -m pip install --quiet jax sentence-transformers 2>/dev/null || true; \
		python3 -c "import jax; print('✅ JAX disponible:', jax.__version__)"; \
		cp $(SRC_DIR)/jax_inference.py $(LIB_DIR)/jax_inference.py; \
		echo "✅ JAX module listo: $(LIB_DIR)/jax_inference.py"; \
	else \
		echo "⚠️  Python3 no encontrado"; \
		echo "   MEMORY_P funcionará sin embeddings ML"; \
		touch $(LIB_DIR)/libjax_ffi.so.stub; \
	fi

# Mojo SIMD Kernels
mojo:
	@echo "⚡ Compilando Mojo SIMD kernels..."
	@if command -v mojo >/dev/null 2>&1; then \
		mojo build $(SRC_DIR)/kernels.mojo \
			-o $(LIB_DIR)/libmojo_kernels.so \
			--release; \
		echo "✅ Mojo kernels compilados: $(LIB_DIR)/libmojo_kernels.so"; \
	else \
		echo "⚠️  Mojo no encontrado. Instalalo desde https://www.modular.com/mojo"; \
		echo "   MEMORY_P funcionará sin optimizaciones SIMD"; \
		touch $(LIB_DIR)/libmojo_kernels.so.stub; \
	fi

# Pony Actor System
pony:
	@echo "🎭 Compilando Pony actor system..."
	@if command -v ponyc >/dev/null 2>&1; then \
		ponyc $(SRC_DIR)/search_actor.pony \
			-o $(BUILD_DIR) \
			--pic \
			--library; \
		mv $(BUILD_DIR)/libsearch_actor.so $(LIB_DIR)/libpony_actors.so 2>/dev/null || true; \
		echo "✅ Pony actors compilados: $(LIB_DIR)/libpony_actors.so"; \
	else \
		echo "⚠️  Pony no encontrado. Instalalo desde https://www.ponylang.io/"; \
		echo "   MEMORY_P funcionará sin concurrencia basada en actores"; \
		touch $(LIB_DIR)/libpony_actors.so.stub; \
	fi

# Tests de integración FFI
test:
	@echo "🧪 Ejecutando tests de integración FFI..."
	@cd .. && cargo test --test ffi_integration -- --nocapture

# Limpieza
clean:
	@echo "🧹 Limpiando archivos compilados..."
	@rm -rf $(LIB_DIR)/* $(BUILD_DIR)/*
	@mkdir -p $(LIB_DIR) $(BUILD_DIR)
	@echo "✅ Limpieza completada"

# Check de dependencias
check-deps:
	@echo "🔍 Verificando dependencias..."
	@echo -n "Rust:  "; command -v rustc >/dev/null 2>&1 && rustc --version || echo "❌ No encontrado"
	@echo -n "Zig:   "; command -v zig >/dev/null 2>&1 && zig version || echo "❌ No encontrado"
	@echo -n "Julia: "; command -v julia >/dev/null 2>&1 && julia --version || echo "❌ No encontrado"
	@echo -n "Python:"; command -v python3 >/dev/null 2>&1 && python3 --version || echo "❌ No encontrado"
	@echo -n "Mojo:  "; command -v mojo >/dev/null 2>&1 && mojo --version || echo "❌ No encontrado"
	@echo -n "Pony:  "; command -v ponyc >/dev/null 2>&1 && ponyc --version || echo "❌ No encontrado"
