/// Fork-specific module: draws the input bar and results list
/// inside a single merged panel separated by a horizontal line.
///
/// This is activated by the `ui.merge_input_and_results = true`
/// config option. Keeping the logic in its own file minimises
/// merge conflicts with upstream.
use crate::{
    channels::entry::Entry,
    config::ui::{BorderType, DEFAULT_PROMPT, Padding},
    event::Key,
    screen::{colors::Colorscheme, layout::InputPosition, result_item},
    utils::input::Input,
};
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{
        Alignment, Constraint, Direction, Layout as RatatuiLayout, Rect,
    },
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, ListState, Padding as RatatuiPadding, Paragraph,
        TitlePosition,
    },
};
use rustc_hash::FxHashSet;

const LOADING_CHAR: &str = "●";

#[allow(clippy::too_many_arguments)]
pub fn draw_merged_input_results(
    f: &mut Frame,
    rect: Rect,
    // input state
    results_count: u32,
    total_count: u32,
    input_state: &Input,
    results_picker_state: &mut ListState,
    matcher_running: bool,
    channel_name: &str,
    // results state
    entries: &[Entry],
    selected_entries: &FxHashSet<Entry>,
    _source_index: usize,
    _source_count: usize,
    _cycle_key: Option<Key>,
    // config
    colorscheme: &Colorscheme,
    position: InputPosition,
    input_header: &Option<String>,
    input_padding: &Padding,
    input_border_type: &BorderType,
    input_prompt: Option<&String>,
    results_padding: &Padding,
) -> Result<()> {
    // ── outer block ─────────────────────────────────────────────
    let header_text =
        input_header.as_ref().map_or(channel_name, |v| v.as_str());

    let title_position = match position {
        InputPosition::Top => TitlePosition::Top,
        InputPosition::Bottom => TitlePosition::Bottom,
    };

    let mut outer_block = Block::default()
        .title_position(title_position)
        .title(
            Line::from(format!(" {} ", header_text))
                .style(Style::default().fg(colorscheme.mode.channel).bold())
                .centered(),
        )
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        );

    if let Some(b) = input_border_type.to_ratatui_border_type() {
        outer_block = outer_block
            .borders(Borders::ALL)
            .border_type(b)
            .border_style(Style::default().fg(colorscheme.general.border_fg));
    }

    let inner = outer_block.inner(rect);
    if inner.area() == 0 {
        return Ok(());
    }
    f.render_widget(outer_block, rect);

    // ── split inner area: input row (1 line), separator (1),
    // rest for results ──
    let input_row_height: u16 = 1 + input_padding.top + input_padding.bottom;
    let separator_height: u16 = 1;

    let (input_rect, separator_rect, results_rect) = match position {
        InputPosition::Top => {
            let chunks = RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(input_row_height),
                    Constraint::Length(separator_height),
                    Constraint::Min(1),
                ])
                .split(inner);
            (chunks[0], chunks[1], chunks[2])
        }
        InputPosition::Bottom => {
            let chunks = RatatuiLayout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(separator_height),
                    Constraint::Length(input_row_height),
                ])
                .split(inner);
            (chunks[2], chunks[1], chunks[0])
        }
    };

    // ── draw separator ──────────────────────────────────────────
    let sep_line = "─".repeat(separator_rect.width as usize);
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            sep_line,
            Style::default().fg(colorscheme.general.border_fg),
        ))),
        separator_rect,
    );

    // ── draw input row ──────────────────────────────────────────
    let input_inner = if input_padding.top > 0
        || input_padding.bottom > 0
        || input_padding.left > 0
        || input_padding.right > 0
    {
        let pad_block =
            Block::default().padding(RatatuiPadding::from(*input_padding));
        let r = pad_block.inner(input_rect);
        f.render_widget(pad_block, input_rect);
        r
    } else {
        input_rect
    };

    if input_inner.area() == 0 {
        return Ok(());
    }

    let prompt_str = input_prompt.map_or(DEFAULT_PROMPT, |p| p.as_str());
    let indicator_len: u16 = if matcher_running { 2 } else { 0 };
    let prompt_len =
        u16::try_from(prompt_str.chars().count() + 1).unwrap_or(2);
    let count_digits = u16::try_from(total_count.max(1).ilog10()).unwrap() + 1;
    let count_len = 3 * count_digits + 3;

    let input_chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(prompt_len),
            Constraint::Fill(1),
            Constraint::Length(count_len),
            Constraint::Length(indicator_len),
        ])
        .split(input_inner);

    // prompt
    f.render_widget(
        Paragraph::new(Span::styled(
            format!("{} ", prompt_str),
            Style::default().fg(colorscheme.input.input_fg).bold(),
        )),
        input_chunks[0],
    );

    // text input
    let width = input_chunks[1].width.max(3) - 3;
    let scroll = input_state.visual_scroll(width as usize);
    f.render_widget(
        Paragraph::new(input_state.value())
            .scroll((0, u16::try_from(scroll)?))
            .style(
                Style::default()
                    .fg(colorscheme.input.input_fg)
                    .bold()
                    .italic(),
            )
            .alignment(Alignment::Left),
        input_chunks[1],
    );

    // loading indicator
    if matcher_running {
        f.render_widget(
            Span::styled(LOADING_CHAR, Style::default().fg(Color::Green)),
            input_chunks[3],
        );
    }

    // result count
    f.render_widget(
        Paragraph::new(Span::styled(
            format!(" {}/{} ", results_count, total_count),
            Style::default()
                .fg(colorscheme.input.results_count_fg)
                .italic(),
        ))
        .alignment(Alignment::Right),
        input_chunks[2],
    );

    // cursor
    f.set_cursor_position((
        input_chunks[1].x.saturating_add(u16::try_from(
            input_state.visual_cursor().max(scroll) - scroll,
        )?),
        input_chunks[1].y,
    ));

    // ── draw results list ───────────────────────────────────────
    let results_inner = if results_padding.top > 0
        || results_padding.bottom > 0
        || results_padding.left > 0
        || results_padding.right > 0
    {
        let pad_block =
            Block::default().padding(RatatuiPadding::from(*results_padding));
        let r = pad_block.inner(results_rect);
        f.render_widget(pad_block, results_rect);
        r
    } else {
        results_rect
    };

    let list_direction = match position {
        InputPosition::Bottom => ratatui::widgets::ListDirection::BottomToTop,
        InputPosition::Top => ratatui::widgets::ListDirection::TopToBottom,
    };

    let has_multi_select = !selected_entries.is_empty();

    // Build the list with no outer block (borders are handled by
    // the outer merged block).
    let results_block = Block::default().style(
        Style::default()
            .bg(colorscheme.general.background.unwrap_or_default()),
    );

    let results_list = result_item::build_results_list(
        results_block,
        entries,
        results_picker_state,
        list_direction,
        &colorscheme.results,
        results_inner.width.saturating_sub(1),
        |entry| {
            if has_multi_select {
                Some(selected_entries.contains(entry))
            } else {
                None
            }
        },
    );

    f.render_stateful_widget(
        results_list,
        results_inner,
        results_picker_state,
    );

    Ok(())
}
