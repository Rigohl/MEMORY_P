// [nuclear_god_mode] PROCESSED AT MAX SPEED
use crate::analyzer::CodeAnalyzer;
use crate::mcp::models::*;
use crate::workspace;
use axum::{
    response::sse::{Event, Sse},
    response::IntoResponse,
    Json,
};
use futures::stream::{self, Stream, StreamExt};
use serde_json::{json, Value};
use std::convert::Infallible;
use std::process::Command;

pub async fn mcp_descriptor_handler() -> impl IntoResponse {
    let descriptor = McpDescriptor {
        name: "MEMORY_P NUCLEAR MCP",
        version: "2025.2.ULTRA",
        description: "Motor de procesamiento masivo nativo para Cursor, Windsurf y VS Code.",
    };
    Json(descriptor)
}

pub async fn mcp_sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| Event::default().data("connected")).map(Ok);

    Sse::new(stream)
}

pub async fn create_project_handler(
    Json(payload): Json<CreateProjectRequest>,
) -> Json<CreateProjectResponse> {
    let base_path = std::path::Path::new(&payload.path);
    if let Err(e) = std::fs::create_dir_all(base_path) {
        return Json(CreateProjectResponse {
            status: format!("Error al crear directorio base: {}", e),
            created_files: vec![],
        });
    }

    let project_path = base_path.join(&payload.name);
    if project_path.exists() {
        let err =
            crate::error::MemoryPError::Other(format!("El proyecto '{}' ya existe", payload.name));
        return Json(CreateProjectResponse {
            status: format!("Error: {}", err),
            created_files: vec![],
        });
    }
    let mut created_files = Vec::new();

    match payload.template.as_str() {
        "rust" => {
            let src_path = project_path.join("src");
            if let Err(e) = std::fs::create_dir_all(&src_path) {
                return Json(CreateProjectResponse {
                    status: format!("Error al crear directorio: {}", e),
                    created_files: vec![],
                });
            }

            let cargo_toml = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
"#,
                payload.name
            );
            let main_rs = r#"fn main() {
    println!("Hello from MEMORY_P generated project!");
}
"#;
            if let Err(e) = std::fs::write(project_path.join("Cargo.toml"), &cargo_toml) {
                return Json(CreateProjectResponse {
                    status: format!("Error al escribir Cargo.toml: {}", e),
                    created_files: vec![],
                });
            }
            if let Err(e) = std::fs::write(src_path.join("main.rs"), main_rs) {
                return Json(CreateProjectResponse {
                    status: format!("Error al escribir main.rs: {}", e),
                    created_files: vec![],
                });
            }
            created_files.push("Cargo.toml".into());
            created_files.push("src/main.rs".into());
        }
        "mcp" => {
            let src_path = project_path.join("src");
            if let Err(e) = std::fs::create_dir_all(&src_path) {
                return Json(CreateProjectResponse {
                    status: format!("Error al crear directorio: {}", e),
                    created_files: vec![],
                });
            }
            let cargo_toml = format!(
                r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
tower-http = {{ version = "0.5", features = ["cors"] }}
"#,
                payload.name
            );
            let main_rs = r#"use axum::{routing::post, Json, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/mcp", post(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4040").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(Json(req): Json<Value>) -> Json<Value> {
    Json(json!({"jsonrpc": "2.0", "result": "ok", "id": req.get("id")}))
}
"#;
            if let Err(e) = std::fs::write(project_path.join("Cargo.toml"), &cargo_toml) {
                return Json(CreateProjectResponse {
                    status: format!("Error al escribir Cargo.toml: {}", e),
                    created_files: vec![],
                });
            }
            if let Err(e) = std::fs::write(src_path.join("main.rs"), main_rs) {
                return Json(CreateProjectResponse {
                    status: format!("Error al escribir main.rs: {}", e),
                    created_files: vec![],
                });
            }
            created_files.push("Cargo.toml".into());
            created_files.push("src/main.rs".into());
        }
        "mojo" => {
            let src_path = project_path.join("src");
            std::fs::create_dir_all(&src_path).ok();
            let hello_mojo = format!(
                r#"fn main():
    print("🔥 Hello from Mojo AI Project: {}")
"#,
                payload.name
            );
            std::fs::write(src_path.join("hello.mojo"), hello_mojo).ok();
            created_files.push("src/hello.mojo".into());
        }
        "python" => {
            let src_path = project_path.join("src");
            std::fs::create_dir_all(&src_path).ok();
            let main_py = format!(
                r#"import jax
import jax.numpy as jnp

def main():
    print("🐍 Hello from Python JAX Project: {}")
    x = jnp.zeros((3, 3))
    print(f"JAX Array: \n{{x}}")

if __name__ == "__main__":
    main()
"#,
                payload.name
            );
            std::fs::write(src_path.join("main.py"), main_py).ok();
            created_files.push("src/main.py".into());
        }
        _ => {
            let err = crate::error::MemoryPError::Unsupported(payload.template.clone());
            return Json(CreateProjectResponse {
                status: format!("Error: {}", err),
                created_files: vec![],
            });
        }
    }

    Json(CreateProjectResponse {
        status: "Created".into(),
        created_files,
    })
}

pub async fn analyze_project_handler(Json(payload): Json<ProjectRequest>) -> Json<ProjectResponse> {
    let ext = payload.extension.as_deref().unwrap_or("rs");
    let config = crate::parallel_engine::ParallelConfig::default();

    match CodeAnalyzer::scan_files(&payload.path, ext, true, false) {
        Ok(paths) => match crate::parallel_engine::ultra_analyze(&paths, config) {
            Ok((results, _stats)) => {
                let formatted: Vec<Value> = results
                    .into_iter()
                    .map(|r| json!(format!("{}: [{}]", r.path, r.findings.join(", "))))
                    .collect();
                Json(ProjectResponse {
                    status: "Done".into(),
                    results: formatted,
                })
            }
            Err(e) => Json(ProjectResponse {
                status: "Error".into(),
                results: vec![json!(format!("Error de procesamiento: {}", e))],
            }),
        },
        Err(e) => Json(ProjectResponse {
            status: "Error".into(),
            results: vec![json!(format!("Error de escaneo: {}", e))],
        }),
    }
}

pub async fn edit_project_handler(Json(payload): Json<ProjectRequest>) -> Json<ProjectResponse> {
    let ext = payload.extension.as_deref().unwrap_or("rs");
    let config = crate::parallel_engine::ParallelConfig::default();

    match CodeAnalyzer::scan_files(&payload.path, ext, true, false) {
        Ok(paths) => {
            // Convertimos paths a FileChanges genéricos para el motor de edición masiva
            let changes: Vec<FileChange> = paths
                .iter()
                .map(|p| FileChange {
                    path: p.to_string_lossy().to_string(),
                    operations: vec![EditOp::Replace {
                        target: "\t".to_string(),
                        replacement: "    ".to_string(),
                    }], // Ejemplo de normalización base
                })
                .collect();

            match crate::parallel_engine::ultra_edit(&changes, config, false) {
                Ok((results, _stats)) => {
                    let formatted: Vec<Value> = results
                        .into_iter()
                        .map(|r| json!(format!("{}: {:?}", r.path, r.status)))
                        .collect();
                    Json(ProjectResponse {
                        status: "Done".into(),
                        results: formatted,
                    })
                }
                Err(e) => Json(ProjectResponse {
                    status: "Error".into(),
                    results: vec![json!(format!("Error de procesamiento: {}", e))],
                }),
            }
        }
        Err(e) => Json(ProjectResponse {
            status: "Error".into(),
            results: vec![json!(format!("Error de escaneo: {}", e))],
        }),
    }
}

pub async fn repair_project_handler(Json(payload): Json<ProjectRequest>) -> Json<ProjectResponse> {
    let ext = payload.extension.as_deref().unwrap_or("rs");
    match CodeAnalyzer::scan_files(&payload.path, ext, true, false) {
        Ok(paths) => match workspace::process_parallel(&paths, workspace::repair_file) {
            Ok(results) => {
                let formatted: Vec<Value> = results
                    .into_iter()
                    .map(|r| match r {
                        Ok(m) => json!(m),
                        Err(e) => json!(format!("ERROR: {}", e)),
                    })
                    .collect();
                Json(ProjectResponse {
                    status: "Done".into(),
                    results: formatted,
                })
            }
            Err(e) => Json(ProjectResponse {
                status: "Error".into(),
                results: vec![json!(format!("Error de procesamiento: {}", e))],
            }),
        },
        Err(e) => Json(ProjectResponse {
            status: "Error".into(),
            results: vec![json!(format!("Error de escaneo: {}", e))],
        }),
    }
}

pub async fn ultra_engine_handler(Json(payload): Json<UltraRequest>) -> Json<UltraResponse> {
    let tasks = payload.max_tasks.unwrap_or(rayon::current_num_threads());
    let dry = payload.dry_run.unwrap_or(false);
    let out = execute_ultra_wsl(
        &payload.target_dir,
        payload.file_extension.as_deref().unwrap_or("rs"),
        tasks,
        dry,
    )
    .await;

    Json(UltraResponse {
        status: if out.contains("Error") {
            "Error".into()
        } else {
            "Success".into()
        },
        engine_output: out,
    })
}

pub async fn execute_ultra_wsl(target_dir: &str, ext: &str, tasks: usize, dry: bool) -> String {
    let target = target_dir.to_string();
    let ext = ext.to_string();

    let res = tokio::task::spawn_blocking(move || {
        let wsl_path_out = Command::new("wsl").args(["wslpath", "-u", &target]).output();
        let wsl_path = match wsl_path_out {
            Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
            _ => return "Error: No se pudo convertir la ruta a WSL.".to_string(),
        };

        let cmd = format!(
            "chpl {wsl_path}/src/ultra_engine.chpl -o {wsl_path}/engine --fast && {wsl_path}/engine --targetDir={wsl_path} --fileExtension={ext} --maxTasks={tasks} --dryRun={dry}"
        );

        let out = Command::new("wsl").args(["bash", "-c", &cmd]).output();
        match out {
            Ok(o) => format!("STDOUT:\n{}\nSTDERR:\n{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)),
            Err(e) => format!("Error en ejecución WSL: {}", e),
        }
    }).await.unwrap_or_else(|_| "Error: Task join failed".into());

    res
}
