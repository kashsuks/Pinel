use super::*;
use iced::widget::column;

impl App {
    pub(super) fn view_command_input_bar(&self) -> Element<'_, Message> {
        let input = text_input(":", &self.command_input.input)
            .id(self.command_input_id.clone())
            .on_input(Message::CommandInputChanged)
            .on_submit(Message::CommandInputSubmit)
            .size(14)
            .padding(iced::Padding {
                top: 10.0,
                right: 14.0,
                bottom: 10.0,
                left: 14.0,
            })
            .style(search_input_style)
            .width(Length::Fill);

        container(input)
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

    pub(super) fn view_settings_panel(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        #[cfg(feature = "unstable-comet")]
        let mut sections = vec![
            ("general", "General"),
            ("preferences", "Preferences"),
            ("wakatime", "WakaTime"),
        ];
        #[cfg(feature = "unstable-comet")]
        sections.push(("developer", "Developer"));
        #[cfg(not(feature = "unstable-comet"))]
        let sections = vec![
            ("general", "General"),
            ("preferences", "Preferences"),
            ("wakatime", "WakaTime"),
        ];

        let nav_items: Vec<Element<'_, Message>> = sections
            .into_iter()
            .map(|(key, label)| {
                let is_active = self.settings_section == key;
                let label_color = if is_active {
                    theme().text_primary
                } else {
                    theme().text_muted
                };
                let bg = if is_active {
                    Some(Background::Color(theme().bg_secondary))
                } else {
                    None
                };

                button(text(label).size(13).color(label_color))
                    .on_press(Message::SettingsNavigate(key.to_string()))
                    .style(move |_theme, _status| button::Style {
                        background: bg,
                        border: iced::Border::default(),
                        text_color: label_color,
                        ..Default::default()
                    })
                    .padding(iced::Padding {
                        top: 8.0,
                        right: 16.0,
                        bottom: 8.0,
                        left: 16.0,
                    })
                    .width(Length::Fill)
                    .into()
            })
            .collect();

        let nav_header = text("Settings").size(14).color(theme().text_primary);

        let close_btn = button(text("×").size(16).color(theme().text_muted))
            .on_press(Message::ToggleSettings)
            .style(|_theme, _status| button::Style {
                background: None,
                border: iced::Border::default(),
                text_color: theme().text_muted,
                ..Default::default()
            })
            .padding(iced::Padding {
                top: 2.0,
                right: 8.0,
                bottom: 2.0,
                left: 8.0,
            });

        let nav_top = row![nav_header, Space::new().width(Length::Fill), close_btn]
            .align_y(iced::Alignment::Center)
            .padding(iced::Padding {
                top: 12.0,
                right: 12.0,
                bottom: 8.0,
                left: 16.0,
            });

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let mut nav_col_items: Vec<Element<'_, Message>> = vec![nav_top.into(), separator.into()];
        nav_col_items.extend(nav_items);

        let nav_sidebar = container(scrollable(column(nav_col_items).spacing(2).padding(
            iced::Padding {
                top: 0.0,
                right: 0.0,
                bottom: 8.0,
                left: 0.0,
            },
        )))
        .width(Length::Fixed(180.0))
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme().bg_secondary)),
            ..Default::default()
        });

        let content_view: Element<'_, Message> = match self.settings_section.as_str() {
            "general" => self.view_settings_general(),
            "preferences" => self.view_settings_preferences(),
            "wakatime" => self.view_settings_wakatime(),
            #[cfg(feature = "unstable-comet")]
            "developer" => self.view_settings_developer(),
            _ => self.view_settings_general(),
        };

        let content_area = container(scrollable(
            container(content_view)
                .padding(iced::Padding {
                    top: 24.0,
                    right: 32.0,
                    bottom: 24.0,
                    left: 32.0,
                })
                .width(Length::Fill),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme().bg_editor)),
            ..Default::default()
        });

        let vert_sep = container(Space::new().width(Length::Fixed(1.0)).height(Length::Fill))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        row![nav_sidebar, vert_sep, content_area]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub(super) fn view_settings_general(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("General").size(18).color(theme().text_primary);
        let desc = text("General application settings and information.")
            .size(12)
            .color(theme().text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let version_row = row![
            text("Version")
                .size(13)
                .color(theme().text_muted)
                .width(Length::Fixed(140.0)),
            text("1.0.0").size(13).color(theme().text_primary),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let app_name_row = row![
            text("Application")
                .size(13)
                .color(theme().text_muted)
                .width(Length::Fixed(140.0)),
            text("Pinel Editor").size(13).color(theme().text_primary),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        let framework_row = row![
            text("Framework")
                .size(13)
                .color(theme().text_muted)
                .width(Length::Fixed(140.0)),
            text("iced 0.14").size(13).color(theme().text_primary),
        ]
        .spacing(12)
        .align_y(iced::Alignment::Center);

        column![
            heading,
            desc,
            separator,
            app_name_row,
            version_row,
            framework_row
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    pub(super) fn view_settings_preferences(&self) -> Element<'_, Message> {
        use crate::theme::BUILTIN_THEMES;
        use iced::widget::Space;

        let heading = text("Preferences").size(18).color(theme().text_primary);
        let desc = text("Configure editor behavior, formatting, and appearance.")
            .size(12)
            .color(theme().text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let tab_size_row = row![
            column![
                text("Tab Size").size(13).color(theme().text_muted),
                text("Number of spaces per indentation level")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            text_input("4", &self.editor_preferences.tab_size.to_string())
                .on_input(Message::SettingsTabSizeChanged)
                .size(13)
                .padding(iced::Padding {
                    top: 8.0,
                    right: 12.0,
                    bottom: 8.0,
                    left: 12.0
                })
                .style(search_input_style)
                .width(Length::Fixed(80.0)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let spaces_label = if self.editor_preferences.use_spaces {
            "Spaces"
        } else {
            "Tabs"
        };
        let spaces_row = row![
            column![
                text("Indent Using").size(13).color(theme().text_muted),
                text("Choose between spaces or tabs for indentation")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            button(text(spaces_label).size(12).color(theme().text_primary))
                .on_press(Message::SettingsToggleUseSpaces)
                .style(|_theme, _status| button::Style {
                    background: Some(Background::Color(theme().bg_secondary)),
                    border: iced::Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    text_color: theme().text_primary,
                    ..Default::default()
                })
                .padding(iced::Padding {
                    top: 6.0,
                    right: 16.0,
                    bottom: 6.0,
                    left: 16.0
                }),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let autosave_label = if self.editor_preferences.autosave_enabled {
            "Enabled"
        } else {
            "Disabled"
        };

        let autosave_toggle_row = row![
            column![
                text("Autosave").size(13).color(theme().text_muted),
                text("Automatically save modified files")
                    .size(11)
                    .color(theme().text_dim)
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            button(text(autosave_label).size(12).color(theme().text_primary))
                .on_press(Message::SettingsToggleAutosave)
                .style(|_theme, _status| button::Style {
                    background: Some(Background::Color(theme().bg_secondary)),
                    border: iced::Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.88),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    text_color: theme().text_primary,
                    ..Default::default()
                })
                .padding(iced::Padding {
                    top: 6.0,
                    right: 16.0,
                    bottom: 6.0,
                    left: 16.0
                }),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let autosave_interval_row = row![
            column![
                text("Autosave Interval").size(13).color(theme().text_muted),
                text("Delay in milliseconds before autosaving (30-1000)")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            text_input(
                "300",
                &self.editor_preferences.autosave_interval_ms.to_string()
            )
            .on_input(Message::SettingsAutosaveIntervalChanged)
            .size(13)
            .padding(iced::Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0
            })
            .style(search_input_style)
            .width(Length::Fixed(100.0)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let tab_drag_label = if self.editor_preferences.tab_drag_floating {
            "Floating"
        } else {
            "Static"
        };
        let tab_drag_row = row![
            column![
                text("Tab Drag Style").size(13).color(theme().text_muted),
                text("Floating: ghost follows cursor  ·  Static: tabs shift live")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            button(text(tab_drag_label).size(12).color(theme().text_primary))
                .on_press(Message::SettingsToggleTabDragFloating)
                .style(|_theme, _status| button::Style {
                    background: Some(Background::Color(theme().bg_secondary)),
                    border: iced::Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    text_color: theme().text_primary,
                    ..Default::default()
                })
                .padding(iced::Padding {
                    top: 6.0,
                    right: 16.0,
                    bottom: 6.0,
                    left: 16.0,
                }),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let line_number_width_row = row![
            column![
                text("Line Number Width").size(13).color(theme().text_muted),
                text("Gutter width in pixels (20\u{2013}120)")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            text_input("40", &self.editor_preferences.line_number_width.to_string())
                .on_input(Message::SettingsLineNumberWidthChanged)
                .size(13)
                .padding(iced::Padding {
                    top: 8.0,
                    right: 12.0,
                    bottom: 8.0,
                    left: 12.0
                })
                .style(search_input_style)
                .width(Length::Fixed(80.0)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let all_themes: Vec<&str> = {
            let mut v: Vec<&str> = BUILTIN_THEMES.to_vec();
            v.push("Custom (theme.lua)");
            v
        };

        let dropdown_trigger = button(
            row![
                text(&self.active_theme_name)
                    .size(12)
                    .color(theme().text_primary),
                Space::new().width(Length::Fill),
                text(if self.theme_dropdown_open {
                    "▲"
                } else {
                    "▼"
                })
                .size(10)
                .color(theme().text_dim),
            ]
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::SettingsNavigate(
            "__toggle_theme_dropdown__".to_string(),
        ))
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(theme().bg_secondary)),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.10),
                width: 1.0,
                radius: 6.0.into(),
            },
            text_color: theme().text_primary,
            ..Default::default()
        })
        .padding(iced::Padding {
            top: 8.0,
            right: 12.0,
            bottom: 8.0,
            left: 14.0,
        })
        .width(Length::Fixed(220.0));

        let mut dropdown_items: Vec<Element<'_, Message>> = vec![dropdown_trigger.into()];
        if self.theme_dropdown_open {
            for name in &all_themes {
                let is_active = self.active_theme_name == *name;
                let item_color = if is_active {
                    ACCENT_PURPLE
                } else {
                    theme().text_muted
                };
                let item_bg = if is_active {
                    Some(Background::Color(ACCENT_PURPLE.scale_alpha(0.15)))
                } else {
                    Some(Background::Color(theme().bg_secondary))
                };
                let name_str = name.to_string();
                let item = button(
                    row![
                        text(if is_active { "●" } else { "○" })
                            .size(9)
                            .color(item_color),
                        text(*name).size(12).color(item_color),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .on_press(if *name == "Custom (theme.lua)" {
                    Message::SettingsReloadTheme
                } else {
                    Message::SettingsSelectTheme(name_str)
                })
                .style(move |_theme, _status| button::Style {
                    background: item_bg,
                    border: iced::Border {
                        color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    text_color: item_color,
                    ..Default::default()
                })
                .padding(iced::Padding {
                    top: 7.0,
                    right: 14.0,
                    bottom: 7.0,
                    left: 14.0,
                })
                .width(Length::Fixed(220.0));
                dropdown_items.push(item.into());
            }
        }

        let theme_row = row![
            column![
                text("Color Theme").size(13).color(theme().text_muted),
                text("Select a color theme for the editor")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            column(dropdown_items)
                .spacing(0)
                .width(Length::Fixed(220.0)),
        ]
        .spacing(16)
        .align_y(iced::Alignment::Start);

        let save_btn = button(
            text("Save Preferences")
                .size(12)
                .color(theme().text_primary),
        )
        .on_press(Message::SettingsSavePreferences)
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
            tab_size_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            spaces_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            autosave_toggle_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            autosave_interval_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            line_number_width_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            tab_drag_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            theme_row,
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

    #[cfg(feature = "unstable-comet")]
    pub(super) fn view_settings_developer(&self) -> Element<'_, Message> {
        use iced::widget::Space;

        let heading = text("Developer").size(18).color(theme().text_primary);
        let desc = text("Debug logging and development tools. Logs may contain sensitive data.")
            .size(12)
            .color(theme().text_dim);

        let separator = container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let developer_mode_label = if self.editor_preferences.developer_mode {
            "Enabled"
        } else {
            "Disabled"
        };
        let developer_mode_btn = button(
            text(developer_mode_label)
                .size(12)
                .color(theme().text_primary),
        )
        .on_press(Message::SettingsToggleDeveloperMode)
        .style(|_theme, _status| button::Style {
            background: Some(Background::Color(
                if self.editor_preferences.developer_mode {
                    Color::from_rgba(0.2, 0.8, 0.2, 0.3)
                } else {
                    theme().bg_secondary
                },
            )),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: theme().text_primary,
            ..Default::default()
        })
        .padding(iced::Padding {
            top: 6.0,
            right: 16.0,
            bottom: 6.0,
            left: 16.0,
        });

        let developer_mode_row = row![
            column![
                text("Developer Mode").size(13).color(theme().text_muted),
                text("Enable debug logging for LSP events and actions")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            developer_mode_btn,
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        let lsp_enabled_label = if self.lsp_enabled {
            "Enabled"
        } else {
            "Disabled"
        };
        let lsp_status_btn = button(text(lsp_enabled_label).size(12).color(theme().text_primary))
            .on_press(Message::ToggleLsp)
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(if self.lsp_enabled {
                    Color::from_rgba(0.2, 0.8, 0.2, 0.3)
                } else {
                    theme().bg_secondary
                })),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: theme().text_primary,
                ..Default::default()
            })
            .padding(iced::Padding {
                top: 6.0,
                right: 16.0,
                bottom: 6.0,
                left: 16.0,
            });

        let lsp_row = row![
            column![
                text("LSP Support").size(13).color(theme().text_muted),
                text("Enable Language Server Protocol for autocompletion and hover")
                    .size(11)
                    .color(theme().text_dim),
            ]
            .spacing(2)
            .width(Length::FillPortion(2)),
            lsp_status_btn,
        ]
        .spacing(16)
        .align_y(iced::Alignment::Center);

        // ── LSP server availability ──────────────────────────────────────────
        let server_status = crate::lsp_setup::lsp_server_status();

        let server_status_heading = text("LSP Servers").size(13).color(theme().text_muted);

        let server_status_desc = text(
            "Language servers found on PATH. Install missing servers to enable LSP for those languages."
        )
        .size(11)
        .color(theme().text_dim);

        let server_rows: Vec<Element<'_, Message>> = server_status
            .iter()
            .map(|(key, path_opt)| {
                let (icon, icon_color, status_text) = match path_opt {
                    Some(p) => (
                        "●",
                        Color::from_rgb(0.30, 0.85, 0.50),
                        p.to_string_lossy().into_owned(),
                    ),
                    None => (
                        "○",
                        Color::from_rgba(0.9, 0.4, 0.3, 0.85),
                        "not found on PATH".to_string(),
                    ),
                };
                row![
                    text(icon)
                        .size(10)
                        .color(icon_color)
                        .width(Length::Fixed(14.0)),
                    text(*key)
                        .size(12)
                        .color(theme().text_muted)
                        .width(Length::Fixed(240.0)),
                    text(status_text)
                        .size(11)
                        .color(if path_opt.is_some() {
                            theme().text_dim
                        } else {
                            Color::from_rgba(0.9, 0.5, 0.35, 0.9)
                        })
                        .width(Length::Fill),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .into()
            })
            .collect();

        let server_status_panel = container(
            column(
                std::iter::once::<Element<'_, Message>>(server_status_heading.into())
                    .chain(std::iter::once::<Element<'_, Message>>(
                        server_status_desc.into(),
                    ))
                    .chain(server_rows)
                    .collect::<Vec<_>>(),
            )
            .spacing(6),
        )
        .padding(iced::Padding {
            top: 10.0,
            right: 12.0,
            bottom: 10.0,
            left: 12.0,
        })
        .width(Length::Fill)
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.025))),
            border: iced::Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.06),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..Default::default()
        });

        let clear_logs_btn = button(text("Clear Logs").size(12).color(theme().text_primary))
            .on_press(Message::ClearDeveloperLogs)
            .style(|_theme, _status| button::Style {
                background: Some(Background::Color(theme().bg_secondary)),
                border: iced::Border {
                    color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: theme().text_primary,
                ..Default::default()
            })
            .padding(iced::Padding {
                top: 6.0,
                right: 16.0,
                bottom: 6.0,
                left: 16.0,
            });

        let log_count = self.developer_logs.len();
        let logs_header = row![
            text("Debug Logs").size(13).color(theme().text_muted),
            Space::new().width(Length::Fill),
            text(format!("{} entries", log_count))
                .size(11)
                .color(theme().text_dim),
            clear_logs_btn,
        ]
        .align_y(iced::Alignment::Center);

        let log_entries: Vec<Element<'_, Message>> = self
            .developer_logs
            .iter()
            .rev()
            .take(50)
            .map(|(time, msg)| {
                let secs = time.elapsed().as_secs();
                let time_str = if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m", secs / 60)
                } else {
                    format!("{}h", secs / 3600)
                };
                let msg = msg.clone();
                let time_str = time_str.clone();
                container(row![
                    text(time_str).size(10).color(theme().text_dim),
                    Space::new().width(Length::Fixed(8.0)),
                    text(msg)
                        .size(11)
                        .color(theme().text_primary)
                        .width(Length::Fill),
                ])
                .padding(iced::Padding {
                    top: 2.0,
                    right: 8.0,
                    bottom: 2.0,
                    left: 8.0,
                })
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.02))),
                    ..Default::default()
                })
                .into()
            })
            .collect();

        let logs_panel: Element<'_, Message> = if log_entries.is_empty() {
            container(
                text("No logs yet. Enable developer mode and trigger actions to see logs.")
                    .size(11)
                    .color(theme().text_dim),
            )
            .padding(16)
            .into()
        } else {
            scrollable(column(log_entries).spacing(2))
                .height(Length::Fixed(300.0))
                .into()
        };

        column![
            heading,
            desc,
            separator,
            developer_mode_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            lsp_row,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            server_status_panel,
            container(Space::new().width(Length::Fill).height(Length::Fixed(1.0))).style(
                |_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
                    ..Default::default()
                }
            ),
            Space::new().height(Length::Fixed(8.0)),
            logs_header,
            Space::new().height(Length::Fixed(8.0)),
            logs_panel,
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    }

    pub(super) fn view_startup_page(&self) -> iced::Element<'_, Message> {
        use crate::theme::BUILTIN_THEMES;
        use iced::widget::{column, container, row, scrollable, text};
        use iced::{Background, Color, Length};

        let themes_to_show = BUILTIN_THEMES;

        // allow the switch helper to toggle
        let toggle = |label: &str, sublabel: &str, on: bool, msg: Message| -> iced::Element<'_, Message> {
            let track_color = if on {
                Color::from_rgb(0.133, 0.773, 0.369) // green
            } else {
                Color::from_rgba(1.0, 1.0, 1.0, 0.15)
            };
            let knob_x = if on { 22.0_f32 } else { 4.0_f32 };

            let track = container(
                container(iced::widget::Space::new())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .style(move |_t| container::Style {
                        background: Some(Background::Color(Color::WHITE)),
                        border: iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .padding(iced::Padding {
                        top:0.0,
                        left: knob_x,
                        bottom: 0.0,
                        right: 0.0,
                    }),
            )
            .width(Length::Fixed(42.0))
            .height(Length::Fixed(24.0))
            .style(move |_t| container::Style {
                background: Some(Background::Color(track_color)),
                border: iced::Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            iced::widget::button(
                row![
                    column![
                        text(label.to_string()).size(13).color(theme().text_primary),
                        text(sublabel.to_string()).size(11).color(theme().text_dim),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    track,
                ]
                .spacing(16)
                .align_y(iced::Alignment::Center),
            )
            .on_press(msg)
            .style(|_t, _s| iced::widget::button::Style {
                background: None,
                border: iced::Border::default(),
                text_color: Color::TRANSPARENT,
                ..Default::default()
            })
            .padding(iced::Padding {
                top: 10.0,
                right: 16.0,
                bottom: 10.0,
                left: 16.0,
            })
            .width(Length::Fill)
            .into()
        };

        let theme_items: Vec<iced::Element<'_, Message>> = themes_to_show
            .iter()
            .map(|&name| {
                let is_selected = self.startup_selected_theme == name;
                let accent = if is_selected {
                    crate::theme::ACCENT_PURPLE
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.0)
                };
                let bg = if is_selected {
                    Color::from_rgba(
                        crate::theme::ACCENT_PURPLE.r,
                        crate::theme::ACCENT_PURPLE.g,
                        crate::theme::ACCENT_PURPLE.b,
                        0.15,
                    )
                } else {
                    Color::from_rgba(1.0, 1.0, 1.0, 0.04)
                };
                iced::widget::button(
                    row![
                        text(if is_selected { "●" } else { "○" })
                            .size(10)
                            .color(if is_selected {
                                crate::theme::ACCENT_PURPLE
                            } else {
                                theme().text_dim
                            }),
                        text(name).size(12).color(if is_selected {
                            theme().text_primary
                        } else {
                            theme().text_muted
                        }),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .on_press(Message::StartupThemeSelected(name.to_string()))
                .style(move |_t, _s| iced::widget::button::Style {
                    background: Some(Background::Color(bg)),
                    border: iced::Border {
                        color: accent,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    text_color: Color::TRANSPARENT,
                    ..Default::default()
                })
                .padding(iced::Padding {
                    top: 8.0,
                    right: 14.0,
                    bottom: 8.0,
                    left: 14.0,
                })
                .width(Length::Fill)
                .into()
            })
            .collect();

        let mut theme_rows: Vec<iced::Element<'_, Message>> = Vec::new();
        let mut iter = theme_items.into_iter().peekable();
        while iter.peek().is_some() {
            let a = iter.next().unwrap();
            if let Some(b) = iter.next() {
                theme_rows.push(row![a, b].spacing(8).into());
            } else {
                theme_rows.push(row![a].spacing(8).into());
            }
        }

        let continue_btn = iced::widget::button(
            text("Get Started →")
                .size(14)
                .color(Color::WHITE),
        )
        .on_press(Message::StartupPageDone)
        .style(|_t, _s| iced::widget::button::Style {
            background: Some(Background::Color(crate::theme::ACCENT_PURPLE)),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            text_color: Color::WHITE,
            ..Default::default()
        })
        .padding(iced::Padding {
            top: 12.0,
            right: 32.0,
            bottom: 12.0,
            left: 32.0,
        });

        let left_panel = container(
            scrollable(
                column![
                    text("Welcome to Pinel")
                        .size(28)
                        .color(theme().text_primary),
                    text("A lightweight editor built with Rust.")
                        .size(13)
                        .color(theme().text_dim),
                    iced::widget::Space::new().height(Length::Fixed(24.0)),
                    text("Choose a theme")
                        .size(14)
                        .color(theme().text_muted),
                    iced::widget::Space::new().height(Length::Fixed(8.0)),
                    column(theme_rows).spacing(8),
                ]
                .spacing(8)
                .padding(iced::Padding {
                    top: 48.0,
                    right: 32.0,
                    bottom: 48.0,
                    left: 48.0,
                })
                .width(Length::Fill),
            )
            .height(Length::Fill),
        )
        .width(Length::FillPortion(3))
        .height(Length::Fill)
        .style(|_t| container::Style {
            background: Some(Background::Color(theme().bg_editor)),
            ..Default::default()
        });

        let divider = container(iced::widget::Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(|_t| container::Style {
                background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.06))),
                ..Default::default()
            });

        let right_panel = container(
            scrollable(
                column![
                    iced::widget::Space::new().height(Length::Fixed(48.0)),
                    text("Editor Mode")
                        .size(14)
                        .color(theme().text_muted)
                        .width(Length::Fill),
                    container(
                        column![
                            toggle(
                                "Vim Mode",
                                "hjkl navigation, normal/insert modes",
                                self.startup_vim_mode,
                                Message::StartupToggleVimMode
                            ),
                            container(iced::widget::Space::new())
                                .width(Length::Fill)
                                .height(Length::Fixed(1.0))
                                .style(|_t| container::Style {
                                    background: Some(Background::Color(Color::from_rgba(
                                        1.0, 1.0, 1.0, 0.05,
                                    ))),
                                    ..Default::default()
                                }),
                            toggle(
                                "Helix Mode",
                                "Selection-first, motion-based editing",
                                self.startup_helix_mode,
                                Message::StartupToggleHelixMode
                            ),
                        ]
                        .spacing(0),
                    )
                    .style(|_t| container::Style {
                        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.04))),
                        border: iced::Border {
                            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                            width: 1.0,
                            radius: 10.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill),
                    iced::widget::Space::new().height(Length::Fixed(20.0)),
                    text("System")
                        .size(14)
                        .color(theme().text_muted)
                        .width(Length::Fill),
                    container(toggle(
                        "Run on Startup",
                        "Launch Pinel automatically at login",
                        self.startup_run_on_startup,
                        Message::StartupToggleRunOnStartup,
                    ))
                    .style(|_t| container::Style {
                        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.04))),
                        border: iced::Border {
                            color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                            width: 1.0,
                            radius: 10.0.into(),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill),
                    iced::widget::Space::new().height(Length::Fill),
                    container(continue_btn)
                        .width(Length::Fill)
                        .center_x(Length::Fill)
                        .padding(iced::Padding {
                            top: 32.0,
                            right: 0.0,
                            bottom: 48.0,
                            left: 0.0,
                        }),
                ]
                .spacing(8)
                .padding(iced::Padding {
                    top: 0.0,
                    right: 40.0,
                    bottom: 0.0,
                    left: 32.0,
                })
                .width(Length::Fill),
            )
            .height(Length::Fill),
        )
        .width(Length::FillPortion(2))
        .height(Length::Fill)
        .style(|_t| container::Style {
            background: Some(Background::Color(theme().bg_secondary)),
            ..Default::default()
        });

        container(
            row![left_panel, divider, right_panel]
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
