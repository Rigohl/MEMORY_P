//! ffi/pony.rs - Pony Actor System Integration

use super::error::{FfiError, Result};

/// Inicializa el runtime de Pony
pub fn init() -> Result<()> {
    #[cfg(feature = "ffi-pony")]
    {
        tracing::info!("🎭 Inicializando Pony actor system");
        // TODO: Inicializar Pony runtime
        Ok(())
    }

    #[cfg(not(feature = "ffi-pony"))]
    {
        tracing::warn!("⚠️  Pony no disponible (feature 'ffi-pony' deshabilitado)");
        Err(FfiError::NotAvailable("Pony".to_string()))
    }
}

/// Finaliza el runtime de Pony
pub fn shutdown() {
    #[cfg(feature = "ffi-pony")]
    {
        tracing::info!("🎭 Finalizando Pony runtime");
        // TODO: Finalizar Pony runtime
    }
}

/// Ejecuta búsqueda distribuida con actores Pony
pub async fn distributed_search(_query: &str, _indices: &[String]) -> Result<Vec<String>> {
    #[cfg(feature = "ffi-pony")]
    {
        tracing::debug!("Búsqueda distribuida con Pony para: '{}'", query);

        // TODO: Llamada real a Pony actors via FFI
        // Stub: Retornar resultados sintéticos
        let results = vec![
            format!("result_1_for_{}", query),
            format!("result_2_for_{}", query),
            format!("result_3_for_{}", query),
        ];

        Ok(results)
    }

    #[cfg(not(feature = "ffi-pony"))]
    {
        Err(FfiError::NotAvailable(
            "Pony distributed_search".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_distributed_search() {
        let query = "test query";
        let indices = vec!["index1".to_string(), "index2".to_string()];

        let result = distributed_search(query, &indices).await;

        if let Ok(results) = result {
            assert!(!results.is_empty());
        }
    }
}
