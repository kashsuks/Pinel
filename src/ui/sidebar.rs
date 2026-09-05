use iced::{
    widget::{button, column, container, image, mouse_area, row, scrollable, text, text_input},
    Element, Length,
};

use crate::{
    features::{
        file_tree::{FileEntry, FileTree},
        icons::{get_file_icon, get_folder_icon, icon_handle, IconAsset},
    },
    message::Message,
    theme::*,
    ui::styles::{rename_input_style, sidebar_container_style, tree_button_style},
};

/// Create an icon element from embedded bytes.
fn icon_widget<'a>(icon: IconAsset) -> Element<'a, Message> {
    image::Image::new(icon_handle(icon, ICON_SIZE as u32))
        .width(Length::Fixed(ICON_SIZE))
        .height(Length::Fixed(ICON_SIZE))
        .into()
}

pub fn view_git_panel<'a>(changes: &'a [(String, String)], width: f32) -> Element<'a, Message> {
    let content: Element<'a, Message> = if changes.is_empty() {
        container(
            column![
                text("No changes").size(13).color(theme().text_muted),
                text("Working tree is clean").size(11).color(theme().text_placeholder),
            ]
            .spacing(4)
            .align_x(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        let header =
            text(format!("Changes ({})", changes.len())).size(11).color(theme().text_muted);

        let mut items: Vec<Element<'a, Message>> = vec![container(header)
            .padding(iced::Padding {
                top: 4.0,
                right: 4.0,
                bottom: 4.0,
                left: 4.0,
            })
            .into()];

        for (status, file) in changes {
            let status_color = match status.as_str() {
                "M" | "MM" => iced::Color::from_rgb(0.98, 0.74, 0.18),
                "A" | "??" => iced::Color::from_rgb(0.36, 0.86, 0.42),
                "D" => iced::Color::from_rgb(0.92, 0.37, 0.37),
                _ => theme().text_muted,
            };

            let label = match status.as_str() {
                "M" | "MM" => "M",
                "A" => "A",
                "D" => "D",
                "??" => "U",
                _ => status.as_str(),
            };

            let file_name = std::path::Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file.as_str());

            let row_item = row![
                text(label).size(11).color(status_color).width(Length::Fixed(16.0)),
                text(file_name).size(12),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center);

            items.push(
                container(row_item)
                    .padding(iced::Padding {
                        top: 3.0,
                        right: 4.0,
                        bottom: 3.0,
                        left: 8.0,
                    })
                    .width(Length::Fill)
                    .into(),
            );
        }

        column(items).spacing(0).into()
    };

    container(scrollable(content).height(Length::Fill))
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
