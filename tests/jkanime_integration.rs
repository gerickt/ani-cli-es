use ani_cli_es::jkanime::{self, JkClient};

#[tokio::test]
async fn test_estrenos_scraping() {
    let client = JkClient::new().unwrap();
    let estrenos = jkanime::get_estrenos(&client).await.unwrap();
    println!("Estrenos found: {}", estrenos.len());
    assert!(!estrenos.is_empty(), "Estrenos should not be empty");
}

#[tokio::test]
async fn test_genres_catalog() {
    let client = JkClient::new().unwrap();
    let url = format!("{}/genero/accion/", client.base_url);
    let html = client.get_text(&url).await.unwrap();
    println!("Genre page HTML length: {}", html.len());
    println!("Contains dir1: {}", html.contains("dir1"));
    println!("Contains anime__item: {}", html.contains("anime__item"));
    println!("Contains card-title: {}", html.contains("card-title"));
    println!("HTML snippet: {:.500}", &html[..html.len().min(500)]);
    let animes = jkanime::get_anime_by_genre(&client, "accion").await.unwrap();
    println!("Action animes found: {}", animes.len());
    assert!(!animes.is_empty(), "Action genre should return animes");
}

#[tokio::test]
async fn test_details_and_streams() {
    let client = JkClient::new().unwrap();
    let details = jkanime::get_anime_details(&client, "ore-dake-level-up-na-ken").await.unwrap();
    println!("Title: {}", details.title);
    assert_eq!(details.slug, "ore-dake-level-up-na-ken");

    let episodes = jkanime::get_episodes(&client, &details).await.unwrap();
    println!("Episodes count: {}", episodes.len());
    assert!(!episodes.is_empty());

    let streams = jkanime::get_stream_urls(&client, "ore-dake-level-up-na-ken", 1).await.unwrap();
    println!("Streams found for ep 1: {:?}", streams);
    assert!(!streams.is_empty(), "Should extract at least one stream URL");
    assert!(streams[0].url.contains(".m3u8"));
}

#[tokio::test]
async fn test_one_piece_episode_count() {
    let client = JkClient::new().unwrap();
    let details = jkanime::get_anime_details(&client, "one-piece").await.unwrap();
    println!("One Piece title: '{}', episodes: {}", details.title, details.episodes_total);
    assert_eq!(details.title, "One Piece");
    assert!(details.episodes_total >= 1180, "One Piece must have at least 1180 episodes, got {}", details.episodes_total);
}

#[tokio::test]
async fn test_movie_and_ova_streams() {
    let client = JkClient::new().unwrap();
    let test_cases = [
        ("dark-machine-the-animation", 1),
        ("tensei-goblin-dakedo-shitsumon-aru", 3),
        ("umayuru-full-gate", 3),
        ("one-piece-taose-kaizoku-ganzack", 1),
        ("one-piece-heroines", 1),
    ];

    for (slug, ep) in test_cases {
        let streams = jkanime::get_stream_urls(&client, slug, ep).await.unwrap();
        println!("Streams found for {} ep {}: {}", slug, ep, streams.len());
        assert!(!streams.is_empty(), "Streams for {} ep {} should not be empty", slug, ep);
    }
}


