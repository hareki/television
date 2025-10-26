use crate::{
    channels::entry::Entry,
    config::ui::{BorderType, Padding},
    screen::{colors::Colorscheme, layout::InputPosition, result_item},
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    prelude::Style,
    text::{Line, Span},
    widgets::{
        Block, Borders, ListState, Padding as RatatuiPadding, Paragraph,
    },
};
use rustc_hash::FxHashSet;

#[allow(clippy::too_many_arguments)]
pub fn draw_results_list(
    f: &mut Frame,
    rect: Rect,
    entries: &[Entry],
    selected_entries: &FxHashSet<Entry>,
    relative_picker_state: &mut ListState,
    input_bar_position: InputPosition,
    colorscheme: &Colorscheme,
    results_panel_padding: &Padding,
    results_panel_border_type: &BorderType,
    header: &Option<String>,
    merge_with_input: bool,
) -> Result<()> {
    let mut results_block = Block::default()
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(RatatuiPadding::from(*results_panel_padding));

    // When merging with input, the header logic changes:
    // - If results_panel_header is set (Some(non-empty)), show it on the shared border
    // - If results_panel_header is None or empty, no header on the shared border
    // When not merging:
    // - If Some("") => no header
    // - If Some(non-empty) => use it
    // - If None => use default " Results "
    if merge_with_input {
        // When merging, only show header if explicitly set and non-empty
        if let Some(h) = header {
            if !h.is_empty() {
                let title_position = match input_bar_position {
                    InputPosition::Top => {
                        ratatui::widgets::block::Position::Top
                    }
                    InputPosition::Bottom => {
                        ratatui::widgets::block::Position::Bottom
                    }
                };
                results_block =
                    results_block.title_position(title_position).title(
                        Line::from(format!(" {} ", h))
                            .alignment(Alignment::Center),
                    );
            }
        }
    } else {
        // Original behavior when not merging
        if let Some(h) = header {
            if !h.is_empty() {
                results_block = results_block.title_top(
                    Line::from(format!(" {} ", h))
                        .alignment(Alignment::Center),
                );
            }
        } else {
            results_block = results_block.title_top(
                Line::from(" Results ").alignment(Alignment::Center),
            );
        }
    }

    if let Some(border_type) =
        results_panel_border_type.to_ratatui_border_type()
    {
        // When merging with input:
        // - If input is at top: results has all borders (the top border is the shared one)
        // - If input is at bottom: results has all borders (the bottom border is the shared one)
        // The input will exclude its adjacent border
        results_block = results_block
            .borders(Borders::ALL)
            .border_type(border_type)
            .border_style(Style::default().fg(colorscheme.general.border_fg));
    }

    let list_direction = match input_bar_position {
        InputPosition::Bottom => ratatui::widgets::ListDirection::BottomToTop,
        InputPosition::Top => ratatui::widgets::ListDirection::TopToBottom,
    };

    let has_multi_select = !selected_entries.is_empty();

    let results_list = result_item::build_results_list(
        results_block,
        entries,
        relative_picker_state,
        list_direction,
        &colorscheme.results,
        rect.width - 1, // right padding
        |entry| {
            if has_multi_select {
                Some(selected_entries.contains(entry))
            } else {
                None
            }
        },
    );

    f.render_stateful_widget(results_list, rect, relative_picker_state);

    // Draw the shared border line with proper junction characters when merging
    if merge_with_input {
        if let Some(border_type_enum) =
            results_panel_border_type.to_ratatui_border_type()
        {
            draw_shared_border(
                f,
                rect,
                input_bar_position,
                border_type_enum,
                header,
                colorscheme,
            );
        }
    }

    Ok(())
}

/// Draw a shared border line between input and results panels with proper T-junction characters
fn draw_shared_border(
    f: &mut Frame,
    rect: Rect,
    input_bar_position: InputPosition,
    border_type: ratatui::widgets::BorderType,
    header: &Option<String>,
    colorscheme: &Colorscheme,
) {
    use ratatui::symbols::border;

    let border_set = match border_type {
        ratatui::widgets::BorderType::Plain => border::PLAIN,
        ratatui::widgets::BorderType::Rounded => border::ROUNDED,
        ratatui::widgets::BorderType::Double => border::DOUBLE,
        ratatui::widgets::BorderType::Thick => border::THICK,
        _ => return, // For other types, don't draw custom border
    };

    // Determine which edge to draw the border on
    let (y, left_char, right_char, line_char) = match input_bar_position {
        InputPosition::Top => {
            // Border is at the top of results panel (bottom of input)
            // Use T-junctions: ├ horizontal ┤
            (
                rect.y,
                border_set.vertical_left,
                border_set.vertical_right,
                border_set.horizontal_top,
            )
        }
        InputPosition::Bottom => {
            // Border is at the bottom of results panel (top of input)
            // Use T-junctions: ├ horizontal ┤
            (
                rect.y + rect.height - 1,
                border_set.vertical_left,
                border_set.vertical_right,
                border_set.horizontal_bottom,
            )
        }
    };

    if rect.width < 2 {
        return;
    }

    let border_style = Style::default().fg(colorscheme.general.border_fg);

    // Build the border line
    let mut border_line = String::new();
    border_line.push_str(left_char);

    // Add the horizontal line with optional header
    if let Some(h) = header {
        if !h.is_empty() {
            let header_text = format!(" {} ", h);
            let header_len = header_text.chars().count();
            let available_width = (rect.width as usize).saturating_sub(2); // subtract left and right junction chars

            if header_len <= available_width {
                let left_padding = (available_width - header_len) / 2;
                let right_padding =
                    available_width - header_len - left_padding;

                border_line.push_str(&line_char.repeat(left_padding));
                border_line.push_str(&header_text);
                border_line.push_str(&line_char.repeat(right_padding));
            } else {
                // Header too long, just draw the line
                border_line.push_str(&line_char.repeat(available_width));
            }
        } else {
            // Empty header, just draw the line
            border_line.push_str(
                &line_char.repeat((rect.width as usize).saturating_sub(2)),
            );
        }
    } else {
        // No header, just draw the line
        border_line.push_str(
            &line_char.repeat((rect.width as usize).saturating_sub(2)),
        );
    }

    border_line.push_str(right_char);

    // Render the border line
    let border_rect = Rect {
        x: rect.x,
        y,
        width: rect.width,
        height: 1,
    };

    let border_paragraph =
        Paragraph::new(Line::from(Span::styled(border_line, border_style)));
    f.render_widget(border_paragraph, border_rect);
}
