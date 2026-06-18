use iced::widget::image;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Length};

use crate::features::file_tree::{FileEntry, FileTree};
use crate::features::icons::{get_file_icon, get_folder_icon, icon_handle, IconAsset};
use crate::message::Message;
use crate::theme::*;
use crate::ui::styles::{sidebar_container_style, tree_button_style};

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
                text("Working tree is clean")
                    .size(11)
                    .color(theme().text_placeholder),
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
        let header = text(format!("Changes ({})", changes.len()))
            .size(11)
            .color(theme().text_muted);

        let mut items: Vec<Element<'a, Message>> = vec![
            container(header)
                .padding(iced::Padding { top: 4.0, right: 4.0, bottom: 4.0, left: 4.0 })
                .into(),
        ];

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
                    .padding(iced::Padding { top: 3.0, right: 4.0, bottom: 3.0, left: 8.0 })
                    .width(Length::Fill)
                    .into(),
            );
        }

        column(items).spacing(0).into()
    };

    container(scrollable(content).height(Length::Fill))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .padding(iced::Padding { top: 2.0, right: 2.0, bottom: 2.0, left: 4.0 })
        .style(sidebar_container_style)
        .into()
}

pub fn view_sidebar<'a>(file_tree: Option<&'a FileTree>, width: f32) -> Element<'a, Message> {
    let sidebar_content: Element<'a, Message> = match file_tree {
        Some(tree) => view_file_tree(tree),
        None => view_empty_sidebar(),
    };

    let sidebar = container(scrollable(sidebar_content).height(Length::Fill))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .padding(iced::Padding {
            top: 2.0,
            right: 2.0,
            bottom: 2.0,
            left: 4.0,
        })
        .style(sidebar_container_style);

    container(sidebar).padding(0).into()
}

fn view_file_tree(tree: &FileTree) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = Vec::new();
    render_entries(&tree.entries, tree, 0, &mut items);
    column(items).spacing(4).into()
}

fn view_empty_sidebar<'a>() -> Element<'a, Message> {
    container(
        column![
            text("No folder open").size(13).color(theme().text_muted),
            text("Cmd+O to open")
                .size(11)
                .color(theme().text_placeholder),
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
                let icon: Element<'_, Message> = icon_widget(get_folder_icon(name, is_expanded));

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

                items.push(btn.into());

                if is_expanded {
                    render_entries(children, tree, depth + 1, items);
                }
            }
            FileEntry::File { path, name } => {
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

                items.push(btn.into());
            }
        }
    }
}
