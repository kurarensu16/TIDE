//! UI Rendering for TIDE using Ratatui.
//! 
//! Recreates the sleek, high-density terminal workspace layout from TIDE.png:
//! - Top Header: branding, workspace context, global shortcuts
//! - Left Pane: Workspace File Tree Explorer
//! - Center Pane: Tabbed PTY Terminal & VT100 Emulator Grid
//! - Right Top Pane: Git status, branch, and commit log
//! - Right Bottom Pane: Active Agent Sessions with confidence tracking
//! - Bottom Status Bar: Path, active shell, cursor position, agent counts

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

use crate::app::{ActivePane, App};
use crate::agents::{AgentState, Confidence};

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let theme = app.theme();

    // Render full canvas background so themes transform the entire workspace background
    f.render_widget(Block::default().style(Style::default().bg(theme.bg)), size);

    // Vertical layout: Header (1), Main Body (Min 0), Status Bar (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    draw_header(f, app, chunks[0]);
    if app.show_welcome_screen {
        draw_welcome_screen(f, app, chunks[1]);
    } else {
        draw_body(f, app, chunks[1]);
    }
    draw_status_bar(f, app, chunks[2]);

    // Modal overlays (if active)
    if app.welcome_prompt_modal != crate::app::WelcomePromptModal::None {
        draw_welcome_prompt_modal(f, app, size);
    } else if app.show_help {
        draw_help_modal(f, app, size);
    } else if app.show_command_palette {
        draw_command_palette_modal(f, app, size);
    } else if app.show_terminal_menu {
        draw_terminal_menu_modal(f, app, size);
    } else if app.show_agent_launch_modal {
        draw_agent_launch_modal(f, app, size);
    } else if app.show_theme_modal {
        draw_theme_modal(f, app, size);
    } else if app.show_commit_modal {
        draw_commit_modal(f, app, size);
    } else if app.show_diff_modal {
        draw_diff_modal(f, app, size);
    } else if app.show_branch_modal {
        draw_branch_modal(f, app, size);
    } else if app.show_stash_modal {
        draw_stash_modal(f, app, size);
    } else if app.show_global_search {
        draw_global_search_modal(f, app, size);
    } else if app.explorer_op_modal != crate::app::ExplorerOpModal::None {
        draw_explorer_op_modal(f, app, size);
    }
}

fn draw_header(f: &mut Frame, app: &mut App, area: Rect) {
    app.header_area = area;
    let w = area.width as usize;
    let theme = app.theme();

    let toggle_btn = |label: &str, active: bool| {
        if active {
            Span::styled(format!("[{}]", label), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("[{}]", label), Style::default().fg(theme.text_muted))
        }
    };

    let (mid_spans, mid_len) = if w >= 100 {
        (
            vec![
                toggle_btn("F2 Exp", app.show_explorer),
                Span::raw(" "),
                toggle_btn("F5 Prev", app.show_preview),
                Span::raw(" "),
                toggle_btn("F3 Git", app.show_git),
                Span::raw(" "),
                toggle_btn("F4 Agt", app.show_agents),
                Span::raw(" "),
                toggle_btn("F11 Max", app.is_maximized),
            ],
            46,
        )
    } else if w >= 80 {
        (
            vec![
                toggle_btn("F2", app.show_explorer),
                Span::raw(" "),
                toggle_btn("F5", app.show_preview),
                Span::raw(" "),
                toggle_btn("F3", app.show_git),
                Span::raw(" "),
                toggle_btn("F4", app.show_agents),
                Span::raw(" "),
                toggle_btn("F11", app.is_maximized),
            ],
            25,
        )
    } else if w >= 60 {
        (
            vec![
                toggle_btn("F2", app.show_explorer),
                Span::raw(" "),
                toggle_btn("F3", app.show_git),
                Span::raw(" "),
                toggle_btn("F4", app.show_agents),
                Span::raw(" "),
                toggle_btn("F11", app.is_maximized),
            ],
            20,
        )
    } else {
        (Vec::new(), 0)
    };

    let (right_spans, right_len) = if w >= 125 {
        (
            vec![
                Span::styled("F7/8", Style::default().fg(theme.accent)),
                Span::raw(" Tabs │ "),
                Span::styled("F6", Style::default().fg(theme.warning)),
                Span::raw(" Pane │ "),
                Span::styled("F12", Style::default().fg(theme.accent)),
                Span::raw(" Theme │ "),
                Span::styled("F1", Style::default().fg(theme.accent)),
                Span::raw(" Help │ "),
                Span::styled("<C-q>", Style::default().fg(theme.danger)),
                Span::raw(" Quit"),
            ],
            54,
        )
    } else if w >= 100 {
        (
            vec![
                Span::styled("F6", Style::default().fg(theme.warning)),
                Span::raw(" Pane │ "),
                Span::styled("F12", Style::default().fg(theme.accent)),
                Span::raw(" Theme │ "),
                Span::styled("F1", Style::default().fg(theme.accent)),
                Span::raw(" Help │ "),
                Span::styled("<C-q>", Style::default().fg(theme.danger)),
                Span::raw(" Quit"),
            ],
            42,
        )
    } else if w >= 80 {
        (
            vec![
                Span::styled("F12", Style::default().fg(theme.accent)),
                Span::raw(" Theme │ "),
                Span::styled("F1", Style::default().fg(theme.accent)),
                Span::raw(" Help │ "),
                Span::styled("<C-q>", Style::default().fg(theme.danger)),
                Span::raw(" Quit"),
            ],
            32,
        )
    } else if w >= 60 {
        (
            vec![
                Span::styled("F12", Style::default().fg(theme.accent)),
                Span::raw(" Theme │ "),
                Span::styled("<C-q>", Style::default().fg(theme.danger)),
                Span::raw(" Quit"),
            ],
            22,
        )
    } else {
        (
            vec![
                Span::styled("F12", Style::default().fg(theme.accent)),
                Span::raw(" │ "),
                Span::styled("<C-q>", Style::default().fg(theme.danger)),
            ],
            12,
        )
    };

    let right_width = ((w.saturating_sub(mid_len)) / 2).max(right_len);
    let left_width = w.saturating_sub(mid_len).saturating_sub(right_width);

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_width as u16),
            Constraint::Length(mid_len as u16),
            Constraint::Length(right_width as u16),
        ])
        .split(area);

    app.header_left_area = header_chunks[0];
    app.header_mid_area = header_chunks[1];
    app.header_right_area = header_chunks[2];

    let wave_icon = crate::icons::pane_icon("wave", app.icon_theme);
    let exp_icon = crate::icons::pane_icon("explorer", app.icon_theme);
    let git_icon = crate::icons::pane_icon("git", app.icon_theme);
    let bell_icon = crate::icons::pane_icon("bell", app.icon_theme);

    let mut left_spans = vec![
        Span::styled(format!(" {} ", wave_icon), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("TIDE", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
    ];

    if left_width >= 35 && w >= 110 {
        left_spans.extend(vec![
            Span::raw(" "),
            Span::styled(exp_icon, Style::default().fg(theme.accent)),
            Span::styled(&app.workspace_name, Style::default().fg(theme.text_primary)),
            Span::raw(" "),
            Span::styled(git_icon, Style::default().fg(theme.accent_secondary)),
            Span::styled(&app.git_info.branch, Style::default().fg(theme.accent_secondary)),
        ]);
    } else if left_width >= 20 {
        left_spans.extend(vec![
            Span::raw(" │ "),
            Span::styled(git_icon, Style::default().fg(theme.accent_secondary)),
            Span::styled(&app.git_info.branch, Style::default().fg(theme.accent_secondary)),
        ]);
    }

    if let Some((msg, time)) = &app.status_message {
        if time.elapsed().as_secs() < 12 && left_width >= 55 {
            left_spans.push(Span::raw(" │ "));
            left_spans.push(Span::styled(format!("{} {}", bell_icon, msg), Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)));
        }
    }

    let header_bg = Style::default().bg(theme.bg);
    f.render_widget(Block::default().style(header_bg), area);
    f.render_widget(Paragraph::new(Line::from(left_spans)).style(header_bg), header_chunks[0]);
    if !mid_spans.is_empty() {
        f.render_widget(Paragraph::new(Line::from(mid_spans)).style(header_bg).alignment(Alignment::Center), header_chunks[1]);
    }
    f.render_widget(Paragraph::new(Line::from(right_spans)).style(header_bg).alignment(Alignment::Right), header_chunks[2]);
}

fn draw_body(f: &mut Frame, app: &mut App, area: Rect) {
    app.body_area = area;

    if app.is_maximized {
        app.explorer_area = Rect::default();
        app.explorer_footer_area = Rect::default();
        app.preview_area = Rect::default();
        app.terminal_area = Rect::default();
        app.git_area = Rect::default();
        app.agents_area = Rect::default();

        match app.active_pane {
            ActivePane::Explorer => {
                app.explorer_area = area;
                draw_explorer_pane(f, app, area);
            }
            ActivePane::FilePreview => {
                app.preview_area = area;
                draw_file_preview_pane(f, app, area);
            }
            ActivePane::Terminal => {
                app.terminal_area = area;
                draw_terminal_pane(f, app, area);
            }
            ActivePane::Git => {
                app.git_area = area;
                draw_git_pane(f, app, area);
            }
            ActivePane::Agents => {
                app.agents_area = area;
                draw_agents_pane(f, app, area);
            }
        }
        return;
    }

    let has_right = app.show_git || app.show_agents;
    let mut col_constraints = Vec::new();

    if app.show_explorer {
        col_constraints.push(Constraint::Percentage(app.left_col_pct.clamp(10, 50)));
    }
    col_constraints.push(Constraint::Min(25)); // Center column takes remaining
    if has_right {
        col_constraints.push(Constraint::Percentage(app.right_col_pct.clamp(12, 50)));
    }

    let col_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(col_constraints)
        .split(area);

    let mut chunk_idx = 0;
    if app.show_explorer {
        app.explorer_area = col_chunks[chunk_idx];
        draw_explorer_pane(f, app, col_chunks[chunk_idx]);
        chunk_idx += 1;
    } else {
        app.explorer_area = Rect::default();
        app.explorer_footer_area = Rect::default();
    }

    let center_area = col_chunks[chunk_idx];
    chunk_idx += 1;

    // Center column: preview + terminal
    if app.show_preview {
        let center_pct = app.center_split_pct.clamp(15, 85);
        let center_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(center_pct),
                Constraint::Percentage(100 - center_pct),
            ])
            .split(center_area);

        app.preview_area = center_chunks[0];
        draw_file_preview_pane(f, app, center_chunks[0]);

        app.terminal_area = center_chunks[1];
        draw_terminal_pane(f, app, center_chunks[1]);
    } else {
        app.preview_area = Rect::default();
        app.terminal_area = center_area;
        draw_terminal_pane(f, app, center_area);
    }

    // Right column: Git and/or Agents
    if has_right {
        let right_area = col_chunks[chunk_idx];
        if app.show_git && app.show_agents {
            let right_pct = app.right_split_pct.clamp(15, 85);
            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(right_pct),
                    Constraint::Percentage(100 - right_pct),
                ])
                .split(right_area);

            app.git_area = right_chunks[0];
            draw_git_pane(f, app, right_chunks[0]);

            app.agents_area = right_chunks[1];
            draw_agents_pane(f, app, right_chunks[1]);
        } else if app.show_git {
            app.git_area = right_area;
            app.agents_area = Rect::default();
            draw_git_pane(f, app, right_area);
        } else {
            app.git_area = Rect::default();
            app.agents_area = right_area;
            draw_agents_pane(f, app, right_area);
        }
    } else {
        app.git_area = Rect::default();
        app.agents_area = Rect::default();
    }
}

fn draw_file_preview_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let is_focused = app.active_pane == ActivePane::FilePreview;
    let border_color = if is_focused {
        theme.border_active
    } else {
        theme.border_inactive
    };

    let p_icon = crate::icons::pane_icon("preview", app.icon_theme);
    let block = Block::default()
        .title(format!(" {}Code Editor [×] ", p_icon))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.surface));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.editor_tabs.is_empty() {
        app.preview_tabs_area = Rect::default();
        let mut empty_msg = Vec::new();
        empty_msg.push(Line::raw(""));
        for mut line in crate::logo::wave_art_for_height(inner.height) {
            line.alignment = Some(Alignment::Center);
            empty_msg.push(line);
        }
        empty_msg.push(Line::raw(""));
        empty_msg.push(Line::styled(
            "TIDE — Terminal Integrated Development Environment",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ).alignment(Alignment::Center));
        empty_msg.push(Line::styled(
            "Select a file in Explorer (j/k or click) or press Ctrl+P to search files.",
            Style::default().fg(theme.text_secondary),
        ).alignment(Alignment::Center));
        f.render_widget(Paragraph::new(empty_msg).style(Style::default().bg(theme.surface)), inner);
        return;
    }

    let constraints = if app.show_find_in_file {
        vec![
            Constraint::Length(1), // Tab strip
            Constraint::Length(1), // Find bar
            Constraint::Min(0),    // Code preview/editor body
            Constraint::Length(1), // Bottom status bar
        ]
    } else {
        vec![
            Constraint::Length(1), // Tab strip
            Constraint::Min(0),    // Code preview/editor body
            Constraint::Length(1), // Bottom status bar
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    app.preview_tabs_area = chunks[0];

    // Horizontal split for tab strip: tabs on left, tab count & F9/F10 hints on right
    let tab_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(26)])
        .split(chunks[0]);

    let total_tabs = app.editor_tabs.len();
    let available_w = tab_bar_chunks[0].width as usize;

    let tab_widths: Vec<usize> = app.editor_tabs.iter().map(|tab| {
        let (f_icon, _) = crate::icons::file_icon(&tab.file_name, app.icon_theme);
        let dirty_len = if tab.dirty { 2 } else { 0 };
        1 + f_icon.chars().count() + 1 + tab.file_name.chars().count() + dirty_len + 5 + 1
    }).collect();

    // Auto-scroll so active_editor_tab_idx is visible
    if app.active_editor_tab_idx < app.editor_tab_scroll {
        app.editor_tab_scroll = app.active_editor_tab_idx;
    }
    let mut test_w = 0;
    for i in app.editor_tab_scroll..total_tabs {
        test_w += tab_widths.get(i).copied().unwrap_or(15);
        if i == app.active_editor_tab_idx {
            break;
        }
    }
    while test_w + 6 > available_w && app.editor_tab_scroll < app.active_editor_tab_idx {
        test_w = test_w.saturating_sub(tab_widths.get(app.editor_tab_scroll).copied().unwrap_or(15));
        app.editor_tab_scroll += 1;
    }

    let mut tab_spans = Vec::new();
    if app.editor_tab_scroll > 0 {
        tab_spans.push(Span::styled(" ‹ ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)));
    }

    let mut accumulated_w = if app.editor_tab_scroll > 0 { 3 } else { 0 };
    let mut last_visible_tab = app.editor_tab_scroll;

    for i in app.editor_tab_scroll..total_tabs {
        let tab = &app.editor_tabs[i];
        let tab_w = tab_widths.get(i).copied().unwrap_or(15);

        if i > app.editor_tab_scroll && accumulated_w + tab_w + 6 > available_w {
            break;
        }

        let is_active = i == app.active_editor_tab_idx;
        let is_dirty = if is_active { app.editor_dirty } else { tab.dirty };
        let (f_icon, icon_color) = crate::icons::file_icon(&tab.file_name, app.icon_theme);

        let tab_bg = if is_active {
            theme.tab_active_bg
        } else {
            theme.tab_inactive_bg
        };

        let tab_fg = if is_active {
            theme.tab_active_fg
        } else {
            theme.tab_inactive_fg
        };

        let mut tab_style = Style::default().bg(tab_bg).fg(tab_fg);
        if is_active {
            tab_style = tab_style.add_modifier(Modifier::BOLD);
        }
        if !tab.is_pinned {
            tab_style = tab_style.add_modifier(Modifier::ITALIC);
        }

        tab_spans.push(Span::styled(" ", tab_style));
        tab_spans.push(Span::styled(format!("{} ", f_icon), tab_style.fg(if is_active { Color::White } else { icon_color })));
        tab_spans.push(Span::styled(&tab.file_name, tab_style));

        if is_dirty {
            tab_spans.push(Span::styled(" ●", tab_style.fg(theme.warning)));
        }

        let close_style = if is_active {
            Style::default().bg(tab_bg).fg(theme.danger).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(tab_bg).fg(theme.text_muted)
        };
        tab_spans.push(Span::styled(" [×] ", close_style));
        tab_spans.push(Span::styled("│", Style::default().fg(theme.border_inactive)));

        accumulated_w += tab_w;
        last_visible_tab = i;
    }

    if last_visible_tab + 1 < total_tabs {
        tab_spans.push(Span::styled(" › ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)));
    }

    let right_tab_controls = vec![
        Span::styled(format!("({}/{}) ", app.active_editor_tab_idx + 1, total_tabs.max(1)), Style::default().fg(theme.text_secondary)),
        Span::styled("[F9 ◀] ", Style::default().fg(theme.warning)),
        Span::styled("[▶ F10]", Style::default().fg(theme.warning)),
        Span::raw(" "),
    ];

    f.render_widget(Paragraph::new(Line::from(tab_spans)), tab_bar_chunks[0]);
    f.render_widget(Paragraph::new(Line::from(right_tab_controls)).alignment(Alignment::Right), tab_bar_chunks[1]);

    // Optional Find Bar
    let (code_body_chunk, status_chunk) = if app.show_find_in_file {
        app.find_bar_area = chunks[1];
        let find_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(25), Constraint::Length(35)])
            .split(chunks[1]);

        let f_icon = crate::icons::action_icon("search", app.icon_theme);
        let mut find_left_spans = vec![
            Span::styled(format!(" {}Find: ", f_icon), Style::default().bg(theme.surface).fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(&app.find_query, Style::default().bg(theme.surface).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().bg(theme.surface).fg(theme.accent)),
            Span::styled(" ", Style::default().bg(theme.surface)),
        ];

        if app.find_query.is_empty() {
            find_left_spans.push(Span::styled("(type to find) ", Style::default().bg(theme.surface).fg(theme.text_muted)));
        } else if app.find_matches.is_empty() {
            find_left_spans.push(Span::styled(" [No matches] ", Style::default().bg(theme.danger).fg(Color::White).add_modifier(Modifier::BOLD)));
        } else {
            let badge = format!(" [{}/{}] ", app.active_find_match_idx + 1, app.find_matches.len());
            find_left_spans.push(Span::styled(badge, Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD)));
        }

        let find_right_spans = vec![
            Span::styled("[Next ▾] ", Style::default().bg(theme.surface).fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled("[Prev ▴] ", Style::default().bg(theme.surface).fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled("[Esc: ×] ", Style::default().bg(theme.surface).fg(theme.danger).add_modifier(Modifier::BOLD)),
        ];

        f.render_widget(Paragraph::new(Line::from(find_left_spans)).style(Style::default().bg(theme.surface)), find_chunks[0]);
        f.render_widget(Paragraph::new(Line::from(find_right_spans)).alignment(Alignment::Right).style(Style::default().bg(theme.surface)), find_chunks[1]);

        (chunks[2], chunks[3])
    } else {
        app.find_bar_area = Rect::default();
        (chunks[1], chunks[2])
    };

    // Active file content lines
    let empty_lines = Vec::new();
    let lines = match &app.preview_content {
        Some((_, l)) => l,
        None => &empty_lines,
    };

    let view_height = code_body_chunk.height as usize;
    let scroll = app.preview_scroll;
    let visible_lines = lines.iter().skip(scroll).take(view_height);

    let mut rendered_lines = Vec::new();
    for (idx, line) in visible_lines.enumerate() {
        let line_num = scroll + idx + 1;
        let mut spans = vec![
            Span::styled(format!("{:>4} │ ", line_num), Style::default().fg(theme.text_muted)),
        ];

        spans.extend(render_code_line_with_search(line, line_num - 1, app));
        rendered_lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(rendered_lines), code_body_chunk);

    // Render visual scrollbar on code preview
    let total_lines = lines.len();
    if total_lines > view_height && view_height > 0 {
        let max_scroll = total_lines.saturating_sub(view_height);
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll.min(max_scroll));
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.accent));
        f.render_stateful_widget(scrollbar, code_body_chunk, &mut scrollbar_state);
    }

    // Bottom buffer status line
    let mode_badge = if app.editor_dirty {
        Span::styled(" MODIFIED ", Style::default().bg(theme.warning).fg(Color::Black).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" EDIT ", Style::default().bg(theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD))
    };
    let name_span = Span::styled(format!(" {} ", app.preview_content.as_ref().map(|(n, _)| n.as_str()).unwrap_or("")), Style::default().fg(theme.text_primary));
    
    let status_w = status_chunk.width as usize;
    let (pos_len, desc_text) = if status_w >= 120 {
        (22, "Ctrl+z: Undo │ Ctrl+y: Redo │ Ctrl+d: Dup │ Alt+↑↓: Move │ Ctrl+/: Comment │ Ctrl+s: Save  ")
    } else if status_w >= 95 {
        (20, "Ctrl+z: Undo │ Ctrl+y: Redo │ Ctrl+d: Dup │ Ctrl+/: Comment │ Ctrl+s: Save  ")
    } else if status_w >= 70 {
        (18, "Ctrl+z: Undo │ Ctrl+y: Redo │ Ctrl+d: Dup │ Ctrl+s: Save  ")
    } else if status_w >= 50 {
        (16, "Ctrl+z: Undo │ Ctrl+s: Save  ")
    } else if status_w >= 35 {
        (14, "Ctrl+s: Save  ")
    } else {
        (10, "")
    };

    let desc_span = Span::styled(desc_text, Style::default().fg(theme.text_secondary));
    let pos_text = if pos_len >= 18 {
        format!("Ln {}, Col {} │ utf-8 ", app.editor_cursor.0 + 1, app.editor_cursor.1 + 1)
    } else {
        format!("L{}, C{} ", app.editor_cursor.0 + 1, app.editor_cursor.1 + 1)
    };
    let pos_span = Span::styled(pos_text, Style::default().fg(theme.text_secondary));

    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(pos_len as u16)])
        .split(status_chunk);

    let left_spans = vec![mode_badge, name_span, desc_span];
    let clamped_left = clamp_spans_to_width(left_spans, status_chunks[0].width as usize);

    f.render_widget(Paragraph::new(Line::from(clamped_left)), status_chunks[0]);
    f.render_widget(Paragraph::new(Line::from(pos_span)).alignment(Alignment::Right), status_chunks[1]);

    // Active Cursor positioning
    if is_focused {
        if app.show_find_in_file {
            let cursor_x = chunks[1].x + 10 + app.find_query.chars().count() as u16;
            if cursor_x < chunks[1].x + chunks[1].width {
                f.set_cursor_position((cursor_x, chunks[1].y));
            }
        } else {
            let cursor_row = app.editor_cursor.0.saturating_sub(scroll);
            let cursor_col = app.editor_cursor.1 + 7; // line numbers margin
            if cursor_row < view_height && (cursor_col as u16) < inner.width {
                f.set_cursor_position((inner.x + cursor_col as u16, code_body_chunk.y + cursor_row as u16));
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SyntaxLang {
    Rust,
    JavaScript,
    Python,
    Json,
    Toml,
    Yaml,
    Html,
    Css,
    Shell,
    Markdown,
    Sql,
    Generic,
}

impl SyntaxLang {
    fn from_filename(file_name: Option<&str>) -> Self {
        let ext = file_name
            .and_then(|f| std::path::Path::new(f).extension())
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        match ext.as_str() {
            "rs" => Self::Rust,
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "astro" => Self::JavaScript,
            "py" | "pyw" => Self::Python,
            "json" | "jsonc" => Self::Json,
            "toml" => Self::Toml,
            "yaml" | "yml" => Self::Yaml,
            "html" | "htm" | "xml" | "svg" => Self::Html,
            "css" | "scss" | "sass" | "less" => Self::Css,
            "sh" | "bash" | "zsh" | "ps1" => Self::Shell,
            "md" | "markdown" => Self::Markdown,
            "sql" => Self::Sql,
            _ => Self::Generic,
        }
    }
}

fn highlight_code_line(line: &str, file_name: Option<&str>) -> Vec<Span<'static>> {
    let trimmed = line.trim_start();
    let lang = SyntaxLang::from_filename(file_name);

    // Full line comment check
    let is_comment = match lang {
        SyntaxLang::Python | SyntaxLang::Shell | SyntaxLang::Toml | SyntaxLang::Yaml => {
            trimmed.starts_with('#')
        }
        SyntaxLang::Sql => trimmed.starts_with("--"),
        SyntaxLang::Html => trimmed.starts_with("<!--"),
        _ => trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*'),
    };

    if is_comment {
        return vec![Span::styled(
            line.to_string(),
            Style::default()
                .fg(Color::Rgb(100, 116, 139))
                .add_modifier(Modifier::ITALIC),
        )];
    }

    // Markdown headers and list markers
    if lang == SyntaxLang::Markdown {
        if trimmed.starts_with('#') {
            return vec![Span::styled(
                line.to_string(),
                Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD),
            )];
        }
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            return vec![Span::styled(
                line.to_string(),
                Style::default().fg(Color::Rgb(251, 146, 60)),
            )];
        }
        if trimmed.starts_with('>') {
            return vec![Span::styled(
                line.to_string(),
                Style::default().fg(Color::Rgb(148, 163, 184)).add_modifier(Modifier::ITALIC),
            )];
        }
    }

    let chars: Vec<char> = line.chars().collect();
    let n = chars.len();
    let mut spans = Vec::new();
    let mut i = 0;

    while i < n {
        let c = chars[i];

        // 1. Whitespace
        if c.is_whitespace() {
            let start = i;
            while i < n && chars[i].is_whitespace() {
                i += 1;
            }
            let ws: String = chars[start..i].iter().collect();
            spans.push(Span::raw(ws));
            continue;
        }

        // 2. Trailing comments
        let is_trailing_comment = (c == '/' && i + 1 < n && chars[i + 1] == '/')
            || (matches!(lang, SyntaxLang::Python | SyntaxLang::Shell | SyntaxLang::Toml | SyntaxLang::Yaml) && c == '#')
            || (lang == SyntaxLang::Sql && c == '-' && i + 1 < n && chars[i + 1] == '-');

        if is_trailing_comment {
            let comment: String = chars[i..n].iter().collect();
            spans.push(Span::styled(
                comment,
                Style::default().fg(Color::Rgb(100, 116, 139)).add_modifier(Modifier::ITALIC),
            ));
            break;
        }

        // 3. Double-quote strings
        if c == '"' {
            let start = i;
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    i += 2;
                } else if chars[i] == '"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            let s: String = chars[start..i].iter().collect();
            // In JSON, keys like "key": are colored in Cyan
            let style = if lang == SyntaxLang::Json && i < n && (chars[i] == ':' || (chars[i].is_whitespace() && chars.get(i + 1) == Some(&':'))) {
                Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(74, 222, 128))
            };
            spans.push(Span::styled(s, style));
            continue;
        }

        // 4. Single-quote strings / character literals
        if c == '\'' {
            let start = i;
            i += 1;
            if lang == SyntaxLang::Rust {
                // Character literal or lifetime
                if i < n && chars[i] == '\\' && i + 2 < n && chars[i + 2] == '\'' {
                    i += 3;
                    let s: String = chars[start..i].iter().collect();
                    spans.push(Span::styled(s, Style::default().fg(Color::Rgb(74, 222, 128))));
                    continue;
                } else if i + 1 < n && chars[i + 1] == '\'' {
                    i += 2;
                    let s: String = chars[start..i].iter().collect();
                    spans.push(Span::styled(s, Style::default().fg(Color::Rgb(74, 222, 128))));
                    continue;
                } else if i < n && (chars[i].is_alphabetic() || chars[i] == '_') {
                    // Lifetime e.g. 'a
                    while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let s: String = chars[start..i].iter().collect();
                    spans.push(Span::styled(s, Style::default().fg(Color::Rgb(192, 132, 252))));
                    continue;
                }
            } else {
                while i < n {
                    if chars[i] == '\\' && i + 1 < n {
                        i += 2;
                    } else if chars[i] == '\'' {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                let s: String = chars[start..i].iter().collect();
                spans.push(Span::styled(s, Style::default().fg(Color::Rgb(74, 222, 128))));
                continue;
            }
        }

        // 5. Backtick template strings / inline code
        if c == '`' {
            let start = i;
            i += 1;
            while i < n {
                if chars[i] == '\\' && i + 1 < n {
                    i += 2;
                } else if chars[i] == '`' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, Style::default().fg(Color::Rgb(45, 212, 191))));
            continue;
        }

        // 6. Attributes / Decorators (@decorator, #[derive])
        if c == '@' || (c == '#' && lang == SyntaxLang::Rust && i + 1 < n && chars[i + 1] == '[') {
            let start = i;
            i += 1;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '[' || chars[i] == ']' || chars[i] == '(' || chars[i] == ')') {
                if chars[i] == ']' || chars[i] == ')' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, Style::default().fg(Color::Rgb(232, 121, 249))));
            continue;
        }

        // 7. Numbers (decimal, float, hex)
        if c.is_ascii_digit() || (c == '.' && i + 1 < n && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            spans.push(Span::styled(s, Style::default().fg(Color::Rgb(251, 146, 60))));
            continue;
        }

        // 8. Words / Identifiers / Keywords
        if c.is_alphabetic() || c == '_' || c == '$' {
            let start = i;
            while i < n && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$') {
                i += 1;
            }
            // Check macro call e.g. println! or vec!
            if i < n && chars[i] == '!' && lang == SyntaxLang::Rust {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            // Lookahead for function call (e.g. foo( )
            let mut peek = i;
            while peek < n && chars[peek].is_whitespace() {
                peek += 1;
            }
            let is_fn_call = peek < n && chars[peek] == '(';

            let style = match lang {
                SyntaxLang::Rust => match word.as_str() {
                    "fn" | "let" | "pub" | "mut" | "struct" | "enum" | "impl" | "use" | "mod" | "match"
                    | "return" | "async" | "await" | "trait" | "type" | "const" | "static" | "where"
                    | "ref" | "move" | "unsafe" | "loop" | "while" | "for" | "in" | "if" | "else"
                    | "break" | "continue" | "as" | "crate" | "super" => {
                        Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)
                    }
                    "Self" | "Option" | "Result" | "String" | "Vec" | "Box" | "Rc" | "Arc" | "bool"
                    | "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize"
                    | "f32" | "f64" | "char" | "str" | "Path" | "PathBuf" => {
                        Style::default().fg(Color::Rgb(250, 204, 21))
                    }
                    "true" | "false" | "Some" | "None" | "Ok" | "Err" => {
                        Style::default().fg(Color::Rgb(244, 63, 94))
                    }
                    w if w.ends_with('!') => {
                        Style::default().fg(Color::Rgb(192, 132, 252)).add_modifier(Modifier::BOLD)
                    }
                    _ if is_fn_call => Style::default().fg(Color::Rgb(125, 211, 252)),
                    _ => Style::default().fg(Color::Rgb(226, 232, 240)),
                },
                SyntaxLang::JavaScript => match word.as_str() {
                    "function" | "const" | "let" | "var" | "return" | "if" | "else" | "for" | "while"
                    | "do" | "switch" | "case" | "default" | "break" | "continue" | "import" | "export"
                    | "from" | "as" | "class" | "extends" | "implements" | "interface" | "type" | "enum"
                    | "new" | "this" | "super" | "async" | "await" | "yield" | "try" | "catch" | "finally"
                    | "throw" | "typeof" | "instanceof" | "void" | "delete" => {
                        Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)
                    }
                    "string" | "number" | "boolean" | "any" | "unknown" | "never" | "Array" | "Record"
                    | "Promise" | "Set" | "Map" | "Object" | "Function" | "Symbol" => {
                        Style::default().fg(Color::Rgb(250, 204, 21))
                    }
                    "true" | "false" | "null" | "undefined" | "NaN" | "Infinity" => {
                        Style::default().fg(Color::Rgb(244, 63, 94))
                    }
                    _ if is_fn_call => Style::default().fg(Color::Rgb(125, 211, 252)),
                    _ => Style::default().fg(Color::Rgb(226, 232, 240)),
                },
                SyntaxLang::Python => match word.as_str() {
                    "def" | "class" | "import" | "from" | "as" | "return" | "if" | "elif" | "else"
                    | "while" | "for" | "in" | "try" | "except" | "finally" | "with" | "raise" | "yield"
                    | "lambda" | "pass" | "break" | "continue" | "global" | "nonlocal" | "assert"
                    | "async" | "await" | "self" | "cls" => {
                        Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)
                    }
                    "int" | "float" | "str" | "bool" | "list" | "dict" | "set" | "tuple" | "bytes"
                    | "Any" | "Optional" | "Union" | "List" | "Dict" | "Set" | "Tuple" => {
                        Style::default().fg(Color::Rgb(250, 204, 21))
                    }
                    "True" | "False" | "None" => {
                        Style::default().fg(Color::Rgb(244, 63, 94))
                    }
                    _ if is_fn_call => Style::default().fg(Color::Rgb(125, 211, 252)),
                    _ => Style::default().fg(Color::Rgb(226, 232, 240)),
                },
                SyntaxLang::Sql => match word.to_ascii_uppercase().as_str() {
                    "SELECT" | "FROM" | "WHERE" | "INSERT" | "INTO" | "UPDATE" | "DELETE" | "CREATE"
                    | "TABLE" | "DROP" | "ALTER" | "JOIN" | "INNER" | "LEFT" | "RIGHT" | "ON"
                    | "GROUP" | "BY" | "ORDER" | "HAVING" | "LIMIT" | "OFFSET" | "AS" | "AND"
                    | "OR" | "NOT" | "PRIMARY" | "KEY" | "FOREIGN" | "REFERENCES" => {
                        Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)
                    }
                    "NULL" | "TRUE" | "FALSE" => Style::default().fg(Color::Rgb(244, 63, 94)),
                    _ => Style::default().fg(Color::Rgb(226, 232, 240)),
                },
                SyntaxLang::Shell => match word.as_str() {
                    "if" | "then" | "else" | "elif" | "fi" | "for" | "while" | "do" | "done"
                    | "case" | "esac" | "function" | "return" | "exit" | "local" | "export" | "source" => {
                        Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)
                    }
                    _ if is_fn_call => Style::default().fg(Color::Rgb(125, 211, 252)),
                    _ => Style::default().fg(Color::Rgb(226, 232, 240)),
                },
                _ => {
                    if is_fn_call {
                        Style::default().fg(Color::Rgb(125, 211, 252))
                    } else {
                        Style::default().fg(Color::Rgb(226, 232, 240))
                    }
                }
            };
            spans.push(Span::styled(word, style));
            continue;
        }

        // 9. Multi-char Operators (=>, ->, ::, ==, !=, <=, >=, &&, ||)
        if i + 1 < n {
            let op2: String = chars[i..i + 2].iter().collect();
            if matches!(op2.as_str(), "=>" | "->" | "::" | "==" | "!=" | "<=" | ">=" | "&&" | "||") {
                spans.push(Span::styled(op2, Style::default().fg(Color::Rgb(56, 189, 248))));
                i += 2;
                continue;
            }
        }

        // 10. Delimiters and Punctuation
        let style = match c {
            '(' | ')' | '[' | ']' | '{' | '}' => Style::default().fg(Color::Rgb(148, 163, 184)),
            ';' | ',' | '.' | ':' => Style::default().fg(Color::Rgb(100, 116, 139)),
            '+' | '-' | '*' | '/' | '%' | '=' | '<' | '>' | '!' | '&' | '|' | '^' => {
                Style::default().fg(Color::Rgb(56, 189, 248))
            }
            _ => Style::default().fg(Color::Rgb(203, 213, 225)),
        };
        spans.push(Span::styled(c.to_string(), style));
        i += 1;
    }

    spans
}

fn render_code_line_with_search(line: &str, row_idx: usize, app: &App) -> Vec<Span<'static>> {
    let file_name = app.editor_file_path.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str());
    if !app.show_find_in_file || app.find_matches.is_empty() {
        return highlight_code_line(line, file_name);
    }

    let line_matches: Vec<(usize, usize, usize)> = app
        .find_matches
        .iter()
        .enumerate()
        .filter_map(|(global_idx, &(r, c, len))| {
            if r == row_idx {
                Some((global_idx, c, len))
            } else {
                None
            }
        })
        .collect();

    if line_matches.is_empty() {
        return highlight_code_line(line, file_name);
    }

    let char_indices: Vec<(usize, char)> = line.char_indices().collect();
    let total_chars = char_indices.len();
    let mut spans = Vec::new();
    let mut curr_char = 0;

    for (global_idx, col, len) in line_matches {
        if col < curr_char {
            continue;
        }

        // Highlight any code before this match with standard syntax highlighter
        if col > curr_char && curr_char < total_chars {
            let start_byte = char_indices[curr_char].0;
            let end_byte = if col < total_chars { char_indices[col].0 } else { line.len() };
            if start_byte < end_byte {
                spans.extend(highlight_code_line(&line[start_byte..end_byte], file_name));
            }
        }

        // Render the search match itself
        let match_end_char = (col + len).min(total_chars);
        if col < total_chars {
            let start_byte = char_indices[col].0;
            let end_byte = if match_end_char < total_chars { char_indices[match_end_char].0 } else { line.len() };
            if start_byte < end_byte {
                let is_active = global_idx == app.active_find_match_idx;
                let match_style = if is_active {
                    Style::default().bg(Color::Rgb(245, 158, 11)).fg(Color::Black).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().bg(Color::Rgb(161, 98, 7)).fg(Color::White).add_modifier(Modifier::BOLD)
                };
                spans.push(Span::styled(line[start_byte..end_byte].to_string(), match_style));
            }
        }
        curr_char = col + len;
    }

    // Render remaining code after the last match
    if curr_char < total_chars {
        let start_byte = char_indices[curr_char].0;
        if start_byte < line.len() {
            spans.extend(highlight_code_line(&line[start_byte..], file_name));
        }
    }

    spans
}

// Helper: Clamps a vector of spans to a maximum display column width so it never overflows
fn clamp_spans_to_width<'a>(spans: Vec<Span<'a>>, max_w: usize) -> Vec<Span<'a>> {
    let mut current_w = 0;
    let mut result = Vec::new();
    for span in spans {
        let span_len = span.content.chars().count();
        if current_w + span_len <= max_w {
            current_w += span_len;
            result.push(span);
        } else {
            let remaining = max_w.saturating_sub(current_w);
            if remaining > 0 {
                let truncated: String = span.content.chars().take(remaining).collect();
                result.push(Span::styled(truncated, span.style));
            }
            break;
        }
    }
    result
}

fn draw_explorer_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let is_focused = app.active_pane == ActivePane::Explorer;
    let border_color = if is_focused {
        theme.border_active
    } else {
        theme.border_inactive
    };

    let exp_icon = crate::icons::pane_icon("explorer", app.icon_theme);
    let title = if area.width >= 16 {
        format!(" {}Explorer [×] ", exp_icon)
    } else if area.width >= 10 {
        format!(" {}Exp ", exp_icon)
    } else {
        format!(" {} ", exp_icon)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.surface));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Responsive footer: calculate height and spans based on available dimensions
    let use_two_line_footer = inner.width < 32 && inner.height >= 7 && inner.width >= 16;
    let has_footer = inner.height >= 5 && inner.width >= 10;
    let footer_height = if use_two_line_footer {
        2
    } else if has_footer {
        1
    } else {
        0
    };

    let chunks = if footer_height > 0 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(footer_height)])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(inner)
    };

    let list_area = chunks[0];
    let view_height = list_area.height as usize;
    let scroll = app.explorer.effective_scroll(view_height);

    let items: Vec<ListItem> = app
        .explorer
        .entries
        .iter()
        .enumerate()
        .skip(scroll)
        .take(view_height)
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.depth);
            let arrow = crate::icons::arrow_icon(entry.is_dir, entry.is_expanded, app.icon_theme);
            let (icon, icon_color) = entry.icon_styled(app.icon_theme);
            let is_selected = i == app.explorer.selected;

            let row_style = if is_selected {
                Style::default()
                    .bg(theme.selection_bg)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(theme.text_secondary)
            } else {
                Style::default().fg(theme.text_primary)
            };

            let spans = vec![
                Span::styled(format!("{}{}", indent, arrow), if is_selected { Style::default().fg(theme.text_secondary) } else { Style::default().fg(theme.text_muted) }),
                Span::styled(icon, if is_selected { Style::default().fg(Color::White) } else { Style::default().fg(icon_color) }),
                Span::styled(entry.name.clone(), row_style),
            ];

            ListItem::new(Line::from(spans)).style(row_style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, list_area);

    if footer_height > 0 {
        app.explorer_footer_area = chunks[1];
        let w = chunks[1].width as usize;

        if footer_height == 2 {
            // Two-line responsive layout (16..31 cols): provides ample breathing room and all 4 operations
            let (r1_left_k, r1_left_lbl, r1_right_k, r1_right_lbl) = if w >= 26 {
                (" [a] ", "New", "[r] ", "Rename")
            } else if w >= 20 {
                (" [a] ", "New", "[r] ", "Ren")
            } else {
                (" [a]", "+", "[r]", "✎")
            };

            let (r2_left_k, r2_left_lbl, r2_right_k, r2_right_lbl) = if w >= 26 {
                (" [d] ", "Delete", "[R] ", "Refresh")
            } else if w >= 20 {
                (" [d] ", "Del", "[R] ", "Ref")
            } else {
                (" [d]", "×", "[R]", "↻")
            };

            let l1_len = r1_left_k.chars().count() + r1_left_lbl.chars().count();
            let r1_len = r1_right_k.chars().count() + r1_right_lbl.chars().count();
            let gap1 = w.saturating_sub(l1_len + r1_len).max(1);

            let row1_spans = vec![
                Span::styled(r1_left_k, Style::default().fg(theme.warning)),
                Span::styled(r1_left_lbl, Style::default().fg(theme.text_secondary)),
                Span::raw(" ".repeat(gap1)),
                Span::styled(r1_right_k, Style::default().fg(theme.warning)),
                Span::styled(r1_right_lbl, Style::default().fg(theme.text_secondary)),
            ];

            let l2_len = r2_left_k.chars().count() + r2_left_lbl.chars().count();
            let r2_len = r2_right_k.chars().count() + r2_right_lbl.chars().count();
            let gap2 = w.saturating_sub(l2_len + r2_len).max(1);

            let row2_spans = vec![
                Span::styled(r2_left_k, Style::default().fg(theme.danger)),
                Span::styled(r2_left_lbl, Style::default().fg(theme.text_secondary)),
                Span::raw(" ".repeat(gap2)),
                Span::styled(r2_right_k, Style::default().fg(theme.warning)),
                Span::styled(r2_right_lbl, Style::default().fg(theme.text_secondary)),
            ];

            let clamped1 = clamp_spans_to_width(row1_spans, w);
            let clamped2 = clamp_spans_to_width(row2_spans, w);

            let footer_lines = vec![Line::from(clamped1), Line::from(clamped2)];
            f.render_widget(Paragraph::new(footer_lines).style(Style::default().bg(theme.surface)), chunks[1]);
        } else {
            // Single-line responsive layout
            let footer_spans: Vec<Span<'static>> = if w >= 42 {
                // Wide verbose (>= 42 cols): 41 cols
                vec![
                    Span::styled(" [a] ", Style::default().fg(theme.warning)),
                    Span::styled("New  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[r] ", Style::default().fg(theme.warning)),
                    Span::styled("Rename  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[d] ", Style::default().fg(theme.danger)),
                    Span::styled("Delete  ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[R] ", Style::default().fg(theme.warning)),
                    Span::styled("Ref", Style::default().fg(theme.text_secondary)),
                ]
            } else if w >= 32 {
                // Standard compact (32..41 cols): exactly 32 cols
                vec![
                    Span::styled(" [a] ", Style::default().fg(theme.warning)),
                    Span::styled("New ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[r] ", Style::default().fg(theme.warning)),
                    Span::styled("Ren ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[d] ", Style::default().fg(theme.danger)),
                    Span::styled("Del ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[R] ", Style::default().fg(theme.warning)),
                    Span::styled("Ref", Style::default().fg(theme.text_secondary)),
                ]
            } else if w >= 28 {
                // Tight 4-actions (28..31 cols when 1-line): exactly 28 cols
                vec![
                    Span::styled(" [a]", Style::default().fg(theme.warning)),
                    Span::styled("New ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[r]", Style::default().fg(theme.warning)),
                    Span::styled("Ren ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[d]", Style::default().fg(theme.danger)),
                    Span::styled("Del ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[R]", Style::default().fg(theme.warning)),
                    Span::styled("Ref", Style::default().fg(theme.text_secondary)),
                ]
            } else if w >= 21 {
                // Core 3-actions (21..27 cols): exactly 20 cols
                vec![
                    Span::styled(" [a]", Style::default().fg(theme.warning)),
                    Span::styled("New ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[r]", Style::default().fg(theme.warning)),
                    Span::styled("Ren ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[d]", Style::default().fg(theme.danger)),
                    Span::styled("Del", Style::default().fg(theme.text_secondary)),
                ]
            } else if w >= 15 {
                // Symbolic icons (15..20 cols): exactly 14 cols
                vec![
                    Span::styled(" [a]", Style::default().fg(theme.warning)),
                    Span::styled("+ ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[r]", Style::default().fg(theme.warning)),
                    Span::styled("✎ ", Style::default().fg(theme.text_secondary)),
                    Span::styled("[d]", Style::default().fg(theme.danger)),
                    Span::styled("×", Style::default().fg(theme.text_secondary)),
                ]
            } else {
                // Keys only (10..14 cols): exactly 10 cols
                vec![
                    Span::styled("[a]", Style::default().fg(theme.warning)),
                    Span::raw(" "),
                    Span::styled("[r]", Style::default().fg(theme.warning)),
                    Span::raw(" "),
                    Span::styled("[d]", Style::default().fg(theme.danger)),
                ]
            };

            let clamped = clamp_spans_to_width(footer_spans, w);
            f.render_widget(Paragraph::new(Line::from(clamped)).style(Style::default().bg(theme.surface)), chunks[1]);
        }
    } else {
        app.explorer_footer_area = Rect::default();
    }

    // Render Explorer scrollbar if entries exceed area height
    let total_entries = app.explorer.entries.len();
    if total_entries > view_height && view_height > 0 {
        let max_scroll = total_entries.saturating_sub(view_height);
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll.min(max_scroll));
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.accent));
        f.render_stateful_widget(scrollbar, list_area, &mut scrollbar_state);
    }
}

fn draw_terminal_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let is_focused = app.active_pane == ActivePane::Terminal;
    let border_color = if is_focused {
        theme.border_active
    } else {
        theme.border_inactive
    };

    // Inner layout: Tabs (1) + Terminal Screen (remaining)
    let term_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.bg));
    
    let inner_area = term_block.inner(area);
    f.render_widget(term_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);

    app.terminal_tabs_area = chunks[0];

    // Tab bar with interactive tabs, [+] button, and [F7 ◀] [▶ F8] navigation controls
    let term_bar_w = chunks[0].width as usize;
    let right_controls_len: u16 = if term_bar_w >= 60 {
        28
    } else if term_bar_w >= 38 {
        16
    } else {
        8
    };

    let tab_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(right_controls_len)])
        .split(chunks[0]);

    let total_terminals = app.terminals.len();
    let available_w = tab_bar_chunks[0].width as usize;

    let tab_widths: Vec<usize> = app.terminals.iter().enumerate().map(|(i, t)| {
        let clean = crate::app::format_terminal_tab_title(&t.title);
        4 + format!("{}: {}", i + 1, clean).chars().count() + 2
    }).collect();

    // Auto-scroll tab window so active_terminal_idx is visible
    if app.active_terminal_idx < app.terminal_tab_scroll {
        app.terminal_tab_scroll = app.active_terminal_idx;
    }
    let mut test_w = 0;
    for i in app.terminal_tab_scroll..total_terminals {
        test_w += tab_widths.get(i).copied().unwrap_or(15);
        if i == app.active_terminal_idx {
            break;
        }
    }
    while test_w + 10 > available_w && app.terminal_tab_scroll < app.active_terminal_idx {
        test_w = test_w.saturating_sub(tab_widths.get(app.terminal_tab_scroll).copied().unwrap_or(15));
        app.terminal_tab_scroll += 1;
    }

    let mut tab_spans = Vec::new();

    // Left overflow arrow if scrolled
    if app.terminal_tab_scroll > 0 {
        tab_spans.push(Span::styled(" ‹ ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)));
    }

    let mut accumulated_w = if app.terminal_tab_scroll > 0 { 3 } else { 0 };
    let mut last_visible_tab = app.terminal_tab_scroll;

    for i in app.terminal_tab_scroll..total_terminals {
        let t = &app.terminals[i];
        let clean_title = crate::app::format_terminal_tab_title(&t.title);
        let tab_w = tab_widths.get(i).copied().unwrap_or(15);

        if i > app.terminal_tab_scroll && accumulated_w + tab_w + 8 > available_w {
            break;
        }

        let is_active = i == app.active_terminal_idx;
        let indicator = if is_active { "● " } else { "○ " };
        let dot_color = if is_active { theme.success } else { theme.text_muted };
        let text_style = if is_active {
            Style::default().bg(theme.tab_active_bg).fg(theme.tab_active_fg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(theme.tab_inactive_bg).fg(theme.tab_inactive_fg)
        };

        tab_spans.push(Span::styled(format!(" {} ", indicator), Style::default().fg(dot_color)));
        tab_spans.push(Span::styled(format!("{}: {} ", i + 1, clean_title), text_style));
        tab_spans.push(Span::styled(" │", Style::default().fg(theme.border_inactive)));

        accumulated_w += tab_w;
        last_visible_tab = i;
    }

    // Right overflow arrow if more tabs exist
    if last_visible_tab + 1 < total_terminals {
        tab_spans.push(Span::styled(" › ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)));
    }

    // [+] New tab button
    tab_spans.push(Span::styled(" [+] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));

    let mut right_tab_controls = Vec::new();

    // Scrollback status badge
    if let Some(session) = app.terminals.get(app.active_terminal_idx) {
        let scroll_offset = session.scrollback();
        if scroll_offset > 0 {
            let sc_icon = crate::icons::action_icon("scroll", app.icon_theme);
            right_tab_controls.push(Span::styled(
                format!(" [{} -{} lines | Shift+End: Live] ", sc_icon.trim(), scroll_offset),
                Style::default().bg(theme.warning).fg(Color::Black).add_modifier(Modifier::BOLD),
            ));
        }
    }

    // Zoom badge
    if app.terminal_zoom_level != 0 {
        let pct = 100 + (app.terminal_zoom_level as i16) * 10;
        let zm_icon = crate::icons::action_icon("zoom", app.icon_theme);
        right_tab_controls.push(Span::styled(
            format!(" [{} {}%] ", zm_icon.trim(), pct),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));
    }

    if term_bar_w >= 60 {
        right_tab_controls.push(Span::styled(format!("({}/{}) ", app.active_terminal_idx + 1, total_terminals.max(1)), Style::default().fg(theme.text_secondary)));
        right_tab_controls.push(Span::styled("[F7 ◀] ", Style::default().fg(theme.warning)));
        right_tab_controls.push(Span::styled("[▶ F8]", Style::default().fg(theme.warning)));
        right_tab_controls.push(Span::raw(" "));
    } else if term_bar_w >= 38 {
        right_tab_controls.push(Span::styled("[F7 ◀] ", Style::default().fg(theme.warning)));
        right_tab_controls.push(Span::styled("[▶ F8]", Style::default().fg(theme.warning)));
    } else {
        right_tab_controls.push(Span::styled("[◀]", Style::default().fg(theme.warning)));
        right_tab_controls.push(Span::styled("[▶]", Style::default().fg(theme.warning)));
    }

    f.render_widget(Paragraph::new(Line::from(tab_spans)), tab_bar_chunks[0]);
    f.render_widget(Paragraph::new(Line::from(right_tab_controls)).alignment(Alignment::Right), tab_bar_chunks[1]);

    // Render VT100 screen buffer
    if let Some(session) = app.terminals.get(app.active_terminal_idx) {
        let screen_area = chunks[1];
        let scroll_offset = session.scrollback();
        let scrollbar_needed = scroll_offset > 0;

        let (screen_grid_area, scrollbar_area) = if scrollbar_needed {
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0), Constraint::Length(1)])
                .split(screen_area);
            (h_chunks[0], h_chunks[1])
        } else {
            (screen_area, Rect::default())
        };
        
        // Dynamically resize PTY to match current pane dimensions
        if screen_grid_area.height > 0 && screen_grid_area.width > 0 {
            session.resize(screen_grid_area.height, screen_grid_area.width);
        }

        if let Ok(parser) = session.parser.lock() {
            let screen = parser.screen();
            let mut lines: Vec<Line> = Vec::new();

            for row in 0..screen_grid_area.height {
                let mut spans: Vec<Span> = Vec::new();
                for col in 0..screen_grid_area.width {
                    let cell = screen.cell(row, col);
                    if let Some(cell) = cell {
                        if cell.is_wide_continuation() {
                            continue;
                        }

                        let ch = cell.contents();
                        let text = if ch.is_empty() { " " } else { &ch };
                        
                        let fg = convert_vt100_color(cell.fgcolor());
                        let bg = convert_vt100_color(cell.bgcolor());

                        let mut style = Style::default();
                        if let Some(c) = fg {
                            style = style.fg(c);
                        } else {
                            style = style.fg(theme.text_primary);
                        }
                        if let Some(c) = bg {
                            style = style.bg(c);
                        }
                        if cell.bold() { style = style.add_modifier(Modifier::BOLD); }
                        if cell.italic() { style = style.add_modifier(Modifier::ITALIC); }
                        if cell.underline() { style = style.add_modifier(Modifier::UNDERLINED); }
                        if cell.inverse() { style = style.add_modifier(Modifier::REVERSED); }

                        spans.push(Span::styled(text.to_string(), style));
                    } else {
                        spans.push(Span::raw(" "));
                    }
                }
                lines.push(Line::from(spans));
            }

            f.render_widget(Paragraph::new(lines), screen_grid_area);

            if scrollbar_needed {
                let max_scroll = 5000.max(scroll_offset);
                let mut scrollbar_state = ScrollbarState::new(max_scroll).position(max_scroll.saturating_sub(scroll_offset));
                let scrollbar = Scrollbar::default()
                    .orientation(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("▲"))
                    .end_symbol(Some("▼"))
                    .track_symbol(Some("│"))
                    .thumb_symbol("█")
                    .style(Style::default().fg(theme.accent));
                f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
            }

            // Active Cursor placement (only display live cursor when not viewing historical scrollback)
            if is_focused && scroll_offset == 0 {
                let (c_row, c_col) = screen.cursor_position();
                if c_row < screen_grid_area.height && c_col < screen_grid_area.width {
                    f.set_cursor_position((screen_grid_area.x + c_col, screen_grid_area.y + c_row));
                }
            }
        }
    }
}

fn draw_git_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let is_focused = app.active_pane == ActivePane::Git;
    let border_color = if is_focused {
        theme.border_active
    } else {
        theme.border_inactive
    };

    let git_icon = crate::icons::pane_icon("git", app.icon_theme);
    let staged_count = app.git_info.staged_files.len();
    let unstaged_count = app.git_info.unstaged_files.len();
    let total_items = staged_count + unstaged_count;

    let title = if area.width >= 35 {
        format!(" {}Git ({} staged, {} unstaged) [×] ", git_icon, staged_count, unstaged_count)
    } else if area.width >= 16 {
        format!(" {}Git ({} chg) [×] ", git_icon, total_items)
    } else {
        format!(" {}Git [×] ", git_icon)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.surface));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Split inner area into:
    // 1. Content list (Min 0)
    // 2. Footer action bar (Length 1 if focused)
    let has_footer = is_focused && inner.height >= 4;
    let chunks = if has_footer {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(inner)
    };

    let content_area = chunks[0];

    let mut lines: Vec<Line> = Vec::new();

    // 1. Branch and repository status line
    let mut branch_spans = vec![
        Span::styled(" ", Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
        Span::styled(&app.git_info.branch, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
    ];
    if content_area.width >= 35 {
        branch_spans.push(Span::styled("  [b] Branch", Style::default().fg(theme.accent_secondary)));
        branch_spans.push(Span::styled(" [z] Stash", Style::default().fg(theme.accent)));
    }
    if app.git_info.is_clean {
        branch_spans.push(Span::styled(" │ ", Style::default().fg(theme.text_muted)));
        branch_spans.push(Span::styled("● Clean", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)));
    } else {
        branch_spans.push(Span::styled(" │ ", Style::default().fg(theme.text_muted)));
        branch_spans.push(Span::styled(format!("● {} staged", staged_count), Style::default().fg(theme.success)));
        branch_spans.push(Span::styled(format!(", {} unstaged", unstaged_count), Style::default().fg(theme.warning)));
    }
    lines.push(Line::from(branch_spans));

    // 2. Staged Changes Section
    lines.push(Line::raw(""));
    let staged_header_spans = vec![
        Span::styled("▼ ", Style::default().fg(theme.success)),
        Span::styled(format!("Staged Changes ({})", staged_count), Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
    ];
    lines.push(Line::from(staged_header_spans));

    if app.git_info.staged_files.is_empty() {
        lines.push(Line::styled("    (No staged changes — press [s] or [a] to stage)", Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC)));
    } else {
        for (i, item) in app.git_info.staged_files.iter().enumerate() {
            let is_selected = is_focused && app.git_selected_idx == i;
            let (badge_text, badge_color) = item.status.badge();
            let prefix = if is_selected { " ▶ " } else { "   " };

            let row_spans = vec![
                Span::styled(prefix, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("[✓] ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", badge_text), Style::default().fg(badge_color).add_modifier(Modifier::BOLD)),
                Span::styled(&item.path, Style::default().fg(if is_selected { theme.text_primary } else { theme.text_secondary })),
            ];

            let row_style = if is_selected {
                Style::default().bg(theme.selection_bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            lines.push(Line::from(row_spans).style(row_style));
        }
    }

    // 3. Unstaged Changes Section
    lines.push(Line::raw(""));
    let unstaged_header_spans = vec![
        Span::styled("▼ ", Style::default().fg(theme.warning)),
        Span::styled(format!("Changes / Untracked ({})", unstaged_count), Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
    ];
    lines.push(Line::from(unstaged_header_spans));

    if app.git_info.unstaged_files.is_empty() {
        lines.push(Line::styled("    (Working tree clean)", Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC)));
    } else {
        for (j, item) in app.git_info.unstaged_files.iter().enumerate() {
            let global_idx = staged_count + j;
            let is_selected = is_focused && app.git_selected_idx == global_idx;
            let (badge_text, badge_color) = item.status.badge();
            let prefix = if is_selected { " ▶ " } else { "   " };

            let row_spans = vec![
                Span::styled(prefix, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("[ ] ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", badge_text), Style::default().fg(badge_color).add_modifier(Modifier::BOLD)),
                Span::styled(&item.path, Style::default().fg(if is_selected { theme.text_primary } else { theme.text_secondary })),
            ];

            let row_style = if is_selected {
                Style::default().bg(theme.selection_bg).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            lines.push(Line::from(row_spans).style(row_style));
        }
    }

    // 4. Recent Commits Section
    if !app.git_info.recent_commits.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::styled("▼ Recent Commits", Style::default().fg(theme.text_muted).add_modifier(Modifier::UNDERLINED)));
        for commit in &app.git_info.recent_commits {
            let mut spans = vec![
                Span::styled(format!("  {} ", commit.hash), Style::default().fg(theme.accent)),
                Span::styled(commit.message.clone(), Style::default().fg(theme.text_secondary)),
            ];
            if content_area.width >= 35 {
                spans.push(Span::styled(format!(" ({})", commit.relative_time), Style::default().fg(theme.text_muted)));
            }
            lines.push(Line::from(spans));
        }
    }

    // Auto-scroll calculation
    let view_height = content_area.height as usize;
    let total_lines = lines.len();

    // Determine target line for selected item to ensure it's visible
    if total_items > 0 {
        if app.git_selected_idx >= total_items {
            app.git_selected_idx = total_items - 1;
        }
        let target_line = if app.git_selected_idx < staged_count {
            3 + app.git_selected_idx
        } else {
            let unstaged_rel = app.git_selected_idx - staged_count;
            let staged_offset = if staged_count == 0 { 1 } else { staged_count };
            3 + staged_offset + 2 + unstaged_rel
        };

        if target_line < app.git_scroll {
            app.git_scroll = target_line;
        } else if view_height > 0 && target_line >= app.git_scroll + view_height {
            app.git_scroll = target_line + 1 - view_height;
        }
    }

    let max_scroll = total_lines.saturating_sub(view_height);
    app.git_scroll = app.git_scroll.min(max_scroll);

    let scrollbar_needed = total_lines > view_height && view_height > 0;
    let (list_area, scrollbar_area) = if scrollbar_needed {
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(content_area);
        (h_chunks[0], h_chunks[1])
    } else {
        (content_area, Rect::default())
    };

    let visible_lines: Vec<Line> = lines.into_iter().skip(app.git_scroll).take(view_height).collect();
    let p = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
    f.render_widget(p, list_area);

    if scrollbar_needed {
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(app.git_scroll);
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.accent));
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // 5. Render Footer Action Bar
    if has_footer {
        let footer_area = chunks[1];
        let w = footer_area.width as usize;

        let footer_spans = if w >= 60 {
            vec![
                Span::styled("[s] ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                Span::styled("Stage ", Style::default().fg(theme.text_primary)),
                Span::styled("[c] ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled("Commit ", Style::default().fg(theme.text_primary)),
                Span::styled("[p] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("Push ", Style::default().fg(theme.text_primary)),
                Span::styled("[P] ", Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
                Span::styled("Pull ", Style::default().fg(theme.text_primary)),
                Span::styled("[b] ", Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
                Span::styled("Branch ", Style::default().fg(theme.text_primary)),
                Span::styled("[z] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("Stash ", Style::default().fg(theme.text_primary)),
                Span::styled("[d] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("Diff", Style::default().fg(theme.text_primary)),
            ]
        } else if w >= 38 {
            vec![
                Span::styled("[s] ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                Span::styled("Stage  ", Style::default().fg(theme.text_primary)),
                Span::styled("[c] ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled("Commit  ", Style::default().fg(theme.text_primary)),
                Span::styled("[b] ", Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
                Span::styled("Branch  ", Style::default().fg(theme.text_primary)),
                Span::styled("[z] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("Stash  ", Style::default().fg(theme.text_primary)),
                Span::styled("[d] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("Diff", Style::default().fg(theme.text_primary)),
            ]
        } else {
            vec![
                Span::styled("[s] [c] [b] [z] [p] [d]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            ]
        };

        f.render_widget(Paragraph::new(Line::from(footer_spans)).style(Style::default().bg(theme.surface)), footer_area);
    }
}

fn draw_agents_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let is_focused = app.active_pane == ActivePane::Agents;
    let border_color = if is_focused {
        theme.border_active
    } else {
        theme.border_inactive
    };

    let total_agents = app.agents.len();
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.surface));

    let inner = block.inner(area);
    let view_height = inner.height as usize;
    let cards_per_screen = (view_height / 4).max(1);

    let agt_icon = crate::icons::pane_icon("agents", app.icon_theme);
    let title = if total_agents > cards_per_screen {
        format!(" {}Agents ({}/{} shown) [×] ", agt_icon, cards_per_screen.min(total_agents), total_agents)
    } else {
        format!(" {}Agents ({} running) [×] ", agt_icon, total_agents)
    };
    block = block.title(title);
    f.render_widget(block, area);

    let mut lines = Vec::new();

    if app.agents.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::styled("  No active agent sessions.", Style::default().fg(theme.text_muted)));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("  Press "),
            Span::styled("[a]", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::raw(" to launch an agent:"),
        ]));
        lines.push(Line::styled("  1. Antigravity CLI (agy)", Style::default().fg(theme.accent)));
        lines.push(Line::styled("  2. Claude Code CLI (claude)", Style::default().fg(theme.accent)));
        lines.push(Line::styled("  3. Worktree Shell (powershell)", Style::default().fg(theme.accent)));
        lines.push(Line::raw(""));
        lines.push(Line::styled("  Each agent runs in an isolated Git worktree.", Style::default().fg(theme.text_muted)));
    } else {
        // Auto-scroll so active_agent_idx is always visible
        if app.active_agent_idx < app.agents_scroll {
            app.agents_scroll = app.active_agent_idx;
        } else if app.active_agent_idx >= app.agents_scroll + cards_per_screen {
            app.agents_scroll = app.active_agent_idx + 1 - cards_per_screen;
        }
        let max_scroll = total_agents.saturating_sub(cards_per_screen);
        app.agents_scroll = app.agents_scroll.min(max_scroll);

        let visible_agents = app.agents.iter().enumerate().skip(app.agents_scroll).take(cards_per_screen);

        for (i, agent) in visible_agents {
            let is_selected = is_focused && i == app.active_agent_idx;
            let (dot_color, status_text) = match &agent.state {
                AgentState::Running => (theme.success, "Running"),
                AgentState::WaitingForInput => (theme.warning, "Waiting for input"),
                AgentState::Completed => (theme.accent, "Completed"),
                AgentState::Failed(_) => (theme.danger, "Failed"),
                AgentState::Idle => (theme.text_muted, "Idle"),
            };

            let conf_tag = match agent.confidence {
                Confidence::Confirmed => "[Confirmed]",
                Confidence::Inferred => "[Inferred?]",
                Confidence::Unknown => "[Unknown]",
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let tab_hint = if let Some(t_idx) = agent.terminal_idx {
                format!("Tab #{}", t_idx + 1)
            } else {
                "Detached".to_string()
            };

            let mut header_spans = vec![
                Span::styled(prefix, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled("● ", Style::default().fg(dot_color)),
                Span::styled(&agent.name, Style::default().fg(if is_selected { Color::White } else { theme.text_primary }).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(status_text, Style::default().fg(dot_color)),
            ];
            if inner.width >= 40 {
                header_spans.push(Span::styled(format!(" {} ", conf_tag), Style::default().fg(theme.text_muted)));
                header_spans.push(Span::styled(agent.formatted_runtime(), Style::default().fg(theme.text_secondary)));
            }
            if is_selected {
                if agent.state == AgentState::Running || agent.state == AgentState::WaitingForInput {
                    header_spans.push(Span::styled(" [s: Stop]", Style::default().bg(theme.danger).fg(Color::White).add_modifier(Modifier::BOLD)));
                }
                if inner.width >= 30 {
                    header_spans.push(Span::styled(" [Enter: Tab]", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)));
                }
            }

            let row_style = if is_selected {
                Style::default().bg(theme.selection_bg)
            } else {
                Style::default()
            };

            lines.push(Line::from(header_spans).style(row_style));

            lines.push(Line::from(vec![
                Span::raw("     ↳ "),
                Span::styled(format!("Branch: {}", agent.worktree_branch), Style::default().fg(theme.accent_secondary)),
                Span::styled(" │ ", Style::default().fg(theme.text_muted)),
                Span::styled(format!("Window: {}", tab_hint), Style::default().fg(theme.accent)),
            ]));

            lines.push(Line::from(vec![
                Span::raw("     ↳ "),
                Span::styled(format!("Task: {}", agent.target_file_or_task), Style::default().fg(theme.text_secondary)),
            ]));

            lines.push(Line::raw(""));
        }
    }

    if is_focused {
        let footer_hint = if inner.width >= 55 {
            "[a] Launch [s] Stop [d] Diff [m] Merge [x] Discard [Enter] Shell"
        } else if inner.width >= 35 {
            "[a] Launch  [s] Stop  [Enter] Shell"
        } else if inner.width >= 20 {
            "[a] Launch  [s] Stop"
        } else {
            "[a] [s] [Enter]"
        };
        lines.push(Line::styled(
            footer_hint,
            Style::default().fg(theme.accent),
        ));
    }

    let scrollbar_needed = total_agents > cards_per_screen && view_height > 0;
    let (content_area, scrollbar_area) = if scrollbar_needed {
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner);
        (h_chunks[0], h_chunks[1])
    } else {
        (inner, Rect::default())
    };

    let p = Paragraph::new(lines).wrap(Wrap { trim: true });
    f.render_widget(p, content_area);

    // Render vertical Scrollbar on Agents pane
    if scrollbar_needed {
        let max_scroll = total_agents.saturating_sub(cards_per_screen);
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(app.agents_scroll.min(max_scroll));
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.accent));
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let w = area.width as usize;
    let (left_constraint, right_constraint) = if w >= 110 {
        (Constraint::Min(30), Constraint::Length(45))
    } else if w >= 80 {
        (Constraint::Min(25), Constraint::Length(28))
    } else {
        (Constraint::Min(20), Constraint::Length(14))
    };

    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([left_constraint, right_constraint])
        .split(area);

    let active_shell = app
        .terminals
        .get(app.active_terminal_idx)
        .map(|t| t.title.as_str())
        .unwrap_or("Shell");

    let raw_path = app.workspace_dir.to_string_lossy();
    let clean_path = raw_path.strip_prefix(r"\\?\").unwrap_or(&raw_path);

    let left = if w >= 80 {
        vec![
            Span::styled(" ● ", Style::default().fg(theme.success)),
            Span::styled("TIDE", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border_inactive)),
            Span::styled(active_shell, Style::default().fg(theme.accent)),
            Span::styled(" │ ", Style::default().fg(theme.border_inactive)),
            Span::styled(clean_path.to_string(), Style::default().fg(theme.text_secondary)),
        ]
    } else {
        vec![
            Span::styled(" ● ", Style::default().fg(theme.success)),
            Span::styled("TIDE", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border_inactive)),
            Span::styled(active_shell, Style::default().fg(theme.accent)),
        ]
    };

    let right = if w >= 110 {
        vec![
            Span::styled(format!("Workspace: {} ", app.workspace_name), Style::default().fg(theme.text_secondary)),
            Span::styled("│ ", Style::default().fg(theme.border_inactive)),
            Span::styled(format!("Git: {} ", app.git_info.branch), Style::default().fg(theme.accent_secondary)),
            Span::styled("│ ", Style::default().fg(theme.border_inactive)),
            Span::styled(format!("Agents: {}/{}", app.agents.len(), app.agents.len()), Style::default().fg(theme.success)),
        ]
    } else if w >= 80 {
        vec![
            Span::styled(format!("Git: {} ", app.git_info.branch), Style::default().fg(theme.accent_secondary)),
            Span::styled("│ ", Style::default().fg(theme.border_inactive)),
            Span::styled(format!("Agt: {}/{}", app.agents.len(), app.agents.len()), Style::default().fg(theme.success)),
        ]
    } else {
        vec![
            Span::styled(format!("Agt: {}/{}", app.agents.len(), app.agents.len()), Style::default().fg(theme.success)),
        ]
    };

    let clamped_left = clamp_spans_to_width(left, status_chunks[0].width as usize);
    let clamped_right = clamp_spans_to_width(right, status_chunks[1].width as usize);

    let bg_style = Style::default().bg(theme.surface_variant);
    f.render_widget(Block::default().style(bg_style), area);
    f.render_widget(Paragraph::new(Line::from(clamped_left)).style(bg_style), status_chunks[0]);
    f.render_widget(Paragraph::new(Line::from(clamped_right)).style(bg_style).alignment(Alignment::Right), status_chunks[1]);
}

fn draw_help_modal(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(76, 85, area);
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let mut help_text = Vec::new();
    // Render truecolor half-block ocean wave emblem
    for mut line in crate::logo::wave_compact_halfblock_art() {
        line.alignment = Some(Alignment::Center);
        help_text.push(line);
    }
    help_text.push(Line::raw(""));
    help_text.push(Line::styled(
        "TIDE — Terminal Integrated Development Environment",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    ).alignment(Alignment::Center));
    help_text.push(Line::styled(
        "Parallel Coding-Agent Orchestrator & Native Terminal Workspace",
        Style::default().fg(theme.text_secondary),
    ).alignment(Alignment::Center));
    help_text.push(Line::raw(""));
    help_text.push(Line::styled("Keyboard & Mouse Shortcuts", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
    help_text.push(Line::raw(""));

    let shortcuts = vec![
        Line::from(vec![
            Span::styled("Mouse Click     ", Style::default().fg(theme.warning)),
            Span::styled("Click any pane to focus, click tabs to switch, click [×] to close", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("F6 / <Tab>      ", Style::default().fg(theme.warning)),
            Span::styled("Cycle active pane (F6 works everywhere, including inside terminal)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("F2 / F3 / F4 / F5", Style::default().fg(theme.warning)),
            Span::styled("Toggle panels on/off: F2 (Explorer), F5 (Preview), F3 (Git), F4 (Agents)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("F11             ", Style::default().fg(theme.warning)),
            Span::styled("Maximize / restore active pane to full screen", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("F12 / Shift+F12 ", Style::default().fg(theme.warning)),
            Span::styled("Color Themes: [F12] live preview modal, [Shift+F12] cycle theme instantly", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Enter / Space   ", Style::default().fg(theme.warning)),
            Span::styled("Explorer: expand / collapse folder; Enter on file opens editor", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("h/l / Left/Right", Style::default().fg(theme.warning)),
            Span::styled("Explorer: expand folder (Right/l) / collapse folder or go to parent (Left/h)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("a / r / d       ", Style::default().fg(theme.warning)),
            Span::styled("Explorer: [a] New File/Folder, [r] Rename, [d] Delete with confirmation", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Mouse Drag      ", Style::default().fg(theme.warning)),
            Span::styled("Drag pane borders to dynamically resize cards (columns & rows)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Alt+[ / Alt+]   ", Style::default().fg(theme.warning)),
            Span::styled("Resize Explorer column width", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Alt+- / Alt+=   ", Style::default().fg(theme.warning)),
            Span::styled("Resize File Preview vs Terminal split height", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Alt+{ / Alt+}   ", Style::default().fg(theme.warning)),
            Span::styled("Resize Right panel (Git & Agents) column width", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("<Ctrl-b>        ", Style::default().fg(theme.warning)),
            Span::styled("Escape child terminal focus and return to TIDE controls", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Type / Ctrl+s   ", Style::default().fg(theme.warning)),
            Span::styled("Code Preview: type directly to edit buffer, [Ctrl+s] save to disk", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+z / Ctrl+y ", Style::default().fg(theme.warning)),
            Span::styled("Code Editor: [Ctrl+z] Undo, [Ctrl+y / Ctrl+Shift+Z] Redo edits", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+d / Ctrl+k ", Style::default().fg(theme.warning)),
            Span::styled("Code Editor: [Ctrl+d] Duplicate line, [Ctrl+k] Delete line", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Alt+Up / Alt+Dn ", Style::default().fg(theme.warning)),
            Span::styled("Code Editor: [Alt+↑ / Alt+↓] Move current line up or down", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+/          ", Style::default().fg(theme.warning)),
            Span::styled("Code Editor: [Ctrl+/] Toggle line comment (auto-detects language)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+f          ", Style::default().fg(theme.warning)),
            Span::styled("Find in File (Enter/Shift+Enter for next/prev match, Esc to exit)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+Shift+F    ", Style::default().fg(theme.warning)),
            Span::styled("Workspace Search (recursive grep across all project files)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+Tab / F9/10", Style::default().fg(theme.warning)),
            Span::styled("Editor Tabs: cycle tabs, [Ctrl+w] close active editor tab", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+e / Ctrl+o ", Style::default().fg(theme.warning)),
            Span::styled("Open current file in VS Code", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("<Ctrl-p>        ", Style::default().fg(theme.warning)),
            Span::styled("Command Palette & Fuzzy File Finder", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("<Ctrl-t>        ", Style::default().fg(theme.warning)),
            Span::styled("New Terminal (Command Prompt, PowerShell, Git Bash, WSL)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("F7 / F8         ", Style::default().fg(theme.warning)),
            Span::styled("Navigate previous / next Terminal tab (works in any pane)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Alt+1 .. Alt+9  ", Style::default().fg(theme.warning)),
            Span::styled("Switch directly to Terminal tab 1..9", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Wheel / Shift+Pg", Style::default().fg(theme.warning)),
            Span::styled("Terminal: mouse wheel scrollback & pass-through to pagers (less/nano/vi)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Shift+End       ", Style::default().fg(theme.warning)),
            Span::styled("Terminal: reset scrollback to live interactive prompt", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+= / Ctrl+- ", Style::default().fg(theme.warning)),
            Span::styled("Terminal Zoom: [Ctrl+=] zoom in, [Ctrl+-] zoom out, [Ctrl+0] reset 100%", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("s / u / S / U   ", Style::default().fg(theme.warning)),
            Span::styled("Git Pane: [s] stage file, [u] unstage, [S] stage all, [U] unstage all", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("c / p / P       ", Style::default().fg(theme.warning)),
            Span::styled("Git Pane: [c] Commit modal, [p] git push, [P] git pull", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("b / z           ", Style::default().fg(theme.warning)),
            Span::styled("Git Pane: [b] Branch switcher & new branch modal, [z] Stash manager modal", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("<Ctrl-a> / a    ", Style::default().fg(theme.warning)),
            Span::styled("Launch coding agent in isolated Git worktree", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("s / d / m / x   ", Style::default().fg(theme.warning)),
            Span::styled("Agents pane: [s] Stop Session, [d] Diff, [m] Merge, [x] Discard", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("<Ctrl-q>        ", Style::default().fg(theme.danger)),
            Span::styled("Quit TIDE (Cleanly stops all Job Objects and child processes)", Style::default().fg(theme.text_primary)),
        ]),
        Line::raw(""),
        Line::styled("Press Esc or F1 to close this help window", Style::default().fg(theme.text_muted)),
    ];
    help_text.extend(shortcuts);

    let help_icon = crate::icons::pane_icon("wave", app.icon_theme);
    let block = Block::default()
        .title(format!(" {}Help (F1) [×] ", help_icon))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface));

    let p = Paragraph::new(help_text).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, popup_area);
}

fn draw_command_palette_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(65, 60, area);
    app.palette_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let pal_icon = crate::icons::action_icon("palette", app.icon_theme);
    let srch_icon = crate::icons::action_icon("search", app.icon_theme);

    let outer_block = Block::default()
        .title(format!(" {}Command Palette & Fuzzy Finder (<Ctrl-p>) [×] ", pal_icon))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface));

    let inner = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Input query
            Constraint::Min(0),     // Results list
            Constraint::Length(1),  // Footer shortcuts
        ])
        .split(inner);

    // Search query box
    let query_line = if app.palette_query.is_empty() {
        Line::from(vec![
            Span::styled(format!(" {} ", srch_icon.trim()), Style::default().fg(theme.warning)),
            Span::styled("Type file name to open, or '>' for IDE commands... ", Style::default().fg(theme.text_muted)),
            Span::styled("█", Style::default().fg(theme.accent)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!(" {} ", srch_icon.trim()), Style::default().fg(theme.warning)),
            Span::styled(&app.palette_query, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(theme.accent)),
        ])
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface_variant));
    f.render_widget(Paragraph::new(query_line).block(input_block), chunks[0]);

    // Results list
    let items = app.get_palette_items();
    let max_visible = chunks[1].height as usize;
    let selected_idx = app.palette_selected_idx.min(items.len().saturating_sub(1));

    let mut list_lines = Vec::new();
    if items.is_empty() {
        list_lines.push(Line::raw(""));
        list_lines.push(Line::styled("  No matching files or commands found.", Style::default().fg(theme.text_secondary)));
    } else {
        let start_idx = if selected_idx >= max_visible {
            selected_idx + 1 - max_visible
        } else {
            0
        };

        for (i, item) in items.iter().enumerate().skip(start_idx).take(max_visible) {
            let is_sel = i == selected_idx;
            if is_sel {
                let spans = vec![
                    Span::styled(" ▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(&item.icon, Style::default().fg(theme.warning)),
                    Span::styled(format!(" {} ", item.title), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("— {} ", item.subtitle), Style::default().fg(theme.text_secondary)),
                ];
                list_lines.push(Line::from(spans).style(Style::default().bg(theme.selection_bg)));
            } else {
                let spans = vec![
                    Span::raw("   "),
                    Span::styled(&item.icon, Style::default().fg(theme.text_secondary)),
                    Span::styled(format!(" {} ", item.title), Style::default().fg(theme.text_primary)),
                    Span::styled(format!("— {} ", item.subtitle), Style::default().fg(theme.text_muted)),
                ];
                list_lines.push(Line::from(spans));
            }
        }
    }
    f.render_widget(Paragraph::new(list_lines), chunks[1]);

    // Footer
    let footer_spans = vec![
        Span::styled("[↑/↓] ", Style::default().fg(theme.warning)),
        Span::raw("Navigate  "),
        Span::styled("[Enter] ", Style::default().fg(theme.warning)),
        Span::raw("Select  "),
        Span::styled("[Esc] ", Style::default().fg(theme.warning)),
        Span::raw("Close  "),
        Span::styled("Prefix '>' ", Style::default().fg(theme.accent)),
        Span::raw("for commands"),
    ];
    f.render_widget(Paragraph::new(Line::from(footer_spans)), chunks[2]);
}

fn draw_terminal_menu_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(64, 52, area);
    app.terminal_menu_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let term_icon = crate::icons::action_icon("terminal", app.icon_theme);
    let outer_block = Block::default()
        .title(format!(" {}New Terminal Session [×] ", term_icon))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface));

    let inner = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Header
            Constraint::Min(0),     // Options
            Constraint::Length(2),  // Confirm & Cancel button
        ])
        .split(inner);

    // Header
    let header_lines = vec![
        Line::styled("Select Terminal Shell Environment", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Line::styled("Press 1-4, use ↑/↓ and Enter, or click to spawn:", Style::default().fg(theme.text_secondary)),
    ];
    f.render_widget(Paragraph::new(header_lines), chunks[0]);

    // Terminal options: cmd.exe, powershell.exe, bash.exe, wsl.exe
    let cmd_ic = crate::icons::shell_icon("cmd", app.icon_theme);
    let ps_ic = crate::icons::shell_icon("powershell", app.icon_theme);
    let bash_ic = crate::icons::shell_icon("bash", app.icon_theme);
    let wsl_ic = crate::icons::shell_icon("wsl", app.icon_theme);

    #[cfg(windows)]
    let options = [
        ("1", format!("{}Command Prompt (cmd.exe)", cmd_ic), "Native Windows Command Line shell", "[ Windows CMD ]", theme.accent),
        ("2", format!("{}PowerShell (powershell.exe)", ps_ic), "Windows PowerShell scripting shell", "[ PowerShell ]", Color::Rgb(59, 130, 246)),
        ("3", format!("{}Git Bash (bash.exe)", bash_ic), "Git Unix Bash shell environment", "[ Git Bash ]", Color::Rgb(249, 115, 22)),
        ("4", format!("{}WSL / Linux (wsl.exe)", wsl_ic), "Windows Subsystem for Linux session", "[ WSL Linux ]", theme.success),
    ];
    #[cfg(not(windows))]
    let options = [
        ("1", format!("{}Bourne Again Shell (bash)", bash_ic), "Standard Linux/Unix Bash shell", "[ Bash ]", theme.accent),
        ("2", format!("{}POSIX Shell (sh)", bash_ic), "Lightweight POSIX system shell", "[ Sh ]", Color::Rgb(59, 130, 246)),
        ("3", format!("{}Z Shell (zsh)", bash_ic), "Z Shell interactive command environment", "[ Zsh ]", Color::Rgb(249, 115, 22)),
        ("4", format!("{}Bash Login (bash --login)", bash_ic), "Login shell with profile exports", "[ Login Shell ]", theme.success),
    ];

    let mut option_lines = Vec::new();
    for (i, (num, title, desc, tag, tag_color)) in options.iter().enumerate() {
        let is_selected = app.terminal_menu_selected_idx == i;
        if is_selected {
            let row1_spans = vec![
                Span::styled(" ▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" [{}] ", num), Style::default().bg(theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", title), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", tag), Style::default().fg(*tag_color).add_modifier(Modifier::BOLD)),
                Span::styled(" [ SELECTED ]", Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD)),
            ];
            let row2_spans = vec![
                Span::raw("     ↳ "),
                Span::styled(*desc, Style::default().fg(theme.text_primary).add_modifier(Modifier::ITALIC)),
            ];
            option_lines.push(Line::from(row1_spans).style(Style::default().bg(theme.selection_bg)));
            option_lines.push(Line::from(row2_spans).style(Style::default().bg(theme.selection_bg)));
        } else {
            let row1_spans = vec![
                Span::raw("   "),
                Span::styled(format!(" {}. ", num), Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", title), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", tag), Style::default().fg(theme.text_muted)),
            ];
            let row2_spans = vec![
                Span::raw("     ↳ "),
                Span::styled(*desc, Style::default().fg(theme.text_secondary)),
            ];
            option_lines.push(Line::from(row1_spans));
            option_lines.push(Line::from(row2_spans));
        }
        option_lines.push(Line::raw(""));
    }
    f.render_widget(Paragraph::new(option_lines), chunks[1]);

    // Footer with Confirm & Cancel
    let footer_spans = vec![
        Span::styled(" [ ✔ Confirm & Launch (Enter) ] ", Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled(" [ ✕ Cancel (Esc) ] ", Style::default().bg(Color::Rgb(71, 85, 105)).fg(Color::White)),
    ];
    f.render_widget(Paragraph::new(Line::from(footer_spans)), chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn convert_vt100_color(color: vt100::Color) -> Option<Color> {
    match color {
        vt100::Color::Default => None,
        vt100::Color::Idx(i) => Some(Color::Indexed(i)),
        vt100::Color::Rgb(r, g, b) => Some(Color::Rgb(r, g, b)),
    }
}

fn draw_agent_launch_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(68, 62, area);
    app.agent_launch_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let agt_icon = crate::icons::action_icon("agent", app.icon_theme);
    let outer_block = Block::default()
        .title(format!(" {}Launch Agent Session [×] ", agt_icon))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface));

    let inner = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Title and subtitle
            Constraint::Min(0),     // Agent options cards
            Constraint::Length(2),  // Worktree & teardown info
            Constraint::Length(2),  // Confirm & Cancel buttons
        ])
        .split(inner);

    // Header
    let header_lines = vec![
        Line::styled("Launch New Coding-Agent in Isolated Git Worktree", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Line::styled("Select an agent below and confirm to start an isolated session:", Style::default().fg(theme.text_secondary)),
    ];
    f.render_widget(Paragraph::new(header_lines), chunks[0]);

    // Agent options
    let agents_data = [
        (
            "1",
            "Antigravity CLI (agy)",
            "Google Deepmind Advanced Coding Agent",
            "[ Autonomous / DeepMind ]",
            theme.accent,
        ),
        (
            "2",
            "Claude Code CLI (claude)",
            "Anthropic Autonomous Agent CLI",
            "[ CLI Agent / Anthropic ]",
            Color::Rgb(249, 115, 22),
        ),
        (
            "3",
            "Isolated Worktree Shell (powershell)",
            "Custom manual worktree session",
            "[ Isolated Worktree ]",
            theme.accent_secondary,
        ),
    ];

    let mut option_lines = Vec::new();
    for (i, (num, title, desc, tag, tag_color)) in agents_data.iter().enumerate() {
        let is_selected = app.agent_launch_selected_idx == i;

        if is_selected {
            // Selected / Panned-to styling: rich background highlight, accent pointer, bold text
            let row1_spans = vec![
                Span::styled(" ▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" [{}] ", num), Style::default().bg(theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", title), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", tag), Style::default().fg(*tag_color).add_modifier(Modifier::BOLD)),
                Span::styled(" [ SELECTED ]", Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD)),
            ];
            let row2_spans = vec![
                Span::raw("     ↳ "),
                Span::styled(*desc, Style::default().fg(theme.text_primary).add_modifier(Modifier::ITALIC)),
            ];

            option_lines.push(Line::from(row1_spans).style(Style::default().bg(theme.selection_bg)));
            option_lines.push(Line::from(row2_spans).style(Style::default().bg(theme.selection_bg)));
        } else {
            // Unselected styling
            let row1_spans = vec![
                Span::raw("   "),
                Span::styled(format!(" {}. ", num), Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", title), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", tag), Style::default().fg(theme.text_muted)),
            ];
            let row2_spans = vec![
                Span::raw("     ↳ "),
                Span::styled(*desc, Style::default().fg(theme.text_secondary)),
            ];

            option_lines.push(Line::from(row1_spans));
            option_lines.push(Line::from(row2_spans));
        }

        option_lines.push(Line::raw(""));
    }

    f.render_widget(Paragraph::new(option_lines), chunks[1]);

    // System info
    let info_lines = vec![
        Line::styled("• Dedicated branch & worktree (.tide/worktrees/<id>) created automatically.", Style::default().fg(theme.text_muted)),
        Line::styled("• All processes bound to Windows Job Object for clean teardown.", Style::default().fg(theme.text_muted)),
    ];
    f.render_widget(Paragraph::new(info_lines), chunks[2]);

    // Action Buttons: [ ✔ Confirm & Launch ] and [ ✖ Cancel ]
    let selected_agent_name = match app.agent_launch_selected_idx {
        0 => "Antigravity",
        1 => "Claude Code",
        _ => "Worktree Shell",
    };

    let button_spans = vec![
        Span::styled(
            format!("  [ ✔ Launch {} (Enter) ]  ", selected_agent_name),
            Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "  [ ✖ Cancel (Esc) ]  ",
            Style::default().bg(Color::Rgb(51, 65, 85)).fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "↑/↓ or j/k: Pan/Highlight │ 1-3: Quick Pick",
            Style::default().fg(theme.text_secondary),
        ),
    ];
    f.render_widget(Paragraph::new(Line::from(button_spans)).alignment(Alignment::Center), chunks[3]);
}

fn draw_theme_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(72, 84, area);
    app.theme_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let thm_icon = crate::icons::action_icon("theme", app.icon_theme);

    let outer_block = Block::default()
        .title(format!(" {}Select Color Theme (<F12>) [×] ", thm_icon))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface));

    let inner = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title and live preview subtitle
            Constraint::Min(0),    // Theme options cards
            Constraint::Length(2), // Confirm / Cancel buttons
        ])
        .split(inner);

    // 1. Header
    let header_lines = vec![
        Line::styled("Instant Live Preview: Color Schemes", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Line::styled("Navigate with ↑/↓ or j/k to preview theme in real-time. Press [Enter] to save:", Style::default().fg(theme.text_secondary)),
    ];
    f.render_widget(Paragraph::new(header_lines), chunks[0]);

    // 2. Theme cards
    let themes = crate::theme::ColorTheme::ALL;
    let available_lines = chunks[1].height as usize;
    let cards_fit = (available_lines / 2).max(1);
    let start_idx = if app.theme_modal_selected_idx >= cards_fit {
        app.theme_modal_selected_idx + 1 - cards_fit
    } else {
        0
    };

    let mut theme_lines = Vec::new();

    for (idx, color_theme) in themes.iter().enumerate().skip(start_idx).take(cards_fit) {
        let is_selected = app.theme_modal_selected_idx == idx;
        let is_current = app.color_theme == *color_theme;
        let (sw_bg, sw_acc, sw_sec, sw_suc) = color_theme.swatches();

        let num_str = if idx < 9 {
            format!(" [{}] ", idx + 1)
        } else {
            " [0] ".to_string()
        };

        if is_selected {
            let row1_spans = vec![
                Span::styled(" ▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(num_str, Style::default().bg(theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", color_theme.name()), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(" [", Style::default().fg(theme.text_muted)),
                Span::styled("● ", Style::default().fg(sw_bg)),
                Span::styled("● ", Style::default().fg(sw_acc)),
                Span::styled("● ", Style::default().fg(sw_sec)),
                Span::styled("●", Style::default().fg(sw_suc)),
                Span::styled("] ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    if is_current { " [ ACTIVE PREVIEW ] " } else { " [ PREVIEWING ] " },
                    Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD),
                ),
            ];

            let row2_spans = vec![
                Span::raw("     ↳ "),
                Span::styled(color_theme.description(), Style::default().fg(theme.text_primary).add_modifier(Modifier::ITALIC)),
            ];

            theme_lines.push(Line::from(row1_spans).style(Style::default().bg(theme.selection_bg)));
            theme_lines.push(Line::from(row2_spans).style(Style::default().bg(theme.selection_bg)));
        } else {
            let row1_spans = vec![
                Span::raw("   "),
                Span::styled(num_str, Style::default().fg(theme.text_secondary)),
                Span::styled(format!(" {} ", color_theme.name()), Style::default().fg(theme.text_primary)),
                Span::styled(" [", Style::default().fg(theme.text_muted)),
                Span::styled("● ", Style::default().fg(sw_bg)),
                Span::styled("● ", Style::default().fg(sw_acc)),
                Span::styled("● ", Style::default().fg(sw_sec)),
                Span::styled("●", Style::default().fg(sw_suc)),
                Span::styled("]", Style::default().fg(theme.text_muted)),
            ];

            let row2_spans = vec![
                Span::raw("     ↳ "),
                Span::styled(color_theme.description(), Style::default().fg(theme.text_muted)),
            ];

            theme_lines.push(Line::from(row1_spans));
            theme_lines.push(Line::from(row2_spans));
        }
    }

    f.render_widget(Paragraph::new(theme_lines), chunks[1]);

    // 3. Action buttons
    let selected_theme_name = themes.get(app.theme_modal_selected_idx).map(|t| t.name()).unwrap_or("Theme");
    let button_spans = vec![
        Span::styled(
            format!("  [ ✔ Save {} (Enter) ]  ", selected_theme_name),
            Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "  [ ✖ Cancel & Revert (Esc) ]  ",
            Style::default().bg(Color::Rgb(51, 65, 85)).fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "↑/↓ or j/k: Live Preview │ 1-9, 0: Quick Pick",
            Style::default().fg(theme.text_secondary),
        ),
    ];
    f.render_widget(Paragraph::new(Line::from(button_spans)).alignment(Alignment::Center), chunks[2]);
}

fn draw_commit_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(65, 48, area);
    app.commit_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let cmt_icon = crate::icons::action_icon("commit", app.icon_theme);
    let ren_icon = crate::icons::action_icon("rename", app.icon_theme);

    let outer_block = Block::default()
        .title(format!(" {}Git Commit Changes [×] ", cmt_icon))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.success))
        .style(Style::default().bg(theme.surface));

    let inner = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header summary
            Constraint::Min(0),    // Staged files preview
            Constraint::Length(3), // Commit message input box
            Constraint::Length(2), // Confirm / Cancel buttons
        ])
        .split(inner);

    let staged_count = app.git_info.staged_files.len();

    // 1. Header summary
    let summary_lines = if staged_count == 0 {
        vec![
            Line::styled("Record Staged Changes to Git History", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Line::styled("Warning: No staged files! Press [Esc], select files and press [s] or [a] to stage before committing.", Style::default().fg(theme.warning)),
        ]
    } else {
        vec![
            Line::styled(format!("Commit {} Staged File{} to '{}'", staged_count, if staged_count == 1 { "" } else { "s" }, app.git_info.branch), Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Line::styled("Enter a descriptive commit message below and press [Enter] to commit:", Style::default().fg(theme.text_secondary)),
        ]
    };
    f.render_widget(Paragraph::new(summary_lines), chunks[0]);

    // 2. Staged files preview
    let mut file_lines = Vec::new();
    if staged_count > 0 {
        for item in app.git_info.staged_files.iter().take(5) {
            let (badge_text, badge_color) = item.status.badge();
            file_lines.push(Line::from(vec![
                Span::styled("   ✓ ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", badge_text), Style::default().fg(badge_color).add_modifier(Modifier::BOLD)),
                Span::styled(&item.path, Style::default().fg(Color::White)),
            ]));
        }
        if staged_count > 5 {
            file_lines.push(Line::styled(format!("   ... and {} more staged file(s)", staged_count - 5), Style::default().fg(theme.text_muted)));
        }
    } else {
        file_lines.push(Line::raw(""));
        file_lines.push(Line::styled("   (No staged changes to commit)", Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC)));
    }
    f.render_widget(Paragraph::new(file_lines), chunks[1]);

    // 3. Commit Message Input Box
    let input_line = if app.commit_message_input.is_empty() {
        Line::from(vec![
            Span::styled(format!(" {} ", ren_icon.trim()), Style::default().fg(theme.warning)),
            Span::styled("Type commit message (e.g. feat: add git pane)...", Style::default().fg(theme.text_muted)),
            Span::styled("█", Style::default().fg(theme.success)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!(" {} ", ren_icon.trim()), Style::default().fg(theme.warning)),
            Span::styled(&app.commit_message_input, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(theme.success)),
        ])
    };

    let input_block = Block::default()
        .title(" Commit Message ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.success))
        .style(Style::default().bg(theme.surface_variant));
    f.render_widget(Paragraph::new(input_line).block(input_block), chunks[2]);

    // 4. Action Buttons
    let button_spans = vec![
        Span::styled(
            "  [ ✔ Commit (Enter) ]  ",
            if staged_count > 0 && !app.commit_message_input.trim().is_empty() {
                Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default().bg(theme.surface_variant).fg(theme.text_muted)
            },
        ),
        Span::raw("   "),
        Span::styled(
            "  [ ✖ Cancel (Esc) ]  ",
            Style::default().bg(theme.surface_variant).fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ];
    f.render_widget(Paragraph::new(Line::from(button_spans)).alignment(Alignment::Center), chunks[3]);
}

fn draw_diff_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(80, 75, area);
    app.diff_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let (title, content) = match &app.diff_modal_content {
        Some((t, c)) => (t.as_str(), c.as_str()),
        None => ("Diff", "No diff available"),
    };

    let is_agent_diff = title.starts_with("Diff:");

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(popup_area);

    let diff_icon = crate::icons::action_icon("diff", app.icon_theme);
    let block = Block::default()
        .title(format!(" {}{} [×] ", diff_icon, title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if is_agent_diff { theme.accent_secondary } else { theme.border_active }))
        .style(Style::default().bg(theme.surface));

    let inner = block.inner(chunks[0]);
    f.render_widget(block, chunks[0]);

    let mut diff_lines = Vec::new();
    for line in content.lines() {
        let style = if line.starts_with('+') && !line.starts_with("+++") {
            Style::default().fg(theme.success)
        } else if line.starts_with('-') && !line.starts_with("---") {
            Style::default().fg(theme.danger)
        } else if line.starts_with("@@") {
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else if line.starts_with("diff --git") {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_primary)
        };
        diff_lines.push(Line::styled(line.to_string(), style));
    }

    let total_lines = diff_lines.len();
    let view_height = inner.height as usize;
    let max_scroll = total_lines.saturating_sub(view_height);
    app.diff_modal_scroll = app.diff_modal_scroll.min(max_scroll);

    let scrollbar_needed = total_lines > view_height && view_height > 0;
    let (content_area, scrollbar_area) = if scrollbar_needed {
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(inner);
        (h_chunks[0], h_chunks[1])
    } else {
        (inner, Rect::default())
    };

    let visible_lines: Vec<Line> = diff_lines.into_iter().skip(app.diff_modal_scroll).take(view_height).collect();
    let p = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
    f.render_widget(p, content_area);

    if scrollbar_needed {
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(app.diff_modal_scroll);
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.accent));
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    let action_spans = if is_agent_diff {
        vec![
            Span::styled(" [A] ", Style::default().bg(theme.success).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" Accept & Merge to Base   ", Style::default().fg(theme.text_primary)),
            Span::styled(" [D] ", Style::default().bg(theme.danger).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" Discard Worktree & Branch   ", Style::default().fg(theme.text_primary)),
            Span::styled(" [Esc] ", Style::default().bg(theme.surface_variant).fg(Color::White)),
            Span::styled(" Close   ", Style::default().fg(theme.text_muted)),
            Span::styled("↑/↓ or j/k / Wheel: Scroll", Style::default().fg(theme.text_secondary)),
        ]
    } else {
        vec![
            Span::styled(" [s] ", Style::default().bg(theme.warning).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" Toggle Stage/Unstage   ", Style::default().fg(theme.text_primary)),
            Span::styled(" [Esc] ", Style::default().bg(theme.surface_variant).fg(Color::White)),
            Span::styled(" Close Diff   ", Style::default().fg(theme.text_muted)),
            Span::styled("↑/↓ or j/k / Wheel: Scroll", Style::default().fg(theme.text_secondary)),
        ]
    };
    f.render_widget(Paragraph::new(Line::from(action_spans)).alignment(Alignment::Center), chunks[1]);
}

fn draw_branch_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(68, 60, area);
    app.branch_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let br_icon = crate::icons::action_icon("branch", app.icon_theme);
    let outer_title = if app.branch_is_creating {
        format!(" {}Create New Git Branch [×] ", br_icon)
    } else {
        format!(" {}Git Branch Manager [×] ", br_icon)
    };

    let outer_block = Block::default()
        .title(outer_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent_secondary))
        .style(Style::default().bg(theme.surface));

    let inner = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);

    if app.branch_is_creating {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Prompt info
                Constraint::Length(3), // Input box
                Constraint::Min(0),    // Note / current branch
                Constraint::Length(2), // Confirm / Cancel buttons
            ])
            .split(inner);

        let info_lines = vec![
            Line::styled("Create and checkout a new branch (git checkout -b <name>):", Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
            Line::styled(format!("Branch will be created from current HEAD ('{}')", app.git_info.branch), Style::default().fg(theme.text_muted)),
        ];
        f.render_widget(Paragraph::new(info_lines), chunks[0]);

        let input_line = if app.branch_new_name.is_empty() {
            Line::from(vec![
                Span::styled(format!(" {} ", br_icon.trim()), Style::default().fg(theme.accent_secondary)),
                Span::styled("feature/your-branch-name... ", Style::default().fg(theme.text_muted)),
                Span::styled("█", Style::default().fg(theme.accent)),
            ])
        } else {
            Line::from(vec![
                Span::styled(format!(" {} ", br_icon.trim()), Style::default().fg(theme.accent_secondary)),
                Span::styled(&app.branch_new_name, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(theme.accent)),
            ])
        };

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.accent))
            .style(Style::default().bg(theme.surface_variant));
        f.render_widget(Paragraph::new(input_line).block(input_block), chunks[1]);

        let hint = Paragraph::new(vec![
            Line::raw(""),
            Line::styled("Tip: Use letters, numbers, dashes, and slashes (e.g. feat/branch-modal).", Style::default().fg(theme.text_secondary)),
            Line::styled("Press [Enter] to create branch and switch to it immediately.", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(hint, chunks[2]);

        let btn_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[3]);

        let confirm_btn = Paragraph::new(Line::from(vec![
            Span::styled("  [Enter] Create & Switch  ", Style::default().fg(Color::White).bg(theme.success).add_modifier(Modifier::BOLD)),
        ]));
        let cancel_btn = Paragraph::new(Line::from(vec![
            Span::styled("  [Esc] Cancel  ", Style::default().fg(theme.text_primary).bg(theme.selection_bg)),
        ]));

        f.render_widget(confirm_btn, btn_chunks[0]);
        f.render_widget(cancel_btn, btn_chunks[1]);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search/Filter bar
                Constraint::Min(0),    // Branch items list
                Constraint::Length(1), // Footer shortcuts
            ])
            .split(inner);

        // 1. Filter input box
        let query_line = if app.branch_filter_query.is_empty() {
            Line::from(vec![
                Span::styled(format!(" {} ", br_icon.trim()), Style::default().fg(theme.accent_secondary)),
                Span::styled("Type to filter branches... ([n] creates new branch) ", Style::default().fg(theme.text_muted)),
                Span::styled("█", Style::default().fg(theme.accent)),
            ])
        } else {
            Line::from(vec![
                Span::styled(format!(" {} ", br_icon.trim()), Style::default().fg(theme.accent_secondary)),
                Span::styled(&app.branch_filter_query, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(theme.accent)),
            ])
        };

        let filter_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border_active))
            .style(Style::default().bg(theme.surface_variant));
        f.render_widget(Paragraph::new(query_line).block(filter_block), chunks[0]);

        // 2. Branch List
        let filtered = app.get_filtered_branches();
        let max_visible = chunks[1].height as usize;
        let selected_idx = app.branch_selected_idx.min(filtered.len().saturating_sub(1));

        let mut list_lines = Vec::new();
        if filtered.is_empty() {
            list_lines.push(Line::raw(""));
            list_lines.push(Line::styled("  No matching branches found. Press [n] to create a new branch.", Style::default().fg(theme.text_secondary)));
        } else {
            let start_idx = if selected_idx >= max_visible {
                selected_idx + 1 - max_visible
            } else {
                0
            };

            for (i, branch) in filtered.iter().enumerate().skip(start_idx).take(max_visible) {
                let is_sel = i == selected_idx;
                let prefix = if is_sel { " ▶ " } else { "   " };

                let (badge_text, badge_color) = if branch.is_current {
                    ("● Active".to_string(), theme.success)
                } else if branch.is_worktree {
                    (format!("{} Worktree", crate::icons::pane_icon("agents", app.icon_theme).trim()), theme.warning)
                } else if branch.is_remote {
                    (format!("{} Remote", crate::icons::action_icon("push", app.icon_theme).trim()), theme.accent_secondary)
                } else {
                    (format!("{} Local", br_icon.trim()), theme.accent)
                };

                let mut row_spans = vec![
                    Span::styled(prefix, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<13} ", badge_text), Style::default().fg(badge_color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<22} ", branch.display_name()), Style::default().fg(if is_sel { Color::White } else { theme.text_primary }).add_modifier(Modifier::BOLD)),
                ];

                if !branch.commit_hash.is_empty() {
                    row_spans.push(Span::styled(format!("{} ", branch.commit_hash), Style::default().fg(theme.text_muted)));
                }
                if !branch.commit_msg.is_empty() {
                    let max_msg_w = chunks[1].width.saturating_sub(44) as usize;
                    let msg_disp = if branch.commit_msg.chars().count() > max_msg_w && max_msg_w > 3 {
                        format!("{}...", branch.commit_msg.chars().take(max_msg_w - 3).collect::<String>())
                    } else {
                        branch.commit_msg.clone()
                    };
                    row_spans.push(Span::styled(format!("— {}", msg_disp), Style::default().fg(theme.text_secondary)));
                }

                let line = Line::from(row_spans);
                if is_sel {
                    list_lines.push(line.style(Style::default().bg(theme.selection_bg)));
                } else {
                    list_lines.push(line);
                }
            }
        }
        f.render_widget(Paragraph::new(list_lines), chunks[1]);

        // 3. Footer
        let footer_spans = vec![
            Span::styled("[Enter] ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled("Checkout  ", Style::default().fg(theme.text_primary)),
            Span::styled("[n] ", Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
            Span::styled("New Branch  ", Style::default().fg(theme.text_primary)),
            Span::styled("[d] ", Style::default().fg(theme.danger).add_modifier(Modifier::BOLD)),
            Span::styled("Delete  ", Style::default().fg(theme.text_primary)),
            Span::styled("[r] ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled("Refresh  ", Style::default().fg(theme.text_primary)),
            Span::styled("[Esc] ", Style::default().fg(theme.text_muted).add_modifier(Modifier::BOLD)),
            Span::styled("Close", Style::default().fg(theme.text_primary)),
        ];
        f.render_widget(Paragraph::new(Line::from(footer_spans)).style(Style::default().bg(theme.surface)), chunks[2]);
    }
}

fn draw_stash_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(68, 56, area);
    app.stash_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let st_icon = crate::icons::action_icon("stash", app.icon_theme);
    let ren_icon = crate::icons::action_icon("rename", app.icon_theme);

    let outer_title = if app.stash_is_creating {
        format!(" {}Save Changes to Git Stash [×] ", st_icon)
    } else {
        format!(" {}Git Stash Manager [×] ", st_icon)
    };

    let outer_block = Block::default()
        .title(outer_title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.accent_secondary))
        .style(Style::default().bg(theme.surface));

    let inner = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);

    if app.stash_is_creating {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header description
                Constraint::Length(3), // Stash message input
                Constraint::Length(2), // Untracked toggle
                Constraint::Min(0),    // Summary
                Constraint::Length(2), // Save / Cancel buttons
            ])
            .split(inner);

        let desc_lines = vec![
            Line::styled("Stash working directory changes (git stash push):", Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
            Line::styled(format!("Current branch: '{}' (staged: {}, unstaged: {})", app.git_info.branch, app.git_info.staged_files.len(), app.git_info.unstaged_files.len()), Style::default().fg(theme.text_muted)),
        ];
        f.render_widget(Paragraph::new(desc_lines), chunks[0]);

        let msg_line = if app.stash_new_message.is_empty() {
            Line::from(vec![
                Span::styled(format!(" {} ", ren_icon.trim()), Style::default().fg(theme.warning)),
                Span::styled("(Optional) Custom stash description... ", Style::default().fg(theme.text_muted)),
                Span::styled("█", Style::default().fg(theme.accent)),
            ])
        } else {
            Line::from(vec![
                Span::styled(format!(" {} ", ren_icon.trim()), Style::default().fg(theme.warning)),
                Span::styled(&app.stash_new_message, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(theme.accent)),
            ])
        };

        let msg_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border_active))
            .style(Style::default().bg(theme.surface_variant));
        f.render_widget(Paragraph::new(msg_line).block(msg_block), chunks[1]);

        let untracked_line = Line::from(vec![
            Span::styled(" Include untracked files (-u): ", Style::default().fg(theme.text_primary)),
            if app.stash_include_untracked {
                Span::styled("[✓] Yes (included)", Style::default().fg(theme.success).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("[ ] No (tracked only)", Style::default().fg(theme.text_muted))
            },
            Span::styled("  (click or Ctrl+u to toggle)", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(untracked_line), chunks[2]);

        let hint = Paragraph::new(vec![
            Line::raw(""),
            Line::styled("Stashing reverts your working copy to HEAD and saves changes to a stash stack.", Style::default().fg(theme.text_secondary)),
            Line::styled("You can pop or re-apply this stash at any time using the Stash Manager.", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(hint, chunks[3]);

        let btn_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[4]);

        let save_btn = Paragraph::new(Line::from(vec![
            Span::styled("  [Enter] Save Stash  ", Style::default().fg(Color::White).bg(theme.success).add_modifier(Modifier::BOLD)),
        ]));
        let cancel_btn = Paragraph::new(Line::from(vec![
            Span::styled("  [Esc] Cancel  ", Style::default().fg(theme.text_primary).bg(theme.selection_bg)),
        ]));

        f.render_widget(save_btn, btn_chunks[0]);
        f.render_widget(cancel_btn, btn_chunks[1]);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Header summary
                Constraint::Min(0),    // Stash list
                Constraint::Length(1), // Footer actions
            ])
            .split(inner);

        let header_lines = vec![
            Line::from(vec![
                Span::styled("Saved Stashes (", Style::default().fg(theme.text_primary)),
                Span::styled(format!("{}", app.stash_items.len()), Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" total) on branch '{}'", app.git_info.branch), Style::default().fg(theme.text_primary)),
            ]),
            Line::styled("Select a stash entry to pop, apply, view diff, or drop:", Style::default().fg(theme.text_muted)),
        ];
        f.render_widget(Paragraph::new(header_lines), chunks[0]);

        let max_visible = chunks[1].height as usize;
        let selected_idx = app.stash_selected_idx.min(app.stash_items.len().saturating_sub(1));

        let mut list_lines = Vec::new();
        if app.stash_items.is_empty() {
            list_lines.push(Line::raw(""));
            list_lines.push(Line::styled("  No saved git stashes found.", Style::default().fg(theme.text_secondary)));
            list_lines.push(Line::styled("  Press [s] to stash current uncommitted changes.", Style::default().fg(theme.accent_secondary)));
        } else {
            let start_idx = if selected_idx >= max_visible {
                selected_idx + 1 - max_visible
            } else {
                0
            };

            for (i, stash) in app.stash_items.iter().enumerate().skip(start_idx).take(max_visible) {
                let is_sel = i == selected_idx;
                let prefix = if is_sel { " ▶ " } else { "   " };

                let mut row_spans = vec![
                    Span::styled(prefix, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<10} ", stash.selector), Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
                ];
                if !stash.relative_time.is_empty() {
                    row_spans.push(Span::styled(format!("({:<14}) ", stash.relative_time), Style::default().fg(theme.text_muted)));
                }
                row_spans.push(Span::styled(&stash.message, Style::default().fg(if is_sel { Color::White } else { theme.text_primary })));

                let line = Line::from(row_spans);
                if is_sel {
                    list_lines.push(line.style(Style::default().bg(theme.selection_bg)));
                } else {
                    list_lines.push(line);
                }
            }
        }
        f.render_widget(Paragraph::new(list_lines), chunks[1]);

        // Footer
        let footer_spans = vec![
            Span::styled("[Enter/p] ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
            Span::styled("Pop  ", Style::default().fg(theme.text_primary)),
            Span::styled("[a] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("Apply  ", Style::default().fg(theme.text_primary)),
            Span::styled("[s] ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
            Span::styled("New Stash  ", Style::default().fg(theme.text_primary)),
            Span::styled("[v] ", Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)),
            Span::styled("Diff  ", Style::default().fg(theme.text_primary)),
            Span::styled("[d] ", Style::default().fg(theme.danger).add_modifier(Modifier::BOLD)),
            Span::styled("Drop  ", Style::default().fg(theme.text_primary)),
            Span::styled("[Esc] ", Style::default().fg(theme.text_muted).add_modifier(Modifier::BOLD)),
            Span::styled("Close", Style::default().fg(theme.text_primary)),
        ];
        f.render_widget(Paragraph::new(Line::from(footer_spans)).style(Style::default().bg(theme.surface)), chunks[2]);
    }
}

fn draw_global_search_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(75, 70, area);
    app.global_search_modal_area = popup_area;
    f.render_widget(Clear, popup_area);

    let theme = app.theme();
    let srch_icon = crate::icons::action_icon("search", app.icon_theme);

    let outer_block = Block::default()
        .title(format!(" {}Workspace Search (<Ctrl-Shift-F>) [×] ", srch_icon))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface));

    let inner = outer_block.inner(popup_area);
    f.render_widget(outer_block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input query box
            Constraint::Min(0),    // Results list
            Constraint::Length(1), // Footer shortcuts
        ])
        .split(inner);

    // Search query box
    let count_badge = if app.global_search_query.trim().is_empty() {
        String::new()
    } else {
        format!(" ({} match{})", app.global_search_results.len(), if app.global_search_results.len() == 1 { "" } else { "es" })
    };

    let query_line = if app.global_search_query.is_empty() {
        Line::from(vec![
            Span::styled(format!(" {} ", srch_icon.trim()), Style::default().fg(theme.warning)),
            Span::styled("Type text to search entire workspace...", Style::default().fg(theme.text_muted)),
            Span::styled("█", Style::default().fg(theme.accent)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!(" {} ", srch_icon.trim()), Style::default().fg(theme.warning)),
            Span::styled(&app.global_search_query, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(theme.accent)),
        ])
    };

    let input_block = Block::default()
        .title(format!(" Search in Workspace{} ", count_badge))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface_variant));
    f.render_widget(Paragraph::new(query_line).block(input_block), chunks[0]);

    // Results list
    let max_visible = chunks[1].height as usize;
    let total_results = app.global_search_results.len();

    let mut list_lines = Vec::new();
    if total_results == 0 {
        list_lines.push(Line::raw(""));
        if app.global_search_query.trim().is_empty() {
            list_lines.push(Line::styled(
                "  Type a query above to search files across the workspace.",
                Style::default().fg(theme.text_secondary),
            ));
            list_lines.push(Line::styled(
                "  Automatically searches source files while ignoring .git, target, and node_modules.",
                Style::default().fg(theme.text_muted),
            ));
        } else {
            list_lines.push(Line::styled(
                format!("  No matches found for \"{}\".", app.global_search_query),
                Style::default().fg(theme.danger),
            ));
        }
    } else {
        let selected_idx = app.global_search_selected_idx.min(total_results.saturating_sub(1));
        let start_idx = if selected_idx >= max_visible {
            selected_idx + 1 - max_visible
        } else {
            0
        };

        let q_lower = app.global_search_query.trim().to_lowercase();

        for (i, res) in app.global_search_results.iter().enumerate().skip(start_idx).take(max_visible) {
            let is_sel = i == selected_idx;
            let (f_icon, icon_color) = crate::icons::file_icon(&res.rel_path, app.icon_theme);

            let mut row_spans = Vec::new();
            if is_sel {
                row_spans.push(Span::styled(" ▶ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            } else {
                row_spans.push(Span::raw("   "));
            }

            row_spans.push(Span::styled(format!("{} ", f_icon), Style::default().fg(icon_color)));
            row_spans.push(Span::styled(&res.rel_path, Style::default().fg(if is_sel { Color::White } else { theme.text_primary }).add_modifier(Modifier::BOLD)));
            row_spans.push(Span::styled(format!(":{} ", res.line_num), Style::default().fg(theme.accent)));
            row_spans.push(Span::styled("│ ", Style::default().fg(theme.border_inactive)));

            // Highlight match inside line content snippet
            let content = &res.line_content;
            let content_lower = content.to_lowercase();
            if !q_lower.is_empty() {
                if let Some(pos) = content_lower.find(&q_lower) {
                    let before = &content[..pos];
                    let matched = &content[pos..pos + q_lower.len()];
                    let after = &content[pos + q_lower.len()..];

                    row_spans.push(Span::styled(before.to_string(), Style::default().fg(if is_sel { Color::White } else { theme.text_secondary })));
                    row_spans.push(Span::styled(matched.to_string(), Style::default().bg(theme.warning).fg(Color::Black).add_modifier(Modifier::BOLD)));
                    row_spans.push(Span::styled(after.to_string(), Style::default().fg(if is_sel { Color::White } else { theme.text_secondary })));
                } else {
                    row_spans.push(Span::styled(content.to_string(), Style::default().fg(if is_sel { Color::White } else { theme.text_secondary })));
                }
            } else {
                row_spans.push(Span::styled(content.to_string(), Style::default().fg(if is_sel { Color::White } else { theme.text_secondary })));
            }

            let row_line = if is_sel {
                Line::from(row_spans).style(Style::default().bg(theme.selection_bg))
            } else {
                Line::from(row_spans)
            };

            list_lines.push(row_line);
        }
    }

    f.render_widget(Paragraph::new(list_lines), chunks[1]);

    // Footer
    let footer_spans = vec![
        Span::styled("[↑/↓] ", Style::default().fg(theme.warning)),
        Span::raw("Navigate  "),
        Span::styled("[Enter] ", Style::default().fg(theme.warning)),
        Span::raw("Jump to Match  "),
        Span::styled("[Esc] ", Style::default().fg(theme.warning)),
        Span::raw("Close  "),
        Span::styled("Workspace: ", Style::default().fg(theme.text_muted)),
        Span::styled(&app.workspace_name, Style::default().fg(theme.text_primary)),
    ];
    f.render_widget(Paragraph::new(Line::from(footer_spans)), chunks[2]);

    // Active Cursor positioning in global search input box
    let cursor_x = chunks[0].x + 4 + app.global_search_query.chars().count() as u16;
    if cursor_x < chunks[0].x + chunks[0].width.saturating_sub(1) {
        f.set_cursor_position((cursor_x, chunks[0].y + 1));
    }
}

fn draw_explorer_op_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    match &app.explorer_op_modal {
        crate::app::ExplorerOpModal::None => {}
        crate::app::ExplorerOpModal::NewItem { target_dir, input, is_folder } => {
            let popup_area = centered_rect(58, 38, area);
            app.explorer_op_modal_area = popup_area;
            f.render_widget(Clear, popup_area);

            let new_item_icon = crate::icons::action_icon("new_item", app.icon_theme);
            let folder_icon = crate::icons::action_icon("folder", app.icon_theme);
            let file_icon = crate::icons::action_icon("file", app.icon_theme);

            let block = Block::default()
                .title(format!(" {}New File or Folder (<Tab> Switch) [×] ", new_item_icon))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_active))
                .style(Style::default().bg(theme.surface));

            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Target dir info
                    Constraint::Length(1), // Type toggle badge
                    Constraint::Length(3), // Input box
                    Constraint::Min(0),    // Hint / info
                    Constraint::Length(1), // Footer buttons
                ])
                .split(inner);

            let dir_display = target_dir
                .strip_prefix(&app.workspace_dir)
                .map(|p| p.to_string_lossy())
                .unwrap_or_else(|_| target_dir.to_string_lossy());
            let dir_str = if dir_display.is_empty() { "workspace root" } else { &dir_display };

            let dir_line = Line::from(vec![
                Span::styled(" Location: ", Style::default().fg(theme.text_secondary)),
                Span::styled(dir_str, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]);
            f.render_widget(Paragraph::new(dir_line), chunks[0]);

            let type_line = if *is_folder {
                Line::from(vec![
                    Span::styled(" Type: ", Style::default().fg(theme.text_secondary)),
                    Span::styled(format!(" [{} Folder] ", folder_icon.trim()), Style::default().bg(theme.warning).fg(Color::Black).add_modifier(Modifier::BOLD)),
                    Span::styled("  (Press <Tab> to switch to File)", Style::default().fg(theme.text_muted)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(" Type: ", Style::default().fg(theme.text_secondary)),
                    Span::styled(format!(" [{} File] ", file_icon.trim()), Style::default().bg(theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD)),
                    Span::styled("  (Press <Tab> to switch to Folder)", Style::default().fg(theme.text_muted)),
                ])
            };
            f.render_widget(Paragraph::new(type_line), chunks[1]);

            let input_line = if input.is_empty() {
                Line::from(vec![
                    Span::styled(if *is_folder { "e.g. components/ or utils/" } else { "e.g. index.ts or src/main.rs" }, Style::default().fg(theme.text_muted)),
                    Span::styled("█", Style::default().fg(theme.accent)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(input, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("█", Style::default().fg(theme.accent)),
                ])
            };

            let input_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_active))
                .style(Style::default().bg(theme.surface_variant));
            f.render_widget(Paragraph::new(input_line).block(input_block), chunks[2]);

            // Hint
            let hint_line = Line::styled(
                "Tip: Subdirectories in path (e.g. 'models/auth.rs') are created automatically.",
                Style::default().fg(theme.text_muted),
            );
            f.render_widget(Paragraph::new(hint_line), chunks[3]);

            // Footer
            let footer_spans = vec![
                Span::styled("[Enter] ", Style::default().fg(theme.warning)),
                Span::raw("Create  "),
                Span::styled("[Tab] ", Style::default().fg(theme.warning)),
                Span::raw("Toggle Type  "),
                Span::styled("[Esc] ", Style::default().fg(theme.warning)),
                Span::raw("Cancel"),
            ];
            f.render_widget(Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center), chunks[4]);

            // Cursor position
            let cursor_x = chunks[2].x + 1 + input.chars().count() as u16;
            if cursor_x < chunks[2].x + chunks[2].width.saturating_sub(1) {
                f.set_cursor_position((cursor_x, chunks[2].y + 1));
            }
        }
        crate::app::ExplorerOpModal::Rename { target_path, input, is_dir } => {
            let popup_area = centered_rect(56, 32, area);
            app.explorer_op_modal_area = popup_area;
            f.render_widget(Clear, popup_area);

            let old_name = target_path.file_name().unwrap_or_default().to_string_lossy();
            let kind_label = if *is_dir { "Folder" } else { "File" };

            let rename_icon = crate::icons::action_icon("rename", app.icon_theme);
            let block = Block::default()
                .title(format!(" {}Rename {} [×] ", rename_icon, kind_label))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.warning))
                .style(Style::default().bg(theme.surface));

            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Old name info
                    Constraint::Length(3), // Input box
                    Constraint::Min(0),    // Spacing
                    Constraint::Length(1), // Footer buttons
                ])
                .split(inner);

            let old_line = Line::from(vec![
                Span::styled(" Current name: ", Style::default().fg(theme.text_secondary)),
                Span::styled(old_name.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]);
            f.render_widget(Paragraph::new(old_line), chunks[0]);

            let input_line = Line::from(vec![
                Span::styled(input, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(theme.warning)),
            ]);

            let input_block = Block::default()
                .title(" New Name ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.warning))
                .style(Style::default().bg(theme.surface_variant));
            f.render_widget(Paragraph::new(input_line).block(input_block), chunks[1]);

            let footer_spans = vec![
                Span::styled("[Enter] ", Style::default().fg(theme.warning)),
                Span::raw("Rename  "),
                Span::styled("[Esc] ", Style::default().fg(theme.warning)),
                Span::raw("Cancel"),
            ];
            f.render_widget(Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center), chunks[3]);

            // Cursor position
            let cursor_x = chunks[1].x + 1 + input.chars().count() as u16;
            if cursor_x < chunks[1].x + chunks[1].width.saturating_sub(1) {
                f.set_cursor_position((cursor_x, chunks[1].y + 1));
            }
        }
        crate::app::ExplorerOpModal::DeleteConfirm { target_path: _, is_dir, name } => {
            let popup_area = centered_rect(56, 32, area);
            app.explorer_op_modal_area = popup_area;
            f.render_widget(Clear, popup_area);

            let del_icon = crate::icons::action_icon("delete", app.icon_theme);
            let block = Block::default()
                .title(format!(" {}Delete Permanently? [×] ", del_icon))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.danger))
                .style(Style::default().bg(theme.surface));

            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Prompt
                    Constraint::Length(2), // Item name & icon
                    Constraint::Min(0),    // Warning message
                    Constraint::Length(2), // Action buttons
                ])
                .split(inner);

            f.render_widget(
                Paragraph::new(Line::styled("Are you sure you want to permanently delete:", Style::default().fg(theme.text_primary))),
                chunks[0],
            );

            let (icon, icon_color) = if *is_dir {
                crate::icons::folder_icon(false, app.icon_theme)
            } else {
                crate::icons::file_icon(name, app.icon_theme)
            };

            let item_line = Line::from(vec![
                Span::raw("   "),
                Span::styled(format!("{} ", icon), Style::default().fg(icon_color)),
                Span::styled(name, Style::default().fg(theme.danger).add_modifier(Modifier::BOLD)),
                Span::styled(if *is_dir { " (folder)" } else { " (file)" }, Style::default().fg(theme.text_secondary)),
            ]);
            f.render_widget(Paragraph::new(item_line), chunks[1]);

            let warning_text = if *is_dir {
                "Warning: This folder and all its contents will be permanently deleted."
            } else {
                "Warning: This file will be permanently removed from disk."
            };
            f.render_widget(
                Paragraph::new(Line::styled(warning_text, Style::default().fg(theme.warning))),
                chunks[2],
            );

            let action_spans = vec![
                Span::styled(" [Y / Enter] Delete ", Style::default().bg(theme.danger).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled(" [N / Esc] Cancel ", Style::default().bg(theme.surface_variant).fg(Color::White).add_modifier(Modifier::BOLD)),
            ];
            f.render_widget(Paragraph::new(Line::from(action_spans)).alignment(Alignment::Center), chunks[3]);
        }
    }
}

fn draw_welcome_screen(f: &mut Frame, app: &mut App, area: Rect) {
    app.welcome_area = area;
    let theme = app.theme();
    let w = area.width;
    let h = area.height;

    // Render full-width background
    f.render_widget(Block::default().style(Style::default().bg(theme.bg)), area);

    if w < 50 || h < 14 {
        let msg = Paragraph::new(vec![
            Line::styled("TIDE Workspace Launcher", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Line::styled("Terminal viewport too compact. Please enlarge window or press Esc.", Style::default().fg(theme.text_muted)),
        ]).alignment(Alignment::Center);
        f.render_widget(msg, area);
        return;
    }

    let is_wide = w >= 95;
    let content_area = if is_wide {
        let container_w = (w.saturating_sub(4)).min(130);
        let container_h = (h.saturating_sub(2)).min(46);
        let pad_x = (w.saturating_sub(container_w)) / 2;
        let pad_y = (h.saturating_sub(container_h)) / 2;
        Rect::new(area.x + pad_x, area.y + pad_y, container_w, container_h)
    } else {
        area
    };

    if is_wide {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(48),
                Constraint::Min(40),
            ])
            .split(content_area);

        draw_welcome_hero(f, app, cols[0]);
        draw_welcome_workspaces(f, app, cols[1]);
    } else {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(16),
                Constraint::Min(10),
            ])
            .split(content_area);

        draw_welcome_hero(f, app, rows[0]);
        draw_welcome_workspaces(f, app, rows[1]);
    }
}

fn draw_welcome_hero(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let hero_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_inactive))
        .style(Style::default().bg(theme.surface));

    let inner = hero_block.inner(area);
    f.render_widget(hero_block, area);

    let logo_lines = crate::logo::wave_art_for_height(inner.height.saturating_sub(10));
    let logo_height = logo_lines.len() as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(logo_height),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(inner);

    let logo_para = Paragraph::new(logo_lines).alignment(Alignment::Center);
    f.render_widget(logo_para, chunks[0]);

    let title_para = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" T I D E ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("v0.1.0", Style::default().fg(theme.text_muted)),
        ]),
        Line::from(vec![
            Span::styled("Terminal Integrated Development Environment", Style::default().fg(theme.text_secondary)),
        ]),
    ]).alignment(Alignment::Center);
    f.render_widget(title_para, chunks[1]);

    let mut action_lines = vec![
        Line::raw(""),
        Line::styled("  QUICK START", Style::default().fg(theme.text_muted).add_modifier(Modifier::BOLD)),
    ];

    let action_item = |hotkey: &str, icon: &str, label: &str, desc: &str| {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("[{}] ", hotkey), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", icon), Style::default().fg(theme.text_primary)),
            Span::styled(format!("{:<14}", label), Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {}", desc), Style::default().fg(theme.text_muted)),
        ])
    };

    let folder_icon = crate::icons::pane_icon("explorer", app.icon_theme);
    let git_icon = crate::icons::pane_icon("git", app.icon_theme);
    let new_icon = crate::icons::action_icon("new_item", app.icon_theme);

    action_lines.push(action_item("o", &folder_icon, "Open Folder", "Browse / open path"));
    action_lines.push(action_item("c", &git_icon, "Clone Repo", "Clone git repository"));
    action_lines.push(action_item("n", &new_icon, "New Project", "Create new folder"));
    if !crate::app::App::is_home_or_root_dir(&app.workspace_dir) {
        action_lines.push(action_item("Esc", "✕", "Resume IDE", &format!("Return to {}", app.workspace_name)));
    }

    action_lines.push(Line::raw(""));
    action_lines.push(Line::styled("  GLOBAL KEYS", Style::default().fg(theme.text_muted).add_modifier(Modifier::BOLD)));
    action_lines.push(Line::from(vec![
        Span::styled("  Ctrl+P ", Style::default().fg(theme.accent)),
        Span::styled("Command Palette  ", Style::default().fg(theme.text_secondary)),
        Span::styled("F12 ", Style::default().fg(theme.accent)),
        Span::styled("Themes", Style::default().fg(theme.text_secondary)),
    ]));
    action_lines.push(Line::from(vec![
        Span::styled("  Ctrl+Shift+F ", Style::default().fg(theme.accent)),
        Span::styled("Global Grep  ", Style::default().fg(theme.text_secondary)),
        Span::styled("F1 ", Style::default().fg(theme.accent)),
        Span::styled("Help", Style::default().fg(theme.text_secondary)),
    ]));

    let actions_para = Paragraph::new(action_lines);
    f.render_widget(actions_para, chunks[2]);
}

fn draw_welcome_workspaces(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app.theme();
    let home_path_str = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_else(|_| "~".to_string());
    let title = format!(" Workspaces in {} ", home_path_str);

    let list_block = Block::default()
        .title(Span::styled(title, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface));

    let inner = list_block.inner(area);
    f.render_widget(list_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(inner);

    let srch_icon = crate::icons::action_icon("search", app.icon_theme);
    let filter_line = if app.welcome_filter_query.is_empty() {
        Line::from(vec![
            Span::styled(format!(" {} ", srch_icon.trim()), Style::default().fg(theme.warning)),
            Span::styled("Filter workspaces or type project name... ", Style::default().fg(theme.text_muted)),
            Span::styled("█", Style::default().fg(theme.accent)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!(" {} ", srch_icon.trim()), Style::default().fg(theme.warning)),
            Span::styled(&app.welcome_filter_query, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(theme.accent)),
        ])
    };

    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .style(Style::default().bg(theme.surface_variant));
    f.render_widget(Paragraph::new(filter_line).block(search_block), chunks[0]);

    let filtered_projects = app.filtered_welcome_workspaces();
    let max_visible = chunks[1].height as usize;
    let selected_idx = app.welcome_selected_idx.min(filtered_projects.len().saturating_sub(1));

    let scroll_offset = if selected_idx >= max_visible {
        selected_idx.saturating_sub(max_visible - 1)
    } else {
        0
    };

    let visible_items = filtered_projects.iter().skip(scroll_offset).take(max_visible);
    let mut list_lines = Vec::new();

    for (i, entry) in visible_items.enumerate() {
        let actual_idx = scroll_offset + i;
        let is_selected = actual_idx == selected_idx;

        let (kind_icon, kind_color) = match entry.kind.as_str() {
            "Rust" => ("", Color::Rgb(255, 140, 50)),
            "Node" | "Next.js" | "Astro" | "Vite" => ("", Color::Rgb(104, 187, 89)),
            "Python" => ("", Color::Rgb(255, 212, 59)),
            "Go" => ("", Color::Rgb(0, 172, 215)),
            "Flutter" => ("", Color::Rgb(84, 197, 248)),
            "PHP" => ("", Color::Rgb(136, 146, 191)),
            "Git" => ("", Color::Rgb(240, 80, 50)),
            _ => ("", Color::Rgb(120, 170, 240)),
        };

        let num_badge = if actual_idx < 9 {
            format!("[{}] ", actual_idx + 1)
        } else {
            "    ".to_string()
        };

        let row_bg = if is_selected {
            theme.selection_bg
        } else {
            Color::Reset
        };

        let pointer = if is_selected { "▶ " } else { "  " };

        let branch_badge = if let Some(ref br) = entry.git_branch {
            format!("  {}", br)
        } else {
            String::new()
        };

        let rel_path_str = entry.path.file_name()
            .map(|n| format!("~/{}", n.to_string_lossy()))
            .unwrap_or_else(|| entry.path.to_string_lossy().to_string());

        let line = Line::from(vec![
            Span::styled(pointer, Style::default().fg(theme.accent).bg(row_bg).add_modifier(Modifier::BOLD)),
            Span::styled(num_badge, Style::default().fg(theme.accent_secondary).bg(row_bg).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", kind_icon), Style::default().fg(kind_color).bg(row_bg)),
            Span::styled(format!("{:<18}", entry.name), Style::default().fg(if is_selected { theme.tab_active_fg } else { theme.text_primary }).bg(row_bg).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<20}", rel_path_str), Style::default().fg(theme.text_secondary).bg(row_bg)),
            Span::styled(format!("{:<10}", entry.kind), Style::default().fg(kind_color).bg(row_bg)),
            Span::styled(branch_badge, Style::default().fg(theme.accent_secondary).bg(row_bg)),
        ]);

        list_lines.push(line);
    }

    if list_lines.is_empty() {
        list_lines.push(Line::styled(
            "   No workspaces matching filter. Press Backspace to clear, or 'o' to open custom folder.",
            Style::default().fg(theme.text_muted),
        ));
    }

    f.render_widget(Paragraph::new(list_lines), chunks[1]);

    let footer_line = Line::from(vec![
        Span::styled(" [1-9] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Quick Open │ ", Style::default().fg(theme.text_muted)),
        Span::styled("[↑/↓] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Select │ ", Style::default().fg(theme.text_muted)),
        Span::styled("[Enter] ", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
        Span::styled("Open │ ", Style::default().fg(theme.text_muted)),
        Span::styled("[o] ", Style::default().fg(theme.warning).add_modifier(Modifier::BOLD)),
        Span::styled("Browse │ ", Style::default().fg(theme.text_muted)),
        Span::styled("[Esc] ", Style::default().fg(theme.danger).add_modifier(Modifier::BOLD)),
        Span::styled("Dismiss", Style::default().fg(theme.text_muted)),
    ]);
    f.render_widget(Paragraph::new(footer_line).alignment(Alignment::Center), chunks[2]);
}

fn draw_welcome_prompt_modal(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_area = centered_rect(55, 30, area);
    f.render_widget(Clear, popup_area);
    let theme = app.theme();

    match &app.welcome_prompt_modal {
        crate::app::WelcomePromptModal::None => {}
        crate::app::WelcomePromptModal::OpenFolder { input } => {
            let block = Block::default()
                .title(" 📁 Open Folder / Workspace [Esc to Cancel] ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(theme.surface));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ])
                .split(inner);

            f.render_widget(Paragraph::new("Enter workspace directory path (e.g. C:\\Users\\sadia\\myproject or ~/myproject):").style(Style::default().fg(theme.text_primary)), chunks[0]);

            let input_line = Line::from(vec![
                Span::styled(" Path: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(input, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(theme.accent)),
            ]);
            let input_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_active))
                .style(Style::default().bg(theme.surface_variant));
            f.render_widget(Paragraph::new(input_line).block(input_block), chunks[1]);

            let hints = vec![
                Line::styled("• Supports absolute paths (C:\\...) and home paths (~/)", Style::default().fg(theme.text_muted)),
                Line::styled("• Press Enter to switch workspace, Esc to cancel", Style::default().fg(theme.text_muted)),
            ];
            f.render_widget(Paragraph::new(hints), chunks[2]);

            let action_spans = vec![
                Span::styled(" [Enter] Open Workspace ", Style::default().bg(theme.accent).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled(" [Esc] Cancel ", Style::default().bg(theme.surface_variant).fg(Color::White).add_modifier(Modifier::BOLD)),
            ];
            f.render_widget(Paragraph::new(Line::from(action_spans)).alignment(Alignment::Center), chunks[3]);
        }
        crate::app::WelcomePromptModal::CloneRepo { url_input, dest_input, active_field } => {
            let block = Block::default()
                .title("  Clone Git Repository [Esc to Cancel] ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(theme.surface));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ])
                .split(inner);

            f.render_widget(Paragraph::new("Enter Git repository URL and local destination path:").style(Style::default().fg(theme.text_primary)), chunks[0]);

            let url_line = Line::from(vec![
                Span::styled(" URL: ", Style::default().fg(if *active_field == 0 { theme.accent } else { theme.text_muted })),
                Span::styled(url_input, Style::default().fg(theme.text_primary)),
                if *active_field == 0 { Span::styled("█", Style::default().fg(theme.accent)) } else { Span::raw("") },
            ]);
            let url_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if *active_field == 0 { theme.border_active } else { theme.border_inactive }))
                .style(Style::default().bg(theme.surface_variant));
            f.render_widget(Paragraph::new(url_line).block(url_block), chunks[1]);

            let dest_line = Line::from(vec![
                Span::styled(" Dest: ", Style::default().fg(if *active_field == 1 { theme.accent } else { theme.text_muted })),
                Span::styled(dest_input, Style::default().fg(theme.text_primary)),
                if *active_field == 1 { Span::styled("█", Style::default().fg(theme.accent)) } else { Span::raw("") },
            ]);
            let dest_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if *active_field == 1 { theme.border_active } else { theme.border_inactive }))
                .style(Style::default().bg(theme.surface_variant));
            f.render_widget(Paragraph::new(dest_line).block(dest_block), chunks[2]);

            f.render_widget(Paragraph::new("Press Tab to toggle fields, Enter to clone and open.").style(Style::default().fg(theme.text_muted)), chunks[3]);

            let action_spans = vec![
                Span::styled(" [Enter] Clone & Open ", Style::default().bg(theme.accent).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled(" [Esc] Cancel ", Style::default().bg(theme.surface_variant).fg(Color::White).add_modifier(Modifier::BOLD)),
            ];
            f.render_widget(Paragraph::new(Line::from(action_spans)).alignment(Alignment::Center), chunks[4]);
        }
        crate::app::WelcomePromptModal::NewProject { name_input } => {
            let block = Block::default()
                .title("  Create New Project Folder [Esc to Cancel] ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(theme.surface));
            let inner = block.inner(popup_area);
            f.render_widget(block, popup_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(2),
                ])
                .split(inner);

            f.render_widget(Paragraph::new("Enter new project directory name:").style(Style::default().fg(theme.text_primary)), chunks[0]);

            let name_line = Line::from(vec![
                Span::styled(" Name: ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(name_input, Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(theme.accent)),
            ]);
            let name_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border_active))
                .style(Style::default().bg(theme.surface_variant));
            f.render_widget(Paragraph::new(name_line).block(name_block), chunks[1]);

            f.render_widget(Paragraph::new("Directory will be created in your user directory and opened as workspace.").style(Style::default().fg(theme.text_muted)), chunks[2]);

            let action_spans = vec![
                Span::styled(" [Enter] Create & Open ", Style::default().bg(theme.accent).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw("    "),
                Span::styled(" [Esc] Cancel ", Style::default().bg(theme.surface_variant).fg(Color::White).add_modifier(Modifier::BOLD)),
            ];
            f.render_widget(Paragraph::new(Line::from(action_spans)).alignment(Alignment::Center), chunks[3]);
        }
    }
}