use std::time::Duration;

use iced::{
    advanced::widget::{
        operate,
        operation::{Outcome, Scrollable},
        Operation,
    },
    widget::{
        button,
        button::Status,
        canvas, column, container, image, mouse_area, row, scrollable,
        scrollable::{Direction, Scrollbar, Status as ScrollStatus},
        stack, text, tooltip, Space,
    },
    Alignment, Background, Color, Element, Font, Length, Point, Rectangle, Vector,
};

use super::{App, Tab, TabKind};
use crate::{
    features::{
        git::ChangeKind,
        icons::{get_file_icon, icon_handle},
        lsp::diagnostic_counts,
    },
    message::Message,
    theme::{theme, ICON_SIZE, RADIUS_SM, SPACE_2, SPACE_3, TEXT_MD, TEXT_SM, TEXT_XS},
    ui::change_color,
};

/// The height of the strip and floating drag preview.
pub(super) const TAB_HEIGHT: f32 = 38.0;
const TAB_GAP: f32 = 1.0;
const TAB_CLOSE_SIZE: f32 = 22.0;
const TAB_TITLE_SIZE: f32 = TEXT_MD;
const TAB_TOP_RADIUS: f32 = 8.0;
const TAB_JOIN_RADIUS: f32 = 6.0;
const TAB_JOIN_INSET: f32 = 2.0;
const TAB_BOTTOM_CLEARANCE: f32 = 2.0;
/// Shared thickness of the tab outlines and editor frame.
pub(super) const EDITOR_FRAME_WIDTH: f32 = 1.0;
const TAB_STRIP_ID: &str = "editor-tabs";

impl App {
    /// The width of a tab slot, including the spacing after it.
    pub(super) fn editor_tab_width(&self, index: usize) -> f32 {
        let tab = &self.tabs[index];
        let name_width: f32 = tab
            .name
            .chars()
            .map(|c| {
                if c.is_ascii() && !matches!(c, 'M' | 'W' | 'm' | 'w' | '@' | '%') {
                    TAB_TITLE_SIZE * 0.65
                } else {
                    TAB_TITLE_SIZE
                }
            })
            .sum();
        let (errors, warnings) = self.tab_problem_counts(tab);
        let problem_width = [errors, warnings]
            .into_iter()
            .filter(|&count| count > 0)
            .map(|count| {
                SPACE_2
                    + if count > 9 {
                        17.0
                    } else {
                        9.0
                    }
            })
            .sum::<f32>();
        let git_width = if self.tab_git_change(tab).is_some() {
            SPACE_2 + 10.0
        } else {
            0.0
        };
        let chrome_width = TAB_GAP + SPACE_3 * 2.0 + ICON_SIZE + TAB_CLOSE_SIZE + SPACE_2 * 2.0;
        (chrome_width + name_width + problem_width + git_width).clamp(128.0, 320.0)
    }

    /// Brings the selected tab fully into view when the strip has room for it.
    pub(super) fn reveal_editor_tab(&self, index: usize) -> iced::Task<Message> {
        let start = (0..index).map(|slot| self.editor_tab_width(slot)).sum::<f32>();
        let tab_width = self.editor_tab_width(index);
        scroll_tab_bar(move |current, width| {
            let end = start + tab_width.min(width);
            current.clamp(end - width, start)
        })
    }

    pub(super) fn view_tab_bar(&self) -> Element<'_, Message> {
        let floating = self.editor_preferences.tab_drag_floating;
        let drag = self.tab_drag_index.zip(self.tab_drag_target);
        let mut order: Vec<_> = (0..self.tabs.len()).collect();
        if let Some((from, to)) = drag.filter(|_| !floating) {
            order.remove(from);
            order.insert(to, from);
        }

        let tabs = order.into_iter().map(|index| {
            let mut tab = if floating && self.tab_drag_index == Some(index) {
                let placeholder = container(Space::new())
                    .width(self.editor_tab_width(index) - TAB_GAP)
                    .height(TAB_HEIGHT - TAB_BOTTOM_CLEARANCE)
                    .style(|_| container::Style {
                        background: Some(theme().bg_hover.into()),
                        border: iced::Border {
                            radius: iced::border::top(TAB_TOP_RADIUS).bottom(TAB_JOIN_RADIUS),
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                container(placeholder).width(self.editor_tab_width(index)).into()
            } else {
                self.view_editor_tab(index, false)
            };

            if let Some((from, to)) =
                drag.filter(|&(from, to)| floating && from != to && to == index)
            {
                let edge = if to < from {
                    Alignment::Start
                } else {
                    Alignment::End
                };
                tab = stack![
                        tab,
                        container(container(Space::new()).width(2).height(Length::Fill).style(
                            |_| container::Style::default().background(theme().text_primary)
                        ),)
                        .width(Length::Fill)
                        .align_x(edge),
                    ]
                .into();
            }
            tab
        });

        let strip = scrollable(row(tabs).height(TAB_HEIGHT))
            .id(iced::widget::Id::new(TAB_STRIP_ID))
            .direction(Direction::Horizontal(
                Scrollbar::new().width(1).scroller_width(1).margin(EDITOR_FRAME_WIDTH),
            ))
            .width(Length::Fill)
            .height(TAB_HEIGHT)
            .style(|iced_theme, status| {
                let mut style = iced::widget::scrollable::default(iced_theme, status);
                style.horizontal_rail.background = None;
                let mut thumb = theme().text_muted;
                thumb.a = match status {
                    ScrollStatus::Active { .. } => 0.24,
                    _ => 0.52,
                };
                style.horizontal_rail.scroller.background = thumb.into();
                style.horizontal_rail.scroller.border.radius = 2.0.into();
                style
            })
            .on_scroll(|viewport| Message::TabBarScrolled(viewport.absolute_offset().x));

        let strip = mouse_area(strip).on_scroll(|delta| {
            use iced::mouse::ScrollDelta;
            let x = match delta {
                ScrollDelta::Lines { y, .. } => -y * 60.0,
                ScrollDelta::Pixels { y, .. } => -y,
            };
            Message::TabBarWheel(x)
        });
        let seam = container(
            container(Space::new())
                .width(Length::Fill)
                .height(EDITOR_FRAME_WIDTH)
                .style(|_| container::Style::default().background(theme().border_subtle)),
        )
        .width(Length::Fill)
        .height(TAB_HEIGHT)
        .align_y(Alignment::End)
        .style(|_| container::Style::default().background(theme().bg_tab_bar));

        stack![seam, strip].into()
    }

    pub(super) fn view_editor_tab(&self, index: usize, floating: bool) -> Element<'_, Message> {
        let tab = &self.tabs[index];
        let active = floating || self.active_tab == Some(index);
        let hovered = self.tab_hover_index == Some(index);
        let dragging = self.tab_drag_index == Some(index);
        let modified =
            matches!(&tab.kind, TabKind::Editor { code_editor, .. } if code_editor.is_modified());
        let preview = matches!(tab.kind, TabKind::Commit { preview: true, .. });
        let (errors, warnings) = self.tab_problem_counts(tab);
        let git = self.tab_git_change(tab);
        let (error_color, warning_color) = tab_problem_colors();
        let title_color = if errors > 0 {
            error_color
        } else if warnings > 0 {
            warning_color
        } else if let Some(kind) = git {
            change_color(kind)
        } else if active || dragging {
            theme().text_primary
        } else {
            theme().text_secondary
        };

        let title = text(&tab.name)
            .size(TAB_TITLE_SIZE)
            .color(title_color)
            .shaping(iced::widget::text::Shaping::Advanced)
            .wrapping(iced::widget::text::Wrapping::None)
            .font(Font {
                family: iced::font::Family::Name("SF Pro"),
                weight: if active {
                    iced::font::Weight::Semibold
                } else {
                    iced::font::Weight::Medium
                },
                style: if preview {
                    iced::font::Style::Italic
                } else {
                    iced::font::Style::Normal
                },
                ..Font::DEFAULT
            });
        let mut label = column![title];
        if let Some(parent) = self.tab_parent_label(tab) {
            label = label.push(
                text(parent)
                    .size(TEXT_XS)
                    .color(theme().text_muted)
                    .wrapping(iced::widget::text::Wrapping::None),
            );
        }

        let icon = image::Image::new(icon_handle(get_file_icon(&tab.name), 32))
            .width(ICON_SIZE)
            .height(ICON_SIZE);
        let mut content = row![icon, container(label).width(Length::Fill).clip(true)]
            .spacing(SPACE_2)
            .align_y(Alignment::Center);
        if errors > 0 {
            content = content.push(text(badge_count(errors)).size(TEXT_SM).color(error_color));
        }
        if warnings > 0 {
            content = content.push(text(badge_count(warnings)).size(TEXT_SM).color(warning_color));
        }
        if let Some(kind) = git {
            content = content.push(text(kind.letter()).size(TEXT_SM).color(change_color(kind)));
        }

        let close_icon = if modified && (!hovered || floating) {
            "●"
        } else if active || hovered {
            "×"
        } else {
            ""
        };
        let close = button(text(close_icon).size(TEXT_MD).center())
            .width(TAB_CLOSE_SIZE)
            .height(TAB_CLOSE_SIZE)
            .padding(0)
            .style(|_, status| iced::widget::button::Style {
                background: matches!(status, Status::Hovered | Status::Pressed)
                    .then(|| Background::Color(theme().bg_pressed)),
                text_color: theme().text_secondary,
                border: iced::Border {
                    radius: RADIUS_SM.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .on_press_maybe((!floating).then_some(Message::TabClosed(index)));

        let joined = active && !floating;
        let face_width = self.editor_tab_width(index) - TAB_GAP;
        let bottom_clearance = if joined || floating {
            0.0
        } else {
            TAB_BOTTOM_CLEARANCE
        };
        let face = container(content.push(close))
            .width(face_width)
            .height(TAB_HEIGHT - bottom_clearance)
            .align_y(Alignment::Center)
            .padding(iced::Padding {
                top: bottom_clearance,
                right: SPACE_3,
                bottom: 0.0,
                left: SPACE_3,
            })
            .style(move |_| {
                if joined {
                    return container::Style::default();
                }
                let colors = theme();
                let background = if floating || dragging {
                    colors.bg_tab_active
                } else if hovered {
                    colors.bg_hover
                } else {
                    colors.bg_tab_inactive
                };
                container::Style {
                    background: Some(background.into()),
                    border: iced::Border {
                        radius: iced::border::top(TAB_TOP_RADIUS).bottom(TAB_JOIN_RADIUS),
                        width: EDITOR_FRAME_WIDTH,
                        color: if active {
                            colors.border_subtle
                        } else {
                            colors.border_very_subtle
                        },
                    },
                    ..Default::default()
                }
            });
        let face: Element<'_, Message> = if joined {
            stack![canvas(JoinedTab).width(face_width).height(TAB_HEIGHT), face,].into()
        } else {
            face.into()
        };
        let tab_face = container(face)
            .padding(iced::padding::right(TAB_GAP))
            .width(self.editor_tab_width(index))
            .height(TAB_HEIGHT);

        if floating {
            return tab_face.into();
        }

        let mut details = if matches!(tab.kind, TabKind::Commit { .. }) {
            tab.name.clone()
        } else {
            tab.path.display().to_string()
        };
        if let Some(kind) = git {
            details.push_str(&format!("\nGit · {}", kind.label()));
        }
        if errors > 0 || warnings > 0 {
            details.push_str(&format!("\nErrors {errors} · Warnings {warnings}"));
        }
        if modified {
            details.push_str("\nUnsaved changes");
        }

        tooltip(
            mouse_area(tab_face)
                .on_press(Message::TabPointerPressed(index))
                .interaction(iced::mouse::Interaction::Pointer)
                .on_middle_press(Message::TabClosed(index))
                .on_enter(Message::TabHoverEnter(index))
                .on_move(move |_| Message::TabHoverEnter(index))
                .on_exit(Message::TabHoverExit(index)),
            container(text(details).size(TEXT_SM).color(theme().text_primary)).max_width(480),
            tooltip::Position::Bottom,
        )
        .delay(Duration::from_millis(600))
        .style(|_| container::Style {
            background: Some(theme().bg_secondary.into()),
            border: iced::Border {
                width: 1.0,
                color: theme().border_subtle,
                radius: RADIUS_SM.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn tab_parent_label(&self, tab: &Tab) -> Option<String> {
        if !tab.path.is_absolute() {
            return None;
        }
        let folder = tab.path.parent()?;
        let other_folders: Vec<_> = self
            .tabs
            .iter()
            .filter(|other| other.path != tab.path && other.name == tab.name)
            .filter_map(|other| other.path.parent())
            .collect();
        if other_folders.is_empty() {
            return None;
        }
        let suffix = folder
            .ancestors()
            .skip(1)
            .filter_map(|prefix| folder.strip_prefix(prefix).ok())
            .find(|suffix| other_folders.iter().all(|other| !other.ends_with(suffix)));
        Some(suffix.unwrap_or(folder).display().to_string())
    }

    fn tab_git_change(&self, tab: &Tab) -> Option<ChangeKind> {
        self.git_tab_changes.get(&tab.path).copied().or_else(|| {
            // Git reports an entirely untracked directory as one entry.
            tab.path.ancestors().skip(1).find_map(|path| {
                self.git_tab_changes
                    .get(path)
                    .copied()
                    .filter(|kind| *kind == ChangeKind::Untracked)
            })
        })
    }

    fn tab_problem_counts(&self, tab: &Tab) -> (usize, usize) {
        self.lsp_diagnostics
            .get(&tab.path)
            .map_or((0, 0), |diagnostics| diagnostic_counts(diagnostics))
    }
}

struct JoinedTab;

impl canvas::Program<Message> for JoinedTab {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let colors = theme();
        let radius = TAB_JOIN_RADIUS;
        let inset = EDITOR_FRAME_WIDTH / 2.0;
        let seam = bounds.height - inset;
        let left = TAB_JOIN_INSET + inset;
        let right = bounds.width - left;
        let shoulder = TAB_TOP_RADIUS;
        let outline = canvas::Path::new(|path| {
            path.move_to(Point::new(inset, seam));
            path.quadratic_curve_to(Point::new(left, seam), Point::new(left, seam - radius));
            path.line_to(Point::new(left, inset + shoulder));
            path.quadratic_curve_to(Point::new(left, inset), Point::new(left + shoulder, inset));
            path.line_to(Point::new(right - shoulder, inset));
            path.quadratic_curve_to(
                Point::new(right, inset),
                Point::new(right, inset + shoulder),
            );
            path.line_to(Point::new(right, seam - radius));
            path.quadratic_curve_to(
                Point::new(right, seam),
                Point::new(bounds.width - inset, seam),
            );
        });

        frame.fill(&outline, colors.bg_editor);
        frame.fill_rectangle(
            Point::new(inset, bounds.height - EDITOR_FRAME_WIDTH),
            iced::Size::new(bounds.width - EDITOR_FRAME_WIDTH, EDITOR_FRAME_WIDTH),
            colors.bg_editor,
        );
        frame.stroke(
            &outline,
            canvas::Stroke::default()
                .with_width(EDITOR_FRAME_WIDTH)
                .with_color(colors.border_subtle),
        );
        vec![frame.into_geometry()]
    }
}

fn badge_count(count: usize) -> String {
    if count > 9 {
        "9+".into()
    } else {
        count.to_string()
    }
}

fn tab_problem_colors() -> (Color, Color) {
    let bg = theme().bg_tab_inactive;
    if bg.r * 0.2126 + bg.g * 0.7152 + bg.b * 0.0722 > 0.5 {
        (Color::from_rgb8(158, 35, 49), Color::from_rgb8(125, 81, 12))
    } else {
        (
            Color::from_rgb8(255, 132, 122),
            Color::from_rgb8(243, 192, 105),
        )
    }
}

/// Scrolls by the vertical wheel's horizontal equivalent.
pub(super) fn scroll_editor_tabs_by(delta: f32) -> iced::Task<Message> {
    scroll_tab_bar(move |current, _| current + delta)
}

fn scroll_tab_bar(offset: impl Fn(f32, f32) -> f32 + Send + 'static) -> iced::Task<Message> {
    struct ScrollTabs<F> {
        offset: F,
        result: Option<f32>,
    }

    impl<F: Fn(f32, f32) -> f32 + Send> Operation<f32> for ScrollTabs<F> {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<f32>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&iced::widget::Id>,
            bounds: Rectangle,
            content: Rectangle,
            translation: Vector,
            state: &mut dyn Scrollable,
        ) {
            if id != Some(&iced::widget::Id::new(TAB_STRIP_ID)) {
                return;
            }

            let offset = (self.offset)(translation.x, bounds.width)
                .clamp(0.0, (content.width - bounds.width).max(0.0));
            self.result = Some(offset);
            if offset != translation.x {
                state.scroll_to(iced::widget::operation::AbsoluteOffset {
                    x: Some(offset),
                    y: None,
                });
            }
        }

        fn finish(&self) -> Outcome<f32> {
            self.result.map_or(Outcome::None, Outcome::Some)
        }
    }

    operate(ScrollTabs {
        offset,
        result: None,
    })
    .map(Message::TabBarScrolled)
}
