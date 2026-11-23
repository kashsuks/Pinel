use iced::widget::column;

use super::*;
use crate::features::icons::icon_handle;

impl App {
    pub(super) fn view_fuzzy_finder_overlay(&self) -> Element<'_, Message> {
        use iced::widget::{center, opaque, stack, Space};
        use syntect::{
            highlighting::{HighlightIterator, HighlightState, Highlighter as SyntectHighlighter},
            parsing::{ParseState, ScopeStack, SyntaxSet},
        };

        let input = text_input("Search files...", &self.fuzzy_finder.input)
            .id(self.fuzzy_finder.input_id.clone())
            .on_input(Message::FuzzyFinderQueryChanged)
            .size(TEXT_LG)
            .padding(iced::Padding {
                top: SPACE_4,
                right: 18.0,
                bottom: SPACE_4,
                left: 18.0,
            })
            .style(search_input_style)
            .width(Length::Fill);

        let folder_label: Element<'_, Message> = if let Some(folder) =
            &self.fuzzy_finder.current_folder
        {
            container(text(format!("{}", folder.display())).size(TEXT_XS).color(theme().text_muted))
                .padding(iced::Padding {
                    top: 0.0,
                    right: 18.0,
                    bottom: 0.0,
                    left: 18.0,
                })
                .into()
        } else {
            container(text("")).into()
        };

        let mut items: Vec<Element<'_, Message>> = Vec::new();

        if self.fuzzy_finder.filtered_files.is_empty() {
            items.push(
                container(text("No files found").size(TEXT_MD).color(theme().text_muted))
                    .padding(20)
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .into(),
            );
        } else {
            for (idx, file) in self.fuzzy_finder.filtered_files.iter().enumerate() {
                let is_selected = idx == self.fuzzy_finder.selected_index;
                let path = file.path.clone();

                let icon_asset = crate::features::icons::get_file_icon(
                    file.path.file_name().unwrap_or_default().to_str().unwrap_or(""),
                );
                let icon: Element<'_, Message> =
                    iced::widget::image::Image::new(icon_handle(icon_asset, 14))
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .into();

                items.push(
                    button(
                        row![
                            icon,
                            text(&file.display_name).size(TEXT_MD).color(if is_selected {
                                theme().text_primary
                            } else {
                                theme().text_muted
                            }),
                        ]
                        .spacing(SPACE_2)
                        .align_y(iced::Alignment::Center),
                    )
                    .style(file_finder_item_style(is_selected))
                    .on_press(Message::FileClicked(path))
                    .padding(iced::Padding {
                        top: 6.0,
                        right: 10.0,
                        bottom: 6.0,
                        left: 10.0,
                    })
                    .width(Length::Fill)
                    .into(),
                );
            }
        }

        let file_list = scrollable(column(items).spacing(2).padding(iced::Padding {
            top: SPACE_1,
            right: SPACE_1,
            bottom: SPACE_1,
            left: SPACE_1,
        }))
        .height(Length::Fill);

        let separator_v = container(Space::new())
            .width(Length::Fixed(1.0))
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(SURFACE_2)),
                ..Default::default()
            });

        let separator_h = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|_theme| container::Style {
                background: Some(Background::Color(SURFACE_2)),
                ..Default::default()
            });

        let preview: Element<'_, Message> = if let Some((preview_path, content)) =
            &self.fuzzy_finder.preview_cache
        {
            let ext = preview_path.extension().and_then(|e| e.to_str()).unwrap_or("");

            let syntax_set = SyntaxSet::load_defaults_newlines();
            let syntax = syntax_set
                .find_syntax_by_extension(ext)
                .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
            let current_theme = theme();
            let highlighter = SyntectHighlighter::new(&current_theme.syntax_theme);
            let mut parse_state = ParseState::new(syntax);
            let mut highlight_state = HighlightState::new(&highlighter, ScopeStack::new());

            let mut line_elements: Vec<Element<'_, Message>> = Vec::new();

            for (line_idx, line) in content.lines().enumerate().take(100) {
                let line_with_newline = format!("{}\n", line);
                let ops =
                    parse_state.parse_line(&line_with_newline, &syntax_set).unwrap_or_default();
                let ranges: Vec<_> = HighlightIterator::new(
                    &mut highlight_state,
                    &ops,
                    &line_with_newline,
                    &highlighter,
                )
                .collect();

                let line_num: Element<'_, Message> =
                    container(text(format!("{}", line_idx + 1)).size(TEXT_XS).color(OVERLAY_2))
                        .width(Length::Fixed(36.0))
                        .align_right(Length::Fixed(36.0))
                        .into();

                let mut spans: Vec<iced::widget::text::Span<'_, iced::Font>> = Vec::new();
                for (style, fragment) in &ranges {
                    let txt = fragment.strip_suffix('\n').unwrap_or(fragment);
                    if txt.is_empty() {
                        continue;
                    }
                    spans.push(
                        iced::widget::text::Span::new(txt.to_string())
                            .color(iced::Color::from_rgba8(
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                                style.foreground.a as f32 / 255.0,
                            ))
                            .size(self.editor_preferences.editor_font_size)
                            .font(self.editor_preferences.code_font()),
                    );
                }

                line_elements
                    .push(row![line_num, iced::widget::rich_text(spans)].spacing(SPACE_2).into());
            }

            let preview_header = container(
                text(preview_path.file_name().unwrap_or_default().to_string_lossy().to_string())
                    .size(TEXT_XS)
                    .color(theme().text_muted),
            )
            .padding(iced::Padding {
                top: SPACE_2,
                right: SPACE_3,
                bottom: 6.0,
                left: SPACE_3,
            });

            let preview_sep = container(Space::new())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(SURFACE_2)),
                    ..Default::default()
                });

            let preview_content =
                scrollable(column(line_elements).spacing(0).padding(iced::Padding {
                    top: SPACE_1,
                    right: SPACE_2,
                    bottom: SPACE_2,
                    left: SPACE_2,
                }))
                .height(Length::Fill);

            column![preview_header, preview_sep, preview_content]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            container(text("No preview available").size(TEXT_MD).color(theme().text_muted))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        };

        let left_panel = column![input, folder_label, separator_h, file_list]
            .width(Length::FillPortion(2))
            .height(Length::Fill);

        let right_panel = container(preview)
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(BG_BASE)),
                ..Default::default()
            });

        let overlay_box =
            container(row![left_panel, separator_v, right_panel].height(Length::Fill))
                .width(Length::Fixed(900.0))
                .height(Length::Fixed(520.0))
                .style(file_finder_panel_style);

        let backdrop = mouse_area(
            container(Space::new())
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                    ..Default::default()
                }),
        )
        .on_press(Message::ToggleFuzzyFinder);

        stack![backdrop, center(opaque(overlay_box))].into()
    }

    pub(super) fn view_file_finder_overlay(&self) -> Element<'_, Message> {
        use iced::widget::{center, opaque, stack, Space};

        let input = text_input("Go to file...", &self.file_finder_query)
            .id(self.file_finder_input_id.clone())
            .on_input(Message::FileFinderQueryChanged)
            .size(TEXT_LG)
            .padding(iced::Padding {
                top: SPACE_4,
                right: 18.0,
                bottom: SPACE_4,
                left: 18.0,
            })
            .style(search_input_style)
            .width(Length::Fill);

        let mut items: Vec<Element<'_, Message>> = Vec::new();

        if self.file_finder_query.is_empty() {
            if !self.recent_files.is_empty() {
                items.push(
                    container(text("Recent Files").size(TEXT_XS).color(theme().text_muted))
                        .padding(iced::Padding {
                            top: SPACE_2,
                            right: SPACE_2,
                            bottom: SPACE_1,
                            left: 14.0,
                        })
                        .into(),
                );
            }
            for (idx, path) in self.recent_files.iter().enumerate() {
                let is_selected = idx == self.file_finder_selected;
                let display = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let parent = path
                    .parent()
                    .and_then(|p| {
                        self.file_tree.as_ref().map(|t| {
                            p.strip_prefix(&t.root).unwrap_or(p).to_string_lossy().to_string()
                        })
                    })
                    .unwrap_or_default();

                let file_path = path.clone();
                items.push(
                    button(
                        row![
                            text(display).size(TEXT_MD).color(if is_selected {
                                theme().text_primary
                            } else {
                                theme().text_muted
                            }),
                            text(parent).size(TEXT_XS).color(theme().text_muted),
                        ]
                        .spacing(10)
                        .align_y(iced::Alignment::Center),
                    )
                    .style(file_finder_item_style(is_selected))
                    .on_press(Message::FileClicked(file_path))
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
        } else {
            for (idx, (_score, display, abs_path)) in self.file_finder_results.iter().enumerate() {
                let is_selected = idx == self.file_finder_selected;
                let path = abs_path.clone();
                items.push(
                    button(text(display).size(TEXT_MD).color(if is_selected {
                        theme().text_primary
                    } else {
                        theme().text_muted
                    }))
                    .style(file_finder_item_style(is_selected))
                    .on_press(Message::FileClicked(path))
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
            let results_column = scrollable(column(items).spacing(2).padding(iced::Padding {
                top: 6.0,
                right: 6.0,
                bottom: 6.0,
                left: 6.0,
            }))
            .height(Length::Shrink);
            column![input, separator, results_column].spacing(0).into()
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
        .on_press(Message::ToggleFileFinder);

        stack![backdrop, center(opaque(overlay_box))].into()
    }
}
