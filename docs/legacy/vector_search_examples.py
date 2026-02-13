"""
Ejemplos de uso de la API de Vector Search de MEMORY_P

Este script muestra cómo usar los diferentes endpoints de búsqueda vectorial
desde Python usando requests.
"""

import requests
import json
from typing import List, Dict, Any, Optional

# Configuración
BASE_URL = "http://localhost:4040/mcp"
HEADERS = {"Content-Type": "application/json"}


class MemoryPVectorClient:
    """Cliente Python para la API de Vector Search de MEMORY_P"""

    def __init__(self, base_url: str = BASE_URL):
        self.base_url = base_url
        self.request_id = 0

    def _make_request(self, method: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Hace una request JSON-RPC 2.0"""
        self.request_id += 1
        payload = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params,
        }

        response = requests.post(self.base_url, json=payload, headers=HEADERS)
        response.raise_for_status()
        return response.json()

    def index_documents(
        self,
        documents: List[Dict[str, Any]],
        model: str = "MiniLM-L6",
        batch_size: int = 32,
    ) -> Dict[str, Any]:
        """
        Indexa documentos con embeddings automáticos.

        Args:
            documents: Lista de documentos con id, text y metadata
            model: Modelo de embeddings a usar
            batch_size: Tamaño de batch para procesamiento

        Returns:
            Resultado de indexación con contadores
        """
        params = {
            "name": "index_documents",
            "arguments": {
                "documents": documents,
                "model": model,
                "batch_size": batch_size,
            },
        }
        return self._make_request("tools/call", params)

    def search(
        self,
        query: str,
        limit: int = 10,
        filters: Optional[Dict[str, Any]] = None,
        model: str = "MiniLM-L6",
        metric: str = "cosine",
    ) -> Dict[str, Any]:
        """
        Búsqueda vectorial con filtros opcionales.

        Args:
            query: Texto de búsqueda
            limit: Número máximo de resultados
            filters: Filtros por metadata (must, must_not, timestamp_range)
            model: Modelo de embeddings
            metric: Métrica de distancia (cosine, euclidean, dotproduct, manhattan)

        Returns:
            Resultados de búsqueda con scores
        """
        arguments = {
            "query": query,
            "limit": limit,
            "model": model,
            "metric": metric,
        }

        if filters:
            arguments["filters"] = filters

        params = {"name": "map_search", "arguments": arguments}
        return self._make_request("tools/call", params)

    def find_similar(
        self,
        document_id: str,
        limit: int = 10,
        filters: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """
        Encuentra documentos similares a uno dado.

        Args:
            document_id: ID del documento de referencia
            limit: Número de resultados
            filters: Filtros opcionales

        Returns:
            Documentos similares ordenados por score
        """
        arguments = {"document_id": document_id, "limit": limit}

        if filters:
            arguments["filters"] = filters

        params = {"name": "similar_docs", "arguments": arguments}
        return self._make_request("tools/call", params)

    def get_stats(self) -> Dict[str, Any]:
        """Obtiene estadísticas del motor vectorial"""
        params = {"name": "vector_stats", "arguments": {}}
        return self._make_request("tools/call", params)


# ============================================================================
# EJEMPLO 1: Indexar y buscar documentos técnicos
# ============================================================================


def example_technical_docs():
    """Indexa y busca en documentación técnica"""
    print("=" * 80)
    print("EJEMPLO 1: Documentación Técnica")
    print("=" * 80)

    client = MemoryPVectorClient()

    # Documentos de ejemplo
    docs = [
        {
            "id": "rust_ownership",
            "text": "Rust's ownership system ensures memory safety without garbage collection. "
            "Each value has a single owner, and when the owner goes out of scope, "
            "the value is dropped.",
            "metadata": {
                "language": "rust",
                "topic": "memory_management",
                "difficulty": "intermediate",
            },
        },
        {
            "id": "python_gil",
            "text": "Python's Global Interpreter Lock (GIL) prevents multiple threads from "
            "executing Python bytecode simultaneously. This affects CPU-bound tasks "
            "but not I/O-bound ones.",
            "metadata": {
                "language": "python",
                "topic": "concurrency",
                "difficulty": "advanced",
            },
        },
        {
            "id": "go_goroutines",
            "text": "Go uses goroutines for lightweight concurrency. Goroutines are functions "
            "that run concurrently with other functions, managed by the Go runtime.",
            "metadata": {
                "language": "go",
                "topic": "concurrency",
                "difficulty": "beginner",
            },
        },
        {
            "id": "rust_async",
            "text": "Rust's async/await syntax enables writing asynchronous code. The futures "
            "are lazy and only execute when polled by an executor like Tokio.",
            "metadata": {
                "language": "rust",
                "topic": "concurrency",
                "difficulty": "intermediate",
            },
        },
    ]

    # Indexar
    print("\n📚 Indexando documentos...")
    result = client.index_documents(docs, model="BGE-Small")
    print(json.dumps(result, indent=2))

    # Búsqueda 1: Concurrency general
    print("\n🔍 Búsqueda: 'concurrent programming patterns'")
    results = client.search("concurrent programming patterns", limit=3)
    print(json.dumps(results, indent=2))

    # Búsqueda 2: Solo documentos de Rust
    print("\n🔍 Búsqueda filtrada: 'memory management' (solo Rust)")
    results = client.search(
        "memory management and safety",
        limit=5,
        filters={"must": {"language": "rust"}},
    )
    print(json.dumps(results, indent=2))

    # Búsqueda 3: Excluir documentos avanzados
    print("\n🔍 Búsqueda filtrada: 'concurrency' (no avanzados)")
    results = client.search(
        "how to write concurrent code",
        limit=5,
        filters={"must_not": {"difficulty": "advanced"}},
    )
    print(json.dumps(results, indent=2))

    # Documentos similares
    print("\n🔗 Documentos similares a 'rust_ownership'")
    similar = client.find_similar("rust_ownership", limit=3)
    print(json.dumps(similar, indent=2))


# ============================================================================
# Ejecutar ejemplos
# ============================================================================

if __name__ == "__main__":
    try:
        print("\n🚀 MEMORY_P Vector Search - Ejemplos de Uso\n")

        # Ejecutar todos los ejemplos
        example_technical_docs()

        print("\n" + "=" * 80)
        print("✅ Todos los ejemplos ejecutados exitosamente")
        print("=" * 80 + "\n")

    except requests.exceptions.ConnectionError:
        print("\n❌ Error: No se pudo conectar al servidor MEMORY_P")
        print("   Asegúrate de que el servidor esté corriendo en http://localhost:4040")
    except Exception as e:
        print(f"\n❌ Error: {e}")
