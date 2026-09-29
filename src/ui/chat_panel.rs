use iced::{
    widget::{button, column, container, scrollable, text},
    Element, Length,
};

use crate::{
    features::chat::ChatSession,
    message::Message,
    theme::*,
    ui::styles::{sidebar_container_style, tab_button_style, tree_button_style},
};

/// Renders the AI chat sidebar panel: a "New Chat" button plus the list of
/// past conversations for the current workspace.
///
/// Picking a provider/model and actually sending messages to one land in
/// follow-up commits - this only handles starting and switching chats.
pub fn view_chat_panel<'a>(
    sessions: &'a [ChatSession],
    active_session: Option<&'a str>,
    width: f32,
) -> Element<'a, Message> {
    let new_chat_button = button(text("+ New Chat").size(13))
        .style(tree_button_style)
        .on_press(Message::ChatNewSession)
        .padding(iced::Padding {
            top: 6.0,
            right: 10.0,
            bottom: 6.0,
            left: 10.0,
        })
        .width(Length::Fill);

    let history: Element<'a, Message> = if sessions.is_empty() {
        container(
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
        .into()
    } else {
        let items: Vec<Element<'a, Message>> = sessions
            .iter()
            .map(|session| {
                let is_active = active_session == Some(session.id.as_str());
                button(text(session.title.as_str()).size(13))
                    .style(tab_button_style(is_active))
                    .on_press(Message::ChatSelectSession(session.id.clone()))
                    .padding(iced::Padding {
                        top: 6.0,
                        right: 10.0,
                        bottom: 6.0,
                        left: 10.0,
                    })
                    .width(Length::Fill)
                    .into()
            })
            .collect();

        scrollable(column(items).spacing(2)).height(Length::Fill).into()
    };

    let content = column![new_chat_button, history].spacing(8).height(Length::Fill);

    container(content)
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
