use std::collections::HashMap;

use iced::{
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Color, Element, Length,
};

use crate::{
    features::{
        ai_client::ModelFetchState,
        chat::{ChatMessage, ChatRole, ChatSession},
        chat_providers::{self, PROVIDERS},
    },
    message::Message,
    theme::*,
    ui::styles::{
        chat_message_bubble_style, context_menu_panel_style, rename_input_style,
        sidebar_container_style, tab_button_style, tree_button_style,
    },
};

const ROW_ACTION_PADDING: iced::Padding = iced::Padding {
    top: 4.0,
    right: 6.0,
    bottom: 4.0,
    left: 6.0,
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
/// is active, or that session's conversation view once one is selected.
/// Actually sending messages to a provider lands in a follow-up commit.
// Each argument is a distinct, independently-changing piece of view state
// (session list, active id, model picker state, input text, and rename
// state) rather than something that naturally groups into a struct yet.
#[allow(clippy::too_many_arguments)]
pub fn view_chat_panel<'a>(
    sessions: &'a [ChatSession],
    active_session: Option<&'a str>,
    model_picker_open: bool,
    picker_provider: Option<&'a str>,
    model_search: &'a str,
    provider_model_state: &'a HashMap<String, ModelFetchState>,
    input_value: &'a str,
    sending: bool,
    rename_target: Option<&'a str>,
    rename_input: &'a str,
    rename_input_id: iced::widget::Id,
    width: f32,
) -> Element<'a, Message> {
    match active_session.and_then(|id| sessions.iter().find(|s| s.id == id)) {
        Some(session) => view_chat_conversation(
            session,
            model_picker_open,
            picker_provider,
            model_search,
            provider_model_state,
            input_value,
            sending,
            width,
        ),
        None => view_chat_history(
            sessions,
            active_session,
            rename_target,
            rename_input,
            rename_input_id,
            width,
        ),
    }
}

fn view_chat_history<'a>(
    sessions: &'a [ChatSession],
    active_session: Option<&'a str>,
    rename_target: Option<&'a str>,
    rename_input: &'a str,
    rename_input_id: iced::widget::Id,
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
                if rename_target == Some(session.id.as_str()) {
                    render_rename_row(rename_input, rename_input_id.clone())
                } else {
                    render_history_row(session, active_session == Some(session.id.as_str()))
                }
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

fn render_history_row(session: &ChatSession, is_active: bool) -> Element<'_, Message> {
    row![
        button(text(session.title.as_str()).size(13))
            .style(tab_button_style(is_active))
            .on_press(Message::ChatSelectSession(session.id.clone()))
            .padding(ROW_PADDING)
            .width(Length::Fill),
        button(text("Rename").size(10).color(theme().text_dim))
            .style(tree_button_style)
            .on_press(Message::ChatRenameStart(session.id.clone()))
            .padding(ROW_ACTION_PADDING),
        button(text("Delete").size(10).color(Color::from_rgb(0.86, 0.35, 0.35)))
            .style(tree_button_style)
            .on_press(Message::ChatDeleteSession(session.id.clone()))
            .padding(ROW_ACTION_PADDING),
    ]
    .spacing(2)
    .align_y(iced::Alignment::Center)
    .into()
}

fn render_rename_row<'a>(
    rename_input: &'a str,
    rename_input_id: iced::widget::Id,
) -> Element<'a, Message> {
    row![
        text_input("Chat name", rename_input)
            .id(rename_input_id)
            .on_input(Message::ChatRenameInputChanged)
            .on_submit(Message::ChatRenameSubmit)
            .size(13)
            .padding(iced::Padding {
                top: 4.0,
                right: 6.0,
                bottom: 4.0,
                left: 6.0,
            })
            .style(rename_input_style)
            .width(Length::Fill),
        button(text("Cancel").size(10).color(theme().text_dim))
            .style(tree_button_style)
            .on_press(Message::ChatRenameCancel)
            .padding(ROW_ACTION_PADDING),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center)
    .into()
}

#[allow(clippy::too_many_arguments)]
fn view_chat_conversation<'a>(
    session: &'a ChatSession,
    model_picker_open: bool,
    picker_provider: Option<&'a str>,
    model_search: &'a str,
    provider_model_state: &'a HashMap<String, ModelFetchState>,
    input_value: &'a str,
    sending: bool,
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

    let thread = message_thread(session, sending);

    let model_trigger = model_trigger_button(session, model_picker_open);

    let input_row = row![
        text_input("Message", input_value)
            .on_input(Message::ChatInputChanged)
            .on_submit(Message::ChatSend)
            .size(13)
            .padding(iced::Padding {
                top: 6.0,
                right: 8.0,
                bottom: 6.0,
                left: 8.0,
            })
            .style(rename_input_style)
            .width(Length::Fill),
        button(
            text(if sending {
                "Sending..."
            } else {
                "Send"
            })
            .size(12)
        )
        .style(tree_button_style)
        .on_press_maybe((!sending).then_some(Message::ChatSend))
        .padding(iced::Padding {
            top: 6.0,
            right: 10.0,
            bottom: 6.0,
            left: 10.0,
        }),
    ]
    .spacing(6)
    .align_y(iced::Alignment::Center);

    // The trigger sits directly above the input box; the picker (when
    // open) pops up directly above the trigger, anchored to the bottom of
    // the panel rather than the top.
    let mut items: Vec<Element<'a, Message>> = vec![header.into(), thread];
    if model_picker_open {
        let browse_provider = picker_provider
            .filter(|p| !p.is_empty())
            .or(Some(session.provider.as_str()).filter(|p| !p.is_empty()))
            .unwrap_or(PROVIDERS[0].name);
        items.push(model_picker(
            browse_provider,
            model_search,
            provider_model_state,
        ));
    }
    items.push(model_trigger);
    items.push(input_row.into());

    let content = column(items).spacing(10).height(Length::Fill);

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

/// Renders the session's messages as a scrollable list of bubbles.
///
/// Only user messages exist for now - sending a message just appends it
/// locally, since no provider is wired up yet.
fn message_thread<'a>(session: &'a ChatSession, sending: bool) -> Element<'a, Message> {
    if session.messages.is_empty() && !sending {
        return container(
            text("Send a message to start the conversation")
                .size(12)
                .color(theme().text_placeholder),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    let mut bubbles: Vec<Element<'a, Message>> =
        session.messages.iter().map(message_bubble).collect();
    if sending {
        bubbles.push(
            container(text("Assistant is typing...").size(12).color(theme().text_placeholder))
                .padding(iced::Padding {
                    top: 6.0,
                    right: 8.0,
                    bottom: 6.0,
                    left: 8.0,
                })
                .into(),
        );
    }

    scrollable(column(bubbles).spacing(6)).height(Length::Fill).into()
}

fn message_bubble(message: &ChatMessage) -> Element<'_, Message> {
    let role_label = match message.role {
        ChatRole::User => "You",
        ChatRole::Assistant => "Assistant",
    };

    container(
        column![
            text(role_label).size(10).color(theme().text_dim),
            text(message.content.as_str()).size(13).color(theme().text_primary),
        ]
        .spacing(2),
    )
    .width(Length::Fill)
    .padding(iced::Padding {
        top: 6.0,
        right: 8.0,
        bottom: 6.0,
        left: 8.0,
    })
    .style(chat_message_bubble_style)
    .into()
}

/// The single "provider/model" field shown right above the message input.
/// Clicking it opens [`model_picker`] above itself.
fn model_trigger_button(session: &ChatSession, is_open: bool) -> Element<'_, Message> {
    let label = if session.provider.is_empty() || session.model.is_empty() {
        "Select model".to_string()
    } else {
        format!("{}/{}", session.provider, session.model)
    };

    button(
        row![
            text(label).size(12).color(theme().text_primary),
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
    .on_press(Message::ChatToggleModelPicker)
    .padding(ROW_PADDING)
    .width(Length::Fill)
    .into()
}

/// The expanded picker: a narrow column of provider badges on the left,
/// and a searchable list of that provider's models on the right - picking
/// a model commits both the provider and model to the session at once.
fn model_picker<'a>(
    browse_provider: &'a str,
    search: &'a str,
    provider_model_state: &'a HashMap<String, ModelFetchState>,
) -> Element<'a, Message> {
    let provider_column: Vec<Element<'a, Message>> = PROVIDERS
        .iter()
        .map(|provider| {
            let is_active = provider.name == browse_provider;
            button(text(provider.badge).size(11))
                .style(tab_button_style(is_active))
                .on_press(Message::ChatPickerProviderSelected(
                    provider.name.to_string(),
                ))
                .padding(iced::Padding {
                    top: 8.0,
                    right: 4.0,
                    bottom: 8.0,
                    left: 4.0,
                })
                .width(Length::Fixed(32.0))
                .into()
        })
        .collect();

    let query = search.to_lowercase();
    let state =
        chat_providers::by_name(browse_provider).and_then(|p| provider_model_state.get(p.id));

    let model_list: Element<'a, Message> = match state {
        None => container(
            text("No API key attached. Add one in Settings → Providers.")
                .size(11)
                .color(theme().text_placeholder),
        )
        .padding(ROW_PADDING)
        .into(),
        Some(ModelFetchState::Loading) => {
            container(text("Loading models...").size(11).color(theme().text_placeholder))
                .padding(ROW_PADDING)
                .into()
        },
        Some(ModelFetchState::Error(message)) => container(
            text(format!("Failed to load models: {message}"))
                .size(11)
                .color(theme().text_placeholder),
        )
        .padding(ROW_PADDING)
        .into(),
        Some(ModelFetchState::Loaded(models)) => {
            let matching_models: Vec<&str> = models
                .iter()
                .map(String::as_str)
                .filter(|model| query.is_empty() || model.to_lowercase().contains(&query))
                .collect();

            if matching_models.is_empty() {
                container(text("No models found").size(11).color(theme().text_placeholder))
                    .padding(ROW_PADDING)
                    .into()
            } else {
                let provider_owned = browse_provider.to_string();
                let rows: Vec<Element<'a, Message>> = matching_models
                    .into_iter()
                    .map(|model| {
                        button(text(model).size(12))
                            .style(tree_button_style)
                            .on_press(Message::ChatModelPicked(
                                provider_owned.clone(),
                                model.to_string(),
                            ))
                            .padding(ROW_PADDING)
                            .width(Length::Fill)
                            .into()
                    })
                    .collect();
                scrollable(column(rows).spacing(2)).height(Length::Fixed(140.0)).into()
            }
        },
    };

    let search_box = text_input("Search models...", search)
        .on_input(Message::ChatModelSearchChanged)
        .size(12)
        .padding(iced::Padding {
            top: 4.0,
            right: 6.0,
            bottom: 4.0,
            left: 6.0,
        })
        .style(rename_input_style)
        .width(Length::Fill);

    let right_column = column![search_box, model_list].spacing(6).width(Length::Fill);

    let picker_body = row![column(provider_column).spacing(2), right_column].spacing(8);

    container(picker_body)
        .padding(iced::Padding {
            top: 8.0,
            right: 8.0,
            bottom: 8.0,
            left: 8.0,
        })
        .width(Length::Fill)
        .style(context_menu_panel_style)
        .into()
}
