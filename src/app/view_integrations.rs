use super::*;
use iced::widget::column;

impl App {
    pub(super) fn view_settings_theme(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("Theme").size(18).color(theme().text_primary);
        let desc = text("Theme options have been moved to Preferences.")
            .size(12)
            .color(theme().text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let hint_btn = button(text("Go to Preferences →").size(13).color(ACCENT_PURPLE))
            .on_press(Message::SettingsNavigate("preferences".to_string()))
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(ACCENT_PURPLE.scale_alpha(0.10))),
                border: iced::Border {
                    color: ACCENT_PURPLE.scale_alpha(0.25),
                    width: 1.0,
                    radius: 6.0.into(),
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
            .size(11)
            .color(theme().text_dim);

        let reload_btn = button(
            text("↻  Reload theme.lua")
                .size(12)
                .color(theme().text_primary),
        )
        .on_press(Message::SettingsReloadTheme)
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(theme().bg_secondary)),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: theme().text_primary,
            ..Default::default()
        })
        .padding(iced::Padding {
            top: 8.0,
            right: 16.0,
            bottom: 8.0,
            left: 16.0,
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
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    pub(super) fn view_settings_wakatime(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("WakaTime").size(18).color(theme().text_primary);
        let desc = text("Configure WakaTime integration for activity tracking.")
            .size(12)
            .color(theme().text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let api_key_field_height = 34.0;

        let api_key_field: Element<'_, Message> = if self.wakatime_api_key_hovered {
            text_input("waka_xxxxx", &self.wakatime.api_key)
                .on_input(Message::WakaTimeApiKeyChanged)
                .secure(false)
                .size(13)
                .padding(iced::Padding {
                    top: 8.0,
                    right: 12.0,
                    bottom: 8.0,
                    left: 12.0,
                })
                .style(search_input_style)
                .width(Length::Fill)
                .into()
        } else {
            let panel_bg = Color::from_rgba(
                theme().bg_secondary.r * 0.94 + ACCENT_BLUE.r * 0.06,
                theme().bg_secondary.g * 0.94 + ACCENT_BLUE.g * 0.06,
                theme().bg_secondary.b * 0.94 + ACCENT_BLUE.b * 0.06,
                0.96,
            );
            let top_line = Color::from_rgba(
                ACCENT_SOFT_BLUE.r,
                ACCENT_SOFT_BLUE.g,
                ACCENT_SOFT_BLUE.b,
                0.55,
            );
            let glow = Color::from_rgba(
                ACCENT_SOFT_BLUE.r,
                ACCENT_SOFT_BLUE.g,
                ACCENT_SOFT_BLUE.b,
                0.42,
            );

            let top_highlight = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(top_line)),
                    ..Default::default()
                });

            let blur_streak = container(Space::new().width(Length::Fill).height(Length::Fixed(4.0)))
                .style(move |_theme| container::Style {
                    background: Some(Background::Color(glow)),
                    border: iced::Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 999.0.into(),
                    },
                    shadow: iced::Shadow {
                        color: Color::from_rgba(
                            ACCENT_SOFT_BLUE.r,
                            ACCENT_SOFT_BLUE.g,
                            ACCENT_SOFT_BLUE.b,
                            0.55,
                        ),
                        offset: iced::Vector::new(0.0, 0.0),
                        blur_radius: 16.0,
                    },
                    ..Default::default()
                });

            container(
                column![
                    top_highlight,
                    container(blur_streak)
                        .padding(iced::Padding {
                            top: 0.0,
                            right: 16.0,
                            bottom: 0.0,
                            left: 16.0,
                        })
                ]
                .spacing(8),
            )
            .padding(iced::Padding {
                top: 0.0,
                right: 12.0,
                bottom: 9.0,
                left: 12.0,
            })
            .style(move |_theme| container::Style {
                background: Some(Background::Color(panel_bg)),
                border: iced::Border {
                    color: Color::from_rgba(ACCENT_BLUE.r, ACCENT_BLUE.g, ACCENT_BLUE.b, 0.14),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            })
            .height(Length::Fixed(api_key_field_height))
            .width(Length::Fill)
            .into()
        };

        let api_key_row = row![
            column![
                text("API Key").size(13).color(theme().text_muted),
                text("Your WakaTime API key for authentication")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            container(
                mouse_area(api_key_field)
                    .on_enter(Message::WakaTimeApiKeyHoverStart)
                    .on_exit(Message::WakaTimeApiKeyHoverEnd)
            )
            .height(Length::Fixed(api_key_field_height))
            .width(Length::FillPortion(3)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let api_url_row = row![
            column![
                text("API URL").size(13).color(theme().text_muted),
                text("WakaTime API endpoint URL")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            text_input("https://api.wakatime.com/api/v1", &self.wakatime.api_url)
                .on_input(Message::WakaTimeApiUrlChanged)
                .size(13)
                .padding(iced::Padding {
                    top: 8.0,
                    right: 12.0,
                    bottom: 8.0,
                    left: 12.0
                })
                .style(search_input_style)
                .width(Length::FillPortion(3)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let save_btn = button(
            text("Save WakaTime Settings")
                .size(12)
                .color(theme().text_primary),
        )
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
            top: 8.0,
            right: 20.0,
            bottom: 8.0,
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
        .spacing(12)
        .width(Length::Fill)
        .into()
    }
}
