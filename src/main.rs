use thiserror::Error;

#[macro_use]
mod logging;
mod env;
mod config;
mod http;
mod request;
mod indexer;
mod download;

#[derive(Debug, Error)]
#[error(transparent)]
enum AppError {
    EnvError(#[from] env::EnvError),
    RouteError(#[from] http::RouteError),
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Setup
    logging::add_sink(Box::new(logging::ConsoleSink::new(None)));

    let env = env::EnvOptions::from_env().map_err(AppError::EnvError)?;

    logging::clear_sinks();
    logging::add_sink(Box::new(logging::ConsoleSink::new(Some(env.log_level.clone()))));

    trace!("Env options: {:?}", env);

    // Config

    // HTTP Router
    let router = http::Router::new(env).await.map_err(AppError::RouteError)?;
    router.serve().await;

    Ok(())
}
