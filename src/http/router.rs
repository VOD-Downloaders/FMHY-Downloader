use axum::routing;

/////////////////////////////////////////////////////
// Router
/////////////////////////////////////////////////////
pub struct Router {
    router: axum::Router,
    listener: tokio::net::TcpListener,
}

impl Router {
    pub async fn new() -> Self {
        let router = axum::Router::new().route("/", routing::get("TEST"));
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

        Self {
            router: router,
            listener: listener,
        }
    }

    pub async fn serve(self) {
        axum::serve(self.listener, self.router).await.unwrap();
    }
}
