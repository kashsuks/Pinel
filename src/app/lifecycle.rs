//! This file is used in order to conduct a periodic
//! states and update check on the system

use super::*;

impl App {
    /// Creates the application state and schedules an initial update check.
    #[allow(dead_code)]
    pub fn new() -> (Self, iced::Task<Message>) {
        Self::new_with_path(None)
    }

    pub fn new_with_path(startup_path: Option<std::path::PathBuf>) -> (Self, iced::Task<Message>) {
        let mut app = Self::default();

        let update_task = iced::Task::perform(
            crate::features::updater::check_for_update(),
            |result| match result {
                Some(info) => Message::UpdateAvailable(info),
                None => Message::DismissUpdateBanner,
            },
        );

        let startup_task = match startup_path {
            Some(path) if path.is_dir() => iced::Task::done(Message::FolderOpened(path)),
            Some(path) => Self::open_path_task(path),
            // An explicit CLI path always wins over the saved session.
            None => app.restore_last_session_task(),
        };

        (app, iced::Task::batch([update_task, startup_task]))
    }

    /// Restores the last saved session: reopens its folder and its tabs.
    ///
    /// The tabs open one after another in their saved order, then
    /// `Message::SessionRestoreFinished` selects the tab that was active.
    pub(super) fn restore_last_session_task(&mut self) -> iced::Task<Message> {
        if !self.editor_preferences.restore_session_enabled {
            return iced::Task::none();
        }

        let Some(session) = crate::config::session::load_session() else {
            return iced::Task::none();
        };

        let active = session.active_tab_index.clone();
        self.pending_active_tab_path = session.active_tab_index;
        self.pending_cursor_restores = session.cursor_position;

        let mut tasks = Vec::new();

        if let Some(folder) = session.folder {
            if folder.is_dir() {
                tasks.push(iced::Task::done(Message::FolderOpened(folder)));
            }
        }

        let open_tabs = session
            .open_tabs
            .into_iter()
            .filter(|path| path.is_file())
            .fold(iced::Task::none(), |acc, path| {
                acc.chain(Self::open_path_task(path))
            });
        tasks.push(open_tabs.chain(iced::Task::done(Message::SessionRestoreFinished(active))));

        iced::Task::batch(tasks)
    }
}
