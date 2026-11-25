use iced::widget::column;

use super::*;

impl App {
    #[allow(dead_code)]
    pub(super) fn view_settings_theme(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("Theme").size(18).color(theme().text_primary);
        let desc = text("Theme options have been moved to Preferences.")
            .size(TEXT_SM)
            .color(theme().text_muted);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let hint_btn = button(text("Go to Preferences →").size(TEXT_MD).color(ACCENT_PURPLE))
            .on_press(Message::SettingsNavigate("preferences".to_string()))
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(ACCENT_PURPLE.scale_alpha(0.10))),
                border: iced::Border {
                    color: ACCENT_PURPLE.scale_alpha(0.25),
                    width: 1.0,
                    radius: RADIUS_SM.into(),
                },
                text_color: ACCENT_PURPLE,
                ..Default::default()
            })
            .padding(iced::Padding {
                top: 10.0,
                right: 18.0,
                bottom: 10.0,
                left: 18.0,
            });

        let reload_desc = text("Or reload a custom theme.lua from ~/.config/pinel/")
            .size(TEXT_XS)
            .color(theme().text_muted);

        let reload_btn =
            button(text("↻  Reload theme.lua").size(TEXT_MD).color(theme().text_primary))
                .on_press(Message::SettingsReloadTheme)
                .style(|_theme, _status| button::Style {
                    background: Some(Background::Color(theme().bg_secondary)),
                    border: iced::Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                        width: 1.0,
                        radius: RADIUS_SM.into(),
                    },
                    text_color: theme().text_primary,
                    ..Default::default()
                })
                .padding(iced::Padding {
                    top: SPACE_2,
                    right: SPACE_4,
                    bottom: SPACE_2,
                    left: SPACE_4,
                });

        column![
            heading,
            desc,
            separator,
            hint_btn,
            Space::new().height(Length::Fixed(8.0)),
            reload_desc,
            reload_btn
        ]
        .spacing(SPACE_3)
        .width(Length::Fill)
        .into()
    }

    pub(super) fn view_settings_wakatime(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("WakaTime").size(18).color(theme().text_primary);
        let desc = text("Configure WakaTime integration for activity tracking.")
            .size(TEXT_SM)
            .color(theme().text_muted);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let field_height = 34.0;
        let has_key = !self.wakatime.api_key.is_empty();

        let key_input = text_input(
            "waka_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            &self.wakatime.api_key,
        )
        .on_input(Message::WakaTimeApiKeyChanged)
        .secure(!self.wakatime_key_visible)
        .size(TEXT_MD)
        .padding(iced::Padding {
            top: SPACE_2,
            right: SPACE_2,
            bottom: SPACE_2,
            left: SPACE_3,
        })
        .style(search_input_style)
        .width(Length::Fill);

        let action_padding = iced::Padding {
            top: SPACE_1,
            right: SPACE_2,
            bottom: SPACE_1,
            left: SPACE_2,
        };

        let visibility_btn = button(
            text(if self.wakatime_key_visible {
                "Hide"
            } else {
                "Show"
            })
            .size(TEXT_SM),
        )
        .style(tree_button_style)
        .on_press(Message::WakaTimeToggleKeyVisibility)
        .padding(action_padding);

        let mut actions = row![visibility_btn].spacing(SPACE_1).align_y(iced::Alignment::Center);

        if has_key {
            let copy_label = if self.wakatime_key_copied {
                text("Copied").size(TEXT_SM).color(ACCENT_GREEN)
            } else {
                text("Copy").size(TEXT_SM)
            };
            actions = actions.push(
                button(copy_label)
                    .style(tree_button_style)
                    .on_press(Message::WakaTimeCopyKey)
                    .padding(action_padding),
            );
        }

        let api_key_field = container(row![key_input, actions].align_y(iced::Alignment::Center))
            .padding(iced::Padding {
                top: 0.0,
                right: SPACE_1,
                bottom: 0.0,
                left: 0.0,
            })
            .style(settings_field_style)
            .height(Length::Fixed(field_height))
            .width(Length::Fill)
            .align_y(iced::Alignment::Center);

        let api_key_row = row![
            column![
                text("API Key").size(TEXT_MD).color(theme().text_muted),
                text("Your WakaTime API key for authentication")
                    .size(TEXT_XS)
                    .color(theme().text_muted),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            container(api_key_field).width(Length::FillPortion(3)),
        ]
        .spacing(SPACE_4)
        .align_y(iced::Alignment::Center);

        let api_url_row = row![
            column![
                text("API URL").size(TEXT_MD).color(theme().text_muted),
                text("WakaTime API endpoint URL").size(TEXT_XS).color(theme().text_muted),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            container(
                text_input("https://api.wakatime.com/api/v1", &self.wakatime.api_url)
                    .on_input(Message::WakaTimeApiUrlChanged)
                    .size(TEXT_MD)
                    .padding(iced::Padding {
                        top: SPACE_2,
                        right: SPACE_3,
                        bottom: SPACE_2,
                        left: SPACE_3
                    })
                    .style(search_input_style)
                    .width(Length::Fill),
            )
            .style(settings_field_style)
            .height(Length::Fixed(field_height))
            .width(Length::FillPortion(3))
            .align_y(iced::Alignment::Center),
        ]
        .spacing(SPACE_4)
        .align_y(iced::Alignment::Center);

        let save_btn =
            button(text("Save WakaTime Settings").size(TEXT_MD).color(theme().text_primary))
                .on_press(Message::SaveWakaTimeSettings)
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

        column![
            heading,
            desc,
            separator,
            api_key_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            api_url_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            Space::new().height(Length::Fixed(8.0)),
            save_btn,
        ]
        .spacing(SPACE_3)
        .width(Length::Fill)
        .into()
    }
}

/// Theme-aware box for settings inputs, so fields follow the active theme instead of fixed accents.
fn settings_field_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme().bg_secondary)),
        border: iced::Border {
            color: theme().border_subtle,
            width: 1.0,
            radius: RADIUS_SM.into(),
        },
        ..Default::default()
    }
}
