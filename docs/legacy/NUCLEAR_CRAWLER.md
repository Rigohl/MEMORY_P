# Nuclear Crawler Hybrid System

Sistema avanzado de crawling con auto-gestión, validación continua y monitoreo constante integrado en MEMORY_P v2.0.

## 🚀 Características Principales

### 1. **Auto-Gestión (FORCED_REBUILDS)**
- ✅ Ajuste automático de módulos sin intervención manual
- ✅ Sistema de prioridades para reconstrucciones
- ✅ Monitoreo continuo de estado de componentes
- ✅ Auto-activación/desactivación basada en métricas

### 2. **Auto-Push y Validación**
- ✅ Workflows de GitHub Actions para validación automática
- ✅ Auto-merge seguro en ramas autorizadas
- ✅ Verificación de unidades críticas antes de merge
- ✅ Detección de cambios sensibles a seguridad

### 3. **Extensión Funcional**

#### DeepWeb Tor
- Navegación segura a través de Tor (SOCKS5)
- Rotación automática de circuitos
- Acceso en tiempo real a contenido deep web
- Timeout y manejo de errores robusto

#### Intelligent Storage
- Almacenamiento con expansión dinámica
- Sistema de prioridades (Low, Medium, High, Critical)
- Auto-limpieza de items de baja prioridad
- Monitoreo de uso en tiempo real

#### Predictive Nodes
- Auto-corrección de búsquedas fallidas
- Aprendizaje continuo de predicciones exitosas
- Múltiples estrategias de corrección
- Tasa de éxito medible

#### Deep Storage Tunnels
- Procesamiento paralelo con Rayon
- Buffers dinámicos adaptativos
- Túneles multi-profundidad
- Optimización automática

### 4. **Monitoreo y Diagnóstico**

#### Prometheus + Grafana
- Métricas en tiempo real exportadas
- Dashboards para visualización
- Alertas configurables
- Histórico de métricas

#### Métricas Exportadas
- `nuclear_crawler_state`: Estado actual del crawler
- `nuclear_crawler_tor_connected`: Estado de conexión Tor
- `nuclear_crawler_storage_size_mb`: Tamaño de almacenamiento
- `nuclear_crawler_predictions_total`: Total de predicciones

## 📦 Arquitectura

```
nuclear_crawler/
├── mod.rs                     # Coordinador principal
├── auto_rebuild.rs            # Sistema FORCED_REBUILDS
├── deepweb_tor.rs            # Cliente Tor para DeepWeb
├── intelligent_storage.rs     # Almacenamiento inteligente
├── predictive_nodes.rs        # Nodos predictivos
├── deep_storage_tunnels.rs    # Túneles de almacenamiento
└── metrics_exporter.rs        # Exportador Prometheus
```

## 🔧 Configuración

### Básica

```rust
use memory_p::nuclear_crawler::{NuclearCrawler, CrawlerConfig};

let config = CrawlerConfig {
    enable_tor: true,                    // Habilitar Tor
    enable_intelligent_storage: true,     // Storage inteligente
    enable_predictive_nodes: true,        // Nodos predictivos
    auto_rebuild_interval: 300,           // Rebuild cada 5 min
    parallel_buffer_size: 1024,           // Buffer de 1024 items
    security_level: 3,                    // Nivel de seguridad (1-5)
};

let crawler = NuclearCrawler::new(config);
```

### Avanzada

```rust
// Iniciar crawler
crawler.start().await?;

// Realizar búsqueda con auto-corrección
let results = crawler.search("query ejemplo").await?;

// Obtener estadísticas
let stats = crawler.get_stats();
println!("{}", serde_json::to_string_pretty(&stats)?);

// Exportar métricas Prometheus
let metrics = crawler.export_prometheus_metrics();
println!("{}", metrics);

// Detener crawler
crawler.stop().await?;
```

## 🔐 Niveles de Seguridad

El sistema soporta 5 niveles de seguridad:

| Nivel | Descripción | Características |
|-------|-------------|-----------------|
| 1     | Básico      | Sin Tor, sin cifrado extra |
| 2     | Estándar    | Cifrado básico, logs limitados |
| 3     | **Medio**   | Tor opcional, almacenamiento seguro |
| 4     | Alto        | Tor requerido, cifrado fuerte |
| 5     | **Máximo**  | ROOT-only, auditoría completa |

## 🚦 CI/CD Workflows

### Validación Automática

**Archivo**: `.github/workflows/nuclear-crawler-validation.yml`

Ejecuta en cada push/PR:
- ✅ Verificación de formato (rustfmt)
- ✅ Linting (clippy)
- ✅ Compilación release
- ✅ Tests unitarios
- ✅ Verificación de seguridad

### Auto-Merge Seguro

**Archivo**: `.github/workflows/nuclear-crawler-automerge.yml`

Condiciones para auto-merge:
- ✅ PR de rama autorizada (`feature/nuclear-crawler-*`)
- ✅ Sin cambios críticos detectados
- ✅ Todas las validaciones pasadas
- ✅ Autor es el owner del repositorio

## 📊 Monitoreo

### Docker Compose

Ya está configurado en `docker-compose.yml`:

```bash
# Iniciar todos los servicios
docker-compose up -d

# Ver logs del crawler
docker-compose logs -f memory-p

# Acceder a servicios
# - Grafana: http://localhost:3000 (admin/admin)
# - Prometheus: http://localhost:9090
# - MEMORY_P API: http://localhost:4040
```

### Métricas en Prometheus

```bash
# Ver métricas del crawler
curl http://localhost:4040/metrics/nuclear-crawler

# Query en Prometheus
nuclear_crawler_state{state="Running"}
nuclear_crawler_storage_size_mb
rate(nuclear_crawler_predictions_total[5m])
```

### Dashboards en Grafana

1. Abrir Grafana: http://localhost:3000
2. Login: admin/admin
3. Agregar Data Source: Prometheus (http://prometheus:9090)
4. Importar dashboards de `config/grafana-dashboards/`

## 🧪 Testing

```bash
# Tests unitarios del módulo
cargo test --package memory_p --lib nuclear_crawler

# Test específico
cargo test --package memory_p --lib nuclear_crawler::tests::test_crawler_lifecycle

# Con output
cargo test --package memory_p --lib nuclear_crawler -- --nocapture
```

## 📝 Ejemplo Completo

```rust
use memory_p::nuclear_crawler::{NuclearCrawler, CrawlerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configurar
    let config = CrawlerConfig {
        enable_tor: true,
        enable_intelligent_storage: true,
        enable_predictive_nodes: true,
        auto_rebuild_interval: 300,
        parallel_buffer_size: 1024,
        security_level: 4,
    };

    // 2. Crear crawler
    let crawler = NuclearCrawler::new(config);

    // 3. Iniciar (auto-gestión activa)
    crawler.start().await?;
    println!("✅ Nuclear Crawler iniciado");

    // 4. Realizar búsquedas con auto-corrección
    match crawler.search("rust async programming").await {
        Ok(results) => {
            println!("📦 Resultados: {} encontrados", results.len());
            for result in results {
                println!("  - {}", result);
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    // 5. Monitorear estado
    let stats = crawler.get_stats();
    println!("📊 Stats: {}", serde_json::to_string_pretty(&stats)?);

    // 6. Exportar métricas
    let metrics = crawler.export_prometheus_metrics();
    println!("📈 Métricas Prometheus:\n{}", metrics);

    // 7. Detener
    crawler.stop().await?;
    println!("🛑 Nuclear Crawler detenido");

    Ok(())
}
```

## 🔄 FORCED_REBUILDS

El sistema de auto-rebuild se ejecuta en background:

```rust
// En lib.rs ya está configurado:
// pub mod nuclear_crawler;

// FORCED_REBUILDS: Sistema de auto-ajuste de módulos
// Los módulos se activan/desactivan automáticamente según métricas
// Ver: nuclear_crawler::auto_rebuild para configuración dinámica
```

Los módulos se reconstruyen automáticamente cada `auto_rebuild_interval` segundos, ajustando su estado basado en:
- Uso de recursos
- Tasa de errores
- Prioridad asignada
- Métricas de rendimiento

## 🚀 Roadmap

### Fase 1 (✅ Completada)
- [x] Módulo nuclear_crawler base
- [x] FORCED_REBUILDS system
- [x] DeepWeb Tor integration
- [x] Intelligent Storage
- [x] Predictive Nodes
- [x] Deep Storage Tunnels
- [x] Metrics Exporter

### Fase 2 (✅ Completada)
- [x] GitHub Actions workflows
- [x] Auto-merge seguro
- [x] Prometheus configuration
- [x] Grafana dashboards setup

### Fase 3 (Futuro)
- [ ] Machine learning para predicciones
- [ ] Distributed crawler nodes
- [ ] Advanced anomaly detection
- [ ] GraphQL API para métricas
- [ ] Real-time alerting system

## 📚 Referencias

- [MCP Protocol 2024-11-05](https://modelcontextprotocol.io)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [Tor Project](https://www.torproject.org/)
- [Rayon Parallel Processing](https://github.com/rayon-rs/rayon)

## 🤝 Contribución

Ver [CONTRIBUTING.md](../CONTRIBUTING.md) para guías de contribución.

## 📄 Licencia

MIT License - Ver [LICENSE](../LICENSE)

---

**MEMORY_P v2.0** - Always-On MCP Server with Nuclear Crawler Hybrid System
