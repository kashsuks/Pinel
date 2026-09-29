use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Element, Length,
};

use crate::{
    features::{
        chat::ChatSession,
        chat_providers::{self, PROVIDERS},
    },
    message::Message,
    theme::*,
    ui::styles::{sidebar_container_style, tab_button_style, tree_button_style},
};

const ROW_PADDING: iced::Padding = iced::Padding {
    top: 6.0,
    right: 10.0,
    bottom: 6.0,
    left: 10.0,
};

/// Renders the AI chat sidebar panel.
///
/// Shows the chat history list (with a "New Chat" button) when no session
/// is active, or that session's conversation view - currently just the
/// provider/model picker - once one is selected. Actually sending messages
/// to a provider lands in a follow-up commit.
pub fn view_chat_panel<'a>(
    sessions: &'a [ChatSession],
    active_session: Option<&'a str>,
    provider_dropdown_open: bool,
    model_dropdown_open: bool,
    width: f32,
) -> Element<'a, Message> {
    match active_session.and_then(|id| sessions.iter().find(|s| s.id == id)) {
        Some(session) => {
            view_chat_conversation(session, provider_dropdown_open, model_dropdown_open, width)
        },
        None => view_chat_history(sessions, active_session, width),
    }
}

fn view_chat_history<'a>(
    sessions: &'a [ChatSession],
    active_session: Option<&'a str>,
    width: f32,
) -> Element<'a, Message> {
    let new_chat_button = button(text("+ New Chat").size(13))
        .style(tree_button_style)
        .on_press(Message::ChatNewSession)
        .padding(ROW_PADDING)
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
                    .padding(ROW_PADDING)
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

fn view_chat_conversation<'a>(
    session: &'a ChatSession,
    provider_dropdown_open: bool,
    model_dropdown_open: bool,
    width: f32,
) -> Element<'a, Message> {
    let header = row![
        button(text("< Chats").size(12).color(theme().text_muted))
            .style(tree_button_style)
            .on_press(Message::ChatBackToHistory)
            .padding(iced::Padding {
                top: 4.0,
                right: 6.0,
                bottom: 4.0,
                left: 6.0,
            }),
        text(session.title.as_str()).size(13).color(theme().text_primary),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    let placeholder =
        container(text("Message thread coming soon").size(12).color(theme().text_placeholder))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

    let content = column![
        header,
        provider_field(session, provider_dropdown_open),
        model_field(session, model_dropdown_open),
        placeholder,
    ]
    .spacing(10)
    .height(Length::Fill);

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

/// A trigger button that expands into a list of options when open. Shared
/// shape for both the provider and model pickers.
fn dropdown<'a>(
    trigger_label: String,
    is_open: bool,
    toggle: Message,
    options: Vec<(String, bool, Message)>,
) -> Element<'a, Message> {
    let trigger = button(
        row![
            text(trigger_label).size(12).color(theme().text_primary),
            Space::new().width(Length::Fill),
            text(if is_open {
                "^"
            } else {
                "v"
            })
            .size(10)
            .color(theme().text_dim),
        ]
        .align_y(iced::Alignment::Center),
    )
    .style(tree_button_style)
    .on_press(toggle)
    .padding(ROW_PADDING)
    .width(Length::Fill);

    let mut items: Vec<Element<'a, Message>> = vec![trigger.into()];
    if is_open {
        for (label, is_active, message) in options {
            items.push(
                button(text(label).size(12))
                    .style(tab_button_style(is_active))
                    .on_press(message)
                    .padding(ROW_PADDING)
                    .width(Length::Fill)
                    .into(),
            );
        }
    }

    column(items).spacing(2).into()
}

fn provider_field<'a>(session: &'a ChatSession, is_open: bool) -> Element<'a, Message> {
    let label = if session.provider.is_empty() {
        "Select provider".to_string()
    } else {
        session.provider.clone()
    };

    let options = PROVIDERS
        .iter()
        .map(|provider| {
            (
                provider.name.to_string(),
                session.provider == provider.name,
                Message::ChatProviderSelected(provider.name.to_string()),
            )
        })
        .collect();

    dropdown(label, is_open, Message::ChatToggleProviderDropdown, options)
}

fn model_field<'a>(session: &'a ChatSession, is_open: bool) -> Element<'a, Message> {
    let label = if session.model.is_empty() {
        "Select model".to_string()
    } else {
        session.model.clone()
    };

    let models = chat_providers::models_for(&session.provider);
    let options = models
        .iter()
        .map(|model| {
            (
                model.to_string(),
                session.model == *model,
                Message::ChatModelSelected(model.to_string()),
            )
        })
        .collect();

    dropdown(label, is_open, Message::ChatToggleModelDropdown, options)
}
