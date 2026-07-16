/// Fork-specific layout helpers.
///
/// Extracted into its own file to minimise merge conflicts with
/// upstream changes to `layout.rs`.
use super::layout::InputPosition;
use ratatui::layout::{Constraint, Rect};

/// Build portrait-mode constraints when `merge_input_and_results`
/// is enabled.  Because the merged drawing function carves out
/// the input bar internally, we do **not** allocate a separate
/// fixed-height chunk for it.  This way a 50 % preview size
/// gives two equally-sized panels.
///
/// The `Percentage(100)/Percentage(0)` placeholders follow the
/// convention of `Layout::build`, which overwrites them with the
/// concrete percentages (or `Fill(1)` when the preview is hidden).
///
/// Returns `(constraints, input_idx, results_idx, preview_idx)`.
/// `input_idx` is set equal to `results_idx` (it is unused by the
/// caller when merging).
pub fn portrait_merged_constraints(
    input_position: InputPosition,
    preview_hidden: bool,
) -> (Vec<Constraint>, usize, usize, Option<usize>) {
    let mut constraints: Vec<Constraint> = Vec::new();
    let results_idx: usize;
    let preview_idx: Option<usize>;

    match input_position {
        InputPosition::Top => {
            if preview_hidden {
                constraints.push(Constraint::Fill(1));
                results_idx = 0;
                preview_idx = None;
            } else {
                // merged (input+results) then preview
                constraints.push(Constraint::Percentage(100));
                constraints.push(Constraint::Percentage(0));
                results_idx = 0;
                preview_idx = Some(1);
            }
        }
        InputPosition::Bottom => {
            if preview_hidden {
                constraints.push(Constraint::Fill(1));
                results_idx = 0;
                preview_idx = None;
            } else {
                // preview then merged (results+input)
                constraints.push(Constraint::Percentage(0));
                constraints.push(Constraint::Percentage(100));
                preview_idx = Some(0);
                results_idx = 1;
            }
        }
    }

    // input_idx is unused when merged; point at results
    let input_idx = results_idx;
    (constraints, input_idx, results_idx, preview_idx)
}

/// Nvim-style popup rect: `ui_scale` percent of `area` is the *inner*
/// size, clamped up to `min_width`/`min_height`; the border sits outside
/// it (footprint = inner + 2 per axis) with the outer top-left at
/// `floor((area - inner) / 2) - 1` per axis, matching the nvim popup math.
/// `ui_scale == 100` with zero minimums returns `area` unchanged.
pub fn bordered_centered_rect(
    ui_scale: u16,
    min_width: u16,
    min_height: u16,
    area: Rect,
) -> Rect {
    let inner_w = (area.width.saturating_mul(ui_scale) / 100).max(min_width);
    let inner_h = (area.height.saturating_mul(ui_scale) / 100).max(min_height);
    let x =
        area.x + (area.width.saturating_sub(inner_w) / 2).saturating_sub(1);
    let y =
        area.y + (area.height.saturating_sub(inner_h) / 2).saturating_sub(1);
    // saturating_add: config-file values bypass the CLI's 10..=100 range
    // check; intersection clips minimums larger than the area itself
    Rect::new(x, y, inner_w.saturating_add(2), inner_h.saturating_add(2))
        .intersection(area)
}

/// Combine the input and results rects into a single bounding
/// rect.  The merged drawing function handles internal
/// sub-splitting.
pub fn merge_input_results_rects(input: Rect, results: Rect) -> (Rect, Rect) {
    let min_x = input.x.min(results.x);
    let min_y = input.y.min(results.y);
    let max_x = (input.x + input.width).max(results.x + results.width);
    let max_y = (input.y + input.height).max(results.y + results.height);
    let merged = Rect::new(
        min_x,
        min_y,
        max_x.saturating_sub(min_x),
        max_y.saturating_sub(min_y),
    );
    (Rect::default(), merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_scale_no_minimums_returns_area_unchanged() {
        let area = Rect::new(0, 0, 120, 30);
        assert_eq!(bordered_centered_rect(100, 0, 0, area), area);
    }

    #[test]
    fn matches_nvim_popup_math() {
        // inner = floor(120 * 0.8) x floor(30 * 0.8) = 96x24,
        // outer top-left = floor(gap / 2) - 1 per axis
        let area = Rect::new(0, 0, 120, 30);
        assert_eq!(
            bordered_centered_rect(80, 0, 0, area),
            Rect::new(11, 2, 98, 26)
        );
    }

    #[test]
    fn odd_leftover_floors_like_nvim() {
        // inner_w = floor(121 * 0.8) = 96, x = floor(25 / 2) - 1 = 11
        let area = Rect::new(0, 0, 121, 30);
        assert_eq!(
            bordered_centered_rect(80, 0, 0, area),
            Rect::new(11, 2, 98, 26)
        );
    }

    #[test]
    fn minimums_clamp_inner_size_up() {
        // inner would be 50x10, clamped up to 90x15
        let area = Rect::new(0, 0, 100, 20);
        assert_eq!(
            bordered_centered_rect(50, 90, 15, area),
            Rect::new(4, 1, 92, 17)
        );
    }

    #[test]
    fn minimums_exceeding_area_are_clipped_to_it() {
        let area = Rect::new(0, 0, 80, 10);
        assert_eq!(bordered_centered_rect(80, 90, 15, area), area);
    }

    #[test]
    fn offset_area_origin_is_respected() {
        // inner = 80x16 within a viewport starting at y = 5
        let area = Rect::new(0, 5, 100, 20);
        assert_eq!(
            bordered_centered_rect(80, 0, 0, area),
            Rect::new(9, 6, 82, 18)
        );
    }
}
