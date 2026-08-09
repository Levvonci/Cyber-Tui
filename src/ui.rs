use crate::app::{App, CreatePostFocus, InputMode, Screen};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
const ACCENT: Color = Color::Cyan;
const MUTED: Color = Color::DarkGray;

pub fn draw(f: &mut Frame, app: &App) {
    match app.screen {
        Screen::Login => draw_login(f, app),
        _ => {
            let area = f.area();
            let h = if app.sidebar_visible {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(22), Constraint::Min(0)])
                    .split(area)
            } else {
                Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(0), Constraint::Min(0)])
                    .split(area)
            };

            if app.sidebar_visible {
                draw_sidebar(f, app, h[0]);
            }
            draw_content(f, app, h[1]);
        }
    }
}

fn draw_login(f: &mut Frame, app: &App) {
    let area = f.area();
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "CYBER-TUI",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Login a cyberspace.online",
            Style::default().fg(MUTED),
        )),
    ])
    .alignment(Alignment::Center);
    f.render_widget(title, v[0]);

    let email_style = if app.login_field_email_active {
        Style::default().fg(ACCENT)
    } else {
        Style::default().fg(MUTED)
    };
    let email_block = Paragraph::new(app.login_email.value())
        .style(email_style)
        .block(Block::default().borders(Borders::ALL).title("Email"));
    f.render_widget(email_block, centered(v[1], 50, 100));

    let password_style = if !app.login_field_email_active {
        Style::default().fg(ACCENT)
    } else {
        Style::default().fg(MUTED)
    };
    let masked: String = "*".repeat(app.login_password.value().len());
    let password_block = Paragraph::new(masked)
        .style(password_style)
        .block(Block::default().borders(Borders::ALL).title("Password"));
    f.render_widget(password_block, centered(v[2], 50, 100));

    let hint = if let Some(err) = &app.login_error {
        Line::from(Span::styled(err.clone(), Style::default().fg(Color::Red)))
    } else {
        Line::from(Span::styled(
            "Tab: cambia campo · Enter: login · Esc: esci",
            Style::default().fg(MUTED),
        ))
    };
    f.render_widget(Paragraph::new(hint).alignment(Alignment::Center), v[3]);
}

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let items = vec![
        ListItem::new("Feed"),
        ListItem::new("Notifications"),
        ListItem::new("c-Mail"),
        ListItem::new("cIRC"),
        ListItem::new("Bookmarks"),
        ListItem::new("Topics"),
        ListItem::new("Profile"),
        ListItem::new("Journal"),
        ListItem::new("Guilds"),
        ListItem::new("Jukebox"),
    ];
    let items: Vec<ListItem> = items
        .into_iter()
        .enumerate()
        .map(|(i, it)| {
            let s = if i == app.sidebar_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            it.style(s)
        })
        .collect();
    f.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Menu")),
        area,
    );
}

fn draw_content(f: &mut Frame, app: &App, area: Rect) {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    match app.screen {
        Screen::Feed => draw_feed(f, app, v[0]),
        Screen::Notifications => draw_notifications(f, app, v[0]),
        Screen::CmailList => draw_cmail_list(f, app, v[0]),
        Screen::CmailConversation => draw_cmail_conversation(f, app, v[0]),
        Screen::CircList => draw_circ_list(f, app, v[0]),
        Screen::CircRoom => draw_circ_room(f, app, v[0]),
        Screen::Bookmarks => draw_bookmarks(f, app, v[0]),
        Screen::Topics => draw_topics(f, app, v[0]),
        Screen::TopicPosts => draw_topic_posts(f, app, v[0]),
        Screen::Profile => draw_profile(f, app, v[0]),
        Screen::Journal => draw_journal(f, app, v[0]),
        Screen::JournalNote => draw_journal_note(f, app, v[0]),
        Screen::Guilds => draw_guilds(f, app, v[0]),
        Screen::GuildDetail => draw_guild_detail(f, app, v[0]),
        Screen::Jukebox => draw_jukebox(f, app, v[0]),
        _ => {}
    }

    draw_status(f, app, v[1]);

    if app.show_create_post_modal {
        draw_overlay_backdrop(f, v[0]);

        let overlay = centered(v[0], 72, 78);

        draw_create_post_modal(f, app, overlay);
    } else if app.show_post_modal {
        draw_overlay_backdrop(f, v[0]);

        let overlay = centered(v[0], 72, 78);

        draw_post_modal(f, app, overlay);

        if app.show_reply_modal {
            let reply_overlay = centered(v[0], 60, 40);
            draw_reply_modal(f, app, reply_overlay);
        }
    }

    draw_error_box(f, app, v[0]);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let hints = if app.show_create_post_modal {
        "Tab: cambia campo | Ctrl+S: pubblica | Esc: annulla"
    } else if app.show_reply_modal {
        "Ctrl+S: invia reply | Esc: annulla"
    } else if app.show_post_modal {
        "r: reply | Esc: chiudi"
    } else if app.input_mode == InputMode::Compose {
        match app.screen {
            Screen::CircRoom => "Enter: invia | Esc: annulla",
            Screen::CmailConversation => "Enter: invia | Esc: annulla",
            _ => "Enter: invia | Esc: annulla",
        }
    } else {
        match app.screen {
            Screen::Feed => {
                "j/k: naviga | Enter: apri | c: nuovo post | w: watch | x: bookmark | s: menu | q: esci"
            }
            Screen::Notifications => {
                "j/k: naviga | s: menu | Esc: indietro | q: esci"
            }
            Screen::CmailList => {
                "j/k: naviga | Enter: apri | s: menu | Esc: indietro | q: esci"
            }
            Screen::CmailConversation => {
                "i: scrivi | Esc: indietro | s: menu | q: esci"
            }
            Screen::CircList => {
                "j/k: naviga | Enter: apri | s: menu | Esc: indietro | q: esci"
            }
            Screen::CircRoom => {
                "i: scrivi | Esc: indietro | s: menu | q: esci"
            }
            Screen::Bookmarks => {
                "j/k: naviga | s: menu | Esc: indietro | q: esci"
            }
            Screen::Topics => {
                "j/k: naviga | Enter: apri | s: menu | q: esci"
            }
            Screen::TopicPosts => {
                "j/k: naviga | Enter: apri | Esc: indietro | q: esci"
            }
            Screen::Profile => {
                "j/k: naviga | Enter: apri | Esc: indietro | q: esci"
            }
            Screen::Journal => {
                "j/k: naviga | Enter: apri | s: menu | q: esci"
            }
            Screen::JournalNote => {
                "Esc: indietro | s: menu | q: esci"
            }
            Screen::Guilds => {
                "j/k: naviga | Enter: apri | s: menu | q: esci"
            }
            Screen::GuildDetail => {
                "Esc: indietro | s: menu | q: esci"
            }
            Screen::Jukebox => {
                "s: menu | q: esci"
            }
            Screen::Login => {
                "Tab: cambia campo | Enter: login | Esc: esci"
            }
        }
    };

    let status = app.status_message.as_deref().unwrap_or("");

    let loading_hint = if app.screen == Screen::Feed && app.feed_loading {
        " | Caricamento altri post..."
    } else {
        ""
    };

    let text = if status.is_empty() {
        hints.to_string()
    } else {
        format!("{hints} | {status}")
    };

    f.render_widget(Paragraph::new(text).style(Style::default().fg(MUTED)), area);
}

fn draw_feed(f: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = app
        .feed
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if i == app.selected_index {
                Style::default().fg(Color::Black).bg(ACCENT)
            } else {
                Style::default()
            };

            let preview: String = p.content.chars().take(120).collect();
            let title = p.title.clone().unwrap_or_default();

            let mut lines = Vec::new();

            lines.push(Line::from(Span::styled(
                format!("@{}", p.author_username),
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            )));

            if !title.is_empty() {
                lines.push(Line::from(Span::styled(
                    title,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::ITALIC),
                )));
            }

            lines.push(Line::from(preview));

            lines.push(Line::from(Span::styled(
                format!(
                    "{} replies · {} bookmarks",
                    p.replies_count, p.bookmarks_count
                ),
                Style::default().fg(MUTED),
            )));

            ListItem::new(lines).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Feed"))
        .highlight_style(Style::default().fg(Color::Black).bg(ACCENT));

    f.render_widget(list, area);
}

fn draw_notifications(f: &mut Frame, app: &App, area: Rect) {
    let it: Vec<ListItem> = app
        .notifications
        .iter()
        .map(|n| {
            let s = if n.read {
                Style::default().fg(MUTED)
            } else {
                Style::default().fg(ACCENT)
            };
            ListItem::new(format!(
                "{}  @{}",
                n.kind,
                n.actor_username.clone().unwrap_or_default()
            ))
            .style(s)
        })
        .collect();
    f.render_widget(
        List::new(it).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Notifications"),
        ),
        area,
    );
}

fn draw_cmail_list(f: &mut Frame, app: &App, area: Rect) {
    let it: Vec<ListItem> = app
        .cmail_conversations
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let s = if i == app.cmail_selected {
                Style::default().fg(Color::Black).bg(ACCENT)
            } else {
                Style::default()
            };
            ListItem::new(format!("@{}", c.other_user.username)).style(s)
        })
        .collect();
    f.render_widget(
        List::new(it).block(Block::default().borders(Borders::ALL).title("c-Mail")),
        area,
    );
}

fn draw_circ_list(f: &mut Frame, app: &App, area: Rect) {
    let it: Vec<ListItem> = app
        .circ_rooms
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let s = if i == app.circ_selected {
                Style::default().fg(Color::Black).bg(ACCENT)
            } else {
                Style::default()
            };
            ListItem::new(r.name.clone().unwrap_or_else(|| r.id.clone())).style(s)
        })
        .collect();
    f.render_widget(
        List::new(it).block(Block::default().borders(Borders::ALL).title("cIRC")),
        area,
    );
}

fn draw_bookmarks(f: &mut Frame, app: &App, area: Rect) {
    let it: Vec<ListItem> = app
        .bookmarks
        .iter()
        .map(|b| ListItem::new(b.post_id.clone().unwrap_or_default()))
        .collect();
    f.render_widget(
        List::new(it).block(Block::default().borders(Borders::ALL).title("Bookmarks")),
        area,
    );
}

fn draw_topics(f: &mut Frame, app: &App, area: Rect) {
    let it: Vec<ListItem> = app
        .topics
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let s = if i == app.topic_selected {
                Style::default().fg(Color::Black).bg(ACCENT)
            } else {
                Style::default()
            };
            ListItem::new(format!("#{}  [{}]", t.name, t.post_count.unwrap_or(0))).style(s)
        })
        .collect();
    f.render_widget(
        List::new(it).block(Block::default().borders(Borders::ALL).title("Topics")),
        area,
    );
}

fn draw_profile(f: &mut Frame, app: &App, area: Rect) {
    let Some(u) = &app.current_profile else {
        f.render_widget(
            Paragraph::new("Nessun profilo").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    };
    let lines = vec![
        Line::from(Span::styled(
            format!("@{}", u.username),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(u.display_name.clone().unwrap_or_default()),
        Line::from(u.bio.clone().unwrap_or_default()),
    ];
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Profile")),
        area,
    );
}

fn draw_journal(f: &mut Frame, app: &App, area: Rect) {
    let it: Vec<ListItem> = app
        .journal_notes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let s = if i == app.journal_selected {
                Style::default().fg(Color::Black).bg(ACCENT)
            } else {
                Style::default()
            };
            let preview: String = n.content.chars().take(60).collect();
            ListItem::new(format!("{} …", preview)).style(s)
        })
        .collect();
    f.render_widget(
        List::new(it).block(Block::default().borders(Borders::ALL).title("Journal")),
        area,
    );
}

fn draw_guilds(f: &mut Frame, app: &App, area: Rect) {
    let it: Vec<ListItem> = app
        .guilds
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let s = if i == app.guild_selected {
                Style::default().fg(Color::Black).bg(ACCENT)
            } else {
                Style::default()
            };
            ListItem::new(format!(
                "{}  @{}  [{}]",
                g.icon.clone().unwrap_or_default(),
                g.name,
                g.member_count.unwrap_or(0)
            ))
            .style(s)
        })
        .collect();
    f.render_widget(
        List::new(it).block(Block::default().borders(Borders::ALL).title("Guilds")),
        area,
    );
}

fn draw_jukebox(f: &mut Frame, _app: &App, area: Rect) {
    f.render_widget(
        Paragraph::new(
            "Jukebox: visualizza post con audioAttachment\n(placeholder — da implementare)",
        )
        .block(Block::default().borders(Borders::ALL).title("Jukebox")),
        area,
    );
}

fn draw_cmail_conversation(f: &mut Frame, app: &App, area: Rect) {
    let name = app
        .current_conversation
        .as_ref()
        .map(|c| c.other_user.username.clone())
        .unwrap_or_default();
    let lines: Vec<Line> = app
        .cmail_messages
        .iter()
        .map(|m| {
            Line::from(format!(
                "@{}: {}",
                m.sender_username.clone().unwrap_or_default(),
                m.content
            ))
        })
        .collect();
    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("c-Mail: @{}", name)),
        ),
        area,
    );
}

fn draw_circ_room(f: &mut Frame, app: &App, area: Rect) {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let room_name = app
        .current_room
        .as_ref()
        .and_then(|r| r.name.clone())
        .unwrap_or_else(|| "cIRC room".to_string());

    let lines: Vec<Line> = app
        .circ_messages
        .iter()
        .map(|m| {
            if m.deleted {
                Line::from(Span::styled("[eliminato]", Style::default().fg(MUTED)))
            } else if m.is_action {
                Line::from(Span::styled(
                    format!(
                        "* {} {}",
                        m.sender_username.clone().unwrap_or_default(),
                        m.content
                    ),
                    Style::default().fg(Color::Magenta),
                ))
            } else {
                Line::from(format!(
                    "@{}: {}",
                    m.sender_username.clone().unwrap_or_default(),
                    m.content
                ))
            }
        })
        .collect();

    let messages_block = Block::default()
        .borders(Borders::ALL)
        .title(format!("cIRC: {}", room_name));
    f.render_widget(Paragraph::new(lines).block(messages_block), v[0]);

    let input_title = if app.input_mode == InputMode::Compose {
        "Messaggio cIRC (Enter: invia, Esc: annulla)"
    } else {
        "Premi i per scrivere un messaggio"
    };

    let input_content = if app.input_mode == InputMode::Compose {
        app.compose_input.value()
    } else {
        ""
    };

    let input_block = Paragraph::new(input_content)
        .style(if app.input_mode == InputMode::Compose {
            Style::default().fg(ACCENT)
        } else {
            Style::default().fg(MUTED)
        })
        .block(Block::default().borders(Borders::ALL).title(input_title));

    f.render_widget(input_block, v[1]);
}

fn draw_topic_posts(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .current_topic_posts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let s = if i == app.selected_index {
                Style::default().fg(Color::Black).bg(ACCENT)
            } else {
                Style::default()
            };
            let preview: String = p.content.chars().take(70).collect();
            ListItem::new(preview).style(s)
        })
        .collect();
    f.render_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Topic Posts")),
        area,
    );
}

fn draw_journal_note(f: &mut Frame, app: &App, area: Rect) {
    let Some(n) = &app.current_note else {
        f.render_widget(
            Paragraph::new("Nessuna nota").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    };
    f.render_widget(
        Paragraph::new(n.content.clone())
            .block(Block::default().borders(Borders::ALL).title("Journal Note")),
        area,
    );
}

fn draw_guild_detail(f: &mut Frame, app: &App, area: Rect) {
    let Some(g) = &app.current_guild else {
        f.render_widget(
            Paragraph::new("Nessuna guild").block(Block::default().borders(Borders::ALL)),
            area,
        );
        return;
    };
    let lines = vec![
        Line::from(Span::styled(
            format!("{} {}", g.icon.clone().unwrap_or_default(), g.name),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "@{}",
            g.founder_username.clone().unwrap_or_default()
        )),
        Line::from(g.bio.clone().unwrap_or_default()),
        Line::from(format!("Membri: {}", g.member_count.unwrap_or(0))),
    ];
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Guild Detail")),
        area,
    );
}

fn centered(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}

fn draw_create_post_modal(f: &mut Frame, app: &App, area: Rect) {
    let bg = Color::Rgb(15, 15, 15);

    f.render_widget(Clear, area);

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .title("Create post (Tab: campo, Ctrl+S: pubblica, Esc: annulla)")
        .style(Style::default().bg(bg));

    let inner = modal_block.inner(area);

    f.render_widget(modal_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(inner);

    let title_style = if app.create_post_focus == CreatePostFocus::Title {
        Style::default().fg(ACCENT).bg(bg)
    } else {
        Style::default().fg(Color::White).bg(bg)
    };

    let content_style = if app.create_post_focus == CreatePostFocus::Content {
        Style::default().fg(ACCENT).bg(bg)
    } else {
        Style::default().fg(Color::White).bg(bg)
    };

    let title = Paragraph::new(app.create_post_title.as_str())
        .style(title_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Title")
                .style(Style::default().bg(bg)),
        );

    let content = Paragraph::new(app.create_post_content.as_str())
        .style(content_style)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Content")
                .style(Style::default().bg(bg)),
        );

    f.render_widget(title, chunks[0]);
    f.render_widget(content, chunks[1]);
}

fn draw_reply_modal(f: &mut Frame, app: &App, area: Rect) {
    let bg = Color::Rgb(15, 15, 15);

    f.render_widget(Clear, area);

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .title("Reply (Ctrl+S: invia, Esc: annulla)")
        .style(Style::default().bg(bg));

    let inner = modal_block.inner(area);

    f.render_widget(modal_block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(inner);

    let content = Paragraph::new(app.reply_content.as_str())
        .style(Style::default().fg(ACCENT).bg(bg))
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Message")
                .style(Style::default().bg(bg)),
        );

    let hint =
        Paragraph::new("Ctrl+S: invia | Esc: annulla").style(Style::default().fg(MUTED).bg(bg));

    f.render_widget(content, chunks[0]);
    f.render_widget(hint, chunks[1]);
}

fn draw_post_modal(f: &mut Frame, app: &App, area: Rect) {
    let Some(post) = &app.current_post else {
        return;
    };

    let bg = Color::Rgb(15, 15, 15);

    f.render_widget(Clear, area);

    let modal_block = Block::default()
        .borders(Borders::ALL)
        .title("Post (Esc: chiudi, r: reply)")
        .style(Style::default().bg(bg));

    f.render_widget(modal_block, area);

    let inner = area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 2,
    });

    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        format!("@{}", post.author_username),
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
    )));

    if let Some(title) = &post.title {
        lines.push(Line::from(Span::styled(
            title.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(post.content.clone()));
    lines.push(Line::from(""));

    if app.replies_loading {
        lines.push(Line::from(Span::styled(
            "Caricamento replies...",
            Style::default().fg(MUTED),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            format!("{} replies", app.current_replies.len()),
            Style::default().fg(MUTED),
        )));

        lines.push(Line::from(""));

        if app.current_replies.is_empty() {
            lines.push(Line::from(Span::styled(
                "Nessuna reply.",
                Style::default().fg(MUTED),
            )));
        } else {
            for reply in &app.current_replies {
                lines.push(Line::from(Span::styled(
                    format!("> @{}", reply.author_username.clone().unwrap_or_default()),
                    Style::default().fg(ACCENT),
                )));

                lines.push(Line::from(reply.content.clone()));
                lines.push(Line::from(""));
            }
        }
    }

    let text = Paragraph::new(lines)
        .style(Style::default().bg(bg))
        .wrap(Wrap { trim: false });

    f.render_widget(text, inner);
}

fn draw_overlay_backdrop(f: &mut Frame, area: Rect) {
    let backdrop = Block::default().style(Style::default().bg(Color::Rgb(15, 15, 15)));
    f.render_widget(backdrop, area);
}

fn draw_error_box(f: &mut Frame, app: &App, area: Rect) {
    let Some(message) = &app.error_message else {
        return;
    };

    if area.width < 10 || area.height < 3 {
        return;
    }

    let bg = Color::Rgb(15, 15, 15);
    let width = area.width.saturating_sub(4).min(90);
    let height = 3;

    let error_area = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + 1,
        width,
        height,
    };

    f.render_widget(Clear, error_area);

    let error_block = Block::default()
        .borders(Borders::ALL)
        .title("Errore")
        .border_style(Style::default().fg(Color::Red))
        .style(Style::default().bg(bg));

    let inner = error_block.inner(error_area);

    f.render_widget(error_block, error_area);

    let text = Paragraph::new(message.as_str())
        .style(Style::default().fg(Color::Red).bg(bg))
        .wrap(Wrap { trim: false });

    f.render_widget(text, inner);
}
