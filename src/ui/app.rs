use crate::config::Config;
use crate::history::{self, HistoryEntry};
use crate::jkanime::models::*;
use crate::jkanime::JkClient;
use crate::library::{self, LocalDownload};
use ratatui::widgets::{ScrollbarState, TableState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Search = 0,
    Estrenos = 1,
    Genres = 2,
    History = 3,
    Library = 4,
    Help = 5,
}

impl ActiveTab {
    pub fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::Search,
            ActiveTab::Estrenos,
            ActiveTab::Genres,
            ActiveTab::History,
            ActiveTab::Library,
            ActiveTab::Help,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            ActiveTab::Search => "🔍 Buscar",
            ActiveTab::Estrenos => "🌟 Estrenos",
            ActiveTab::Genres => "🏷️ Géneros",
            ActiveTab::History => "🕒 Historial",
            ActiveTab::Library => "📂 Descargas",
            ActiveTab::Help => "ℹ️ Ayuda",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    List,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenresColumn {
    Categories,
    Animes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Overview = 0,
    Episodes = 1,
    Related = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalDialog {
    None,
    EpisodeActions {
        slug: String,
        title: String,
        ep_no: u32,
        selected_index: usize,
    },
    SelectServer {
        slug: String,
        title: String,
        ep_no: u32,
        sources: Vec<StreamSource>,
        selected_index: usize,
        is_download: bool,
    },
    ConfirmClearHistory,
    PlayerStatus {
        title: String,
        ep_no: u32,
        server_name: String,
        logs: Vec<String>,
        is_running: bool,
        has_error: bool,
    },
}

#[derive(Debug, Clone)]
pub enum AppAction {
    PlayStream {
        url: String,
        server_name: String,
        title: String,
        ep_no: u32,
        slug: String,
    },
    DownloadStream {
        url: String,
        server_name: String,
        title: String,
        ep_no: u32,
        slug: String,
    },
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub is_error: bool,
    pub timestamp: std::time::Instant,
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum AsyncEvent {
    SearchSuccess {
        request_id: usize,
        query: String,
        results: Vec<AnimeItem>,
    },
    DetailsSuccess {
        request_id: usize,
        slug: String,
        details: AnimeDetails,
        episodes: Vec<EpisodeItem>,
    },
    EstrenosSuccess {
        estrenos: Vec<EstrenoItem>,
    },
    GenreAnimesSuccess {
        genre_id: String,
        animes: Vec<AnimeItem>,
    },
    StreamsSuccess {
        slug: String,
        title: String,
        ep_no: u32,
        sources: Vec<StreamSource>,
        is_download: bool,
    },
    NetworkError {
        message: String,
    },
    PlayerOutputLine(String),
    PlayerTerminated {
        success: bool,
        error_msg: Option<String>,
    },
}

#[allow(dead_code)]
pub struct App {
    pub client: JkClient,
    pub config: Config,
    pub active_tab: ActiveTab,
    pub active_pane: ActivePane,
    pub detail_tab: DetailTab,
    pub modal: ModalDialog,
    pub should_quit: bool,
    pub is_loading: bool,
    pub is_searching: bool,
    pub loading_message: String,
    pub spinner_tick: usize,
    pub current_request_id: usize,
    pub toast: Option<Toast>,

    // Search state
    pub search_input: String,
    pub is_search_editing: bool,
    pub search_results: Vec<AnimeItem>,
    pub search_table_state: TableState,
    pub search_scrollbar_state: ScrollbarState,

    // Estrenos state
    pub estrenos_list: Vec<EstrenoItem>,
    pub estrenos_table_state: TableState,
    pub estrenos_scrollbar_state: ScrollbarState,

    // Genres state
    pub genres_list: Vec<GenreCategory>,
    pub genres_table_state: TableState,
    pub genres_column: GenresColumn,
    pub selected_genre_id: Option<String>,
    pub genre_animes: Vec<AnimeItem>,
    pub genre_animes_table_state: TableState,

    // Details & Episodes state
    pub current_details: Option<AnimeDetails>,
    pub episodes_list: Vec<EpisodeItem>,
    pub episodes_table_state: TableState,
    pub episodes_scrollbar_state: ScrollbarState,
    pub related_table_state: TableState,

    // History state
    pub history_list: Vec<HistoryEntry>,
    pub history_table_state: TableState,

    // Offline Library state
    pub library_list: Vec<LocalDownload>,
    pub library_table_state: TableState,

    // Out-of-TUI action to perform (e.g., spawn player)
    pub pending_action: Option<AppAction>,
    pub player_child: Option<tokio::process::Child>,
}

#[allow(dead_code)]
impl App {
    pub fn new(config: Config, client: JkClient) -> Self {
        let history_list = history::load_history(&config.history_file);
        let library_list = library::scan_library(&config.download_dir);
        let genres_list = crate::jkanime::get_available_genres();

        let mut search_table_state = TableState::default();
        search_table_state.select(Some(0));

        let mut estrenos_table_state = TableState::default();
        estrenos_table_state.select(Some(0));

        let mut genres_table_state = TableState::default();
        genres_table_state.select(Some(0));

        let mut genre_animes_table_state = TableState::default();
        genre_animes_table_state.select(Some(0));

        let mut episodes_table_state = TableState::default();
        episodes_table_state.select(Some(0));

        let mut related_table_state = TableState::default();
        related_table_state.select(Some(0));

        let mut history_table_state = TableState::default();
        history_table_state.select(Some(0));

        let mut library_table_state = TableState::default();
        library_table_state.select(Some(0));

        Self {
            client,
            config,
            active_tab: ActiveTab::Search,
            active_pane: ActivePane::List,
            detail_tab: DetailTab::Overview,
            modal: ModalDialog::None,
            should_quit: false,
            is_loading: false,
            is_searching: false,
            loading_message: String::new(),
            spinner_tick: 0,
            current_request_id: 0,
            toast: None,

            search_input: String::new(),
            is_search_editing: true,
            search_results: Vec::new(),
            search_table_state,
            search_scrollbar_state: ScrollbarState::default(),

            estrenos_list: Vec::new(),
            estrenos_table_state,
            estrenos_scrollbar_state: ScrollbarState::default(),

            genres_list,
            genres_table_state,
            genres_column: GenresColumn::Categories,
            selected_genre_id: None,
            genre_animes: Vec::new(),
            genre_animes_table_state,

            current_details: None,
            episodes_list: Vec::new(),
            episodes_table_state,
            episodes_scrollbar_state: ScrollbarState::default(),
            related_table_state,

            history_list,
            history_table_state,

            library_list,
            library_table_state,

            pending_action: None,
            player_child: None,
        }
    }

    pub fn spinner_char(&self) -> &'static str {
        const SPINNER_CHARS: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        SPINNER_CHARS[(self.spinner_tick / 4) % SPINNER_CHARS.len()]
    }

    pub fn set_toast(&mut self, message: &str, is_error: bool) {
        self.toast = Some(Toast {
            message: message.to_string(),
            is_error,
            timestamp: std::time::Instant::now(),
        });
    }

    pub fn reload_history(&mut self) {
        self.history_list = history::load_history(&self.config.history_file);
        if let Some(sel) = self.history_table_state.selected() {
            if sel >= self.history_list.len() && !self.history_list.is_empty() {
                self.history_table_state.select(Some(self.history_list.len() - 1));
            }
        }
    }

    pub fn reload_library(&mut self) {
        self.library_list = library::scan_library(&self.config.download_dir);
        if let Some(sel) = self.library_table_state.selected() {
            if sel >= self.library_list.len() && !self.library_list.is_empty() {
                self.library_table_state.select(Some(self.library_list.len() - 1));
            }
        }
    }

    pub fn select_next_tab(&mut self) {
        let current = self.active_tab as usize;
        let next = (current + 1) % ActiveTab::all().len();
        self.active_tab = ActiveTab::all()[next];
    }

    pub fn select_prev_tab(&mut self) {
        let current = self.active_tab as usize;
        let prev = if current == 0 {
            ActiveTab::all().len() - 1
        } else {
            current - 1
        };
        self.active_tab = ActiveTab::all()[prev];
    }

    pub fn toggle_pane(&mut self) {
        self.active_pane = match self.active_pane {
            ActivePane::List => ActivePane::Detail,
            ActivePane::Detail => ActivePane::List,
        };
    }
}
