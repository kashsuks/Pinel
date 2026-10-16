//! drawing the lanes and dots of the commit graph

use iced::{
    mouse,
    widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke},
    Color, Element, Length, Point, Rectangle, Renderer, Theme,
};

use crate::{
    features::git_graph::{GraphRow, Segment},
    message::Message,
    theme::theme,
};

/// height of one commit row, shared with the text beside the graph
pub const ROW_HEIGHT: f32 = 22.0;
/// horizontal space given to each lane
pub const LANE_WIDTH: f32 = 14.0;
/// lanes beyond this are clipped so a wide graph cannot crowd out the text
pub const MAX_LANES: usize = 8;
const DOT_RADIUS: f32 = 4.0;
const LINE_WIDTH: f32 = 1.5;

/// colours the lanes cycle through
const PALETTE: [Color; 8] = [
    Color::from_rgb(0.40, 0.69, 0.98),
    Color::from_rgb(0.36, 0.86, 0.42),
    Color::from_rgb(0.98, 0.74, 0.18),
    Color::from_rgb(0.80, 0.52, 0.96),
    Color::from_rgb(0.92, 0.37, 0.37),
    Color::from_rgb(0.30, 0.85, 0.80),
    Color::from_rgb(0.98, 0.55, 0.30),
    Color::from_rgb(0.95, 0.50, 0.75),
];

/// the colour for a colour index, wrapping around the palette
pub fn lane_color(index: usize) -> Color {
    PALETTE[index % PALETTE.len()]
}

/// the width of the graph column when the widest row needs `lanes` lanes
pub fn graph_width(lanes: usize) -> f32 {
    lanes.clamp(1, MAX_LANES) as f32 * LANE_WIDTH
}

fn lane_x(lane: usize) -> f32 {
    lane as f32 * LANE_WIDTH + LANE_WIDTH / 2.0
}

/// a straight line when the lanes match, an S-curve when they do not
fn connector(from: Point, to: Point) -> Path {
    Path::new(|builder| {
        builder.move_to(from);
        if (from.x - to.x).abs() < f32::EPSILON {
            builder.line_to(to);
        } else {
            let mid_y = (from.y + to.y) / 2.0;
            builder.bezier_curve_to(Point::new(from.x, mid_y), Point::new(to.x, mid_y), to);
        }
    })
}

/// draws one row of the graph
struct RowProgram<'a> {
    row: &'a GraphRow,
}

impl<Message> canvas::Program<Message> for RowProgram<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let (top, middle, bottom) = (0.0, bounds.height / 2.0, bounds.height);
        let stroke = |segment: &Segment| {
            Stroke::default().with_color(lane_color(segment.color)).with_width(LINE_WIDTH)
        };

        for segment in &self.row.passing {
            let path = connector(
                Point::new(lane_x(segment.from), top),
                Point::new(lane_x(segment.to), bottom),
            );
            frame.stroke(&path, stroke(segment));
        }
        for segment in &self.row.incoming {
            let path = connector(
                Point::new(lane_x(segment.from), top),
                Point::new(lane_x(segment.to), middle),
            );
            frame.stroke(&path, stroke(segment));
        }
        for segment in &self.row.outgoing {
            let path = connector(
                Point::new(lane_x(segment.from), middle),
                Point::new(lane_x(segment.to), bottom),
            );
            frame.stroke(&path, stroke(segment));
        }

        let center = Point::new(lane_x(self.row.lane), middle);
        let dot = Path::circle(center, DOT_RADIUS);
        let color = lane_color(self.row.color);
        if self.row.is_merge {
            // hollow, so a merge reads differently from an ordinary commit
            frame.fill(&dot, theme().bg_secondary);
            frame.stroke(
                &dot,
                Stroke::default().with_color(color).with_width(LINE_WIDTH),
            );
        } else {
            frame.fill(&dot, color);
        }

        vec![frame.into_geometry()]
    }
}

/// the graph cell for one commit, `lanes` wide
pub fn view_graph_row<'a>(row: &'a GraphRow, lanes: usize) -> Element<'a, Message> {
    Canvas::new(RowProgram { row })
        .width(Length::Fixed(graph_width(lanes)))
        .height(Length::Fixed(ROW_HEIGHT))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lane_color_wraps_around_palette() {
        assert_eq!(lane_color(0), lane_color(PALETTE.len()));
        assert_ne!(lane_color(0), lane_color(1));
    }

    #[test]
    fn test_graph_width_is_clamped() {
        assert_eq!(graph_width(0), LANE_WIDTH);
        assert_eq!(graph_width(3), 3.0 * LANE_WIDTH);
        assert_eq!(graph_width(500), MAX_LANES as f32 * LANE_WIDTH);
    }

    #[test]
    fn test_lane_x_centres_in_lane() {
        assert_eq!(lane_x(0), LANE_WIDTH / 2.0);
        assert_eq!(lane_x(2), 2.0 * LANE_WIDTH + LANE_WIDTH / 2.0);
    }
}
