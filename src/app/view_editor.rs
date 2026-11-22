use frostmark::MarkWidget;
use iced::widget::column;

use super::*;

impl App {
    pub(super) fn view_search_panel(&self) -> Element<'_, Message> {
        let input = text_input("Search across workspace...", &self.search_query)
            .id(self.search_input_id.clone())
            .on_input(Message::SearchQueryChanged)
            .style(search_input_style)
            .size(TEXT_MD)
            .padding(10)
            .width(Length::Fill);

        let mut content_col = column![input].spacing(6);

        if !self.search_results.is_empty() {
            let mut result_items: Vec<Element<'_, Message>> = Vec::new();

            for result in &self.search_results {
                result_items.push(
                    container(text(&result.file_name).size(TEXT_XS).color(theme().text_secondary))
                        .padding(iced::Padding {
                            top: 6.0,
                            right: 6.0,
                            bottom: 2.0,
                            left: 6.0,
                        })
                        .into(),
                );

                for m in result.matches.iter().take(3) {
                    let line_text = format!("  {}:  {}", m.line_number, m.line_content.trim());
                    let path = result.path.clone();
                    let line_num = m.line_number;

                    result_items.push(
                        button(text(line_text).size(TEXT_XS).color(theme().text_muted))
                            .style(tree_button_style)
                            .on_press(Message::SearchResultClicked(path, line_num))
                            .padding(iced::Padding {
                                top: 3.0,
                                right: 6.0,
                                bottom: 3.0,
                                left: SPACE_3,
                            })
                            .width(Length::Fill)
                            .into(),
                    );
                }

                if result.matches.len() > 3 {
                    result_items.push(
                        container(
                            text(format!("  ... and {} more", result.matches.len() - 3))
                                .size(10)
                                .color(theme().text_dim),
                        )
                        .padding(iced::Padding {
                            top: 1.0,
                            right: 6.0,
                            bottom: 2.0,
                            left: SPACE_3,
                        })
                        .into(),
                    );
                }
            }

            let results_scroll = scrollable(column(result_items).spacing(1)).height(Length::Shrink);

            content_col = content_col.push(container(results_scroll).max_height(400.0));
        }

        container(content_col)
            .width(Length::Fixed(320.0))
            .padding(10)
            .style(search_panel_style)
            .into()
    }

    pub(super) fn view_editor(&self) -> Element<'_, Message> {
        if self.pending_sensitive_open.is_some() {
            return container(
                column![
                    text("You are opening a sensitive file, continue?")
                        .size(18)
                        .color(theme().text_muted),
                    row![
                        button(text("Yes").size(TEXT_MD))
                            .on_press(Message::SensitiveFileOpenConfirm(true))
                            .padding(iced::Padding {
                                top: SPACE_2,
                                right: SPACE_4,
                                bottom: SPACE_2,
                                left: SPACE_4,
                            }),
                        button(text("No").size(TEXT_MD))
                            .on_press(Message::SensitiveFileOpenConfirm(false))
                            .padding(iced::Padding {
                                top: SPACE_2,
                                right: SPACE_4,
                                bottom: SPACE_2,
                                left: SPACE_4,
                            }),
                    ]
                    .spacing(SPACE_3)
                    .align_y(iced::Alignment::Center),
                ]
                .spacing(SPACE_4)
                .align_x(iced::Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(idx) {
                match &tab.kind {
                    TabKind::Editor { code_editor, .. } => {
                        let editor = container(code_editor.view().map(Message::CodeEditorEvent))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(|_theme| container::Style {
                                background: Some(iced::Background::Color(theme().bg_editor)),
                                ..Default::default()
                            });
                        let editor = mouse_area(editor).on_press(Message::FocusEditor);

                        let lsp_overlay = if self.lsp_enabled {
                            iced_code_editor::view_lsp_overlay(
                                &self.lsp_overlay,
                                code_editor,
                                &iced::Theme::CatppuccinMocha,
                                code_editor.font_size(),
                                code_editor.line_height(),
                                Message::LspOverlay,
                            )
                        } else {
                            container(iced::widget::Space::new()).into()
                        };

                        let show_panel = !self.lsp_enabled
                            && self.autocomplete.active
                            && !self.autocomplete.suggestions.is_empty();
                        let editor_stack: Element<'_, Message> = if show_panel {
                            // ── Purple autocomplete modal with navigation ───────────────────
                            let kind_color = |kind: &crate::autocomplete::types::SuggestionKind| {
                                use crate::autocomplete::types::SuggestionKind;
                                match kind {
                                    SuggestionKind::Keyword => Color::from_rgb(0.796, 0.651, 0.969),
                                    SuggestionKind::Function => {
                                        Color::from_rgb(0.000, 0.663, 1.000)
                                    },
                                    SuggestionKind::Method => Color::from_rgb(0.537, 0.863, 0.922),
                                    SuggestionKind::Type => Color::from_rgb(0.976, 0.886, 0.686),
                                    SuggestionKind::Constant => {
                                        Color::from_rgb(0.976, 0.886, 0.686)
                                    },
                                    SuggestionKind::Variable => {
                                        Color::from_rgb(0.706, 0.745, 0.996)
                                    },
                                    SuggestionKind::Property => {
                                        Color::from_rgb(0.000, 0.663, 1.000)
                                    },
                                    SuggestionKind::Module => Color::from_rgb(0.537, 0.863, 0.922),
                                    SuggestionKind::Macro => Color::from_rgb(0.000, 1.000, 0.824),
                                    SuggestionKind::Snippet => Color::from_rgb(0.976, 0.886, 0.686),
                                }
                            };

                            let accent_purple = Color::from_rgb(0.796, 0.651, 0.969);
                            let bg_modal = Color::from_rgb(0.149, 0.149, 0.212);
                            let bg_selected = Color::from_rgba(0.796, 0.651, 0.969, 0.18);
                            let divider = Color::from_rgba(1.0, 1.0, 1.0, 0.06);

                            let mut items: Vec<Element<'_, Message>> = Vec::new();
                            let visible_count = self.autocomplete.suggestions.len().min(8);

                            for (i, suggestion) in
                                self.autocomplete.suggestions.iter().take(visible_count).enumerate()
                            {
                                let is_selected = i == self.autocomplete.selected_index;
                                let ic = kind_color(&suggestion.kind);
                                let label_color = if is_selected {
                                    theme().text_primary
                                } else {
                                    theme().text_muted
                                };
                                let row_bg = if is_selected {
                                    Some(iced::Background::Color(bg_selected))
                                } else {
                                    None
                                };
                                items.push(
                                    container(
                                        row![
                                            container(
                                                text(suggestion.kind.icon())
                                                    .size(TEXT_XS)
                                                    .color(ic)
                                            )
                                            .width(Length::Fixed(20.0))
                                            .center_x(Length::Fixed(20.0)),
                                            text(&suggestion.text).size(TEXT_SM).color(label_color),
                                            iced::widget::Space::new().width(Length::Fill),
                                            text(format!("{:?}", suggestion.kind).to_lowercase())
                                                .size(10)
                                                .color(Color::from_rgba(ic.r, ic.g, ic.b, 0.65)),
                                        ]
                                        .spacing(6)
                                        .align_y(iced::Alignment::Center),
                                    )
                                    .padding(iced::Padding {
                                        top: SPACE_1,
                                        right: 10.0,
                                        bottom: SPACE_1,
                                        left: SPACE_2,
                                    })
                                    .width(Length::Fill)
                                    .style(move |_theme| container::Style {
                                        background: row_bg,
                                        border: iced::Border {
                                            color: Color::TRANSPARENT,
                                            width: 0.0,
                                            radius: 4.0.into(),
                                        },
                                        ..Default::default()
                                    })
                                    .into(),
                                );
                            }

                            // Navigation footer
                            items.push(
                                container(
                                    container(
                                        row![
                                            text("↑↓").size(9).color(accent_purple),
                                            text(" navigate · ").size(9).color(theme().text_dim),
                                            text("↵").size(9).color(accent_purple),
                                            text(" accept · ").size(9).color(theme().text_dim),
                                            text("esc").size(9).color(accent_purple),
                                            text(" dismiss").size(9).color(theme().text_dim),
                                        ]
                                        .spacing(0)
                                        .align_y(iced::Alignment::Center),
                                    )
                                    .padding(iced::Padding {
                                        top: SPACE_1,
                                        right: SPACE_2,
                                        bottom: SPACE_1,
                                        left: SPACE_2,
                                    })
                                    .width(Length::Fill)
                                    .style(move |_theme| {
                                        container::Style {
                                            background: Some(iced::Background::Color(
                                                Color::from_rgba(0.796, 0.651, 0.969, 0.06),
                                            )),
                                            border: iced::Border {
                                                color: divider,
                                                width: 1.0,
                                                radius: 0.0.into(),
                                            },
                                            ..Default::default()
                                        }
                                    }),
                                )
                                .width(Length::Fill)
                                .into(),
                            );

                            let panel = container(column(items).spacing(1))
                                .padding(SPACE_1)
                                .max_width(320.0)
                                .style(move |_theme| container::Style {
                                    background: Some(iced::Background::Color(bg_modal)),
                                    border: iced::Border {
                                        color: Color::from_rgba(
                                            accent_purple.r,
                                            accent_purple.g,
                                            accent_purple.b,
                                            0.35,
                                        ),
                                        width: 1.0,
                                        radius: 8.0.into(),
                                    },
                                    shadow: iced::Shadow {
                                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                                        offset: iced::Vector::new(0.0, 4.0),
                                        blur_radius: 20.0,
                                    },
                                    ..Default::default()
                                });

                            let cursor_pos = code_editor
                                .cursor_screen_position()
                                .unwrap_or(iced::Point::new(48.0, 20.0));
                            let x = cursor_pos.x.clamp(0.0, 500.0);
                            let y = (cursor_pos.y + 20.0).clamp(0.0, 560.0);

                            let positioned_panel = container(panel)
                                .padding(iced::Padding {
                                    top: y,
                                    left: x,
                                    bottom: 0.0,
                                    right: 0.0,
                                })
                                .width(Length::Fill)
                                .height(Length::Fill);

                            stack![editor, positioned_panel, lsp_overlay]
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .into()
                        } else {
                            stack![editor, lsp_overlay]
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .into()
                        };

                        if let Some(preview) = self
                            .markdown_preview
                            .as_ref()
                            .filter(|preview| preview.source_path == tab.path)
                        {
                            let separator = container(text(""))
                                .width(Length::Fixed(1.0))
                                .height(Length::Fill)
                                .style(|_theme| container::Style {
                                    background: Some(iced::Background::Color(Color::from_rgba(
                                        1.0, 1.0, 1.0, 0.08,
                                    ))),
                                    ..Default::default()
                                });

                            let preview_panel = container(
                                scrollable(
                                    container(MarkWidget::new(&preview.state).style(
                                        frostmark::Style {
                                            text_color: Some(theme().text_primary),
                                            ..Default::default()
                                        },
                                    ))
                                    .padding(SPACE_4)
                                    .width(Length::Fill),
                                )
                                .height(Length::Fill),
                            )
                            .width(Length::FillPortion(1))
                            .height(Length::Fill)
                            .style(|_theme| container::Style {
                                background: Some(iced::Background::Color(theme().bg_secondary)),
                                ..Default::default()
                            });

                            return row![
                                container(editor_stack)
                                    .width(Length::FillPortion(1))
                                    .height(Length::Fill),
                                separator,
                                preview_panel,
                            ]
                            .height(Length::Fill)
                            .into();
                        }

                        return editor_stack;
                    },
                    TabKind::Preview { md_items } => {
                        return scrollable(
                            markdown::view(
                                md_items,
                                markdown::Settings::with_style(markdown::Style::from_palette(
                                    iced::theme::Palette::CATPPUCCIN_MOCHA,
                                )),
                            )
                            .map(Message::MarkdownLinkClicked),
                        )
                        .height(Length::Fill)
                        .into();
                    },
                    TabKind::Commit {
                        hash,
                        diff,
                        collapsed,
                        ..
                    } => {
                        return self.view_commit_tab(hash, diff.as_deref(), collapsed);
                    },
                    TabKind::Image { handle } => {
                        return container(
                            scrollable(
                                container(
                                    iced::widget::image::Image::new(handle.clone())
                                        .content_fit(iced::ContentFit::Contain),
                                )
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center_x(Length::Fill)
                                .center_y(Length::Fill),
                            )
                            .width(Length::Fill)
                            .height(Length::Fill),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(theme().bg_editor)),
                            ..Default::default()
                        })
                        .into();
                    },
                    #[cfg(feature = "video")]
                    TabKind::Video { player } => {
                        return container(
                            column![
                                // Video frame fills the space
                                container(
                                    iced_video_player::VideoPlayer::new(player)
                                        .width(Length::Fill)
                                        .height(Length::Fill)
                                        .content_fit(iced::ContentFit::Contain)
                                        .on_end_of_stream(Message::VideoEndOfStream)
                                        .on_new_frame(Message::VideoNewFrame)
                                )
                                .width(Length::Fill)
                                .height(Length::Fill),
                                // Playback controls bar
                                container(
                                    row![
                                        button(
                                            text(if player.paused() {
                                                "▶"
                                            } else {
                                                "⏸"
                                            })
                                            .size(14)
                                            .color(theme().text_primary)
                                        )
                                        .on_press(Message::VideoTogglePause)
                                        .style(|_t, _s| button::Style {
                                            background: Some(iced::Background::Color(
                                                theme().bg_secondary,
                                            )),
                                            border: iced::Border {
                                                color: iced::Color::TRANSPARENT,
                                                width: 0.0,
                                                radius: RADIUS_SM.into(),
                                            },
                                            text_color: theme().text_primary,
                                            ..Default::default()
                                        })
                                        .padding(
                                            iced::Padding {
                                                top: 6.0,
                                                right: SPACE_3,
                                                bottom: 6.0,
                                                left: SPACE_3,
                                            }
                                        ),
                                        text(format!(
                                            "{:.0}s / {:.0}s",
                                            player.position().as_secs_f64(),
                                            player.duration().as_secs_f64()
                                        ))
                                        .size(TEXT_SM)
                                        .color(theme().text_muted),
                                    ]
                                    .spacing(SPACE_3)
                                    .align_y(iced::Alignment::Center),
                                )
                                .padding(iced::Padding {
                                    top: SPACE_2,
                                    right: SPACE_3,
                                    bottom: SPACE_2,
                                    left: SPACE_3,
                                })
                                .width(Length::Fill)
                                .style(|_theme| container::Style {
                                    background: Some(
                                        iced::Background::Color(theme().bg_secondary,)
                                    ),
                                    ..Default::default()
                                }),
                            ]
                            .spacing(0),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(Color::BLACK)),
                            ..Default::default()
                        })
                        .into();
                    },
                    TabKind::Audio {
                        playing,
                        duration_secs,
                        position_secs,
                        file_path,
                        ..
                    } => {
                        let pct = if *duration_secs > 0.0 {
                            position_secs / duration_secs
                        } else {
                            0.0
                        };

                        let fmt_time = |secs: f32| -> String {
                            let m = (secs as u32) / 60;
                            let s = (secs as u32) % 60;
                            format!("{m}:{s:02}")
                        };

                        let file_name =
                            file_path.file_name().unwrap_or_default().to_string_lossy().to_string();

                        let play_pause_btn = button(
                            text(if *playing {
                                "⏸  Pause"
                            } else {
                                "▶  Play"
                            })
                            .size(14)
                            .color(theme().text_primary),
                        )
                        .on_press(if *playing {
                            Message::AudioPause
                        } else {
                            Message::AudioPlay
                        })
                        .style(|_t, _s| button::Style {
                            background: Some(iced::Background::Color(
                                crate::theme::ACCENT_PURPLE.scale_alpha(0.2),
                            )),
                            border: iced::Border {
                                color: crate::theme::ACCENT_PURPLE.scale_alpha(0.4),
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            text_color: theme().text_primary,
                            ..Default::default()
                        })
                        .padding(iced::Padding {
                            top: 10.0,
                            right: 24.0,
                            bottom: 10.0,
                            left: 24.0,
                        });

                        let stop_btn = button(text("⏹  Stop").size(14).color(theme().text_muted))
                            .on_press(Message::AudioStop)
                            .style(|_t, _s| button::Style {
                                background: Some(iced::Background::Color(theme().bg_secondary)),
                                border: iced::Border {
                                    color: iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                                    width: 1.0,
                                    radius: 8.0.into(),
                                },
                                text_color: theme().text_muted,
                                ..Default::default()
                            })
                            .padding(iced::Padding {
                                top: 10.0,
                                right: 24.0,
                                bottom: 10.0,
                                left: 24.0,
                            });

                        // Progress bar drawn as two stacked containers
                        let bar_filled = container(iced::widget::Space::new())
                            .width(Length::FillPortion((pct * 1000.0) as u16))
                            .height(Length::Fixed(4.0))
                            .style(|_t| container::Style {
                                background: Some(iced::Background::Color(
                                    crate::theme::ACCENT_PURPLE,
                                )),
                                border: iced::Border {
                                    radius: 2.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });

                        let bar_empty = container(iced::widget::Space::new())
                            .width(Length::FillPortion(((1.0 - pct) * 1000.0) as u16))
                            .height(Length::Fixed(4.0))
                            .style(|_t| container::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgba(
                                    1.0, 1.0, 1.0, 0.10,
                                ))),
                                border: iced::Border {
                                    radius: 2.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            });

                        let progress_row =
                            row![bar_filled, bar_empty].width(Length::Fill).spacing(0);

                        return container(
                            column![
                                // Waveform icon placeholder
                                container(text("🎵").size(64))
                                    .width(Length::Fill)
                                    .center_x(Length::Fill),
                                // File name
                                text(file_name).size(18).color(theme().text_primary),
                                // Progress bar
                                progress_row,
                                // Time display
                                row![
                                    text(fmt_time(*position_secs))
                                        .size(TEXT_SM)
                                        .color(theme().text_muted),
                                    iced::widget::Space::new().width(Length::Fill),
                                    text(fmt_time(*duration_secs))
                                        .size(TEXT_SM)
                                        .color(theme().text_dim),
                                ]
                                .width(Length::Fill),
                                // Controls
                                row![play_pause_btn, stop_btn]
                                    .spacing(SPACE_3)
                                    .align_y(iced::Alignment::Center),
                            ]
                            .spacing(20)
                            .align_x(iced::Alignment::Center)
                            .width(Length::Fill),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .padding(40)
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(theme().bg_editor)),
                            ..Default::default()
                        })
                        .into();
                    },
                }
            }
        }
        empty_editor()
    }

    pub(super) fn view_terminal_panel(&self) -> Element<'_, Message> {
        let height = Length::Fixed(self.terminal_panel_height);

        if let Some(term) = &self.terminal_pane {
            let header = container(
                row![
                    text("Terminal").size(TEXT_SM).color(theme().text_muted),
                    iced::widget::Space::new().width(Length::Fill),
                    button(text("x").size(TEXT_SM).color(theme().text_dim))
                        .style(tab_close_button_style)
                        .on_press(Message::ToggleTerminal),
                ]
                .align_y(iced::Alignment::Center),
            )
            .padding(iced::Padding {
                top: 6.0,
                right: SPACE_2,
                bottom: 6.0,
                left: 10.0,
            })
            .style(|_theme| container::Style {
                background: Some(Background::Color(theme().bg_secondary)),
                border: iced::Border {
                    color: theme().border_subtle,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            });

            let terminal_view =
                mouse_area(iced_term::TerminalView::show(term).map(Message::TerminalEvent))
                    .on_press(Message::FocusTerminal);

            let body =
                container(terminal_view)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(theme().bg_editor)),
                        ..Default::default()
                    });

            return container(column![header, body].spacing(0))
                .width(Length::Fill)
                .height(height)
                .into();
        }

        container(text("Embedded terminal unavailable").size(TEXT_SM).color(theme().text_dim))
            .padding(10)
            .width(Length::Fill)
            .height(height)
            .into()
    }

    /// Builds the `project › file › symbol` row shown under the tab bar.
    ///
    /// Returns `None` when breadcrumbs are turned off or the active tab is not
    /// a text editor. The symbol part appears once the language server has
    /// sent the document outline.
    pub(super) fn view_breadcrumbs(&self) -> Option<Element<'_, Message>> {
        if !self.editor_preferences.show_breadcrumbs {
            return None;
        }
        let tab = self.active_tab.and_then(|idx| self.tabs.get(idx))?;
        if !matches!(tab.kind, TabKind::Editor { .. }) {
            return None;
        }

        let root = self.file_tree.as_ref().map(|tree| tree.root.as_path());
        let path_crumbs = crate::features::breadcrumbs::path_segments(&tab.path, root);
        let cursor_line = u32::try_from(self.cursor_line.saturating_sub(1)).unwrap_or(u32::MAX);
        let symbol_crumbs = self
            .document_symbols
            .get(&crate::features::lsp::document_uri(&tab.path))
            .map(|symbols| iced_code_editor::symbol_path_at(symbols, cursor_line))
            .unwrap_or_default();

        let total = path_crumbs.len() + symbol_crumbs.len();
        let mut items: Vec<Element<'_, Message>> = Vec::with_capacity(total * 2);
        for (index, crumb) in path_crumbs.into_iter().chain(symbol_crumbs).enumerate() {
            if index > 0 {
                items.push(text("\u{203a}").size(TEXT_SM).color(theme().text_placeholder).into());
            }
            let color = if index + 1 == total {
                theme().text_secondary
            } else {
                theme().text_dim
            };
            items.push(text(crumb).size(TEXT_SM).color(color).into());
        }

        Some(
            container(row(items).spacing(6).align_y(iced::Alignment::Center))
                .padding(iced::Padding {
                    top: 3.0,
                    right: SPACE_3,
                    bottom: 3.0,
                    left: SPACE_3,
                })
                .width(Length::Fill)
                .clip(true)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(theme().bg_editor)),
                    ..container::Style::default()
                })
                .into(),
        )
    }

    pub(super) fn view_status_bar(&self) -> Element<'_, Message> {
        let file_info = self
            .active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.name.clone())
            .unwrap_or_default();

        let left = row![text(file_info).size(TEXT_SM).color(theme().text_dim),]
            .spacing(SPACE_2)
            .align_y(iced::Alignment::Center);

        let current_line_diag = self
            .active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.path.clone())
            .and_then(|path| self.lsp_diagnostics.get(&path))
            .and_then(|items| items.iter().find(|d| d.line == self.cursor_line))
            .and_then(|d| d.message.lines().next())
            .unwrap_or_default();

        let right = row![
            text(format!("Ln {}, Col {}", self.cursor_line, self.cursor_col))
                .size(TEXT_SM)
                .color(theme().text_placeholder),
            container(
                text(current_line_diag)
                    .size(TEXT_SM)
                    .color(theme().text_secondary)
                    .wrapping(iced::widget::text::Wrapping::None),
            )
            .max_width(360)
            .clip(true),
        ]
        .spacing(SPACE_2)
        .align_y(iced::Alignment::Center);

        container(
            row![left, iced::widget::Space::new().width(Length::Fill), right,]
                .align_y(iced::Alignment::Center),
        )
        .padding(iced::Padding {
            top: 6.0,
            right: SPACE_4,
            bottom: 7.0,
            left: SPACE_4,
        })
        .width(Length::Fill)
        .style(status_bar_style)
        .into()
    }

    pub(super) fn view_welcome_screen(&self) -> iced::widget::Container<'_, Message> {
        let folder_name = self
            .file_tree
            .as_ref()
            .map(|t| t.root.file_name().unwrap_or_default().to_string_lossy().to_string())
            .unwrap_or_else(|| String::from("No folder open"));

        container(
            column![
                text(folder_name).size(24).color(theme().text_muted),
                text("Select a file from the sidebar to begin editing")
                    .size(TEXT_MD)
                    .color(theme().text_placeholder),
            ]
            .spacing(SPACE_3)
            .align_x(iced::Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    }
}
