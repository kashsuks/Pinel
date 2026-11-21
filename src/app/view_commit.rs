//! the tab that shows one commit: its details, then the diff of every file

use std::collections::HashSet;

use iced::{
    widget::{button, column, container, row, scrollable, text, Space},
    Background, Border, Color, Element, Length,
};

use super::App;
use crate::{
    features::{
        git::Commit,
        git_commit::{relative_time, unix_now, CommitDetail},
        git_diff::{DiffLine, FileDiff, FileStatus, LineKind},
    },
    message::Message,
    theme::{theme, RADIUS_SM, SPACE_1, SPACE_2, SPACE_3, SPACE_4, TEXT_MD, TEXT_SM, TEXT_XS},
    ui::{
        commit_card::{GREEN, RED},
        git_graph::{pill, ref_color},
    },
};

const AMBER: Color = Color::from_rgb(0.98, 0.74, 0.18);
const BLUE: Color = Color::from_rgb(0.40, 0.69, 0.98);

impl App {
    /// The whole commit tab: header, then one foldable section per changed file.
    pub(super) fn view_commit_tab<'a>(
        &'a self,
        hash: &'a str,
        diff: Option<&'a [FileDiff]>,
        collapsed: &HashSet<usize>,
    ) -> Element<'a, Message> {
        let detail = self.git_commit_details.get(hash);
        let commit = self.git_history.iter().find(|c| c.hash == hash);
        let muted = theme().text_muted;

        let mut page = column![view_header(hash, detail, commit)].spacing(SPACE_4).padding(20);
        match diff {
            None => page = page.push(text("Loading diff...").size(TEXT_SM).color(muted)),
            Some([]) => page = page.push(text("No file changes").size(TEXT_SM).color(muted)),
            Some(files) => {
                for (idx, file) in files.iter().enumerate() {
                    page = page.push(view_file_diff(idx, file, collapsed.contains(&idx)));
                }
            },
        }

        scrollable(page).width(Length::Fill).height(Length::Fill).into()
    }
}

/// a plain text button in the link colour
fn link<'a>(label: &str, on_press: Message) -> Element<'a, Message> {
    button(text(label.to_string()).size(TEXT_SM).color(BLUE))
        .padding(0)
        .style(|_theme, _status| button::Style {
            text_color: BLUE,
            ..button::Style::default()
        })
        .on_press(on_press)
        .into()
}

fn view_header<'a>(
    hash: &'a str,
    detail: Option<&'a CommitDetail>,
    commit: Option<&'a Commit>,
) -> Element<'a, Message> {
    let (primary, muted) = (theme().text_primary, theme().text_muted);
    let subject = detail
        .map(|d| d.subject.as_str())
        .or(commit.map(|c| c.subject.as_str()))
        .unwrap_or_default();

    let mut header = column![text(subject).size(20).color(primary)].spacing(SPACE_2);

    if let Some(body) = detail.map(|d| d.body.as_str()).filter(|b| !b.is_empty()) {
        header = header.push(text(body).size(TEXT_MD).color(muted));
    }

    match detail {
        Some(d) => {
            header = header
                .push(text(format!("{} <{}>", d.author, d.email)).size(TEXT_SM).color(primary))
                .push(
                    text(format!(
                        "{} ({})",
                        relative_time(unix_now(), d.timestamp),
                        d.date
                    ))
                    .size(TEXT_SM)
                    .color(muted),
                )
                .push(
                    row![
                        text(format!(
                            "{} file{} changed,",
                            d.files_changed,
                            if d.files_changed == 1 {
                                ""
                            } else {
                                "s"
                            }
                        ))
                        .size(TEXT_SM)
                        .color(muted),
                        text(format!("{} insertions(+),", d.insertions)).size(TEXT_SM).color(GREEN),
                        text(format!("{} deletions(-)", d.deletions)).size(TEXT_SM).color(RED),
                    ]
                    .spacing(SPACE_1),
                );
        },
        None => header = header.push(text("Loading details...").size(TEXT_SM).color(muted)),
    }

    let mut hash_row = row![
        text(hash).size(TEXT_SM).color(muted).font(iced::Font::MONOSPACE),
        link("Copy", Message::CopyText(hash.to_string())),
    ]
    .spacing(SPACE_3)
    .align_y(iced::Alignment::Center);
    if let Some(url) = detail.and_then(|d| d.github_url.clone()) {
        hash_row = hash_row.push(link("Open on GitHub", Message::OpenUrl(url)));
    }
    header = header.push(hash_row);

    if let Some(commit) = commit {
        if !commit.parents.is_empty() {
            let mut parents = row![text("Parents").size(TEXT_SM).color(muted)]
                .spacing(SPACE_2)
                .align_y(iced::Alignment::Center);
            for parent in &commit.parents {
                parents = parents.push(link(
                    &parent[..parent.len().min(7)],
                    Message::OpenCommitTab(parent.clone()),
                ));
            }
            header = header.push(parents);
        }
        if !commit.refs.is_empty() {
            header = header.push(
                row(commit.refs.iter().map(|r| pill(r.name.clone(), ref_color(r)))).spacing(6),
            );
        }
    }

    header.into()
}

fn view_file_diff<'a>(idx: usize, file: &'a FileDiff, folded: bool) -> Element<'a, Message> {
    let (primary, muted) = (theme().text_primary, theme().text_muted);
    let (letter, color) = match file.status {
        FileStatus::Added => ("A", GREEN),
        FileStatus::Modified => ("M", AMBER),
        FileStatus::Deleted => ("D", RED),
        FileStatus::Renamed => ("R", BLUE),
    };
    let title = if file.status == FileStatus::Renamed {
        format!("{} -> {}", file.old_path, file.new_path)
    } else {
        file.display_path().to_string()
    };

    let header = button(
        row![
            text(if folded {
                "▸"
            } else {
                "▾"
            })
            .size(TEXT_SM)
            .color(muted),
            text(letter).size(TEXT_SM).color(color),
            text(title).size(TEXT_SM).color(primary),
            Space::new().width(Length::Fill),
            text(format!("+{}", file.added)).size(TEXT_XS).color(GREEN),
            text(format!("-{}", file.removed)).size(TEXT_XS).color(RED),
        ]
        .spacing(SPACE_2)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding(SPACE_2)
    .on_press(Message::ToggleCommitFile(idx))
    .style(|_theme, status| button::Style {
        background: Some(Background::Color(
            if matches!(status, button::Status::Hovered) {
                theme().bg_hover
            } else {
                theme().bg_secondary
            },
        )),
        text_color: theme().text_primary,
        border: Border {
            radius: RADIUS_SM.into(),
            ..Border::default()
        },
        ..button::Style::default()
    });

    let mut section = column![header].spacing(2);
    if !folded {
        if file.binary {
            section = section.push(text("Binary file not shown").size(TEXT_SM).color(muted));
        }
        for hunk in &file.hunks {
            section = section
                .push(text(&hunk.header).size(TEXT_XS).color(BLUE).font(iced::Font::MONOSPACE));

            section = section.push(column(hunk.lines.iter().map(view_diff_line)).spacing(0));
        }
        if file.hidden_lines > 0 {
            section = section.push(
                text(format!("{} more lines not shown", file.hidden_lines))
                    .size(TEXT_XS)
                    .color(muted),
            );
        }
    }

    section.into()
}

fn view_diff_line<'a>(line: &'a DiffLine) -> Element<'a, Message> {
    let (background, sign) = match line.kind {
        LineKind::Added => (Some(Color { a: 0.14, ..GREEN }), '+'),
        LineKind::Removed => (Some(Color { a: 0.14, ..RED }), '-'),
        LineKind::Context => (None, ' '),
    };
    let number = |n: Option<usize>| {
        text(n.map(|n| n.to_string()).unwrap_or_default())
            .size(TEXT_XS)
            .color(theme().text_dim)
            .font(iced::Font::MONOSPACE)
            .width(Length::Fixed(44.0))
    };

    container(
        row![
            number(line.old_line),
            number(line.new_line),
            text(format!("{sign}{}", line.text))
                .size(TEXT_SM)
                .color(theme().text_primary)
                .font(iced::Font::MONOSPACE)
                .wrapping(iced::widget::text::Wrapping::None),
        ]
        .spacing(SPACE_2),
    )
    .width(Length::Fill)
    .style(move |_theme| container::Style {
        background: background.map(Background::Color),
        ..container::Style::default()
    })
    .into()
}
