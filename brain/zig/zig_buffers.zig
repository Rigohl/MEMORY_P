// FFI/src/zig_buffers.zig
// Zig Shared Memory Buffers - Zero-copy memory management

const std = @import("std");
const Allocator = std.mem.Allocator;

/// Buffer compartido para transferencia zero-copy entre lenguajes
pub const SharedBuffer = extern struct {
    data: [*]u8,
    len: usize,
    capacity: usize,
    ref_count: usize,
};

/// Contexto de memoria compartida
pub const SharedMemoryContext = struct {
    allocator: Allocator,
    buffers: std.ArrayList(*SharedBuffer),

    pub fn init(allocator: Allocator) !SharedMemoryContext {
        return SharedMemoryContext{
            .allocator = allocator,
            .buffers = std.ArrayList(*SharedBuffer).init(allocator),
        };
    }

    pub fn deinit(self: *SharedMemoryContext) void {
        for (self.buffers.items) |buffer| {
            self.destroyBuffer(buffer);
        }
        self.buffers.deinit();
    }

    /// Crea un nuevo buffer compartido
    pub fn createBuffer(self: *SharedMemoryContext, capacity: usize) !*SharedBuffer {
        const buffer = try self.allocator.create(SharedBuffer);
        const data = try self.allocator.alloc(u8, capacity);

        buffer.* = SharedBuffer{
            .data = data.ptr,
            .len = 0,
            .capacity = capacity,
            .ref_count = 1,
        };

        try self.buffers.append(buffer);
        return buffer;
    }

    /// Incrementa reference count
    pub fn retainBuffer(buffer: *SharedBuffer) void {
        buffer.ref_count += 1;
    }

    /// Decrementa reference count y libera si es 0
    pub fn releaseBuffer(self: *SharedMemoryContext, buffer: *SharedBuffer) void {
        buffer.ref_count -= 1;
        if (buffer.ref_count == 0) {
            self.destroyBuffer(buffer);
        }
    }

    fn destroyBuffer(self: *SharedMemoryContext, buffer: *SharedBuffer) void {
        const data_slice = buffer.data[0..buffer.capacity];
        self.allocator.free(data_slice);
        self.allocator.destroy(buffer);
    }
};

/// Pool de buffers para reutilización
pub const BufferPool = struct {
    allocator: Allocator,
    free_buffers: std.ArrayList(*SharedBuffer),
    buffer_size: usize,

    pub fn init(allocator: Allocator, buffer_size: usize) BufferPool {
        return BufferPool{
            .allocator = allocator,
            .free_buffers = std.ArrayList(*SharedBuffer).init(allocator),
            .buffer_size = buffer_size,
        };
    }

    pub fn deinit(self: *BufferPool) void {
        for (self.free_buffers.items) |buffer| {
            const data_slice = buffer.data[0..buffer.capacity];
            self.allocator.free(data_slice);
            self.allocator.destroy(buffer);
        }
        self.free_buffers.deinit();
    }

    /// Adquiere un buffer del pool
    pub fn acquire(self: *BufferPool) !*SharedBuffer {
        if (self.free_buffers.items.len > 0) {
            return self.free_buffers.pop();
        }

        // Crear nuevo buffer
        const buffer = try self.allocator.create(SharedBuffer);
        const data = try self.allocator.alloc(u8, self.buffer_size);

        buffer.* = SharedBuffer{
            .data = data.ptr,
            .len = 0,
            .capacity = self.buffer_size,
            .ref_count = 1,
        };

        return buffer;
    }

    /// Retorna un buffer al pool
    pub fn release(self: *BufferPool, buffer: *SharedBuffer) !void {
        buffer.len = 0;  // Reset length
        buffer.ref_count = 1;
        try self.free_buffers.append(buffer);
    }
};

/// Operaciones de alto rendimiento sobre buffers

/// Copia rápida entre buffers
pub fn fastCopy(dest: *SharedBuffer, src: *const SharedBuffer) !void {
    if (dest.capacity < src.len) {
        return error.BufferTooSmall;
    }

    @memcpy(dest.data[0..src.len], src.data[0..src.len]);
    dest.len = src.len;
}

/// Serialización de Vector<f64> a buffer
pub fn serializeF64Vec(buffer: *SharedBuffer, vec: []const f64) !void {
    const bytes_needed = vec.len * @sizeOf(f64);
    if (buffer.capacity < bytes_needed) {
        return error.BufferTooSmall;
    }

    const dest = @as([*]f64, @ptrCast(@alignCast(buffer.data)));
    @memcpy(dest[0..vec.len], vec);
    buffer.len = bytes_needed;
}

/// Deserialización de buffer a Vector<f64>
pub fn deserializeF64Vec(allocator: Allocator, buffer: *const SharedBuffer) ![]f64 {
    const count = buffer.len / @sizeOf(f64);
    const vec = try allocator.alloc(f64, count);

    const src = @as([*]const f64, @ptrCast(@alignCast(buffer.data)));
    @memcpy(vec, src[0..count]);

    return vec;
}

// ============================================================================
// FFI Exports para Rust
// ============================================================================

/// Crea contexto de memoria compartida
export fn zig_shared_memory_create() ?*SharedMemoryContext {
    const allocator = std.heap.c_allocator;
    const ctx = allocator.create(SharedMemoryContext) catch return null;
    ctx.* = SharedMemoryContext.init(allocator) catch {
        allocator.destroy(ctx);
        return null;
    };
    return ctx;
}

/// Destruye contexto de memoria compartida
export fn zig_shared_memory_destroy(ctx: ?*SharedMemoryContext) void {
    if (ctx) |context| {
        context.deinit();
        std.heap.c_allocator.destroy(context);
    }
}

/// Crea buffer compartido
export fn zig_create_shared_buffer(ctx: ?*SharedMemoryContext, capacity: usize) ?*SharedBuffer {
    if (ctx) |context| {
        return context.createBuffer(capacity) catch null;
    }
    return null;
}

/// Libera buffer compartido
export fn zig_release_shared_buffer(ctx: ?*SharedMemoryContext, buffer: ?*SharedBuffer) void {
    if (ctx) |context| {
        if (buffer) |buf| {
            context.releaseBuffer(buf);
        }
    }
}

/// Copia datos a buffer
export fn zig_buffer_write(buffer: ?*SharedBuffer, data: [*]const u8, len: usize) c_int {
    if (buffer) |buf| {
        if (buf.capacity < len) return -1;
        @memcpy(buf.data[0..len], data[0..len]);
        buf.len = len;
        return 0;
    }
    return -1;
}

/// Lee datos de buffer
export fn zig_buffer_read(buffer: ?*const SharedBuffer, data: [*]u8, max_len: usize) c_int {
    if (buffer) |buf| {
        const copy_len = @min(buf.len, max_len);
        @memcpy(data[0..copy_len], buf.data[0..copy_len]);
        return @intCast(copy_len);
    }
    return -1;
}

/// Crea pool de buffers
export fn zig_buffer_pool_create(buffer_size: usize) ?*BufferPool {
    const allocator = std.heap.c_allocator;
    const pool = allocator.create(BufferPool) catch return null;
    pool.* = BufferPool.init(allocator, buffer_size);
    return pool;
}

/// Destruye pool de buffers
export fn zig_buffer_pool_destroy(pool: ?*BufferPool) void {
    if (pool) |p| {
        p.deinit();
        std.heap.c_allocator.destroy(p);
    }
}

/// Adquiere buffer del pool
export fn zig_buffer_pool_acquire(pool: ?*BufferPool) ?*SharedBuffer {
    if (pool) |p| {
        return p.acquire() catch null;
    }
    return null;
}

/// Retorna buffer al pool
export fn zig_buffer_pool_release(pool: ?*BufferPool, buffer: ?*SharedBuffer) c_int {
    if (pool) |p| {
        if (buffer) |buf| {
            p.release(buf) catch return -1;
            return 0;
        }
    }
    return -1;
}

// ============================================================================
// Tests
// ============================================================================

test "create and destroy shared buffer" {
    const allocator = std.testing.allocator;
    var ctx = try SharedMemoryContext.init(allocator);
    defer ctx.deinit();

    const buffer = try ctx.createBuffer(1024);
    try std.testing.expectEqual(@as(usize, 1024), buffer.capacity);
    try std.testing.expectEqual(@as(usize, 1), buffer.ref_count);
}

test "buffer pool acquire and release" {
    const allocator = std.testing.allocator;
    var pool = BufferPool.init(allocator, 512);
    defer pool.deinit();

    const buf1 = try pool.acquire();
    try std.testing.expectEqual(@as(usize, 512), buf1.capacity);

    try pool.release(buf1);
    try std.testing.expectEqual(@as(usize, 1), pool.free_buffers.items.len);

    const buf2 = try pool.acquire();
    try std.testing.expectEqual(buf1, buf2);  // Should reuse same buffer
}

test "serialize and deserialize f64 vector" {
    const allocator = std.testing.allocator;
    var ctx = try SharedMemoryContext.init(allocator);
    defer ctx.deinit();

    const buffer = try ctx.createBuffer(1024);

    const original = [_]f64{ 1.0, 2.0, 3.0, 4.0, 5.0 };
    try serializeF64Vec(buffer, &original);

    const deserialized = try deserializeF64Vec(allocator, buffer);
    defer allocator.free(deserialized);

    try std.testing.expectEqualSlices(f64, &original, deserialized);
}
