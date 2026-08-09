use crate::api::ApiClient;
use crate::models::*;
use crate::session::Session;
use tui_input::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Login,
    Feed,
    Notifications,
    CmailList,
    CmailConversation,
    CircList,
    CircRoom,
    Bookmarks,
    Topics,
    TopicPosts,
    Profile,
    Journal,
    JournalNote,
    Guilds,
    GuildDetail,
    Jukebox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Compose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatePostFocus {
    Title,
    Content,
}

pub struct App {
    
    // Navigazione
    pub screen: Screen,
    pub sidebar_visible: bool,
    pub sidebar_selected: usize,

    // Auth
    pub api: ApiClient,
    pub session: Session,

    //Login
    pub login_email: Input,
    pub login_password: Input,
    pub login_error: Option<String>,
    pub login_field_email_active: bool,

    // Feed
    pub feed: Vec<Post>,
    pub selected_index: usize,
    pub current_post: Option<Post>,
    pub current_replies: Vec<Reply>,
    pub feed_cursor: Option<String>,
    pub feed_loading: bool,
    pub feed_load_task: Option<tokio::task::JoinHandle<anyhow::Result<ApiList<Post>>>>,

    // Notifications
    pub notifications: Vec<Notification>,
    pub unread_count: u32,

    // C-Mail
    pub cmail_conversations: Vec<CmailConversationRef>,
    pub cmail_selected: usize,
    pub current_conversation: Option<CmailConversationRef>,
    pub cmail_messages: Vec<CmailMessage>,

    // cIRC
    pub circ_rooms: Vec<CircRoom>,
    pub circ_selected: usize,
    pub current_room: Option<CircRoom>,
    pub circ_messages: Vec<CircMessage>,

    // Bookmarks
    pub bookmarks: Vec<Bookmark>,

    // Topics
    pub topics: Vec<Topic>,
    pub topic_selected: usize,
    pub current_topic_posts: Vec<Post>,

    // Profile
    pub current_profile: Option<User>,
    pub current_profile_posts: Vec<Post>,

    // Journal
    pub journal_notes: Vec<Note>,
    pub journal_selected: usize,
    pub current_note: Option<Note>,

    // Guilds
    pub guilds: Vec<Guild>,
    pub guild_selected: usize,
    pub current_guild: Option<Guild>,
    pub current_guild_posts: Vec<Post>,

    // UI
    pub input_mode: InputMode,
    pub compose_input: Input,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub should_quit: bool,

    // Modal
    pub show_post_modal: bool,

    pub show_create_post_modal: bool,
    pub create_post_title: String,
    pub create_post_content: String,
    pub create_post_focus: CreatePostFocus,

    pub show_reply_modal: bool,
    pub reply_content: String,

    pub replies_loading: bool,
    pub reply_load_task: Option<tokio::task::JoinHandle<anyhow::Result<ApiList<Reply>>>>,

    pub feed_loading_cursor: Option<String>,
    pub feed_last_requested_cursor: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let session = Session::load().unwrap_or_default();
        let api = match (&session.id_token, &session.refresh_token) {
            (Some(id), rt) => ApiClient::with_tokens(id.clone(), rt.clone()),
            _ => ApiClient::new(),
        };

        let initial_screen = if session.is_authenticated() {
            Screen::Feed
        } else {
            Screen::Login
        };

        Self {
            screen: initial_screen,
            sidebar_visible: false,
            sidebar_selected: 0,
            api,
            session,

            login_email: Input::default(),
            login_password: Input::default(),
            login_error: None,
            login_field_email_active: true,

            feed: Vec::new(),
            selected_index: 0,
            current_post: None,
            current_replies: Vec::new(),
            notifications: Vec::new(),
            unread_count: 0,
            cmail_conversations: Vec::new(),
            cmail_selected: 0,
            current_conversation: None,
            cmail_messages: Vec::new(),
            circ_rooms: Vec::new(),
            circ_selected: 0,
            current_room: None,
            circ_messages: Vec::new(),
            bookmarks: Vec::new(),
            topics: Vec::new(),
            topic_selected: 0,
            current_topic_posts: Vec::new(),
            current_profile: None,
            current_profile_posts: Vec::new(),
            journal_notes: Vec::new(),
            journal_selected: 0,
            current_note: None,
            guilds: Vec::new(),
            guild_selected: 0,
            current_guild: None,
            current_guild_posts: Vec::new(),
            input_mode: InputMode::Normal,
            compose_input: Input::default(),
            status_message: None,
            should_quit: false,
            show_post_modal: false,
            show_create_post_modal: false,
            create_post_title: String::new(),
            create_post_content: String::new(),
            create_post_focus: CreatePostFocus::Content,
            show_reply_modal: false,
            reply_content: String::new(),
            replies_loading: false,
            reply_load_task: None,
            error_message: None,
            feed_cursor: None,
            feed_loading: false,
            feed_load_task: None,
            feed_loading_cursor: None,
            feed_last_requested_cursor: None,
        }
    }
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.error_message = None;
        self.status_message = Some(msg.into());
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
        self.status_message = None;
    }

    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    pub fn move_sidebar_down(&mut self) {
        let len = 10;
        self.sidebar_selected = (self.sidebar_selected + 1).min(len - 1);
    }

    pub fn move_sidebar_up(&mut self) {
        if self.sidebar_selected > 0 {
            self.sidebar_selected -= 1;
        }
    }

    pub fn select_sidebar_item(&mut self) {
        match self.sidebar_selected {
            0 => self.screen = Screen::Feed,
            1 => self.screen = Screen::Notifications,
            2 => self.screen = Screen::CmailList,
            3 => self.screen = Screen::CircList,
            4 => self.screen = Screen::Bookmarks,
            5 => self.screen = Screen::Topics,
            6 => self.screen = Screen::Profile,
            7 => self.screen = Screen::Journal,
            8 => self.screen = Screen::Guilds,
            9 => self.screen = Screen::Jukebox,
            _ => {}
        }
        self.sidebar_visible = false;
    }

    pub fn move_selection_down(&mut self) {
        let len = self.active_list_len();
        if len == 0 {
            return;
        }
        match self.screen {
            Screen::CmailList => self.cmail_selected = (self.cmail_selected + 1).min(len - 1),
            Screen::CircList => self.circ_selected = (self.circ_selected + 1).min(len - 1),
            Screen::Topics => self.topic_selected = (self.topic_selected + 1).min(len - 1),
            Screen::Journal => self.journal_selected = (self.journal_selected + 1).min(len - 1),
            Screen::Guilds => self.guild_selected = (self.guild_selected + 1).min(len - 1),
            _ => self.selected_index = (self.selected_index + 1).min(len - 1),
        }
    }

    pub fn move_selection_up(&mut self) {
        match self.screen {
            Screen::CmailList => {
                if self.cmail_selected > 0 {
                    self.cmail_selected -= 1;
                }
            }
            Screen::CircList => {
                if self.circ_selected > 0 {
                    self.circ_selected -= 1;
                }
            }
            Screen::Topics => {
                if self.topic_selected > 0 {
                    self.topic_selected -= 1;
                }
            }
            Screen::Journal => {
                if self.journal_selected > 0 {
                    self.journal_selected -= 1;
                }
            }
            Screen::Guilds => {
                if self.guild_selected > 0 {
                    self.guild_selected -= 1;
                }
            }
            _ => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
        }
    }

    pub fn selected_post(&self) -> Option<&Post> {
        self.feed.get(self.selected_index)
    }

    #[allow(dead_code)]
    pub fn logout(&mut self) {
        let _ = Session::clear();

        if let Some(task) = self.reply_load_task.take() {
            task.abort();
        }

        self.feed_loading_cursor = None;
        self.feed_loading = false;
        self.feed.clear();
        self.session = Session::default();
        self.api = ApiClient::new();

        self.screen = Screen::Login;
        self.notifications.clear();
        self.cmail_conversations.clear();
        self.circ_rooms.clear();
        self.bookmarks.clear();
        self.topics.clear();
        self.journal_notes.clear();
        self.guilds.clear();

        self.current_post = None;
        self.current_replies.clear();
        self.replies_loading = false;
        self.show_post_modal = false;
        self.show_reply_modal = false;
        self.reply_content.clear();

        self.set_status("Disconnesso");
    }

    pub fn persist_tokens(&mut self) {
        let (id, rt) = self.api.snapshot_tokens();
        self.session.id_token = id;
        self.session.refresh_token = rt;
        let _ = self.session.save();
    }

    fn active_list_len(&self) -> usize {
        match self.screen {
            Screen::Feed => self.feed.len(),
            Screen::Notifications => self.notifications.len(),
            Screen::CmailList => self.cmail_conversations.len(),
            Screen::CircList => self.circ_rooms.len(),
            Screen::Bookmarks => self.bookmarks.len(),
            Screen::Topics => self.topics.len(),
            Screen::Profile => self.current_profile_posts.len(),
            Screen::Journal => self.journal_notes.len(),
            Screen::Guilds => self.guilds.len(),
            Screen::Jukebox => self.feed.iter().filter(|_p| false).count(),
            _ => 0,
        }
    }
}
