use url::Url;
use serde::{Serialize, Deserialize};

use super::MovieInfo;
use super::EpisodeInfo;
use super::SeasonInfo;
use super::SeriesInfo;

use super::super::request;

/////////////////////////////////////////////////////
// Config
/////////////////////////////////////////////////////
const TMDB_API_URL: &str = "https://tmdb-proxy-chi-ivory.vercel.app";

/////////////////////////////////////////////////////
// Parameters
/////////////////////////////////////////////////////
#[derive(Serialize)]
struct MovieSearchParameters {
    pub query: String,
    pub include_adult: bool,
    pub page: i32,
}

impl Default for MovieSearchParameters {
    fn default() -> Self {
        Self {
            query: "".to_string(),
            include_adult: false,
            page: 1,
        }
    }
}

/////////////////////////////////////////////////////
// TMDB interface
/////////////////////////////////////////////////////
pub async fn tmdb_get_movies(movie_name: &str, requester: &request::Requester) -> Vec<MovieInfo> {
    let parameters = serde_url_params::to_string(&MovieSearchParameters {
        query: movie_name.to_string(),
        ..MovieSearchParameters::default()
    })
    .unwrap();
    let api_call = format!("{}/movies/search?{}", TMDB_API_URL, parameters);
    let url = Url::parse(api_call.as_str()).unwrap();

    trace!("Requesting movie results from: \"{}\".", api_call);
    let result = requester.get_file_contents(&url, None).await;
    let Ok(response) = result else {
        error!("Failed to retrieve movie results for \"{}\", error: {}", movie_name, result.unwrap_err());
        return Vec::new();
    };

    let Ok(json_str) = String::from_utf8(response) else {
        error!("Failed to convert movie results response to json.");
        return Vec::new();
    };

    trace!("{}", json_str);

    // let Ok(json) = serde_json::from_str(json_str.as_str()) else {
    //     error!("Failed to convert movie results response to json.");
    //     return Vec::new();
    // };

    Vec::new()
}

