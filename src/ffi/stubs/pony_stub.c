#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

void pony_init() {}
void pony_shutdown() {}

char* pony_distributed_search(const char* query, size_t query_len, const char** indices, size_t indices_count) {
    // Return a dummy JSON string
    const char* dummy = "{\"status\":\"stubbed\", \"results\":[]}";
    char* result = malloc(strlen(dummy) + 1);
    strcpy(result, dummy);
    return result;
}
