//! Windowing maths for long list whose rows all share one height

use std::ops::Range;

pub fn visible_range(
    offset: f32,
    viewport: f32,
    row_height: f32,
    total: usize,
    overscan: usize,
) -> Range<usize> {
    if total == 0 || row_height <= 0.0 {
        return 0..0;
    }
    let first = (offset / row_height).floor().max(0.0) as usize;
    let last = ((offset + viewport.max(0.0)) / row_height).ceil().max(0.0) as usize;
    let start = first.saturating_sub(overscan).min(total);
    let end = last.saturating_add(overscan).min(total);
    start..end
}

/// Keeps `offset` inside a list of `total` rows.
///
/// The scrollable clamps itself when the list shrinks but reports nothing, so
/// a stored offset can end up past the new end.
pub fn clamp_offset(offset: f32, viewport: f32, total: usize, row_height: f32) -> f32 {
    let max = (total as f32 * row_height - viewport).max(0.0);
    offset.max(0.0).min(max)
}

pub fn spacer_heights(range: &Range<usize>, total: usize, row_height: f32) -> (f32, f32) {
    let above = range.start as f32 * row_height;
    let below = total.saturating_sub(range.end) as f32 * row_height;
    (above, below)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_range_empty_list() {
        assert_eq!(visible_range(0.0, 500.0, 22.0, 0, 3), 0..0);
    }

    #[test]
    fn test_visible_range_top_of_list() {
        assert_eq!(visible_range(0.0, 220.0, 22.0, 1000, 0), 0..10);
    }

    #[test]
    fn test_visible_range_middle_with_overscan() {
        assert_eq!(visible_range(220.0, 220.0, 22.0, 1000, 2), 8..22);
    }

    #[test]
    fn test_visible_range_past_the_end_is_empty_at_total() {
        assert_eq!(visible_range(99_999.0, 500.0, 22.0, 10, 0), 10..10);
    }

    #[test]
    fn test_visible_range_end_is_clamped() {
        assert_eq!(visible_range(0.0, 10_000.0, 22.0, 5, 4), 0..5);
    }

    #[test]
    fn test_visible_range_list_below_viewport_top() {
        // the list starts 100px below the top of the viewport
        assert_eq!(visible_range(-100.0, 220.0, 22.0, 1000, 0), 0..6);
    }

    #[test]
    fn test_visible_range_list_entirely_below_viewport() {
        assert_eq!(visible_range(-5_000.0, 220.0, 22.0, 1000, 0), 0..0);
    }

    #[test]
    fn test_visible_range_rejects_bad_row_height() {
        assert_eq!(visible_range(0.0, 500.0, 0.0, 100, 2), 0..0);
        assert_eq!(visible_range(0.0, 500.0, -1.0, 100, 2), 0..0);
    }

    #[test]
    fn test_clamp_offset_inside_list_is_unchanged() {
        assert_eq!(clamp_offset(100.0, 200.0, 100, 22.0), 100.0);
    }

    #[test]
    fn test_clamp_offset_past_the_end_stops_at_the_last_screen() {
        assert_eq!(clamp_offset(99_999.0, 200.0, 100, 22.0), 2_000.0);
    }

    #[test]
    fn test_clamp_offset_short_list_is_zero() {
        assert_eq!(clamp_offset(500.0, 1_000.0, 10, 22.0), 0.0);
    }

    #[test]
    fn test_clamp_offset_negative_and_nan_are_zero() {
        assert_eq!(clamp_offset(-5.0, 200.0, 100, 22.0), 0.0);
        assert_eq!(clamp_offset(f32::NAN, 200.0, 100, 22.0), 0.0);
    }

    #[test]
    fn test_spacer_heights_cover_the_hidden_rows() {
        assert_eq!(spacer_heights(&(8..22), 1000, 22.0), (176.0, 21_516.0));
    }
}
