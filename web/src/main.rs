use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use bengal_compiler::compiler::{Compiler, CompilerOptions};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod bytecode_viewer;

#[tokio::main]
async fn main() {
    let app_state = Arc::new(AppState {});

    let app = Router::new()
        .route("/", get(serve_frontend))
        .route("/compile", post(compile_code))
        .route("/disassemble", post(disassemble_code))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Bengal Godbolt running on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

struct AppState {}

#[derive(Debug, Deserialize)]
struct CompileRequest {
    source: String,
    unsafe_fast: Option<bool>,
}

#[derive(Debug, Serialize)]
struct CompileResponse {
    success: bool,
    error: Option<String>,
    assembly: Option<String>,
}

async fn serve_frontend() -> impl IntoResponse {
    Html(include_str!("../frontend.html"))
}

async fn compile_code(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    let unsafe_fast = req.unsafe_fast.unwrap_or(false);
    
    let mut compiler = Compiler::with_path_and_options(&req.source, "<input>", unsafe_fast);
    compiler.enable_type_checking = false;
    
    // Add std search path - try multiple locations
    if let Ok(current_dir) = std::env::current_dir() {
        compiler.search_paths.push(current_dir.join("../std").to_string_lossy().to_string());
        compiler.search_paths.push(current_dir.join("std").to_string_lossy().to_string());
    }
    // Also try relative to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            compiler.search_paths.push(parent.join("../std").to_string_lossy().to_string());
            compiler.search_paths.push(parent.join("std").to_string_lossy().to_string());
        }
    }
    
    // Use compile_with_options to properly disable type checking
    let options = CompilerOptions {
        enable_type_checking: false,
        search_paths: compiler.search_paths.clone(),
        unsafe_fast,
    };
    
    match compiler.compile_with_options(&options) {
        Ok(bytecode) => {
            let assembly = bytecode_viewer::display_bytecode(&bytecode);
            (
                StatusCode::OK,
                Json(CompileResponse {
                    success: true,
                    error: None,
                    assembly: Some(assembly),
                }),
            )
        }
        Err(e) => (
            StatusCode::OK,
            Json(CompileResponse {
                success: false,
                error: Some(e.to_string()),
                assembly: None,
            }),
        ),
    }
}

async fn disassemble_code(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CompileRequest>,
) -> impl IntoResponse {
    // Same as compile_code, kept for API compatibility
    compile_code(State(_state), Json(req)).await
}
