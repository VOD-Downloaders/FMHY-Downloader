use core::fmt;

#[macro_use]
mod logging;
mod env;
mod http;

#[derive(Debug)]
enum AppError {
    EnvError(env::EnvError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::EnvError(error) => {
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

    let router = http::Router::new().await;
    router.serve().await;

    Ok(())
}
