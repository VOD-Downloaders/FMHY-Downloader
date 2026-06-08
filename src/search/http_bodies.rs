use serde::{Serialize, Deserialize};

/////////////////////////////////////////////////////
// MovieBody
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct MovieBody {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub genre_ids: Vec<u32>,
    pub id: u32,
    pub title: String,
    // pub original_language: String,
    // pub original_title: String,
    pub overview: String,
    // pub popularity: f32,
    pub poster_path: Option<String>,
    pub release_date: String, // Datetime<chrono::Utc>
}

/////////////////////////////////////////////////////
// MovieResultBody
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct MovieResultBody {
    pub page: u32,
    pub results: Vec<MovieBody>,
    pub total_pages: u32,
    pub total_results: u32,
}

impl Default for MovieResultBody {
    fn default() -> Self {
        Self {
            page: 1,
            results: Vec::new(),
            total_pages: 1,
            total_results: 0,
        }
    }
}

/////////////////////////////////////////////////////
// SeriesBody
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesBody {
    pub adult: bool,
    pub backdrop_path: Option<String>,
    pub genre_ids: Vec<u32>,
    pub id: u32,
    // pub origin_country: Vec<String>,
    // pub original_language: String,
    // pub original_name: String,
    pub overview: String,
    // pub popularity: f32,
    pub poster_path: Option<String>,
    pub first_air_date: String, // Datetime<chrono::Utc>
    pub name: String,
}

/////////////////////////////////////////////////////
// SeriesResultBody
/////////////////////////////////////////////////////
#[derive(Debug, Serialize, Deserialize)]
pub struct SeriesResultBody {
    pub page: u32,
    pub results: Vec<SeriesBody>,
    pub total_pages: u32,
    pub total_results: u32,
}

impl Default for SeriesResultBody {
    fn default() -> Self {
        Self {
            page: 1,
            results: Vec::new(),
            total_pages: 1,
            total_results: 0,
        }
    }
}
