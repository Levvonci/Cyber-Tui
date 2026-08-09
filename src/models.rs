use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ApiWrap<T> {
    pub data: T,
}
#[derive(Debug, Deserialize)]
pub struct ApiList<T> {
    pub data: Vec<T>,
    #[allow(dead_code)]
    #[serde(default)]
    pub cursor: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct ApiErrorBody {
    pub error: ApiErrorDetail,
}
#[derive(Debug, Deserialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    #[serde(rename = "idToken")]
    pub id_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Post {
    #[serde(rename = "postId")]
    pub post_id: String,
    #[serde(rename = "authorUsername")]
    pub author_username: String,
    pub content: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(rename = "repliesCount", default)]
    pub replies_count: u32,
    #[serde(rename = "bookmarksCount", default)]
    pub bookmarks_count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Reply {
    #[serde(rename = "authorUsername", default)]
    pub author_username: Option<String>,
    pub content: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct User {
    #[serde(default)]
    pub username: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Watch {
    pub id: String,
    #[serde(rename = "postId")]
    pub post_id: String,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Bookmark {
    pub id: String,
    #[serde(rename = "postId", default)]
    pub post_id: Option<String>,
}
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Notification {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "actorUsername", default)]
    pub actor_username: Option<String>,
    #[serde(default)]
    pub read: bool,
}
#[derive(Debug, Deserialize, Clone)]
pub struct CmailConversationRef {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    #[serde(rename = "otherUser")]
    pub other_user: CmailUser,
}
#[derive(Debug, Deserialize, Clone)]
pub struct CmailUser {
    #[serde(default)]
    pub username: String,
}
#[derive(Debug, Deserialize, Clone)]
pub struct CmailMessage {
    #[serde(default)]
    pub content: String,
    #[serde(rename = "senderUsername", default)]
    pub sender_username: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct CircRoom {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct CircMessage {
    #[serde(default)]
    pub content: String,
    #[serde(rename = "senderUsername", default)]
    pub sender_username: Option<String>,
    #[serde(rename = "isAction", default)]
    pub is_action: bool,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Serialize)]
pub struct NewPost<'a> {
    pub content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<&'a str>,
    #[serde(rename = "isPublic")]
    pub is_public: bool,
    #[serde(rename = "isNSFW")]
    pub is_nsfw: bool,
}
#[derive(Debug, Serialize)]
pub struct NewReply<'a> {
    #[serde(rename = "postId")]
    pub post_id: &'a str,
    pub content: &'a str,
    #[serde(rename = "parentReplyId", skip_serializing_if = "Option::is_none")]
    pub parent_reply_id: Option<&'a str>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Topic {
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "postCount")]
    pub post_count: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Note {
    pub id: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Guild {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(rename = "founderUsername", default)]
    pub founder_username: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(rename = "bio", default)]
    pub bio: Option<String>,
    #[serde(rename = "memberCount", default)]
    pub member_count: Option<u32>,
}
