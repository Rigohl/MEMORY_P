//! Ejemplo de uso del Nuclear Crawler Hybrid System

use memory_p::nuclear_crawler::{CrawlerConfig, NuclearCrawler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar logger
    tracing_subscriber::fmt::init();

    println!("🚀 Iniciando Nuclear Crawler Hybrid System Demo\n");

    // 1. Configurar el crawler
    let config = CrawlerConfig {
        enable_tor: false, // Tor deshabilitado para demo
        enable_intelligent_storage: true,
        enable_predictive_nodes: true,
        auto_rebuild_interval: 60, // 1 minuto para demo
        parallel_buffer_size: 512,
        security_level: 3,
    };

    println!("📋 Configuración:");
    println!("  - Tor: {}", config.enable_tor);
    println!(
        "  - Intelligent Storage: {}",
        config.enable_intelligent_storage
    );
    println!("  - Predictive Nodes: {}", config.enable_predictive_nodes);
    println!("  - Security Level: {}", config.security_level);
    println!();

    // 2. Crear el crawler
    let crawler = NuclearCrawler::new(config);
    println!("✅ Nuclear Crawler creado\n");

    // 3. Iniciar el sistema (auto-gestión activada)
    crawler.start().await?;
    println!("✅ Sistema iniciado - Auto-gestión activa\n");

    // 4. Simular operaciones
    println!("🔍 Realizando búsquedas con auto-corrección...");

    let queries = vec![
        "rust async programming",
        "machine learning algorithms",
        "distributed systems",
    ];

    for query in queries {
        println!("\n📝 Query: {}", query);
        match crawler.search(query).await {
            Ok(results) => {
                println!("  ✅ {} resultados encontrados", results.len());
                for (i, result) in results.iter().take(3).enumerate() {
                    println!("    {}. {}", i + 1, result);
                }
            }
            Err(e) => println!("  ❌ Error: {}", e),
        }
    }

    // 5. Obtener estadísticas
    println!("\n📊 Estadísticas del sistema:");
    let stats = crawler.get_stats();
    println!("{}\n", serde_json::to_string_pretty(&stats)?);

    // 6. Exportar métricas Prometheus
    println!("📈 Métricas Prometheus (primeras 10 líneas):");
    let metrics = crawler.export_prometheus_metrics();
    for (i, line) in metrics.lines().take(10).enumerate() {
        println!("  {}: {}", i + 1, line);
    }

    // 7. Esperar un poco para ver auto-gestión
    println!("\n⏳ Esperando para ver auto-gestión en acción...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 8. Detener el sistema
    println!("\n🛑 Deteniendo Nuclear Crawler...");
    crawler.stop().await?;
    println!("✅ Sistema detenido correctamente");

    println!("\n🎉 Demo completada exitosamente!");

    Ok(())
}
