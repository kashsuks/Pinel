//! drawing the lanes and dots of the commit graph

use iced::{
    mouse,
    widget::{
        canvas::{self, Canvas, Frame, Geometry, Path, Stroke},
        container, row, text,
    },
    Background, Border, Color, Element, Length, Point, Rectangle, Renderer, Theme,
};

use crate::{
    features::{
        git::{Commit, GitRef, RefKind},
        git_graph::{GraphRow, Segment},
    },
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

/// rough width of one character at the subject's text size
const SUBJECT_CHAR_WIDTH: f32 = 6.4;
/// rough width of one character at the pill's text size
const PILL_CHAR_WIDTH: f32 = 5.6;
/// padding and gap around a pill's text
const PILL_CHROME: f32 = 14.0;
/// most pills shown on one commit before the rest fold into `+N`
const MAX_PILLS: usize = 2;
/// space the panel keeps for its own padding and the scrollbar
const PANEL_CHROME: f32 = 24.0;
/// gap between the graph, the pills and the subject
const CELL_GAP: f32 = 6.0;

/// Cuts `subject` to at most `max_chars` characters, ending in `…` if cut.
///
/// Counts characters rather than bytes so multi-byte text never splits.
pub fn truncate_subject(subject: &str, max_chars: usize) -> String {
    if subject.chars().count() <= max_chars {
        return subject.to_string();
    }
    match max_chars {
        0 => String::new(),
        1 => "…".to_string(),
        n => {
            let mut cut: String = subject.chars().take(n - 1).collect();
            cut.push('…');
            cut
        },
    }
}

/// The refs that get a pill, and how many were left out.
pub fn visible_refs(refs: &[GitRef]) -> (&[GitRef], usize) {
    let shown = refs.len().min(MAX_PILLS);
    (&refs[..shown], refs.len() - shown)
}

/// pixels the pills for these refs take up, including the `+N` marker
fn pills_width(refs: &[GitRef]) -> f32 {
    let (shown, hidden) = visible_refs(refs);
    let mut width: f32 = shown
        .iter()
        .map(|r| r.name.chars().count() as f32 * PILL_CHAR_WIDTH + PILL_CHROME + CELL_GAP)
        .sum();
    if hidden > 0 {
        width += format!("+{hidden}").len() as f32 * PILL_CHAR_WIDTH + PILL_CHROME + CELL_GAP;
    }
    width
}

/// How many subject characters fit beside the graph and pills.
///
/// Grows as the sidebar is dragged wider, which is the point: a wider
/// panel shows more of each message.
pub fn subject_budget(panel_width: f32, lanes: usize, refs: &[GitRef]) -> usize {
    let room = panel_width - PANEL_CHROME - graph_width(lanes) - CELL_GAP - pills_width(refs);
    (room.max(0.0) / SUBJECT_CHAR_WIDTH) as usize
}

fn ref_color(git_ref: &GitRef) -> Color {
    match git_ref.kind {
        _ if git_ref.is_head => Color::from_rgb(0.40, 0.69, 0.98),
        RefKind::LocalBranch => Color::from_rgb(0.36, 0.86, 0.42),
        RefKind::RemoteBranch => Color::from_rgb(0.80, 0.52, 0.96),
        RefKind::Tag => Color::from_rgb(0.98, 0.74, 0.18),
        RefKind::Head => Color::from_rgb(0.40, 0.69, 0.98),
    }
}

fn pill<'a>(label: String, color: Color) -> Element<'a, Message> {
    container(text(label).size(10).color(Color::from_rgb(0.08, 0.09, 0.12)))
        .padding(iced::Padding {
            top: 1.0,
            right: 6.0,
            bottom: 1.0,
            left: 6.0,
        })
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                radius: 8.0.into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .into()
}

/// One commit's line: graph cell, ref pills, then as much subject as fits.
pub fn view_commit_row<'a>(
    commit: &'a Commit,
    graph_row: &'a GraphRow,
    lanes: usize,
    panel_width: f32,
) -> Element<'a, Message> {
    let color = if graph_row.is_merge {
        theme().text_muted
    } else {
        theme().text_primary
    };
    let (shown, hidden) = visible_refs(&commit.refs);

    let mut line = row![view_graph_row(graph_row, lanes)]
        .spacing(CELL_GAP)
        .align_y(iced::Alignment::Center)
        .height(Length::Fixed(ROW_HEIGHT));
    for git_ref in shown {
        line = line.push(pill(git_ref.name.clone(), ref_color(git_ref)));
    }
    if hidden > 0 {
        line = line.push(pill(format!("+{hidden}"), theme().text_muted));
    }

    let budget = subject_budget(panel_width, lanes, &commit.refs);
    line.push(
        text(truncate_subject(&commit.subject, budget))
            .size(12)
            .color(color)
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .into()
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

    fn git_ref(name: &str) -> GitRef {
        GitRef {
            name: name.to_string(),
            kind: RefKind::LocalBranch,
            is_head: false,
        }
    }

    #[test]
    fn test_truncate_subject_short_text_untouched() {
        assert_eq!(truncate_subject("fix bug", 7), "fix bug");
        assert_eq!(truncate_subject("fix bug", 50), "fix bug");
    }

    #[test]
    fn test_truncate_subject_adds_ellipsis_within_limit() {
        let cut = truncate_subject("fix the bug", 6);
        assert_eq!(cut, "fix t…");
        assert_eq!(cut.chars().count(), 6);
    }

    #[test]
    fn test_truncate_subject_edge_limits() {
        assert_eq!(truncate_subject("abc", 0), "");
        assert_eq!(truncate_subject("abc", 1), "…");
    }

    #[test]
    fn test_truncate_subject_never_splits_multibyte() {
        assert_eq!(truncate_subject("héllo wörld", 5), "héll…");
        assert_eq!(truncate_subject("日本語のテスト", 3), "日本…");
    }

    #[test]
    fn test_subject_budget_grows_with_width() {
        let narrow = subject_budget(200.0, 2, &[]);
        let wide = subject_budget(400.0, 2, &[]);
        assert!(wide > narrow);
    }

    #[test]
    fn test_subject_budget_shrinks_for_pills_and_lanes() {
        let plain = subject_budget(300.0, 1, &[]);
        assert!(subject_budget(300.0, 1, &[git_ref("master")]) < plain);
        assert!(subject_budget(300.0, 4, &[]) < plain);
    }

    #[test]
    fn test_subject_budget_never_negative() {
        assert_eq!(
            subject_budget(10.0, 8, &[git_ref("a-very-long-branch-name")]),
            0
        );
    }

    #[test]
    fn test_visible_refs_folds_overflow() {
        let refs = vec![git_ref("a"), git_ref("b"), git_ref("c"), git_ref("d")];
        let (shown, hidden) = visible_refs(&refs);
        assert_eq!(shown.len(), MAX_PILLS);
        assert_eq!(hidden, 2);
        assert_eq!(visible_refs(&refs[..1]).1, 0);
    }

    #[test]
    fn test_lane_x_centres_in_lane() {
        assert_eq!(lane_x(0), LANE_WIDTH / 2.0);
        assert_eq!(lane_x(2), 2.0 * LANE_WIDTH + LANE_WIDTH / 2.0);
    }
}
