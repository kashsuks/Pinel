use iced::keyboard::{key, Key};
use iced::widget::text_editor::{Binding, Content, KeyPress, Motion, TextEditor};
use iced::widget::{column, container, row, text};
use iced::{Background, Border, Element, Length};

use crate::features::lsp::InlineDiagnostic;
use crate::features::syntax::{Settings, VscodeHighlighter};
use crate::message::Message;
use crate::theme::{theme, ACCENT_RED, ACCENT_SKY, ACCENT_YELLOW};
use crate::ui::styles::text_editor_style;

pub const GUTTER_VISIBLE_LINES: usize = 60;

pub fn create_editor<'a>(
    content: &'a Content,
    extension: &str,
    current_line: usize,
    scroll_line: usize,
    diagnostics: &[InlineDiagnostic],
) -> Element<'a, Message> {
    let total_lines = content.line_count().max(1);
    let active_line = current_line.clamp(1, total_lines);
    let max_start_line = total_lines.saturating_sub(GUTTER_VISIBLE_LINES - 1).max(1);
    let start_line = scroll_line.clamp(1, max_start_line);
    let end_line = (start_line + GUTTER_VISIBLE_LINES - 1).min(total_lines);

    let mut gutter_lines = Vec::with_capacity(end_line - start_line + 3);

    if start_line > 1 {
        gutter_lines.push(
            container(text("...").size(12).color(theme().text_dim))
                .width(Length::Fixed(52.0))
                .padding(iced::Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .align_right(Length::Fixed(52.0))
                .into(),
        );
    }

    for line in start_line..=end_line {
        let is_active = line == active_line;
        let marker = marker_for_line(line, diagnostics);
        gutter_lines.push(
            container(
                row![
                    text(marker.0).size(12).color(marker.1),
                    text(format!("{line:>4}")).size(12).color(if is_active {
                        theme().text_primary
                    } else {
                        theme().text_dim
                    }),
                ]
                .spacing(6),
            )
            .width(Length::Fixed(64.0))
            .padding(iced::Padding {
                top: 0.0,
                right: 8.0,
                bottom: 0.0,
                left: 0.0,
            })
            .align_right(Length::Fixed(64.0))
            .into(),
        );
    }

    if end_line < total_lines {
        gutter_lines.push(
            container(text("...").size(12).color(theme().text_dim))
                .width(Length::Fixed(52.0))
                .padding(iced::Padding {
                    top: 0.0,
                    right: 8.0,
                    bottom: 0.0,
                    left: 0.0,
                })
                .align_right(Length::Fixed(52.0))
                .into(),
        );
    }

    let gutter = container(column(gutter_lines).spacing(0))
        .width(Length::Fixed(68.0))
        .padding(iced::Padding {
            top: 4.0,
            right: 2.0,
            bottom: 4.0,
            left: 2.0,
        })
        .style(|_theme| container::Style {
            background: None,
            border: Border {
                color: iced::Color::TRANSPARENT,
                width: 0.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        });

    let editor = TextEditor::new(content)
        .on_action(Message::EditorAction)
        .key_binding(editor_key_bindings)
        .highlight_with::<VscodeHighlighter>(
            Settings {
                extension: extension.to_string(),
            },
            |highlight, _theme| highlight.to_format(),
        )
        .style(text_editor_style)
        .padding(iced::Padding {
            top: 4.0,
            right: 4.0,
            bottom: 4.0,
            left: 4.0,
        })
        .height(Length::Fill);

    container(row![gutter, editor].height(Length::Fill))
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme().bg_editor)),
            ..Default::default()
        })
        .into()
}

fn editor_key_bindings(key_press: KeyPress) -> Option<Binding<Message>> {
    let modifiers = key_press.modifiers;

    if let Key::Character(_c) = key_press.key.as_ref() {
        if modifiers.command() {
            return None;
        }
    }

    match key_press.key.as_ref() {
        Key::Named(key::Named::Backspace) => {
            if modifiers.command() {
                Some(Binding::Sequence(vec![
                    // Detects when the cmd key is pressed and begin a sequence
                    Binding::Select(Motion::Home),
                    Binding::Backspace, // If home + backspace is detected, remove whole line
                ]))
            } else if modifiers.alt() {
                Some(Binding::Sequence(vec![
                    Binding::Select(Motion::WordLeft),
                    Binding::Backspace, // If the alt + delete, the word to the left is gone
                ]))
            } else {
                Binding::from_key_press(key_press) // Returns the default key press.
            }
        }
        Key::Named(key::Named::Delete) => {
            if modifiers.command() {
                Some(Binding::Sequence(vec![
                    Binding::Select(Motion::End),
                    Binding::Delete, // cmd + delete (the one that deletes a character to the right) deletes the line to the right of the cursor
                ]))
            } else if modifiers.alt() {
                Some(Binding::Sequence(vec![
                    Binding::Select(Motion::WordRight),
                    Binding::Delete, // alt + delete removes the word to the right
                ]))
            } else {
                Binding::from_key_press(key_press) // Again, ensures default actions
            }
        }
        _ => Binding::from_key_press(key_press),
    }
}

fn marker_for_line(line: usize, diagnostics: &[InlineDiagnostic]) -> (&'static str, iced::Color) {
    let mut has_error = false;
    let mut has_warning = false;
    let mut has_info = false;

    for diag in diagnostics.iter().filter(|d| d.line == line) {
        if diag.severity == lsp_types::DiagnosticSeverity::ERROR {
            has_error = true;
        } else if diag.severity == lsp_types::DiagnosticSeverity::WARNING {
            has_warning = true;
        } else {
            has_info = true;
        }
    }

    if has_error {
        ("●", ACCENT_RED)
    } else if has_warning {
        ("●", ACCENT_YELLOW)
    } else if has_info {
        ("●", ACCENT_SKY)
    } else {
        (" ", theme().text_dim)
    }
}

pub fn empty_editor<'a>() -> Element<'a, Message> {
    iced::widget::text("").into()
}
