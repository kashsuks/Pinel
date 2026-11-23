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
    theme::{theme, SPACE_1, SPACE_2, SPACE_3, TEXT_SM, TEXT_XS},
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
        text(&commit.author).size(TEXT_SM).color(primary),
        text(when).size(TEXT_SM).color(muted),
    ]
    .spacing(SPACE_2);

    let mut message = column![text(&commit.subject).size(TEXT_SM).color(primary)].spacing(SPACE_1);
    if let Some(body) = detail.map(|d| d.body.as_str()).filter(|b| !b.is_empty()) {
        message = message.push(text(body).size(TEXT_XS).color(muted));
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
            .size(TEXT_XS)
            .color(muted),
            text(format!("{} insertions(+),", d.insertions)).size(TEXT_XS).color(GREEN),
            text(format!("{} deletions(-)", d.deletions)).size(TEXT_XS).color(RED),
        ]
        .spacing(SPACE_1)
        .into(),
        None => text("Loading details...").size(TEXT_XS).color(muted).into(),
    };

    let pills = row(commit
        .refs
        .iter()
        .take(MAX_CARD_REFS)
        .map(|r| pill(r.name.clone(), ref_color(r))))
    .spacing(6);

    let mut footer = row![text(commit.short_hash()).size(TEXT_XS).color(muted)]
        .spacing(SPACE_3)
        .align_y(iced::Alignment::Center);
    if let Some(url) = detail.and_then(|d| d.github_url.clone()) {
        footer = footer.push(
            button(text("Open on GitHub").size(TEXT_XS).color(LINK))
                .padding(0)
                .style(|_theme, _status| button::Style {
                    text_color: LINK,
                    ..button::Style::default()
                })
                .on_press(Message::OpenUrl(url)),
        );
    }

    let card = container(column![header, message, stats, pills, footer].spacing(SPACE_2))
        .padding(SPACE_3)
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
