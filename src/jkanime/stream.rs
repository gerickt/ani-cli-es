use super::client::JkClient;
use super::models::StreamSource;
use anyhow::Result;
use base64::prelude::*;
use regex::Regex;
use serde::Deserialize;

#[derive(Deserialize)]
struct ServerEntry {
    remote: String,
    server: String,
}

pub async fn get_stream_urls(
    client: &JkClient,
    slug: &str,
    ep_no: u32,
) -> Result<Vec<StreamSource>> {
    let mut candidate_urls = Vec::new();
    candidate_urls.push(format!("{}/{}/{}/", client.base_url, slug, ep_no));
    if ep_no == 1 {
        candidate_urls.push(format!("{}/{}/pelicula/", client.base_url, slug));
        candidate_urls.push(format!("{}/{}/ova/", client.base_url, slug));
        candidate_urls.push(format!("{}/{}/especial/", client.base_url, slug));
        candidate_urls.push(format!("{}/{}/serie/", client.base_url, slug));
        candidate_urls.push(format!("{}/{}/", client.base_url, slug));
    }

    let mut ep_page = String::new();
    let mut last_err = None;

    for url in &candidate_urls {
        match client.get_text(url).await {
            Ok(page) => {
                if page.contains("var video") || page.contains("var servers") || page.contains("player_conte") {
                    ep_page = page;
                    break;
                }
            }
            Err(e) => {
                last_err = Some(e);
            }
        }
    }

    if ep_page.is_empty() {
        if let Some(err) = last_err {
            return Err(err);
        }
        anyhow::bail!("No se pudo cargar la página del episodio {} para '{}'", ep_no, slug);
    }

    let embed_re = Regex::new(r#"video(?:\[[0-9]+\]|\.push)\s*=\s*['"]<iframe[^>]+src=['"]([^'"]+)['"]"#)?;
    let mut embed_urls = Vec::new();
    for cap in embed_re.captures_iter(&ep_page) {
        if let Some(m) = cap.get(1) {
            let mut src = m.as_str().to_string();
            if src.starts_with('/') {
                src = format!("{}{}", client.base_url, src);
            }
            embed_urls.push(src);
        }
    }

    if embed_urls.is_empty() {
        let iframe_src_re = Regex::new(r#"<iframe[^>]+src=['"](https?://jkanime\.net/jkplayer/[^'"]+|/[^'"]+)['"]"#)?;
        for cap in iframe_src_re.captures_iter(&ep_page) {
            if let Some(m) = cap.get(1) {
                let mut src = m.as_str().to_string();
                if src.starts_with('/') {
                    src = format!("{}{}", client.base_url, src);
                }
                embed_urls.push(src);
            }
        }
    }

    let load_source_re = Regex::new(r#"loadSource\(\s*['"]([^'"]+)['"]"#)?;
    let atob_re = Regex::new(r#"atob\(['"]([^'"]+)['"]\)"#)?;
    let source_tag_re = Regex::new(r#"<source[^>]+src=['"]([^'"]+)['"]"#)?;
    let url_param_re = Regex::new(r#"url:\s*['"]([^'"]+\.m3u8[^'"]*)['"]"#)?;
    let file_param_re = Regex::new(r#"file:\s*['"]([^'"]+\.m3u8[^'"]*)['"]"#)?;

    let mut stream_sources = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    for embed_url in embed_urls {
        let embed_page = match client.get_text(&embed_url).await {
            Ok(p) => p,
            Err(_) => continue,
        };

        let mut candidate_url = None;

        // 1. loadSource('...')
        if let Some(cap) = load_source_re.captures(&embed_page) {
            if let Some(m) = cap.get(1) {
                let url = m.as_str().trim();
                if url.starts_with("http") {
                    candidate_url = Some(url.to_string());
                }
            }
        }

        // 2. atob('...')
        if candidate_url.is_none() {
            if let Some(cap) = atob_re.captures(&embed_page) {
                if let Some(m) = cap.get(1) {
                    if let Ok(decoded_bytes) = BASE64_STANDARD.decode(m.as_str().trim()) {
                        if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                            if decoded_str.starts_with("http") && decoded_str.contains(".m3u8") {
                                candidate_url = Some(decoded_str);
                            }
                        }
                    }
                }
            }
        }

        // 3. <source src='...'>
        if candidate_url.is_none() {
            if let Some(cap) = source_tag_re.captures(&embed_page) {
                if let Some(m) = cap.get(1) {
                    let url = m.as_str().trim();
                    if url.starts_with("http") {
                        candidate_url = Some(url.to_string());
                    }
                }
            }
        }

        // 4. url: '...'
        if candidate_url.is_none() {
            if let Some(cap) = url_param_re.captures(&embed_page) {
                if let Some(m) = cap.get(1) {
                    let url = m.as_str().trim();
                    if url.starts_with("http") {
                        candidate_url = Some(url.to_string());
                    }
                }
            }
        }

        // 5. file: '...'
        if candidate_url.is_none() {
            if let Some(cap) = file_param_re.captures(&embed_page) {
                if let Some(m) = cap.get(1) {
                    let url = m.as_str().trim();
                    if url.starts_with("http") {
                        candidate_url = Some(url.to_string());
                    }
                }
            }
        }

        if let Some(final_url) = candidate_url {
            if seen_urls.insert(final_url.clone()) {
                let server = if embed_url.contains("umv") {
                    "Servidor V"
                } else if embed_url.contains("um") {
                    "Servidor Principal"
                } else {
                    "Servidor Alternativo"
                };

                stream_sources.push(StreamSource {
                    quality: "1080p".to_string(),
                    url: final_url,
                    server_name: server.to_string(),
                });
            }
        }
    }

    // Extract third-party servers from var servers = [...] as fallbacks
    // Only include servers known to have direct playable streams without anti-bot wrappers (mp4upload)
    let servers_re = Regex::new(r#"var\s+servers\s*=\s*(\[\s*\{.*?\}\s*\]);"#)?;
    if let Some(caps) = servers_re.captures(&ep_page) {
        if let Some(json_str) = caps.get(1) {
            if let Ok(entries) = serde_json::from_str::<Vec<ServerEntry>>(json_str.as_str()) {
                for entry in entries {
                    let s_lower = entry.server.to_lowercase();
                    // Filter out obfuscated anti-bot wrappers and non-streamable lockers
                    if !s_lower.contains("mp4upload") {
                        continue;
                    }
                    if let Ok(decoded_bytes) = BASE64_STANDARD.decode(&entry.remote) {
                        if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                            let clean_url = decoded_str.trim().to_string();
                            if clean_url.starts_with("http") && seen_urls.insert(clean_url.clone()) {
                                stream_sources.push(StreamSource {
                                    quality: "1080p".to_string(),
                                    url: clean_url,
                                    server_name: "Mp4upload".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(stream_sources)
}
