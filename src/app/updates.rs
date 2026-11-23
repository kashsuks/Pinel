//! Update tasks and notification policy at the application boundary.

use super::{App, Tab, TabKind, UpdateInfo};
use crate::features::updater;
use crate::message::Message;
use iced::widget::markdown;
use std::path::PathBuf;

impl App {
    pub(super) fn save_update_state(&mut self) {
        if let Err(error) = self.update_state.save() {
            self.update_status = format!("Could not save update preferences: {error}");
        }
    }

    pub(super) fn start_update_check(&mut self, manual: bool) -> iced::Task<Message> {
        if self.update_check_in_flight {
            return iced::Task::none();
        }
        self.update_check_in_flight = true;
        self.update_status = "Checking for updates…".into();
        iced::Task::perform(
            async { updater::check_for_update().await.map_err(|error| error.to_string()) },
            move |result| Message::UpdateChecked { manual, result },
        )
    }

    pub(super) fn finish_update_check(
        &mut self,
        manual: bool,
        result: Result<Option<UpdateInfo>, String>,
    ) {
        self.update_check_in_flight = false;
        self.update_state.record_check(&result, updater::now());
        self.update_status = match &result {
            Ok(Some(info)) => format!("Pinel {} is available.", info.version),
            Ok(None) => "You're up to date.".into(),
            Err(_) => "Unable to check for updates. Please try again later.".into(),
        };
        self.update_banner = if manual {
            // A deliberate check can reveal a skipped release, even with automatic checks off.
            result.ok().flatten()
        } else {
            self.update_state.notification(updater::CURRENT_VERSION, updater::now())
        };
        self.save_update_state();
    }
}

impl App {
    pub(super) fn start_release_notes(&mut self, manual: bool) -> iced::Task<Message> {
        if !manual
            && (!self.startup_ready
                || self.startup_page_open
                || self.update_state.disabled
                || !self.update_state.needs_release_notes(updater::CURRENT_VERSION))
        {
            return iced::Task::none();
        }
        self.release_notes_manual |= manual;
        if self.release_notes_in_flight {
            return iced::Task::none();
        }
        if let Some(notes) = self
            .update_state
            .release_notes
            .clone()
            .filter(|notes| notes.version == updater::CURRENT_VERSION)
        {
            return self.finish_release_notes(Ok(notes));
        }
        self.release_notes_in_flight = true;
        if manual {
            self.update_status = "Loading release notes…".into();
        }
        iced::Task::perform(
            async { updater::fetch_release_notes().await.map_err(|error| error.to_string()) },
            Message::ReleaseNotesLoaded,
        )
    }

    pub(super) fn finish_release_notes(
        &mut self,
        result: Result<updater::ReleaseNotes, String>,
    ) -> iced::Task<Message> {
        self.release_notes_in_flight = false;
        let manual = std::mem::take(&mut self.release_notes_manual);
        let notes = match result {
            Ok(notes) => notes,
            Err(_) => {
                // Keep the installed version unseen so the next launch can retry.
                if manual {
                    self.update_status =
                        "Release notes are unavailable. Please try again later.".into();
                }
                return iced::Task::none();
            },
        };
        // An automatic request may have finished after the user disabled checks.
        if !manual && self.update_state.disabled {
            return iced::Task::none();
        }
        let path = PathBuf::from(format!("pinel:release-notes:{}", notes.version));
        let index = if let Some(index) = self.tabs.iter().position(|tab| tab.path == path) {
            index
        } else {
            self.tabs.push(release_notes_tab(&notes));
            self.tabs.len() - 1
        };
        if self.update_state.needs_release_notes(&notes.version) {
            self.update_state.last_seen_version = Some(notes.version.clone());
        }
        self.update_state.release_notes = Some(notes);
        if manual {
            self.update_status = "Release notes loaded.".into();
        }
        self.save_update_state();
        self.settings_open = false;
        self.update(Message::TabSelected(index))
    }
}

fn release_notes_tab(notes: &updater::ReleaseNotes) -> Tab {
    Tab {
        path: PathBuf::from(format!("pinel:release-notes:{}", notes.version)),
        name: format!("What's New — {}", notes.version),
        kind: TabKind::Preview {
            md_items: markdown::parse(&notes.markdown()).collect(),
        },
        autosave_requested_at: None,
        autosave_in_flight: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_notes_tab_is_read_only_and_has_no_autosave() {
        let tab = release_notes_tab(&updater::ReleaseNotes {
            version: "1.0.0".into(),
            url: "https://github.com/kashsuks/pinel/releases/tag/v1.0.0".into(),
            body: "## Changes\n- A better editor".into(),
        });
        assert_eq!(tab.name, "What's New — 1.0.0");
        assert!(!tab.path.is_absolute());
        assert!(matches!(tab.kind, TabKind::Preview { .. }));
        assert!(tab.autosave_requested_at.is_none());
        assert!(!tab.autosave_in_flight);
    }
}
