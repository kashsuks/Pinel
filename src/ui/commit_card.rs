//! the floating card shown when the cursor rests on a commit in the graph

use iced::{
    widget::{button, column, container, mouse_area, row, text},
    Background, Border, Color, Element, Length, Shadow, Vector,
};

use crate::{
    features::{
        git::Commit,
        git_commit::{relative_time, CommitDetail},
    },
    message::Message,
    theme::theme,
    ui::git_graph::{pill, ref_color},
};

/// Colour for insertions.
pub const GREEN: Color = Color::from_rgb(0.36, 0.86, 0.42);
/// Colour for deletions.
pub const RED: Color = Color::from_rgb(0.92, 0.37, 0.32);
const LINK: Color = Color::from_rgb(0.40, 0.69, 0.98);

/// Width of the card in pixels.
pub const CARD_WIDTH: f32 = 420.0;
/// Rough height, used to keep the card inside the window.
pub const CARD_HEIGHT_ESTIMATE: f32 = 190.0;
/// Most ref pills the card lists.
const MAX_CARD_REFS: usize = 6;

/// The card for `commit`. `detail` is `None` while `git show` is still running.
pub fn view_commit_card<'a>(
    commit: &'a Commit,
    detail: Option<&'a CommitDetail>,
    now: i64,
) -> Element<'a, Message> {
    let (primary, muted) = (theme().text_primary, theme().text_muted);

    let when = match detail {
        Some(d) => format!("{} ({})", relative_time(now, commit.timestamp), d.date),
        None => relative_time(now, commit.timestamp),
    };
    let header = row![
        text(&commit.author).size(12).color(primary),
        text(when).size(12).color(muted),
    ]
    .spacing(8);

    let mut message = column![text(&commit.subject).size(12).color(primary)].spacing(4);
    if let Some(body) = detail.map(|d| d.body.as_str()).filter(|b| !b.is_empty()) {
        message = message.push(text(body).size(11).color(muted));
    }

    let stats: Element<'a, Message> = match detail {
        Some(d) => row![
            text(format!(
                "{} file{} changed,",
                d.files_changed,
                if d.files_changed == 1 {
                    ""
                } else {
                    "s"
                }
            ))
            .size(11)
            .color(muted),
            text(format!("{} insertions(+),", d.insertions)).size(11).color(GREEN),
            text(format!("{} deletions(-)", d.deletions)).size(11).color(RED),
        ]
        .spacing(4)
        .into(),
        None => text("Loading details...").size(11).color(muted).into(),
    };

    let pills = row(commit
        .refs
        .iter()
        .take(MAX_CARD_REFS)
        .map(|r| pill(r.name.clone(), ref_color(r))))
    .spacing(6);

    let mut footer = row![text(commit.short_hash()).size(11).color(muted)]
        .spacing(12)
        .align_y(iced::Alignment::Center);
    if let Some(url) = detail.and_then(|d| d.github_url.clone()) {
        footer = footer.push(
            button(text("Open on GitHub").size(11).color(LINK))
                .padding(0)
                .style(|_theme, _status| button::Style {
                    text_color: LINK,
                    ..button::Style::default()
                })
                .on_press(Message::OpenUrl(url)),
        );
    }

    let card = container(column![header, message, stats, pills, footer].spacing(8))
        .padding(12)
        .width(Length::Fixed(CARD_WIDTH))
        .style(|_theme| container::Style {
            background: Some(Background::Color(theme().bg_secondary)),
            border: Border {
                color: theme().border_subtle,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..container::Style::default()
        });

    // the card stays open while the cursor is on it, which is what makes its link clickable
    mouse_area(card)
        .on_enter(Message::GitCardHovered(true))
        .on_exit(Message::GitCardHovered(false))
        .into()
}
