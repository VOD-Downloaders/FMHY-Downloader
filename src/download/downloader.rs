use std::path::Path;

use url::Url;

use super::super::config::DownloadMethod;

/////////////////////////////////////////////////////
// DownloadError
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum DownloadError {}

pub async fn download_file(index_specification: &DownloadMethod, input_url: &Url, output_file: &Path) -> Result<(), DownloadError> {
    Ok(())
}
