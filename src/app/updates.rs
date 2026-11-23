//! Update tasks and notification policy at the application boundary.

use super::*;
use crate::features::updater;

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
