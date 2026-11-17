pub mod app;
pub mod components;
pub mod theme;

use self::app::{ActivePane, ActiveTab, App, AppAction, AsyncEvent, DetailTab, GenresColumn, ModalDialog};
use crate::jkanime::{self, JkClient};
use crate::player::Player;
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;
use std::time::Duration;
use tokio::sync::mpsc;

pub async fn run_tui(mut app: App) -> Result<()> {
    let mut terminal = ratatui::init();
    let mut events = EventStream::new();
    let mut render_interval = tokio::time::interval(Duration::from_millis(16));
    let (tx, mut rx) = mpsc::channel::<AsyncEvent>(64);

    // Initial background loading if started with items but no details
    if app.active_tab == ActiveTab::Search && !app.search_results.is_empty() && app.current_details.is_none() {
        if let Some(first) = app.search_results.first() {
            app.is_loading = true;
            app.loading_message = "Cargando detalles...".to_string();
            app.current_request_id += 1;
            spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, first.slug.clone());
        }
    } else if app.active_tab == ActiveTab::Estrenos && !app.estrenos_list.is_empty() && app.current_details.is_none() {
        if let Some(first) = app.estrenos_list.first() {
            app.is_loading = true;
            app.loading_message = "Cargando detalles...".to_string();
            app.current_request_id += 1;
            spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, first.slug.clone());
        }
    }

    while !app.should_quit {
        tokio::select! {
            _ = render_interval.tick() => {
                app.spinner_tick = app.spinner_tick.wrapping_add(1);

                if let Some(child) = &mut app.player_child {
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            app.player_child = None;
                            let _ = tx.send(AsyncEvent::PlayerTerminated {
                                success: status.success(),
                                error_msg: if status.success() {
                                    None
                                } else {
                                    Some(format!("El reproductor finalizó con código {:?}", status.code()))
                                },
                            }).await;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            app.player_child = None;
                            let _ = tx.send(AsyncEvent::PlayerTerminated {
                                success: false,
                                error_msg: Some(e.to_string()),
                            }).await;
                        }
                    }
                }

                terminal.draw(|f| render_ui(&mut app, f))?;
            }
            Some(Ok(event)) = events.next() => {
                if let Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        handle_key_event(&mut app, &tx, key.code, key.modifiers).await?;
                    }
                }
            }
            Some(async_event) = rx.recv() => {
                handle_async_event(&mut app, &tx, async_event).await;
            }
        }

        // Process pending external actions
        if let Some(action) = app.pending_action.take() {
            match action {
                AppAction::PlayStream { url, server_name, title, ep_no, slug } => {
                    let _ = crate::history::save_entry(&app.config.history_file, &slug, &title, ep_no);
                    app.reload_history();
                    app.set_toast(&format!("Reproduciendo Ep. {:02} con {}", ep_no, app.config.player), false);

                    if let Some(mut old_child) = app.player_child.take() {
                        let _ = old_child.kill().await;
                    }

                    let player_title = format!("{} - Episodio {}", title, ep_no);
                    match Player::spawn_player_process(&app.config.player, &url, &player_title, app.config.skip_intro) {
                        Ok(mut child) => {
                            if let Some(stdout) = child.stdout.take() {
                                let tx_out = tx.clone();
                                tokio::spawn(async move {
                                    use tokio::io::AsyncBufReadExt;
                                    let reader = tokio::io::BufReader::new(stdout);
                                    let mut lines = reader.lines();
                                    while let Ok(Some(line)) = lines.next_line().await {
                                        let text = line.split('\r').next_back().unwrap_or(&line).trim().to_string();
                                        if !text.is_empty() {
                                            let _ = tx_out.send(AsyncEvent::PlayerOutputLine(text)).await;
                                        }
                                    }
                                });
                            }

                            if let Some(stderr) = child.stderr.take() {
                                let tx_err = tx.clone();
                                tokio::spawn(async move {
                                    use tokio::io::AsyncBufReadExt;
                                    let reader = tokio::io::BufReader::new(stderr);
                                    let mut lines = reader.lines();
                                    while let Ok(Some(line)) = lines.next_line().await {
                                        let text = line.split('\r').next_back().unwrap_or(&line).trim().to_string();
                                        if !text.is_empty() {
                                            let _ = tx_err.send(AsyncEvent::PlayerOutputLine(text)).await;
                                        }
                                    }
                                });
                            }

                            app.player_child = Some(child);
                            app.modal = ModalDialog::PlayerStatus {
                                title,
                                ep_no,
                                server_name,
                                logs: Vec::new(),
                                is_running: true,
                                has_error: false,
                            };
                        }
                        Err(e) => {
                            app.set_toast(&format!("Error al iniciar reproductor: {}", e), true);
                            app.modal = ModalDialog::PlayerStatus {
                                title,
                                ep_no,
                                server_name,
                                logs: vec![format!("ERROR: No se pudo iniciar el reproductor: {}", e)],
                                is_running: false,
                                has_error: true,
                            };
                        }
                    }
                }
                AppAction::DownloadStream { url, server_name, title, ep_no, slug } => {
                    ratatui::restore();
                    println!("\nDescargando '{}' (Ep. {}) desde [{}]...", title, ep_no, server_name);
                    let filename = format!("{}_ep{:02}", slug, ep_no);
                    let _ = Player::download(&url, &app.config.download_dir, &filename);
                    app.reload_library();
                    app.set_toast(&format!("Descarga completada: {}", filename), false);
                    terminal = ratatui::init();
                }
            }
        }
    }

    if let Some(mut child) = app.player_child.take() {
        let _ = child.kill().await;
    }
    ratatui::restore();
    Ok(())
}

fn spawn_search(tx: mpsc::Sender<AsyncEvent>, client: JkClient, request_id: usize, query: String) {
    tokio::spawn(async move {
        match jkanime::search_anime(&client, &query).await {
            Ok(results) => {
                let _ = tx.send(AsyncEvent::SearchSuccess { request_id, query, results }).await;
            }
            Err(e) => {
                let _ = tx.send(AsyncEvent::NetworkError { message: format!("Error en búsqueda: {}", e) }).await;
            }
        }
    });
}

fn spawn_load_details(tx: mpsc::Sender<AsyncEvent>, client: JkClient, request_id: usize, slug: String) {
    tokio::spawn(async move {
        match jkanime::get_anime_details(&client, &slug).await {
            Ok(details) => {
                let episodes = jkanime::get_episodes(&client, &details).await.unwrap_or_default();
                let _ = tx.send(AsyncEvent::DetailsSuccess { request_id, slug, details, episodes }).await;
            }
            Err(e) => {
                let _ = tx.send(AsyncEvent::NetworkError { message: format!("Error al cargar ficha: {}", e) }).await;
            }
        }
    });
}

fn spawn_load_estrenos(tx: mpsc::Sender<AsyncEvent>, client: JkClient) {
    tokio::spawn(async move {
        match jkanime::get_estrenos(&client).await {
            Ok(estrenos) => {
                let _ = tx.send(AsyncEvent::EstrenosSuccess { estrenos }).await;
            }
            Err(e) => {
                let _ = tx.send(AsyncEvent::NetworkError { message: format!("Error al obtener estrenos: {}", e) }).await;
            }
        }
    });
}

fn spawn_load_genre_animes(tx: mpsc::Sender<AsyncEvent>, client: JkClient, genre_id: String) {
    tokio::spawn(async move {
        match jkanime::get_anime_by_genre(&client, &genre_id).await {
            Ok(animes) => {
                let _ = tx.send(AsyncEvent::GenreAnimesSuccess { genre_id, animes }).await;
            }
            Err(e) => {
                let _ = tx.send(AsyncEvent::NetworkError { message: format!("Error al cargar catálogo: {}", e) }).await;
            }
        }
    });
}

fn spawn_load_streams(
    tx: mpsc::Sender<AsyncEvent>,
    client: JkClient,
    slug: String,
    title: String,
    ep_no: u32,
    is_download: bool,
) {
    tokio::spawn(async move {
        match jkanime::get_stream_urls(&client, &slug, ep_no).await {
            Ok(sources) => {
                let _ = tx
                    .send(AsyncEvent::StreamsSuccess {
                        slug,
                        title,
                        ep_no,
                        sources,
                        is_download,
                    })
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(AsyncEvent::NetworkError {
                        message: format!("Error al resolver enlaces: {}", e),
                    })
                    .await;
            }
        }
    });
}

async fn handle_async_event(
    app: &mut App,
    tx: &mpsc::Sender<AsyncEvent>,
    event: AsyncEvent,
) {
    match event {
        AsyncEvent::SearchSuccess { request_id, query: _, results } => {
            if request_id == app.current_request_id {
                app.search_results = results;
                app.is_loading = false;
                app.loading_message.clear();
                if !app.search_results.is_empty() {
                    app.search_table_state.select(Some(0));
                    if let Some(first) = app.search_results.first() {
                        let slug = first.slug.clone();
                        app.is_loading = true;
                        app.loading_message = "Cargando ficha...".to_string();
                        app.current_request_id += 1;
                        spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, slug);
                    }
                } else {
                    app.current_details = None;
                    app.episodes_list.clear();
                }
            }
        }
        AsyncEvent::DetailsSuccess { request_id, slug: _, details, episodes } => {
            if request_id == app.current_request_id {
                app.current_details = Some(details);
                app.episodes_list = episodes;
                app.episodes_table_state.select(Some(0));
                app.related_table_state.select(Some(0));
                app.is_loading = false;
                app.loading_message.clear();
            }
        }
        AsyncEvent::EstrenosSuccess { estrenos } => {
            app.estrenos_list = estrenos;
            app.is_loading = false;
            app.loading_message.clear();
            if !app.estrenos_list.is_empty() {
                app.estrenos_table_state.select(Some(0));
                if let Some(first) = app.estrenos_list.first() {
                    let slug = first.slug.clone();
                    app.is_loading = true;
                    app.loading_message = "Cargando ficha...".to_string();
                    app.current_request_id += 1;
                    spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, slug);
                }
            }
        }
        AsyncEvent::GenreAnimesSuccess { genre_id: _, animes } => {
            app.genre_animes = animes;
            app.is_loading = false;
            app.loading_message.clear();
            if !app.genre_animes.is_empty() {
                app.genres_column = GenresColumn::Animes;
                app.genre_animes_table_state.select(Some(0));
                if let Some(first) = app.genre_animes.first() {
                    let slug = first.slug.clone();
                    app.is_loading = true;
                    app.loading_message = "Cargando ficha...".to_string();
                    app.current_request_id += 1;
                    spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, slug);
                }
            } else {
                app.genres_column = GenresColumn::Categories;
                app.set_toast("No se encontraron animes para este género", false);
            }
        }
        AsyncEvent::StreamsSuccess { slug, title, ep_no, sources, is_download } => {
            app.is_searching = false;
            if sources.is_empty() {
                app.set_toast(&format!("No se encontraron fuentes de reproducción para el episodio {}", ep_no), true);
            } else if sources.len() == 1 {
                let first = &sources[0];
                let action = if is_download {
                    AppAction::DownloadStream {
                        url: first.url.clone(),
                        server_name: first.server_name.clone(),
                        title,
                        ep_no,
                        slug,
                    }
                } else {
                    AppAction::PlayStream {
                        url: first.url.clone(),
                        server_name: first.server_name.clone(),
                        title,
                        ep_no,
                        slug,
                    }
                };
                app.pending_action = Some(action);
            } else {
                app.modal = ModalDialog::SelectServer {
                    slug,
                    title,
                    ep_no,
                    sources,
                    selected_index: 0,
                    is_download,
                };
            }
        }
        AsyncEvent::NetworkError { message } => {
            app.is_loading = false;
            app.is_searching = false;
            app.loading_message.clear();
            if app.active_tab == ActiveTab::Genres && app.genre_animes.is_empty() {
                app.genres_column = GenresColumn::Categories;
            }
            app.set_toast(&message, true);
        }
        AsyncEvent::PlayerOutputLine(line) => {
            if let ModalDialog::PlayerStatus { logs, has_error, .. } = &mut app.modal {
                if line.contains("ERROR:") || line.contains("Failed") {
                    *has_error = true;
                }
                logs.push(line);
                if logs.len() > 50 {
                    logs.remove(0);
                }
            }
        }
        AsyncEvent::PlayerTerminated { success, error_msg } => {
            if let ModalDialog::PlayerStatus { is_running, has_error, logs, .. } = &mut app.modal {
                *is_running = false;
                if !success {
                    *has_error = true;
                    if let Some(err) = error_msg {
                        logs.push(format!("ERROR: {}", err));
                    }
                }
            }
            if !success {
                app.set_toast("El reproductor finalizó con error", true);
            }
        }
    }
}

fn render_ui(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top Navigation Bar & Tabs
            Constraint::Min(10),   // Active Tab Screen
            Constraint::Length(3), // Bottom Control Deck & Toast
        ])
        .split(area);

    // 1. Header
    components::header::Header::render(app, chunks[0], frame.buffer_mut());

    // 2. Active Screen
    match app.active_tab {
        ActiveTab::Search => components::search_view::SearchView::render(app, chunks[1], frame.buffer_mut()),
        ActiveTab::Estrenos => components::estrenos_view::EstrenosView::render(app, chunks[1], frame.buffer_mut()),
        ActiveTab::Genres => components::genres_view::GenresView::render(app, chunks[1], frame.buffer_mut()),
        ActiveTab::History => components::history_view::HistoryView::render(app, chunks[1], frame.buffer_mut()),
        ActiveTab::Library => components::library_view::LibraryView::render(app, chunks[1], frame.buffer_mut()),
        ActiveTab::Help => components::help_view::HelpView::render(app, chunks[1], frame.buffer_mut()),
    }

    // 3. Footer
    components::footer::Footer::render(app, chunks[2], frame.buffer_mut());

    // 4. Overlaid Modal Dialog
    components::modal::Modal::render(app, area, frame.buffer_mut());
}

async fn handle_key_event(
    app: &mut App,
    tx: &mpsc::Sender<AsyncEvent>,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Result<()> {
    // Global Ctrl+C quits
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        app.should_quit = true;
        return Ok(());
    }

    // Modal takes priority if open
    if app.modal != ModalDialog::None {
        return handle_modal_keys(app, tx, code).await;
    }

    // Text search input mode
    if app.active_tab == ActiveTab::Search && app.is_search_editing {
        return handle_search_typing(app, tx, code).await;
    }

    // Global Tab numbers (1 - 6)
    match code {
        KeyCode::Char('1') => {
            app.active_tab = ActiveTab::Search;
            return Ok(());
        }
        KeyCode::Char('2') => {
            app.active_tab = ActiveTab::Estrenos;
            if app.estrenos_list.is_empty() {
                app.is_loading = true;
                app.loading_message = "Cargando estrenos...".to_string();
                spawn_load_estrenos(tx.clone(), app.client.clone());
            }
            return Ok(());
        }
        KeyCode::Char('3') => {
            app.active_tab = ActiveTab::Genres;
            return Ok(());
        }
        KeyCode::Char('4') => {
            app.active_tab = ActiveTab::History;
            app.reload_history();
            return Ok(());
        }
        KeyCode::Char('5') => {
            app.active_tab = ActiveTab::Library;
            app.reload_library();
            return Ok(());
        }
        KeyCode::Char('6') => {
            app.active_tab = ActiveTab::Help;
            return Ok(());
        }
        KeyCode::Tab => {
            if app.active_tab == ActiveTab::Genres {
                match app.active_pane {
                    ActivePane::List => match app.genres_column {
                        GenresColumn::Categories => {
                            if !app.genre_animes.is_empty() {
                                app.genres_column = GenresColumn::Animes;
                            } else {
                                app.active_pane = ActivePane::Detail;
                            }
                        }
                        GenresColumn::Animes => {
                            app.active_pane = ActivePane::Detail;
                        }
                    },
                    ActivePane::Detail => {
                        app.active_pane = ActivePane::List;
                        app.genres_column = GenresColumn::Categories;
                    }
                }
            } else {
                app.toggle_pane();
            }
            return Ok(());
        }
        KeyCode::BackTab => {
            if app.active_tab == ActiveTab::Genres {
                match app.active_pane {
                    ActivePane::List => match app.genres_column {
                        GenresColumn::Categories => {
                            app.active_pane = ActivePane::Detail;
                        }
                        GenresColumn::Animes => {
                            app.genres_column = GenresColumn::Categories;
                        }
                    },
                    ActivePane::Detail => {
                        app.active_pane = ActivePane::List;
                        if !app.genre_animes.is_empty() {
                            app.genres_column = GenresColumn::Animes;
                        } else {
                            app.genres_column = GenresColumn::Categories;
                        }
                    }
                }
            } else {
                app.toggle_pane();
            }
            return Ok(());
        }
        KeyCode::Char('/') => {
            app.active_tab = ActiveTab::Search;
            app.is_search_editing = true;
            return Ok(());
        }
        KeyCode::Char('q') => {
            app.should_quit = true;
            return Ok(());
        }
        _ => {}
    }

    // Navigation inside active pane
    if app.active_pane == ActivePane::Detail {
        handle_detail_pane_keys(app, tx, code).await
    } else {
        handle_list_pane_keys(app, tx, code).await
    }
}

async fn handle_modal_keys(
    app: &mut App,
    tx: &mpsc::Sender<AsyncEvent>,
    code: KeyCode,
) -> Result<()> {
    match &mut app.modal {
        ModalDialog::EpisodeActions { slug, title, ep_no, selected_index } => {
            match code {
                KeyCode::Esc => {
                    app.modal = ModalDialog::None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *selected_index < 2 {
                        *selected_index += 1;
                    }
                }
                KeyCode::Char('1') => {
                    let slug = slug.clone();
                    let title = title.clone();
                    let ep_no = *ep_no;
                    app.set_toast(&format!("Obteniendo enlaces para Ep. {:02}...", ep_no), false);
                    app.is_searching = true;
                    spawn_load_streams(tx.clone(), app.client.clone(), slug, title, ep_no, false);
                    app.modal = ModalDialog::None;
                }
                KeyCode::Char('2') | KeyCode::Char('d') => {
                    let slug = slug.clone();
                    let title = title.clone();
                    let ep_no = *ep_no;
                    app.set_toast(&format!("Obteniendo enlaces de descarga para Ep. {:02}...", ep_no), false);
                    app.is_searching = true;
                    spawn_load_streams(tx.clone(), app.client.clone(), slug, title, ep_no, true);
                    app.modal = ModalDialog::None;
                }
                KeyCode::Enter => {
                    let sel = *selected_index;
                    let slug = slug.clone();
                    let title = title.clone();
                    let ep_no = *ep_no;
                    app.modal = ModalDialog::None;
                    match sel {
                        0 => {
                            app.set_toast(&format!("Obteniendo enlaces para Ep. {:02}...", ep_no), false);
                            app.is_searching = true;
                            spawn_load_streams(tx.clone(), app.client.clone(), slug, title, ep_no, false);
                        }
                        1 => {
                            app.set_toast(&format!("Obteniendo enlaces de descarga para Ep. {:02}...", ep_no), false);
                            app.is_searching = true;
                            spawn_load_streams(tx.clone(), app.client.clone(), slug, title, ep_no, true);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        ModalDialog::SelectServer {
            slug,
            title,
            ep_no,
            sources,
            selected_index,
            is_download,
        } => {
            match code {
                KeyCode::Esc => {
                    app.modal = ModalDialog::None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if *selected_index > 0 {
                        *selected_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !sources.is_empty() && *selected_index + 1 < sources.len() {
                        *selected_index += 1;
                    }
                }
                KeyCode::Char(c @ '1'..='9') => {
                    let idx = (c as usize) - ('1' as usize);
                    if let Some(source) = sources.get(idx) {
                        let action = if *is_download {
                            AppAction::DownloadStream {
                                url: source.url.clone(),
                                server_name: source.server_name.clone(),
                                title: title.clone(),
                                ep_no: *ep_no,
                                slug: slug.clone(),
                            }
                        } else {
                            AppAction::PlayStream {
                                url: source.url.clone(),
                                server_name: source.server_name.clone(),
                                title: title.clone(),
                                ep_no: *ep_no,
                                slug: slug.clone(),
                            }
                        };
                        app.pending_action = Some(action);
                        app.modal = ModalDialog::None;
                    }
                }
                KeyCode::Enter => {
                    let action = sources.get(*selected_index).map(|source| {
                        if *is_download {
                            AppAction::DownloadStream {
                                url: source.url.clone(),
                                server_name: source.server_name.clone(),
                                title: title.clone(),
                                ep_no: *ep_no,
                                slug: slug.clone(),
                            }
                        } else {
                            AppAction::PlayStream {
                                url: source.url.clone(),
                                server_name: source.server_name.clone(),
                                title: title.clone(),
                                ep_no: *ep_no,
                                slug: slug.clone(),
                            }
                        }
                    });
                    app.modal = ModalDialog::None;
                    if let Some(act) = action {
                        app.pending_action = Some(act);
                    }
                }
                _ => {}
            }
        }
        ModalDialog::ConfirmClearHistory => {
            match code {
                KeyCode::Enter => {
                    let _ = crate::history::clear_history(&app.config.history_file);
                    app.reload_history();
                    app.modal = ModalDialog::None;
                    app.set_toast("Historial de reproducción borrado.", false);
                }
                KeyCode::Esc => {
                    app.modal = ModalDialog::None;
                }
                _ => {}
            }
        }
        ModalDialog::PlayerStatus { is_running, .. } => {
            match code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    if *is_running {
                        if let Some(mut child) = app.player_child.take() {
                            let _ = child.kill().await;
                        }
                    }
                    app.modal = ModalDialog::None;
                }
                KeyCode::Tab => {
                    app.modal = ModalDialog::None;
                }
                _ => {}
            }
        }
        ModalDialog::None => {}
    }
    Ok(())
}

async fn handle_search_typing(
    app: &mut App,
    tx: &mpsc::Sender<AsyncEvent>,
    code: KeyCode,
) -> Result<()> {
    match code {
        KeyCode::Enter => {
            let q = app.search_input.trim().to_string();
            if !q.is_empty() {
                app.is_loading = true;
                app.loading_message = format!("Buscando '{}'...", q);
                app.is_search_editing = false;
                app.current_request_id += 1;
                spawn_search(tx.clone(), app.client.clone(), app.current_request_id, q);
            }
        }
        KeyCode::Esc => {
            app.is_search_editing = false;
        }
        KeyCode::Backspace => {
            app.search_input.pop();
        }
        KeyCode::Char(c) => {
            app.search_input.push(c);
        }
        _ => {}
    }
    Ok(())
}

async fn handle_list_pane_keys(
    app: &mut App,
    tx: &mpsc::Sender<AsyncEvent>,
    code: KeyCode,
) -> Result<()> {
    match app.active_tab {
        ActiveTab::Search => {
            let len = app.search_results.len();
            if len == 0 {
                return Ok(());
            }
            let current = app.search_table_state.selected().unwrap_or(0);
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if current > 0 { current - 1 } else { len - 1 };
                    app.search_table_state.select(Some(next));
                    if let Some(item) = app.search_results.get(next) {
                        app.is_loading = true;
                        app.loading_message = "Cargando ficha...".to_string();
                        app.current_request_id += 1;
                        spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, item.slug.clone());
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let next = (current + 1) % len;
                    app.search_table_state.select(Some(next));
                    if let Some(item) = app.search_results.get(next) {
                        app.is_loading = true;
                        app.loading_message = "Cargando ficha...".to_string();
                        app.current_request_id += 1;
                        spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, item.slug.clone());
                    }
                }
                KeyCode::Enter => {
                    app.active_pane = ActivePane::Detail;
                    app.detail_tab = DetailTab::Episodes;
                }
                _ => {}
            }
        }
        ActiveTab::Estrenos => {
            let len = app.estrenos_list.len();
            if len == 0 {
                return Ok(());
            }
            let current = app.estrenos_table_state.selected().unwrap_or(0);
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if current > 0 { current - 1 } else { len - 1 };
                    app.estrenos_table_state.select(Some(next));
                    if let Some(item) = app.estrenos_list.get(next) {
                        app.is_loading = true;
                        app.loading_message = "Cargando ficha...".to_string();
                        app.current_request_id += 1;
                        spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, item.slug.clone());
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let next = (current + 1) % len;
                    app.estrenos_table_state.select(Some(next));
                    if let Some(item) = app.estrenos_list.get(next) {
                        app.is_loading = true;
                        app.loading_message = "Cargando ficha...".to_string();
                        app.current_request_id += 1;
                        spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, item.slug.clone());
                    }
                }
                KeyCode::Enter => {
                    if let Some(item) = app.estrenos_list.get(current).cloned() {
                        app.modal = ModalDialog::EpisodeActions {
                            slug: item.slug,
                            title: item.title,
                            ep_no: item.episode,
                            selected_index: 0,
                        };
                    }
                }
                _ => {}
            }
        }
        ActiveTab::Genres => match app.genres_column {
            GenresColumn::Categories => {
                let len = app.genres_list.len();
                let current = app.genres_table_state.selected().unwrap_or(0);
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        let next = if current > 0 { current - 1 } else { len - 1 };
                        app.genres_table_state.select(Some(next));
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let next = (current + 1) % len;
                        app.genres_table_state.select(Some(next));
                    }
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                        if let Some(cat) = app.genres_list.get(current).cloned() {
                            app.genre_animes.clear();
                            app.genre_animes_table_state.select(None);
                            app.selected_genre_id = Some(cat.id.clone());
                            app.genres_column = GenresColumn::Animes;
                            app.is_loading = true;
                            app.loading_message = format!("Cargando {}...", cat.name);
                            spawn_load_genre_animes(tx.clone(), app.client.clone(), cat.id);
                        }
                    }
                    _ => {}
                }
            }
            GenresColumn::Animes => {
                let len = app.genre_animes.len();
                let current = app.genre_animes_table_state.selected().unwrap_or(0);
                match code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if len > 0 {
                            let next = if current > 0 { current - 1 } else { len - 1 };
                            app.genre_animes_table_state.select(Some(next));
                            if let Some(item) = app.genre_animes.get(next) {
                                app.is_loading = true;
                                app.loading_message = "Cargando ficha...".to_string();
                                app.current_request_id += 1;
                                spawn_load_details(
                                    tx.clone(),
                                    app.client.clone(),
                                    app.current_request_id,
                                    item.slug.clone(),
                                );
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if len > 0 {
                            let next = (current + 1) % len;
                            app.genre_animes_table_state.select(Some(next));
                            if let Some(item) = app.genre_animes.get(next) {
                                app.is_loading = true;
                                app.loading_message = "Cargando ficha...".to_string();
                                app.current_request_id += 1;
                                spawn_load_details(
                                    tx.clone(),
                                    app.client.clone(),
                                    app.current_request_id,
                                    item.slug.clone(),
                                );
                            }
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => {
                        app.genres_column = GenresColumn::Categories;
                    }
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') if len > 0 => {
                        app.active_pane = ActivePane::Detail;
                        app.detail_tab = DetailTab::Episodes;
                    }
                    _ => {}
                }
            }
        },
        ActiveTab::History => {
            let len = app.history_list.len();
            if len == 0 {
                return Ok(());
            }
            let current = app.history_table_state.selected().unwrap_or(0);
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if current > 0 { current - 1 } else { len - 1 };
                    app.history_table_state.select(Some(next));
                    if let Some(item) = app.history_list.get(next) {
                        app.is_loading = true;
                        app.loading_message = "Cargando ficha...".to_string();
                        app.current_request_id += 1;
                        spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, item.slug.clone());
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let next = (current + 1) % len;
                    app.history_table_state.select(Some(next));
                    if let Some(item) = app.history_list.get(next) {
                        app.is_loading = true;
                        app.loading_message = "Cargando ficha...".to_string();
                        app.current_request_id += 1;
                        spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, item.slug.clone());
                    }
                }
                KeyCode::Enter => {
                    if let Some(item) = app.history_list.get(current).cloned() {
                        app.modal = ModalDialog::EpisodeActions {
                            slug: item.slug,
                            title: item.title,
                            ep_no: item.episode + 1,
                            selected_index: 0,
                        };
                    }
                }
                KeyCode::Char('D') => {
                    app.modal = ModalDialog::ConfirmClearHistory;
                }
                _ => {}
            }
        }
        ActiveTab::Library => {
            let len = app.library_list.len();
            if len == 0 {
                return Ok(());
            }
            let current = app.library_table_state.selected().unwrap_or(0);
            match code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let next = if current > 0 { current - 1 } else { len - 1 };
                    app.library_table_state.select(Some(next));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let next = (current + 1) % len;
                    app.library_table_state.select(Some(next));
                }
                KeyCode::Enter => {
                    if let Some(file) = app.library_list.get(current) {
                        let path_str = file.path.to_string_lossy().to_string();
                        let _ = Player::play(&app.config.player, &path_str, &file.file_name, false);
                    }
                }
                _ => {}
            }
        }
        ActiveTab::Help => {}
    }
    Ok(())
}

async fn handle_detail_pane_keys(
    app: &mut App,
    tx: &mpsc::Sender<AsyncEvent>,
    code: KeyCode,
) -> Result<()> {
    match code {
        KeyCode::Left | KeyCode::Char('h') => {
            app.detail_tab = match app.detail_tab {
                DetailTab::Overview => DetailTab::Related,
                DetailTab::Episodes => DetailTab::Overview,
                DetailTab::Related => DetailTab::Episodes,
            };
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.detail_tab = match app.detail_tab {
                DetailTab::Overview => DetailTab::Episodes,
                DetailTab::Episodes => DetailTab::Related,
                DetailTab::Related => DetailTab::Overview,
            };
        }
        KeyCode::Up | KeyCode::Char('k') => match app.detail_tab {
            DetailTab::Episodes => {
                let len = app.episodes_list.len();
                if len > 0 {
                    let cur = app.episodes_table_state.selected().unwrap_or(0);
                    let next = if cur > 0 { cur - 1 } else { len - 1 };
                    app.episodes_table_state.select(Some(next));
                }
            }
            DetailTab::Related => {
                if let Some(d) = &app.current_details {
                    let len = d.related_seasons.len();
                    if len > 0 {
                        let cur = app.related_table_state.selected().unwrap_or(0);
                        let next = if cur > 0 { cur - 1 } else { len - 1 };
                        app.related_table_state.select(Some(next));
                    }
                }
            }
            _ => {}
        },
        KeyCode::Down | KeyCode::Char('j') => match app.detail_tab {
            DetailTab::Episodes => {
                let len = app.episodes_list.len();
                if len > 0 {
                    let cur = app.episodes_table_state.selected().unwrap_or(0);
                    let next = (cur + 1) % len;
                    app.episodes_table_state.select(Some(next));
                }
            }
            DetailTab::Related => {
                if let Some(d) = &app.current_details {
                    let len = d.related_seasons.len();
                    if len > 0 {
                        let cur = app.related_table_state.selected().unwrap_or(0);
                        let next = (cur + 1) % len;
                        app.related_table_state.select(Some(next));
                    }
                }
            }
            _ => {}
        },
        KeyCode::Enter => match app.detail_tab {
            DetailTab::Episodes => {
                if let (Some(d), Some(ep)) = (&app.current_details, app.episodes_list.get(app.episodes_table_state.selected().unwrap_or(0))) {
                    app.modal = ModalDialog::EpisodeActions {
                        slug: d.slug.clone(),
                        title: d.title.clone(),
                        ep_no: ep.number,
                        selected_index: 0,
                    };
                }
            }
            DetailTab::Related => {
                if let Some(d) = &app.current_details {
                    let cur = app.related_table_state.selected().unwrap_or(0);
                    if let Some(rel) = d.related_seasons.get(cur).cloned() {
                        app.is_loading = true;
                        app.loading_message = "Cargando temporada...".to_string();
                        app.current_request_id += 1;
                        spawn_load_details(tx.clone(), app.client.clone(), app.current_request_id, rel.slug);
                    }
                }
            }
            _ => {}
        },
        KeyCode::Char('d') => {
            if let (Some(d), Some(ep)) = (&app.current_details, app.episodes_list.get(app.episodes_table_state.selected().unwrap_or(0))) {
                app.modal = ModalDialog::EpisodeActions {
                    slug: d.slug.clone(),
                    title: d.title.clone(),
                    ep_no: ep.number,
                    selected_index: 1, // Preselect download
                };
            }
        }
        KeyCode::Esc => {
            app.active_pane = ActivePane::List;
        }
        _ => {}
    }
    Ok(())
}
