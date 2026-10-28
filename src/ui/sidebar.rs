use iced::{
    widget::{
        button, column, container, image, mouse_area, row, scrollable, text, text_input, Space,
    },
    Element, Length,
};

use crate::{
    features::{
        file_tree::{FileEntry, FileTree},
        git::{split_changes, ChangeKind, Commit, FileChange},
        git_graph::GraphRow,
        icons::{get_file_icon, get_folder_icon, icon_handle, IconAsset},
        virtual_list::{spacer_heights, visible_range},
    },
    message::Message,
    theme::*,
    ui::{
        git_graph::{view_commit_row, ROW_HEIGHT},
        styles::{rename_input_style, sidebar_container_style, tree_button_style},
    },
};

/// Create an icon element from embedded bytes.
fn icon_widget<'a>(icon: IconAsset) -> Element<'a, Message> {
    image::Image::new(icon_handle(icon, ICON_SIZE as u32))
        .width(Length::Fixed(ICON_SIZE))
        .height(Length::Fixed(ICON_SIZE))
        .into()
}

/// The colour each status letter is drawn in.
fn change_color(kind: ChangeKind) -> iced::Color {
    match kind {
        ChangeKind::Modified | ChangeKind::TypeChanged => iced::Color::from_rgb(0.98, 0.74, 0.18),
        ChangeKind::Added | ChangeKind::Untracked | ChangeKind::Copied => {
            iced::Color::from_rgb(0.36, 0.86, 0.42)
        },
        ChangeKind::Deleted | ChangeKind::Conflicted => iced::Color::from_rgb(0.92, 0.37, 0.37),
        ChangeKind::Renamed => iced::Color::from_rgb(0.40, 0.69, 0.98),
        ChangeKind::Ignored => theme().text_muted,
    }
}

/// Extra rows built above and below the viewport so fast scrolling never
/// shows a gap.
const OVERSCAN: usize = 6;

/// One line of the status section: a heading or a changed file.
enum StatusEntry<'a> {
    Header(String),
    Change(&'a FileChange, ChangeKind),
}

/// Flattens staged and unstaged changes, with their headings, into one list
/// so it can be windowed by index.
fn status_entries(changes: &[FileChange]) -> Vec<StatusEntry<'_>> {
    let (staged, unstaged) = split_changes(changes);
    let mut entries = Vec::with_capacity(staged.len() + unstaged.len() + 2);
    if !staged.is_empty() {
        entries.push(StatusEntry::Header(format!(
            "Staged Changes ({})",
            staged.len()
        )));
        entries.extend(
            staged
                .into_iter()
                .filter_map(|c| c.staged_kind().map(|k| StatusEntry::Change(c, k))),
        );
    }
    if !unstaged.is_empty() {
        entries.push(StatusEntry::Header(format!("Changes ({})", unstaged.len())));
        entries.extend(
            unstaged
                .into_iter()
                .filter_map(|c| c.unstaged_kind().map(|k| StatusEntry::Change(c, k))),
        );
    }
    entries
}

/// Which commits to build, given where the commit section sits in the list.
///
/// The section starts after every status entry plus the "Commits" heading,
/// so the scroll offset is measured from there.
fn commit_window(
    scroll_offset: f32,
    viewport_h: f32,
    status_len: usize,
    commit_len: usize,
) -> std::ops::Range<usize> {
    let section_top = (status_len + 1) as f32 * ROW_HEIGHT;
    visible_range(
        scroll_offset - section_top,
        viewport_h,
        ROW_HEIGHT,
        commit_len,
        OVERSCAN,
    )
}

/// The git panel: changed files, then the commit graph below them.
pub fn view_git_panel<'a>(
    changes: &'a [FileChange],
    history: &'a [Commit],
    graph: &'a [GraphRow],
    width: f32,
    scroll_offset: f32,
    viewport_h: f32,
) -> Element<'a, Message> {
    let mut items: Vec<Element<'a, Message>> = Vec::new();

    if changes.is_empty() && history.is_empty() {
        items.push(
            container(
                column![
                    text("No changes").size(13).color(theme().text_muted),
                    text("Working tree is clean").size(11).color(theme().text_placeholder),
                ]
                .spacing(4)
                .align_x(iced::Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(16)
            .into(),
        );
    }

    let entries = status_entries(changes);
    let range = visible_range(
        scroll_offset,
        viewport_h,
        ROW_HEIGHT,
        entries.len(),
        OVERSCAN,
    );
    let (above, below) = spacer_heights(&range, entries.len(), ROW_HEIGHT);

    if above > 0.0 {
        items.push(Space::new().height(Length::Fixed(above)).into());
    }
    for entry in &entries[range] {
        match entry {
            StatusEntry::Header(label) => items.push(section_header(label.clone())),
            StatusEntry::Change(change, kind) => items.extend(change_row(change, Some(*kind))),
        }
    }
    if below > 0.0 {
        items.push(Space::new().height(Length::Fixed(below)).into());
    }

    if !history.is_empty() {
        items.push(section_header("Commits".to_string()));
        let window = commit_window(scroll_offset, viewport_h, entries.len(), history.len());
        items.push(view_commit_graph(history, graph, width, window));
    }

    container(
        scrollable(column(items).spacing(0)).height(Length::Fill).on_scroll(|viewport| {
            Message::GitPanelScrolled {
                offset: viewport.absolute_offset().y,
                viewport: viewport.bounds().height,
            }
        }),
    )
    .width(Length::Fixed(width))
    .height(Length::Fill)
    .padding(iced::Padding {
        top: 14.0,
        right: 2.0,
        bottom: 2.0,
        left: 4.0,
    })
    .style(sidebar_container_style)
    .into()
}

/// One changed file: status letter, name, and its folder dimmed after it.
fn change_row<'a>(
    change: &'a FileChange,
    kind: Option<ChangeKind>,
) -> Option<Element<'a, Message>> {
    let kind = kind?;
    let mut line = row![
        text(kind.letter())
            .size(11)
            .color(change_color(kind))
            .width(Length::Fixed(16.0)),
        text(change.display_name())
            .size(12)
            .wrapping(iced::widget::text::Wrapping::None),
    ]
    .spacing(4)
    .align_y(iced::Alignment::Center);

    let dir = change.parent_dir();
    if !dir.is_empty() {
        line = line.push(
            text(dir)
                .size(11)
                .color(theme().text_placeholder)
                .wrapping(iced::widget::text::Wrapping::None),
        );
    }

    Some(
        container(line)
            .padding(iced::Padding {
                top: 0.0,
                right: 4.0,
                bottom: 0.0,
                left: 8.0,
            })
            .width(Length::Fill)
            .height(Length::Fixed(ROW_HEIGHT))
            .align_y(iced::alignment::Vertical::Center)
            .clip(true)
            .into(),
    )
}

fn section_header<'a>(label: String) -> Element<'a, Message> {
    container(text(label).size(11).color(theme().text_muted))
        .padding(iced::Padding {
            top: 0.0,
            right: 4.0,
            bottom: 0.0,
            left: 4.0,
        })
        .width(Length::Fill)
        .height(Length::Fixed(ROW_HEIGHT))
        .align_y(iced::alignment::Vertical::Center)
        .into()
}

/// One row per commit, with refs and a subject sized to the panel.
fn view_commit_graph<'a>(
    history: &'a [Commit],
    graph: &'a [GraphRow],
    width: f32,
    window: std::ops::Range<usize>,
) -> Element<'a, Message> {
    // lane count comes from the whole history so the graph column keeps one
    // width however far the user has scrolled
    let lanes = graph.iter().map(|r| r.width).max().unwrap_or(1);
    let (above, below) = spacer_heights(&window, history.len(), ROW_HEIGHT);

    let mut rows: Vec<Element<'a, Message>> = Vec::new();
    if above > 0.0 {
        rows.push(Space::new().height(Length::Fixed(above)).into());
    }
    rows.extend(
        history[window.clone()]
            .iter()
            .zip(&graph[window])
            .map(|(commit, graph_row)| view_commit_row(commit, graph_row, lanes, width)),
    );
    if below > 0.0 {
        rows.push(Space::new().height(Length::Fixed(below)).into());
    }

    column(rows).spacing(0).clip(true).into()
}

pub fn view_sidebar<'a>(
    file_tree: Option<&'a FileTree>,
    width: f32,
    rename_target: Option<&'a (std::path::PathBuf, bool)>,
    rename_input: &'a str,
    rename_input_id: iced::widget::Id,
) -> Element<'a, Message> {
    let sidebar_content: Element<'a, Message> = match file_tree {
        Some(tree) => view_file_tree(tree, rename_target, rename_input, rename_input_id),
        None => view_empty_sidebar(),
    };

    let sidebar = container(scrollable(sidebar_content).height(Length::Fill))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 14.0,
            right: 2.0,
            bottom: 2.0,
            left: 4.0,
        })
        .style(sidebar_container_style);

    // Wrap the whole panel (outside the scrollable) so right-clicks on
    // empty space - below the last row, in row gaps, anywhere the
    // scrollable's content doesn't cover - still open the context menu,
    // targeting the tree's root folder. Row-level mouse_areas are nested
    // inside and consume the event first when the click actually lands on
    // a row, so this only fires for genuine empty-space clicks.
    //
    // Note: this can't be done by making the scrollable's *content* fill
    // the panel - iced marks a vertical scrollable's content height as
    // "compressed", which makes Length::Fill resolve to the content's
    // intrinsic size instead of the available space, so it never actually
    // reaches the empty area below the rows.
    let sidebar: Element<'a, Message> = match file_tree {
        Some(tree) => mouse_area(sidebar)
            .on_right_press(Message::FileTreeContextMenuOpen(tree.root.clone(), true))
            .into(),
        None => sidebar.into(),
    };

    container(sidebar).padding(0).into()
}

fn view_file_tree<'a>(
    tree: &'a FileTree,
    rename_target: Option<&'a (std::path::PathBuf, bool)>,
    rename_input: &'a str,
    rename_input_id: iced::widget::Id,
) -> Element<'a, Message> {
    let mut items: Vec<Element<'a, Message>> = Vec::new();
    render_entries(
        &tree.entries,
        tree,
        0,
        &mut items,
        rename_target,
        rename_input,
        rename_input_id,
    );
    column(items).spacing(4).into()
}

fn view_empty_sidebar<'a>() -> Element<'a, Message> {
    let shortcut_hint = if cfg!(target_os = "macos") {
        "⌘ + Shift + O"
    } else {
        "Ctrl + Shift + O to open"
    };

    container(
        column![
            text("No folder open").size(13).color(theme().text_muted),
            text(shortcut_hint).size(11).color(theme().text_placeholder),
        ]
        .spacing(4)
        .align_x(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn render_entries<'a>(
    entries: &'a [FileEntry],
    tree: &'a FileTree,
    depth: usize,
    items: &mut Vec<Element<'a, Message>>,
    rename_target: Option<&'a (std::path::PathBuf, bool)>,
    rename_input: &'a str,
    rename_input_id: iced::widget::Id,
) {
    let indent_width = INDENT_WIDTH * depth as f32;

    for entry in entries {
        match entry {
            FileEntry::Directory {
                path,
                name,
                children,
            } => {
                let is_expanded = tree.is_expanded(path);
                let is_renaming = rename_target.map(|(p, _)| p.as_path()) == Some(path.as_path());

                let row_el: Element<'a, Message> = if is_renaming {
                    render_rename_row(
                        indent_width,
                        get_folder_icon(name, is_expanded),
                        rename_input,
                        rename_input_id.clone(),
                    )
                } else {
                    let icon: Element<'_, Message> =
                        icon_widget(get_folder_icon(name, is_expanded));

                    let btn = button(
                        row![
                            container(text("")).width(Length::Fixed(indent_width)),
                            icon,
                            text(name).size(13),
                        ]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                    )
                    .style(tree_button_style)
                    .on_press(Message::FolderToggled(path.clone()))
                    .padding(iced::Padding {
                        top: 6.0,
                        right: 10.0,
                        bottom: 6.0,
                        left: 10.0,
                    })
                    .width(Length::Fill);

                    mouse_area(btn)
                        .on_right_press(Message::FileTreeContextMenuOpen(path.clone(), true))
                        .into()
                };

                items.push(row_el);

                if is_expanded {
                    render_entries(
                        children,
                        tree,
                        depth + 1,
                        items,
                        rename_target,
                        rename_input,
                        rename_input_id.clone(),
                    );
                }
            },
            FileEntry::File { path, name } => {
                let is_renaming = rename_target.map(|(p, _)| p.as_path()) == Some(path.as_path());

                let row_el: Element<'a, Message> = if is_renaming {
                    render_rename_row(
                        indent_width,
                        get_file_icon(name),
                        rename_input,
                        rename_input_id.clone(),
                    )
                } else {
                    let icon: Element<'_, Message> = icon_widget(get_file_icon(name));

                    let btn = button(
                        row![
                            container(text("")).width(Length::Fixed(indent_width)),
                            icon,
                            text(name).size(13),
                        ]
                        .spacing(6)
                        .align_y(iced::Alignment::Center),
                    )
                    .style(tree_button_style)
                    .on_press(Message::FileClicked(path.clone()))
                    .padding(iced::Padding {
                        top: 6.0,
                        right: 10.0,
                        bottom: 6.0,
                        left: 10.0,
                    })
                    .width(Length::Fill);

                    mouse_area(btn)
                        .on_right_press(Message::FileTreeContextMenuOpen(path.clone(), false))
                        .into()
                };

                items.push(row_el);
            },
        }
    }
}

fn render_rename_row<'a>(
    indent_width: f32,
    icon: IconAsset,
    rename_input: &'a str,
    rename_input_id: iced::widget::Id,
) -> Element<'a, Message> {
    let input = text_input("", rename_input)
        .id(rename_input_id)
        .on_input(Message::FileTreeRenameInputChanged)
        .on_submit(Message::FileTreeRenameSubmit)
        .size(13)
        .padding(iced::Padding {
            top: 2.0,
            right: 4.0,
            bottom: 2.0,
            left: 4.0,
        })
        .style(rename_input_style)
        .width(Length::Fill);

    container(
        row![
            container(text("")).width(Length::Fixed(indent_width)),
            icon_widget(icon),
            input,
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center),
    )
    .padding(iced::Padding {
        top: 6.0,
        right: 10.0,
        bottom: 6.0,
        left: 10.0,
    })
    .width(Length::Fill)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_window_top_of_panel_shows_no_commits() {
        // 10 status rows + the heading = 11 rows (242px) above the commits
        let window = commit_window(0.0, 220.0, 10, 500);
        assert_eq!(window, 0..OVERSCAN);
    }

    #[test]
    fn test_commit_window_starts_at_section_boundary() {
        // viewport top sits exactly on the first commit
        let window = commit_window(242.0, 220.0, 10, 500);
        assert_eq!(window, 0..(10 + OVERSCAN));
    }

    #[test]
    fn test_commit_window_scrolled_into_the_middle() {
        let window = commit_window(242.0 + 22.0 * 100.0, 220.0, 10, 500);
        assert_eq!(window, (100 - OVERSCAN)..(110 + OVERSCAN));
    }

    #[test]
    fn test_commit_window_clamped_at_the_end() {
        let window = commit_window(1_000_000.0, 220.0, 10, 500);
        assert_eq!(window, 500..500);
    }

    #[test]
    fn test_commit_window_no_status_entries() {
        let window = commit_window(22.0, 220.0, 0, 500);
        assert_eq!(window, 0..(10 + OVERSCAN));
    }

    #[test]
    fn test_commit_window_empty_history() {
        assert_eq!(commit_window(0.0, 220.0, 10, 0), 0..0);
    }
}
