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

    let env = env::EnvOptions::from_env().map_err(|error| {
        return AppError::EnvError(error);
    })?;

    trace!("Env options: {:?}", env);

    let creds = request::get_credentials(&env.flaresolverr_url, "https://cineby.sc").await.unwrap();

    info!("Credentials: {:?}", creds);

    let router = http::Router::new(env).await.map_err(|error| {
        return AppError::RouteError(error);
    })?;
    router.serve().await;

    Ok(())
}
