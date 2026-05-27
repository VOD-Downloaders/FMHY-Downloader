use core::fmt;

#[macro_use]
mod logging;
mod env;
mod http;
mod request;
mod downloader;

#[derive(Debug)]
enum AppError {
    EnvError(env::EnvError),
    RouteError(http::RouteError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::EnvError(error) => {
                write!(f, "{}", error)
            },
            AppError::RouteError(error) => {
                write!(f, "{}", error)
            },
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Setup
    logging::add_sink(Box::new(logging::ConsoleSink::new(None)));

    let env = env::EnvOptions::from_env().map_err(AppError::EnvError)?;

    logging::clear_sinks();
    logging::add_sink(Box::new(logging::ConsoleSink::new(Some(env.log_level.clone()))));

    trace!("Env options: {:?}", env);

    let router = http::Router::new(env).await.map_err(AppError::RouteError)?;
    router.serve().await;

    Ok(())
}
