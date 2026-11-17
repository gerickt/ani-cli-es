use super::client::JkClient;
use super::models::{AnimeItem, EstrenoItem, GenreCategory};
use anyhow::Result;
use regex::Regex;
use scraper::{Html, Selector};

pub fn get_available_genres() -> Vec<GenreCategory> {
    vec![
        GenreCategory {
            id: "accion".to_string(),
            name: "Acción".to_string(),
            description: "Combates, peleas y adrenalina".to_string(),
            icon: "⚔️".to_string(),
        },
        GenreCategory {
            id: "aventura".to_string(),
            name: "Aventura".to_string(),
            description: "Viajes, exploración y mundos abiertos".to_string(),
            icon: "🗺️".to_string(),
        },
        GenreCategory {
            id: "isekai".to_string(),
            name: "Isekai".to_string(),
            description: "Reencarnaciones y mundos paralelos".to_string(),
            icon: "🌀".to_string(),
        },
        GenreCategory {
            id: "comedia".to_string(),
            name: "Comedia".to_string(),
            description: "Humor, risas y situaciones cómicas".to_string(),
            icon: "🎭".to_string(),
        },
        GenreCategory {
            id: "sci-fi".to_string(),
            name: "Ciencia Ficción".to_string(),
            description: "Futuro, tecnología y espacio".to_string(),
            icon: "🚀".to_string(),
        },
        GenreCategory {
            id: "fantasia".to_string(),
            name: "Fantasía".to_string(),
            description: "Magia, reinos y criaturas míticas".to_string(),
            icon: "🧙".to_string(),
        },
        GenreCategory {
            id: "romance".to_string(),
            name: "Romance".to_string(),
            description: "Historias de amor y drama romántico".to_string(),
            icon: "💖".to_string(),
        },
        GenreCategory {
            id: "seinen".to_string(),
            name: "Seinen".to_string(),
            description: "Historias maduras, profundas y complejas".to_string(),
            icon: "🩸".to_string(),
        },
        GenreCategory {
            id: "shounen".to_string(),
            name: "Shounen".to_string(),
            description: "Superación, batallas y energía juvenil".to_string(),
            icon: "🔥".to_string(),
        },
        GenreCategory {
            id: "sobrenatural".to_string(),
            name: "Sobrenatural".to_string(),
            description: "Fantasmas, espíritus y lo oculto".to_string(),
            icon: "👻".to_string(),
        },
        GenreCategory {
            id: "terror".to_string(),
            name: "Terror".to_string(),
            description: "Horror, suspenso y escalofríos".to_string(),
            icon: "💀".to_string(),
        },
        GenreCategory {
            id: "misterio".to_string(),
            name: "Misterio".to_string(),
            description: "Casos policiales, enigmas y acertijos".to_string(),
            icon: "🔍".to_string(),
        },
        GenreCategory {
            id: "artes-marciales".to_string(),
            name: "Artes Marciales".to_string(),
            description: "Técnicas de combate cuerpo a cuerpo".to_string(),
            icon: "🥊".to_string(),
        },
        GenreCategory {
            id: "psicologico".to_string(),
            name: "Psicológico".to_string(),
            description: "Juegos mentales, tensión y thrillers".to_string(),
            icon: "🧠".to_string(),
        },
        GenreCategory {
            id: "deportes".to_string(),
            name: "Deportes".to_string(),
            description: "Fútbol, basket, boxeo y atletismo".to_string(),
            icon: "⚽".to_string(),
        },
        GenreCategory {
            id: "magia".to_string(),
            name: "Magia".to_string(),
            description: "Hechizos, academias y magia arcana".to_string(),
            icon: "✨".to_string(),
        },
        GenreCategory {
            id: "mecha".to_string(),
            name: "Mecha".to_string(),
            description: "Robots gigantes y combate mecánico".to_string(),
            icon: "🤖".to_string(),
        },
        GenreCategory {
            id: "historico".to_string(),
            name: "Histórico".to_string(),
            description: "Épocas antiguas, periodos samurai y batallas".to_string(),
            icon: "👑".to_string(),
        },
        GenreCategory {
            id: "musica".to_string(),
            name: "Música".to_string(),
            description: "Bandas, ídols, orquestas y conciertos".to_string(),
            icon: "🎵".to_string(),
        },
        GenreCategory {
            id: "juegos".to_string(),
            name: "Juegos".to_string(),
            description: "Gaming, torneos y realidad virtual".to_string(),
            icon: "🎮".to_string(),
        },
        GenreCategory {
            id: "gourmet".to_string(),
            name: "Gourmet".to_string(),
            description: "Cocina, platillos deliciosos y recetas".to_string(),
            icon: "🍳".to_string(),
        },
        GenreCategory {
            id: "escolar".to_string(),
            name: "Escolar".to_string(),
            description: "Vida estudiantil, clubes y academias".to_string(),
            icon: "🏫".to_string(),
        },
        GenreCategory {
            id: "recuentos-de-la-vida".to_string(),
            name: "Slice of Life".to_string(),
            description: "Recuentos entrañables de la vida cotidiana".to_string(),
            icon: "🌸".to_string(),
        },
    ]
}

pub async fn get_estrenos(client: &JkClient) -> Result<Vec<EstrenoItem>> {
    let html = client.get_text(&client.base_url).await?;
    let doc = Html::parse_document(&html);

    let ep_re = Regex::new(r"https://jkanime\.net/([^/]+)/([0-9]+)/")?;
    let container_sel = Selector::parse(".dir1").unwrap();
    let link_sel = Selector::parse("a").unwrap();
    let title_sel = Selector::parse("h5, .card-title").unwrap();

    let mut estrenos = Vec::new();

    for elem in doc.select(&container_sel) {
        if let Some(link) = elem.select(&link_sel).next() {
            if let Some(href) = link.value().attr("href") {
                if let Some(caps) = ep_re.captures(href) {
                    let slug = caps.get(1).map_or("", |m| m.as_str()).to_string();
                    let ep_num: u32 = caps.get(2).map_or("1", |m| m.as_str()).parse().unwrap_or(1);

                    let title = elem
                        .select(&title_sel)
                        .next()
                        .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
                        .unwrap_or_else(|| slug.clone());

                    estrenos.push(EstrenoItem {
                        slug,
                        episode: ep_num,
                        title,
                        tag: "Hoy".to_string(),
                    });
                }
            }
        }
    }

    Ok(estrenos)
}

pub async fn get_anime_by_genre(client: &JkClient, genre_slug: &str) -> Result<Vec<AnimeItem>> {
    let url = format!("{}/genero/{}/", client.base_url, genre_slug);
    let html = client.get_text(&url).await?;
    let doc = Html::parse_document(&html);

    let slug_re = Regex::new(r"https://jkanime\.net/([^/]+)/")?;
    let mut items = Vec::new();
    let mut seen_slugs = std::collections::HashSet::new();

    // 1. Modern JKAnime format: .anime__item
    let item_sel = Selector::parse(".anime__item").unwrap();
    let h5_link_sel = Selector::parse("h5 a").unwrap();
    let type_sel = Selector::parse("li.anime").unwrap();
    let pic_sel = Selector::parse(".anime__item__pic").unwrap();

    for elem in doc.select(&item_sel) {
        if let Some(link) = elem.select(&h5_link_sel).next() {
            let title = link.text().collect::<Vec<_>>().join(" ").trim().to_string();
            if let Some(href) = link.value().attr("href") {
                if let Some(caps) = slug_re.captures(href) {
                    let slug = caps.get(1).map_or("", |m| m.as_str()).to_string();
                    if slug.is_empty()
                        || slug == "genero"
                        || slug == "buscar"
                        || slug == "directorio"
                        || slug == "estrenos"
                        || !seen_slugs.insert(slug.clone())
                    {
                        continue;
                    }

                    let anime_type = elem.select(&type_sel).next().map(|t| {
                        t.text().collect::<Vec<_>>().join(" ").trim().to_string()
                    });

                    let image_url = elem.select(&pic_sel).next().and_then(|p| {
                        p.value().attr("data-setbg").map(|s| s.to_string())
                    });

                    items.push(AnimeItem {
                        slug,
                        title,
                        anime_type,
                        year: None,
                        episodes_count: None,
                        has_latino: false,
                        status: None,
                        image_url,
                    });
                }
            }
        }
    }

    // 2. Legacy format: .dir1
    if items.is_empty() {
        let dir1_sel = Selector::parse(".dir1").unwrap();
        let a_sel = Selector::parse("a").unwrap();
        let card_title_sel = Selector::parse(".card-title, h5").unwrap();
        let card_type_sel = Selector::parse(".badge.TV, .badge.badge-primary, li.anime").unwrap();
        let card_img_sel = Selector::parse("img.card-img-top, img").unwrap();

        for elem in doc.select(&dir1_sel) {
            if let Some(link) = elem.select(&a_sel).next() {
                if let Some(href) = link.value().attr("href") {
                    if let Some(caps) = slug_re.captures(href) {
                        let slug = caps.get(1).map_or("", |m| m.as_str()).to_string();
                        if slug.is_empty()
                            || slug == "genero"
                            || slug == "buscar"
                            || slug == "directorio"
                            || slug == "estrenos"
                            || !seen_slugs.insert(slug.clone())
                        {
                            continue;
                        }

                        let title = elem
                            .select(&card_title_sel)
                            .next()
                            .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string())
                            .unwrap_or_else(|| slug.clone());

                        let anime_type = elem
                            .select(&card_type_sel)
                            .next()
                            .map(|t| t.text().collect::<Vec<_>>().join(" ").trim().to_string());

                        let image_url = elem
                            .select(&card_img_sel)
                            .next()
                            .and_then(|img| img.value().attr("src"))
                            .map(|s| s.to_string());

                        items.push(AnimeItem {
                            slug,
                            title,
                            anime_type,
                            year: None,
                            episodes_count: None,
                            has_latino: false,
                            status: None,
                            image_url,
                        });
                    }
                }
            }
        }
    }

    // 3. Fallback: Any valid anime anchor in the page
    if items.is_empty() {
        let general_a_sel = Selector::parse("a").unwrap();
        for elem in doc.select(&general_a_sel) {
            if let Some(href) = elem.value().attr("href") {
                if let Some(caps) = slug_re.captures(href) {
                    let slug = caps.get(1).map_or("", |m| m.as_str()).to_string();
                    if slug.is_empty()
                        || slug == "genero"
                        || slug == "buscar"
                        || slug == "directorio"
                        || slug == "estrenos"
                        || slug == "top"
                        || slug == "horario"
                        || slug == "politica-de-privacidad"
                        || slug == "dmca"
                        || !seen_slugs.insert(slug.clone())
                    {
                        continue;
                    }

                    let title = elem.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !title.is_empty() && title.len() > 1 && !title.starts_with("http") {
                        items.push(AnimeItem {
                            slug,
                            title,
                            anime_type: Some("Serie".to_string()),
                            year: None,
                            episodes_count: None,
                            has_latino: false,
                            status: None,
                            image_url: None,
                        });
                    }
                }
            }
        }
    }

    Ok(items)
}
