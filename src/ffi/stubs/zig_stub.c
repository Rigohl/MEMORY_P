#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

typedef struct {
    size_t capacity;
    size_t used;
    size_t available;
    uint32_t ref_count;
    int initialized;
} BufferInfo;

void* shared_memory_buffer_new(size_t capacity) {
    // Return a dummy pointer (non-null)
    return (void*)0xDEADBEEF;
}

long shared_memory_buffer_write(void* buffer, const char* data, size_t len) {
    return (long)len;
}

long shared_memory_buffer_read(const void* buffer, size_t offset, char* dest, size_t len) {
    // Fill with dummy data
    for (size_t i = 0; i < len; i++) dest[i] = 0;
    return (long)len;
}

void shared_memory_buffer_free(void* buffer) {}

BufferInfo shared_memory_buffer_info(const void* buffer) {
    BufferInfo info = {1024, 0, 1024, 1, 1};
    return info;
}

void shared_memory_buffer_ref(void* buffer) {}
void shared_memory_buffer_unref(void* buffer) {}
