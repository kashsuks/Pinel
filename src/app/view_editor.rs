use frostmark::MarkWidget;
use iced::widget::column;

use super::*;
use crate::{features::lsp::diagnostic_counts, ui::git_graph::truncate_subject};

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
                                .size(TEXT_XS)
                                .color(theme().text_muted),
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
                                            text(&suggestion.text).size(TEXT_MD).color(label_color),
                                            iced::widget::Space::new().width(Length::Fill),
                                            text(format!("{:?}", suggestion.kind).to_lowercase())
                                                .size(TEXT_XS)
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
                                            text("↑↓").size(TEXT_XS).color(accent_purple),
                                            text(" navigate · ")
                                                .size(TEXT_XS)
                                                .color(theme().text_muted),
                                            text("↵").size(TEXT_XS).color(accent_purple),
                                            text(" accept · ")
                                                .size(TEXT_XS)
                                                .color(theme().text_muted),
                                            text("esc").size(TEXT_XS).color(accent_purple),
                                            text(" dismiss")
                                                .size(TEXT_XS)
                                                .color(theme().text_muted),
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
                                .max_width(420.0)
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
                            let y = (cursor_pos.y + code_editor.line_height()).clamp(0.0, 560.0);

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
                        return container(
                            scrollable(
                                markdown::view(
                                    md_items,
                                    markdown::Settings::with_style(markdown::Style::from_palette(
                                        iced::theme::Palette::CATPPUCCIN_MOCHA,
                                    )),
                                )
                                .map(Message::MarkdownLinkClicked),
                            )
                            .height(Length::Fill),
                        )
                        .padding(24)
                        .height(Length::Fill)
                        // Markdown body text inherits the renderer color, not its palette.
                        .style(|_| container::Style {
                            text_color: Some(theme().text_primary),
                            ..Default::default()
                        })
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
                                        .color(theme().text_muted),
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
                    button(text("x").size(TEXT_SM).color(theme().text_muted))
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

        container(text("Embedded terminal unavailable").size(TEXT_SM).color(theme().text_muted))
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
                items.push(text("\u{203a}").size(TEXT_SM).color(theme().text_muted).into());
            }
            let color = if index + 1 == total {
                theme().text_secondary
            } else {
                theme().text_muted
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

    /// Builds the bar along the bottom of the window.
    ///
    /// Left: the Vim mode pill, or the open Vim command line in its place;
    /// the git branch; and the active file's error and warning count. Right:
    /// the current line's diagnostic, pending Vim keys, the cursor position,
    /// indentation, language and language server. Only the branch shows
    /// unless a text editor tab is active.
    pub(super) fn view_status_bar(&self) -> Element<'_, Message> {
        // One step brighter than the muted grey used for hints elsewhere, so
        // the bar is easy to read on both dark and light themes.
        let item_color = theme().text_secondary;
        let mut left: Vec<Element<'_, Message>> = Vec::new();
        let mut right: Vec<Element<'_, Message>> = Vec::new();
        let mut current_line_message = String::new();

        let active_editor =
            self.active_tab
                .and_then(|idx| self.tabs.get(idx))
                .and_then(|tab| match &tab.kind {
                    TabKind::Editor { code_editor, .. } => Some((tab, code_editor)),
                    _ => None,
                });

        if let Some((_, code_editor)) = active_editor {
            if let Some(command_line) = code_editor.vim_command_line() {
                let command_line = truncate_subject(&command_line, STATUS_COMMAND_LINE_MAX_CHARS);
                left.push(status_text(command_line, theme().text_primary).font(MONO_FONT).into());
            } else if let Some(mode) = code_editor.vim_mode() {
                left.push(vim_mode_pill(mode));
            }
        }

        if let Some(branch) = &self.git_branch {
            let branch = truncate_subject(branch, STATUS_BRANCH_MAX_CHARS);
            left.push(status_text(format!("\u{2387} {branch}"), item_color).into());
        }

        if let Some((tab, code_editor)) = active_editor {
            let diagnostics = self.lsp_diagnostics.get(&tab.path).map(Vec::as_slice).unwrap_or(&[]);
            let (errors, warnings) = diagnostic_counts(diagnostics);
            if let Some(summary) = diagnostic_summary(errors, warnings) {
                left.push(status_text(summary, item_color).into());
            }

            // Read from the editor itself: the app's own copy of the cursor
            // does not follow Vim motions.
            let (line, col) = code_editor.cursor_position();
            let (line, col) = (line + 1, col + 1);
            if let Some(diagnostic) = diagnostics.iter().find(|d| d.line == line) {
                current_line_message = diagnostic_preview(&diagnostic.message);
            }

            if let Some(keys) = code_editor.vim_pending_keys().filter(|keys| !keys.is_empty()) {
                let keys = truncate_subject(&keys, STATUS_PENDING_KEYS_MAX_CHARS);
                right.push(status_text(keys, item_color).font(MONO_FONT).into());
            }
            right.push(status_text(format!("Ln {line}, Col {col}"), item_color).into());
            right.push(
                status_text(indentation_label(code_editor.indent_style()), item_color).into(),
            );
            if let Some(language) = language_display_name(&tab.path) {
                right.push(status_text(language, item_color).into());
            }
            if self.lsp_enabled {
                if let Some(server) = self.lsp_server_keys.get(&tab.path) {
                    right.push(status_text(*server, item_color).into());
                }
            }
        }

        // The only `Fill` item, so when the bar runs out of room this is the
        // first thing squeezed away; the bar's clip hides anything past that.
        let message = container(status_text(current_line_message, theme().text_primary))
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Right)
            .clip(true);

        container(
            row![
                row(left).spacing(SPACE_3).align_y(iced::Alignment::Center),
                message,
                row(right).spacing(SPACE_3).align_y(iced::Alignment::Center),
            ]
            .spacing(SPACE_3)
            .align_y(iced::Alignment::Center),
        )
        .padding([0.0, SPACE_4])
        .width(Length::Fill)
        .center_y(Length::Fixed(STATUS_BAR_HEIGHT))
        .clip(true)
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
                    .color(theme().text_muted),
            ]
            .spacing(SPACE_3)
            .align_x(iced::Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    }
}

/// Size of the status bar's text: the 15px body size, so the bar reads at a
/// glance rather than at the 12px used for other secondary text.
const STATUS_TEXT_SIZE: f32 = TEXT_MD;

/// Size of the Vim mode pill's label, one step below the bar's text so the
/// pill sits inside the bar with room around it.
const VIM_PILL_TEXT_SIZE: f32 = TEXT_SM;

/// Vertical padding inside the Vim mode pill, above and below its label.
const VIM_PILL_PADDING_Y: f32 = 2.0;

/// How far the pill's label is moved down from the middle of the pill.
///
/// iced's line box keeps room for descenders under the baseline, and the
/// labels are all capitals, which have none, so the letters sit high in the
/// box. Measured on screen at [`VIM_PILL_TEXT_SIZE`] (14px), the capitals
/// were 2px above the pill's centre; re-measure if that size changes.
const VIM_PILL_LABEL_NUDGE: f32 = 2.0;

// The nudge comes out of the bottom padding, which must not go negative.
const _: () = assert!(VIM_PILL_LABEL_NUDGE <= VIM_PILL_PADDING_Y);

/// Fixed height of the status bar: `SPACE_2` above and below one line of
/// [`STATUS_TEXT_SIZE`] text, which iced lays out at its default line height
/// of 1.3 times the font size (15 × 1.3 = 19.5px). 35.5px in all, and the bar
/// cannot grow with its contents.
const STATUS_BAR_HEIGHT: f32 = SPACE_2 + STATUS_TEXT_SIZE * 1.3 + SPACE_2;

// The Vim mode pill is one label line plus its padding, so it has to fit
// inside the bar without making it taller.
const _: () = assert!(VIM_PILL_TEXT_SIZE * 1.3 + 2.0 * VIM_PILL_PADDING_Y < STATUS_BAR_HEIGHT);

/// Fira Code's advance width as a fraction of the font size: every glyph is
/// 1200 units wide on a 1950-unit em, in both the regular and the bold cut.
const FIRA_CODE_ADVANCE_EM: f32 = 1200.0 / 1950.0;

/// Width of the Vim mode pill: the longest label, `VISUAL LINE` (11
/// characters), in Fira Code at [`VIM_PILL_TEXT_SIZE`] (about 81.2px), plus
/// `SPACE_2` on each side, about 97.2px. Every mode uses this width, so
/// switching modes never moves the items after the pill.
const VIM_PILL_WIDTH: f32 = 11.0 * FIRA_CODE_ADVANCE_EM * VIM_PILL_TEXT_SIZE + 2.0 * SPACE_2;

/// Text on the Vim mode pill. The accent colours behind it are pastel on
/// every theme, so one dark colour (Catppuccin base) reads on all of them.
const VIM_PILL_TEXT: Color = BG_BASE;

/// Fira Code, for the status bar's Vim command line and pending keys.
const MONO_FONT: iced::Font = iced::Font {
    family: iced::font::Family::Name("Fira Code"),
    ..iced::Font::DEFAULT
};

/// Bold Fira Code, for the Vim mode pill.
const MONO_BOLD_FONT: iced::Font = iced::Font {
    weight: iced::font::Weight::Bold,
    ..MONO_FONT
};

/// Longest branch name the status bar shows before cutting it short.
const STATUS_BRANCH_MAX_CHARS: usize = 32;
/// Longest Vim command line the status bar shows before cutting it short.
const STATUS_COMMAND_LINE_MAX_CHARS: usize = 80;
/// Longest diagnostic message the status bar shows before cutting it short.
const STATUS_DIAGNOSTIC_MAX_CHARS: usize = 80;
/// Longest run of pending Vim keys the status bar shows before cutting it
/// short. Counts can be typed with any number of digits.
const STATUS_PENDING_KEYS_MAX_CHARS: usize = 16;

/// One item of status bar text: [`STATUS_TEXT_SIZE`], kept to a single line.
fn status_text<'a>(
    content: impl iced::widget::text::IntoFragment<'a>,
    color: Color,
) -> iced::widget::Text<'a> {
    text(content)
        .size(STATUS_TEXT_SIZE)
        .color(color)
        .wrapping(iced::widget::text::Wrapping::None)
}

/// The Vim mode pill: the mode's label, centred in a fixed-width capsule of
/// the mode's colour.
fn vim_mode_pill<'a>(mode: iced_code_editor::VimMode) -> Element<'a, Message> {
    let background = vim_mode_color(mode);
    container(
        text(vim_mode_label(mode))
            .size(VIM_PILL_TEXT_SIZE)
            .font(MONO_BOLD_FONT)
            .color(VIM_PILL_TEXT)
            .wrapping(iced::widget::text::Wrapping::None),
    )
    // Same total height as even padding, with the label moved down so the
    // capitals sit in the middle of the pill.
    .padding(iced::Padding {
        top: VIM_PILL_PADDING_Y + VIM_PILL_LABEL_NUDGE,
        right: 0.0,
        bottom: VIM_PILL_PADDING_Y - VIM_PILL_LABEL_NUDGE,
        left: 0.0,
    })
    .center_x(Length::Fixed(VIM_PILL_WIDTH))
    .clip(true)
    .style(move |_theme| container::Style {
        background: Some(Background::Color(background)),
        border: iced::Border {
            radius: RADIUS_SM.into(),
            ..iced::Border::default()
        },
        ..container::Style::default()
    })
    .into()
}

/// The label on the Vim mode pill, matching the editor's own status line.
fn vim_mode_label(mode: iced_code_editor::VimMode) -> &'static str {
    match mode {
        iced_code_editor::VimMode::Normal => "NORMAL",
        iced_code_editor::VimMode::Insert => "INSERT",
        iced_code_editor::VimMode::Visual => "VISUAL",
        iced_code_editor::VimMode::VisualLine => "VISUAL LINE",
    }
}

/// The colour of the Vim mode pill: blue for Normal, green for Insert and
/// purple for both Visual modes.
fn vim_mode_color(mode: iced_code_editor::VimMode) -> Color {
    match mode {
        iced_code_editor::VimMode::Normal => ACCENT_BLUE,
        iced_code_editor::VimMode::Insert => ACCENT_GREEN,
        iced_code_editor::VimMode::Visual | iced_code_editor::VimMode::VisualLine => ACCENT_PURPLE,
    }
}

/// The indentation the editor inserts, as the status bar shows it.
fn indentation_label(style: iced_code_editor::IndentStyle) -> String {
    match style {
        iced_code_editor::IndentStyle::Spaces(width) => format!("Spaces: {width}"),
        iced_code_editor::IndentStyle::Tab => "Tabs".to_string(),
    }
}

/// The name of a file's language, from its extension in any letter case.
///
/// `None` for extensions not listed here and for files without one; the
/// status bar then shows nothing rather than a bare extension.
fn language_display_name(path: &std::path::Path) -> Option<&'static str> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let name = match extension.as_str() {
        "rs" => "Rust",
        "py" => "Python",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" | "mjs" | "cjs" => "JavaScript",
        "go" => "Go",
        "lua" => "Lua",
        "md" => "Markdown",
        "toml" => "TOML",
        "json" => "JSON",
        "yaml" | "yml" => "YAML",
        "sh" | "bash" | "zsh" => "Shell",
        "c" | "h" => "C",
        "cpp" | "cc" | "hpp" => "C++",
        "html" => "HTML",
        "css" => "CSS",
        _ => return None,
    };
    Some(name)
}

/// The part of a diagnostic message the status bar shows: its first line,
/// cut to [`STATUS_DIAGNOSTIC_MAX_CHARS`].
fn diagnostic_preview(message: &str) -> String {
    truncate_subject(
        message.lines().next().unwrap_or(""),
        STATUS_DIAGNOSTIC_MAX_CHARS,
    )
}

/// The error and warning count as the status bar shows it, such as
/// `2 errors, 1 warning`. `None` when there are neither.
fn diagnostic_summary(errors: usize, warnings: usize) -> Option<String> {
    let plural = |count: usize, noun: &str| {
        if count == 1 {
            format!("1 {noun}")
        } else {
            format!("{count} {noun}s")
        }
    };
    match (errors, warnings) {
        (0, 0) => None,
        (errors, 0) => Some(plural(errors, "error")),
        (0, warnings) => Some(plural(warnings, "warning")),
        (errors, warnings) => Some(format!(
            "{}, {}",
            plural(errors, "error"),
            plural(warnings, "warning")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::lsp::InlineDiagnostic;
    use iced_code_editor::{IndentStyle, VimMode};
    use std::path::Path;

    const ALL_MODES: [VimMode; 4] = [
        VimMode::Normal,
        VimMode::Insert,
        VimMode::Visual,
        VimMode::VisualLine,
    ];

    fn diagnostic(severity: lsp_types::DiagnosticSeverity) -> InlineDiagnostic {
        InlineDiagnostic {
            line: 1,
            severity,
            message: String::new(),
        }
    }

    #[test]
    fn test_vim_mode_label_matches_editor_labels() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(vim_mode_label(VimMode::Normal), "NORMAL");
        assert_eq!(vim_mode_label(VimMode::Insert), "INSERT");
        assert_eq!(vim_mode_label(VimMode::Visual), "VISUAL");
        assert_eq!(vim_mode_label(VimMode::VisualLine), "VISUAL LINE");
        Ok(())
    }

    #[test]
    fn test_vim_mode_label_fits_pill_width() -> Result<(), Box<dyn std::error::Error>> {
        for mode in ALL_MODES {
            let label = vim_mode_label(mode);
            let width = label.chars().count() as f32 * FIRA_CODE_ADVANCE_EM * VIM_PILL_TEXT_SIZE;
            assert!(
                width + 2.0 * SPACE_2 <= VIM_PILL_WIDTH + f32::EPSILON,
                "{label} needs {width}px"
            );
        }
        Ok(())
    }

    #[test]
    fn test_vim_mode_color_visual_modes_share_purple() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(vim_mode_color(VimMode::Normal), ACCENT_BLUE);
        assert_eq!(vim_mode_color(VimMode::Insert), ACCENT_GREEN);
        assert_eq!(vim_mode_color(VimMode::Visual), ACCENT_PURPLE);
        assert_eq!(vim_mode_color(VimMode::VisualLine), ACCENT_PURPLE);
        Ok(())
    }

    #[test]
    fn test_indentation_label_spaces_and_tabs() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(indentation_label(IndentStyle::Spaces(4)), "Spaces: 4");
        assert_eq!(indentation_label(IndentStyle::Spaces(2)), "Spaces: 2");
        assert_eq!(indentation_label(IndentStyle::Tab), "Tabs");
        Ok(())
    }

    #[test]
    fn test_language_display_name_known_extensions() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            language_display_name(Path::new("src/main.rs")),
            Some("Rust")
        );
        assert_eq!(
            language_display_name(Path::new("app.tsx")),
            Some("TypeScript")
        );
        assert_eq!(
            language_display_name(Path::new("index.mjs")),
            Some("JavaScript")
        );
        assert_eq!(language_display_name(Path::new("ci.yml")), Some("YAML"));
        assert_eq!(language_display_name(Path::new("lib.h")), Some("C"));
        assert_eq!(language_display_name(Path::new("lib.hpp")), Some("C++"));
        Ok(())
    }

    #[test]
    fn test_language_display_name_ignores_case() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            language_display_name(Path::new("README.MD")),
            Some("Markdown")
        );
        assert_eq!(language_display_name(Path::new("Main.Rs")), Some("Rust"));
        Ok(())
    }

    #[test]
    fn test_language_display_name_unknown_extension() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(language_display_name(Path::new("photo.xyz")), None);
        Ok(())
    }

    #[test]
    fn test_language_display_name_no_extension() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(language_display_name(Path::new("Makefile")), None);
        // a leading dot names the file; it is not an extension
        assert_eq!(language_display_name(Path::new(".bashrc")), None);
        assert_eq!(language_display_name(Path::new("untitled")), None);
        Ok(())
    }

    #[test]
    fn test_diagnostic_preview_first_line_only() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            diagnostic_preview("mismatched types\nexpected `u8`, found `i32`"),
            "mismatched types"
        );
        assert_eq!(diagnostic_preview(""), "");
        Ok(())
    }

    #[test]
    fn test_diagnostic_preview_long_line_is_cut() -> Result<(), Box<dyn std::error::Error>> {
        let preview = diagnostic_preview(&"x".repeat(200));
        assert_eq!(preview.chars().count(), STATUS_DIAGNOSTIC_MAX_CHARS);
        assert!(preview.ends_with('\u{2026}'));
        Ok(())
    }

    #[test]
    fn test_diagnostic_counts_errors_and_warnings() -> Result<(), Box<dyn std::error::Error>> {
        let diagnostics = [
            diagnostic(lsp_types::DiagnosticSeverity::ERROR),
            diagnostic(lsp_types::DiagnosticSeverity::WARNING),
            diagnostic(lsp_types::DiagnosticSeverity::ERROR),
            diagnostic(lsp_types::DiagnosticSeverity::HINT),
            diagnostic(lsp_types::DiagnosticSeverity::INFORMATION),
        ];
        assert_eq!(diagnostic_counts(&diagnostics), (2, 1));
        Ok(())
    }

    #[test]
    fn test_diagnostic_counts_empty() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(diagnostic_counts(&[]), (0, 0));
        Ok(())
    }

    #[test]
    fn test_diagnostic_summary_hidden_when_zero() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(diagnostic_summary(0, 0), None);
        Ok(())
    }

    #[test]
    fn test_diagnostic_summary_singular_and_plural() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(diagnostic_summary(1, 0).as_deref(), Some("1 error"));
        assert_eq!(diagnostic_summary(0, 3).as_deref(), Some("3 warnings"));
        assert_eq!(
            diagnostic_summary(2, 1).as_deref(),
            Some("2 errors, 1 warning")
        );
        Ok(())
    }

    #[test]
    fn test_status_bar_height_is_one_line_plus_padding() -> Result<(), Box<dyn std::error::Error>> {
        // 8px + one 15px line at iced's 1.3 line height + 8px
        assert!((STATUS_BAR_HEIGHT - 35.5).abs() < 0.01);
        Ok(())
    }
}
