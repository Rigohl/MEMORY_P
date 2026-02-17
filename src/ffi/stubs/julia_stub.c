#include <stddef.h>
#include <stdint.h>

int julia_init() { return 0; }
int julia_shutdown() { return 0; }

double julia_chaos_analysis_ffi(const double* data, int len) {
    return 0.5; // Dummy chaos value
}

// Added this symbol as it was missing in the original read but I planned to add it
int julia_get_decision_ffi(double entropy, double chaos, double stability, char* buffer, size_t buffer_len) {
    // Just write "HYBRID_STUB" into buffer
    const char* decision = "HYBRID_STUB";
    size_t i = 0;
    while (decision[i] && i < buffer_len - 1) {
        buffer[i] = decision[i];
        i++;
    }
    buffer[i] = '\0';
    return 0;
}
