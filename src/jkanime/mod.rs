pub mod client;
pub mod catalog;
pub mod details;
pub mod models;
pub mod search;
pub mod stream;

pub use catalog::{get_anime_by_genre, get_available_genres, get_estrenos};
pub use client::JkClient;
pub use details::{get_anime_details, get_episodes};
#[allow(unused_imports)]
pub use models::{AnimeDetails, AnimeItem, EpisodeItem, EstrenoItem, GenreCategory, StreamSource};
pub use search::search_anime;
pub use stream::get_stream_urls;
