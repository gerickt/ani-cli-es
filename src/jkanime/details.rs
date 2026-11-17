use super::client::JkClient;
use super::models::{AnimeDetails, EpisodeItem, RelatedSeason};
use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Deserialize)]
struct AjaxEp {
    number: serde_json::Value,
    timestamp: Option<String>,
}

pub async fn get_anime_details(client: &JkClient, slug: &str) -> Result<AnimeDetails> {
    let url = format!("{}/{}/", client.base_url, slug);
    let html = client.get_text(&url).await?;
    let doc = Html::parse_document(&html);

    let h3_sel = Selector::parse(".anime_info h3, .anime__details__text h3").unwrap();
    let title = doc
        .select(&h3_sel)
        .next()
        .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| {
            let title_sel = Selector::parse("title").unwrap();
            doc.select(&title_sel)
                .next()
                .map(|t| {
                    t.text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .replace(" - anime", "")
                        .replace(" online JkAnime", "")
                        .replace(" JkAnime", "")
                        .trim()
                        .to_string()
                })
                .unwrap_or_else(|| slug.to_string())
        });

    // Metadata extraction via regex for resilience against layout changes
    let type_re = Regex::new(r"<span>Tipo:</span>\s*<[^>]*>([^<]+)<|<span>Tipo:</span>\s*([^<]+)")?;
    let anime_type = type_re
        .captures(&html)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map_or("Serie", |m| m.as_str().trim())
        .to_string();

    let status_re = Regex::new(r"<span>Estado:</span>\s*<div[^>]*>([^<]+)<")?;
    let status = status_re
        .captures(&html)
        .map_or("Desconocido", |c| c.get(1).map_or("Desconocido", |m| m.as_str().trim()))
        .to_string();

    let yr_re = Regex::new(r"<span>\s*Emitido:\s*</span>\s*([0-9]{4})|temporada/[^&quot;]*([0-9]{4})")?;
    let year = yr_re
        .captures(&html)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map_or("N/A", |m| m.as_str().trim())
        .to_string();

    let ep_re = Regex::new(r"<span>Episodios:</span>\s*([0-9]+)")?;
    let uep_re = Regex::new(r#"id=["']uep["'][^>]*>.*?([0-9]+)|href=["'][^"']*/([0-9]+)/["'][^>]*id=["']uep["']|id=["']uep["'][^>]*href=["'][^"']*/([0-9]+)/["']"#)?;
    let mut total_eps: u32 = ep_re
        .captures(&html)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(0);

    if total_eps == 0 {
        if let Some(caps) = uep_re.captures(&html) {
            total_eps = caps
                .get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
        }
    }

    if total_eps == 0 {
        total_eps = 1;
    }

    let dur_re = Regex::new(r"<span>Duracion:</span>\s*([^<]+)")?;
    let duration = dur_re
        .captures(&html)
        .map_or("24 min", |c| c.get(1).map_or("24 min", |m| m.as_str().trim()))
        .to_string();

    let studio_re = Regex::new(r"<span>Estudios:</span>\s*<[^>]*>([^<]+)<")?;
    let studio = studio_re
        .captures(&html)
        .map_or("N/A", |c| c.get(1).map_or("N/A", |m| m.as_str().trim()))
        .to_string();

    // Genres (deduplicated)
    let mut genres = Vec::new();
    let mut seen_genres = std::collections::HashSet::new();
    let genre_re = Regex::new(r#"href="https://jkanime\.net/genero/([^/]+)/">([^<]+)<"#)?;
    for caps in genre_re.captures_iter(&html) {
        if let Some(g) = caps.get(2) {
            let name = g.as_str().trim().to_string();
            if !name.is_empty() && seen_genres.insert(name.clone()) {
                genres.push(name);
            }
        }
    }

    // Synopsis
    let scroll_p_sel = Selector::parse(".anime_info p.scroll, .sinopsis p").unwrap();
    let synopsis = doc
        .select(&scroll_p_sel)
        .next()
        .map(|p| p.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            let syn_re = Regex::new(r#"(?s)<div class="sinopsis">.*?<p[^>]*>(.*?)</p>"#).unwrap();
            syn_re
                .captures(&html)
                .map_or("", |c| c.get(1).map_or("", |m| m.as_str()))
                .replace("<br>", "\n")
                .replace("<br/>", "\n")
                .replace("<br />", "\n")
                .trim()
                .to_string()
        });

    // Poster image
    let poster_re = Regex::new(r#"https://cdn\.jkdesa\.com/assets/images/animes/image/[^"'\s]+\.jpg"#)?;
    let poster_url = poster_re
        .find(&html)
        .map_or("", |m| m.as_str())
        .to_string();

    // Internal ID and CSRF
    let id_re = Regex::new(r#"data-anime="([0-9]+)""#)?;
    let anime_id = id_re.captures(&html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());

    let csrf_re = Regex::new(r#"csrf-token"\s+content="([^"]+)""#)?;
    let csrf_token = csrf_re.captures(&html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());

    // Related seasons
    let mut related_seasons = Vec::new();
    let rel_re = Regex::new(r#"href="https://jkanime\.net/([a-z0-9-]+)/">([^<]+)</a>"#)?;
    if let Some(rel_section) = html.split("Temporadas y relacionados").nth(1) {
        let block = rel_section.split("</div>").next().unwrap_or("");
        for caps in rel_re.captures_iter(block) {
            let rel_slug = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let rel_title = caps.get(2).map_or("", |m| m.as_str()).to_string();
            if !rel_slug.is_empty() && rel_slug != slug {
                related_seasons.push(RelatedSeason {
                    slug: rel_slug,
                    title: rel_title,
                });
            }
        }
    }

    Ok(AnimeDetails {
        slug: slug.to_string(),
        title,
        anime_type,
        status,
        year,
        episodes_total: total_eps,
        duration,
        studio,
        genres,
        synopsis,
        poster_url,
        anime_id,
        csrf_token,
        related_seasons,
    })
}

pub async fn get_episodes(client: &JkClient, details: &AnimeDetails) -> Result<Vec<EpisodeItem>> {
    let mut date_map = std::collections::HashMap::new();

    if let (Some(anime_id), Some(token)) = (&details.anime_id, &details.csrf_token) {
        let ajax_url = format!("{}/ajax/episodes/{}/1", client.base_url, anime_id);
        if let Ok(json_str) = client.post_ajax(&ajax_url, token, &[("_token", token)]).await {
            if let Ok(ajax_eps) = serde_json::from_str::<Vec<AjaxEp>>(&json_str) {
                for ep in ajax_eps {
                    let ep_num = match ep.number {
                        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) as u32,
                        serde_json::Value::String(s) => s.parse().unwrap_or(0),
                        _ => 0,
                    };
                    if ep_num > 0 {
                        let date = ep.timestamp.map(|t| t.split_whitespace().next().unwrap_or("").to_string());
                        date_map.insert(ep_num, date);
                    }
                }
            }
        }
    }

    let mut episodes = Vec::new();
    let total = if details.episodes_total == 0 { 1 } else { details.episodes_total };

    if total == 1 {
        let date = date_map.get(&1).cloned().flatten();
        episodes.push(EpisodeItem {
            number: 1,
            title: "🎬 Película / Especial (Episodio Único)".to_string(),
            date,
            is_new: false,
        });
    } else {
        for num in 1..=total {
            let date = date_map.get(&num).cloned().flatten();
            let is_new = num == total;
            episodes.push(EpisodeItem {
                number: num,
                title: format!("📺 Episodio {:02}", num),
                date,
                is_new,
            });
        }
    }

    Ok(episodes)
}
