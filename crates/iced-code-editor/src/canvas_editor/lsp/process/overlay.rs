//! LSP overlay UI components for displaying hover tooltips and completion menus.
//!
//! Provides [`LspOverlayState`] for storing overlay display state and
//! [`view_lsp_overlay`] for rendering it on top of a [`CodeEditor`].

use crate::CodeEditor;
use iced::widget::{
    Id, Space, button, column, container, markdown, mouse_area, rich_text, row, rule, scrollable,
    stack, text,
};
use iced::{
    Background, Border, Color, Element, Font, Length, Point, Shadow, Theme, Vector, padding,
};

/// Maximum number of completion items shown at once in the menu.
const MAX_COMPLETION_ITEMS: usize = 8;
/// Height in pixels of each completion item row.
const COMPLETION_ITEM_HEIGHT: f32 = 20.0;
/// Height in pixels of the completion menu header area.
const COMPLETION_HEADER_HEIGHT: f32 = 24.0;
/// Padding in pixels around the completion item list.
const COMPLETION_PADDING: f32 = 4.0;
/// Maximum width in pixels of the completion menu.
const COMPLETION_MENU_WIDTH: f32 = 250.0;
/// Border radius in pixels applied to scrollable rail and scroller borders.
const SCROLLABLE_BORDER_RADIUS: f32 = 4.0;

/// Padding in pixels between the hover box border and its content.
const HOVER_PADDING: f32 = 8.0;
/// Transparent gap in pixels between the hovered line and the hover box.
///
/// The gap is padding *inside* the hover's mouse area, so the hit area still
/// touches the hovered line: moving the mouse from the word into the popup
/// never crosses a strip of bare editor, which would clear the hover.
const HOVER_GAP: f32 = 4.0;
/// Number of editor lines the hover box shows before its content scrolls.
///
/// The box is capped at `HOVER_MAX_VISIBLE_LINES * line_height` plus its
/// padding (256 px at the default 20 px line height), and further by the space
/// left on the side of the hovered line it opens on.
const HOVER_MAX_VISIBLE_LINES: f32 = 12.0;
/// Horizontal room in pixels the hover box always leaves free in the viewport.
const HOVER_VIEWPORT_INSET: f32 = 24.0;
/// Minimum gap in pixels between the hover box and the viewport's right edge.
const HOVER_EDGE_MARGIN: f32 = 4.0;
/// Extra width in pixels given to the measured content, so rounding in text
/// shaping never wraps a code line that measured exactly to the box width.
const HOVER_WIDTH_SLACK: f32 = 2.0;
/// Corner radius in pixels of the hover box.
const HOVER_RADIUS: f32 = 6.0;
/// Hairline colour (white at 8 % opacity) of the hover border and divider.
const HOVER_HAIRLINE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.08,
};
/// Colour of the hover box's drop shadow.
const HOVER_SHADOW_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.35,
};
/// Downward offset in pixels of the hover box's drop shadow.
const HOVER_SHADOW_OFFSET_Y: f32 = 4.0;
/// Blur radius in pixels of the hover box's drop shadow.
const HOVER_SHADOW_BLUR: f32 = 14.0;
/// Vertical spacing between markdown blocks, as a fraction of the text size.
///
/// Tighter than iced's default (0.875) so the signature, divider and docs read
/// as one compact tooltip.
const HOVER_ITEM_SPACING_RATIO: f32 = 0.5;

/// State for the LSP overlay display (hover tooltips and completion menus).
///
/// This struct aggregates all display-related LSP state. Instantiate once in
/// your application and pass it to [`view_lsp_overlay`] for rendering.
///
/// # Example
///
/// ```
/// use iced_code_editor::LspOverlayState;
///
/// let mut state = LspOverlayState::new();
/// assert!(!state.hover_visible);
/// assert!(!state.completion_visible);
/// ```
pub struct LspOverlayState {
    /// The hover text received from the LSP server.
    pub hover_text: Option<String>,
    /// Parsed markdown items derived from `hover_text`.
    pub hover_items: Vec<iced::widget::markdown::Item>,
    /// Whether the hover tooltip is currently visible.
    pub hover_visible: bool,
    /// Screen position where the hover tooltip should be rendered.
    pub hover_position: Option<Point>,
    /// Whether the mouse cursor is currently over the hover tooltip.
    pub hover_interactive: bool,
    /// All completion items received from the LSP server.
    pub all_completions: Vec<String>,
    /// Current filter string applied to completion items.
    pub completion_filter: String,
    /// Filtered completion items to display.
    pub completion_items: Vec<String>,
    /// Whether the completion menu is currently visible.
    pub completion_visible: bool,
    /// Index of the currently selected completion item.
    pub completion_selected: usize,
    /// Whether completion has been suppressed after applying an item.
    pub completion_suppressed: bool,
    /// Screen position of the completion menu anchor.
    pub completion_position: Option<Point>,
}

impl LspOverlayState {
    /// Creates a new [`LspOverlayState`] with all fields at their default values.
    ///
    /// # Example
    ///
    /// ```
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let state = LspOverlayState::new();
    /// assert!(!state.hover_visible);
    /// assert!(!state.completion_visible);
    /// ```
    pub fn new() -> Self {
        Self {
            hover_text: None,
            hover_items: Vec::new(),
            hover_visible: false,
            hover_position: None,
            hover_interactive: false,
            all_completions: Vec::new(),
            completion_filter: String::new(),
            completion_items: Vec::new(),
            completion_visible: false,
            completion_selected: 0,
            completion_suppressed: false,
            completion_position: None,
        }
    }

    /// Sets the screen position where the hover tooltip should appear.
    ///
    /// Call this when dispatching a hover request to the LSP server.
    ///
    /// # Example
    ///
    /// ```
    /// use iced::Point;
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.set_hover_position(Point::new(100.0, 200.0));
    /// assert_eq!(state.hover_position, Some(Point::new(100.0, 200.0)));
    /// ```
    pub fn set_hover_position(&mut self, point: Point) {
        self.hover_position = Some(point);
    }

    /// Displays a hover tooltip with the given text.
    ///
    /// Parses the text as markdown and marks the tooltip as visible.
    ///
    /// # Example
    ///
    /// ```
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.show_hover("**bold** text".to_string());
    /// assert!(state.hover_visible);
    /// assert!(state.hover_text.is_some());
    /// ```
    pub fn show_hover(&mut self, text: String) {
        self.hover_items = iced::widget::markdown::parse(&text).collect();
        self.hover_text = Some(text);
        self.hover_visible = true;
    }

    /// Clears all hover-related state.
    ///
    /// Resets hover text, items, visibility, position, and interaction flags.
    ///
    /// # Example
    ///
    /// ```
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.show_hover("some text".to_string());
    /// state.clear_hover();
    /// assert!(!state.hover_visible);
    /// assert!(state.hover_text.is_none());
    /// ```
    pub fn clear_hover(&mut self) {
        self.hover_text = None;
        self.hover_items.clear();
        self.hover_visible = false;
        self.hover_position = None;
        self.hover_interactive = false;
    }

    /// Sets the completion items and their display position.
    ///
    /// Resets the selection to index 0 and applies the current filter.
    ///
    /// # Example
    ///
    /// ```
    /// use iced::Point;
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.set_completions(
    ///     vec!["foo".to_string(), "bar".to_string()],
    ///     Point::ORIGIN,
    /// );
    /// assert_eq!(state.completion_items.len(), 2);
    /// ```
    pub fn set_completions(&mut self, items: Vec<String>, position: Point) {
        self.all_completions = items;
        self.completion_selected = 0;
        self.completion_position = Some(position);
        self.filter_completions();
    }

    /// Clears all completion-related state.
    ///
    /// # Example
    ///
    /// ```
    /// use iced::Point;
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.set_completions(vec!["foo".to_string()], Point::ORIGIN);
    /// state.clear_completions();
    /// assert!(!state.completion_visible);
    /// assert!(state.all_completions.is_empty());
    /// ```
    pub fn clear_completions(&mut self) {
        self.all_completions.clear();
        self.completion_items.clear();
        self.completion_filter.clear();
        self.completion_visible = false;
        self.completion_suppressed = false;
    }

    /// Filters `all_completions` into `completion_items` using `completion_filter`.
    ///
    /// Updates `completion_visible` and clamps `completion_selected` if needed.
    ///
    /// # Example
    ///
    /// ```
    /// use iced::Point;
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.set_completions(
    ///     vec!["foo".to_string(), "bar".to_string()],
    ///     Point::ORIGIN,
    /// );
    /// state.completion_filter = "fo".to_string();
    /// state.filter_completions();
    /// assert_eq!(state.completion_items, vec!["foo".to_string()]);
    /// ```
    pub fn filter_completions(&mut self) {
        let filter = self.completion_filter.to_lowercase();
        if filter.is_empty() {
            self.completion_items = self.all_completions.clone();
        } else {
            self.completion_items = self
                .all_completions
                .iter()
                .filter(|item| item.to_lowercase().contains(&filter))
                .cloned()
                .collect();
        }
        self.completion_visible = !self.completion_items.is_empty();
        if self.completion_selected >= self.completion_items.len() {
            self.completion_selected = self.completion_items.len().saturating_sub(1);
        }
    }

    /// Navigates through the completion list by `delta` steps, wrapping at boundaries.
    ///
    /// # Example
    ///
    /// ```
    /// use iced::Point;
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.set_completions(
    ///     vec!["a".to_string(), "b".to_string(), "c".to_string()],
    ///     Point::ORIGIN,
    /// );
    /// state.navigate(1);
    /// assert_eq!(state.completion_selected, 1);
    /// state.navigate(-1);
    /// assert_eq!(state.completion_selected, 0);
    /// ```
    pub fn navigate(&mut self, delta: i32) {
        if self.completion_items.is_empty() {
            return;
        }
        let len = self.completion_items.len();
        let current = self.completion_selected as i32;
        self.completion_selected = ((current + delta).rem_euclid(len as i32)) as usize;
    }

    /// Returns the currently selected completion item, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use iced::Point;
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.set_completions(vec!["foo".to_string()], Point::ORIGIN);
    /// assert_eq!(state.selected_item(), Some("foo"));
    /// ```
    pub fn selected_item(&self) -> Option<&str> {
        self.completion_items.get(self.completion_selected).map(String::as_str)
    }

    /// Returns the vertical scroll offset in pixels to keep the selected
    /// completion item visible when navigating with the keyboard.
    ///
    /// Pass the returned value to `scrollable::AbsoluteOffset::y`.
    ///
    /// # Example
    ///
    /// ```
    /// use iced::Point;
    /// use iced_code_editor::LspOverlayState;
    ///
    /// let mut state = LspOverlayState::new();
    /// state.set_completions(
    ///     vec!["a".to_string(), "b".to_string(), "c".to_string()],
    ///     Point::ORIGIN,
    /// );
    /// state.navigate(2);
    /// assert_eq!(state.scroll_offset_for_selected(), 40.0);
    /// ```
    pub fn scroll_offset_for_selected(&self) -> f32 {
        self.completion_selected as f32 * COMPLETION_ITEM_HEIGHT
    }
}

impl Default for LspOverlayState {
    fn default() -> Self {
        Self::new()
    }
}

/// Messages produced by LSP overlay UI interactions.
///
/// Use these in your application's `update` function to handle hover
/// and completion interactions, driving [`LspOverlayState`] with the matching
/// method ([`LspOverlayState::navigate`], [`LspOverlayState::clear_completions`],
/// and friends). The keyboard variants let a host route Up/Down/Enter to the
/// completion menu while it is open, instead of to the editor.
///
/// # Examples
///
/// ```
/// use iced::Point;
/// use iced_code_editor::{LspOverlayMessage, LspOverlayState};
///
/// let mut state = LspOverlayState::new();
/// state.set_completions(
///     vec!["main".to_string(), "map".to_string()],
///     Point::ORIGIN,
/// );
///
/// // Route the keyboard variants to the overlay while the menu is open.
/// let selected = match LspOverlayMessage::CompletionNavigateDown {
///     LspOverlayMessage::CompletionNavigateDown => {
///         state.navigate(1);
///         state.selected_item()
///     }
///     LspOverlayMessage::CompletionNavigateUp => {
///         state.navigate(-1);
///         state.selected_item()
///     }
///     _ => None,
/// };
/// assert_eq!(selected, Some("map"));
///
/// // Dismissing clears the menu.
/// state.clear_completions();
/// assert!(!state.completion_visible);
/// ```
#[derive(Debug, Clone)]
pub enum LspOverlayMessage {
    /// The mouse cursor entered the hover tooltip area.
    HoverEntered,
    /// The mouse cursor exited the hover tooltip area.
    HoverExited,
    /// A completion item at the given index was clicked.
    CompletionSelected(usize),
    /// The completion menu was dismissed by clicking outside it.
    CompletionClosed,
    /// Navigate up in the completion list (e.g., keyboard Up arrow).
    CompletionNavigateUp,
    /// Navigate down in the completion list (e.g., keyboard Down arrow).
    CompletionNavigateDown,
    /// Confirm the currently highlighted completion item (e.g., Enter key).
    CompletionConfirm,
}

/// Renders LSP overlay elements (hover tooltip and completion menu) on top of a [`CodeEditor`].
///
/// Returns an [`Element`] containing the overlays positioned relative to the editor viewport.
/// The function maps [`LspOverlayMessage`] values to the application message type `M` via `f`.
///
/// # Arguments
///
/// * `state` — current overlay display state
/// * `editor` — the editor this overlay is associated with (used for viewport
///   measurements, and for the font hover code blocks are rendered in)
/// * `theme` — the active Iced theme for styling
/// * `font_size` — font size in points, used for markdown rendering; pass
///   [`CodeEditor::font_size`] so hover text matches the editor
/// * `line_height` — line height in pixels, used for vertical positioning and
///   for hover code lines; pass [`CodeEditor::line_height`]
/// * `f` — mapping function from [`LspOverlayMessage`] to the app's message type
///
/// The hover box opens below the hovered line and flips above it only when it
/// does not fit below and there is more room above. It is sized to its content,
/// capped to the viewport width and to a dozen lines of height (scrolling
/// beyond that), and shifted left when it would cross the viewport's right edge.
///
/// # Example
///
/// ```no_run
/// use iced_code_editor::{
///     CodeEditor, LspOverlayMessage, LspOverlayState, view_lsp_overlay,
/// };
///
/// struct App {
///     editor: CodeEditor,
///     overlay: LspOverlayState,
/// }
///
/// #[derive(Clone)]
/// enum Message {
///     Overlay(LspOverlayMessage),
/// }
///
/// fn view(app: &App) -> iced::Element<'_, Message> {
///     view_lsp_overlay(
///         &app.overlay,
///         &app.editor,
///         &iced::Theme::Dark,
///         app.editor.font_size(),
///         app.editor.line_height(),
///         Message::Overlay,
///     )
/// }
/// ```
pub fn view_lsp_overlay<'a, M: Clone + 'a>(
    state: &'a LspOverlayState,
    editor: &'a CodeEditor,
    theme: &'a Theme,
    font_size: f32,
    line_height: f32,
    f: impl Fn(LspOverlayMessage) -> M + 'a,
) -> Element<'a, M> {
    // Pre-compute messages so we can clone them freely
    let msg_hover_entered = f(LspOverlayMessage::HoverEntered);
    let msg_hover_exited = f(LspOverlayMessage::HoverExited);
    let msg_completion_closed = f(LspOverlayMessage::CompletionClosed);
    let msg_completion_selected: Vec<M> = (0..state.completion_items.len())
        .map(|i| f(LspOverlayMessage::CompletionSelected(i)))
        .collect();

    let mut has_overlay = false;

    // Build the hover tooltip layer
    let hover_layer: Element<'a, M> = build_hover_layer(
        state,
        editor,
        theme,
        (font_size, line_height),
        msg_hover_entered,
        msg_hover_exited,
        &mut has_overlay,
    );

    // Build the auto-completion menu layer
    let completion_layer: Element<'a, M> = build_completion_layer(
        state,
        editor,
        line_height,
        msg_completion_closed,
        msg_completion_selected,
        &mut has_overlay,
    );

    if !has_overlay {
        return container(Space::new().width(Length::Shrink).height(Length::Shrink)).into();
    }

    let base = container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill);

    // Hover appears on top of completion
    stack![base, completion_layer, hover_layer].into()
}

/// Builds the hover tooltip layer.
///
/// The layer fills the editor viewport and places the hover box with spacers.
/// Below the line, a spacer as tall as the line's bottom edge pushes the box's
/// hit area down to it. Above the line, a container as tall as the line's top
/// edge aligns the box to its bottom, so the box ends on the line whatever its
/// rendered height. Either way the transparent [`HOVER_GAP`] sits inside the
/// hit area, which therefore touches the hovered line.
fn build_hover_layer<'a, M: Clone + 'a>(
    state: &'a LspOverlayState,
    editor: &'a CodeEditor,
    theme: &'a Theme,
    text_metrics: (f32, f32),
    msg_entered: M,
    msg_exited: M,
    has_overlay: &mut bool,
) -> Element<'a, M> {
    let (font_size, line_height) = text_metrics;
    if !state.hover_visible {
        return empty_overlay();
    }

    let Some(hover) = state.hover_text.as_ref().filter(|t| !t.trim().is_empty()) else {
        return empty_overlay();
    };

    let mut markdown_style = markdown::Style::from_palette(theme.palette());
    markdown_style.code_block_font = editor.font;
    markdown_style.inline_code_font = editor.font;
    let mut markdown_settings = markdown::Settings::with_text_size(font_size, markdown_style);
    markdown_settings.spacing = (font_size * HOVER_ITEM_SPACING_RATIO).into();

    // Width: the widest content line, capped to the viewport.
    let measure = |line: &str| editor.measure_text_width(line);
    let viewport_width = editor.viewport_width();
    let natural_width =
        hover_content_width(hover, &state.hover_items, &measure).ceil() + HOVER_WIDTH_SLACK;
    let content_max_width = (viewport_width - HOVER_VIEWPORT_INSET - 2.0 * HOVER_PADDING).max(0.0);
    let content_width = if content_max_width > 0.0 {
        natural_width.min(content_max_width)
    } else {
        natural_width
    };
    let hover_width = content_width + 2.0 * HOVER_PADDING;

    // Height: the box shrink-wraps its content up to `max_box_height`; the
    // estimate only picks the side of the line the box opens on.
    let max_box_height = HOVER_MAX_VISIBLE_LINES * line_height + 2.0 * HOVER_PADDING;
    let estimated_height = HeightEstimator {
        settings: markdown_settings,
        line_height,
        measure: &measure,
    }
    .hover_height(&state.hover_items, content_width)
    .min(max_box_height)
        + HOVER_GAP;

    // The anchor is the hovered word's top-left corner in content coordinates.
    let (anchor_x, anchor_top) = match state.hover_position {
        Some(anchor) => (
            anchor.x - editor.horizontal_scroll_offset,
            anchor.y - editor.viewport_scroll(),
        ),
        None => (HOVER_EDGE_MARGIN, HOVER_EDGE_MARGIN),
    };
    let placement = hover_vertical_placement(
        anchor_top,
        line_height,
        estimated_height,
        editor.viewport_height(),
    );
    let offset_x = hover_offset_x(anchor_x, hover_width, viewport_width);
    let (side_space, gap) = match placement {
        HoverPlacement::Below { max_height, .. } => (max_height, padding::top(HOVER_GAP)),
        HoverPlacement::Above { max_height, .. } => (max_height, padding::bottom(HOVER_GAP)),
    };
    let box_max_height = max_box_height.min(side_space - HOVER_GAP).max(0.0);

    let viewer = HoverViewer {
        font: editor.font,
        font_size: editor.font_size(),
        line_height,
    };

    let entered_for_map = msg_entered.clone();
    let entered_for_enter = msg_entered.clone();
    let entered_for_move = msg_entered;

    let hover_content = scrollable(
        container(
            markdown::view_with(&state.hover_items, markdown_settings, &viewer)
                .map(move |_| entered_for_map.clone()),
        )
        .width(Length::Fixed(hover_width))
        .padding(HOVER_PADDING),
    )
    .width(Length::Fixed(hover_width))
    .height(Length::Shrink)
    .style(|theme: &Theme, _status| {
        let palette = theme.extended_palette();
        scrollable::Style {
            container: container::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..container::Style::default()
            },
            vertical_rail: lsp_scrollable_rail(palette),
            horizontal_rail: lsp_scrollable_rail(palette),
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Color::TRANSPARENT.into(),
                border: Border::default(),
                shadow: Shadow::default(),
                icon: Color::TRANSPARENT,
            },
        }
    });

    let hover_box = container(hover_content).max_height(box_max_height).style(hover_box_style);

    // `Idle` makes the popup report an interaction, so the stacks beneath it
    // see a levitating cursor: the editor stops receiving `MouseHover` (which
    // would re-anchor or clear the hover) while the mouse is over the popup.
    let hit_area = mouse_area(container(hover_box).padding(gap))
        .interaction(iced::mouse::Interaction::Idle)
        .on_enter(entered_for_enter)
        .on_move(move |_| entered_for_move.clone())
        .on_exit(msg_exited);

    let positioned = row![Space::new().width(Length::Fixed(offset_x)), hit_area];

    let layer: Element<'a, M> = match placement {
        HoverPlacement::Below { top, .. } => {
            column![Space::new().height(Length::Fixed(top)), positioned].into()
        },
        HoverPlacement::Above { bottom, .. } => container(positioned)
            .width(Length::Fill)
            .align_bottom(Length::Fixed(bottom))
            .into(),
    };

    *has_overlay = true;

    container(layer).width(Length::Fill).height(Length::Fill).into()
}

/// Styles the hover box: the theme's weak background, a faint hairline border
/// and a soft drop shadow.
fn hover_box_style(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(
            theme.extended_palette().background.weak.color,
        )),
        border: Border {
            color: HOVER_HAIRLINE,
            width: 1.0,
            radius: HOVER_RADIUS.into(),
        },
        shadow: Shadow {
            color: HOVER_SHADOW_COLOR,
            offset: Vector::new(0.0, HOVER_SHADOW_OFFSET_Y),
            blur_radius: HOVER_SHADOW_BLUR,
        },
        ..container::Style::default()
    }
}

/// Where the hover box opens relative to the hovered line.
///
/// All values are in pixels, relative to the top of the editor viewport, and
/// describe the box's hit area, which includes the transparent [`HOVER_GAP`].
#[derive(Debug, Clone, Copy, PartialEq)]
enum HoverPlacement {
    /// Below the line: the hit area starts at `top`, the line's bottom edge,
    /// and may extend at most `max_height` down to the viewport's bottom.
    Below {
        /// Top edge of the hit area.
        top: f32,
        /// Space between `top` and the viewport's bottom edge.
        max_height: f32,
    },
    /// Above the line: the hit area ends at `bottom`, the line's top edge,
    /// and may extend at most `max_height` up to the viewport's top.
    Above {
        /// Bottom edge of the hit area.
        bottom: f32,
        /// Space between the viewport's top edge and `bottom`.
        max_height: f32,
    },
}

/// Chooses the side of the hovered line the hover box opens on.
///
/// Below is the default. The box flips above only when `estimated_height`
/// does not fit below the line *and* there is more room above than below, so
/// a box too tall for either side still opens on the roomier one. The returned
/// `max_height` is the space on the chosen side, never negative.
///
/// # Arguments
///
/// * `anchor_top` — top edge of the hovered line, relative to the viewport top
/// * `line_height` — height of the hovered line
/// * `estimated_height` — expected height of the hit area (box plus gap)
/// * `viewport_height` — height of the editor viewport
fn hover_vertical_placement(
    anchor_top: f32,
    line_height: f32,
    estimated_height: f32,
    viewport_height: f32,
) -> HoverPlacement {
    let viewport_height = viewport_height.max(0.0);
    let line_bottom = (anchor_top + line_height).max(0.0).min(viewport_height);
    let space_below = viewport_height - line_bottom;
    let space_above = anchor_top.max(0.0).min(viewport_height);

    if estimated_height > space_below && space_above > space_below {
        HoverPlacement::Above {
            bottom: space_above,
            max_height: space_above,
        }
    } else {
        HoverPlacement::Below {
            top: line_bottom,
            max_height: space_below,
        }
    }
}

/// Returns the left edge of the hover box, relative to the viewport's left.
///
/// The box starts at the hovered word (`anchor_x`, already corrected for
/// horizontal scroll) and shifts left as needed to keep [`HOVER_EDGE_MARGIN`]
/// free at the viewport's right edge, but never past the viewport's left edge.
fn hover_offset_x(anchor_x: f32, hover_width: f32, viewport_width: f32) -> f32 {
    let max_x = (viewport_width - hover_width - HOVER_EDGE_MARGIN).max(0.0);
    anchor_x.min(max_x).max(0.0)
}

/// Returns the natural content width of a hover, in pixels.
///
/// Code blocks render in the editor's font, so their lines, taken from the
/// parsed `items`, measure exactly. Prose renders in the proportional UI font
/// and wraps, so its raw source lines (outside code fences) give an estimate.
/// The result is the widest line of either kind.
fn hover_content_width(
    markdown: &str,
    items: &[markdown::Item],
    measure: &dyn Fn(&str) -> f32,
) -> f32 {
    let mut code_blocks = Vec::new();
    collect_code_blocks(items, &mut code_blocks);
    let code_width = code_blocks
        .iter()
        .flat_map(|code| code.lines())
        .map(measure)
        .fold(0.0, f32::max);
    let prose_width = prose_lines(markdown).map(measure).fold(0.0, f32::max);
    code_width.max(prose_width)
}

/// Appends the code of every code block in `items`, including blocks nested
/// in lists and quotes, to `out`.
fn collect_code_blocks<'a>(items: &'a [markdown::Item], out: &mut Vec<&'a str>) {
    for item in items {
        match item {
            markdown::Item::CodeBlock { code, .. } => out.push(code),
            markdown::Item::Quote(items) => collect_code_blocks(items, out),
            markdown::Item::List { bullets, .. } => {
                for bullet in bullets {
                    collect_code_blocks(bullet_items(bullet), out);
                }
            },
            _ => {},
        }
    }
}

/// Returns the items of a markdown list bullet.
fn bullet_items(bullet: &markdown::Bullet) -> &[markdown::Item] {
    match bullet {
        markdown::Bullet::Point { items } | markdown::Bullet::Task { items, .. } => items,
    }
}

/// Returns the lines of `markdown` outside fenced code blocks, without the
/// fence lines themselves.
fn prose_lines(markdown: &str) -> impl Iterator<Item = &str> {
    let mut fence: Option<&str> = None;
    markdown.lines().filter(move |line| {
        let line = line.trim_start();
        match fence {
            Some(marker) => {
                if line.starts_with(marker) {
                    fence = None;
                }
                false
            },
            None => match ["```", "~~~"].into_iter().find(|marker| line.starts_with(marker)) {
                Some(marker) => {
                    fence = Some(marker);
                    false
                },
                None => true,
            },
        }
    })
}

/// Estimates the rendered height of hover markdown, for the flip decision.
///
/// The estimate aims to be close rather than exact: every rendered line counts
/// as one `line_height` (code lines render at exactly that height), prose
/// wraps at the content width, blocks are separated by the markdown spacing,
/// and headings, lists, quotes and tables are approximated.
struct HeightEstimator<'m> {
    /// The markdown settings the hover is rendered with.
    settings: markdown::Settings,
    /// Height in pixels of one rendered line.
    line_height: f32,
    /// Measures the width in pixels of one line of text.
    measure: &'m dyn Fn(&str) -> f32,
}

impl HeightEstimator<'_> {
    /// Estimates the hover box height for `items` laid out `content_width`
    /// wide, box padding included.
    fn hover_height(&self, items: &[markdown::Item], content_width: f32) -> f32 {
        self.blocks(items, content_width, self.settings.spacing.0) + 2.0 * HOVER_PADDING
    }

    /// Estimates the height of `items` stacked `spacing` apart, `width` wide.
    fn blocks(&self, items: &[markdown::Item], width: f32, spacing: f32) -> f32 {
        let gaps = items.len().saturating_sub(1) as f32 * spacing;
        items.iter().map(|item| self.block(item, width, spacing)).sum::<f32>() + gaps
    }

    /// Estimates the height of one markdown block, `width` wide.
    fn block(&self, item: &markdown::Item, width: f32, spacing: f32) -> f32 {
        let text_size = self.settings.text_size.0;
        match item {
            markdown::Item::CodeBlock { code, .. } => {
                self.wrapped_lines(code, width) * self.line_height
            },
            markdown::Item::Rule => 1.0,
            markdown::Item::Paragraph(text) => {
                self.wrapped_lines(&self.plain_text(text), width) * self.line_height
            },
            markdown::Item::Heading(level, text) => {
                let scale = if text_size > 0.0 {
                    self.heading_size(*level) / text_size
                } else {
                    1.0
                };
                let lines = self.wrapped_lines(&self.plain_text(text), width / scale);
                lines * self.line_height * scale + text_size / 2.0
            },
            markdown::Item::List { bullets, .. } => {
                let width = (width - 2.0 * spacing).max(0.0);
                let gaps = bullets.len().saturating_sub(1) as f32 * spacing;
                bullets
                    .iter()
                    .map(|bullet| self.blocks(bullet_items(bullet), width, spacing))
                    .sum::<f32>()
                    + gaps
            },
            markdown::Item::Quote(items) => self.blocks(items, width, spacing),
            markdown::Item::Table { rows, .. } => {
                (rows.len() + 1) as f32 * (self.line_height + spacing)
            },
            markdown::Item::Image { .. } => self.line_height,
        }
    }

    /// Returns how many lines `text` takes when wrapped at `width`, counting
    /// every hard line at least once.
    fn wrapped_lines(&self, text: &str, width: f32) -> f32 {
        let lines: f32 = text
            .lines()
            .map(|line| {
                if width > 0.0 {
                    ((self.measure)(line) / width).ceil().max(1.0)
                } else {
                    1.0
                }
            })
            .sum();
        lines.max(1.0)
    }

    /// Returns the text size in pixels of a heading of `level`.
    fn heading_size(&self, level: markdown::HeadingLevel) -> f32 {
        let settings = &self.settings;
        match level {
            markdown::HeadingLevel::H1 => settings.h1_size.0,
            markdown::HeadingLevel::H2 => settings.h2_size.0,
            markdown::HeadingLevel::H3 => settings.h3_size.0,
            markdown::HeadingLevel::H4 => settings.h4_size.0,
            markdown::HeadingLevel::H5 => settings.h5_size.0,
            markdown::HeadingLevel::H6 => settings.h6_size.0,
        }
    }

    /// Returns the plain text of parsed markdown text.
    ///
    /// Uses the same style the hover renders with, so the spans cached on
    /// `text` stay valid for rendering.
    fn plain_text(&self, text: &markdown::Text) -> String {
        text.spans(self.settings.style).iter().map(|span| &*span.text).collect()
    }
}

/// Markdown [`Viewer`](markdown::Viewer) for hover content.
///
/// Code blocks, such as the signature, render as plain lines of highlighted
/// text in the editor's font, size and line height, without iced's default
/// code-block chrome (dark box, padding, nested horizontal scrollable), so
/// they look like the code they came from and measure exactly as the editor
/// measures text. Rules render as a faint 1 px divider. Everything else falls
/// back to iced's default rendering.
#[derive(Debug, Clone, Copy)]
struct HoverViewer {
    /// The editor's font, used for code blocks.
    font: Font,
    /// The editor's font size in pixels, used for code blocks.
    font_size: f32,
    /// Height in pixels of one code line.
    line_height: f32,
}

impl<'a> markdown::Viewer<'a, markdown::Uri> for HoverViewer {
    fn on_link_click(url: markdown::Uri) -> markdown::Uri {
        url
    }

    fn code_block(
        &self,
        settings: markdown::Settings,
        _language: Option<&'a str>,
        _code: &'a str,
        lines: &'a [markdown::Text],
    ) -> Element<'a, markdown::Uri> {
        let HoverViewer {
            font,
            font_size,
            line_height,
        } = *self;
        column(lines.iter().map(|line| {
            rich_text(spans_in_font_family(&line.spans(settings.style), font))
                .font(font)
                .size(font_size)
                .line_height(text::LineHeight::Absolute(line_height.into()))
                .into()
        }))
        .into()
    }

    fn rule(&self, _settings: markdown::Settings) -> Element<'a, markdown::Uri> {
        rule::horizontal(1)
            .style(|_theme: &Theme| rule::Style {
                color: HOVER_HAIRLINE,
                radius: 0.0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            })
            .into()
    }
}

/// Moves syntax-highlighted spans onto `font`'s family.
///
/// iced's markdown highlighter gives bold and italic tokens an explicit
/// [`Font::MONOSPACE`], which overrides the font set on the line. Each such
/// span keeps its weight and style but takes `font`'s family, so every token
/// renders in the editor's face.
fn spans_in_font_family(
    spans: &[text::Span<'static, markdown::Uri>],
    font: Font,
) -> Vec<text::Span<'static, markdown::Uri>> {
    spans
        .iter()
        .map(|span| {
            let mut span = span.clone();
            span.font = span.font.map(|token_font| Font {
                family: font.family,
                ..token_font
            });
            span
        })
        .collect()
}

/// Builds the auto-completion menu layer.
fn build_completion_layer<'a, M: Clone + 'a>(
    state: &'a LspOverlayState,
    editor: &'a CodeEditor,
    line_height: f32,
    msg_closed: M,
    msg_selected: Vec<M>,
    has_overlay: &mut bool,
) -> Element<'a, M> {
    if !state.completion_visible || state.completion_items.is_empty() || state.completion_suppressed
    {
        return empty_overlay();
    }

    let visible_count = state.completion_items.len().min(MAX_COMPLETION_ITEMS);
    let menu_height = COMPLETION_HEADER_HEIGHT
        + (visible_count as f32 * COMPLETION_ITEM_HEIGHT)
        + (COMPLETION_PADDING * 2.0);

    let cursor_pos = state.completion_position.unwrap_or(Point::new(4.0, 4.0));
    let viewport_width = editor.viewport_width();
    let viewport_height = editor.viewport_height();
    let viewport_scroll = editor.viewport_scroll();

    let menu_width = COMPLETION_MENU_WIDTH.min(viewport_width - 8.0);
    let adjusted_y = (cursor_pos.y - viewport_scroll).max(0.0);
    let space_below = viewport_height - adjusted_y - line_height;
    let show_above = space_below < menu_height + 4.0 && adjusted_y >= menu_height + 4.0;

    let offset_x = cursor_pos.x.min(viewport_width - menu_width - 4.0).max(4.0);
    let offset_y = if show_above {
        (adjusted_y - menu_height - 4.0).max(0.0)
    } else {
        adjusted_y + line_height + 4.0
    };

    let completion_elements: Vec<Element<'_, M>> = state
        .completion_items
        .iter()
        .enumerate()
        .zip(msg_selected)
        .map(|((index, item), msg)| {
            let is_selected = index == state.completion_selected;
            button(
                text(item.clone())
                    .size(12)
                    .line_height(iced::widget::text::LineHeight::Relative(1.5)),
            )
            .padding([2, 8])
            .width(Length::Fill)
            .on_press(msg)
            .style(move |theme: &Theme, _status| {
                let palette = theme.extended_palette();
                if is_selected {
                    button::Style {
                        background: Some(iced::Background::Color(palette.primary.weak.color)),
                        text_color: Color::WHITE,
                        ..Default::default()
                    }
                } else {
                    button::Style {
                        background: Some(iced::Background::Color(palette.background.weak.color)),
                        text_color: Color::WHITE,
                        ..Default::default()
                    }
                }
            })
            .into()
        })
        .collect();

    let completion_box = scrollable(column(completion_elements).spacing(0))
        .height(Length::Fixed(menu_height))
        .width(Length::Fixed(menu_width))
        .id(Id::new("completion_scrollable"))
        .style(|theme: &Theme, _status| {
            let palette = theme.extended_palette();
            scrollable::Style {
                container: container::Style {
                    background: Some(iced::Background::Color(palette.background.weak.color)),
                    border: iced::Border {
                        color: palette.primary.weak.color,
                        width: 1.0,
                        radius: SCROLLABLE_BORDER_RADIUS.into(),
                    },
                    ..Default::default()
                },
                vertical_rail: lsp_scrollable_rail(palette),
                horizontal_rail: lsp_scrollable_rail(palette),
                gap: None,
                auto_scroll: scrollable::AutoScroll {
                    background: Color::TRANSPARENT.into(),
                    border: Border::default(),
                    shadow: Shadow::default(),
                    icon: Color::TRANSPARENT,
                },
            }
        });

    *has_overlay = true;

    let click_outside = button(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .on_press(msg_closed)
        .style(|_theme: &Theme, _status| button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        });

    let completion_content = container(
        column![
            Space::new().height(Length::Fixed(offset_y)),
            row![Space::new().width(Length::Fixed(offset_x)), completion_box]
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    stack![click_outside, completion_content].into()
}

/// Creates the scrollable rail style used in LSP overlay panels.
///
/// Both the hover tooltip and the completion menu share the same rail appearance:
/// a translucent background track with a primary-coloured scroller and no border.
fn lsp_scrollable_rail(palette: &iced::theme::palette::Extended) -> scrollable::Rail {
    crate::canvas_editor::render::view::scrollable_rail(
        palette.background.weak.color,
        palette.primary.weak.color,
        SCROLLABLE_BORDER_RADIUS,
    )
}

/// Returns a zero-size placeholder element.
fn empty_overlay<'a, M: 'a>() -> Element<'a, M> {
    container(Space::new().width(Length::Shrink).height(Length::Shrink)).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Point;

    #[test]
    fn test_lsp_overlay_state_new() {
        let state = LspOverlayState::new();
        assert!(!state.hover_visible);
        assert!(!state.completion_visible);
        assert!(state.hover_text.is_none());
        assert!(state.completion_items.is_empty());
    }

    #[test]
    fn test_show_hover() {
        let mut state = LspOverlayState::new();
        state.show_hover("hello".to_string());
        assert!(state.hover_visible);
        assert_eq!(state.hover_text, Some("hello".to_string()));
    }

    #[test]
    fn test_clear_hover() {
        let mut state = LspOverlayState::new();
        state.show_hover("hello".to_string());
        state.hover_interactive = true;
        state.hover_position = Some(Point::ORIGIN);
        state.clear_hover();
        assert!(!state.hover_visible);
        assert!(state.hover_text.is_none());
        assert!(!state.hover_interactive);
        assert!(state.hover_position.is_none());
    }

    #[test]
    fn test_set_hover_position() {
        let mut state = LspOverlayState::new();
        state.set_hover_position(Point::new(10.0, 20.0));
        assert_eq!(state.hover_position, Some(Point::new(10.0, 20.0)));
    }

    #[test]
    fn test_set_completions() {
        let mut state = LspOverlayState::new();
        state.set_completions(vec!["foo".to_string(), "bar".to_string()], Point::ORIGIN);
        assert_eq!(state.completion_items.len(), 2);
        assert!(state.completion_visible);
        assert_eq!(state.completion_selected, 0);
    }

    #[test]
    fn test_clear_completions() {
        let mut state = LspOverlayState::new();
        state.set_completions(vec!["foo".to_string()], Point::ORIGIN);
        state.clear_completions();
        assert!(!state.completion_visible);
        assert!(state.all_completions.is_empty());
        assert!(state.completion_items.is_empty());
    }

    #[test]
    fn test_filter_completions() {
        let mut state = LspOverlayState::new();
        state.set_completions(
            vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
            Point::ORIGIN,
        );
        state.completion_filter = "ba".to_string();
        state.filter_completions();
        assert_eq!(state.completion_items.len(), 2);
        assert!(state.completion_items.contains(&"bar".to_string()));
        assert!(state.completion_items.contains(&"baz".to_string()));
    }

    #[test]
    fn test_navigate() {
        let mut state = LspOverlayState::new();
        state.set_completions(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            Point::ORIGIN,
        );
        state.navigate(1);
        assert_eq!(state.completion_selected, 1);
        state.navigate(-1);
        assert_eq!(state.completion_selected, 0);
        // Wrap around going up
        state.navigate(-1);
        assert_eq!(state.completion_selected, 2);
        // Wrap around going down
        state.navigate(1);
        assert_eq!(state.completion_selected, 0);
    }

    #[test]
    // The asserted offsets are `0.0` and exact multiples of
    // `COMPLETION_ITEM_HEIGHT` produced by a single multiplication, never by
    // accumulation, so they are bit-for-bit reproducible. An epsilon
    // comparison here would hide that exactness rather than protect anything.
    #[allow(clippy::float_cmp)]
    fn test_scroll_offset_for_selected() {
        let mut state = LspOverlayState::new();
        assert_eq!(state.scroll_offset_for_selected(), 0.0);
        state.set_completions(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            Point::ORIGIN,
        );
        assert_eq!(state.scroll_offset_for_selected(), 0.0);
        state.navigate(1);
        assert_eq!(state.scroll_offset_for_selected(), COMPLETION_ITEM_HEIGHT);
        state.navigate(1);
        assert_eq!(
            state.scroll_offset_for_selected(),
            2.0 * COMPLETION_ITEM_HEIGHT
        );
    }

    #[test]
    fn test_selected_item() {
        let mut state = LspOverlayState::new();
        assert_eq!(state.selected_item(), None);
        state.set_completions(
            vec!["first".to_string(), "second".to_string()],
            Point::ORIGIN,
        );
        assert_eq!(state.selected_item(), Some("first"));
        state.navigate(1);
        assert_eq!(state.selected_item(), Some("second"));
    }

    // -------------------------------------------------------------------------
    // Hover layout helpers
    // -------------------------------------------------------------------------

    /// Tolerance for comparing computed pixel values.
    const EPSILON: f32 = 1e-3;

    /// Returns whether two pixel values are equal within [`EPSILON`].
    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    /// Measures text as a monospace font 8 px per character would.
    fn measure_8px(text: &str) -> f32 {
        text.chars().count() as f32 * 8.0
    }

    /// Markdown settings at a 14 px text size with 7 px block spacing.
    fn settings_14px() -> markdown::Settings {
        let mut settings = markdown::Settings::with_text_size(
            14.0,
            markdown::Style::from_palette(Theme::Dark.palette()),
        );
        settings.spacing = 7.0.into();
        settings
    }

    #[test]
    fn test_hover_vertical_placement_defaults_below() {
        // Line top at 100 px with 20 px lines: a 120 px box fits below.
        let placement = hover_vertical_placement(100.0, 20.0, 120.0, 600.0);
        assert!(
            matches!(
                placement,
                HoverPlacement::Below { top, max_height }
                    if approx_eq(top, 120.0) && approx_eq(max_height, 480.0)
            ),
            "got {placement:?}"
        );
    }

    #[test]
    fn test_hover_vertical_placement_flips_above_when_no_room() {
        // Only 80 px below the line, 500 px above it.
        let placement = hover_vertical_placement(500.0, 20.0, 120.0, 600.0);
        assert!(
            matches!(
                placement,
                HoverPlacement::Above { bottom, max_height }
                    if approx_eq(bottom, 500.0) && approx_eq(max_height, 500.0)
            ),
            "got {placement:?}"
        );
    }

    #[test]
    fn test_hover_vertical_placement_stays_below_when_above_is_smaller() {
        // The box fits on neither side, but below (140 px) beats above (40 px).
        let placement = hover_vertical_placement(40.0, 20.0, 300.0, 200.0);
        assert!(
            matches!(
                placement,
                HoverPlacement::Below { top, max_height }
                    if approx_eq(top, 60.0) && approx_eq(max_height, 140.0)
            ),
            "got {placement:?}"
        );
    }

    #[test]
    fn test_hover_vertical_placement_max_height_never_negative() {
        let cases = [
            // Line partly below the viewport's bottom edge.
            (590.0, 20.0, 0.0, 600.0),
            // Line partly above the viewport's top edge.
            (-10.0, 20.0, 500.0, 100.0),
            // Viewport not laid out yet.
            (10.0, 20.0, 50.0, 0.0),
            // Negative viewport height.
            (10.0, 20.0, 50.0, -5.0),
        ];
        for (anchor_top, line_height, estimated_height, viewport_height) in cases {
            let placement = hover_vertical_placement(
                anchor_top,
                line_height,
                estimated_height,
                viewport_height,
            );
            let (edge, max_height) = match placement {
                HoverPlacement::Below { top, max_height } => (top, max_height),
                HoverPlacement::Above { bottom, max_height } => (bottom, max_height),
            };
            assert!(max_height >= 0.0, "negative max_height in {placement:?}");
            assert!(edge >= 0.0, "edge outside the viewport in {placement:?}");
        }
    }

    #[test]
    fn test_hover_offset_x_starts_at_anchor() {
        assert!(approx_eq(hover_offset_x(100.0, 200.0, 800.0), 100.0));
    }

    #[test]
    fn test_hover_offset_x_clamps_to_viewport() {
        // Too far right: shift left to keep the edge margin free.
        let max_x = 800.0 - 200.0 - HOVER_EDGE_MARGIN;
        assert!(approx_eq(hover_offset_x(700.0, 200.0, 800.0), max_x));
        // Left of the viewport (scrolled horizontally): pin to its left edge.
        assert!(approx_eq(hover_offset_x(-50.0, 200.0, 800.0), 0.0));
    }

    #[test]
    fn test_hover_offset_x_narrow_viewport_is_zero() {
        assert!(approx_eq(hover_offset_x(100.0, 300.0, 200.0), 0.0));
    }

    #[test]
    fn test_prose_lines_skips_fenced_code() {
        let markdown = "```rust\nfn main()\n```\nprose\n~~~\ncode\n~~~\n---\nmore";
        let lines: Vec<&str> = prose_lines(markdown).collect();
        assert_eq!(lines, vec!["prose", "---", "more"]);
    }

    #[test]
    fn test_hover_content_width_uses_widest_line() {
        // The signature is wider than any prose line (or the fence lines).
        let markdown = "```rust\nfn long_signature()\n```\n\nshort docs";
        let items: Vec<markdown::Item> = markdown::parse(markdown).collect();
        let width = hover_content_width(markdown, &items, &measure_8px);
        assert!(
            approx_eq(width, measure_8px("fn long_signature()")),
            "got {width}"
        );

        // A prose line wider than the code wins.
        let markdown = "```rust\nfn f()\n```\n\nthese docs are the widest";
        let items: Vec<markdown::Item> = markdown::parse(markdown).collect();
        let width = hover_content_width(markdown, &items, &measure_8px);
        assert!(
            approx_eq(width, measure_8px("these docs are the widest")),
            "got {width}"
        );
    }

    #[test]
    fn test_hover_height_estimate_counts_code_rule_and_prose() {
        let items: Vec<markdown::Item> =
            markdown::parse("```rust\nfn foo()\n```\n\n---\n\nSome docs.").collect();
        let estimator = HeightEstimator {
            settings: settings_14px(),
            line_height: 20.0,
            measure: &measure_8px,
        };
        // Code line 20 + spacing 7 + rule 1 + spacing 7 + prose line 20,
        // plus the box padding on both sides.
        let expected = 20.0 + 7.0 + 1.0 + 7.0 + 20.0 + 2.0 * HOVER_PADDING;
        assert!(approx_eq(estimator.hover_height(&items, 400.0), expected));
    }

    #[test]
    fn test_hover_height_estimate_wraps_prose() {
        // 99 characters at 8 px is 792 px: two lines in a 400 px box.
        let paragraph = "word ".repeat(20);
        let items: Vec<markdown::Item> = markdown::parse(paragraph.trim_end()).collect();
        let estimator = HeightEstimator {
            settings: settings_14px(),
            line_height: 20.0,
            measure: &measure_8px,
        };
        let height = estimator.hover_height(&items, 400.0);
        assert!(
            approx_eq(height, 2.0 * 20.0 + 2.0 * HOVER_PADDING),
            "got {height}"
        );
    }
}
