use std::{
    collections::HashMap,
    env, fs, io,
    ops::Deref,
    path::{Path, PathBuf},
    process::Command,
};

use axum::{
    Json, Router,
    extract::{Path as AxumPath, State},
    http::{Method, StatusCode, header::CONTENT_TYPE},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
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

    let github_token =
        env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN environment variable must be set");

    let state = AppState {
        temp_path: PathBuf::from("/run/agbrs-playground"),
        github_token,
        http_client: reqwest::Client::new(),
    };

    let app = Router::new()
        .route("/build", post(compile))
        .route("/gist", post(create_gist))
        .route("/gist/{id}", get(get_gist))
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::new()
                    .allow_methods([Method::POST, Method::GET])
                    .allow_origin(Any)
                    .allow_headers([CONTENT_TYPE]),
            ),
        )
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:5409").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Clone)]
struct AppState {
    temp_path: PathBuf,
    github_token: String,
    http_client: reqwest::Client,
}

async fn compile(
    State(state): State<AppState>,
    Json(arguments): Json<CompileArguments>,
) -> Result<Vec<u8>, (StatusCode, Json<CompileResponse>)> {
    let builder = Builder {
        temp_path: state.temp_path,
    };

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
struct CreateGistRequest {
    code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GistIdResponse {
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GistCodeResponse {
    code: String,
}

async fn create_gist(
    State(state): State<AppState>,
    Json(request): Json<CreateGistRequest>,
) -> Result<Json<GistIdResponse>, (StatusCode, String)> {
    if request.code.is_empty() || request.code.len() > 500_000 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Code must not be empty or exceed 500000 bytes".to_string(),
        ));
    }

    #[derive(Serialize)]
    struct GistFile {
        content: String,
    }

    #[derive(Serialize)]
    struct CreateGistBody {
        public: bool,
        files: HashMap<String, GistFile>,
    }

    let mut files = HashMap::new();
    files.insert(
        "main.rs".to_string(),
        GistFile {
            content: request.code,
        },
    );

    let body = CreateGistBody {
        public: false,
        files,
    };

    let response = state
        .http_client
        .post("https://api.github.com/gists")
        .bearer_auth(&state.github_token)
        .header("User-Agent", "agb-playground")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to call GitHub API");
            (StatusCode::BAD_GATEWAY, "Failed to create gist".to_string())
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(status = %status, body = %body, "GitHub API returned error");
        return Err((StatusCode::BAD_GATEWAY, "Failed to create gist".to_string()));
    }

    #[derive(Deserialize)]
    struct GitHubGistResponse {
        id: String,
    }

    let gist: GitHubGistResponse = response.json().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to parse GitHub response");
        (
            StatusCode::BAD_GATEWAY,
            "Failed to parse gist response".to_string(),
        )
    })?;

    tracing::info!(id = %gist.id, "Created gist");

    Ok(Json(GistIdResponse { id: gist.id }))
}

async fn get_gist(
    State(state): State<AppState>,
    AxumPath(id): AxumPath<String>,
) -> Result<Json<GistCodeResponse>, (StatusCode, String)> {
    tracing::info!(id = %id, "Fetching gist");

    if id.len() != 32 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid gist ID".to_string()));
    }

    let response = state
        .http_client
        .get(format!("https://api.github.com/gists/{id}"))
        .bearer_auth(&state.github_token)
        .header("User-Agent", "agb-playground")
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to call GitHub API");
            (StatusCode::BAD_GATEWAY, "Failed to fetch gist".to_string())
        })?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err((StatusCode::NOT_FOUND, "Gist not found".to_string()));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        tracing::error!(status = %status, body = %body, "GitHub API returned error");
        return Err((StatusCode::BAD_GATEWAY, "Failed to fetch gist".to_string()));
    }

    #[derive(Deserialize)]
    struct GistFileContent {
        content: String,
    }

    #[derive(Deserialize)]
    struct GitHubGistDetail {
        files: HashMap<String, GistFileContent>,
    }

    let gist: GitHubGistDetail = response.json().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to parse GitHub response");
        (
            StatusCode::BAD_GATEWAY,
            "Failed to parse gist response".to_string(),
        )
    })?;

    let code = gist
        .files
        .get("main.rs")
        .or_else(|| gist.files.values().next())
        .map(|f| f.content.clone())
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Gist has no files".to_string()))?;

    Ok(Json(GistCodeResponse { code }))
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
            .args(["-i", "ghcr.io/agbrs/playground-builder:latest"]);

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
