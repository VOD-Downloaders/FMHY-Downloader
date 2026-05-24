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

    logging::clear_sinks();
    logging::add_sink(Box::new(logging::ConsoleSink::new(Some(env.log_level.clone()))));

    trace!("Env options: {:?}", env);

    let creds = request::get_credentials(&env.flaresolverr_url, "https://cineby.sc").await.unwrap();

    trace!("Credentials: {:?}", creds);

    let index_data = downloader::get_index(&env, "https://www.cineby.sc/tv/66732/1/1?play=true", &creds)
        .await
        .unwrap();

    let path = std::path::PathBuf::from("/segments.ts");
    downloader::download_file(&env, index_data, &creds, path.as_path()).unwrap();

    let router = http::Router::new(env).await.map_err(|error| {
        return AppError::RouteError(error);
    })?;
    router.serve().await;

    Ok(())
}
