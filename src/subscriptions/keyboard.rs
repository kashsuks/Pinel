//! Keyboard event subscription handlers.

use iced::{keyboard::Key, window, Event, Subscription};

use crate::message::Message;

/// Emits keyboard shortcut messages for global editor actions.
pub fn shortcuts() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            shortcut_for(&key, modifiers, false)
        },
        _ => None,
    })
}

/// Emits keyboard shortcut messages while the embedded terminal has focus.
///
/// Bare-Ctrl chords are left to the terminal, except Ctrl+J and Ctrl+Shift chords
/// other than Ctrl+Shift+V.
pub fn terminal_shortcuts() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            shortcut_for(&key, modifiers, true)
        },
        _ => None,
    })
}

/// Emits startup page messages while the first-launch page is shown.
///
/// Replaces [`shortcuts`] there, so editor shortcuts can't fire behind the page.
pub fn startup_page() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => startup_key_for(&key),
        _ => None,
    })
}

/// Maps a key press on the startup page to its message: Enter continues.
pub fn startup_key_for(key: &Key) -> Option<Message> {
    match key {
        Key::Named(iced::keyboard::key::Named::Enter) => Some(Message::StartupPageDone),
        _ => None,
    }
}

/// Maps a key press to the editor shortcut message it triggers, if any.
///
/// When `terminal_focused` is true, bare-Ctrl chords (used by readline and the
/// terminal) are ignored, except Ctrl+J and Ctrl+Shift chords other than Ctrl+Shift+V.
pub fn shortcut_for(
    key: &Key,
    modifiers: iced::keyboard::Modifiers,
    terminal_focused: bool,
) -> Option<Message> {
    let primary = modifiers.command();
    let navigation_msg = match key {
        Key::Named(iced::keyboard::key::Named::ArrowUp) => Some(Message::FuzzyFinderNavigate(-1)),
        Key::Named(iced::keyboard::key::Named::ArrowDown) => Some(Message::FuzzyFinderNavigate(1)),
        Key::Named(iced::keyboard::key::Named::Enter) => Some(Message::FuzzyFinderSelect),
        _ => None,
    };

    if navigation_msg.is_some() {
        return navigation_msg;
    }

    if terminal_focused && modifiers.control() && !modifiers.logo() {
        let allowed = match key {
            Key::Character(c) if modifiers.shift() => !c.as_str().eq_ignore_ascii_case("v"),
            Key::Character(c) => c.as_str().eq_ignore_ascii_case("j"),
            _ => false,
        };
        if !allowed {
            return None;
        }
    }

    if let Key::Character(c) = key {
        if modifiers.command() && modifiers.control() && c.as_str() == "f" {
            return Some(Message::ToggleFullscreen(window::Mode::Fullscreen));
        } else if primary && modifiers.shift() {
            match c.as_str() {
                "v" | "V" => return Some(Message::PreviewMarkdown),
                "f" | "F" => return Some(Message::ToggleFuzzyFinder),
                "p" | "P" => return Some(Message::ToggleCommandPalette),
                "s" | "S" => return Some(Message::ToggleSettings),
                "o" | "O" => return Some(Message::OpenFolderDialog),
                "t" | "T" => return Some(Message::ReopenClosedTab),
                _ => {},
            }
        } else if primary {
            match c.as_str() {
                "b" | "r" => return Some(Message::ToggleSidebar),
                "o" | "O" => return Some(Message::OpenFileOrFolder),
                "w" | "W" => return Some(Message::CloseActiveTab),
                "s" | "S" => return Some(Message::SaveFile),
                "t" | "T" => return Some(Message::ToggleFileFinder),
                "j" | "J" => return Some(Message::ToggleTerminal),
                "f" | "F" => return Some(Message::ToggleFindReplace),
                "n" | "N" => return Some(Message::NewFile),
                "1" => return Some(Message::SelectTabByIndex(0)),
                "2" => return Some(Message::SelectTabByIndex(1)),
                "3" => return Some(Message::SelectTabByIndex(2)),
                "4" => return Some(Message::SelectTabByIndex(3)),
                "5" => return Some(Message::SelectTabByIndex(4)),
                "6" => return Some(Message::SelectTabByIndex(5)),
                "7" => return Some(Message::SelectTabByIndex(6)),
                "8" => return Some(Message::SelectTabByIndex(7)),
                "9" => return Some(Message::SelectTabByIndex(8)),
                _ => {},
            }
        }
    }

    if !modifiers.command() && !modifiers.control() {
        if let Key::Named(iced::keyboard::key::Named::Escape) = key {
            return Some(Message::EscapePressed);
        }
        if let Key::Named(iced::keyboard::key::Named::F12) = key {
            return Some(Message::GoToDefinition);
        }
    }

    None
}

/// Emits the current modifier key state whenever it changes or a key is pressed or released.
pub fn modifier_state() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
            Some(Message::ModifierStateChanged(modifiers))
        },
        Event::Keyboard(iced::keyboard::Event::KeyPressed { modifiers, .. }) => {
            Some(Message::ModifierStateChanged(modifiers))
        },
        Event::Keyboard(iced::keyboard::Event::KeyReleased { modifiers, .. }) => {
            Some(Message::ModifierStateChanged(modifiers))
        },

        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::key::Named;
    use iced::keyboard::Modifiers;

    fn ch(c: &str) -> Key {
        Key::Character(c.into())
    }

    #[test]
    fn test_startup_key_for_enter_continues() {
        assert!(matches!(
            startup_key_for(&Key::Named(Named::Enter)),
            Some(Message::StartupPageDone)
        ));
    }

    #[test]
    fn test_startup_key_for_other_keys_is_ignored() {
        assert!(startup_key_for(&ch("j")).is_none());
        assert!(startup_key_for(&Key::Named(Named::Escape)).is_none());
    }

    #[test]
    fn test_shortcut_for_ctrl_w_in_terminal_is_ignored() {
        assert!(shortcut_for(&ch("w"), Modifiers::CTRL, true).is_none());
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_shortcut_for_ctrl_j_in_terminal_toggles_terminal() {
        assert!(matches!(
            shortcut_for(&ch("j"), Modifiers::CTRL, true),
            Some(Message::ToggleTerminal)
        ));
    }

    #[test]
    fn test_shortcut_for_ctrl_shift_v_in_terminal_is_ignored() {
        assert!(shortcut_for(&ch("V"), Modifiers::CTRL | Modifiers::SHIFT, true).is_none());
        assert!(shortcut_for(&ch("v"), Modifiers::CTRL | Modifiers::SHIFT, true).is_none());
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_shortcut_for_ctrl_shift_p_in_terminal_opens_palette() {
        assert!(matches!(
            shortcut_for(&ch("P"), Modifiers::CTRL | Modifiers::SHIFT, true),
            Some(Message::ToggleCommandPalette)
        ));
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_shortcut_for_ctrl_w_in_editor_closes_tab() {
        assert!(matches!(
            shortcut_for(&ch("w"), Modifiers::CTRL, false),
            Some(Message::CloseActiveTab)
        ));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_shortcut_for_ctrl_w_on_macos_is_ignored() {
        assert!(shortcut_for(&ch("w"), Modifiers::CTRL, false).is_none());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_shortcut_for_cmd_w_on_macos_closes_tab() {
        assert!(matches!(
            shortcut_for(&ch("w"), Modifiers::LOGO, true),
            Some(Message::CloseActiveTab)
        ));
        assert!(matches!(
            shortcut_for(&ch("w"), Modifiers::LOGO, false),
            Some(Message::CloseActiveTab)
        ));
    }

    #[test]
    fn test_shortcut_for_escape_in_terminal_still_works() {
        assert!(matches!(
            shortcut_for(&Key::Named(Named::Escape), Modifiers::empty(), true),
            Some(Message::EscapePressed)
        ));
    }
}
