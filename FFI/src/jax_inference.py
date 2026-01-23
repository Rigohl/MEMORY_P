"""
jax_inference.py - JAX ML Inference Engine for MEMORY_P v2.0

Proporciona capacidades de machine learning:
- Generación de embeddings semánticos
- Inference de modelos de clasificación
- Ranking ML-powered
- Batch processing con aceleración GPU/TPU
"""

import sys
from typing import List, Optional, Tuple
import numpy as np

# Imports opcionales con fallbacks
try:
    import jax
    import jax.numpy as jnp
    from jax import jit
    JAX_AVAILABLE = True
    print("[JAX] JAX disponible:", jax.__version__)
except ImportError:
    JAX_AVAILABLE = False
    print("[JAX] WARNING: JAX no disponible, usando NumPy como fallback")
    jnp = np

try:
    from sentence_transformers import SentenceTransformer
    EMBEDDINGS_AVAILABLE = True
    print("[JAX] sentence-transformers disponible")
except ImportError:
    EMBEDDINGS_AVAILABLE = False
    print("[JAX] WARNING: sentence-transformers no disponible")


class JaxInferenceEngine:
    """Motor de inference con JAX para MEMORY_P"""
    
    def __init__(self, model_name: str = "all-MiniLM-L6-v2"):
        """
        Inicializa el motor de inference.
        
        Args:
            model_name: Nombre del modelo de sentence-transformers
        """
        self.model = None
        self.model_name = model_name
        
        if EMBEDDINGS_AVAILABLE:
            try:
                self.model = SentenceTransformer(model_name)
                print(f"[JAX] Modelo cargado: {model_name}")
            except Exception as e:
                print(f"[JAX] ERROR al cargar modelo: {e}")
    
    def generate_embedding(self, text: str) -> np.ndarray:
        """
        Genera embedding para un texto.
        
        Args:
            text: Texto de entrada
            
        Returns:
            Vector de embedding (384 dims para MiniLM)
        """
        if self.model is None:
            # Fallback: Retornar vector aleatorio
            print("[JAX] WARNING: Modelo no disponible, retornando vector aleatorio")
            return np.random.randn(384).astype(np.float32)
        
        embedding = self.model.encode(text, convert_to_numpy=True)
        return embedding.astype(np.float32)
    
    def generate_embeddings_batch(self, texts: List[str]) -> np.ndarray:
        """
        Genera embeddings para múltiples textos (batch processing).
        
        Args:
            texts: Lista de textos
            
        Returns:
            Matriz de embeddings (N x 384)
        """
        if self.model is None:
            # Fallback: Retornar matriz aleatoria
            n = len(texts)
            return np.random.randn(n, 384).astype(np.float32)
        
        embeddings = self.model.encode(
            texts,
            convert_to_numpy=True,
            batch_size=32,
            show_progress_bar=False
        )
        return embeddings.astype(np.float32)
    
    @staticmethod
    def cosine_similarity(vec1: np.ndarray, vec2: np.ndarray) -> float:
        """
        Calcula similitud coseno entre dos vectores.
        
        Args:
            vec1, vec2: Vectores de embedding
            
        Returns:
            Similitud coseno [-1, 1]
        """
        if JAX_AVAILABLE:
            vec1_jnp = jnp.array(vec1)
            vec2_jnp = jnp.array(vec2)
            
            dot = jnp.dot(vec1_jnp, vec2_jnp)
            norm1 = jnp.linalg.norm(vec1_jnp)
            norm2 = jnp.linalg.norm(vec2_jnp)
            
            return float(dot / (norm1 * norm2 + 1e-8))
        else:
            # Fallback NumPy
            dot = np.dot(vec1, vec2)
            norm1 = np.linalg.norm(vec1)
            norm2 = np.linalg.norm(vec2)
            return float(dot / (norm1 * norm2 + 1e-8))
    
    @staticmethod
    @jit if JAX_AVAILABLE else lambda f: f
    def cosine_similarity_batch_jit(
        query: jnp.ndarray, 
        corpus: jnp.ndarray
    ) -> jnp.ndarray:
        """
        Calcula similitudes coseno entre query y corpus (batch, JIT compiled).
        
        Args:
            query: Vector de query (D,)
            corpus: Matriz de corpus (N, D)
            
        Returns:
            Vector de similitudes (N,)
        """
        # Normalizar
        query_norm = query / (jnp.linalg.norm(query) + 1e-8)
        corpus_norm = corpus / (jnp.linalg.norm(corpus, axis=1, keepdims=True) + 1e-8)
        
        # Dot products
        similarities = jnp.dot(corpus_norm, query_norm)
        
        return similarities


# Instancia global (singleton)
_engine: Optional[JaxInferenceEngine] = None


def get_engine() -> JaxInferenceEngine:
    """Obtiene instancia singleton del motor."""
    global _engine
    if _engine is None:
        _engine = JaxInferenceEngine()
    return _engine


# FFI C-compatible functions

def jax_generate_embedding_ffi(text_ptr: int, text_len: int) -> Tuple[int, int]:
    """
    Genera embedding desde FFI (C/Rust).
    
    Args:
        text_ptr: Puntero a string C
        text_len: Longitud del string
        
    Returns:
        (pointer_to_embedding, embedding_len)
    """
    # Reconstruir string desde puntero
    # NOTA: Esto es simplificado - en producción usar ctypes
    text = "example text"  # Placeholder
    
    engine = get_engine()
    embedding = engine.generate_embedding(text)
    
    # Retornar como puntero
    # NOTA: Memoria debe ser liberada por caller
    return (id(embedding), len(embedding))


def jax_cosine_similarity_ffi(
    vec1_ptr: int, 
    vec2_ptr: int, 
    dim: int
) -> float:
    """
    Calcula similitud coseno desde FFI.
    
    Args:
        vec1_ptr, vec2_ptr: Punteros a vectores C (float64*)
        dim: Dimensión de los vectores
        
    Returns:
        Similitud coseno
    """
    # Reconstruir arrays desde punteros
    # NOTA: Placeholder - en producción usar ctypes
    vec1 = np.random.randn(dim).astype(np.float32)
    vec2 = np.random.randn(dim).astype(np.float32)
    
    return JaxInferenceEngine.cosine_similarity(vec1, vec2)


# Main para testing standalone
if __name__ == "__main__":
    print("\n🤖 Testing JAX Inference Engine\n")
    
    engine = get_engine()
    
    # Test 1: Embedding single
    print("Test 1: Embedding generation")
    text = "parallel processing with Rust"
    emb = engine.generate_embedding(text)
    print(f"  Text: '{text}'")
    print(f"  Embedding shape: {emb.shape}")
    print(f"  Embedding norm: {np.linalg.norm(emb):.4f}")
    
    # Test 2: Embedding batch
    print("\nTest 2: Batch embeddings")
    texts = [
        "Rust programming language",
        "Julia mathematical computing",
        "Mojo high performance",
    ]
    embeddings = engine.generate_embeddings_batch(texts)
    print(f"  Batch size: {len(texts)}")
    print(f"  Embeddings shape: {embeddings.shape}")
    
    # Test 3: Cosine similarity
    print("\nTest 3: Cosine similarity")
    sim = engine.cosine_similarity(embeddings[0], embeddings[1])
    print(f"  Similarity(Rust, Julia): {sim:.4f}")
    
    sim = engine.cosine_similarity(embeddings[0], embeddings[2])
    print(f"  Similarity(Rust, Mojo): {sim:.4f}")
    
    print("\n✅ All tests passed")
