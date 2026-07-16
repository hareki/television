/// Fork-specific module: draws the input bar and results list
/// inside a single merged panel separated by a horizontal line.
///
/// This is activated by the `ui.merge_input_and_results = true`
/// config option. Keeping the logic in its own file minimises
/// merge conflicts with upstream.
use crate::{
    channels::entry::Entry,
    config::{layers::MergedConfig, ui::DEFAULT_PROMPT},
    screen::{
        colors::Colorscheme, constants::POINTER_SYMBOL, layout::InputPosition,
        result_item,
    },
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
    config: &MergedConfig,
    colorscheme: &Colorscheme,
    input_state: &Input,
    results_picker_state: &mut ListState,
    results_count: u32,
    total_count: u32,
    matcher_running: bool,
    channel_name: &str,
    entries: &[Entry],
    selected_entries: &FxHashSet<Entry>,
) -> Result<()> {
    let position = config.input_bar_position;
    let input_padding = &config.input_bar_padding;
    let results_padding = &config.results_panel_padding;
    let prompt = config.input_bar_prompt.as_ref();

    // ── outer block ─────────────────────────────────────────────
    // an empty header means "no header at all"
    let header = config.input_bar_header.as_deref().unwrap_or(channel_name);

    let mut outer_block = Block::default().style(
        Style::default()
            .bg(colorscheme.general.background.unwrap_or_default()),
    );
    if !header.is_empty() {
        outer_block = outer_block
            .title_position(match position {
                InputPosition::Top => TitlePosition::Top,
                InputPosition::Bottom => TitlePosition::Bottom,
            })
            .title(
                Line::from(format!(" {} ", header))
                    .style(
                        Style::default().fg(colorscheme.mode.channel).bold(),
                    )
                    .centered(),
            );
    }
    if let Some(b) = config.input_bar_border_type.to_ratatui_border_type() {
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

    // ── split inner area: input row (1 line + padding),
    // separator (1), rest for results ──
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
    let input_inner =
        if *input_padding == crate::config::ui::Padding::default() {
            input_rect
        } else {
            let pad_block =
                Block::default().padding(RatatuiPadding::from(*input_padding));
            let r = pad_block.inner(input_rect);
            f.render_widget(pad_block, input_rect);
            r
        };

    if input_inner.area() == 0 {
        return Ok(());
    }

    // an empty prompt means "no prompt at all"
    let prompt_len = prompt
        .map(|p| {
            if p.is_empty() {
                0
            } else {
                u16::try_from(p.chars().count() + 1)
                    .expect("Prompt length should fit in u16")
            }
        })
        .unwrap_or(2);
    let indicator_len: u16 = if matcher_running { 2 } else { 0 };
    let count_line = Line::from(Span::styled(
        format!(" {}/{} ", results_count, total_count),
        Style::default().fg(colorscheme.input.results_count_fg),
    ));

    let input_chunks = RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(prompt_len),
            Constraint::Fill(1),
            Constraint::Length(
                u16::try_from(count_line.width())
                    .expect("Count line width should fit in u16"),
            ),
            Constraint::Length(indicator_len),
        ])
        .split(input_inner);

    // prompt
    if prompt_len > 0 {
        f.render_widget(
            Paragraph::new(Span::styled(
                format!("{} ", prompt.unwrap_or(&DEFAULT_PROMPT.to_string())),
                Style::default().fg(colorscheme.input.input_fg),
            )),
            input_chunks[0],
        );
    }

    // text input
    let width = input_chunks[1].width.max(3) - 3;
    let scroll = input_state.visual_scroll(width as usize);
    f.render_widget(
        Paragraph::new(input_state.value())
            .scroll((0, u16::try_from(scroll)?))
            .style(Style::default().fg(colorscheme.input.text_fg))
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
        Paragraph::new(count_line).alignment(Alignment::Right),
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
    let list_direction = match position {
        InputPosition::Bottom => ratatui::widgets::ListDirection::BottomToTop,
        InputPosition::Top => ratatui::widgets::ListDirection::TopToBottom,
    };

    let has_multi_select = !selected_entries.is_empty();

    // Borders are handled by the outer merged block; this inner block
    // only applies the results padding.
    let results_block = Block::default()
        .style(
            Style::default()
                .bg(colorscheme.general.background.unwrap_or_default()),
        )
        .padding(RatatuiPadding::from(*results_padding));

    let results_list = result_item::build_results_list(
        results_block,
        entries,
        results_picker_state,
        list_direction,
        &colorscheme.results,
        results_rect.width.saturating_sub(1), // right padding
        // inside the merged panel the pointer is always shown, even though
        // the inner results block itself is borderless
        POINTER_SYMBOL,
        |entry| {
            if has_multi_select {
                Some(selected_entries.contains(entry))
            } else {
                None
            }
        },
    );

    f.render_stateful_widget(results_list, results_rect, results_picker_state);

    Ok(())
}
