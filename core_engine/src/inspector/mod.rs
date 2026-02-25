//! Traffic Inspector Module
//! 
//! Monitors network requests and DOM mutations to determine page readiness.
//! Blueprint 3.1: Traffic Inspector Implementation
//!
//! Key features:
//! 1. Listen to Network.requestWillBeSent -> active_requests + 1
//! 2. Listen to Network.loadingFinished/loadingFailed -> active_requests - 1
//! 3. Listen to DOM.childNodeInserted -> update last_mutation_time
//! 4. Provide async wait_until_ready(): loop until (active_requests == 0) && (DOM idle > 500ms)

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tracing::{debug, info, warn};

use crate::CoreError;

/// Traffic inspector state
#[derive(Debug, Clone, Default)]
pub struct InspectorState {
    /// Currently active network requests
    pub active_requests: u32,
    
    /// Last DOM mutation timestamp
    pub last_mutation: Option<Instant>,
    
    /// Whether page is ready
    pub is_ready: bool,
    
    /// Current URL
    pub url: Option<String>,
    
    /// Page title
    pub title: Option<String>,
}

/// Traffic inspector configuration
#[derive(Debug, Clone)]
pub struct InspectorConfig {
    /// Minimum idle time for DOM (ms)
    pub dom_idle_threshold: Duration,
    
    /// Minimum idle time for network (ms)
    pub network_idle_threshold: Duration,
    
    /// Maximum wait time (ms)
    pub max_wait_time: Duration,
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self {
            dom_idle_threshold: Duration::from_millis(500),
            network_idle_threshold: Duration::from_millis(1000),
            max_wait_time: Duration::from_secs(30),
        }
    }
}

/// Event handler trait for inspector events
pub trait InspectorEventHandler: Send + Sync {
    /// Called when a network request starts
    fn on_request_started(&self, request_id: &str, url: &str);
    
    /// Called when a network request finishes
    fn on_request_finished(&self, request_id: &str);
    
    /// Called when a network request fails
    fn on_request_failed(&self, request_id: &str, error: &str);
    
    /// Called when DOM mutates
    fn on_dom_mutated(&self);
    
    /// Called when page navigates
    fn on_navigate(&self, url: &str);
}

/// Traffic inspector for monitoring page state
pub struct TrafficInspector {
    state: RwLock<InspectorState>,
    config: InspectorConfig,
    /// Count active requests using atomic for quick checks
    active_count: AtomicU32,
}

impl TrafficInspector {
    /// Create a new inspector
    pub fn new(config: InspectorConfig) -> Self {
        Self {
            state: RwLock::new(InspectorState::default()),
            config,
            active_count: AtomicU32::new(0),
        }
    }

    /// Create with default config
    pub fn default_inspector() -> Self {
        Self::new(InspectorConfig::default())
    }

    /// Record network request started
    /// Corresponds to CDP "Network.requestWillBeSent"
    pub fn request_started(&self) {
        let count = self.active_count.fetch_add(1, Ordering::SeqCst);
        let mut state = self.state.write();
        state.active_requests = count + 1;
        state.is_ready = false;
        debug!("Network request started, active: {}", state.active_requests);
    }

    /// Record network request finished
    /// Corresponds to CDP "Network.loadingFinished"
    pub fn request_finished(&self) {
        let count = self.active_count.fetch_sub(1, Ordering::SeqCst);
        let mut state = self.state.write();
        if count > 0 {
            state.active_requests = count - 1;
        } else {
            state.active_requests = 0;
        }
        debug!("Network request finished, active: {}", state.active_requests);
        
        // Check if ready after request completes
        self.check_ready();
    }

    /// Record network request failed
    /// Corresponds to CDP "Network.loadingFailed"
    pub fn request_failed(&self) {
        self.request_finished();
    }

    /// Record DOM mutation
    /// Corresponds to CDP "DOM.childNodeInserted" or similar
    pub fn dom_mutated(&self) {
        let mut state = self.state.write();
        state.last_mutation = Some(Instant::now());
        state.is_ready = false;
        debug!("DOM mutated");
    }

    /// Update page info
    pub fn set_page_info(&self, url: Option<String>, title: Option<String>) {
        let mut state = self.state.write();
        if let Some(u) = url {
            state.url = Some(u);
        }
        if let Some(t) = title {
            state.title = Some(t);
        }
    }

    /// Record navigation (new page load)
    pub fn on_navigate(&self, url: &str) {
        let mut state = self.state.write();
        state.url = Some(url.to_string());
        state.active_requests = 0;
        state.last_mutation = Some(Instant::now());
        state.is_ready = false;
        self.active_count.store(0, Ordering::SeqCst);
        info!("Navigated to: {}", url);
    }

    /// Check if page is ready
    fn check_ready(&self) {
        let mut state = self.state.write();
        
        if state.active_requests == 0 {
            if let Some(last_mutation) = state.last_mutation {
                let idle_time = last_mutation.elapsed();
                if idle_time >= self.config.dom_idle_threshold {
                    state.is_ready = true;
                    info!("Page is now ready after {:?} of DOM idle", idle_time);
                }
            } else {
                // No mutations recorded yet, consider ready
                state.is_ready = true;
            }
        }
    }

    /// Wait until page is ready
    /// Blueprint 3.1: Core algorithm
    /// loop until (active_requests == 0) && (DOM idle > dom_idle_threshold)
    pub async fn wait_until_ready(&self) -> Result<InspectorState, CoreError> {
        let start = Instant::now();
        
        loop {
            // Quick check using atomic
            if self.active_count.load(Ordering::SeqCst) == 0 {
                let state = self.state.read();
                if state.is_ready {
                    return Ok(state.clone());
                }
                
                // Check DOM idle time
                if let Some(last_mutation) = state.last_mutation {
                    if last_mutation.elapsed() >= self.config.dom_idle_threshold {
                        drop(state);
                        let mut s = self.state.write();
                        s.is_ready = true;
                        return Ok(s.clone());
                    }
                } else {
                    // No mutations, consider ready
                    drop(state);
                    let mut s = self.state.write();
                    s.is_ready = true;
                    return Ok(s.clone());
                }
            }
            
            // Check timeout
            if start.elapsed() > self.config.max_wait_time {
                warn!("Timeout waiting for page ready");
                return Err(CoreError::Inspector("Timeout waiting for page ready".to_string()));
            }
            
            // Wait before checking again (100ms polling)
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get current state
    pub fn get_state(&self) -> InspectorState {
        self.state.read().clone()
    }

    /// Reset inspector state (for new page)
    pub fn reset(&self) {
        let mut state = self.state.write();
        state.active_requests = 0;
        state.last_mutation = Some(Instant::now());
        state.is_ready = false;
        state.url = None;
        state.title = None;
        self.active_count.store(0, Ordering::SeqCst);
        info!("Inspector state reset");
    }

    /// Check if network is idle
    pub fn is_network_idle(&self) -> bool {
        self.active_count.load(Ordering::SeqCst) == 0
    }

    /// Get active request count
    pub fn active_request_count(&self) -> u32 {
        self.active_count.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_creation() {
        let inspector = TrafficInspector::default_inspector();
        assert_eq!(inspector.active_request_count(), 0);
    }

    #[test]
    fn test_request_tracking() {
        let inspector = TrafficInspector::default_inspector();
        
        inspector.request_started();
        assert_eq!(inspector.active_request_count(), 1);
        
        inspector.request_started();
        assert_eq!(inspector.active_request_count(), 2);
        
        inspector.request_finished();
        assert_eq!(inspector.active_request_count(), 1);
        
        inspector.request_finished();
        assert_eq!(inspector.active_request_count(), 0);
    }

    #[test]
    fn test_reset() {
        let inspector = TrafficInspector::default_inspector();
        
        inspector.request_started();
        inspector.dom_mutated();
        
        inspector.reset();
        
        assert_eq!(inspector.active_request_count(), 0);
    }
}
