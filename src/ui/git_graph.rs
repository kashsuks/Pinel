//! drawing the lanes and dots of the commit graph

use iced::{
    mouse,
    widget::{
        canvas::{self, Canvas, Frame, Geometry, Path, Stroke},
        container, mouse_area, row, text,
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
/// most characters a pill shows before its name is cut with `…`
const MAX_PILL_CHARS: usize = 24;
/// fewest characters a pill is cut to, however narrow the panel
const MIN_PILL_CHARS: usize = 6;
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

/// Fits a ref name into `max_chars`, keeping the part that tells refs apart.
///
/// When the name is too long, each leading path segment shrinks to its first
/// letter, so `upstream/mxg-add-balance-alerts` becomes
/// `u/mxg-add-balance-alerts`, and only then is the end cut with `…`.
pub fn short_ref_name(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        return name.to_string();
    }
    let abbreviated = match name.rsplit_once('/') {
        Some((dirs, last)) => {
            let mut short: String = dirs
                .split('/')
                .filter_map(|segment| segment.chars().next())
                .flat_map(|first| [first, '/'])
                .collect();
            short.push_str(last);
            short
        },
        None => name.to_string(),
    };
    truncate_subject(&abbreviated, max_chars)
}

/// The refs that get a pill, and how many were left out.
pub fn visible_refs(refs: &[GitRef]) -> (&[GitRef], usize) {
    let shown = refs.len().min(MAX_PILLS);
    (&refs[..shown], refs.len() - shown)
}

/// How many characters each pill may show beside a graph `lanes` wide.
///
/// Pills share at most half the room left of the graph, so a commit with
/// long branch names still shows some of its subject.
pub fn pill_char_budget(panel_width: f32, lanes: usize, refs: &[GitRef]) -> usize {
    let (shown, hidden) = visible_refs(refs);
    let pills = shown.len() + usize::from(hidden > 0);
    if pills == 0 {
        return MAX_PILL_CHARS;
    }
    let room = panel_width - PANEL_CHROME - graph_width(lanes) - CELL_GAP;
    let per_pill = room.max(0.0) / 2.0 / pills as f32 - PILL_CHROME - CELL_GAP;
    ((per_pill.max(0.0) / PILL_CHAR_WIDTH) as usize).clamp(MIN_PILL_CHARS, MAX_PILL_CHARS)
}

/// pixels the pills for these refs take up, including the `+N` marker
fn pills_width(refs: &[GitRef], max_chars: usize) -> f32 {
    let (shown, hidden) = visible_refs(refs);
    let mut width: f32 = shown
        .iter()
        .map(|r| {
            short_ref_name(&r.name, max_chars).chars().count() as f32 * PILL_CHAR_WIDTH
                + PILL_CHROME
                + CELL_GAP
        })
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
    let pills = pills_width(refs, pill_char_budget(panel_width, lanes, refs));
    let room = panel_width - PANEL_CHROME - graph_width(lanes) - CELL_GAP - pills;
    (room.max(0.0) / SUBJECT_CHAR_WIDTH) as usize
}

/// the colour of the pill for a ref
pub fn ref_color(git_ref: &GitRef) -> Color {
    match git_ref.kind {
        _ if git_ref.is_head => Color::from_rgb(0.40, 0.69, 0.98),
        RefKind::LocalBranch => Color::from_rgb(0.36, 0.86, 0.42),
        RefKind::RemoteBranch => Color::from_rgb(0.80, 0.52, 0.96),
        RefKind::Tag => Color::from_rgb(0.98, 0.74, 0.18),
        RefKind::Head => Color::from_rgb(0.40, 0.69, 0.98),
    }
}

/// a rounded label in `color`
pub fn pill<'a>(label: String, color: Color) -> Element<'a, Message> {
    container(
        text(label)
            .size(10)
            .color(Color::from_rgb(0.08, 0.09, 0.12))
            // a wrapped pill grows taller than its row and spills onto the next
            .wrapping(iced::widget::text::Wrapping::None),
    )
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
///
/// The graph cell is only as wide as the lanes this row touches, so the
/// text starts right after the rightmost line rather than in one column.
/// The whole row reports when the cursor enters or leaves it.
pub fn view_commit_row<'a>(
    commit: &'a Commit,
    graph_row: &'a GraphRow,
    panel_width: f32,
) -> Element<'a, Message> {
    let lanes = graph_row.width;
    let color = if graph_row.is_merge {
        theme().text_muted
    } else {
        theme().text_primary
    };
    let (shown, hidden) = visible_refs(&commit.refs);
    let pill_chars = pill_char_budget(panel_width, lanes, &commit.refs);

    let mut line = row![view_graph_row(graph_row)]
        .spacing(CELL_GAP)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fixed(ROW_HEIGHT));
    for git_ref in shown {
        line = line.push(pill(
            short_ref_name(&git_ref.name, pill_chars),
            ref_color(git_ref),
        ));
    }
    if hidden > 0 {
        line = line.push(pill(format!("+{hidden}"), theme().text_muted));
    }

    let budget = subject_budget(panel_width, lanes, &commit.refs);
    let line = line.push(
        text(truncate_subject(&commit.subject, budget))
            .size(12)
            .color(color)
            .wrapping(iced::widget::text::Wrapping::None),
    );

    mouse_area(line)
        .on_enter(Message::GitCommitHovered(commit.hash.clone()))
        .on_exit(Message::GitCommitUnhovered)
        .into()
}

/// the graph cell for one commit, as wide as the lanes it touches
pub fn view_graph_row<'a>(row: &'a GraphRow) -> Element<'a, Message> {
    Canvas::new(RowProgram { row })
        .width(Length::Fixed(graph_width(row.width)))
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
    fn test_short_ref_name_fits_untouched() {
        assert_eq!(short_ref_name("origin/main", 24), "origin/main");
    }

    #[test]
    fn test_short_ref_name_abbreviates_leading_segments() {
        assert_eq!(
            short_ref_name("upstream/mxg-add-balance-alerts", 24),
            "u/mxg-add-balance-alerts"
        );
        assert_eq!(
            short_ref_name("upstream/gh-readonly-queue/main/pr-15185", 24),
            "u/g/m/pr-15185"
        );
    }

    #[test]
    fn test_short_ref_name_cuts_after_abbreviating() {
        let short = short_ref_name("upstream/gh-readonly-queue/main/pr-14996-975d8b0c", 12);
        assert_eq!(short, "u/g/m/pr-14…");
        assert_eq!(short.chars().count(), 12);
    }

    #[test]
    fn test_short_ref_name_without_slash_only_cuts() {
        assert_eq!(short_ref_name("a-very-long-branch-name", 8), "a-very-…");
    }

    #[test]
    fn test_pill_char_budget_capped_on_a_wide_panel() {
        assert_eq!(
            pill_char_budget(2000.0, 1, &[git_ref("main")]),
            MAX_PILL_CHARS
        );
    }

    #[test]
    fn test_pill_char_budget_floored_on_a_narrow_panel() {
        let refs = [git_ref("a"), git_ref("b"), git_ref("c")];
        assert_eq!(pill_char_budget(50.0, 8, &refs), MIN_PILL_CHARS);
    }

    #[test]
    fn test_pill_char_budget_leaves_room_for_the_subject() {
        let long = "upstream/gh-readonly-queue/main/pr-14996-975d8b0c989c2fb3bc4d83445aca5dc4";
        let refs = [git_ref(long), git_ref(long)];
        assert!(subject_budget(300.0, 4, &refs) > 0);
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
