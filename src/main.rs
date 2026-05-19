#[macro_use]
mod logging;
mod env;

fn main() -> Result<(), ()> {
    logging::add_sink(Box::new(logging::ConsoleSink::new(None)));

    let result = env::EnvOptions::from_env();
    let Ok(env) = result else {
        error!("{:?}", result.unwrap_err());
        return Err(());
    };

    info!("Env options: {:?}", env);

    loop {}

    Ok(())
}
