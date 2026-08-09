mod api;
mod app;
mod models;
mod session;
mod ui;
use crate::session::Session;
use anyhow::Result;
use app::{App, CreatePostFocus, InputMode, Screen};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};
use tui_input::backend::crossterm::EventHandler;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    if app.session.is_authenticated() {
        load_initial_data(&mut app).await;
    }
    let res = run(&mut terminal, &mut app).await;
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    res
}

async fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let mut last_feed_refresh = Instant::now();
    let mut last_circ_refresh = Instant::now();

    loop {
        poll_feed_load(app).await;
        poll_replies_load(app).await;

        terminal.draw(|f| ui::draw(f, app))?;

        if app.screen != Screen::Login
            && app.screen != Screen::Feed
            && app.input_mode == InputMode::Normal
            && !app.show_post_modal
            && !app.show_create_post_modal
            && !app.show_reply_modal
            && app.feed_load_task.is_none()
            && (app.screen != Screen::Feed || app.selected_index <= 2)
            && last_feed_refresh.elapsed() >= Duration::from_secs(8)
        {
            load_feed(app).await;
            refresh_unread(app).await;
            last_feed_refresh = Instant::now();
        }

        if app.screen == Screen::CircRoom
            && app.input_mode == InputMode::Normal
            && last_circ_refresh.elapsed() >= Duration::from_secs(2)
        {
            if let Some(room) = &app.current_room {
                let room_id = room.id.clone();
                load_circ(app, &room_id).await;
            }

            last_circ_refresh = Instant::now();
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if app.screen == Screen::Login {
                    handle_login(app, key).await;
                } else if app.input_mode == InputMode::Compose {
                    handle_compose(app, key).await;
                } else {
                    handle_main(app, key).await;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_login(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.should_quit = true;
        }

        KeyCode::Tab => {
            app.login_field_email_active = !app.login_field_email_active;
        }

        KeyCode::Enter => {
            let email = app.login_email.value().to_string();
            let password = app.login_password.value().to_string();

            if email.is_empty() || password.is_empty() {
                app.login_error = Some("Inserisci email e password".to_string());
                return;
            }

            app.login_error = None;

            let started = Instant::now();

            match app.api.login(&email, &password).await {
                Ok(()) => {
                    eprintln!("[perf] login completato in {:?}", started.elapsed());

                    let (id, rt) = app.api.snapshot_tokens();
                    app.session.id_token = id;
                    app.session.refresh_token = rt;
                    app.session.username = Some(email.clone());
                    let _ = app.session.save();

                    app.screen = Screen::Feed;
                    load_initial_data(app).await;
                    app.set_status("Login ok");
                }

                Err(e) => {
                    eprintln!("[perf] login fallito dopo {:?}: {e}", started.elapsed());

                    app.login_error = Some(format!("Errore login: {e}"));
                }
            }
        }

        _ => {
            let ev = Event::Key(key);

            if app.login_field_email_active {
                app.login_email.handle_event(&ev);
            } else {
                app.login_password.handle_event(&ev);
            }
        }
    }
}

async fn handle_compose(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.compose_input = tui_input::Input::default();
        }

        KeyCode::Enter => {
            let txt = app.compose_input.value().to_string();
            if txt.trim().is_empty() {
                app.input_mode = InputMode::Normal;
                return;
            }

            if app.screen == Screen::Feed {
                match app.api.create_post(&txt, None, vec![], true, false).await {
                    Ok(_) => {
                        app.set_status("Post pubblicato");
                        load_feed(app).await;
                    }
                    Err(e) => app.set_error(format!("Errore pubblicazione: {e}")),
                }
            }
            else if app.screen == Screen::CircRoom {
                if let Some(r) = &app.current_room {
                    let id = r.id.clone();
                    match app.api.circ_send(&id, &txt).await {
                        Ok(()) => load_circ(app, &id).await,
                        Err(e) => app.set_error(format!("Errore nel messaggio: {e}")),
                    }
                }
            }
            else if app.screen == Screen::CmailConversation {
                if let Some(c) = &app.current_conversation {
                    let id = c.conversation_id.clone();
                    match app.api.cmail_send(&id, &txt).await {
                        Ok(()) => load_cmail(app, &id).await,
                        Err(e) => app.set_status(format!("Errore nel messaggio: {e}")),
                    }
                }
            }

            app.input_mode = InputMode::Normal;
            app.compose_input = tui_input::Input::default();
        }

        _ => {
            app.compose_input.handle_event(&Event::Key(key));
        }
    }
}

async fn handle_create_post_modal(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.show_create_post_modal = false;
            app.create_post_title.clear();
            app.create_post_content.clear();
            app.create_post_focus = CreatePostFocus::Content;
        }

        KeyCode::Tab => {
            app.create_post_focus = match app.create_post_focus {
                CreatePostFocus::Title => CreatePostFocus::Content,
                CreatePostFocus::Content => CreatePostFocus::Title,
            };
        }

        KeyCode::Backspace => match app.create_post_focus {
            CreatePostFocus::Title => {
                app.create_post_title.pop();
            }
            CreatePostFocus::Content => {
                app.create_post_content.pop();
            }
        },

        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let content = app.create_post_content.trim().to_string();

            if content.is_empty() {
                app.set_error("Il contenuto non può essere vuoto");
                return;
            }

            let title_value = app.create_post_title.trim().to_string();
            let title = if title_value.is_empty() {
                None
            } else {
                Some(title_value.as_str())
            };

            match app
                .api
                .create_post(&content, title, vec![], true, false)
                .await
            {
                Ok(_) => {
                    app.show_create_post_modal = false;
                    app.create_post_title.clear();
                    app.create_post_content.clear();
                    app.create_post_focus = CreatePostFocus::Content;
                    app.set_status("Post pubblicato");
                    load_feed(app).await;
                }
                Err(e) => {
                    app.set_error(format!("Errore pubblicazione: {e}"));
                }
            }
        }

        KeyCode::Char(c) => match app.create_post_focus {
            CreatePostFocus::Title => {
                app.create_post_title.push(c);
            }
            CreatePostFocus::Content => {
                app.create_post_content.push(c);
            }
        },

        _ => {}
    }
}

async fn handle_reply_modal(app: &mut App, key: crossterm::event::KeyEvent) {
    use crossterm::event::{KeyCode, KeyModifiers};

    if key.code == KeyCode::Esc {
        app.show_reply_modal = false;
        app.reply_content.clear();
        return;
    }

    let submit = key.code == KeyCode::Enter
        || (key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL));

    if submit {
        let content = app.reply_content.trim().to_string();

        if content.is_empty() {
            return;
        }

        let Some(post) = app.current_post.as_ref() else {
            return;
        };

        let pid = post.post_id.clone();

        match app.api.create_reply(&pid, &content, None).await {
            Ok(()) => {
                app.show_reply_modal = false;
                app.reply_content.clear();
                app.set_status("Reply inviata");

                start_replies_load(app, pid);
            }

            Err(e) => {
                app.set_error(format!("Errore reply: {e}"));
            }
        }

        return;
    }

    match key.code {
        KeyCode::Char(c) => {
            app.reply_content.push(c);
        }

        KeyCode::Backspace => {
            app.reply_content.pop();
        }

        _ => {}
    }
}

async fn handle_main(app: &mut App, key: crossterm::event::KeyEvent) {
    if app.show_create_post_modal {
        handle_create_post_modal(app, key).await;
        return;
    }

    if app.show_reply_modal {
        handle_reply_modal(app, key).await;
        return;
    }

    if app.show_post_modal {
        match key.code {
            KeyCode::Esc => {
                app.show_post_modal = false;
            }

            KeyCode::Char('r') => {
                app.show_reply_modal = true;
                app.reply_content.clear();
                app.set_status("Scrivi reply (Ctrl+s: invia, Esc: annulla)");
            }

            _ => {}
        }

        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('s') => app.toggle_sidebar(),
        KeyCode::Char('j') | KeyCode::Down => {
            if app.sidebar_visible {
                app.move_sidebar_down();
            } else {
                app.move_selection_down();

                if app.screen == Screen::Feed
                    && app.selected_index + 3 >= app.feed.len()
                    && !app.feed_loading
                    && app.feed_cursor.is_some()
                {
                    start_next_feed_page_load(app);
                }
            }
        }

        KeyCode::Char('k') | KeyCode::Up => {
            if app.sidebar_visible {
                app.move_sidebar_up();
            } else {
                app.move_selection_up();
            }
        }

        KeyCode::Enter => {
            if app.sidebar_visible {
                app.select_sidebar_item();
                match app.screen {
                    Screen::Feed => load_feed(app).await,
                    Screen::Notifications => load_notifications(app).await,
                    Screen::CmailList => load_cmail_list(app).await,
                    Screen::CircList => load_circ_list(app).await,
                    Screen::Bookmarks => load_bookmarks(app).await,
                    Screen::Topics => load_topics(app).await,
                    Screen::Profile => load_profile(app).await,
                    Screen::Journal => load_journal(app).await,
                    Screen::Guilds => load_guilds(app).await,
                    Screen::Jukebox => load_feed(app).await,
                    _ => {}
                }
            } else {
                match app.screen {
                    Screen::Feed => {
                        if let Some(p) = app.selected_post().cloned() {
                            let pid = p.post_id.clone();

                            app.current_post = Some(p);
                            app.current_replies.clear();
                            app.replies_loading = true;
                            app.show_post_modal = true;

                            start_replies_load(app, pid);
                        }
                    }
                    Screen::CmailList => {
                        if let Some(c) = app.cmail_conversations.get(app.cmail_selected).cloned() {
                            app.current_conversation = Some(c.clone());
                            load_cmail(app, &c.conversation_id).await;
                            app.screen = Screen::CmailConversation;
                        }
                    }
                    Screen::CircList => {
                        if let Some(r) = app.circ_rooms.get(app.circ_selected).cloned() {
                            app.current_room = Some(r.clone());
                            load_circ(app, &r.id).await;
                            app.screen = Screen::CircRoom;
                        }
                    }
                    Screen::Topics => {
                        if let Some(t) = app.topics.get(app.topic_selected).cloned() {
                            load_topic_posts(app, &t.slug).await;
                            app.screen = Screen::TopicPosts;
                        }
                    }
                    Screen::Guilds => {
                        if let Some(g) = app.guilds.get(app.guild_selected).cloned() {
                            app.current_guild = Some(g.clone());
                            load_guild_posts(app, &g.slug).await;
                            app.screen = Screen::GuildDetail;
                        }
                    }
                    Screen::Journal => {
                        if let Some(n) = app.journal_notes.get(app.journal_selected).cloned() {
                            app.current_note = Some(n.clone());
                            app.screen = Screen::JournalNote;
                        }
                    }
                    _ => {}
                }
            }
        }

        KeyCode::Esc => {
            if app.sidebar_visible {
                app.sidebar_visible = false;
            } else if app.show_post_modal {
                app.show_post_modal = false;
            } else {
                match app.screen {
                    Screen::Profile
                    | Screen::TopicPosts
                    | Screen::JournalNote
                    | Screen::GuildDetail => app.screen = Screen::Feed,
                    Screen::CmailConversation => app.screen = Screen::CmailList,
                    Screen::CircRoom => app.screen = Screen::CircList,
                    _ => {}
                }
            }
        }

        KeyCode::Char('c') if app.screen == Screen::Feed => {
            app.show_create_post_modal = true;
            app.create_post_title.clear();
            app.create_post_content.clear();
            app.create_post_focus = CreatePostFocus::Content;
            app.set_status("Nuovo post");
        }

        KeyCode::Char('i') if app.screen == Screen::CircRoom => {
            app.input_mode = InputMode::Compose;
            app.compose_input = tui_input::Input::default();
            app.set_status("Scrivi messaggio cIRC (Enter: invia, Esc: annulla)");
        }

        KeyCode::Char('w') if app.screen == Screen::Feed => {
            if let Some(p) = app.selected_post().cloned() {
                match app.api.watch(&p.post_id).await {
                    Ok(()) => app.set_status("Watch aggiunto"),
                    Err(e) => app.set_error(format!("Errore: {e}")),
                }
            }
        }

        KeyCode::Char('x') if app.screen == Screen::Feed => {
            if let Some(p) = app.selected_post().cloned() {
                match app.api.add_bookmark_post(&p.post_id).await {
                    Ok(()) => app.set_status("Bookmark aggiunto"),
                    Err(e) => app.set_error(format!("Errore: {e}")),
                }
            }
        }

        _ => {}
    }
}

fn is_unauthorized(e: &anyhow::Error) -> bool {
    let s = e.to_string();
    s.contains("UNAUTHORIZED") || s.contains("401")
}

async fn try_refresh_or_logout(app: &mut App) -> bool {
    match app.api.refresh().await {
        Ok(()) => {
            app.persist_tokens();
            app.set_status("Sessione rinnovata");
            true
        }
        Err(_) => {
            let _ = Session::clear();
            app.session = Session::default();
            app.screen = Screen::Login;
            app.set_status("Sessione scaduta, effettua di nuovo il login");
            false
        }
    }
}

async fn load_feed(app: &mut App) {
    app.selected_index = 0;

    app.feed_loading = false;
    app.feed_cursor = None;
    let started = Instant::now();

    match app.api.get_posts(None).await {
        Ok(posts) => {
            eprintln!("[perf] load_feed completato in {:?}", started.elapsed());

            app.feed = posts.data;
            app.feed_cursor = posts.cursor;
        }

        Err(e) => {
            if is_unauthorized(&e) {
                if try_refresh_or_logout(app).await {
                    match app.api.get_posts(None).await {
                        Ok(posts) => {
                            app.feed = posts.data;
                            app.feed_cursor = posts.cursor;
                        }

                        Err(e2) => {
                            app.set_error(format!("Errore feed: {e2}"));
                        }
                    }
                }
            } else {
                app.set_error(format!("Errore feed: {e}"));
            }
        }
    }
}

async fn load_initial_data(app: &mut App) {
    app.selected_index = 0;

    app.feed_loading = false;
    app.feed_cursor = None;

    let started = Instant::now();

    match app.api.get_posts(None).await {
        Ok(posts) => {
            app.feed = posts.data;
            app.feed_cursor = posts.cursor;

            eprintln!(
                "[feed] caricati {} post, cursor iniziale={:?}, durata={:?}",
                app.feed.len(),
                app.feed_cursor,
                started.elapsed()
            );
        }

        Err(e) => {
            app.set_error(format!("Errore feed: {e}"));
        }
    }
}

async fn load_notifications(app: &mut App) {
    match app.api.get_notifications().await {
        Ok(l) => app.notifications = l.data,
        Err(e) => app.set_error(format!("Errore notifiche: {e}")),
    }
    let _ = app.api.mark_all_read().await;
    refresh_unread(app).await;
}

async fn load_cmail_list(app: &mut App) {
    match app.api.cmail_list().await {
        Ok(l) => app.cmail_conversations = l.data,
        Err(e) => app.set_error(format!("Errore cmail: {e}")),
    }
}

async fn load_cmail(app: &mut App, cid: &str) {
    match app.api.cmail_messages(cid).await {
        Ok(l) => app.cmail_messages = l.data,
        Err(e) => app.set_error(format!("Errore messaggi: {e}")),
    }
    let _ = app.api.cmail_mark_read(cid).await;
}

async fn load_circ_list(app: &mut App) {
    match app.api.circ_rooms().await {
        Ok(l) => app.circ_rooms = l.data,
        Err(e) => app.set_error(format!("Errore circ: {e}")),
    }
}

async fn load_circ(app: &mut App, rid: &str) {
    match app.api.circ_messages(rid).await {
        Ok(l) => app.circ_messages = l.data,
        Err(e) => {
            if is_unauthorized(&e) {
                if try_refresh_or_logout(app).await {
                    if let Ok(l) = app.api.circ_messages(rid).await {
                        app.circ_messages = l.data;
                    }
                }
            } else {
                app.set_error(format!("Errore messaggi: {e}"));
            }
        }
    }
    let _ = app.api.circ_presence_heartbeat(rid).await;
}

async fn load_bookmarks(app: &mut App) {
    match app.api.get_bookmarks().await {
        Ok(l) => app.bookmarks = l.data,
        Err(e) => app.set_error(format!("Errore bookmarks: {e}")),
    }
}

async fn load_topics(app: &mut App) {
    match app.api.get_topics().await {
        Ok(l) => app.topics = l.data,
        Err(e) => app.set_error(format!("Errore topics: {e}")),
    }
}

async fn load_topic_posts(app: &mut App, slug: &str) {
    match app.api.get_topic_posts(slug).await {
        Ok(l) => app.current_topic_posts = l.data,
        Err(e) => app.set_error(format!("Errore post topic: {e}")),
    }
}

async fn load_profile(app: &mut App) {
    match app.api.get_me().await {
        Ok(u) => {
            app.current_profile = Some(u.clone());
            match app.api.get_user_posts(&u.username).await {
                Ok(l) => app.current_profile_posts = l.data,
                Err(e) => app.set_error(format!("Errore entry: {e}")),
            }
        }
        Err(e) => app.set_error(format!("Errore profilo: {e}")),
    }
}

async fn load_journal(app: &mut App) {
    match app.api.get_notes().await {
        Ok(l) => app.journal_notes = l.data,
        Err(e) => app.set_error(format!("Errore journal: {e}")),
    }
}

async fn load_guilds(app: &mut App) {
    match app.api.get_guilds().await {
        Ok(l) => app.guilds = l.data,
        Err(e) => app.set_error(format!("Errore guilds: {e}")),
    }
}

async fn load_guild_posts(app: &mut App, slug: &str) {
    match app.api.get_guild_posts(slug).await {
        Ok(l) => app.current_guild_posts = l.data,
        Err(e) => app.set_error(format!("Errore post guild: {e}")),
    }
}

async fn refresh_unread(app: &mut App) {
    match app.api.unread_count().await {
        Ok(c) => app.unread_count = c,
        Err(e) => {
            if is_unauthorized(&e) {
                let _ = try_refresh_or_logout(app).await;
            }
        }
    }
}

fn start_replies_load(app: &mut App, pid: String) {
    if let Some(task) = app.reply_load_task.take() {
        task.abort();
    }

    app.replies_loading = true;
    app.current_replies.clear();

    let api = app.api.clone();

    app.reply_load_task = Some(tokio::spawn(async move { api.get_replies(&pid).await }));
}

async fn poll_replies_load(app: &mut App) {
    let finished = app
        .reply_load_task
        .as_ref()
        .map(|task| task.is_finished())
        .unwrap_or(false);

    if !finished {
        return;
    }

    let Some(task) = app.reply_load_task.take() else {
        return;
    };

    match task.await {
        Ok(Ok(replies)) => {
            app.current_replies = replies.data;
            app.replies_loading = false;
        }

        Ok(Err(e)) => {
            app.replies_loading = false;
            app.set_error(format!("Errore risposte: {e}"));
        }

        Err(e) => {
            app.replies_loading = false;
            app.set_error(format!("Errore task replies: {e}"));
        }
    }
}

fn start_next_feed_page_load(app: &mut App) {
    if app.feed_loading {
        return;
    }

    let Some(cursor) = app.feed_cursor.clone() else {
        return;
    };

    if app.feed_last_requested_cursor.as_deref() == Some(cursor.as_str()) {
        return;
    }

    app.feed_loading = true;
    app.feed_last_requested_cursor = Some(cursor.clone());

    let api = app.api.clone();

    app.feed_load_task = Some(tokio::spawn(
        async move { api.get_posts(Some(&cursor)).await },
    ));
}

async fn poll_feed_load(app: &mut App) {
    eprintln!("[feed] lunghezza feed dopo extend = {}", app.feed.len());
    let finished = app
        .feed_load_task
        .as_ref()
        .map(|task| task.is_finished())
        .unwrap_or(false);

    if !finished {
        return;
    }

    let Some(task) = app.feed_load_task.take() else {
        return;
    };

    match task.await {
        Ok(Ok(posts)) => {
            eprintln!(
                "[feed] pagina ricevuta: {} post, nuovo cursor={:?}",
                posts.data.len(),
                posts.cursor
            );

            app.feed.extend(posts.data);
            app.feed_cursor = posts.cursor;
            app.feed_loading = false;
        }

        Ok(Err(e)) => {
            eprintln!("[feed] errore pagina successiva: {e}");
            app.feed_loading = false;
            app.set_error(format!("Errore caricamento altri post: {e}"));
        }

        Err(e) => {
            eprintln!("[feed] task fallita: {e}");
            app.feed_loading = false;
            app.set_error(format!("Errore task feed: {e}"));
        }
    }
}
