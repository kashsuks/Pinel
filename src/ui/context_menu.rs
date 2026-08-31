use iced::widget::{button, column, container, text, Space};
use iced::{Element, Length};

use crate::message::Message;
use crate::ui::styles::{
    context_menu_item_style, context_menu_item_style_destructive, context_menu_panel_style,
    context_menu_seperator_style,
};

fn menu_item(label: &'static str, message: Message) -> Element<'static, Message> {
    button(text(label).size(13))
        .on_press(message)
        .style(context_menu_item_style)
        .padding(iced::Padding { top: 7.0, right: 14.0, bottom: 7.0, left: 14.0 })
        .width(Length::Fill)
        .into()
}

fn menu_item_destructive(label: &'static str, message: Message) -> Element<'static, Message> {
    button(text(label).size(13))
        .on_press(message)
        .style(context_menu_item_style_destructive)
        .padding(iced::Padding { top: 7.0, right: 14.0, bottom: 7.0, left: 14.0 })
        .width(Length::Fill)
        .into()
}

fn seperator<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
        .style(context_menu_seperator_style)
        .padding(iced::Padding { top: 4.0, right: 0.0, bottom: 4.0, left: 0.0 })
        .into()
}

fn reveal_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "Reveal in Finder"
    } else if cfg!(target_os = "windows") {
        "Reveal in File Explorer"
    } else {
        "Open Containing Folder"
    }
}

pub fn view_context_menu<'a>() -> Element<'a, Message> {
    let items = column![
        menu_item("New File", Message::FileTreeNewFile),
        menu_item("New Folder", Message::FileTreeNewFolder),
        seperator(),
        menu_item("Rename...", Message::FileTreeRenameStart),
        seperator(),
        menu_item(reveal_label(), Message::FileTreeReveal),
        menu_item("Copy Path", Message::FileTreeCopyPath),
        seperator(),
        menu_item_destructive("Delete", Message::FileTreeDelete),
    ]
    .spacing(1)
    .padding(iced::Padding { top: 6.0, right: 6.0, bottom: 6.0, left: 6.0});

    container(items)
        .width(Length::Fixed(220.0))
        .style(context_menu_panel_style)
        .into()
}
