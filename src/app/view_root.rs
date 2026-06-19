use super::*;

impl App {
    /// Builds the root application view tree.
    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::stack;

        if self.startup_page_open {
            return self.view_startup_page();
        }

        let editor_area: Element<'_, Message> = if self.settings_open {
            self.view_settings_panel()
        } else {
            let tab_bar = self.view_tab_bar();
            let editor_widget = self.view_editor();

            let mut editor_col_items: Vec<Element<'_, Message>> = Vec::new();
            if self.find_replace.open {
                editor_col_items.push(self.view_find_replace_panel());
            }

            editor_col_items.push(tab_bar);
            editor_col_items.push(editor_widget);
            if self.terminal_open {
                editor_col_items.push(self.view_terminal_panel());
            }
            if self.command_input.open {
                editor_col_items.push(self.view_command_input_bar());
            }

            let editor_container =
                if self.active_tab.is_some() || self.pending_sensitive_open.is_some() {
                    container(column(editor_col_items))
                } else {
                    self.view_welcome_screen()
                }
                .width(Length::Fill)
                .height(Length::Fill)
                .style(editor_container_style);

            container(editor_container)
                .padding(0)
                .width(Length::Fill)
                .into()
        };

        use crate::app::ActivePanel;
        use crate::features::icons::{icon_handle, IconAsset, IconFormat};

        static ICON_FILES: &[u8] = include_bytes!("../assets/icons/activity-bar-files.svg");
        static ICON_GIT: &[u8] = include_bytes!("../assets/icons/activity-bar-git.svg");

        let make_activity_icon = |bytes: &'static [u8], panel: ActivePanel| -> Element<'_, Message> {
            let is_active = self.active_panel == panel && self.sidebar_visible;
            let icon_color = if is_active {
                iced::Color::from_rgba(1.0, 1.0, 1.0, 0.88)
            } else {
                iced::Color::from_rgba(1.0, 1.0, 1.0, 0.35)
            };

            let asset = IconAsset { format: IconFormat::Svg, bytes };
            let img = iced::widget::image::Image::new(icon_handle(asset, 40))
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(20.0));

            let bg = if is_active {
                iced::Color::from_rgba(1.0, 1.0, 1.0, 0.08)
            } else {
                iced::Color::TRANSPARENT
            };

            iced::widget::button(
                container(img)
                    .width(Length::Fixed(40.0))
                    .height(Length::Fixed(40.0))
                    .center_x(Length::Fixed(40.0))
                    .center_y(Length::Fixed(40.0))
                    .style(move |_t| container::Style {
                        background: Some(Background::Color(bg)),
                        border: iced::Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            )
            .on_press(Message::SetActivePanel(panel))
            .style(move |_t, status| iced::widget::button::Style {
                background: Some(Background::Color(match status {
                    iced::widget::button::Status::Hovered => {
                        iced::Color::from_rgba(1.0, 1.0, 1.0, 0.06)
                    }
                    _ => iced::Color::TRANSPARENT,
                })),
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                text_color: icon_color,
                ..Default::default()
            })
            .padding(0)
            .into()
        };

        let activity_bar: Element<'_, Message> = container(
            iced::widget::column![
                make_activity_icon(ICON_FILES, ActivePanel::Files),
                make_activity_icon(ICON_GIT, ActivePanel::Git),
            ]
            .spacing(4)
            .padding(iced::Padding { top: 8.0, right: 4.0, bottom: 4.0, left: 4.0 }),
        )
        .width(Length::Fixed(48.0))
        .height(Length::Fill)
        .style(|_t| container::Style {
            background: Some(Background::Color(crate::theme::theme().bg_secondary)),
            ..Default::default()
        })
        .into();

        let base_content: Element<'_, Message> = if self.sidebar_visible {
            let panel: Element<'_, Message> = match self.active_panel {
                ActivePanel::Files => view_sidebar(self.file_tree.as_ref(), self.sidebar_width),
                ActivePanel::Git => crate::ui::view_git_panel(&self.git_changes, self.sidebar_width),
            };

            let activity_separator = container(text(""))
                .width(Length::Fixed(1.0))
                .height(Length::Fill)
                .style(activity_panel_separator_style);

            let separator = container(text(""))
                .width(Length::Fixed(1.0))
                .height(Length::Fill)
                .style(sidebar_editor_separator_style);

            let resize_zone = mouse_area(
                container(text(""))
                    .width(Length::Fixed(4.0))
                    .height(Length::Fill),
            )
            .on_press(Message::SidebarResizeStart)
            .interaction(iced::mouse::Interaction::ResizingHorizontally);

            row![activity_bar, activity_separator, panel, separator, resize_zone, editor_area].into()
        } else {
            let activity_separator = container(text(""))
                .width(Length::Fixed(1.0))
                .height(Length::Fill)
                .style(activity_panel_separator_style);

            row![activity_bar, activity_separator, editor_area].into()
        };

        let status_bar = self.view_status_bar();

        let with_status = iced::widget::column![
            container(base_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(theme().bg_editor)),
                    ..Default::default()
                }),
            status_bar,
        ];

        let wrapped = container(with_status)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(theme().bg_editor)),
                ..Default::default()
            });

        let base_view: Element<'_, Message> = if self.command_palette.open {
            stack![wrapped, self.view_command_palette_overlay()].into()
        } else if self.fuzzy_finder.open {
            stack![wrapped, self.view_fuzzy_finder_overlay()].into()
        } else if self.file_finder_visible {
            stack![wrapped, self.view_file_finder_overlay()].into()
        } else if self.search_visible {
            let search_panel = container(self.view_search_panel())
                .padding(iced::Padding {
                    top: 20.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 20.0,
                })
                .width(Length::Fill)
                .height(Length::Fill);
            stack![wrapped, search_panel].into()
        } else {
            wrapped.into()
        };

        let with_notification: Element<'_, Message> = if self.notification.is_some() {
            stack![base_view, self.view_notification_toast()].into()
        } else {
            base_view
        };

        // Ghost tab follows the cursor at window level so it isn't clipped by the tab bar.
        let with_drag_ghost: Element<'_, Message> = if let Some(ghost) = self.view_floating_drag_ghost() {
            stack![with_notification, ghost].into()
        } else {
            with_notification
        };

        if self.update_banner.is_some() {
            stack![with_drag_ghost, self.view_update_banner()].into()
        } else {
            with_drag_ghost
        }
    }
}
