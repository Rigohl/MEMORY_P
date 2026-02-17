#include <stddef.h>
#include <stdint.h>

int jax_init_ffi() { return 0; }
int jax_shutdown_ffi() { return 0; }

int jax_generate_embedding_ffi(const char* text, size_t text_len, float* result, size_t result_len) {
    for (size_t i = 0; i < result_len; i++) result[i] = 0.0f;
    return 0;
}

float jax_cosine_similarity_ffi(const float* vec1, const float* vec2, size_t dim) {
    return 0.0f;
}

int jax_predict_next_moves_ffi(const float* context_vec, size_t dim, size_t n_moves, float* result) {
    for (size_t i = 0; i < dim * n_moves; i++) result[i] = 0.0f;
    return 0;
}
