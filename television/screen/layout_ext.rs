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
                // merged (results+input) then preview
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
