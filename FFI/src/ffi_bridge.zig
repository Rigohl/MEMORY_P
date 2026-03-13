// ffi_bridge.zig - FFI Bridge Layer for MEMORY_P v2.0
//
// Este módulo actúa como dispatcher central para todos los lenguajes FFI.
// Proporciona una interfaz C ABI consistente para Rust.

const std = @import("std");
const builtin = @import("builtin");

/// Enum de lenguajes soportados
pub const Language = enum(u8) {
    Julia = 0,
    Jax = 1,
    Mojo = 2,
    Pony = 3,
    Zig = 4,
};

/// Estructura para vectores compartidos vía FFI
pub const FfiVec = extern struct {
    data: ?[*]f64,
    len: usize,
    cap: usize,

    /// Crea FfiVec desde slice de Zig
    pub fn from_slice(slice: []const f64, allocator: std.mem.Allocator) !FfiVec {
        const data = try allocator.alloc(f64, slice.len);
        std.mem.copy(f64, data, slice);
        return FfiVec{
            .data = data.ptr,
            .len = data.len,
            .cap = data.len,
        };
    }

    /// Libera memoria del FfiVec
    pub fn deinit(self: *FfiVec, allocator: std.mem.Allocator) void {
        if (self.data) |ptr| {
            const slice = ptr[0..self.len];
            allocator.free(slice);
        }
    }
};

/// Estructura para resultados de operaciones FFI
pub const FfiResult = extern struct {
    success: bool,
    data: FfiVec,
    error_msg: ?[*:0]const u8,
};

/// Inicializa el sistema FFI
export fn ffi_init() callconv(.c) bool {
    std.debug.print("[Zig FFI] Inicializando FFI bridge...\n", .{});
    // TODO: Inicializar cada runtime (Julia, JAX, etc)
    return true;
}

/// Finaliza el sistema FFI y libera recursos
export fn ffi_shutdown() callconv(.c) void {
    std.debug.print("[Zig FFI] Cerrando FFI bridge...\n", .{});
    // TODO: Finalizar cada runtime
}

/// Despacha llamada al lenguaje apropiado
export fn ffi_dispatch(
    lang: Language,
    operation: [*:0]const u8,
    input: FfiVec,
) callconv(.c) FfiResult {
    std.debug.print("[Zig FFI] Despachando operación '{s}' a lenguaje {}\n", .{ operation, lang });

    return switch (lang) {
        .Julia => julia_call(operation, input),
        .Jax => jax_call(operation, input),
        .Mojo => mojo_call(operation, input),
        .Pony => pony_call(operation, input),
        .Zig => zig_call(operation, input),
    };
}

/// Llamada a Julia (stub - implementación real en FFI/src/julia_math.jl)
fn julia_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;
    _ = input;
    std.debug.print("[Zig FFI] Julia call - using fallback (real implementation in julia_math.jl via src/ffi/julia.rs)\n", .{});

    // REAL IMPLEMENTATION: julia_math.jl provides:
    // - predict_next_agent_moves(embedding, lookahead)
    // - chaos_analysis(state)
    // - optimize_weights(weights)
    // Fallback: Pure Rust chaos analysis in src/ffi/julia.rs (cfg-gated)

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "Julia: using Rust fallback (see src/ffi/julia.rs for implementation)",
    };
}

/// Llamada a JAX (stub - implementación real en FFI/src/jax_transformer.py)
fn jax_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;
    _ = input;
    std.debug.print("[Zig FFI] JAX call - using fallback (real implementation in jax_transformer.py via src/ffi/jax.rs)\n", .{});

    // REAL IMPLEMENTATION: jax_transformer.py provides:
    // - embed_text(text) -> Vector[Float32]
    // - generate_embeddings(texts) -> Matrix[Float32]
    // Fallback: EmbeddingGenerator in src/ffi/jax.rs with MiniLM-L6-v2 (cfg-gated)

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "JAX: using Rust EmbeddingGenerator fallback (see src/ffi/jax.rs for implementation)",
    };
}

/// Llamada a Mojo (stub - implementación real en FFI/src/kernels.mojo)
fn mojo_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;
    _ = input;
    std.debug.print("[Zig FFI] Mojo call - using fallback (real implementation in kernels.mojo via src/ffi/mojo.rs)\n", .{});

    // REAL IMPLEMENTATION: kernels.mojo provides:
    // - dot_product(a: DType.float32[], b: DType.float32[]) -> float32
    // - cosine_similarity(a, b) -> float32
    // - batch_simd_inference(x) -> Matrix[Float32]
    // Fallback: Pure Rust SIMD in src/ffi/mojo.rs (cfg-gated)

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "Mojo: using Rust libmojo SIMD fallback (see src/ffi/mojo.rs for implementation)",
    };
}

/// Llamada a Pony (stub - implementación real en FFI/src/search_actor.pony)
fn pony_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;
    _ = input;
    std.debug.print("[Zig FFI] Pony call - using fallback (real implementation in search_actor.pony via src/ffi/pony.rs)\n", .{});

    // REAL IMPLEMENTATION: search_actor.pony provides:
    // - distributed_search(queries, num_shards) -> Vec[SearchResult]
    // - pony_actor_pool(num_actors) -> ActorPool
    // Fallback: Tokio actor simulation in src/ffi/pony.rs (cfg-gated)

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "Pony: using Tokio actor fallback (see src/ffi/pony.rs for implementation)",
    };
}

/// Llamada nativa en Zig (ejemplo)
fn zig_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;

    std.debug.print("[Zig FFI] Zig native call\n", .{});

    // Ejemplo: Duplicar cada elemento
    if (input.data) |data_ptr| {
        var allocator = std.heap.page_allocator;
        const output = allocator.alloc(f64, input.len) catch {
            return FfiResult{
                .success = false,
                .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
                .error_msg = "Memory allocation failed",
            };
        };

        const input_slice = data_ptr[0..input.len];
        for (input_slice, 0..) |val, i| {
            output[i] = val * 2.0;
        }

        return FfiResult{
            .success = true,
            .data = FfiVec{
                .data = output.ptr,
                .len = output.len,
                .cap = output.len,
            },
            .error_msg = null,
        };
    }

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "Invalid input data",
    };
}

/// Libera memoria de un FfiResult desde Rust
export fn ffi_free_result(result: *FfiResult) callconv(.c) void {
    var allocator = std.heap.page_allocator;

    if (result.data.data) |ptr| {
        const slice = ptr[0..result.data.len];
        allocator.free(slice);
    }

    result.* = FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = null,
    };
}

// Tests
test "FfiVec creation" {
    var allocator = std.testing.allocator;
    const data = [_]f64{ 1.0, 2.0, 3.0 };
    var vec = try FfiVec.from_slice(&data, allocator);
    defer vec.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 3), vec.len);
    try std.testing.expect(vec.data != null);
}

test "Zig native call" {
    const input_data = [_]f64{ 1.0, 2.0, 3.0 };
    var allocator = std.heap.page_allocator;
    const input = try FfiVec.from_slice(&input_data, allocator);

    var result = zig_call("multiply", input);
    defer ffi_free_result(&result);

    try std.testing.expect(result.success);
    try std.testing.expectEqual(@as(usize, 3), result.data.len);
}
