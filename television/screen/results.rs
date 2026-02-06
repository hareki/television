use crate::{
    channels::entry::Entry,
    config::ui::{BorderType, Padding},
    event::Key,
    screen::{colors::Colorscheme, layout::InputPosition, result_item},
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    prelude::{Span, Style},
    text::Line,
    widgets::{Block, Borders, ListState, Padding as RatatuiPadding},
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
    results_panel_header: &Option<String>,
    source_index: usize,
    source_count: usize,
    cycle_key: Option<Key>,
) -> Result<()> {
    // None = use default, Some("") = hide, Some(text) = custom
    let should_show_title = results_panel_header
        .as_ref()
        .map_or(true, |h| !h.is_empty());

    let mut results_block = Block::default()
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(RatatuiPadding::from(*results_panel_padding));

    if should_show_title {
        let header_text = results_panel_header
            .as_ref()
            .map_or("Results", |h| h.as_str());

        let title = if source_count > 1 {
            let mut spans = vec![Span::from(format!(" {} ", header_text))];
            let dots: String = (0..source_count)
                .map(|i| if i == source_index { "●" } else { "○" })
                .collect::<Vec<_>>()
                .join(" ");
            spans.push(Span::styled(
                format!("⟨ {} ⟩", dots),
                Style::default().fg(colorscheme.input.results_count_fg),
            ));
            if let Some(key) = cycle_key {
                spans.push(Span::styled(
                    format!(" {}", key),
                    Style::default().fg(colorscheme.general.border_fg),
                ));
            }
            spans.push(Span::from(" "));
            Line::from(spans).alignment(Alignment::Center)
        } else {
            Line::from(format!(" {} ", header_text))
                .alignment(Alignment::Center)
        };

        results_block = results_block.title_top(title);
    }

    if let Some(border_type) =
        results_panel_border_type.to_ratatui_border_type()
    {
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
    Ok(())
}
