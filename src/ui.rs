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
    draw_body(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);

    // Modal overlays (if active)
    if app.show_help {
        draw_help_modal(f, size);
    } else if app.show_command_palette {
        draw_command_palette_modal(f, size);
    } else if app.show_agent_launch_modal {
        draw_agent_launch_modal(f, app, size);
    } else if app.show_diff_modal {
        draw_diff_modal(f, app, size);
    }
}

fn draw_header(f: &mut Frame, app: &mut App, area: Rect) {
    app.header_area = area;
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(32), Constraint::Length(45), Constraint::Length(46)])
        .split(area);

    let mut left_spans = vec![
        Span::styled(" ● ", Style::default().fg(Color::Rgb(34, 197, 94))),
        Span::styled("TIDE", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled("📁 ", Style::default().fg(Color::Rgb(56, 189, 248))),
        Span::styled(&app.workspace_name, Style::default().fg(Color::White)),
        Span::raw(" "),
        Span::styled(" ", Style::default().fg(Color::Rgb(168, 85, 247))),
        Span::styled(&app.git_info.branch, Style::default().fg(Color::Rgb(168, 85, 247))),
    ];

    if let Some((msg, time)) = &app.status_message {
        if time.elapsed().as_secs() < 12 {
            left_spans.push(Span::raw(" │ "));
            left_spans.push(Span::styled(format!("🔔 {}", msg), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        }
    }

    let toggle_btn = |label: &str, active: bool| {
        if active {
            Span::styled(format!(" [{}] ", label), Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!(" [{}] ", label), Style::default().fg(Color::Rgb(100, 116, 139)))
        }
    };

    let mid_spans = vec![
        toggle_btn("F2 Exp", app.show_explorer),
        toggle_btn("F5 Prev", app.show_preview),
        toggle_btn("F3 Git", app.show_git),
        toggle_btn("F4 Agt", app.show_agents),
        toggle_btn("F11 Max", app.is_maximized),
    ];

    let right_spans = vec![
        Span::styled("F7/8", Style::default().fg(Color::Rgb(56, 189, 248))),
        Span::raw(" Tabs │ "),
        Span::styled("F6", Style::default().fg(Color::Yellow)),
        Span::raw(" Pane │ "),
        Span::styled("F1", Style::default().fg(Color::Rgb(56, 189, 248))),
        Span::raw(" Help │ "),
        Span::styled("<C-q>", Style::default().fg(Color::Rgb(239, 68, 68))),
        Span::raw(" Quit"),
    ];

    f.render_widget(Paragraph::new(Line::from(left_spans)), header_chunks[0]);
    f.render_widget(Paragraph::new(Line::from(mid_spans)).alignment(Alignment::Center), header_chunks[1]);
    f.render_widget(Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right), header_chunks[2]);
}

fn draw_body(f: &mut Frame, app: &mut App, area: Rect) {
    app.body_area = area;

    if app.is_maximized {
        app.explorer_area = Rect::default();
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

fn draw_file_preview_pane(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_pane == ActivePane::FilePreview;
    let border_color = if is_focused {
        Color::Rgb(56, 189, 248)
    } else {
        Color::Rgb(30, 41, 59)
    };

    let dirty_tag = if app.editor_dirty { " ● [Modified]" } else { "" };
    let (title, lines) = match &app.preview_content {
        Some((name, lines)) => (format!(" 📄 {}{} [×] ", name, dirty_tag), lines.as_slice()),
        None => (" 📄 File Preview [×] ".to_string(), [].as_slice()),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if lines.is_empty() {
        let empty_msg = vec![
            Line::raw(""),
            Line::styled("  Select a file in Explorer (j/k or click) to preview its contents.", Style::default().fg(Color::Rgb(100, 116, 139))),
        ];
        f.render_widget(Paragraph::new(empty_msg), inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let view_height = chunks[0].height as usize;
    let scroll = app.preview_scroll;
    let visible_lines = lines.iter().skip(scroll).take(view_height);

    let mut rendered_lines = Vec::new();
    for (idx, line) in visible_lines.enumerate() {
        let line_num = scroll + idx + 1;
        let mut spans = vec![
            Span::styled(format!("{:>4} │ ", line_num), Style::default().fg(Color::Rgb(100, 116, 139))),
        ];

        spans.extend(highlight_code_line(line));
        rendered_lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(rendered_lines), chunks[0]);

    // Render visual scrollbar on code preview
    let total_lines = lines.len();
    if total_lines > view_height {
        let max_scroll = total_lines.saturating_sub(view_height);
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll.min(max_scroll));
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(Color::Rgb(56, 189, 248)));
        f.render_stateful_widget(scrollbar, chunks[0], &mut scrollbar_state);
    }

    // Bottom buffer status line
    let mode_badge = if app.editor_dirty {
        Span::styled(" MODIFIED ", Style::default().bg(Color::Rgb(245, 158, 11)).fg(Color::Black).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" EDIT ", Style::default().bg(Color::Rgb(56, 189, 248)).fg(Color::Black).add_modifier(Modifier::BOLD))
    };
    let name_span = Span::styled(format!(" {} ", app.preview_content.as_ref().map(|(n, _)| n.as_str()).unwrap_or("")), Style::default().fg(Color::White));
    let desc_span = Span::styled("Ctrl+s: Save │ Ctrl+e: VS Code │ Esc: Explorer  ", Style::default().fg(Color::Rgb(148, 163, 184)));
    let pos_span = Span::styled(format!("Ln {}, Col {} │ utf-8 ", app.editor_cursor.0 + 1, app.editor_cursor.1 + 1), Style::default().fg(Color::Rgb(148, 163, 184)));

    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(35), Constraint::Length(30)])
        .split(chunks[1]);

    f.render_widget(Paragraph::new(Line::from(vec![mode_badge, name_span, desc_span])), status_chunks[0]);
    f.render_widget(Paragraph::new(Line::from(pos_span)).alignment(Alignment::Right), status_chunks[1]);

    // Active Cursor positioning
    if is_focused {
        let cursor_row = app.editor_cursor.0.saturating_sub(scroll);
        let cursor_col = app.editor_cursor.1 + 7; // line numbers margin
        if cursor_row < view_height && (cursor_col as u16) < inner.width {
            f.set_cursor_position((inner.x + cursor_col as u16, chunks[0].y + cursor_row as u16));
        }
    }
}

fn highlight_code_line(line: &str) -> Vec<Span<'static>> {
    if line.trim_start().starts_with("//") || line.trim_start().starts_with("#") {
        return vec![Span::styled(line.to_string(), Style::default().fg(Color::Rgb(100, 116, 139)).add_modifier(Modifier::ITALIC))];
    }

    let mut spans = Vec::new();
    let words = line.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_');
    for word in words {
        let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
        let style = match trimmed {
            "fn" | "let" | "pub" | "mut" | "struct" | "enum" | "impl" | "use" | "mod" | "match" | "return" | "async" | "trait" | "type" | "const" => {
                Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)
            }
            "Self" | "Option" | "Result" | "String" | "Vec" | "bool" | "u8" | "u16" | "u32" | "u64" | "usize" | "i32" | "i64" => {
                Style::default().fg(Color::Rgb(234, 179, 8))
            }
            "true" | "false" | "Some" | "None" | "Ok" | "Err" => {
                Style::default().fg(Color::Rgb(244, 63, 94))
            }
            _ => {
                if word.contains('"') {
                    Style::default().fg(Color::Rgb(34, 197, 94))
                } else {
                    Style::default().fg(Color::Rgb(226, 232, 240))
                }
            }
        };
        spans.push(Span::styled(word.to_string(), style));
    }
    spans
}

fn draw_explorer_pane(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_pane == ActivePane::Explorer;
    let border_color = if is_focused {
        Color::Rgb(56, 189, 248)
    } else {
        Color::Rgb(30, 41, 59)
    };

    let items: Vec<ListItem> = app
        .explorer
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let indent = "  ".repeat(entry.depth);
            let icon = entry.icon();
            let is_selected = i == app.explorer.selected;

            let line_style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(30, 58, 138))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(Color::Rgb(148, 163, 184))
            } else {
                Style::default().fg(Color::Rgb(226, 232, 240))
            };

            ListItem::new(format!("{}{}{}", indent, icon, entry.name)).style(line_style)
        })
        .collect();

    let block = Block::default()
        .title(" 📁 Explorer [×] ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    let list = List::new(items).block(block);
    f.render_widget(list, area);

    // Render Explorer scrollbar if entries exceed area height
    let total_entries = app.explorer.entries.len();
    let view_height = inner.height as usize;
    if total_entries > view_height && view_height > 0 {
        let max_scroll = total_entries.saturating_sub(view_height);
        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(app.explorer.selected.min(max_scroll));
        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(Color::Rgb(100, 116, 139)));
        f.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }
}

fn draw_terminal_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.active_pane == ActivePane::Terminal;
    let border_color = if is_focused {
        Color::Rgb(56, 189, 248)
    } else {
        Color::Rgb(30, 41, 59)
    };

    // Inner layout: Tabs (1) + Terminal Screen (remaining)
    let term_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    
    let inner_area = term_block.inner(area);
    f.render_widget(term_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner_area);

    app.terminal_tabs_area = chunks[0];

    // Tab bar with interactive tabs, [+] button, and [F7 ◀] [▶ F8] navigation controls
    let tab_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(22)])
        .split(chunks[0]);

    let mut tab_spans = Vec::new();
    for (i, t) in app.terminals.iter().enumerate() {
        let is_active = i == app.active_terminal_idx;
        let indicator = if is_active { "● " } else { "○ " };
        let dot_color = if is_active { Color::Rgb(34, 197, 94) } else { Color::Rgb(100, 116, 139) };
        let text_style = if is_active {
            Style::default().bg(Color::Rgb(30, 58, 138)).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(148, 163, 184))
        };

        tab_spans.push(Span::styled(format!(" {} ", indicator), Style::default().fg(dot_color)));
        tab_spans.push(Span::styled(format!("{}: {} ", i + 1, t.title), text_style));
        tab_spans.push(Span::styled(" │", Style::default().fg(Color::Rgb(51, 65, 85))));
    }

    // [+] New tab button
    tab_spans.push(Span::styled(" [+] ", Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)));

    let right_tab_controls = vec![
        Span::styled("[F7 ◀] ", Style::default().fg(Color::Yellow)),
        Span::styled("[▶ F8]", Style::default().fg(Color::Yellow)),
        Span::raw(" "),
    ];

    f.render_widget(Paragraph::new(Line::from(tab_spans)), tab_bar_chunks[0]);
    f.render_widget(Paragraph::new(Line::from(right_tab_controls)).alignment(Alignment::Right), tab_bar_chunks[1]);

    // Render VT100 screen buffer
    if let Some(session) = app.terminals.get(app.active_terminal_idx) {
        let screen_area = chunks[1];
        
        // Dynamically resize PTY to match current pane dimensions
        if screen_area.height > 0 && screen_area.width > 0 {
            session.resize(screen_area.height, screen_area.width);
        }

        if let Ok(parser) = session.parser.lock() {
            let screen = parser.screen();
            let mut lines: Vec<Line> = Vec::new();

            for row in 0..screen_area.height {
                let mut spans: Vec<Span> = Vec::new();
                for col in 0..screen_area.width {
                    let cell = screen.cell(row, col);
                    if let Some(cell) = cell {
                        let ch = cell.contents();
                        let text = if ch.is_empty() { " " } else { &ch };
                        
                        let fg = convert_vt100_color(cell.fgcolor());
                        let bg = convert_vt100_color(cell.bgcolor());

                        let mut style = Style::default();
                        if let Some(c) = fg { style = style.fg(c); }
                        if let Some(c) = bg { style = style.bg(c); }
                        if cell.bold() { style = style.add_modifier(Modifier::BOLD); }
                        if cell.italic() { style = style.add_modifier(Modifier::ITALIC); }
                        if cell.underline() { style = style.add_modifier(Modifier::UNDERLINED); }

                        spans.push(Span::styled(text.to_string(), style));
                    } else {
                        spans.push(Span::raw(" "));
                    }
                }
                lines.push(Line::from(spans));
            }

            f.render_widget(Paragraph::new(lines), screen_area);

            // Active Cursor placement
            if is_focused {
                let (c_row, c_col) = screen.cursor_position();
                if c_row < screen_area.height && c_col < screen_area.width {
                    f.set_cursor_position((screen_area.x + c_col, screen_area.y + c_row));
                }
            }
        }
    }
}

fn draw_git_pane(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_pane == ActivePane::Git;
    let border_color = if is_focused {
        Color::Rgb(56, 189, 248)
    } else {
        Color::Rgb(30, 41, 59)
    };

    let block = Block::default()
        .title(" 🌿 Git [×] ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let mut lines = vec![
        Line::from(vec![
            Span::styled(" ", Style::default().fg(Color::Rgb(168, 85, 247))),
            Span::styled(&app.git_info.branch, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
    ];

    if app.git_info.is_clean {
        lines.push(Line::from(vec![
            Span::styled("◆ ", Style::default().fg(Color::Rgb(34, 197, 94))),
            Span::styled("Clean working tree", Style::default().fg(Color::Rgb(148, 163, 184))),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("◆ ", Style::default().fg(Color::Rgb(234, 179, 8))),
            Span::styled(format!("{} modified files", app.git_info.changed_files.len()), Style::default().fg(Color::Rgb(234, 179, 8))),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled("Recent commits", Style::default().fg(Color::Rgb(148, 163, 184)).add_modifier(Modifier::UNDERLINED)));

    for commit in &app.git_info.recent_commits {
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", commit.hash), Style::default().fg(Color::Rgb(56, 189, 248))),
            Span::styled(commit.message.clone(), Style::default().fg(Color::Rgb(226, 232, 240))),
            Span::styled(format!(" ({})", commit.relative_time), Style::default().fg(Color::Rgb(100, 116, 139))),
        ]));
    }

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_agents_pane(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.active_pane == ActivePane::Agents;
    let border_color = if is_focused {
        Color::Rgb(56, 189, 248)
    } else {
        Color::Rgb(30, 41, 59)
    };

    let title = format!(" 🤖 Agents ({} running) [×] ", app.agents.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let mut lines = Vec::new();

    if app.agents.is_empty() {
        lines.push(Line::raw(""));
        lines.push(Line::styled("  No active agent sessions.", Style::default().fg(Color::Rgb(148, 163, 184))));
        lines.push(Line::raw(""));
        lines.push(Line::from(vec![
            Span::raw("  Press "),
            Span::styled("[a]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" to launch an agent:"),
        ]));
        lines.push(Line::styled("  1. Antigravity CLI (agy)", Style::default().fg(Color::Rgb(56, 189, 248))));
        lines.push(Line::styled("  2. Claude Code CLI (claude)", Style::default().fg(Color::Rgb(56, 189, 248))));
        lines.push(Line::styled("  3. Worktree Shell (powershell)", Style::default().fg(Color::Rgb(56, 189, 248))));
        lines.push(Line::raw(""));
        lines.push(Line::styled("  Each agent runs in an isolated Git worktree.", Style::default().fg(Color::Rgb(100, 116, 139))));
    } else {
        for (i, agent) in app.agents.iter().enumerate() {
            let is_selected = is_focused && i == app.active_agent_idx;
            let (dot_color, status_text) = match &agent.state {
                AgentState::Running => (Color::Rgb(34, 197, 94), "Running"),
                AgentState::WaitingForInput => (Color::Rgb(234, 179, 8), "Waiting for input"),
                AgentState::Completed => (Color::Rgb(56, 189, 248), "Completed"),
                AgentState::Failed(_) => (Color::Rgb(239, 68, 68), "Failed"),
                AgentState::Idle => (Color::Rgb(148, 163, 184), "Idle"),
            };

            let conf_tag = match agent.confidence {
                Confidence::Confirmed => "[Confirmed]",
                Confidence::Inferred => "[Inferred?]",
                Confidence::Unknown => "[Unknown]",
            };

            let prefix = if is_selected { "▶ " } else { "  " };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)),
                Span::styled("● ", Style::default().fg(dot_color)),
                Span::styled(&agent.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(status_text, Style::default().fg(dot_color)),
                Span::styled(format!(" {} ", conf_tag), Style::default().fg(Color::Rgb(100, 116, 139))),
                Span::styled(agent.formatted_runtime(), Style::default().fg(Color::Rgb(148, 163, 184))),
            ]));

            lines.push(Line::from(vec![
                Span::raw("     ↳ "),
                Span::styled(format!("Branch: {}", agent.worktree_branch), Style::default().fg(Color::Rgb(168, 85, 247))),
            ]));

            lines.push(Line::from(vec![
                Span::raw("     ↳ "),
                Span::styled(format!("Task: {}", agent.target_file_or_task), Style::default().fg(Color::Rgb(148, 163, 184))),
            ]));

            lines.push(Line::raw(""));
        }
    }

    if is_focused {
        lines.push(Line::styled(
            "[a] Launch  [d] Diff  [m] Merge  [x] Discard  [Enter] Shell",
            Style::default().fg(Color::Rgb(56, 189, 248)),
        ));
    }

    let p = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(45)])
        .split(area);

    let active_shell = app
        .terminals
        .get(app.active_terminal_idx)
        .map(|t| t.title.as_str())
        .unwrap_or("Shell");

    let raw_path = app.workspace_dir.to_string_lossy();
    let clean_path = raw_path.strip_prefix(r"\\?\").unwrap_or(&raw_path);

    let left = vec![
        Span::styled(" ● ", Style::default().fg(Color::Rgb(34, 197, 94))),
        Span::styled("TIDE", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" │ "),
        Span::styled(active_shell, Style::default().fg(Color::Rgb(56, 189, 248))),
        Span::raw(" │ "),
        Span::styled(clean_path.to_string(), Style::default().fg(Color::Rgb(148, 163, 184))),
    ];

    let right = vec![
        Span::styled(format!("Workspace: {} │ ", app.workspace_name), Style::default().fg(Color::Rgb(148, 163, 184))),
        Span::styled(format!("Git: {} │ ", app.git_info.branch), Style::default().fg(Color::Rgb(168, 85, 247))),
        Span::styled(format!("Agents: {}/{}", app.agents.len(), app.agents.len()), Style::default().fg(Color::Rgb(34, 197, 94))),
    ];

    f.render_widget(Paragraph::new(Line::from(left)), status_chunks[0]);
    f.render_widget(Paragraph::new(Line::from(right)).alignment(Alignment::Right), status_chunks[1]);
}

fn draw_help_modal(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 50, area);
    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::styled("TIDE Keyboard & Mouse Shortcuts", Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::from(vec![
            Span::styled("Mouse Click     ", Style::default().fg(Color::Yellow)),
            Span::raw("Click any pane to focus, click tabs to switch, click [×] to close"),
        ]),
        Line::from(vec![
            Span::styled("F6 / <Tab>      ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle active pane (F6 works everywhere, including inside terminal)"),
        ]),
        Line::from(vec![
            Span::styled("F2 / F3 / F4 / F5", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle panels on/off: F2 (Explorer), F5 (Preview), F3 (Git), F4 (Agents)"),
        ]),
        Line::from(vec![
            Span::styled("F11             ", Style::default().fg(Color::Yellow)),
            Span::raw("Maximize / restore active pane to full screen"),
        ]),
        Line::from(vec![
            Span::styled("Mouse Drag      ", Style::default().fg(Color::Yellow)),
            Span::raw("Drag pane borders to dynamically resize cards (columns & rows)"),
        ]),
        Line::from(vec![
            Span::styled("Alt+[ / Alt+]   ", Style::default().fg(Color::Yellow)),
            Span::raw("Resize Explorer column width"),
        ]),
        Line::from(vec![
            Span::styled("Alt+- / Alt+=   ", Style::default().fg(Color::Yellow)),
            Span::raw("Resize File Preview vs Terminal split height"),
        ]),
        Line::from(vec![
            Span::styled("Alt+{ / Alt+}   ", Style::default().fg(Color::Yellow)),
            Span::raw("Resize Right panel (Git & Agents) column width"),
        ]),
        Line::from(vec![
            Span::styled("<Ctrl-b>        ", Style::default().fg(Color::Yellow)),
            Span::raw("Escape child terminal focus and return to TIDE controls"),
        ]),
        Line::from(vec![
            Span::styled("Type / Ctrl+s   ", Style::default().fg(Color::Yellow)),
            Span::raw("Code Preview: type directly to edit buffer, [Ctrl+s] save to disk"),
        ]),
        Line::from(vec![
            Span::styled("Ctrl+e / Ctrl+o ", Style::default().fg(Color::Yellow)),
            Span::raw("Open current file in VS Code"),
        ]),
        Line::from(vec![
            Span::styled("Alt+1 .. Alt+9  ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch directly to Terminal tab 1..9"),
        ]),
        Line::from(vec![
            Span::styled("<Ctrl-a> / a    ", Style::default().fg(Color::Yellow)),
            Span::raw("Launch coding agent in isolated Git worktree"),
        ]),
        Line::from(vec![
            Span::styled("d / m / x       ", Style::default().fg(Color::Yellow)),
            Span::raw("Agents pane: [d] Diff Review, [m] Merge into main, [x] Discard"),
        ]),
        Line::from(vec![
            Span::styled("<Ctrl-q>        ", Style::default().fg(Color::Rgb(239, 68, 68))),
            Span::raw("Quit TIDE (Cleanly stops all Job Objects and child processes)"),
        ]),
        Line::raw(""),
        Line::styled("Press Esc or F1 to close this help window", Style::default().fg(Color::Rgb(100, 116, 139))),
    ];

    let block = Block::default()
        .title(" Help (F1) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(56, 189, 248)));

    let p = Paragraph::new(help_text).block(block).wrap(Wrap { trim: true });
    f.render_widget(p, popup_area);
}

fn draw_command_palette_modal(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(50, 40, area);
    f.render_widget(Clear, popup_area);

    let commands = vec![
        Line::from(vec![Span::styled("1. ", Style::default().fg(Color::Yellow)), Span::raw("Spawn New Shell Terminal (PowerShell)")]),
        Line::from(vec![Span::styled("2. ", Style::default().fg(Color::Yellow)), Span::raw("Launch Agent in Isolated Git Worktree")]),
        Line::from(vec![Span::styled("3. ", Style::default().fg(Color::Yellow)), Span::raw("Git: Refresh Status & Commit Log")]),
        Line::from(vec![Span::styled("4. ", Style::default().fg(Color::Yellow)), Span::raw("Explorer: Refresh Workspace Files")]),
        Line::from(vec![Span::styled("5. ", Style::default().fg(Color::Yellow)), Span::raw("Quit TIDE")]),
        Line::raw(""),
        Line::styled("Type number or press Esc to cancel", Style::default().fg(Color::Rgb(100, 116, 139))),
    ];

    let block = Block::default()
        .title(" Command Palette (<Ctrl-p>) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(56, 189, 248)));

    let p = Paragraph::new(commands).block(block);
    f.render_widget(p, popup_area);
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

    let outer_block = Block::default()
        .title(" 🤖 Launch Agent Session [×] ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(56, 189, 248)));

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
        Line::styled("Launch New Coding-Agent in Isolated Git Worktree", Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)),
        Line::styled("Select an agent below and confirm to start an isolated session:", Style::default().fg(Color::Rgb(148, 163, 184))),
    ];
    f.render_widget(Paragraph::new(header_lines), chunks[0]);

    // Agent options
    let agents_data = [
        (
            "1",
            "Antigravity CLI (agy)",
            "Google Deepmind Advanced Coding Agent",
            "[ Autonomous / DeepMind ]",
            Color::Rgb(56, 189, 248),
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
            Color::Rgb(168, 85, 247),
        ),
    ];

    let mut option_lines = Vec::new();
    for (i, (num, title, desc, tag, tag_color)) in agents_data.iter().enumerate() {
        let is_selected = app.agent_launch_selected_idx == i;

        if is_selected {
            // Selected / Panned-to styling: rich background highlight, cyan pointer, bold text
            let row1_spans = vec![
                Span::styled(" ▶ ", Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" [{}] ", num), Style::default().bg(Color::Rgb(56, 189, 248)).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", title), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", tag), Style::default().fg(*tag_color).add_modifier(Modifier::BOLD)),
                Span::styled(" [ SELECTED ]", Style::default().bg(Color::Rgb(34, 197, 94)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            ];
            let row2_spans = vec![
                Span::raw("     ↳ "),
                Span::styled(*desc, Style::default().fg(Color::Rgb(226, 232, 240)).add_modifier(Modifier::ITALIC)),
            ];

            option_lines.push(Line::from(row1_spans).style(Style::default().bg(Color::Rgb(30, 58, 138))));
            option_lines.push(Line::from(row2_spans).style(Style::default().bg(Color::Rgb(30, 58, 138))));
        } else {
            // Unselected styling
            let row1_spans = vec![
                Span::raw("   "),
                Span::styled(format!(" {}. ", num), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", title), Style::default().fg(Color::Rgb(226, 232, 240)).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", tag), Style::default().fg(Color::Rgb(100, 116, 139))),
            ];
            let row2_spans = vec![
                Span::raw("     ↳ "),
                Span::styled(*desc, Style::default().fg(Color::Rgb(148, 163, 184))),
            ];

            option_lines.push(Line::from(row1_spans));
            option_lines.push(Line::from(row2_spans));
        }

        option_lines.push(Line::raw(""));
    }

    f.render_widget(Paragraph::new(option_lines), chunks[1]);

    // System info
    let info_lines = vec![
        Line::styled("• Dedicated branch & worktree (.tide/worktrees/<id>) created automatically.", Style::default().fg(Color::Rgb(100, 116, 139))),
        Line::styled("• All processes bound to Windows Job Object for clean teardown.", Style::default().fg(Color::Rgb(100, 116, 139))),
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
            Style::default().bg(Color::Rgb(34, 197, 94)).fg(Color::Black).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "  [ ✖ Cancel (Esc) ]  ",
            Style::default().bg(Color::Rgb(51, 65, 85)).fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "↑/↓ or j/k: Pan/Highlight │ 1-3: Quick Pick",
            Style::default().fg(Color::Rgb(148, 163, 184)),
        ),
    ];
    f.render_widget(Paragraph::new(Line::from(button_spans)).alignment(Alignment::Center), chunks[3]);
}

fn draw_diff_modal(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(80, 75, area);
    f.render_widget(Clear, popup_area);

    let (title, content) = match &app.diff_modal_content {
        Some((t, c)) => (t.as_str(), c.as_str()),
        None => ("Agent Diff", "No diff available"),
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(popup_area);

    let block = Block::default()
        .title(format!(" 🔍 {} ", title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(168, 85, 247)));

    let mut diff_lines = Vec::new();
    for line in content.lines() {
        let style = if line.starts_with('+') && !line.starts_with("+++") {
            Style::default().fg(Color::Rgb(34, 197, 94))
        } else if line.starts_with('-') && !line.starts_with("---") {
            Style::default().fg(Color::Rgb(239, 68, 68))
        } else if line.starts_with("@@") {
            Style::default().fg(Color::Rgb(56, 189, 248)).add_modifier(Modifier::BOLD)
        } else if line.starts_with("diff --git") {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(226, 232, 240))
        };
        diff_lines.push(Line::styled(line.to_string(), style));
    }

    let p = Paragraph::new(diff_lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(p, chunks[0]);

    let action_spans = vec![
        Span::styled(" [A] ", Style::default().bg(Color::Rgb(34, 197, 94)).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" Accept & Merge to Base   "),
        Span::styled(" [D] ", Style::default().bg(Color::Rgb(239, 68, 68)).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" Discard Worktree & Branch   "),
        Span::styled(" [Esc] ", Style::default().bg(Color::Rgb(100, 116, 139)).fg(Color::White)),
        Span::raw(" Close"),
    ];
    f.render_widget(Paragraph::new(Line::from(action_spans)).alignment(Alignment::Center), chunks[1]);
}
