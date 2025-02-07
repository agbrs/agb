use std::{
    fs, io,
    ops::Deref,
    path::{Path, PathBuf},
    process::Command,
};

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
                .compact(),
        )
        .init();

    let state = Builder {
        temp_path: PathBuf::from("/tmp/agb-build"),
    };

    let app = Router::new()
        .route("/build", post(compile))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn compile(
    State(builder): State<Builder>,
    Json(arguments): Json<CompileArguments>,
) -> Result<Vec<u8>, (StatusCode, Json<CompileResponse>)> {
    tokio::task::spawn_blocking(move || builder.build(&arguments.code))
        .await
        .map_err(|join_error| {
            tracing::error!(error = join_error.to_string(), "Build panicked");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CompileResponse {
                    error: "Unknown error occurred".to_string(),
                }),
            )
        })?
        .map_err(|e| match e {
            CompileError::CompileFail(output) => (
                StatusCode::BAD_REQUEST,
                Json(CompileResponse { error: output }),
            ),
            CompileError::UnknownError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(CompileResponse {
                    error: "Unknown error occurred".to_string(),
                }),
            ),
        })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompileArguments {
    code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompileResponse {
    error: String,
}

#[derive(Debug, Clone)]
struct Builder {
    temp_path: PathBuf,
}

impl Builder {
    fn build(&self, rust_code: &str) -> Result<Vec<u8>, CompileError> {
        let id: Uuid = Uuid::new_v4();

        let _span = tracing::info_span!("compile code", id = id.to_string()).entered();

        let temp_folder =
            TempFolder::new(id, &self.temp_path).map_err(|_| CompileError::UnknownError)?;

        if let Err(e) = fs::write(temp_folder.join("main.rs"), rust_code) {
            tracing::error!(error = e.to_string(), "Failed to write main.rs");
            return Err(CompileError::UnknownError);
        }

        let mut permissions_command = Command::new("chmod");
        permissions_command
            .args(["-R", "777"])
            .arg(temp_folder.as_os_str());

        if let Err(e) = permissions_command.status() {
            tracing::error!(
                error = e.to_string(),
                "Failed to set permissions for temp folder {}",
                temp_folder.display()
            );
            return Err(CompileError::UnknownError);
        }

        let mut launch_command = Command::new("timeout");

        launch_command
            .args([
                "30s",
                "docker",
                "run",
                "--cap-drop=ALL",
                "--net=none",
                "--memory=256m",
                "--memory-swap=512m",
                "--pids-limit=512",
                "--oom-score-adj=1000",
                "--rm",
                "-v",
            ])
            .arg(format!("{}:/out", temp_folder.display()))
            .args(["-i", "agb-build:latest"]);

        match launch_command.output() {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    tracing::info!("Compile output: {stderr}");
                    return Err(CompileError::CompileFail(stderr.to_string()));
                }
            }
            Err(e) => {
                tracing::error!(error = e.to_string(), "Failed to launch docker: {e}");
                return Err(CompileError::UnknownError);
            }
        };

        let result = fs::read(temp_folder.join("agb.gba.gz")).map_err(|e| {
            tracing::error!(
                error = e.to_string(),
                "Failed to read output for {}",
                temp_folder.display()
            );
            CompileError::UnknownError
        })?;

        tracing::info!("Built rom with size {}kb", result.len() / 1000);

        Ok(result)
    }
}

enum CompileError {
    CompileFail(String),
    UnknownError,
}

struct TempFolder {
    path: PathBuf,
}

impl TempFolder {
    fn new(id: Uuid, temp_path: &Path) -> Result<Self, io::Error> {
        let temp_directory = temp_path.join(format!("agb-compile-{id}"));

        if let Err(e) = fs::create_dir(&temp_directory) {
            tracing::error!(
                error = e.to_string(),
                "Failed to create folder: {}",
                temp_directory.display()
            );
            return Err(e);
        }

        Ok(Self {
            path: temp_directory,
        })
    }
}

impl Deref for TempFolder {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_dir_all(&self.path) {
            tracing::error!(
                error = e.to_string(),
                "Failed to deleted folder {}",
                self.path.display(),
            );
        }
    }
}
