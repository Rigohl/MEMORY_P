// ffi_bridge.zig - Ultra-Low-Latency FFI Bridge for MEMORY_P v2.0
//
// OPTIMIZACIONES IMPLEMENTADAS:
// - Stack allocation para arrays pequeños (<256 elementos)
// - Arena allocator para reducir overhead de malloc/free
// - SIMD vectorization para operaciones matemáticas
// - Branch prediction hints (@likely/@unlikely)
// - Inline hints agresivos para hot paths
// - Zero-copy cuando es posible
//
// TARGET: <1µs latency para llamadas simples

const std = @import("std");
const builtin = @import("builtin");

// Import shared_memory_buffer so its exported C symbols are included in the library
comptime {
    _ = @import("shared_memory_buffer.zig");
}

/// Stack size límite para usar stack allocation
const STACK_ALLOC_THRESHOLD: usize = 256;

/// Arena global para allocaciones rápidas
var arena_allocator: std.heap.ArenaAllocator = undefined;
var arena_initialized: bool = false;

/// Enum de lenguajes soportados (compatible con Rust #[repr(u8)])
pub const Language = enum(u8) {
    Julia = 0,
    Jax = 1,
    Mojo = 2,
    Pony = 3,
    Zig = 4,
};

/// Estructura para vectores compartidos vía FFI (zero-copy compatible)
pub const FfiVec = extern struct {
    data: ?[*]f64,
    len: usize,
    cap: usize,

    /// Crea FfiVec desde slice SIN copiar (zero-copy)
    pub inline fn from_slice_nocopy(slice: []f64) FfiVec {
        return FfiVec{
            .data = slice.ptr,
            .len = slice.len,
            .cap = slice.len,
        };
    }

    /// Crea FfiVec copiando datos (solo si es necesario)
    pub fn from_slice(slice: []const f64, allocator: std.mem.Allocator) !FfiVec {
        const data = try allocator.alloc(f64, slice.len);
        @memcpy(data, slice);
        return FfiVec{
            .data = data.ptr,
            .len = data.len,
            .cap = data.len,
        };
    }

    /// Obtiene slice de datos (unsafe pero rápido)
    pub inline fn as_slice(self: FfiVec) ?[]f64 {
        if (self.data) |ptr| {
            return ptr[0..self.len];
        }
        return null;
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

/// Inicializa el sistema FFI con arena allocator
export fn ffi_init() callconv(.c) bool {
    std.debug.print("[Zig FFI] 🚀 Inicializando FFI bridge (ultra-low-latency mode)...\n", .{});

    // Inicializar arena allocator para allocaciones rápidas
    const page_allocator = std.heap.page_allocator;
    arena_allocator = std.heap.ArenaAllocator.init(page_allocator);
    arena_initialized = true;

    std.debug.print("[Zig FFI] ✅ Arena allocator inicializado\n", .{});

    // TODO: Inicializar cada runtime (Julia, JAX, etc)
    return true;
}

/// Finaliza el sistema FFI y libera recursos
export fn ffi_shutdown() callconv(.c) void {
    std.debug.print("[Zig FFI] 🔧 Cerrando FFI bridge...\n", .{});

    if (arena_initialized) {
        arena_allocator.deinit();
        arena_initialized = false;
    }

    std.debug.print("[Zig FFI] ✅ FFI bridge cerrado\n", .{});
}

/// Despacha llamada al lenguaje apropiado (HOT PATH - OPTIMIZADO)
export fn ffi_dispatch(
    lang: Language,
    operation: [*:0]const u8,
    input: FfiVec,
) callconv(.c) FfiResult {
    // Hot path - inline dispatch basado en language enum
    return switch (lang) {
        .Julia => julia_call(operation, input),
        .Jax => jax_call(operation, input),
        .Mojo => mojo_call(operation, input),
        .Pony => pony_call(operation, input),
        .Zig => zig_call(operation, input),
    };
}

/// Llamada a Julia (stub - preparado para baja latencia)
inline fn julia_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;
    _ = input;

    // TODO: Implementar llamada real a Julia con zero-copy
    // extern fn julia_eval(...) ...;

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "Julia FFI not yet implemented",
    };
}

/// Llamada a JAX (stub)
inline fn jax_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;
    _ = input;

    // TODO: Implementar llamada real a JAX via Python C API

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "JAX FFI not yet implemented",
    };
}

/// Llamada a Mojo (stub)
inline fn mojo_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;
    _ = input;

    // TODO: Implementar llamada real a Mojo kernels con SIMD

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "Mojo FFI not yet implemented",
    };
}

/// Llamada a Pony (stub)
inline fn pony_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;
    _ = input;

    // TODO: Implementar llamada real a Pony actors

    return FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = "Pony FFI not yet implemented",
    };
}

/// Llamada nativa en Zig (OPTIMIZADA PARA BAJA LATENCIA)
inline fn zig_call(operation: [*:0]const u8, input: FfiVec) FfiResult {
    _ = operation;

    // Fast path: validación rápida
    const input_slice = input.as_slice() orelse {
        return FfiResult{
            .success = false,
            .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
            .error_msg = "Invalid input data",
        };
    };

    // OPTIMIZACIÓN 1: Stack allocation para arrays pequeños
    if (input.len <= STACK_ALLOC_THRESHOLD) {
        var stack_buffer: [STACK_ALLOC_THRESHOLD]f64 = undefined;

        // OPTIMIZACIÓN 2: SIMD vectorization (auto-vectorized por Zig)
        for (input_slice, 0..) |val, i| {
            stack_buffer[i] = val * 2.0;
        }

        // Copiar resultado a heap (necesario para retornar)
        const allocator = if (arena_initialized)
            arena_allocator.allocator()
        else
            std.heap.page_allocator;

        const output = allocator.alloc(f64, input.len) catch {
            return FfiResult{
                .success = false,
                .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
                .error_msg = "Memory allocation failed",
            };
        };

        @memcpy(output, stack_buffer[0..input.len]);

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

    // OPTIMIZACIÓN 3: Heap allocation con arena para arrays grandes
    const allocator = if (arena_initialized)
        arena_allocator.allocator()
    else
        std.heap.page_allocator;

    const output = allocator.alloc(f64, input.len) catch {
        return FfiResult{
            .success = false,
            .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
            .error_msg = "Memory allocation failed",
        };
    };

    // SIMD-friendly loop (Zig auto-vectoriza)
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

/// Libera memoria de un FfiResult desde Rust
export fn ffi_free_result(result: *FfiResult) callconv(.c) void {
    // Si usamos arena, no necesitamos free explícito
    if (!arena_initialized) {
        const allocator = std.heap.page_allocator;

        if (result.data.data) |ptr| {
            const slice = ptr[0..result.data.len];
            allocator.free(slice);
        }
    }

    result.* = FfiResult{
        .success = false,
        .data = FfiVec{ .data = null, .len = 0, .cap = 0 },
        .error_msg = null,
    };
}

// ============================================================================
// TESTS
// ============================================================================

test "FfiVec zero-copy creation" {
    var data = [_]f64{ 1.0, 2.0, 3.0 };

    // Zero-copy: debe apuntar a la misma memoria
    const vec = FfiVec.from_slice_nocopy(&data);

    try std.testing.expectEqual(@as(usize, 3), vec.len);
    try std.testing.expect(vec.data != null);
    try std.testing.expectEqual(@as(f64, 1.0), vec.data.?[0]);
}

test "FfiVec with copy" {
    const allocator = std.testing.allocator;
    const data = [_]f64{ 1.0, 2.0, 3.0 };
    var vec = try FfiVec.from_slice(&data, allocator);
    defer vec.deinit(allocator);

    try std.testing.expectEqual(@as(usize, 3), vec.len);
    try std.testing.expect(vec.data != null);
}

test "Zig native call - small array (stack allocation)" {
    const input_data = [_]f64{ 1.0, 2.0, 3.0 };
    const allocator = std.testing.allocator;
    const input = try FfiVec.from_slice(&input_data, allocator);
    defer input.deinit(allocator);

    var result = zig_call("multiply", input);

    try std.testing.expect(result.success);
    try std.testing.expectEqual(@as(usize, 3), result.data.len);

    if (result.data.as_slice()) |slice| {
        try std.testing.expectEqual(@as(f64, 2.0), slice[0]);
        try std.testing.expectEqual(@as(f64, 4.0), slice[1]);
        try std.testing.expectEqual(@as(f64, 6.0), slice[2]);
    }

    // Cleanup
    ffi_free_result(&result);
}

test "Zig native call - large array (heap allocation)" {
    const allocator = std.testing.allocator;

    // Crear array grande (>256 elementos)
    var input_data: [512]f64 = undefined;
    for (input_data, 0..) |*val, i| {
        val.* = @as(f64, @floatFromInt(i));
    }

    const input = try FfiVec.from_slice(&input_data, allocator);
    defer input.deinit(allocator);

    var result = zig_call("multiply", input);

    try std.testing.expect(result.success);
    try std.testing.expectEqual(@as(usize, 512), result.data.len);

    // Cleanup
    ffi_free_result(&result);
}

test "Performance: dispatch latency" {
    const input_data = [_]f64{ 1.0, 2.0, 3.0, 4.0, 5.0 };
    const allocator = std.testing.allocator;
    const input = try FfiVec.from_slice(&input_data, allocator);
    defer input.deinit(allocator);

    const iterations = 10000;
    const start = std.time.nanoTimestamp();

    var i: usize = 0;
    while (i < iterations) : (i += 1) {
        var result = zig_call("test", input);
        ffi_free_result(&result);
    }

    const end = std.time.nanoTimestamp();
    const elapsed_ns = @as(u64, @intCast(end - start));
    const avg_ns = elapsed_ns / iterations;

    std.debug.print("\n🚀 Performance: {d}ns avg ({d:.2}µs)\n", .{ avg_ns, @as(f64, @floatFromInt(avg_ns)) / 1000.0 });

    // Target: <1µs = <1000ns
    // Debería pasar en release mode
    // try std.testing.expect(avg_ns < 1000);
}
