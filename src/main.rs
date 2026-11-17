mod cli;
mod config;
mod history;
mod jkanime;
mod library;
mod player;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use config::Config;
use jkanime::JkClient;
use player::Player;
use ui::app::{ActiveTab, App};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let mut config = Config::load();
    let client = JkClient::new()?;

    if args.vlc {
        config.player = "vlc".to_string();
    }
    if args.skip {
        config.skip_intro = true;
    }
    if let Some(q) = args.quality {
        config.quality = q;
    }

    // Direct flag: Delete history
    if args.delete_history {
        history::clear_history(&config.history_file)?;
        println!("Historial de reproducción borrado correctamente.");
        return Ok(());
    }

    // Direct CLI headless mode: when both query and episode are provided with download flag
    if args.download {
        if let (Some(query), Some(ep_no)) = (&args.query, args.episode) {
            println!("Buscando '{}'...", query);
            let search_results = jkanime::search_anime(&client, query).await?;
            if let Some(item) = search_results.first() {
                println!("Encontrado: {} ({})", item.title, item.slug);
                println!("Obteniendo enlaces de descarga para el episodio {}...", ep_no);
                let streams = jkanime::get_stream_urls(&client, &item.slug, ep_no).await?;
                if let Some(first) = streams.first() {
                    let filename = format!("{}_ep{:02}", item.slug, ep_no);
                    Player::download(&first.url, &config.download_dir, &filename)?;
                    return Ok(());
                } else {
                    eprintln!("Error: No se encontraron fuentes de video para el episodio {}.", ep_no);
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: No se encontraron resultados para '{}'.", query);
                std::process::exit(1);
            }
        }
    }

    // Direct CLI preview mode
    if args.preview {
        if let Some(query) = &args.query {
            println!("Obteniendo ficha técnica de '{}'...", query);
            let search_results = jkanime::search_anime(&client, query).await?;
            if let Some(item) = search_results.first() {
                let details = jkanime::get_anime_details(&client, &item.slug).await?;
                println!("\n╭─ 🍿 ANIME DECK ───────────────────────────────╮");
                println!("│ ✦ {} ({})", details.title, details.year);
                println!("◄═══════════════════════════════════════════════►");
                println!("│ 📌 Formato:    [ {} ]", details.anime_type);
                println!("│ 📺 Episodios:  {} eps ({})", details.episodes_total, details.duration);
                println!("│ ⚡ Estado:     ● {}", details.status);
                println!("│ 🎬 Estudio:    {}", details.studio);
                println!("│ 🏷️ Géneros:    {}", details.genres.join(", "));
                println!("╰───────────────────────────────────────────────╯");
                if !details.synopsis.is_empty() {
                    println!("╭─ 📝 SINOPSIS ─────────────────────────────────╮");
                    for s in details.synopsis.lines() {
                        println!("│ {}", s);
                    }
                    println!("╰───────────────────────────────────────────────╯");
                }
                return Ok(());
            }
        }
    }

    // Launch Ratatui TUI with initial state according to flags
    let mut app = App::new(config, client);

    if let Some(query) = args.query {
        app.search_input = query.clone();
        app.is_search_editing = false;
        app.is_loading = true;
        if let Ok(results) = jkanime::search_anime(&app.client, &query).await {
            app.search_results = results;
            if !app.search_results.is_empty() {
                app.search_table_state.select(Some(0));
            }
        }
        app.is_loading = false;
        app.active_tab = ActiveTab::Search;
    } else if args.estrenos {
        app.is_loading = true;
        if let Ok(estrenos) = jkanime::get_estrenos(&app.client).await {
            app.estrenos_list = estrenos;
            if !app.estrenos_list.is_empty() {
                app.estrenos_table_state.select(Some(0));
            }
        }
        app.is_loading = false;
        app.active_tab = ActiveTab::Estrenos;
    } else if args.generos {
        app.active_tab = ActiveTab::Genres;
    } else if args.continue_history {
        app.reload_history();
        app.active_tab = ActiveTab::History;
    } else if args.biblioteca {
        app.reload_library();
        app.active_tab = ActiveTab::Library;
    }

    ui::run_tui(app).await?;

    Ok(())
}
