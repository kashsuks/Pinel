//! This file is used in order to conduct a periodic
//! states and update check on the system

use super::*;

impl App {
    /// Creates the application state and schedules an initial update check.
    pub fn new() -> (Self, iced::Task<Message>) {
        Self::new_with_path(None)
    }

    pub fn new_with_path(startup_path: Option<std::path::PathBuf>) -> (Self, iced::Task<Message>) {
        let app = Self::default();

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
            None => iced::Task::none(),
        };

        (app, iced::Task::batch([update_task, startup_task]))
    }
}
