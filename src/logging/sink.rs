use std::{fs::{File, OpenOptions}, io::Write, path::Path, sync::Mutex};

/////////////////////////////////////////////////////
// LogLevel
/////////////////////////////////////////////////////
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel 
{
    Trace,
    Info,
    Warn,
    Error,
}

/////////////////////////////////////////////////////
// Sink types
/////////////////////////////////////////////////////
pub trait Sink: Send
{
    fn log(&mut self, log_level: LogLevel, message: &str);
}

#[derive(Debug)]
pub struct ConsoleSink
{
    pub minimum_level: LogLevel,
}

impl ConsoleSink
{
    pub fn new(minimum_level: Option<LogLevel>) -> Self
    {
        Self {
            minimum_level: minimum_level.unwrap_or(LogLevel::Info)
        }
    }
}

impl Sink for ConsoleSink
{
    fn log(&mut self, log_level: LogLevel, message: &str)
    {
        if log_level >= self.minimum_level
        {
            println!("{}", message);
        }
    }
}

#[derive(Debug)]
pub struct FileSink
{
    pub output_file: File,
    pub minimum_level: LogLevel,
}

impl FileSink
{
    pub fn new(file_path: &Path, minimum_level: Option<LogLevel>) -> Result<Self, std::io::Error>
    {
        Ok(Self {
            output_file: OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?,
            minimum_level: minimum_level.unwrap_or(LogLevel::Info)
        })
    }
}

impl Sink for FileSink
{
    fn log(&mut self, log_level: LogLevel, message: &str)
    {
        if log_level >= self.minimum_level
        {
            let _ = self.output_file.write_all(message.as_bytes());
        }
    }
}

/////////////////////////////////////////////////////
// Sinks
/////////////////////////////////////////////////////
static SINKS: Mutex<Vec<Box<dyn Sink>>> = Mutex::new(Vec::new());

pub fn add_sink(sink: Box<dyn Sink>)
{
    if let Ok(mut sinks) = SINKS.lock()
    {
        sinks.push(sink);
    }
}

pub fn clear_sinks()
{
    if let Ok(mut sinks) = SINKS.lock()
    {
        sinks.clear();
    }
}

pub fn log_to_all_sinks(log_level: LogLevel, message: &str)
{
    if let Ok(mut sinks) = SINKS.lock()
    {
        for sink in sinks.iter_mut()
        {
            sink.log(log_level.clone(), message);
        }
    }
}