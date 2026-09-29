use iced::{
    widget::{column, container, scrollable, text},
    Element, Length,
};

use crate::{message::Message, theme::*, ui::styles::sidebar_container_style};

/// Renders the AI chat sidebar panel.
///
/// This is currently a placeholder empty state - conversation history,
/// the new chat button, and the provider/model picker land in follow-up
/// commits.
pub fn view_chat_panel<'a>(width: f32) -> Element<'a, Message> {
    let content: Element<'a, Message> = container(
        column![
            text("No chats yet").size(13).color(theme().text_muted),
            text("Start a new chat to begin").size(11).color(theme().text_placeholder),
        ]
        .spacing(4)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into();

    container(scrollable(content).height(Length::Fill))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 14.0,
            right: 2.0,
            bottom: 2.0,
            left: 4.0,
        })
        .style(sidebar_container_style)
        .into()
}
