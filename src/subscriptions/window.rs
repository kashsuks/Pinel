//! Window event subscription handlers.

use crate::message::Message;
use iced::{window, Event, Subscription};

/// Emits window resize messages to persist size preferences.
pub fn resizes() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Window(window::Event::Resized(size)) => Some(Message::WindowResized(
            size.width.max(0.0) as u32,
            size.height.max(0.0) as u32,
        )),
        _ => None,
    })
}
