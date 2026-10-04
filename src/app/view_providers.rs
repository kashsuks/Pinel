use iced::widget::{column, Space};

use crate::features::ai_client::ModelFetchState;
use crate::features::chat_providers::PROVIDERS;

use super::*;

impl App {
    pub(super) fn view_settings_providers(&self) -> Element<'_, Message> {
        let heading = text("Providers").size(18).color(theme().text_primary);
        let desc = text("Attach an API key for each provider you want to use from the chat tab.")
            .size(12)
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
                let has_key = self.provider_api_key(provider.id).is_some();
                let label = if has_key {
                    format!("{} ●", provider.name)
                } else {
                    provider.name.to_string()
                };
                button(text(label).size(12))
                    .style(tab_button_style(is_active))
                    .on_press(Message::ProvidersSelect(provider.id.to_string()))
                    .padding(iced::Padding {
                        top: 8.0,
                        right: 12.0,
                        bottom: 8.0,
                        left: 12.0,
                    })
                    .into()
            })
            .collect();

        let selected = crate::features::chat_providers::by_id(&self.providers_selected);

        let key_field = text_input("sk-...", &self.providers_key_input)
            .on_input(Message::ProvidersApiKeyChanged)
            .secure(!self.providers_key_visible)
            .size(13)
            .padding(iced::Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0,
            })
            .style(search_input_style)
            .width(Length::Fill);

        let visibility_btn = button(
            text(if self.providers_key_visible {
                "Hide"
            } else {
                "Show"
            })
            .size(12),
        )
        .style(tree_button_style)
        .on_press(Message::ProvidersToggleKeyVisibility)
        .padding(iced::Padding {
            top: 8.0,
            right: 12.0,
            bottom: 8.0,
            left: 12.0,
        });

        let key_row = row![key_field, visibility_btn].spacing(8).align_y(iced::Alignment::Center);

        let base_url_hint = text(selected.map(|p| p.base_url).unwrap_or_default())
            .size(11)
            .color(theme().text_dim);

        let save_btn = button(text("Save").size(12).color(theme().text_primary))
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
                top: 8.0,
                right: 20.0,
                bottom: 8.0,
                left: 20.0,
            });

        let test_btn = button(text("Test Connection").size(12))
            .style(tree_button_style)
            .on_press(Message::ProvidersTestConnection(
                self.providers_selected.clone(),
            ))
            .padding(iced::Padding {
                top: 8.0,
                right: 16.0,
                bottom: 8.0,
                left: 16.0,
            });

        let remove_btn = button(text("Remove").size(12).color(Color::from_rgb(0.86, 0.35, 0.35)))
            .style(tree_button_style)
            .on_press(Message::ProvidersRemove(self.providers_selected.clone()))
            .padding(iced::Padding {
                top: 8.0,
                right: 16.0,
                bottom: 8.0,
                left: 16.0,
            });

        let actions_row = row![save_btn, test_btn, remove_btn].spacing(8);

        let status: Element<'_, Message> =
            match self.provider_model_state.get(&self.providers_selected) {
                None => Space::new().height(Length::Fixed(0.0)).into(),
                Some(ModelFetchState::Loading) => {
                    text("Testing connection...").size(12).color(theme().text_dim).into()
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
                .size(12)
                .color(Color::from_rgb(0.4, 0.75, 0.45))
                .into(),
                Some(ModelFetchState::Error(error)) => text(format!("Connection failed: {error}"))
                    .size(12)
                    .color(Color::from_rgb(0.86, 0.35, 0.35))
                    .into(),
            };

        column![
            heading,
            desc,
            separator,
            row(provider_tabs).spacing(4),
            base_url_hint,
            key_row,
            actions_row,
            status,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }
}
