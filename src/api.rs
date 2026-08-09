use crate::models::*;
use anyhow::{anyhow, Result};
use reqwest::{Client, Method, StatusCode};
use serde::Serialize;
use std::time::Duration;

const BASE_URL: &str = "https://api.cyberspace.online";

#[derive(Debug, Clone)]
pub struct ApiClient {
    http: Client,
    id_token: Option<String>,
    refresh_token: Option<String>,
}

#[allow(dead_code)]
impl ApiClient {
    pub fn new() -> Self {
        Self {
            http: Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(15))
                .build()
                .expect("client"),
            id_token: None,
            refresh_token: None,
        }
    }

    pub fn with_tokens(id: String, rt: Option<String>) -> Self {
        let mut c = Self::new();
        c.id_token = Some(id);
        c.refresh_token = rt;
        c
    }

    fn req(&self, m: Method, p: &str) -> reqwest::RequestBuilder {
        let u = format!("{BASE_URL}{p}");
        let mut r = self.http.request(m, u);
        if let Some(t) = &self.id_token {
            r = r.bearer_auth(t);
        }
        r
    }

    async fn err(resp: reqwest::Response) -> anyhow::Error {
        let s = resp.status();
        match resp.json::<ApiErrorBody>().await {
            Ok(b) => anyhow!("{} ({}): {}", b.error.code, s, b.error.message),
            Err(_) => anyhow!("HTTP {s}"),
        }
    }

    pub async fn login(&mut self, email: &str, pass: &str) -> Result<()> {
        #[derive(Serialize)]
        struct B<'a> {
            email: &'a str,
            password: &'a str,
        }
        let r = self
            .http
            .post(format!("{BASE_URL}/v1/auth/login"))
            .json(&B {
                email,
                password: pass,
            })
            .send()
            .await?;
        if r.status() != StatusCode::OK {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<LoginData> = r.json().await?;
        self.id_token = Some(w.data.id_token);
        self.refresh_token = Some(w.data.refresh_token);
        Ok(())
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let refresh_token = self
            .refresh_token
            .clone()
            .ok_or_else(|| anyhow!("Nessun refresh token disponibile"))?;

        #[derive(Serialize)]
        struct RefreshBody<'a> {
            #[serde(rename = "refreshToken")]
            refresh_token: &'a str,
        }

        let response = self
            .http
            .post(format!("{BASE_URL}/v1/auth/refresh"))
            .json(&RefreshBody {
                refresh_token: &refresh_token,
            })
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            return Err(Self::err(response).await);
        }

        let wrapper: ApiWrap<LoginData> = response.json().await?;

        self.id_token = Some(wrapper.data.id_token);
        self.refresh_token = Some(wrapper.data.refresh_token);

        Ok(())
    }

    pub fn snapshot_tokens(&self) -> (Option<String>, Option<String>) {
        (self.id_token.clone(), self.refresh_token.clone())
    }

    pub async fn get_posts(&self, cursor: Option<&str>) -> Result<ApiList<Post>> {
        let path = match cursor {
            Some(cursor) => format!("/v1/posts?limit=20&cursor={cursor}"),
            None => "/v1/posts?limit=20".to_string(),
        };

        let r = self.req(Method::GET, &path).send().await?;

        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }

        Ok(r.json().await?)
    }

    pub async fn get_post(&self, id: &str) -> Result<Post> {
        let r = self
            .req(Method::GET, &format!("/v1/posts/{id}"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<Post> = r.json().await?;
        Ok(w.data)
    }

    pub async fn get_replies(&self, pid: &str) -> Result<ApiList<Reply>> {
        let r = self
            .req(Method::GET, &format!("/v1/posts/{pid}/replies?limit=20"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }

    pub async fn create_post(
        &self,
        content: &str,
        title: Option<&str>,
        topics: Vec<&str>,
        is_public: bool,
        is_nsfw: bool,
    ) -> Result<()> {
        let body = NewPost {
            content,
            title,
            topics,
            is_public,
            is_nsfw,
        };

        let response = self
            .req(Method::POST, "/v1/posts")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::err(response).await);
        }

        Ok(())
    }

    pub async fn create_reply(&self, pid: &str, content: &str, parent: Option<&str>) -> Result<()> {
        let body = NewReply {
            post_id: pid,
            content,
            parent_reply_id: parent,
        };

        let response = self
            .req(Method::POST, "/v1/replies")
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Self::err(response).await);
        }

        Ok(())
    }

    pub async fn watch(&self, pid: &str) -> Result<()> {
        let r = self
            .req(Method::POST, &format!("/v1/posts/{pid}/watch"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }
    pub async fn get_watches(&self) -> Result<ApiList<Watch>> {
        let r = self.req(Method::GET, "/v1/watches?limit=20").send().await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }
    pub async fn get_bookmarks(&self) -> Result<ApiList<Bookmark>> {
        let r = self
            .req(Method::GET, "/v1/bookmarks?limit=20")
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }
    pub async fn add_bookmark_post(&self, pid: &str) -> Result<()> {
        #[derive(Serialize)]
        struct B<'a> {
            #[serde(rename = "postId")]
            pid: &'a str,
            #[serde(rename = "type")]
            k: &'a str,
        }
        let r = self
            .req(Method::POST, "/v1/bookmarks")
            .json(&B { pid, k: "post" })
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }

    pub async fn get_notifications(&self) -> Result<ApiList<Notification>> {
        let r = self
            .req(Method::GET, "/v1/notifications?limit=20")
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }
    pub async fn unread_count(&self) -> Result<u32> {
        #[derive(serde::Deserialize)]
        struct C {
            count: u32,
        }
        let r = self
            .req(Method::GET, "/v1/notifications/unread-count")
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<C> = r.json().await?;
        Ok(w.data.count)
    }
    pub async fn mark_all_read(&self) -> Result<()> {
        let r = self
            .req(Method::POST, "/v1/notifications/read-all")
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }

    pub async fn cmail_list(&self) -> Result<ApiList<CmailConversationRef>> {
        let r = self.req(Method::GET, "/v1/cmail").send().await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }
    pub async fn cmail_messages(&self, cid: &str) -> Result<ApiList<CmailMessage>> {
        let r = self
            .req(Method::GET, &format!("/v1/cmail/{cid}?limit=50"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }
    pub async fn cmail_send(&self, cid: &str, content: &str) -> Result<()> {
        #[derive(Serialize)]
        struct B<'a> {
            content: &'a str,
        }
        let r = self
            .req(Method::POST, &format!("/v1/cmail/{cid}"))
            .json(&B { content })
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }
    pub async fn cmail_mark_read(&self, cid: &str) -> Result<()> {
        let r = self
            .req(Method::POST, &format!("/v1/cmail/{cid}/read"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }

    pub async fn circ_rooms(&self) -> Result<ApiList<CircRoom>> {
        let r = self.req(Method::GET, "/v1/circ").send().await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }
    pub async fn circ_messages(&self, rid: &str) -> Result<ApiList<CircMessage>> {
        let r = self
            .req(Method::GET, &format!("/v1/circ/{rid}?limit=50"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }
    pub async fn circ_send(&self, rid: &str, content: &str) -> Result<()> {
        #[derive(Serialize)]
        struct B<'a> {
            content: &'a str,
        }
        let r = self
            .req(Method::POST, &format!("/v1/circ/{rid}"))
            .json(&B { content })
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }
    pub async fn circ_presence_heartbeat(&self, rid: &str) -> Result<()> {
        #[derive(Serialize)]
        struct B {
            #[serde(rename = "lastActivity")]
            last: i64,
        }
        let now = chrono::Utc::now().timestamp_millis();
        let r = self
            .req(Method::POST, &format!("/v1/circ/{rid}/presence"))
            .json(&B { last: now })
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }

    pub async fn get_me(&self) -> Result<User> {
        let r = self.req(Method::GET, "/v1/users/me").send().await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<User> = r.json().await?;
        Ok(w.data)
    }
    pub async fn get_user_posts(&self, u: &str) -> Result<ApiList<Post>> {
        let r = self
            .req(Method::GET, &format!("/v1/users/{u}/posts?limit=20"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }

    // Topics
    pub async fn get_topics(&self) -> Result<ApiList<Topic>> {
        let r = self.req(Method::GET, "/v1/topics").send().await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }

    pub async fn get_topic_posts(&self, slug: &str) -> Result<ApiList<Post>> {
        let r = self
            .req(Method::GET, &format!("/v1/topics/{}/posts?limit=20", slug))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }

    // Journal / Notes
    pub async fn get_notes(&self) -> Result<ApiList<Note>> {
        let r = self.req(Method::GET, "/v1/notes?limit=20").send().await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }

    pub async fn get_note(&self, id: &str) -> Result<Note> {
        let r = self
            .req(Method::GET, &format!("/v1/notes/{id}"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<Note> = r.json().await?;
        Ok(w.data)
    }

    pub async fn create_note(&self, content: &str, topics: Vec<&str>) -> Result<Note> {
        #[derive(Serialize)]
        struct B<'a> {
            content: &'a str,
            topics: Vec<&'a str>,
        }
        let r = self
            .req(Method::POST, "/v1/notes")
            .json(&B { content, topics })
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<Note> = r.json().await?;
        Ok(w.data)
    }

    pub async fn update_note(&self, id: &str, content: &str, topics: Vec<&str>) -> Result<Note> {
        #[derive(Serialize)]
        struct B<'a> {
            content: &'a str,
            topics: Vec<&'a str>,
        }
        let r = self
            .req(Method::PATCH, &format!("/v1/notes/{id}"))
            .json(&B { content, topics })
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<Note> = r.json().await?;
        Ok(w.data)
    }

    pub async fn delete_note(&self, id: &str) -> Result<()> {
        let r = self
            .req(Method::DELETE, &format!("/v1/notes/{id}"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }

    pub async fn get_guilds(&self) -> Result<ApiList<Guild>> {
        let r = self.req(Method::GET, "/v1/guilds?limit=20").send().await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }

    pub async fn get_guild(&self, slug: &str) -> Result<Guild> {
        let r = self
            .req(Method::GET, &format!("/v1/guilds/{slug}"))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<Guild> = r.json().await?;
        Ok(w.data)
    }

    pub async fn get_guild_posts(&self, slug: &str) -> Result<ApiList<Post>> {
        let r = self
            .req(Method::GET, &format!("/v1/guilds/{}/posts?limit=20", slug))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(r.json().await?)
    }

    pub async fn join_guild(&self, slug: &str) -> Result<()> {
        let r = self
            .req(Method::POST, &format!("/v1/guilds/{}/join", slug))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }

    pub async fn leave_guild(&self, slug: &str) -> Result<()> {
        let r = self
            .req(Method::POST, &format!("/v1/guilds/{}/leave", slug))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        Ok(())
    }

    pub async fn update_profile(
        &self,
        bio: Option<&str>,
        display_name: Option<&str>,
        website_url: Option<&str>,
        website_name: Option<&str>,
    ) -> Result<User> {
        #[derive(Serialize)]
        struct B<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            bio: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none", rename = "displayName")]
            display_name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none", rename = "websiteUrl")]
            website_url: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none", rename = "websiteName")]
            website_name: Option<&'a str>,
        }
        let r = self
            .req(Method::PATCH, "/v1/users/me")
            .json(&B {
                bio,
                display_name,
                website_url,
                website_name,
            })
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Self::err(r).await);
        }
        let w: ApiWrap<User> = r.json().await?;
        Ok(w.data)
    }
}
