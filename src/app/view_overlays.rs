use super::*;
use iced::widget::column;

impl App {
    pub(super) fn view_notification_toast(&self) -> Element<'_, Message> {
        let check_circle = container(text("✓").size(14).color(Color::from_rgb(0.40, 0.90, 0.55)))
            .width(Length::Fixed(26.0))
            .height(Length::Fixed(26.0))
            .center_x(Length::Fixed(26.0))
            .center_y(Length::Fixed(26.0))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.30, 0.85, 0.50, 0.15))),
                border: iced::Border {
                    color: Color::from_rgba(0.35, 0.88, 0.52, 0.35),
                    width: 1.0,
                    radius: 13.0.into(),
                },
                ..Default::default()
            });

        let label = text("Preferences saved")
            .size(13)
            .color(Color::from_rgb(0.85, 0.97, 0.88));

        let dismiss_btn = button(
            text("×")
                .size(14)
                .color(Color::from_rgba(0.65, 0.90, 0.70, 0.7)),
        )
        .on_press(Message::DismissNotification)
        .style(|_theme, _status| button::Style {
            background: None,
            border: iced::Border::default(),
            text_color: Color::from_rgba(0.65, 0.90, 0.70, 0.7),
            ..Default::default()
        })
        .padding(iced::Padding {
            top: 0.0,
            right: 4.0,
            bottom: 0.0,
            left: 8.0,
        });

        let toast_inner = row![check_circle, label, dismiss_btn]
            .spacing(10)
            .align_y(iced::Alignment::Center);

        let toast = container(toast_inner)
            .padding(iced::Padding {
                top: 10.0,
                right: 16.0,
                bottom: 10.0,
                left: 12.0,
            })
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.07, 0.20, 0.10, 0.96))),
                border: iced::Border {
                    color: Color::from_rgba(0.30, 0.78, 0.45, 0.40),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.55),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 32.0,
                },
                ..Default::default()
            });

        container(column![
            container(toast).center_x(Length::Fill),
            iced::widget::Space::new().height(Length::Fill),
        ])
        .padding(iced::Padding {
            top: 20.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub(super) fn view_update_banner(&self) -> Element<'_, Message> {
        let Some(info) = &self.update_banner else {
            return iced::widget::Space::new().into();
        };

        let label = text(format!("Pinel {} is available", info.version))
            .size(13)
            .color(Color::from_rgb(0.85, 0.93, 1.0));

        let open_btn = button(
            text("Open release page →")
                .size(12)
                .color(Color::from_rgb(0.55, 0.75, 1.0)),
        )
        .on_press(Message::DismissUpdateBanner)
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(Color::from_rgba(0.30, 0.55, 1.0, 0.15))),
            border: iced::Border {
                color: Color::from_rgba(0.40, 0.65, 1.0, 0.35),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: Color::from_rgb(0.55, 0.75, 1.0),
            ..Default::default()
        })
        .padding(iced::Padding {
            top: 5.0,
            right: 12.0,
            bottom: 5.0,
            left: 12.0,
        });

        let dismiss_btn = button(
            text("×")
                .size(14)
                .color(Color::from_rgba(0.65, 0.80, 1.0, 0.7)),
        )
        .on_press(Message::DismissUpdateBanner)
        .style(|_theme, _status| button::Style {
            background: None,
            border: iced::Border::default(),
            text_color: Color::from_rgba(0.65, 0.80, 1.0, 0.7),
            ..Default::default()
        })
        .padding(iced::Padding {
            top: 0.0,
            right: 4.0,
            bottom: 0.0,
            left: 8.0,
        });

        let banner_inner = row![label, open_btn, dismiss_btn]
            .spacing(12)
            .align_y(iced::Alignment::Center);

        let banner = container(banner_inner)
            .padding(iced::Padding {
                top: 10.0,
                right: 16.0,
                bottom: 10.0,
                left: 16.0,
            })
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.05, 0.10, 0.25, 0.96))),
                border: iced::Border {
                    color: Color::from_rgba(0.35, 0.55, 1.0, 0.40),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.55),
                    offset: iced::Vector::new(0.0, 8.0),
                    blur_radius: 32.0,
                },
                ..Default::default()
            });

        container(column![
            iced::widget::Space::new().height(Length::Fill),
            container(banner).padding(iced::Padding {
                top: 0.0,
                right: 20.0,
                bottom: 40.0,
                left: 0.0,
            }),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_right(Length::Fill)
        .into()
    }

    pub(super) fn view_command_palette_overlay(&self) -> Element<'_, Message> {
        use iced::widget::{center, opaque, stack, Space};

        let input = text_input("> Type a command...", &self.command_palette.input)
            .id(self.command_palette_input_id.clone())
            .on_input(Message::CommandPaletteQueryChanged)
            .size(15)
            .padding(iced::Padding {
                top: 16.0,
                right: 18.0,
                bottom: 16.0,
                left: 18.0,
            })
            .style(search_input_style)
            .width(Length::Fill);

        let mut items: Vec<Element<'_, Message>> = Vec::new();
        for (idx, cmd) in self.command_palette.filtered_commands.iter().enumerate() {
            let is_selected = idx == self.command_palette_selected;
            let cmd_name = cmd.name.clone();
            let shortcut_text = cmd.description.clone();

            items.push(
                button(
                    row![
                        text(&cmd.name).size(13).color(if is_selected {
                            theme().text_primary
                        } else {
                            theme().text_muted
                        }),
                        iced::widget::Space::new().width(Length::Fill),
                        text(shortcut_text).size(11).color(theme().text_dim),
                    ]
                    .align_y(iced::Alignment::Center),
                )
                .style(file_finder_item_style(is_selected))
                .on_press(Message::CommandPaletteSelect(cmd_name))
                .padding(iced::Padding {
                    top: 7.0,
                    right: 10.0,
                    bottom: 7.0,
                    left: 10.0,
                })
                .width(Length::Fill)
                .into(),
            );
        }

        let has_results = !items.is_empty();
        let separator = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.07))),
                ..Default::default()
            });

        let inner: Element<'_, Message> = if has_results {
            let results_col = scrollable(column(items).spacing(2).padding(iced::Padding {
                top: 6.0,
                right: 6.0,
                bottom: 6.0,
                left: 6.0,
            }))
            .height(Length::Shrink);
            column![input, separator, results_col].spacing(0).into()
        } else {
            input.into()
        };

        let overlay_box = container(inner)
            .width(Length::Fixed(520.0))
            .max_height(440.0)
            .style(file_finder_panel_style);

        let backdrop = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.45))),
                    ..Default::default()
                }),
        )
        .on_press(Message::ToggleCommandPalette);

        stack![backdrop, center(opaque(overlay_box))].into()
    }

    pub(super) fn view_find_replace_panel(&self) -> Element<'_, Message> {
        let find_input = text_input("Find...", &self.find_replace.find_text)
            .id(self.find_input_id.clone())
            .on_input(Message::FindQueryChanged)
            .size(13)
            .padding(iced::Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0,
            })
            .style(search_input_style)
            .width(Length::Fill);

        let replace_input = text_input("Replace...", &self.find_replace.replace_text)
            .id(self.replace_input_id.clone())
            .on_input(Message::ReplaceQueryChanged)
            .size(13)
            .padding(iced::Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0,
            })
            .style(search_input_style)
            .width(Length::Fill);

        let match_info = text(self.find_replace.match_status())
            .size(11)
            .color(theme().text_dim);

        let case_btn = button(
            text(if self.find_replace.case_sensitive {
                "Aa"
            } else {
                "aa"
            })
            .size(11),
        )
        .on_press(Message::ToggleCaseSensitive)
        .style(tab_close_button_style)
        .padding(iced::Padding {
            top: 3.0,
            right: 6.0,
            bottom: 3.0,
            left: 6.0,
        });

        let prev_btn = button(text("↑").size(12))
            .on_press(Message::FindPrev)
            .style(tab_close_button_style)
            .padding(iced::Padding {
                top: 3.0,
                right: 6.0,
                bottom: 3.0,
                left: 6.0,
            });

        let next_btn = button(text("↓").size(12))
            .on_press(Message::FindNext)
            .style(tab_close_button_style)
            .padding(iced::Padding {
                top: 3.0,
                right: 6.0,
                bottom: 3.0,
                left: 6.0,
            });

        let replace_btn = button(text("Replace").size(11).color(theme().text_muted))
            .on_press(Message::ReplaceOne)
            .style(tab_close_button_style)
            .padding(iced::Padding {
                top: 3.0,
                right: 8.0,
                bottom: 3.0,
                left: 8.0,
            });

        let replace_all_btn = button(text("All").size(11).color(theme().text_muted))
            .on_press(Message::ReplaceAll)
            .style(tab_close_button_style)
            .padding(iced::Padding {
                top: 3.0,
                right: 8.0,
                bottom: 3.0,
                left: 8.0,
            });

        let close_btn = button(text("✕").size(12).color(theme().text_muted))
            .on_press(Message::ToggleFindReplace)
            .style(tab_close_button_style)
            .padding(iced::Padding {
                top: 3.0,
                right: 6.0,
                bottom: 3.0,
                left: 6.0,
            });

        let find_row = row![find_input, match_info, case_btn, prev_btn, next_btn, close_btn]
            .spacing(6)
            .align_y(iced::Alignment::Center);

        let replace_row = row![replace_input, replace_btn, replace_all_btn]
            .spacing(6)
            .align_y(iced::Alignment::Center);

        container(column![find_row, replace_row].spacing(6))
            .padding(iced::Padding {
                top: 10.0,
                right: 14.0,
                bottom: 10.0,
                left: 14.0,
            })
            .width(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(theme().bg_secondary)),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            })
            .into()
    }
}
