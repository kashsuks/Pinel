//! Gutter rendering: line numbers, wrap indicators, and fold chevrons for
//! [`CodeEditor`].

use iced::Point;
use iced::widget::canvas;

use super::text::RenderContext;
use super::wrapping::VisualLine;
use crate::canvas_editor::features::folding;
use crate::canvas_editor::{CodeEditor, LINE_NUMBER_RIGHT_PADDING, measure_text_width};

impl CodeEditor {
    /// Draws line numbers and wrap indicators in the gutter area.
    ///
    /// # Arguments
    ///
    /// * `frame` - The canvas frame to draw on
    /// * `ctx` - Rendering context containing visual lines and metrics
    /// * `visual_line` - The visual line to render
    /// * `y` - Y position for rendering
    pub(super) fn draw_line_numbers(
        &self,
        frame: &mut canvas::Frame,
        ctx: &RenderContext,
        visual_line: &VisualLine,
        y: f32,
    ) {
        // The line-number area is the left part of the gutter; the fold margin
        // (when folding is enabled) is the right strip adjacent to the text.
        let number_area_width = self.line_number_gutter_width();

        if self.line_numbers_enabled {
            if visual_line.is_first_segment() {
                // Draw line number for first segment, right-aligned against the fold margin.
                let line_num = visual_line.logical_line + 1;
                let line_num_text = format!("{}", line_num);
                let x_pos = self.line_number_x(ctx, &line_num_text);
                frame.fill_text(canvas::Text {
                    content: line_num_text,
                    position: Point::new(x_pos, y + 2.0),
                    color: self.style.line_number_color,
                    size: ctx.font_size.into(),
                    font: ctx.font,
                    ..canvas::Text::default()
                });
            } else {
                // Draw wrap indicator for continuation lines.
                frame.fill_text(canvas::Text {
                    content: "↪".to_string(),
                    position: Point::new(self.line_number_x(ctx, "↪"), y + 2.0),
                    color: self.style.line_number_color,
                    size: ctx.font_size.into(),
                    font: ctx.font,
                    ..canvas::Text::default()
                });
            }
        }

        self.draw_fold_chevron(frame, ctx, visual_line, y, number_area_width);
    }

    /// Returns the x position at which `label` must start so that its right
    /// edge sits [`LINE_NUMBER_RIGHT_PADDING`] pixels left of the fold margin.
    fn line_number_x(&self, ctx: &RenderContext, label: &str) -> f32 {
        let width = measure_text_width(label, ctx.full_char_width, ctx.char_width);
        (self.line_number_gutter_width() - width - LINE_NUMBER_RIGHT_PADDING).max(0.0)
    }

    /// Redraws the line number of a line holding a cursor in
    /// `line_number_active_color`, on top of the dimmed number drawn by the content layer.
    pub(super) fn draw_active_line_number(
        &self,
        frame: &mut canvas::Frame,
        ctx: &RenderContext,
        visual_line: &VisualLine,
        y: f32,
    ) {
        if !self.line_numbers_enabled || !visual_line.is_first_segment() {
            return;
        }
        if !self.cursors.iter().any(|c| c.position.0 == visual_line.logical_line) {
            return;
        }

        let label = (visual_line.logical_line + 1).to_string();
        let x_pos = self.line_number_x(ctx, &label);
        frame.fill_text(canvas::Text {
            content: label,
            position: Point::new(x_pos, y + 2.0),
            color: self.style.line_number_active_color,
            size: ctx.font_size.into(),
            font: ctx.font,
            ..canvas::Text::default()
        });
    }

    /// Draws the fold chevron in the fold margin for a foldable header line.
    ///
    /// Draws nothing when folding is disabled, on continuation (wrapped)
    /// segments, or on lines that are not fold headers.
    ///
    /// # Arguments
    ///
    /// * `frame` - The canvas frame to draw on
    /// * `ctx` - Rendering context containing metrics
    /// * `visual_line` - The visual line to render
    /// * `y` - Y position for rendering
    /// * `number_area_width` - Width of the line-number area (start of the fold margin)
    fn draw_fold_chevron(
        &self,
        frame: &mut canvas::Frame,
        ctx: &RenderContext,
        visual_line: &VisualLine,
        y: f32,
        number_area_width: f32,
    ) {
        if !self.folding_enabled || !visual_line.is_first_segment() {
            return;
        }

        if !folding::is_line_fold_header(&self.buffer, visual_line.logical_line) {
            return;
        }

        // `▶` when collapsed, `▼` when expanded.
        let chevron = if self.is_folded(visual_line.logical_line) {
            "▶"
        } else {
            "▼"
        };
        frame.fill_text(canvas::Text {
            content: chevron.to_string(),
            position: Point::new(number_area_width + 1.0, y + 2.0),
            color: self.style.line_number_color,
            size: ctx.font_size.into(),
            font: ctx.font,
            ..canvas::Text::default()
        });
    }

    /// Draws a `⋯` marker after the text of a collapsed fold header, signalling
    /// that lines are hidden below it (VS Code-style cue).
    ///
    /// Draws nothing unless folding is enabled and `visual_line` is the header
    /// of a currently collapsed region. Intended to be called inside the clipped
    /// code area so the marker cannot bleed into the gutter.
    pub(super) fn draw_fold_collapsed_marker(
        &self,
        frame: &mut canvas::Frame,
        ctx: &RenderContext,
        visual_line: &VisualLine,
        y: f32,
    ) {
        if !self.folding_enabled
            || !visual_line.is_first_segment()
            || !self.is_folded(visual_line.logical_line)
        {
            return;
        }

        let line_content = self.buffer.line(visual_line.logical_line);
        let line_width = measure_text_width(line_content, ctx.full_char_width, ctx.char_width);
        let x = ctx.text_start_x() - ctx.horizontal_scroll_offset + line_width + 6.0;
        frame.fill_text(canvas::Text {
            content: "⋯".to_string(),
            position: Point::new(x, y + 2.0),
            color: self.style.line_number_color,
            size: ctx.font_size.into(),
            font: ctx.font,
            ..canvas::Text::default()
        });
    }
}
