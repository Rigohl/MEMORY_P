#include <stddef.h>
#include <stdint.h>

double mojo_dot_product(const double* a, const double* b, size_t n) {
    return 0.0;
}

double mojo_cosine_similarity(const double* a, const double* b, size_t n) {
    return 0.0;
}

void mojo_cosine_similarity_batch(const double* query, const double* corpus, size_t n_docs, size_t dim, double* results) {
    for (size_t i = 0; i < n_docs; i++) {
        results[i] = 0.0;
    }
}
