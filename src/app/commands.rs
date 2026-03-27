//! This module is used for defining some basic commands
//! Such as opening files, sidebar, terminal access
//! TODO: More commands for commands access and other
//!
//! Example file usage:
//!
//! This code is used in `terminal.rs` under the root
//!
//! ```rust
//! "Toggle Terminal" => {
//!     return self.toggle_terminal_panel();
//! }
//! ```
//!
//! A simple implementation is being used to invoke
//! a command action

use super::*;

impl App {
    pub(super) fn execute_palette_command(&mut self, command: &str) -> iced::Task<Message> {
        match command {
            "Toggle Sidebar" => {
                self.sidebar_visible = !self.sidebar_visible;
            }
            "Open File" => {
                return iced::Task::perform(async {}, |_| Message::OpenFileDialog);
            }
            "Open Folder" => {
                return iced::Task::perform(async {}, |_| Message::OpenFolderDialog);
            }
            "Toggle Terminal" => {
                return self.toggle_terminal_panel();
            }
            "Find and Replace" => {
                self.find_replace.toggle();
                if self.find_replace.open {
                    self.vim_refresh_cursor_style();
                    return iced::widget::operation::focus(self.find_input_id.clone());
                }
            }
            "New File" => {
                let editor = self.configured_code_editor("", "txt");
                self.tabs.push(Tab {
                    path: PathBuf::from("untitled"),
                    name: "untitled".to_string(),
                    kind: TabKind::Editor {
                        code_editor: editor,
                        buffer: crate::features::editor_buffer::EditorBuffer::from_text(""),
                    },
                    autosave_requested_at: None,
                    autosave_in_flight: false,
                });
                self.active_tab = Some(self.tabs.len() - 1);
                self.vim_refresh_cursor_style();
            }
            "Save File" => {
                return iced::Task::perform(async {}, |_| Message::SaveFile);
            }
            "Close Tab" => {
                return iced::Task::perform(async {}, |_| Message::CloseActiveTab);
            }
            "Settings" => {
                self.settings_open = !self.settings_open;
            }
            "Theme" => {
                self.settings_open = true;
                self.settings_section = "theme".to_string();
                self.theme_dropdown_open = false;
            }
            "Save As" => {
                return iced::Task::perform(async {}, |_| Message::SaveAs);
            }
            "Toggle Fullscreen" => {
                return iced::Task::perform(async {}, |_| {
                    Message::ToggleFullscreen(window::Mode::Fullscreen)
                });
            }
            "Render Markdown" => {
                return iced::Task::perform(async {}, |_| Message::PreviewMarkdown);
            }
            _ => {}
        }
        self.vim_refresh_cursor_style();
        iced::Task::none()
    }
}
