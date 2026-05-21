use core::fmt;

#[macro_use]
mod logging;
mod env;

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

    info!("Env options: {:?}", env);

    // Download worker thread(s)

    // HTTP Server

    loop {}

    Ok(())
}
