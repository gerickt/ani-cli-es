use super::client::JkClient;
use super::models::AnimeItem;
use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};

pub async fn search_anime(client: &JkClient, query: &str) -> Result<Vec<AnimeItem>> {
    let clean_query = query.trim().replace([' ', '+'], "_");
    let search_url = format!("{}/buscar/{}/", client.base_url, clean_query);

    let html_content = client.get_text(&search_url).await.unwrap_or_default();

    let mut results = Vec::new();
    let slug_re = Regex::new(r"https://jkanime\.net/([^/]+)/")?;

    if !html_content.is_empty() {
        let document = Html::parse_document(&html_content);
        let item_selector = Selector::parse(".anime__item").unwrap();
        let link_selector = Selector::parse("h5 a").unwrap();
        let type_selector = Selector::parse("li.anime").unwrap();
        let status_selector = Selector::parse(".anime__item__text ul li").unwrap();
        let pic_selector = Selector::parse(".anime__item__pic").unwrap();

        for element in document.select(&item_selector) {
            let mut slug = String::new();
            let mut title = String::new();
            let mut anime_type = None;
            let mut status = None;
            let mut image_url = None;

            if let Some(link) = element.select(&link_selector).next() {
                title = link.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if let Some(href) = link.value().attr("href") {
                    if let Some(caps) = slug_re.captures(href) {
                        let candidate = caps.get(1).map_or("", |m| m.as_str());
                        if candidate != "buscar"
                            && candidate != "directorio"
                            && candidate != "historial"
                            && candidate != "guardado"
                            && candidate != "estrenos"
                            && candidate != "top"
                        {
                            slug = candidate.to_string();
                        }
                    }
                }
            }

            if slug.is_empty() {
                // Check direct anchor if h5 a wasn't matched
                let a_selector = Selector::parse("a").unwrap();
                for a in element.select(&a_selector) {
                    if let Some(href) = a.value().attr("href") {
                        if let Some(caps) = slug_re.captures(href) {
                            let candidate = caps.get(1).map_or("", |m| m.as_str());
                            if !candidate.is_empty()
                                && candidate != "buscar"
                                && candidate != "directorio"
                                && candidate != "historial"
                                && candidate != "guardado"
                                && candidate != "estrenos"
                            {
                                slug = candidate.to_string();
                                break;
                            }
                        }
                    }
                }
            }

            if let Some(t) = element.select(&type_selector).next() {
                let text = t.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !text.is_empty() {
                    anime_type = Some(text);
                }
            }

            if let Some(s) = element.select(&status_selector).next() {
                let text = s.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !text.is_empty() && anime_type.as_deref() != Some(&text) {
                    status = Some(text);
                }
            }

            if let Some(pic) = element.select(&pic_selector).next() {
                if let Some(bg) = pic.value().attr("data-setbg") {
                    image_url = Some(bg.to_string());
                }
            }

            if !slug.is_empty() && !title.is_empty() {
                results.push(AnimeItem {
                    slug,
                    title,
                    anime_type,
                    year: None,
                    episodes_count: None,
                    has_latino: false,
                    status,
                    image_url,
                });
            }
        }
    }

    // Fallback: If 0 results, try direct slug heuristic
    if results.is_empty() {
        let fallback_slug = query
            .to_lowercase()
            .trim()
            .replace([' ', '+'], "-");
        let direct_url = format!("{}/{}/", client.base_url, fallback_slug);
        if let Ok(direct_page) = client.get_text(&direct_url).await {
            if !direct_page.contains("404.png") && direct_page.contains("<title>") {
                let doc = Html::parse_document(&direct_page);
                let title_sel = Selector::parse("title").unwrap();
                let title = doc
                    .select(&title_sel)
                    .next()
                    .map(|e| {
                        e.text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .replace(" - anime", "")
                            .trim()
                            .to_string()
                    })
                    .unwrap_or_else(|| fallback_slug.clone());

                results.push(AnimeItem {
                    slug: fallback_slug,
                    title,
                    anime_type: Some("Serie".to_string()),
                    year: None,
                    episodes_count: None,
                    has_latino: direct_page.to_lowercase().contains("latino"),
                    status: None,
                    image_url: None,
                });
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let client = JkClient::new().unwrap();
        let res_text = client.get_text("https://jkanime.net/buscar/solo_leveling/").await;
        match res_text {
            Ok(t) => println!("Success! Received {} bytes", t.len()),
            Err(e) => println!("Error getting text: {:?}", e),
        }
        let res = search_anime(&client, "solo leveling").await.unwrap();
        println!("Search results count: {}", res.len());
        assert!(!res.is_empty());
    }
}
