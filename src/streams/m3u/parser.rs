use std::collections::HashMap;

use thiserror::Error;

/////////////////////////////////////////////////////
// ParseError
/////////////////////////////////////////////////////
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Missing or invalid #EXTM3U header")]
    InvalidHeader,
    #[error("The M3U file is empty or type could not be determined")]
    UnknownFormat,
    #[error("Invalid RESOLUTION attribute.")]
    InvalidResolutionAttribute,
    #[error("Failed to parse RESOLUTION to u32.")]
    FailedToParseResolution,
    #[error("Stream URL missing after stream info tag")]
    MissingStreamUrl,
}

/////////////////////////////////////////////////////
// M3UResult
/////////////////////////////////////////////////////
#[derive(Debug)]
pub enum M3UResult {
    Master(HashMap<(u32, u32), String>),
    Index(Vec<String>),
}

impl M3UResult {
    pub const DEFAULT_RESOLUTION: (u32, u32) = (1080, 720);
}

/////////////////////////////////////////////////////
// Parser
/////////////////////////////////////////////////////
pub fn parse_m3u_contents(contents: &str) -> Result<M3UResult, ParseError> {
    let trimmed = contents.trim();

    if !trimmed.starts_with("#EXTM3U") {
        return Err(ParseError::InvalidHeader);
    }

    // Parse correct type
    if trimmed.contains("#EXT-X-STREAM-INF") {
        parse_master_playlist(trimmed)
    } else if trimmed.contains("#EXTINF") {
        parse_index_playlist(trimmed)
    } else {
        Err(ParseError::UnknownFormat)
    }
}

fn parse_master_playlist(contents: &str) -> Result<M3UResult, ParseError> {
    let mut master_map = HashMap::new();
    let mut lines = contents.lines().map(|s| s.trim()).filter(|s| !s.is_empty());

    while let Some(line) = lines.next() {
        if line.starts_with("#EXT-X-STREAM-INF:") {
            // Get all XXX=... attributes
            let mut attributes = line.split(',');

            let resolution = {
                let resolution_attribute = attributes.find(|part| part.trim().starts_with("RESOLUTION="));

                if let Some(resolution_attribute) = resolution_attribute {
                    let dimensions = resolution_attribute.split('=').nth(1).ok_or(ParseError::InvalidResolutionAttribute)?;

                    let mut parts = dimensions.split('x');
                    let width = parts
                        .next()
                        .ok_or(ParseError::FailedToParseResolution)?
                        .parse::<u32>()
                        .map_err(|_error| ParseError::FailedToParseResolution)?;
                    let height = parts
                        .next()
                        .ok_or(ParseError::FailedToParseResolution)?
                        .parse::<u32>()
                        .map_err(|_error| ParseError::FailedToParseResolution)?;

                    (width, height)
                } else {
                    trace!(
                        "No RESOLUTION found in master file, defaulting to ({}, {})...",
                        M3UResult::DEFAULT_RESOLUTION.0,
                        M3UResult::DEFAULT_RESOLUTION.1
                    );
                    M3UResult::DEFAULT_RESOLUTION
                }
            };

            let url = lines.next().ok_or(ParseError::MissingStreamUrl)?.to_string();

            master_map.insert(resolution, url);
        }
    }

    Ok(M3UResult::Master(master_map))
}

fn parse_index_playlist(contents: &str) -> Result<M3UResult, ParseError> {
    let mut segments = Vec::new();

    for line in contents.lines().map(|s| s.trim()) {
        // Segments are lines that do not start with a #
        if !line.starts_with('#') {
            segments.push(line.to_string());
        }
    }

    Ok(M3UResult::Index(segments))
}
