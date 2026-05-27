use core::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use serde_json::json;
use axum::{routing, response};

use super::api;
use super::super::env;

/////////////////////////////////////////////////////
// RouteError
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum RouteError {
    FailedToBind { port: u16, error: std::io::Error },
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RouteError::FailedToBind { port, error } => {
                write!(f, "Failed to bind to port {} with error: {}.", port, error)
            },
        }
    }
}

/////////////////////////////////////////////////////
// Router
/////////////////////////////////////////////////////
pub struct Router {
    router: axum::Router,
    listener: tokio::net::TcpListener,
}

impl Router {
    pub async fn new(environment: env::EnvOptions) -> Result<Self, RouteError> {
        let address = "0.0.0.0:".to_string() + environment.webui_port.to_string().as_str();
        let listener = tokio::net::TcpListener::bind(address.as_str()).await.map_err(|error| {
            return RouteError::FailedToBind {
                port: environment.webui_port,
                error: error,
            };
        })?;

        info!("HTTP server listening on {}.", address.as_str());

        let router = Self::init_router(environment);

        Ok(Self {
            router: router,
            listener: listener,
        })
    }

    pub async fn serve(self) {
        // NOTE: Never returns an error
        axum::serve(self.listener, self.router)
            .with_graceful_shutdown(Self::shutdown_signal())
            .await
            .unwrap();
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to install terminate signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }

    fn get_file_contents(path: &Path) -> String {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                trace!("Read {}'s contents.", path.display());
                contents
            },
            Err(error) => {
                error!("Failed to read file \"{}\", got error: {}", path.display(), error);
                "NOT FOUND".into()
            },
        }
    }

    fn make_js(contents: String) -> ([(&'static str, &'static str); 1], String) {
        ([("content-type", "application/javascript")], contents)
    }

    fn make_css(contents: String) -> ([(&'static str, &'static str); 1], String) {
        ([("content-type", "text/css")], contents)
    }

    fn init_router(environment: env::EnvOptions) -> axum::Router {
        let index = Self::get_file_contents(PathBuf::from("web/index.html").as_path());
        let style_css = Self::get_file_contents(PathBuf::from("web/style.css").as_path());
        let index_js = Self::get_file_contents(PathBuf::from("web/index.js").as_path());

        let router = axum::Router::new()
            // Static routes
            .route("/", routing::get(response::Html(index)))

            // Static files
            .route("/index.js", routing::get(Self::make_js(index_js)))
            .route("/styles.css", routing::get(Self::make_css(style_css)))

            // Dynamic API calls
            .route("/api/download", routing::post(api::post_download))
            .route("/api/downloadStatus/{id}", routing::get(api::get_download_status))

            // State
            .with_state(Arc::new(Mutex::new(api::AppState::new(environment))));

        trace!("Created HTTP router.");

        router
    }
}
