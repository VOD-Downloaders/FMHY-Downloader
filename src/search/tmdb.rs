use url::Url;
use serde::{Serialize, Deserialize};

use super::MovieResultBody;
use super::SeriesResultBody;

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

#[derive(Serialize)]
struct SeriesSearchParameters {
    pub query: String,
    pub include_adult: bool,
    pub page: i32,
}

impl Default for SeriesSearchParameters {
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
pub async fn tmdb_run_api_call<T>(api_call: &Url, requester: &request::Requester) -> T
where
    T: serde::de::DeserializeOwned + Default + std::fmt::Debug,
{
    trace!("Requesting API results from: \"{}\".", api_call);
    let result = requester.get_file_contents(api_call, None).await;
    let Ok(response) = result else {
        error!("Failed to retrieve API results from \"{}\", error: {}", api_call, result.unwrap_err());
        return T::default();
    };

    let result = String::from_utf8(response);
    let Ok(json_str) = result else {
        error!("Failed to convert API results response to a string, error: {}.", result.unwrap_err());
        return T::default();
    };

    trace!("{}", json_str);

    let result = serde_json::from_str::<T>(json_str.as_str());
    let Ok(body) = result else {
        error!("Failed to convert API results response to json, error: {}.", result.unwrap_err());
        return T::default();
    };

    trace!("Got these API results: {:?}", body);

    body
}

pub async fn tmdb_get_movies(movie_name: &str, page: Option<i32>, requester: &request::Requester) -> MovieResultBody {
    let page = page.unwrap_or(1);
    let parameters = serde_url_params::to_string(&MovieSearchParameters {
        query: movie_name.to_string(),
        page: page,
        ..MovieSearchParameters::default()
    })
    .unwrap();
    let api_call = format!("{}/movies/search?{}", TMDB_API_URL, parameters);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<MovieResultBody>(&url, requester).await
}

pub async fn tmdb_get_series(series_name: &str, page: Option<i32>, requester: &request::Requester) -> SeriesResultBody {
    let page = page.unwrap_or(1);
    let parameters = serde_url_params::to_string(&SeriesSearchParameters {
        query: series_name.to_string(),
        page: page,
        ..SeriesSearchParameters::default()
    })
    .unwrap();
    let api_call = format!("{}/tv/search?{}", TMDB_API_URL, parameters);
    let url = Url::parse(api_call.as_str()).unwrap();

    tmdb_run_api_call::<SeriesResultBody>(&url, requester).await
}
