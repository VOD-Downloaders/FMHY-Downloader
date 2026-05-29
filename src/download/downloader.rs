use core::fmt;
use std::path::PathBuf;

use url::Url;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum DownloadError {}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // match self {}
        write!(f, "TODO")
    }
}

/////////////////////////////////////////////////////
// Downloader
/////////////////////////////////////////////////////
pub trait Downloader {
    fn download(input_url: &Url) -> Result<(), DownloadError>;
}
