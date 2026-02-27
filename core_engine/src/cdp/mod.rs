//! CDP (Chrome DevTools Protocol) Client Module
//!
//! Using chromiumoxide for real browser control.

use std::sync::Arc;
use std::thread;
use parking_lot::RwLock;
use tracing::{info, error};
use chromiumoxide_cdp::cdp::js_protocol::runtime::EvaluateParams;
use chromiumoxide::{Browser, BrowserConfig};
use tokio::sync::mpsc;

use crate::CoreError;

/// CDP Client with real chromiumoxide browser
#[derive(Clone)]
pub struct CdpClient {
    inner: Arc<RwLock<CdpClientInner>>,
    // Browser instance (kept in separate thread)
    browser_handle: Arc<RwLock<Option<BrowserHandle>>>,
    // Shared content storage (for async browser thread to write to)
    content_cache: Arc<RwLock<String>>,
}

struct CdpClientInner {
    current_url: String,
    current_title: String,
    current_content: String,
}

struct BrowserHandle {
    #[allow(dead_code)]
    thread: thread::JoinHandle<()>,
    /// Channel to send commands to browser thread
    cmd_tx: mpsc::Sender<BrowserCmd>,
}

enum BrowserCmd {
    Navigate(String, mpsc::Sender<Result<String, CoreError>>),
    Click(String, mpsc::Sender<Result<(), CoreError>>),
    Type(String, String, mpsc::Sender<Result<(), CoreError>>),
    Scroll(i64, mpsc::Sender<Result<(), CoreError>>),
    Evaluate(String, mpsc::Sender<Result<String, CoreError>>),
    Content(mpsc::Sender<Result<String, CoreError>>),
    Close,
}

impl CdpClient {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CdpClientInner {
                current_url: String::new(),
                current_title: String::new(),
                current_content: String::new(),
            })),
            browser_handle: Arc::new(RwLock::new(None)),
            content_cache: Arc::new(RwLock::new(String::new())),
        }
    }

    /// Launch browser in a separate thread with its own tokio runtime
    pub fn launch(&self, headless: bool, sandbox: bool) -> Result<(), CoreError> {
        info!("[CDP] Launch browser (headless={}, sandbox={})", headless, sandbox);
        
        // Create channel for commands
        let (cmd_tx, cmd_rx) = mpsc::channel::<BrowserCmd>(32);
        
        // Clone content cache for browser thread
        let content_cache = self.content_cache.clone();
        
        // Spawn browser thread with its own tokio runtime
        let browser_thread = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");
            
            rt.block_on(async {
                // Build browser config
                let builder = BrowserConfig::builder();
                let mut builder = if !headless {
                    builder.with_head()
                } else {
                    builder
                };
                let mut builder = if !sandbox {
                    builder.no_sandbox()
                } else {
                    builder
                };
                
                match builder.build() {
                    Ok(config) => {
                        match Browser::launch(config).await {
                            Ok((browser, _handler)) => {
                                info!("[Browser Thread] Browser launched successfully");
                                
                                // Create page - chromiumoxide 0.9 returns Page directly
                                match browser.new_page("about:blank").await {
                                    Ok(page) => {
                                        info!("[Browser Thread] Page created");
                                        
                                        // Handle commands
                                        let mut rx = cmd_rx;
                                        while let Some(cmd) = rx.recv().await {
                                            match cmd {
                                                BrowserCmd::Navigate(url, resp) => {
                                                    match page.goto(&url).await {
                                                        Ok(_) => {
                                                            let title = page.get_title().await.ok().flatten().unwrap_or_default();
                                                            let _ = resp.send(Ok(title));
                                                        }
                                                        Err(e) => {
                                                            error!("[Browser Thread] Navigate error: {}", e);
                                                            let _ = resp.send(Err(CoreError::Browser(e.to_string())));
                                                        }
                                                    }
                                                }
                                                BrowserCmd::Click(selector, resp) => {
                                                    // Click via JS evaluation
                                                    let js = format!(
                                                        "document.querySelector('{}').click()",
                                                        selector
                                                    );
                                                    let params = EvaluateParams::builder()
                                                        .expression(&js)
                                                        .build()
                                                        .unwrap();
                                                    match page.evaluate_expression(params).await {
                                                        Ok(_) => { let _ = resp.send(Ok(())).await; }
                                                        Err(e) => {
                                                            error!("[Browser Thread] Click error: {}", e);
                                                            let _ = resp.send(Err(CoreError::Browser(e.to_string()))).await;
                                                        }
                                                    }
                                                }
                                                BrowserCmd::Type(selector, text, resp) => {
                                                    let js = format!(
                                                        "document.querySelector('{}').value = '{}'",
                                                        selector,
                                                        text.replace("'", "\\'")
                                                    );
                                                    let params = EvaluateParams::builder()
                                                        .expression(&js)
                                                        .build()
                                                        .unwrap();
                                                    match page.evaluate_expression(params).await {
                                                        Ok(_) => { let _ = resp.send(Ok(())).await; }
                                                        Err(e) => {
                                                            error!("[Browser Thread] Type error: {}", e);
                                                            let _ = resp.send(Err(CoreError::Browser(e.to_string()))).await;
                                                        }
                                                    }
                                                }
                                                BrowserCmd::Scroll(pixels, resp) => {
                                                    let js = format!("window.scrollBy(0, {})", pixels);
                                                    let params = EvaluateParams::builder()
                                                        .expression(&js)
                                                        .build()
                                                        .unwrap();
                                                    match page.evaluate_expression(params).await {
                                                        Ok(_) => { let _ = resp.send(Ok(())).await; }
                                                        Err(e) => {
                                                            error!("[Browser Thread] Scroll error: {}", e);
                                                            let _ = resp.send(Err(CoreError::Browser(e.to_string()))).await;
                                                        }
                                                    }
                                                }
                                                BrowserCmd::Evaluate(js_code, resp) => {
                                                    let params = EvaluateParams::builder()
                                                        .expression(&js_code)
                                                        .build()
                                                        .unwrap();
                                                    match page.evaluate_expression(params).await {
                                                        Ok(result) => {
                                                            let text = result.value()
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_default();
                                                            let _ = resp.send(Ok(text));
                                                        }
                                                        Err(e) => {
                                                            error!("[Browser Thread] Evaluate error: {}", e);
                                                            let _ = resp.send(Err(CoreError::Browser(e.to_string())));
                                                        }
                                                    }
                                                }
                                                BrowserCmd::Content(resp) => {
                                                    let params = EvaluateParams::builder()
                                                        .expression("document.body.innerHTML")
                                                        .build()
                                                        .unwrap();
                                                    match page.evaluate_expression(params).await {
                                                        Ok(result) => {
                                                            let html = result.value()
                                                                .map(|v| v.to_string())
                                                                .unwrap_or_default();
                                                            // Also update shared cache
                                                            {
                                                                let mut cache = content_cache.write();
                                                                *cache = html.clone();
                                                            }
                                                            let _ = resp.send(Ok(html));
                                                        }
                                                        Err(e) => {
                                                            error!("[Browser Thread] Content error: {}", e);
                                                            let _ = resp.send(Err(CoreError::Browser(e.to_string())));
                                                        }
                                                    }
                                                }
                                                BrowserCmd::Close => {
                                                    info!("[Browser Thread] Close command received");
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("[Browser Thread] Failed to create page: {}", e);
                                    }
                                }
                                
                                info!("[Browser Thread] Shutting down browser");
                            }
                            Err(e) => {
                                error!("[Browser Thread] Failed to launch browser: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("[Browser Thread] Failed to build config: {}", e);
                    }
                }
            });
        });
        
        // Store handle
        {
            let mut handle = self.browser_handle.write();
            *handle = Some(BrowserHandle {
                thread: browser_thread,
                cmd_tx,
            });
        }
        
        // Update state
        {
            let mut inner = self.inner.write();
            inner.current_url = "about:blank".to_string();
        }
        
        info!("[CDP] Browser launch initiated");
        Ok(())
    }

    /// Navigate to URL - async via channel
    pub fn navigate(&self, url: &str) -> Result<(), CoreError> {
        info!("[CDP] Navigate to: {}", url);
        
        let (resp_tx, _) = mpsc::channel(1);
        
        // Send command to browser thread
        {
            let handle = self.browser_handle.read();
            if let Some(ref h) = *handle {
                let cmd = BrowserCmd::Navigate(url.to_string(), resp_tx);
                let _ = h.cmd_tx.try_send(cmd);
            } else {
                return Err(CoreError::Browser("Browser not running".to_string()));
            }
        }
        
        // Update state immediately (actual result comes async)
        {
            let mut inner = self.inner.write();
            inner.current_url = url.to_string();
            inner.current_title = "Loading...".to_string();
        }
        
        Ok(())
    }

    /// Get current URL
    pub fn get_url(&self) -> String {
        let inner = self.inner.read();
        inner.current_url.clone()
    }

    /// Get current page title
    pub fn get_title(&self) -> String {
        let inner = self.inner.read();
        inner.current_title.clone()
    }

    /// Get page content (from cached state)
    pub fn content(&self) -> String {
        let inner = self.inner.read();
        inner.current_content.clone()
    }

    /// Refresh and get page content from real browser
    /// This triggers a content fetch and waits briefly for the result
    pub fn refresh_content(&self) -> Result<String, CoreError> {
        info!("[CDP] Refreshing content from browser");
        
        let (resp_tx, _) = mpsc::channel(1);
        
        let handle = self.browser_handle.read();
        if let Some(ref h) = *handle {
            // Send content request to browser
            let cmd = BrowserCmd::Content(resp_tx);
            let _ = h.cmd_tx.try_send(cmd);
        } else {
            return Err(CoreError::Browser("Browser not running".to_string()));
        }
        drop(handle);
        
        // Give browser thread time to respond (brief sleep)
        // This is a hack - proper solution would be async
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Read from shared cache (browser thread updates it)
        let cache = self.content_cache.read();
        let content = cache.clone();
        drop(cache);
        
        // Update inner state
        let mut inner = self.inner.write();
        inner.current_content = content.clone();
        
        Ok(content)
    }

    /// Click element
    pub fn click(&self, selector: &str) -> Result<(), CoreError> {
        info!("[CDP] Click: {}", selector);
        
        let (resp_tx, _) = mpsc::channel(1);
        
        let handle = self.browser_handle.read();
        if let Some(ref h) = *handle {
            let cmd = BrowserCmd::Click(selector.to_string(), resp_tx);
            let _ = h.cmd_tx.try_send(cmd);
        } else {
            return Err(CoreError::Browser("Browser not running".to_string()));
        }
        
        Ok(())
    }

    /// Type text into element
    pub fn type_text(&self, selector: &str, text: &str) -> Result<(), CoreError> {
        info!("[CDP] Type into {}: {}", selector, text);
        
        let (resp_tx, _) = mpsc::channel(1);
        
        let handle = self.browser_handle.read();
        if let Some(ref h) = *handle {
            let cmd = BrowserCmd::Type(selector.to_string(), text.to_string(), resp_tx);
            let _ = h.cmd_tx.try_send(cmd);
        } else {
            return Err(CoreError::Browser("Browser not running".to_string()));
        }
        
        Ok(())
    }

    /// Scroll page
    pub fn scroll(&self, pixels: i64) -> Result<(), CoreError> {
        info!("[CDP] Scroll: {} pixels", pixels);
        
        let (resp_tx, _) = mpsc::channel(1);
        
        let handle = self.browser_handle.read();
        if let Some(ref h) = *handle {
            let cmd = BrowserCmd::Scroll(pixels, resp_tx);
            let _ = h.cmd_tx.try_send(cmd);
        } else {
            return Err(CoreError::Browser("Browser not running".to_string()));
        }
        
        Ok(())
    }

    /// Evaluate JavaScript
    pub fn evaluate(&self, js: &str) -> Result<(), CoreError> {
        info!("[CDP] Evaluate: {}", js);
        
        let (resp_tx, _) = mpsc::channel(1);
        
        let handle = self.browser_handle.read();
        if let Some(ref h) = *handle {
            let cmd = BrowserCmd::Evaluate(js.to_string(), resp_tx);
            let _ = h.cmd_tx.try_send(cmd);
        } else {
            return Err(CoreError::Browser("Browser not running".to_string()));
        }
        
        Ok(())
    }

    /// Close browser
    pub fn close(&self) {
        info!("[CDP] Close browser");
        
        let mut handle = self.browser_handle.write();
        if let Some(h) = handle.take() {
            let _ = h.cmd_tx.try_send(BrowserCmd::Close);
        }
        
        let mut inner = self.inner.write();
        inner.current_url = String::new();
    }

    /// Check if browser is running
    pub fn is_running(&self) -> bool {
        let handle = self.browser_handle.read();
        handle.is_some()
    }
}

impl Default for CdpClient {
    fn default() -> Self { Self::new() }
}
