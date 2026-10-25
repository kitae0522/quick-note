//! Native application state, construction and framework integration.
use super::Mode;
use super::files;
use super::settings_section::SettingsSection;
use crate::appearance;
use crate::appearance::Theme;
use crate::i18n;
use crate::model::CursorLocation;
use crate::model::Note;
use crate::model::NoteSummary;
use crate::model::SearchIndex;
use crate::shortcut::Platform;
use crate::shortcut::Shortcuts;
use crate::storage::Result;
use crate::storage::Store;
use crate::ui::EditorMetrics;
use crate::ui::header::HeaderState;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::Rangef;
use eframe::egui::Rect;
use eframe::egui::Ui;
use eframe::egui::Visuals;
use egui_commonmark::CommonMarkCache;
use std::fs;
use std::path::PathBuf;

/// Owns one locked note store and its native-window interaction state.
pub struct QuickNoteApp {
    pub store: Store,
    pub note: Note,
    pub tabs: Vec<NoteSummary>,
    pub theme: Theme,
    pub mode: Mode,
    pub header: HeaderState,
    pub dirty: bool,
    pub error: Option<i18n::Error>,
    pub pending_delete: Option<String>,
    pub locale: i18n::Localizer,
    pub(super) directory: PathBuf,
    pub(super) theme_id: String,
    pub(super) theme_error: Option<i18n::Error>,
    pub(super) shortcuts: Shortcuts,
    pub(super) shortcut_error: Option<i18n::Error>,
    pub(super) fonts: appearance::FontCatalog,
    pub(super) font_pending: bool,
    pub(super) font_query: String,
    pub(super) font_focus: bool,
    pub(super) font_action_focus: Option<Id>,
    pub(super) font_order_focus: Option<Id>,
    pub(super) font_workspace_header: Option<Id>,
    pub(super) font_workspace_bounds: Option<Rect>,
    pub(super) font_index: usize,
    pub(super) font_choices: appearance::FontChoices,
    pub(super) focus_editor: bool,
    pub(super) restore_cursor: bool,
    pub(super) editor_metrics: EditorMetrics,
    pub(super) location: CursorLocation,
    pub(super) search_open: bool,
    pub(super) search_focus_popup: bool,
    pub(super) search_results: Vec<NoteSummary>,
    pub(super) search_cache: Option<SearchIndex>,
    pub(super) search_query: Option<String>,
    pub(super) search_index: usize,
    pub(super) search_reveal: bool,
    pub(super) search_rect: Rect,
    pub(super) search_was_focused: bool,
    pub(super) previous_focus: Option<Id>,
    pub(super) settings: Option<SettingsSection>,
    pub(super) settings_request: Option<SettingsSection>,
    pub(super) settings_focus_new: bool,
    pub(super) composing: bool,
    pub(super) modal_new: bool,
    pub(super) close_blocked: bool,
    pub(super) window_was_focused: bool,
    pub(super) markdown_enabled: bool,
    pub(super) reduce_motion: bool,
    pub(super) system_reduce_motion: bool,
    pub(super) markdown_cache: Option<CommonMarkCache>,
    pub(super) recovery_id: Option<String>,
    pub(super) pending_menu: Vec<crate::menu::MenuAction>,
    #[cfg(target_os = "macos")]
    pub(super) native_menu: Option<crate::menu::MacMenu>,
    #[cfg(target_os = "macos")]
    pub(super) menu_dirty: bool,
    pub(super) _session_lock: fs::File,
}

impl QuickNoteApp {
    /// Opens the note directory, migrates old data and restores a pending recovery copy.
    /// Returns an error when the directory is locked or the existing store cannot open.
    pub fn try_new(directory: PathBuf) -> Result<Self> {
        fs::create_dir_all(directory.join("Themes"))?;
        let session_lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(directory.join("session.lock"))?;
        session_lock
            .try_lock()
            .map_err(|error| i18n::message(|m| &m.error_lock).cause("error", error))?;
        let mut store = Store::try_new(&directory.join("notes.sqlite3"))?;
        let language = store.setting("language")?.unwrap_or_else(|| "en".into());
        let locale = i18n::Localizer::load(i18n::Language::from_code(&language), &directory)?;
        let (shortcuts, shortcut_error) = match Shortcuts::load(&directory, Platform::current()) {
            Ok(shortcuts) => (shortcuts, None),
            Err(error) => (
                Shortcuts::defaults(Platform::current()),
                Some(
                    i18n::message(|m| &m.error_shortcut_load)
                        .cause("error", error)
                        .into(),
                ),
            ),
        };
        let note = store.resume()?;
        let tabs = store.summaries()?;
        let markdown_enabled = store.setting("markdownEnabled")?.as_deref() != Some("false");
        let reduce_motion = store.setting("reduceMotion")?.as_deref() == Some("true");
        let theme_id = store.setting("theme")?.unwrap_or_else(|| "helium".into());
        let (mut theme, mut theme_error) = match files::read_theme(&directory, &theme_id) {
            Ok(theme) => (theme, None),
            Err(error) => (
                Theme::builtin("helium"),
                Some(
                    i18n::message(|m| &m.error_theme_load)
                        .cause("error", error)
                        .into(),
                ),
            ),
        };
        if let Err(error) = super::font_preferences::apply_saved(&store, &mut theme) {
            theme_error.get_or_insert(error);
        }
        let editor_metrics = EditorMetrics::new(&note);
        let (_, caret) = editor_metrics
            .lines()
            .char_range(&note.body, &note.selection);
        let location = editor_metrics.lines().location(&note.body, caret);
        let mut app = Self {
            store,
            locale,
            note,
            tabs,
            theme,
            theme_id,
            theme_error,
            shortcuts,
            shortcut_error,
            fonts: appearance::FontCatalog::default(),
            font_pending: false,
            font_query: String::new(),
            font_focus: false,
            font_action_focus: None,
            font_order_focus: None,
            font_workspace_header: None,
            font_workspace_bounds: None,
            font_index: 0,
            font_choices: appearance::FontChoices::default(),
            directory,
            mode: Mode::Edit,
            header: HeaderState {
                reveal_active: true,
                ..Default::default()
            },
            dirty: false,
            error: None,
            pending_delete: None,
            focus_editor: true,
            restore_cursor: true,
            editor_metrics,
            location,
            search_open: false,
            search_focus_popup: false,
            search_results: Vec::new(),
            search_cache: None,
            search_query: None,
            search_index: 0,
            search_reveal: true,
            search_rect: Rect::NOTHING,
            search_was_focused: false,
            previous_focus: None,
            settings: None,
            settings_request: None,
            settings_focus_new: false,
            composing: false,
            modal_new: false,
            close_blocked: false,
            window_was_focused: true,
            markdown_enabled,
            reduce_motion,
            system_reduce_motion: true,
            markdown_cache: None,
            recovery_id: None,
            pending_menu: Vec::new(),
            #[cfg(target_os = "macos")]
            native_menu: None,
            #[cfg(target_os = "macos")]
            menu_dirty: false,
            _session_lock: session_lock,
        };
        app.restore_recovery();
        if let Err(error) = app.fonts.preload(&app.theme.typography.font_families()) {
            app.theme_error.get_or_insert(error);
        }
        Ok(app)
    }

    pub fn configure(&mut self, context: &Context) {
        self.configure_editor_font(context);
        self.theme.apply(context);
        self.apply_language(context);
        self.configure_motion(context);
    }
}

impl eframe::App for QuickNoteApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        frame.set_traffic_lights_position(
            ui.ctx(),
            Rangef::new(0.0, self.theme.layout.toolbar_height),
            12.0,
        );
        self.show(ui);
    }
    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        self.theme.palette().background.to_normalized_gamma_f32()
    }
}
