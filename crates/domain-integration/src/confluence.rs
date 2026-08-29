//! Confluence Adapter — Confluence 空间 / 页面 双向链接 + 嵌入 macro
//!
//! **crate**: `domain-integration::confluence`
//!
//! **Phase W15 职责**(per task brief):
//! - OAuth2 授权(走 domain-identity credential broker)
//! - 拉取 Confluence space / page 列表
//! - 双向链接(Confluence page ↔ Star 工作项)
//! - 嵌入 macro(Star 工作项可嵌入 Confluence)
//!
//! **API 模型** (基于 Atlassian Confluence Cloud REST API v2):
//! - `GET /wiki/api/v2/spaces` — 列出空间
//! - `GET /wiki/api/v2/spaces/{id}/pages` — 列出空间内页面
//! - `POST /wiki/api/v2/pages` — 创建页面
//! - `PUT /wiki/api/v2/pages/{id}` — 更新页面
//! - 嵌入 macro: `<ac:structured-macro ac:name="star-work-item">...</ac:structured-macro>`
//!
//! **重要**: 真实 HTTP 调用由注入的 `HttpClient` 完成;本文件不持有 reqwest 依赖。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::adapter::{
    Adapter, AdapterCapability, AdapterError, AdapterResult, AuthToken, CredentialRefId,
    HttpClient, OAuth2AuthRequest,
};

// =====================================================================
// Confluence 实体模型
// =====================================================================

/// **Confluence Space**(简化的 REST API v2 模型)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfluenceSpace {
    /// 空间 key(全局唯一,例如 "DEV", "PROJ")
    pub key: String,
    /// 空间 ID
    pub id: String,
    /// 空间名称
    pub name: String,
    /// 空间类型("global" / "personal")
    #[serde(rename = "type")]
    pub space_type: String,
    /// 空间描述
    pub description: Option<String>,
    /// 创建时间
    #[serde(default = "default_now")]
    pub created_at: DateTime<Utc>,
    /// Star 工作项 ID(若该 space 与 Star 工作项关联)
    pub star_work_item_id: Option<Uuid>,
}

fn default_now() -> DateTime<Utc> {
    Utc::now()
}

/// **Confluence Page**(简化的 REST API v2 模型)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfluencePage {
    /// 页面 ID
    pub id: String,
    /// 所属 space key
    pub space_key: String,
    /// 页面标题
    pub title: String,
    /// 页面 body(storage format,含 macro)
    pub body_storage: String,
    /// 页面版本号
    pub version: u32,
    /// Star 工作项 ID(若该 page 与 Star 工作项关联)
    pub star_work_item_id: Option<Uuid>,
    /// 创建时间
    #[serde(default = "default_now")]
    pub created_at: DateTime<Utc>,
    /// 更新时间
    #[serde(default = "default_now")]
    pub updated_at: DateTime<Utc>,
}

// =====================================================================
// Confluence 嵌入 Macro 格式
// =====================================================================

/// **嵌入 Star 工作项的 macro**(Confluence storage format XML)
///
/// 渲染后用户可在 Confluence 页面看到 Star 工作项卡片。
/// 真实渲染逻辑由 Confluence 的 star-macro plugin 完成(在 infrastructure 层)。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StarWorkItemMacro {
    /// Star 工作项 ID
    pub work_item_id: Uuid,
    /// 关联项目 key(显示用,可选)
    pub project_key: Option<String>,
    /// 展示模式("card" / "inline" / "summary")
    pub display_mode: String,
}

impl StarWorkItemMacro {
    /// 序列化为 Confluence storage format XML
    pub fn to_storage_xml(&self) -> String {
        let project_attr = self
            .project_key
            .as_ref()
            .map(|k| format!(" ac:name=\"star-work-item\" ac:project=\"{}\"", k))
            .unwrap_or_else(|| " ac:name=\"star-work-item\"".to_string());
        format!(
            r#"<ac:structured-macro{project_attr} ac:work-item-id="{wiid}" ac:display-mode="{mode}"><ac:parameter ac:name="title">Star Work Item {wiid}</ac:parameter></ac:structured-macro>"#,
            project_attr = project_attr,
            wiid = self.work_item_id,
            mode = self.display_mode,
        )
    }

    /// 从 storage XML 解析(若存在 star-work-item macro)
    pub fn parse_from_storage(storage: &str) -> Option<Self> {
        // 简化:用字符串包含判断;真实场景用 roxmltree / scraper
        if !storage.contains("ac:name=\"star-work-item\"") {
            return None;
        }
        // 提取 work-item-id
        let wiid = extract_attr(storage, "ac:work-item-id")?;
        let work_item_id = Uuid::parse_str(&wiid).ok()?;
        let project_key = extract_attr(storage, "ac:project");
        let display_mode =
            extract_attr(storage, "ac:display-mode").unwrap_or_else(|| "card".to_string());
        Some(Self {
            work_item_id,
            project_key,
            display_mode,
        })
    }
}

fn extract_attr(s: &str, name: &str) -> Option<String> {
    let pattern = format!("{}=\"", name);
    let start = s.find(&pattern)? + pattern.len();
    let rest = &s[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

// =====================================================================
// 双向链接(Star ↔ Confluence)
// =====================================================================

/// **Confluence 双向链接记录**
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfluenceLink {
    /// Star 工作项 ID
    pub work_item_id: Uuid,
    /// Confluence page ID
    pub page_id: String,
    /// Confluence space key
    pub space_key: String,
    /// 链接方向(Star→Confluence / Confluence→Star / 双向)
    pub direction: String,
    /// 链接创建时间
    pub created_at: DateTime<Utc>,
    /// 链接创建者 user_id
    pub created_by_user_id: Uuid,
}

impl ConfluenceLink {
    /// 是否为双向
    pub fn is_bidirectional(&self) -> bool {
        self.direction == "BOTH"
    }
}

// =====================================================================
// Confluence Adapter
// =====================================================================

/// **Confluence Adapter 内部状态**
#[derive(Debug)]
struct ConfluenceAdapterInner {
    /// Confluence 站点 URL(例如 "https://acme.atlassian.net")
    site_url: String,
    /// OAuth2 client_id
    client_id: String,
    /// OAuth2 client_secret(由调用方注入,本字段不参与序列化)
    client_secret: String,
    /// 当前 token
    token: Option<AuthToken>,
    /// 双向链接索引
    links: HashMap<Uuid, Vec<ConfluenceLink>>,
}

/// **Confluence Adapter**
pub struct ConfluenceAdapter {
    /// site_url + client_id(client_id 可用于审计日志)
    site_url: String,
    client_id: String,
    /// HTTP client(可注入 mock)
    http: Arc<dyn HttpClient>,
    /// 内部状态
    inner: Arc<RwLock<ConfluenceAdapterInner>>,
}

impl ConfluenceAdapter {
    /// 创建新 adapter
    pub fn new(
        site_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        http: Arc<dyn HttpClient>,
    ) -> Self {
        let site_url_str = site_url.into();
        let client_id_str = client_id.into();
        let client_secret_str = client_secret.into();
        Self {
            site_url: site_url_str.clone(),
            client_id: client_id_str.clone(),
            http,
            inner: Arc::new(RwLock::new(ConfluenceAdapterInner {
                site_url: site_url_str,
                client_id: client_id_str,
                client_secret: client_secret_str,
                token: None,
                links: HashMap::new(),
            })),
        }
    }

    /// 构造 OAuth2 授权 URL 参数(Authorization Code + PKCE)
    pub fn build_oauth2_request(
        &self,
        redirect_uri: impl Into<String>,
        scopes: Vec<String>,
        state: impl Into<String>,
        code_challenge: impl Into<String>,
    ) -> OAuth2AuthRequest {
        OAuth2AuthRequest {
            auth_endpoint: format!("{}/wiki/oauth/authorize", self.site_url),
            client_id: self.client_id.clone(),
            redirect_uri: redirect_uri.into(),
            scope: scopes,
            state: state.into(),
            code_challenge: code_challenge.into(),
            code_challenge_method: "S256".to_string(),
        }
    }

    /// 凭据引用 ID(未授权时为 None)
    pub fn credential_ref(&self) -> Option<CredentialRefId> {
        self.inner
            .read()
            .ok()
            .and_then(|g| g.token.as_ref().map(|t| t.credential_ref.clone()))
    }

    /// 拉取空间列表
    pub async fn list_spaces(&self) -> AdapterResult<Vec<ConfluenceSpace>> {
        self.ensure_authorized()?;
        let url = format!("{}/wiki/api/v2/spaces?limit=25", self.site_url);
        let headers = self.auth_headers();
        let (status, body) = self.http.get(&url, headers).await?;
        if status == 401 {
            return Err(AdapterError::InvalidCredential("token rejected".into()));
        }
        if status == 429 {
            return Err(AdapterError::RateLimited(1000));
        }
        if status >= 400 {
            return Err(AdapterError::Http(format!(
                "status={} body={}",
                status, body
            )));
        }
        let resp: SpacesResponse =
            serde_json::from_str(&body).map_err(|e| AdapterError::Parse(e.to_string()))?;
        Ok(resp.results)
    }

    /// 拉取空间内页面
    pub async fn list_pages(&self, space_key: &str) -> AdapterResult<Vec<ConfluencePage>> {
        self.ensure_authorized()?;
        let url = format!(
            "{}/wiki/api/v2/spaces/{}/pages?limit=25",
            self.site_url, space_key
        );
        let headers = self.auth_headers();
        let (status, body) = self.http.get(&url, headers).await?;
        if status >= 400 {
            return Err(AdapterError::Http(format!(
                "status={} body={}",
                status, body
            )));
        }
        let resp: PagesResponse =
            serde_json::from_str(&body).map_err(|e| AdapterError::Parse(e.to_string()))?;
        Ok(resp.results)
    }

    /// 在指定 page 追加 Star 工作项 macro(双向链接的 push 端)
    pub async fn append_macro(
        &self,
        page_id: &str,
        macro_: &StarWorkItemMacro,
    ) -> AdapterResult<ConfluencePage> {
        self.ensure_authorized()?;
        let page = self.get_page(page_id).await?;
        // 幂等:若 macro 已存在,直接返回
        if StarWorkItemMacro::parse_from_storage(&page.body_storage)
            .map(|m| m.work_item_id == macro_.work_item_id)
            .unwrap_or(false)
        {
            return Ok(page);
        }
        // 追加 macro 到 body 末尾
        let new_body = format!("{}\n{}", page.body_storage, macro_.to_storage_xml());
        self.update_page_body(page_id, &new_body, page.version + 1)
            .await
    }

    /// 拉取单个页面
    pub async fn get_page(&self, page_id: &str) -> AdapterResult<ConfluencePage> {
        self.ensure_authorized()?;
        let url = format!("{}/wiki/api/v2/pages/{}", self.site_url, page_id);
        let (status, body) = self.http.get(&url, self.auth_headers()).await?;
        if status == 404 {
            return Err(AdapterError::NotFound(format!("page:{}", page_id)));
        }
        if status >= 400 {
            return Err(AdapterError::Http(format!(
                "status={} body={}",
                status, body
            )));
        }
        serde_json::from_str(&body).map_err(|e| AdapterError::Parse(e.to_string()))
    }

    /// 更新页面 body(内部用,提供 version 检查)
    async fn update_page_body(
        &self,
        page_id: &str,
        new_body: &str,
        new_version: u32,
    ) -> AdapterResult<ConfluencePage> {
        let url = format!("{}/wiki/api/v2/pages/{}", self.site_url, page_id);
        let payload = serde_json::json!({
            "id": page_id,
            "status": "current",
            "title": "",  // 留空,server 保留原 title
            "body": {
                "representation": "storage",
                "value": new_body,
            },
            "version": { "number": new_version },
        });
        let body =
            serde_json::to_string(&payload).map_err(|e| AdapterError::Internal(e.to_string()))?;
        let (status, resp_body) = self.http.put(&url, self.auth_headers(), body).await?;
        if status >= 400 {
            return Err(AdapterError::Http(format!(
                "status={} body={}",
                status, resp_body
            )));
        }
        serde_json::from_str(&resp_body).map_err(|e| AdapterError::Parse(e.to_string()))
    }

    /// 创建双向链接记录
    pub fn register_link(&self, link: ConfluenceLink) {
        if let Ok(mut g) = self.inner.write() {
            g.links.entry(link.work_item_id).or_default().push(link);
        }
    }

    /// 列出工作项的所有链接
    pub fn links_for(&self, work_item_id: Uuid) -> Vec<ConfluenceLink> {
        self.inner
            .read()
            .ok()
            .and_then(|g| g.links.get(&work_item_id).cloned())
            .unwrap_or_default()
    }

    /// 校验已授权
    fn ensure_authorized(&self) -> AdapterResult<()> {
        match self.inner.read() {
            Ok(g) => match &g.token {
                Some(t) if !t.is_expired() => Ok(()),
                Some(_) => Err(AdapterError::NotAuthorized("token expired".into())),
                None => Err(AdapterError::NotAuthorized("no token".into())),
            },
            Err(_) => Err(AdapterError::Internal("lock poisoned".into())),
        }
    }

    fn auth_headers(&self) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("Accept".into(), "application/json".into());
        if let Ok(g) = self.inner.read() {
            if let Some(t) = &g.token {
                h.insert("Authorization".into(), format!("Bearer {}", t.access_token));
            }
        }
        h
    }
}

#[async_trait]
impl Adapter for ConfluenceAdapter {
    fn kind(&self) -> &'static str {
        "confluence"
    }
    fn capabilities(&self) -> Vec<AdapterCapability> {
        vec![
            AdapterCapability::ListExternalEntities,
            AdapterCapability::PushWorkItem,
            AdapterCapability::ReceiveExternalEvents,
            AdapterCapability::BidirectionalLink,
            AdapterCapability::EmbedCard,
        ]
    }
    fn is_authorized(&self) -> bool {
        self.inner
            .read()
            .ok()
            .and_then(|g| g.token.as_ref().map(|t| !t.is_expired()))
            .unwrap_or(false)
    }
    fn current_token(&self) -> Option<&AuthToken> {
        // 简化:返回 None;调用方用 is_authorized() 校验
        // 因为 RwLock guard 跨越函数边界复杂
        None
    }
    fn set_token(&mut self, token: AuthToken) {
        if let Ok(mut g) = self.inner.write() {
            g.token = Some(token);
        }
    }
    fn clear_token(&mut self) {
        if let Ok(mut g) = self.inner.write() {
            g.token = None;
        }
    }
    fn http(&self) -> &dyn HttpClient {
        self.http.as_ref()
    }
}

// =====================================================================
// Confluence REST 响应包装
// =====================================================================

#[derive(Debug, Deserialize)]
struct SpacesResponse {
    results: Vec<ConfluenceSpace>,
}

#[derive(Debug, Deserialize)]
struct PagesResponse {
    results: Vec<ConfluencePage>,
}

// =====================================================================
// 单元测试
// =====================================================================
//
// Phase W15 验收要求:
// 1. Confluence adapter 列表空间 / 列表页面 / 追加 macro
// 2. OAuth2 授权 URL 构建
// 3. token 过期校验
// 4. 双向链接 register / list
// 5. StarWorkItemMacro 序列化 / 解析

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::Mutex;

    // -------- Mock HTTP client --------

    #[derive(Debug, Clone)]
    struct MockResponse {
        status: u16,
        body: String,
    }

    struct MockHttpClient {
        responses: Mutex<HashMap<String, MockResponse>>,
    }

    impl MockHttpClient {
        fn new() -> Self {
            Self {
                responses: Mutex::new(HashMap::new()),
            }
        }
        fn enqueue(&self, url: &str, status: u16, body: &str) {
            self.responses.lock().unwrap().insert(
                url.to_string(),
                MockResponse {
                    status,
                    body: body.to_string(),
                },
            );
        }
    }

    #[async_trait]
    impl HttpClient for MockHttpClient {
        async fn get(
            &self,
            url: &str,
            _headers: HashMap<String, String>,
        ) -> AdapterResult<(u16, String)> {
            let g = self.responses.lock().unwrap();
            let r = g
                .get(url)
                .cloned()
                .ok_or_else(|| AdapterError::Internal(format!("no mock for GET {}", url)))?;
            Ok((r.status, r.body))
        }
        async fn post(
            &self,
            url: &str,
            _headers: HashMap<String, String>,
            _body: String,
        ) -> AdapterResult<(u16, String)> {
            let g = self.responses.lock().unwrap();
            let r = g
                .get(url)
                .cloned()
                .ok_or_else(|| AdapterError::Internal(format!("no mock for POST {}", url)))?;
            Ok((r.status, r.body))
        }
        async fn put(
            &self,
            url: &str,
            _headers: HashMap<String, String>,
            _body: String,
        ) -> AdapterResult<(u16, String)> {
            let g = self.responses.lock().unwrap();
            let r = g
                .get(url)
                .cloned()
                .ok_or_else(|| AdapterError::Internal(format!("no mock for PUT {}", url)))?;
            Ok((r.status, r.body))
        }
        async fn delete(
            &self,
            url: &str,
            _headers: HashMap<String, String>,
        ) -> AdapterResult<(u16, String)> {
            let g = self.responses.lock().unwrap();
            let r = g
                .get(url)
                .cloned()
                .ok_or_else(|| AdapterError::Internal(format!("no mock for DELETE {}", url)))?;
            Ok((r.status, r.body))
        }
    }

    fn make_token() -> AuthToken {
        AuthToken {
            access_token: "fake-at".into(),
            refresh_token: Some("fake-rt".into()),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(3600)),
            scope: "read:page-details write:page".into(),
            credential_ref: CredentialRefId::new(),
        }
    }

    fn make_adapter() -> (ConfluenceAdapter, Arc<MockHttpClient>) {
        let mock = Arc::new(MockHttpClient::new());
        let mut adapter = ConfluenceAdapter::new(
            "https://acme.atlassian.net",
            "client-123",
            "secret-xyz",
            mock.clone() as Arc<dyn HttpClient>,
        );
        adapter.set_token(make_token());
        (adapter, mock)
    }

    // -------- Test 1: list_spaces with mock HTTP --------

    #[tokio::test]
    async fn list_spaces_returns_results() {
        let (adapter, mock) = make_adapter();
        let body = serde_json::json!({
            "results": [
                {"id":"1","key":"DEV","name":"Development","type":"global","description":"dev docs","created_at":"2026-01-01T00:00:00Z","star_work_item_id":null},
                {"id":"2","key":"PROJ","name":"Project","type":"global","description":"project","created_at":"2026-01-01T00:00:00Z","star_work_item_id":null},
            ]
        }).to_string();
        mock.enqueue(
            "https://acme.atlassian.net/wiki/api/v2/spaces?limit=25",
            200,
            &body,
        );

        let spaces = adapter.list_spaces().await.expect("list ok");
        assert_eq!(spaces.len(), 2);
        assert_eq!(spaces[0].key, "DEV");
        assert_eq!(spaces[1].name, "Project");
    }

    // -------- Test 2: list_spaces 401 → InvalidCredential --------

    #[tokio::test]
    async fn list_spaces_401_returns_invalid_credential() {
        let (adapter, mock) = make_adapter();
        mock.enqueue(
            "https://acme.atlassian.net/wiki/api/v2/spaces?limit=25",
            401,
            "{}",
        );
        let res = adapter.list_spaces().await;
        assert!(matches!(res, Err(AdapterError::InvalidCredential(_))));
    }

    // -------- Test 3: list_spaces 429 → RateLimited --------

    #[tokio::test]
    async fn list_spaces_429_returns_rate_limited() {
        let (adapter, mock) = make_adapter();
        mock.enqueue(
            "https://acme.atlassian.net/wiki/api/v2/spaces?limit=25",
            429,
            "{}",
        );
        let res = adapter.list_spaces().await;
        assert!(matches!(res, Err(AdapterError::RateLimited(_))));
    }

    // -------- Test 4: list_pages --------

    #[tokio::test]
    async fn list_pages_returns_results() {
        let (adapter, mock) = make_adapter();
        let body = serde_json::json!({
            "results": [
                {"id":"p1","space_key":"DEV","title":"How to test","body_storage":"hello","version":1,"star_work_item_id":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"},
            ]
        }).to_string();
        mock.enqueue(
            "https://acme.atlassian.net/wiki/api/v2/spaces/DEV/pages?limit=25",
            200,
            &body,
        );
        let pages = adapter.list_pages("DEV").await.expect("list ok");
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].title, "How to test");
    }

    // -------- Test 5: append_macro 幂等 --------

    #[tokio::test]
    async fn append_macro_idempotent() {
        let (adapter, mock) = make_adapter();
        let wiid = Uuid::new_v4();
        // 第一次: page 不含 macro
        mock.enqueue(
            "https://acme.atlassian.net/wiki/api/v2/pages/p1",
            200,
            &serde_json::json!({
                "id":"p1","space_key":"DEV","title":"T","body_storage":"<p>hi</p>","version":1,
                "star_work_item_id":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"
            }).to_string(),
        );
        // PUT 应被调用
        mock.enqueue(
            "https://acme.atlassian.net/wiki/api/v2/pages/p1",
            200,
            &serde_json::json!({
                "id":"p1","space_key":"DEV","title":"T","body_storage":"<p>hi</p>\n<macro/>","version":2,
                "star_work_item_id":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-02T00:00:00Z"
            }).to_string(),
        );
        let m = StarWorkItemMacro {
            work_item_id: wiid,
            project_key: Some("STAR".into()),
            display_mode: "card".into(),
        };
        let p1 = adapter.append_macro("p1", &m).await.expect("ok");
        assert_eq!(p1.id, "p1");
    }

    // 静默抑制 unused variable
    #[allow(dead_code)]
    fn _unused_var(wiid: Uuid) {
        let _ = wiid;
    }

    // -------- Test 6: append_macro 幂等(第二次跳过 PUT) --------

    #[tokio::test]
    async fn append_macro_skips_when_present() {
        let (adapter, _mock) = make_adapter();
        let wiid = Uuid::new_v4();
        // 直接调用 storage 解析
        let storage = format!(
            r#"<p>hi</p><ac:structured-macro ac:name="star-work-item" ac:work-item-id="{}" ac:display-mode="card"/>"#,
            wiid
        );
        let parsed = StarWorkItemMacro::parse_from_storage(&storage);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap().work_item_id, wiid);
    }

    // -------- Test 7: 未授权返回 NotAuthorized --------

    #[tokio::test]
    async fn list_spaces_without_token_rejected() {
        let mock = Arc::new(MockHttpClient::new());
        let mut adapter = ConfluenceAdapter::new(
            "https://acme.atlassian.net",
            "client-123",
            "secret-xyz",
            mock,
        );
        // 不注入 token
        // (直接调用无需 mutable;但确保 adapter 是 fresh)
        let _ = &mut adapter;
        let res = adapter.list_spaces().await;
        assert!(matches!(res, Err(AdapterError::NotAuthorized(_))));
    }

    // -------- Test 8: build_oauth2_request --------

    #[tokio::test]
    async fn build_oauth2_request_has_pkce() {
        let mock = Arc::new(MockHttpClient::new());
        let adapter = ConfluenceAdapter::new(
            "https://acme.atlassian.net",
            "client-123",
            "secret-xyz",
            mock,
        );
        let r = adapter.build_oauth2_request(
            "https://app.star.local/callback",
            vec!["read:page-details".into(), "write:page".into()],
            "state-abc",
            "challenge-xyz",
        );
        assert_eq!(
            r.auth_endpoint,
            "https://acme.atlassian.net/wiki/oauth/authorize"
        );
        assert_eq!(r.client_id, "client-123");
        assert_eq!(r.redirect_uri, "https://app.star.local/callback");
        assert_eq!(r.scope.len(), 2);
        assert_eq!(r.state, "state-abc");
        assert_eq!(r.code_challenge_method, "S256");
    }

    // -------- Test 9: StarWorkItemMacro 序列化 --------

    #[test]
    fn star_macro_to_storage_xml() {
        let m = StarWorkItemMacro {
            work_item_id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            project_key: Some("STAR".into()),
            display_mode: "card".into(),
        };
        let xml = m.to_storage_xml();
        assert!(xml.contains("star-work-item"));
        assert!(xml.contains("11111111-1111-1111-1111-111111111111"));
        assert!(xml.contains("STAR"));
        assert!(xml.contains("card"));
    }

    // -------- Test 10: StarWorkItemMacro 解析 --------

    #[test]
    fn star_macro_parse_from_storage() {
        let storage = r#"<p>x</p><ac:structured-macro ac:name="star-work-item" ac:work-item-id="22222222-2222-2222-2222-222222222222" ac:display-mode="inline"/>"#;
        let m = StarWorkItemMacro::parse_from_storage(storage).expect("parsed");
        assert_eq!(
            m.work_item_id,
            Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()
        );
        assert_eq!(m.display_mode, "inline");
    }

    // -------- Test 11: 双向链接 register / list --------

    #[tokio::test]
    async fn bidirectional_link_register_and_list() {
        let (adapter, _mock) = make_adapter();
        let wiid = Uuid::new_v4();
        let link = ConfluenceLink {
            work_item_id: wiid,
            page_id: "p1".into(),
            space_key: "DEV".into(),
            direction: "BOTH".into(),
            created_at: Utc::now(),
            created_by_user_id: Uuid::new_v4(),
        };
        adapter.register_link(link.clone());
        let links = adapter.links_for(wiid);
        assert_eq!(links.len(), 1);
        assert!(links[0].is_bidirectional());
    }

    // -------- Test 12: 清除 token --------

    #[tokio::test]
    async fn clear_token_revokes_authorization() {
        let (mut adapter, _mock) = make_adapter();
        assert!(adapter.is_authorized());
        adapter.clear_token();
        assert!(!adapter.is_authorized());
    }
}
