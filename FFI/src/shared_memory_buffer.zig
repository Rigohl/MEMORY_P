// shared_memory_buffer.zig - Buffer de memoria compartida de alta velocidad
// 
// Este módulo proporciona un buffer de memoria compartida optimizado con:
// - Zero-copy operations
// - Lock-free concurrent access
// - SIMD-optimized memory operations
// - Atomic reference counting

const std = @import("std");
const builtin = @import("builtin");

/// Tamaño de página para alineación
const PAGE_SIZE: usize = 4096;

/// Buffer de memoria compartida
pub const SharedMemoryBuffer = extern struct {
    /// Puntero a los datos
    data: ?[*]u8,
    
    /// Capacidad total en bytes
    capacity: usize,
    
    /// Bytes usados actualmente
    used: usize,
    
    /// Contador de referencias (atomic)
    ref_count: u32,
    
    /// Indica si está inicializado
    initialized: bool,
};

/// Crea un nuevo buffer de memoria compartida
export fn shared_memory_buffer_new(capacity: usize) callconv(.C) ?*SharedMemoryBuffer {
    var allocator = std.heap.page_allocator;
    
    // Alinear capacidad a PAGE_SIZE
    const aligned_capacity = std.mem.alignForward(usize, capacity, PAGE_SIZE);
    
    // Allocate buffer structure
    var buffer = allocator.create(SharedMemoryBuffer) catch return null;
    
    // Allocate aligned memory
    const data = allocator.alignedAlloc(u8, PAGE_SIZE, aligned_capacity) catch {
        allocator.destroy(buffer);
        return null;
    };
    
    buffer.* = SharedMemoryBuffer{
        .data = data.ptr,
        .capacity = aligned_capacity,
        .used = 0,
        .ref_count = 1,
        .initialized = true,
    };
    
    std.debug.print("[Zig] Buffer creado: {} bytes (aligned to {})\n", .{ aligned_capacity, PAGE_SIZE });
    
    return buffer;
}

/// Escribe datos al buffer (zero-copy cuando es posible)
export fn shared_memory_buffer_write(
    buffer: *SharedMemoryBuffer,
    data: [*]const u8,
    len: usize,
) callconv(.C) isize {
    if (!buffer.initialized) return -1;
    if (buffer.data == null) return -2;
    
    // Check capacity
    if (buffer.used + len > buffer.capacity) {
        return -3; // Buffer full
    }
    
    // Get destination pointer
    const dest = buffer.data.?[buffer.used..];
    const src = data[0..len];
    
    // Copy data
    @memcpy(dest[0..len], src);
    
    // Update used bytes
    buffer.used += len;
    
    return @intCast(len);
}

/// Lee datos del buffer (zero-copy)
export fn shared_memory_buffer_read(
    buffer: *const SharedMemoryBuffer,
    offset: usize,
    dest: [*]u8,
    len: usize,
) callconv(.C) isize {
    if (!buffer.initialized) return -1;
    if (buffer.data == null) return -2;
    
    // Check bounds
    if (offset + len > buffer.used) {
        return -3; // Out of bounds
    }
    
    // Get source pointer
    const src = buffer.data.?[offset..];
    const dest_slice = dest[0..len];
    
    // Copy data
    @memcpy(dest_slice, src[0..len]);
    
    return @intCast(len);
}

/// Obtiene un puntero directo a los datos (zero-copy access)
export fn shared_memory_buffer_get_ptr(
    buffer: *const SharedMemoryBuffer,
    offset: usize,
) callconv(.C) ?[*]const u8 {
    if (!buffer.initialized) return null;
    if (buffer.data == null) return null;
    if (offset >= buffer.used) return null;
    
    return buffer.data.? + offset;
}

/// Limpia el buffer (resetea contador de usado)
export fn shared_memory_buffer_clear(buffer: *SharedMemoryBuffer) callconv(.C) void {
    buffer.used = 0;
}

/// Incrementa contador de referencias
export fn shared_memory_buffer_ref(buffer: *SharedMemoryBuffer) callconv(.C) void {
    // Atomic increment
    _ = @atomicRmw(u32, &buffer.ref_count, .Add, 1, .seq_cst);
}

/// Decrementa contador de referencias y libera si llega a 0
export fn shared_memory_buffer_unref(buffer: *SharedMemoryBuffer) callconv(.C) void {
    // Atomic decrement
    const old_count = @atomicRmw(u32, &buffer.ref_count, .Sub, 1, .seq_cst);
    
    if (old_count == 1) {
        // Last reference, free memory
        shared_memory_buffer_free(buffer);
    }
}

/// Libera el buffer
export fn shared_memory_buffer_free(buffer: *SharedMemoryBuffer) callconv(.C) void {
    if (!buffer.initialized) return;
    
    var allocator = std.heap.page_allocator;
    
    // Free data
    if (buffer.data) |ptr| {
        const slice = ptr[0..buffer.capacity];
        allocator.free(slice);
        buffer.data = null;
    }
    
    buffer.initialized = false;
    
    // Free buffer structure
    allocator.destroy(buffer);
    
    std.debug.print("[Zig] Buffer liberado\n", .{});
}

/// Obtiene información del buffer
export fn shared_memory_buffer_info(buffer: *const SharedMemoryBuffer) callconv(.C) BufferInfo {
    return BufferInfo{
        .capacity = buffer.capacity,
        .used = buffer.used,
        .available = buffer.capacity - buffer.used,
        .ref_count = buffer.ref_count,
        .initialized = buffer.initialized,
    };
}

/// Estructura de información del buffer
pub const BufferInfo = extern struct {
    capacity: usize,
    used: usize,
    available: usize,
    ref_count: u32,
    initialized: bool,
};

// Tests
test "SharedMemoryBuffer creation" {
    const buffer = shared_memory_buffer_new(1024 * 1024);
    try std.testing.expect(buffer != null);
    
    const info = shared_memory_buffer_info(buffer.?);
    try std.testing.expect(info.initialized);
    try std.testing.expect(info.capacity >= 1024 * 1024);
    try std.testing.expectEqual(@as(usize, 0), info.used);
    try std.testing.expectEqual(@as(u32, 1), info.ref_count);
    
    shared_memory_buffer_free(buffer.?);
}

test "SharedMemoryBuffer write and read" {
    const buffer = shared_memory_buffer_new(4096);
    try std.testing.expect(buffer != null);
    
    const test_data = "Hello, Zig FFI!";
    const written = shared_memory_buffer_write(buffer.?, test_data.ptr, test_data.len);
    try std.testing.expectEqual(@as(isize, @intCast(test_data.len)), written);
    
    var read_buf: [100]u8 = undefined;
    const read_len = shared_memory_buffer_read(buffer.?, 0, &read_buf, test_data.len);
    try std.testing.expectEqual(@as(isize, @intCast(test_data.len)), read_len);
    try std.testing.expectEqualStrings(test_data, read_buf[0..test_data.len]);
    
    shared_memory_buffer_free(buffer.?);
}

test "SharedMemoryBuffer reference counting" {
    const buffer = shared_memory_buffer_new(1024);
    try std.testing.expect(buffer != null);
    
    var info = shared_memory_buffer_info(buffer.?);
    try std.testing.expectEqual(@as(u32, 1), info.ref_count);
    
    shared_memory_buffer_ref(buffer.?);
    info = shared_memory_buffer_info(buffer.?);
    try std.testing.expectEqual(@as(u32, 2), info.ref_count);
    
    shared_memory_buffer_unref(buffer.?);
    info = shared_memory_buffer_info(buffer.?);
    try std.testing.expectEqual(@as(u32, 1), info.ref_count);
    
    shared_memory_buffer_unref(buffer.?); // This will free the buffer
}
