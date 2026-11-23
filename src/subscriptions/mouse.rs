//! Mouse event subscription handlers.

use iced::{Event, Subscription};

use crate::message::Message;

/// Tracks the raw cursor position at all times (not gated on any drag/resize state)
/// TO-DO: in the future please ensure that such high priority
/// data transfers happen with high rates and uptime
pub fn cursor_tracker() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::CursorPositionTracked(position.x, position.y))
        },
        _ => None,
    })
}

/// Emits sidebar resize messages from mouse events.
pub fn sidebar_resize() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::SidebarResizing(position.x))
        },
        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::SidebarResizeEnd)
        },
        _ => None,
    })
}

/// Finishes or cancels a tab gesture even when the pointer leaves the strip.
pub fn tab_drag() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::TabDragEnd)
        },
        Event::Window(iced::window::Event::Unfocused) => Some(Message::TabDragCancelled),
        _ => None,
    })
}
