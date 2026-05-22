use std::path::Path;
use std::path::PathBuf;

use axum::{routing, response};

use super::super::env;

/////////////////////////////////////////////////////
// Router
/////////////////////////////////////////////////////
pub struct Router {
    router: axum::Router,
    listener: tokio::net::TcpListener,
}

impl Router {
    pub async fn new(options: env::EnvOptions) -> Self {
        let address = "0.0.0.0:".to_string() + options.webui_port.to_string().as_str();
        let listener = tokio::net::TcpListener::bind(address).await.unwrap();

        let router = Self::init_router().await;

        Self {
            router: router,
            listener: listener,
        }
    }

    pub async fn serve(self) {
        axum::serve(self.listener, self.router).await.unwrap();
    }

    async fn get_file_contents(path: &Path) -> String {
        match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) => {
                error!("Internal logic error, tried to retrieve file from {}, got error: {}", path.display(), error);
                "NOT FOUND".into()
            },
        }
    }

    async fn init_router() -> axum::Router {
        let index = Self::get_file_contents(PathBuf::from("web/index.html").as_path()).await;

        axum::Router::new().route("/", routing::get(response::Html(index)))
    }
}
