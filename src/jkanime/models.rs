use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeItem {
    pub slug: String,
    pub title: String,
    pub anime_type: Option<String>,
    pub year: Option<String>,
    pub episodes_count: Option<String>,
    pub has_latino: bool,
    pub status: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeDetails {
    pub slug: String,
    pub title: String,
    pub anime_type: String,
    pub status: String,
    pub year: String,
    pub episodes_total: u32,
    pub duration: String,
    pub studio: String,
    pub genres: Vec<String>,
    pub synopsis: String,
    pub poster_url: String,
    pub anime_id: Option<String>,
    pub csrf_token: Option<String>,
    pub related_seasons: Vec<RelatedSeason>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedSeason {
    pub slug: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeItem {
    pub number: u32,
    pub title: String,
    pub date: Option<String>,
    pub is_new: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamSource {
    pub quality: String,
    pub url: String,
    pub server_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EstrenoItem {
    pub slug: String,
    pub episode: u32,
    pub title: String,
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
}
