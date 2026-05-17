#[macro_use]
mod logging;

fn main()
{
    logging::add_sink(Box::new(logging::ConsoleSink::new(None)));

    trace!("{}", "Hello, world!");
    info!("{}", "Hello, world!");
    warning!("{}", "Hello, world!");
    error!("{}", "Hello, world!");
}
