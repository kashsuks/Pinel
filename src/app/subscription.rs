use super::*;
use std::time::Duration;

impl App {
    /// Registers global event listeners and maps them to [`Message`] values.
    pub fn subscription(&self) -> Subscription<Message> {
       let mut subs = vec![
           crate::subscriptions::keyboard::shortcuts(),
           crate::subscriptions::mouse::sidebar_resize(),
           iced::time::every(Duration::from_millis(150)).map(|_| Message::LspTick),
       ];

       if let Some(term) = &self.terminal_pane {
           subs.push(term.subscription().map(Message::TerminalEvent));
       }

       Subscription::batch(subs)
    }
}
