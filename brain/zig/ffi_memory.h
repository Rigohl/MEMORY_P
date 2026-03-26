/**
 * ffi_memory.h - C Header for MEMORY_P Zig FFI Bridge
 *
 * Auto-generated from brain/zig/*.zig exports.
 * Mirrors the extern structs and exported functions from:
 *   - ffi_bridge.zig      (FfiVec, FfiResult, ffi_init/shutdown/dispatch)
 *   - shared_memory_buffer.zig (SharedMemoryBuffer, BufferInfo)
 *   - zig_buffers.zig     (buffer pool management)
 *
 * MEMORY_P v2.0 - Ultra-Low-Latency FFI Layer
 */

#ifndef FFI_MEMORY_H
#define FFI_MEMORY_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================
 * ffi_bridge.zig types
 * ======================================================================== */

/** Language enum — must match Zig Language enum(u8) */
typedef enum {
    FFI_LANG_JULIA = 0,
    FFI_LANG_JAX   = 1,
    FFI_LANG_MOJO  = 2,
    FFI_LANG_PONY  = 3,
    FFI_LANG_ZIG   = 4,
} FfiLanguage;

/** Zero-copy vector for FFI data exchange */
typedef struct {
    double *data;
    size_t  len;
    size_t  cap;
} FfiVec;

/** Result structure from FFI dispatch operations */
typedef struct {
    bool        success;
    FfiVec      data;
    const char *error_msg;
} FfiResult;

/* ffi_bridge.zig exported functions */
bool      ffi_init(void);
void      ffi_shutdown(void);
FfiResult ffi_dispatch(FfiLanguage lang, const char *operation, FfiVec input);
void      ffi_free_result(FfiResult *result);

/* ========================================================================
 * shared_memory_buffer.zig types
 * ======================================================================== */

/** Page-aligned shared memory buffer with atomic ref-counting */
typedef struct {
    uint8_t *data;
    size_t   capacity;
    size_t   used;
    uint32_t ref_count;
    bool     initialized;
} SharedMemoryBuffer;

/** Buffer info snapshot (read-only) */
typedef struct {
    size_t   capacity;
    size_t   used;
    size_t   available;
    uint32_t ref_count;
    bool     initialized;
} BufferInfo;

/* shared_memory_buffer.zig exported functions */
SharedMemoryBuffer *shared_memory_buffer_new(size_t capacity);
ssize_t             shared_memory_buffer_write(SharedMemoryBuffer *buf,
                                               const uint8_t *data, size_t len);
ssize_t             shared_memory_buffer_read(const SharedMemoryBuffer *buf,
                                              size_t offset, uint8_t *dest, size_t len);
const uint8_t      *shared_memory_buffer_get_ptr(const SharedMemoryBuffer *buf,
                                                  size_t offset);
void                shared_memory_buffer_clear(SharedMemoryBuffer *buf);
void                shared_memory_buffer_ref(SharedMemoryBuffer *buf);
void                shared_memory_buffer_unref(SharedMemoryBuffer *buf);
void                shared_memory_buffer_free(SharedMemoryBuffer *buf);
BufferInfo          shared_memory_buffer_info(const SharedMemoryBuffer *buf);

/* ========================================================================
 * zig_buffers.zig exported functions (buffer pool management)
 * ======================================================================== */

/** Initialize shared memory subsystem */
bool    zig_shared_memory_create(size_t capacity);

/** Create a named shared buffer */
void   *zig_create_shared_buffer(const char *name, size_t size);

/** Write to named buffer */
ssize_t zig_write_shared_buffer(void *handle, const uint8_t *data, size_t len);

/** Read from named buffer */
ssize_t zig_read_shared_buffer(void *handle, uint8_t *dest, size_t len);

/** Free a named buffer */
void    zig_free_shared_buffer(void *handle);

/** Destroy shared memory subsystem */
void    zig_shared_memory_destroy(void);

#ifdef __cplusplus
}
#endif

#endif /* FFI_MEMORY_H */