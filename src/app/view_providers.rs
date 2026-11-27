use iced::widget::{column, Space};

use crate::features::ai_client::ModelFetchState;
use crate::features::chat_providers::PROVIDERS;

use super::*;

impl App {
    pub(super) fn view_settings_providers(&self) -> Element<'_, Message> {
        let heading = text("Providers").size(18).color(theme().text_primary);
        let desc = text(
            "Attach an API key for hosted providers, or point a local one (Ollama, LM Studio, ...) at your own server.",
        )
        .size(TEXT_SM)
        .color(theme().text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let provider_tabs: Vec<Element<'_, Message>> = PROVIDERS
            .iter()
            .map(|provider| {
                let is_active = provider.id == self.providers_selected;
                let is_saved = self.provider_api_key(provider.id).is_some();
                let label = if is_saved {
                    format!("{} ●", provider.name)
                } else {
                    provider.name.to_string()
                };
                button(text(label).size(TEXT_MD))
                    .style(tab_button_style(is_active))
                    .on_press(Message::ProvidersSelect(provider.id.to_string()))
                    .padding(iced::Padding {
                        top: SPACE_2,
                        right: SPACE_3,
                        bottom: SPACE_2,
                        left: SPACE_3,
                    })
                    .into()
            })
            .collect();

        let selected = crate::features::chat_providers::by_id(&self.providers_selected);
        let is_local = selected.map(|p| !p.requires_api_key).unwrap_or(false);

        let key_label = text(if is_local {
            "API Key (optional)"
        } else {
            "API Key"
        })
        .size(TEXT_XS)
        .color(theme().text_dim);

        let key_field = text_input("sk-...", &self.providers_key_input)
            .on_input(Message::ProvidersApiKeyChanged)
            .secure(!self.providers_key_visible)
            .size(TEXT_MD)
            .padding(iced::Padding {
                top: SPACE_2,
                right: SPACE_3,
                bottom: SPACE_2,
                left: SPACE_3,
            })
            .style(search_input_style)
            .width(Length::Fill);

        let visibility_btn = button(
            text(if self.providers_key_visible {
                "Hide"
            } else {
                "Show"
            })
            .size(TEXT_MD),
        )
        .style(tree_button_style)
        .on_press(Message::ProvidersToggleKeyVisibility)
        .padding(iced::Padding {
            top: SPACE_2,
            right: SPACE_3,
            bottom: SPACE_2,
            left: SPACE_3,
        });

        let key_row = row![key_field, visibility_btn]
            .spacing(SPACE_2)
            .align_y(iced::Alignment::Center);

        // Local providers (Ollama, LM Studio, Custom) run on a host/port
        // the user controls, so their URL is editable; hosted providers
        // have a fixed, well-known endpoint shown as a read-only hint.
        let server_url_section: Element<'_, Message> = if is_local {
            let url_field = text_input(
                selected.map(|p| p.base_url).unwrap_or_default(),
                &self.providers_base_url_input,
            )
            .on_input(Message::ProvidersBaseUrlChanged)
            .size(TEXT_MD)
            .padding(iced::Padding {
                top: SPACE_2,
                right: SPACE_3,
                bottom: SPACE_2,
                left: SPACE_3,
            })
            .style(search_input_style)
            .width(Length::Fill);

            column![
                text("Server URL").size(TEXT_XS).color(theme().text_dim),
                url_field,
            ]
            .spacing(SPACE_1)
            .into()
        } else {
            text(selected.map(|p| p.base_url).unwrap_or_default())
                .size(TEXT_XS)
                .color(theme().text_dim)
                .into()
        };

        let save_btn = button(text("Save").size(TEXT_MD).color(theme().text_primary))
            .on_press(Message::ProvidersSave)
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(ACCENT_PURPLE.scale_alpha(0.2))),
                border: iced::Border {
                    color: ACCENT_PURPLE.scale_alpha(0.4),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: theme().text_primary,
                ..Default::default()
            })
            .padding(iced::Padding {
                top: SPACE_2,
                right: 20.0,
                bottom: SPACE_2,
                left: 20.0,
            });

        let test_btn = button(text("Test Connection").size(TEXT_MD))
            .style(tree_button_style)
            .on_press(Message::ProvidersTestConnection)
            .padding(iced::Padding {
                top: SPACE_2,
                right: SPACE_4,
                bottom: SPACE_2,
                left: SPACE_4,
            });

        let remove_btn =
            button(text("Remove").size(TEXT_MD).color(Color::from_rgb(0.86, 0.35, 0.35)))
                .style(tree_button_style)
                .on_press(Message::ProvidersRemove(self.providers_selected.clone()))
                .padding(iced::Padding {
                    top: SPACE_2,
                    right: SPACE_4,
                    bottom: SPACE_2,
                    left: SPACE_4,
                });

        let actions_row = row![save_btn, test_btn, remove_btn].spacing(SPACE_2);

        let status: Element<'_, Message> = match self.provider_connection_test.as_ref() {
            None => Space::new().height(Length::Fixed(0.0)).into(),
            Some(ModelFetchState::Loading(_)) => {
                text("Testing connection...").size(TEXT_SM).color(theme().text_dim).into()
            },
            Some(ModelFetchState::Loaded(models)) => text(format!(
                "Connected — {} model{} available",
                models.len(),
                if models.len() == 1 {
                    ""
                } else {
                    "s"
                }
            ))
            .size(TEXT_SM)
            .color(Color::from_rgb(0.4, 0.75, 0.45))
            .into(),
            Some(ModelFetchState::Error(error)) => text(error.as_str())
                .size(TEXT_SM)
                .color(Color::from_rgb(0.86, 0.35, 0.35))
                .into(),
        };

        column![
            heading,
            desc,
            separator,
            row(provider_tabs).spacing(SPACE_1),
            server_url_section,
            key_label,
            key_row,
            actions_row,
            status,
        ]
        .spacing(SPACE_3)
        .width(Length::Fill)
        .into()
    }
}
