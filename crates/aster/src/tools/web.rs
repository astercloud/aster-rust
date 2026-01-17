//! Web 工具 - WebFetch 和 WebSearch
//!
//! 对齐 Claude Agent SDK 的 Web 工具功能

use super::base::{PermissionCheckResult, Tool};
use super::context::{ToolContext, ToolResult};
use super::error::ToolError;
use async_trait::async_trait;
use lru::LruCache;
use reqwest::Client;
use scraper::Html;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use url::Url;

/// 响应体大小限制 (10MB)
const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

/// WebFetch 缓存 TTL (15分钟)
const WEB_FETCH_CACHE_TTL: Duration = Duration::from_secs(15 * 60);

/// WebSearch 缓存 TTL (1小时)
const WEB_SEARCH_CACHE_TTL: Duration = Duration::from_secs(60 * 60);

/// 缓存内容结构
#[derive(Debug, Clone)]
struct CachedContent {
    content: String,
    content_type: String,
    status_code: u16,
    fetched_at: SystemTime,
}

/// 搜索结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub publish_date: Option<String>,
}

/// 缓存的搜索结果
#[derive(Debug, Clone)]
struct CachedSearchResults {
    query: String,
    results: Vec<SearchResult>,
    fetched_at: SystemTime,
    allowed_domains: Option<Vec<String>>,
    blocked_domains: Option<Vec<String>>,
}

/// WebFetchTool 输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFetchInput {
    /// 要获取的 URL
    pub url: String,
    /// 处理内容的提示词
    pub prompt: String,
}

/// WebSearchTool 输入参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchInput {
    /// 搜索查询
    pub query: String,
    /// 允许的域名列表
    pub allowed_domains: Option<Vec<String>>,
    /// 阻止的域名列表
    pub blocked_domains: Option<Vec<String>>,
}

/// Web 工具的共享缓存
pub struct WebCache {
    fetch_cache: Arc<Mutex<LruCache<String, CachedContent>>>,
    search_cache: Arc<Mutex<LruCache<String, CachedSearchResults>>>,
}

impl Default for WebCache {
    fn default() -> Self {
        Self::new()
    }
}

impl WebCache {
    /// 创建新的 Web 缓存
    pub fn new() -> Self {
        Self {
            fetch_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
            search_cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(500).unwrap()))),
        }
    }

    /// 获取缓存的内容
    fn get_cached_content(&self, url: &str) -> Option<CachedContent> {
        let mut cache = self.fetch_cache.lock().unwrap();
        if let Some(cached) = cache.get(url) {
            // 检查是否过期
            if cached.fetched_at.elapsed().unwrap_or(Duration::MAX) < WEB_FETCH_CACHE_TTL {
                return Some(cached.clone());
            } else {
                // 过期，移除
                cache.pop(url);
            }
        }
        None
    }

    /// 缓存内容
    fn cache_content(&self, url: String, content: CachedContent) {
        let mut cache = self.fetch_cache.lock().unwrap();
        cache.put(url, content);
    }

    /// 生成搜索缓存键
    fn generate_search_cache_key(
        query: &str,
        allowed_domains: &Option<Vec<String>>,
        blocked_domains: &Option<Vec<String>>,
    ) -> String {
        let normalized_query = query.trim().to_lowercase();
        let allowed = allowed_domains
            .as_ref()
            .map(|domains| {
                let mut sorted = domains.clone();
                sorted.sort();
                sorted.join(",")
            })
            .unwrap_or_default();
        let blocked = blocked_domains
            .as_ref()
            .map(|domains| {
                let mut sorted = domains.clone();
                sorted.sort();
                sorted.join(",")
            })
            .unwrap_or_default();

        format!("{}|{}|{}", normalized_query, allowed, blocked)
    }

    /// 获取缓存的搜索结果
    fn get_cached_search(&self, cache_key: &str) -> Option<CachedSearchResults> {
        let mut cache = self.search_cache.lock().unwrap();
        if let Some(cached) = cache.get(cache_key) {
            // 检查是否过期
            if cached.fetched_at.elapsed().unwrap_or(Duration::MAX) < WEB_SEARCH_CACHE_TTL {
                return Some(cached.clone());
            } else {
                // 过期，移除
                cache.pop(cache_key);
            }
        }
        None
    }

    /// 缓存搜索结果
    fn cache_search(&self, cache_key: String, results: CachedSearchResults) {
        let mut cache = self.search_cache.lock().unwrap();
        cache.put(cache_key, results);
    }
}

/// WebFetchTool - Web 内容获取工具
///
/// 对齐 Claude Agent SDK 的 WebFetchTool 功能
pub struct WebFetchTool {
    client: Client,
    cache: Arc<WebCache>,
}

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebFetchTool {
    /// 创建新的 WebFetchTool
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; AsterAgent/1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            cache: Arc::new(WebCache::new()),
        }
    }

    /// 使用共享缓存创建 WebFetchTool
    pub fn with_cache(cache: Arc<WebCache>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; AsterAgent/1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, cache }
    }

    /// 检查域名安全性
    fn check_domain_safety(&self, url: &Url) -> Result<(), String> {
        let host = url.host_str().ok_or("无效的主机名")?;
        let host_lower = host.to_lowercase();

        // 不安全域名黑名单
        let unsafe_domains = [
            "localhost",
            "127.0.0.1",
            "0.0.0.0",
            "::1",
            "169.254.169.254",          // AWS 元数据服务
            "metadata.google.internal", // GCP 元数据服务
        ];

        for unsafe_domain in &unsafe_domains {
            if host_lower == *unsafe_domain || host_lower.ends_with(&format!(".{}", unsafe_domain))
            {
                return Err(format!("域名 {} 因安全原因被禁止访问", host));
            }
        }

        // 检查私有 IP 地址
        if self.is_private_ip(&host_lower) {
            return Err(format!("私有 IP 地址 {} 被禁止访问", host));
        }

        Ok(())
    }

    /// 检查是否为私有 IP 地址
    fn is_private_ip(&self, host: &str) -> bool {
        // 简单的 IPv4 私有地址检查
        if let Ok(addr) = host.parse::<std::net::Ipv4Addr>() {
            return addr.is_private() || addr.is_loopback() || addr.is_link_local();
        }
        false
    }

    /// HTML 转 Markdown
    fn html_to_markdown(&self, html: &str) -> String {
        let _document = Html::parse_document(html);

        // 移除 script 和 style 标签
        let mut cleaned_html = html.to_string();

        // 简单的标签清理
        cleaned_html = cleaned_html
            .replace("<script", "<removed-script")
            .replace("</script>", "</removed-script>")
            .replace("<style", "<removed-style")
            .replace("</style>", "</removed-style>");

        // 基本的 HTML 到文本转换
        self.html_to_text(&cleaned_html)
    }

    /// HTML 转纯文本（简化版）
    fn html_to_text(&self, html: &str) -> String {
        // 使用正则表达式移除 HTML 标签
        let re = regex::Regex::new(r"<[^>]+>").unwrap();
        let text = re.replace_all(html, " ");

        // 清理空白字符
        let re_whitespace = regex::Regex::new(r"\s+").unwrap();
        let cleaned = re_whitespace.replace_all(&text, " ");

        // HTML 实体解码
        cleaned
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#x27;", "'")
            .trim()
            .to_string()
    }

    /// 实际的 URL 抓取逻辑
    async fn fetch_url(&self, url: &str) -> Result<(String, String, u16), String> {
        let parsed_url = Url::parse(url).map_err(|e| format!("无效的 URL: {}", e))?;

        // 域名安全检查
        self.check_domain_safety(&parsed_url)?;

        let response = self
            .client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (compatible; AsterAgent/1.0)")
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;

        let status_code = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("")
            .to_string();

        // 检查响应体大小
        if let Some(content_length) = response.content_length() {
            if content_length > MAX_RESPONSE_SIZE as u64 {
                return Err(format!(
                    "响应体大小 ({} 字节) 超过最大限制 ({} 字节)",
                    content_length, MAX_RESPONSE_SIZE
                ));
            }
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("读取响应体失败: {}", e))?;

        // 检查处理后内容的大小
        if body.len() > MAX_RESPONSE_SIZE {
            return Err(format!(
                "内容大小 ({} 字节) 超过最大限制 ({} 字节)",
                body.len(),
                MAX_RESPONSE_SIZE
            ));
        }

        let processed_content = if content_type.contains("text/html") {
            self.html_to_markdown(&body)
        } else if content_type.contains("application/json") {
            // 格式化 JSON
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => serde_json::to_string_pretty(&json).unwrap_or(body),
                Err(_) => body,
            }
        } else {
            body
        };

        Ok((processed_content, content_type, status_code))
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "WebFetch"
    }

    fn description(&self) -> &str {
        "获取指定 URL 的内容并使用 AI 模型处理。\n\
         输入 URL 和提示词，获取 URL 内容，将 HTML 转换为 Markdown，\n\
         然后使用小型快速模型处理内容并返回模型对内容的响应。\n\
         当需要检索和分析 Web 内容时使用此工具。"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "format": "uri",
                    "description": "要获取内容的 URL"
                },
                "prompt": {
                    "type": "string",
                    "description": "用于处理获取内容的提示词"
                }
            },
            "required": ["url", "prompt"]
        })
    }

    async fn check_permissions(
        &self,
        _params: &serde_json::Value,
        _context: &ToolContext,
    ) -> PermissionCheckResult {
        PermissionCheckResult::allow()
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let input: WebFetchInput = serde_json::from_value(params)
            .map_err(|e| ToolError::execution_failed(format!("输入参数解析失败: {}", e)))?;

        let mut url = input.url;
        let prompt = input.prompt;

        // URL 验证和规范化
        let parsed_url = Url::parse(&url)
            .map_err(|e| ToolError::execution_failed(format!("无效的 URL: {}", e)))?;

        // HTTP 到 HTTPS 自动升级
        if parsed_url.scheme() == "http" {
            let mut new_url = parsed_url;
            new_url.set_scheme("https").map_err(|_| {
                ToolError::execution_failed("无法将 HTTP URL 升级为 HTTPS".to_string())
            })?;
            url = new_url.to_string();
        }

        // 检查缓存
        if let Some(cached) = self.cache.get_cached_content(&url) {
            let max_length = 100_000;
            let mut content = cached.content.clone();
            if content.len() > max_length {
                // 安全地截断字符串，避免在 UTF-8 字符中间切割
                let truncated = content.chars().take(max_length).collect::<String>();
                content = format!("{}...\n\n[内容已截断]", truncated);
            }

            return Ok(ToolResult::success(format!(
                "URL: {}\n提示词: {}\n\n--- 内容 (缓存) ---\n{}",
                url, prompt, content
            )));
        }

        // 获取内容
        match self.fetch_url(&url).await {
            Ok((content, content_type, status_code)) => {
                if status_code >= 400 {
                    return Err(ToolError::execution_failed(format!(
                        "HTTP 错误: {} {}",
                        status_code,
                        match status_code {
                            404 => "Not Found",
                            403 => "Forbidden",
                            500 => "Internal Server Error",
                            _ => "Unknown Error",
                        }
                    )));
                }

                // 截断过长的内容
                let max_length = 100_000;
                let display_content = if content.len() > max_length {
                    // 安全地截断字符串，避免在 UTF-8 字符中间切割
                    let truncated = content.chars().take(max_length).collect::<String>();
                    format!("{}...\n\n[内容已截断]", truncated)
                } else {
                    content.clone()
                };

                // 缓存结果
                self.cache.cache_content(
                    url.clone(),
                    CachedContent {
                        content: content.clone(),
                        content_type,
                        status_code,
                        fetched_at: SystemTime::now(),
                    },
                );

                Ok(ToolResult::success(format!(
                    "URL: {}\n提示词: {}\n\n--- 内容 ---\n{}",
                    url, prompt, display_content
                )))
            }
            Err(e) => Err(ToolError::execution_failed(format!("获取失败: {}", e))),
        }
    }
}

/// WebSearchTool - Web 搜索工具
///
/// 对齐 Claude Agent SDK 的 WebSearchTool 功能
pub struct WebSearchTool {
    client: Client,
    cache: Arc<WebCache>,
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSearchTool {
    /// 创建新的 WebSearchTool
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (compatible; AsterAgent/1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            cache: Arc::new(WebCache::new()),
        }
    }

    /// 使用共享缓存创建 WebSearchTool
    pub fn with_cache(cache: Arc<WebCache>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (compatible; AsterAgent/1.0)")
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { client, cache }
    }

    /// 从 URL 提取域名
    fn extract_domain(&self, url: &str) -> String {
        match Url::parse(url) {
            Ok(parsed) => {
                // 移除 www. 前缀
                parsed.host_str().unwrap_or("").replace("www.", "")
            }
            Err(_) => String::new(),
        }
    }

    /// 应用域名过滤
    fn apply_domain_filters(
        &self,
        results: Vec<SearchResult>,
        allowed_domains: &Option<Vec<String>>,
        blocked_domains: &Option<Vec<String>>,
    ) -> Vec<SearchResult> {
        let mut filtered = results;

        // 应用白名单
        if let Some(allowed) = allowed_domains {
            if !allowed.is_empty() {
                let normalized_allowed: Vec<String> =
                    allowed.iter().map(|d| d.to_lowercase()).collect();
                filtered.retain(|result| {
                    let domain = self.extract_domain(&result.url).to_lowercase();
                    normalized_allowed.contains(&domain)
                });
            }
        }

        // 应用黑名单
        if let Some(blocked) = blocked_domains {
            if !blocked.is_empty() {
                let normalized_blocked: Vec<String> =
                    blocked.iter().map(|d| d.to_lowercase()).collect();
                filtered.retain(|result| {
                    let domain = self.extract_domain(&result.url).to_lowercase();
                    !normalized_blocked.contains(&domain)
                });
            }
        }

        filtered
    }

    /// 格式化搜索结果为 Markdown
    fn format_search_results(&self, results: &[SearchResult], query: &str) -> String {
        let mut output = format!("搜索查询: \"{}\"\n\n", query);

        if results.is_empty() {
            output.push_str("未找到结果。\n");
            return output;
        }

        // 结果列表
        for (index, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. [{}]({})\n",
                index + 1,
                result.title,
                result.url
            ));
            if let Some(snippet) = &result.snippet {
                output.push_str(&format!("   {}\n", snippet));
            }
            if let Some(publish_date) = &result.publish_date {
                output.push_str(&format!("   发布时间: {}\n", publish_date));
            }
            output.push('\n');
        }

        // 来源部分
        output.push_str("\n来源:\n");
        for result in results {
            output.push_str(&format!("- [{}]({})\n", result.title, result.url));
        }

        output
    }

    /// 执行搜索
    async fn perform_search(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        // 优先使用 Bing Search API（如果配置）
        if let Ok(bing_api_key) = std::env::var("BING_SEARCH_API_KEY") {
            if let Ok(results) = self.search_with_bing(query, &bing_api_key).await {
                return Ok(results);
            }
        }

        // 优先使用 Google Custom Search API（如果配置）
        if let (Ok(google_api_key), Ok(google_cx)) = (
            std::env::var("GOOGLE_SEARCH_API_KEY"),
            std::env::var("GOOGLE_SEARCH_ENGINE_ID"),
        ) {
            if let Ok(results) = self
                .search_with_google(query, &google_api_key, &google_cx)
                .await
            {
                return Ok(results);
            }
        }

        // 回退到 DuckDuckGo（免费，无需 API 密钥）
        self.search_with_duckduckgo(query).await
    }

    /// DuckDuckGo Instant Answer API 搜索
    async fn search_with_duckduckgo(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let response = self
            .client
            .get("https://api.duckduckgo.com/")
            .query(&[
                ("q", query),
                ("format", "json"),
                ("no_html", "1"),
                ("skip_disambig", "1"),
            ])
            .send()
            .await
            .map_err(|e| format!("DuckDuckGo 请求失败: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析 DuckDuckGo 响应失败: {}", e))?;

        let mut results = Vec::new();

        // 提取相关主题
        if let Some(related_topics) = data.get("RelatedTopics").and_then(|rt| rt.as_array()) {
            for topic in related_topics.iter().take(10) {
                // 处理嵌套主题
                if let Some(topics) = topic.get("Topics").and_then(|t| t.as_array()) {
                    for sub_topic in topics.iter().take(3) {
                        if let (Some(text), Some(url)) = (
                            sub_topic.get("Text").and_then(|t| t.as_str()),
                            sub_topic.get("FirstURL").and_then(|u| u.as_str()),
                        ) {
                            let title = text.split(" - ").next().unwrap_or(text);
                            results.push(SearchResult {
                                title: title.to_string(),
                                url: url.to_string(),
                                snippet: Some(text.to_string()),
                                publish_date: None,
                            });
                        }
                    }
                } else if let (Some(text), Some(url)) = (
                    topic.get("Text").and_then(|t| t.as_str()),
                    topic.get("FirstURL").and_then(|u| u.as_str()),
                ) {
                    let title = text.split(" - ").next().unwrap_or(text);
                    results.push(SearchResult {
                        title: title.to_string(),
                        url: url.to_string(),
                        snippet: Some(text.to_string()),
                        publish_date: None,
                    });
                }
            }
        }

        // 添加抽象答案（如果有）
        if let (Some(abstract_text), Some(abstract_url)) = (
            data.get("Abstract").and_then(|a| a.as_str()),
            data.get("AbstractURL").and_then(|u| u.as_str()),
        ) {
            if !abstract_text.is_empty() && !abstract_url.is_empty() {
                let title = data
                    .get("Heading")
                    .and_then(|h| h.as_str())
                    .unwrap_or("DuckDuckGo Instant Answer");
                results.insert(
                    0,
                    SearchResult {
                        title: title.to_string(),
                        url: abstract_url.to_string(),
                        snippet: Some(abstract_text.to_string()),
                        publish_date: None,
                    },
                );
            }
        }

        Ok(results)
    }

    /// Bing Search API 搜索
    async fn search_with_bing(
        &self,
        query: &str,
        api_key: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let response = self
            .client
            .get("https://api.bing.microsoft.com/v7.0/search")
            .query(&[("q", query), ("count", "10")])
            .header("Ocp-Apim-Subscription-Key", api_key)
            .send()
            .await
            .map_err(|e| format!("Bing Search API 请求失败: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析 Bing 响应失败: {}", e))?;

        let empty_vec = vec![];
        let web_pages = data
            .get("webPages")
            .and_then(|wp| wp.get("value"))
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        let results = web_pages
            .iter()
            .filter_map(|page| {
                let title = page.get("name")?.as_str()?.to_string();
                let url = page.get("url")?.as_str()?.to_string();
                let snippet = page
                    .get("snippet")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());
                let publish_date = page
                    .get("dateLastCrawled")
                    .and_then(|d| d.as_str())
                    .map(|d| d.to_string());

                Some(SearchResult {
                    title,
                    url,
                    snippet,
                    publish_date,
                })
            })
            .collect();

        Ok(results)
    }

    /// Google Custom Search API 搜索
    async fn search_with_google(
        &self,
        query: &str,
        api_key: &str,
        cx: &str,
    ) -> Result<Vec<SearchResult>, String> {
        let response = self
            .client
            .get("https://www.googleapis.com/customsearch/v1")
            .query(&[("key", api_key), ("cx", cx), ("q", query), ("num", "10")])
            .send()
            .await
            .map_err(|e| format!("Google Search API 请求失败: {}", e))?;

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析 Google 响应失败: {}", e))?;

        let empty_vec = vec![];
        let items = data
            .get("items")
            .and_then(|i| i.as_array())
            .unwrap_or(&empty_vec);

        let results = items
            .iter()
            .filter_map(|item| {
                let title = item.get("title")?.as_str()?.to_string();
                let url = item.get("link")?.as_str()?.to_string();
                let snippet = item
                    .get("snippet")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());

                Some(SearchResult {
                    title,
                    url,
                    snippet,
                    publish_date: None,
                })
            })
            .collect();

        Ok(results)
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "WebSearch"
    }

    fn description(&self) -> &str {
        "允许 Claude 搜索网络并使用结果来提供响应。\n\
         提供超出 Claude 知识截止日期的最新信息。\n\
         返回格式化为搜索结果块的搜索结果信息，包括 Markdown 超链接。\n\
         用于访问 Claude 知识截止日期之外的信息。\n\
         搜索在单个 API 调用中自动执行。"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "minLength": 2,
                    "description": "要使用的搜索查询"
                },
                "allowed_domains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "仅包含来自这些域名的结果"
                },
                "blocked_domains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "永远不包含来自这些域名的结果"
                }
            },
            "required": ["query"]
        })
    }

    async fn check_permissions(
        &self,
        _params: &serde_json::Value,
        _context: &ToolContext,
    ) -> PermissionCheckResult {
        PermissionCheckResult::allow()
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _context: &ToolContext,
    ) -> Result<ToolResult, ToolError> {
        let input: WebSearchInput = serde_json::from_value(params)
            .map_err(|e| ToolError::execution_failed(format!("输入参数解析失败: {}", e)))?;

        let query = &input.query;
        let allowed_domains = &input.allowed_domains;
        let blocked_domains = &input.blocked_domains;

        // 参数冲突验证
        if allowed_domains.is_some() && blocked_domains.is_some() {
            return Err(ToolError::execution_failed(
                "不能同时指定 allowed_domains 和 blocked_domains".to_string(),
            ));
        }

        // 生成缓存键
        let cache_key =
            WebCache::generate_search_cache_key(query, allowed_domains, blocked_domains);

        // 检查缓存
        if let Some(cached) = self.cache.get_cached_search(&cache_key) {
            let cache_age = cached
                .fetched_at
                .elapsed()
                .unwrap_or(Duration::ZERO)
                .as_secs()
                / 60; // 分钟

            let output = format!(
                "{}\n\n_[缓存结果，来自 {} 分钟前]_",
                self.format_search_results(&cached.results, query),
                cache_age
            );

            return Ok(ToolResult::success(output));
        }

        // 执行搜索
        match self.perform_search(query).await {
            Ok(raw_results) => {
                // 应用域名过滤
                let filtered_results = self.apply_domain_filters(
                    raw_results.clone(),
                    allowed_domains,
                    blocked_domains,
                );

                // 缓存结果（即使为空也缓存，避免重复请求）
                self.cache.cache_search(
                    cache_key,
                    CachedSearchResults {
                        query: query.clone(),
                        results: filtered_results.clone(),
                        fetched_at: SystemTime::now(),
                        allowed_domains: allowed_domains.clone(),
                        blocked_domains: blocked_domains.clone(),
                    },
                );

                // 如果有真实结果，格式化并返回
                if !filtered_results.is_empty() {
                    Ok(ToolResult::success(
                        self.format_search_results(&filtered_results, query),
                    ))
                } else if !raw_results.is_empty() {
                    // 如果搜索返回了结果但被过滤器全部过滤掉了
                    let allowed_str = allowed_domains
                        .as_ref()
                        .map(|d: &Vec<String>| d.join(", "))
                        .unwrap_or_else(|| "全部".to_string());
                    let blocked_str = blocked_domains
                        .as_ref()
                        .map(|d: &Vec<String>| d.join(", "))
                        .unwrap_or_else(|| "无".to_string());

                    Ok(ToolResult::success(format!(
                        "网络搜索: \"{}\"\n\n应用域名过滤器后未找到结果。\n\n应用的过滤器:\n- 允许的域名: {}\n- 阻止的域名: {}\n\n尝试调整您的域名过滤器或搜索查询。",
                        query, allowed_str, blocked_str
                    )))
                } else {
                    // 如果搜索 API 没有返回结果
                    Ok(ToolResult::success(format!(
                        "网络搜索: \"{}\"\n\n未找到结果。这可能是由于:\n1. 搜索查询过于具体或不常见\n2. DuckDuckGo Instant Answer API 覆盖范围有限\n3. 网络或 API 问题\n\n建议:\n- 尝试不同的搜索查询\n- 配置 Bing 或 Google Search API 以获得更好的结果:\n  * Bing: 设置 BING_SEARCH_API_KEY 环境变量\n  * Google: 设置 GOOGLE_SEARCH_API_KEY 和 GOOGLE_SEARCH_ENGINE_ID\n\n当前搜索提供商: DuckDuckGo Instant Answer API (免费)",
                        query
                    )))
                }
            }
            Err(e) => Err(ToolError::execution_failed(format!("搜索失败: {}", e))),
        }
    }
}

/// 缓存统计信息
pub fn get_web_cache_stats(cache: &WebCache) -> serde_json::Value {
    serde_json::json!({
        "fetch": {
            "size": cache.fetch_cache.lock().unwrap().len(),
            "capacity": cache.fetch_cache.lock().unwrap().cap(),
        },
        "search": {
            "size": cache.search_cache.lock().unwrap().len(),
            "capacity": cache.search_cache.lock().unwrap().cap(),
        }
    })
}

/// 清除所有 Web 缓存
pub fn clear_web_caches(cache: &WebCache) {
    cache.fetch_cache.lock().unwrap().clear();
    cache.search_cache.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_fetch_tool_creation() {
        let tool = WebFetchTool::new();
        assert_eq!(tool.name(), "WebFetch");
        assert!(!tool.description().is_empty());
    }

    #[tokio::test]
    async fn test_web_search_tool_creation() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.name(), "WebSearch");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_web_cache_creation() {
        let cache = WebCache::new();
        assert!(cache.fetch_cache.lock().unwrap().is_empty());
        assert!(cache.search_cache.lock().unwrap().is_empty());
    }

    #[test]
    fn test_search_cache_key_generation() {
        let key1 = WebCache::generate_search_cache_key(
            "test query",
            &Some(vec!["example.com".to_string()]),
            &None,
        );
        let key2 = WebCache::generate_search_cache_key(
            "test query",
            &Some(vec!["example.com".to_string()]),
            &None,
        );
        let key3 = WebCache::generate_search_cache_key(
            "different query",
            &Some(vec!["example.com".to_string()]),
            &None,
        );

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_domain_extraction() {
        let tool = WebSearchTool::new();

        assert_eq!(
            tool.extract_domain("https://www.example.com/path"),
            "example.com"
        );
        assert_eq!(tool.extract_domain("https://example.com"), "example.com");
        assert_eq!(
            tool.extract_domain("http://subdomain.example.com"),
            "subdomain.example.com"
        );
        assert_eq!(tool.extract_domain("invalid-url"), "");
    }

    #[test]
    fn test_domain_filtering() {
        let tool = WebSearchTool::new();
        let results = vec![
            SearchResult {
                title: "Example 1".to_string(),
                url: "https://example.com/1".to_string(),
                snippet: None,
                publish_date: None,
            },
            SearchResult {
                title: "Test 1".to_string(),
                url: "https://test.com/1".to_string(),
                snippet: None,
                publish_date: None,
            },
        ];

        // 测试白名单过滤
        let allowed = Some(vec!["example.com".to_string()]);
        let filtered = tool.apply_domain_filters(results.clone(), &allowed, &None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Example 1");

        // 测试黑名单过滤
        let blocked = Some(vec!["test.com".to_string()]);
        let filtered = tool.apply_domain_filters(results, &None, &blocked);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Example 1");
    }
}
