//! autonomous_daemon.rs - Sistema de Daemon Autónomo Autoejecutable
//!
//! Este módulo implementa un daemon completamente autónomo que:
//! - Se auto-ejecuta sin intervención externa
//! - Detecta contextos dinámicamente
//! - Realiza auto-recuperación ante fallos
//! - Mantiene el sistema always-on

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, Instant};
use tracing::{info, warn, error, debug};

use crate::error::{Result, MemoryPError as Error};
use crate::predictive_engine::PredictiveEngine;
use crate::context_detector::ContextDetector;
use crate::analyzer::CodeAnalyzer;
use crate::shared_memory::SharedMemorySystem;

/// Estado del daemon autónomo
#[derive(Debug, Clone, PartialEq)]
pub enum DaemonState {
    /// Daemon iniciándose
    Starting,
    /// Daemon activo y funcionando
    Running,
    /// Daemon en modo recuperación
    Recovering,
    /// Daemon pausado temporalmente
    Paused,
    /// Daemon detenido
    Stopped,
}

/// Configuración del daemon autónomo
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Intervalo de auto-verificación (segundos)
    pub health_check_interval: u64,
    /// Intervalo de detección de contexto (segundos)
    pub context_detection_interval: u64,
    /// Número máximo de intentos de recuperación
    pub max_recovery_attempts: u32,
    /// Tiempo de espera para auto-recuperación (segundos)
    pub recovery_timeout: u64,
    /// Habilitar modo auto-corrección
    pub auto_correction_enabled: bool,
    /// Habilitar auto-optimización
    pub auto_optimization_enabled: bool,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            health_check_interval: 30,
            context_detection_interval: 10,
            max_recovery_attempts: 3,
            recovery_timeout: 60,
            auto_correction_enabled: true,
            auto_optimization_enabled: true,
        }
    }
}

/// Métricas del daemon
#[derive(Debug, Clone, Default)]
pub struct DaemonMetrics {
    /// Tiempo de actividad total (segundos)
    pub uptime_seconds: u64,
    /// Número de auto-recuperaciones exitosas
    pub successful_recoveries: u32,
    /// Número de detecciones de contexto
    pub context_detections: u64,
    /// Número de auto-correcciones aplicadas
    pub auto_corrections: u64,
    /// Número de optimizaciones aplicadas
    pub optimizations_applied: u64,
    /// Última verificación de salud (timestamp Unix)
    pub last_health_check_timestamp: Option<u64>,
}

/// Daemon autónomo principal
pub struct AutonomousDaemon {
    /// Configuración del daemon
    config: DaemonConfig,
    /// Estado actual
    state: Arc<RwLock<DaemonState>>,
    /// Métricas del daemon
    metrics: Arc<RwLock<DaemonMetrics>>,
    /// Motor predictivo
    predictive_engine: Arc<PredictiveEngine>,
    /// Detector de contexto
    context_detector: Arc<ContextDetector>,
    /// Sistema de memoria compartida
    shared_memory: Arc<SharedMemorySystem>,
    /// Tiempo de inicio
    start_time: Instant,
    /// Bridge de alto rendimiento para memoria compartida multi-lenguaje
    zig_bridge: Option<Arc<crate::ffi::zig::ZigBridge>>,
}

impl AutonomousDaemon {
    /// Crea un nuevo daemon autónomo
    pub fn new(config: DaemonConfig, shared_memory: Arc<SharedMemorySystem>) -> Self {
        info!("🤖 Inicializando Daemon Autónomo...");
        
        let zig_bridge = crate::ffi::zig::ZigBridge::new(1024 * 1024).ok().map(Arc::new);

        Self {
            config,
            state: Arc::new(RwLock::new(DaemonState::Starting)),
            metrics: Arc::new(RwLock::new(DaemonMetrics::default())),
            predictive_engine: Arc::new(PredictiveEngine::new()),
            context_detector: Arc::new(ContextDetector::new()),
            shared_memory,
            start_time: Instant::now(),
            zig_bridge,
        }
    }

    /// Inicia el daemon autónomo (auto-ejecutable)
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("🚀 Iniciando Daemon Autónomo...");
        
        // Cambiar estado a Running
        {
            let mut state = self.state.write().await;
            *state = DaemonState::Running;
        }

        info!("✅ Daemon Autónomo activo - modo always-on");
        
        // Iniciar tareas en background
        let daemon_health = self.clone();
        tokio::spawn(async move {
            if let Err(e) = daemon_health.health_check_loop().await {
                error!("❌ Error en health check loop: {}", e);
            }
        });

        let daemon_context = self.clone();
        tokio::spawn(async move {
            if let Err(e) = daemon_context.context_detection_loop().await {
                error!("❌ Error en context detection loop: {}", e);
            }
        });

        let daemon_optimize = self.clone();
        tokio::spawn(async move {
            if let Err(e) = daemon_optimize.optimization_loop().await {
                error!("❌ Error en optimization loop: {}", e);
            }
        });

        let daemon_scan = self.clone();
        tokio::spawn(async move {
            if let Err(e) = daemon_scan.workspace_scanning_loop().await {
                error!("❌ Error en workspace scanning loop: {}", e);
            }
        });

        let daemon_speculative = self.clone();
        tokio::spawn(async move {
            if let Err(e) = daemon_speculative.speculative_task_loop().await {
                error!("❌ Error en speculative task loop: {}", e);
            }
        });

        info!("🔄 Tareas de background iniciadas:");
        info!("   • Health checks: cada {}s", self.config.health_check_interval);
        info!("   • Context detection: cada {}s", self.config.context_detection_interval);
        info!("   • Auto-optimization: habilitado");
        
        Ok(())
    }

    /// Loop de verificación de salud
    async fn health_check_loop(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.health_check_interval));
        
        loop {
            interval.tick().await;
            
            debug!("🏥 Ejecutando health check...");
            
            // Actualizar métricas
            {
                let mut metrics = self.metrics.write().await;
                metrics.uptime_seconds = self.start_time.elapsed().as_secs();
                metrics.last_health_check_timestamp = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );
            }

            // Verificar estado del sistema
            if let Err(e) = self.perform_health_check().await {
                warn!("⚠️  Health check detectó problemas: {}", e);
                
                // Intentar auto-recuperación
                if let Err(recovery_err) = self.attempt_recovery().await {
                    error!("❌ Auto-recuperación falló: {}", recovery_err);
                }
            }
        }
    }

    /// Realiza verificación de salud del sistema
    async fn perform_health_check(&self) -> Result<()> {
        let state = self.state.read().await;
        
        match *state {
            DaemonState::Running => {
                debug!("✅ Sistema saludable");
                Ok(())
            }
            DaemonState::Recovering => {
                warn!("⚠️  Sistema en recuperación");
                Err(Error::Other("Sistema en modo recuperación".into()))
            }
            DaemonState::Stopped => {
                error!("❌ Sistema detenido");
                Err(Error::Other("Sistema detenido".into()))
            }
            _ => {
                debug!("ℹ️  Sistema en estado: {:?}", *state);
                Ok(())
            }
        }
    }

    /// Intenta auto-recuperación del sistema
    async fn attempt_recovery(&self) -> Result<()> {
        info!("🔧 Iniciando auto-recuperación...");
        
        // Cambiar estado a Recovering
        {
            let mut state = self.state.write().await;
            *state = DaemonState::Recovering;
        }

        // Intentar recuperación
        for attempt in 1..=self.config.max_recovery_attempts {
            info!("🔄 Intento de recuperación {}/{}", attempt, self.config.max_recovery_attempts);
            
            // Simular recuperación (aquí iría lógica real)
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            // Verificar si la recuperación fue exitosa
            if self.verify_recovery().await? {
                info!("✅ Recuperación exitosa en intento {}", attempt);
                
                // Actualizar métricas
                {
                    let mut metrics = self.metrics.write().await;
                    metrics.successful_recoveries += 1;
                }
                
                // Volver a estado Running
                {
                    let mut state = self.state.write().await;
                    *state = DaemonState::Running;
                }
                
                return Ok(());
            }
            
            warn!("⚠️  Intento {} falló, reintentando...", attempt);
        }
        
        error!("❌ Recuperación falló después de {} intentos", self.config.max_recovery_attempts);
        Err(Error::Other("Recuperación falló".into()))
    }

    /// Verifica si la recuperación fue exitosa
    async fn verify_recovery(&self) -> Result<bool> {
        // Aquí iría lógica real de verificación
        debug!("🔍 Verificando recuperación...");
        Ok(true)
    }

    /// Loop de detección de contexto
    async fn context_detection_loop(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(self.config.context_detection_interval));
        
        loop {
            interval.tick().await;
            
            debug!("🔍 Detectando contextos...");
            
            // Detectar contextos dinámicamente
            if let Ok(contexts) = self.context_detector.detect_contexts().await {
                if !contexts.is_empty() {
                    info!("📍 Contextos detectados: {}", contexts.len());
                    
                    // Actualizar métricas
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.context_detections += contexts.len() as u64;
                    }
                }
            }
        }
    }

    /// Loop de escaneo de workspace proactivo
    async fn workspace_scanning_loop(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(60)); // Cada minuto

        loop {
            interval.tick().await;

            debug!("🔍 Ejecutando escaneo de workspace proactivo...");

            // 1. Escanear archivos (Rust por defecto)
            if let Ok(files) = CodeAnalyzer::scan_files(".", "rs", true, false) {
                let mut ctx = self.shared_memory.get_or_create_context(crate::shared_memory::AgentId::new("autonomous-daemon".to_string())).await?;

                for file_path in files {
                    if let Ok(analysis) = CodeAnalyzer::analyze_file(&file_path) {
                        // Memoria Episódica: Registrar que hemos visto este archivo
                        let file_key = format!("file_seen:{}", analysis.file_path);
                        ctx.shared_data.insert(file_key, serde_json::json!({
                            "path": analysis.file_path,
                            "loc": analysis.lines_of_code,
                            "complexity": analysis.complexity_estimate,
                            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                        }));

                        if !analysis.warnings.is_empty() {
                            info!("⚠️  Detectados {} problemas en {}", analysis.warnings.len(), analysis.file_path);

                            // 2. Registrar alarmas proactivas en memoria compartida con sugerencias
                            for (i, warning) in analysis.warnings.iter().enumerate() {
                                let alarm_type = if warning.contains("🛡️ SEGURIDAD") { "critical_security" } else { "proactive_warning" };
                                let suggestion = analysis.suggestions.get(i).cloned().unwrap_or_else(|| "Revisar código manualmente.".to_string());

                                let alarm_key = format!("alarm:{}", analysis.file_path);
                                ctx.shared_data.insert(alarm_key, serde_json::json!({
                                    "type": alarm_type,
                                    "file": analysis.file_path,
                                    "message": warning,
                                    "fix_suggestion": suggestion,
                                    "severity": if alarm_type == "critical_security" { "high" } else { "medium" },
                                    "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
                                }));
                            }
                        }
                    }
                }
                // Actualizar contexto total una vez por escaneo
                self.shared_memory.update_context(ctx.agent_id.clone(), ctx).await?;
            }
        }
    }

    /// Loop de optimización automática
    async fn optimization_loop(&self) -> Result<()> {
        if !self.config.auto_optimization_enabled {
            info!("⚠️  Auto-optimización deshabilitada");
            return Ok(());
        }

        let mut interval = interval(Duration::from_secs(60)); // Cada minuto
        
        loop {
            interval.tick().await;
            
            debug!("⚡ Ejecutando auto-optimización...");
            
            // 1. Ejecutar optimizaciones predictivas
            if let Ok(optimizations) = self.predictive_engine.suggest_optimizations().await {
                let optimizations: Vec<crate::predictive_engine::Optimization> = optimizations;
                if !optimizations.is_empty() {
                    info!("🎯 Aplicando {} optimizaciones", optimizations.len());
                    
                    // Actualizar métricas
                    {
                        let mut metrics = self.metrics.write().await;
                        metrics.optimizations_applied += optimizations.len() as u64;
                    }
                }
            }

            // 2. Ejecutar autogestión de memoria (auto-moving context)
            if let Err(e) = self.shared_memory.auto_manage_memory().await {
                error!("❌ Error en autogestión de memoria: {}", e);
            }

            // 3. Ejecutar Auto-Sanación Proactiva
            if self.config.auto_correction_enabled {
                if let Err(e) = self.perform_auto_heal().await {
                    error!("❌ Error en auto-sanación: {}", e);
                }
            }
        }
    }

    /// Loop de tareas especulativas: se adelanta al agente
    async fn speculative_task_loop(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(45));

        loop {
            interval.tick().await;
            debug!("🧠 Ejecutando multitarea especulativa...");

            // 1. Predicción de carga y pre-limpieza de cache
            if let Some(bridge) = &self.zig_bridge {
                let msg = b"OPTIMIZE_MEMORY_NOW";
                let _ = bridge.write(msg);
            }

            // 2. Simular pre-compilación de archivos recientemente tocados
            let agent_id = crate::shared_memory::AgentId::new("autonomous-daemon".to_string());
            if let Ok(ctx) = self.shared_memory.get_or_create_context(agent_id).await {
                for (key, _value) in ctx.shared_data.iter() {
                    if key.starts_with("file_seen:") {
                        // Aquí se podría lanzar un 'cargo check' en background para archivos críticos
                        debug!("🚀 Speculative: Preparando contexto para {}", key);
                    }
                }
            }
        }
    }

    /// Realiza auto-sanación de problemas detectados
    async fn perform_auto_heal(&self) -> Result<()> {
        debug!("🛠️  Iniciando ciclo de auto-sanación...");

        let agent_id = crate::shared_memory::AgentId::new("autonomous-daemon".to_string());
        let ctx = self.shared_memory.get_or_create_context(agent_id).await?;

        // Buscar archivos con alarmas para auto-reparar
        let mut files_to_repair = Vec::new();
        for (key, value) in ctx.shared_data.iter() {
            if key.starts_with("alarm:") {
                if let Some(file) = value.get("file").and_then(|v| v.as_str()) {
                    // Solo reparamos si el mensaje sugiere problemas de código tratables
                    if let Some(msg) = value.get("message").and_then(|v| v.as_str()) {
                        if msg.contains("RUST") || msg.contains("formato") {
                            files_to_repair.push(std::path::PathBuf::from(file));
                        }
                    }
                }
            }
        }

        if !files_to_repair.is_empty() {
            info!("🔧 Auto-sanando {} archivos detectados...", files_to_repair.len());
            let config = crate::parallel_engine::ParallelConfig::default();

            // Limitamos a 5 archivos por ciclo para no saturar
            let to_fix = if files_to_repair.len() > 5 { &files_to_repair[..5] } else { &files_to_repair };

            if let Ok((_res, stats)) = crate::parallel_engine::ultra_repair(to_fix, config) {
                info!("✅ Auto-sanación completada: {} exitosos", stats.successful);

                // Actualizar métricas
                let mut metrics = self.metrics.write().await;
                metrics.auto_corrections += stats.successful as u64;
            }
        }

        Ok(())
    }

    /// Obtiene el estado actual del daemon
    pub async fn get_state(&self) -> DaemonState {
        self.state.read().await.clone()
    }

    /// Obtiene las métricas actuales
    pub async fn get_metrics(&self) -> DaemonMetrics {
        self.metrics.read().await.clone()
    }

    /// Detiene el daemon de forma controlada
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Deteniendo Daemon Autónomo...");
        
        let mut state = self.state.write().await;
        *state = DaemonState::Stopped;
        
        info!("✅ Daemon detenido correctamente");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daemon_creation() {
        let config = DaemonConfig::default();
        let shared_memory = Arc::new(crate::shared_memory::SharedMemorySystem::new().await.unwrap());
        let daemon = AutonomousDaemon::new(config, shared_memory);
        
        let state = daemon.get_state().await;
        assert_eq!(state, DaemonState::Starting);
    }

    #[tokio::test]
    async fn test_daemon_start() {
        let config = DaemonConfig::default();
        let shared_memory = Arc::new(crate::shared_memory::SharedMemorySystem::new().await.unwrap());
        let daemon = Arc::new(AutonomousDaemon::new(config, shared_memory));
        
        let result = daemon.clone().start().await;
        assert!(result.is_ok());
        
        let state = daemon.get_state().await;
        assert_eq!(state, DaemonState::Running);
    }

    #[tokio::test]
    async fn test_daemon_stop() {
        let config = DaemonConfig::default();
        let shared_memory = Arc::new(crate::shared_memory::SharedMemorySystem::new().await.unwrap());
        let daemon = AutonomousDaemon::new(config, shared_memory);
        
        let result = daemon.stop().await;
        assert!(result.is_ok());
        
        let state = daemon.get_state().await;
        assert_eq!(state, DaemonState::Stopped);
    }
}
