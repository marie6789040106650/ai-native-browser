//! Traffic Inspector Module
//! 
//! Monitors network requests and DOM mutations to determine page readiness.

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tokio::sync::mpsc;
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

/// Traffic inspector for monitoring page state
pub struct TrafficInspector {
    state: RwLock<InspectorState>,
    config: InspectorConfig,
}

impl TrafficInspector {
    /// Create a new inspector
    pub fn new(config: InspectorConfig) -> Self {
        Self {
            state: RwLock::new(InspectorState::default()),
            config,
        }
    }

    /// Create with default config
    pub fn default_inspector() -> Self {
        Self::new(InspectorConfig::default())
    }

    /// Record network request started
    pub fn request_started(&self) {
        let mut state = self.state.write();
        state.active_requests += 1;
        state.is_ready = false;
        debug!("Network request started, active: {}", state.active_requests);
    }

    /// Record network request finished
    pub fn request_finished(&self) {
        let mut state = self.state.write();
        if state.active_requests > 0 {
            state.active_requests -= 1;
        }
        debug!("Network request finished, active: {}", state.active_requests);
        
        // Check if ready after request completes
        self.check_ready();
    }

    /// Record network request failed
    pub fn request_failed(&self) {
        self.request_finished();
    }

    /// Record DOM mutation
    pub fn dom_mutated(&self) {
        let mut state = self.state.write();
        state.last_mutation = Some(Instant::now());
        state.is_ready = false;
        debug!("DOM mutated");
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
    pub async fn wait_until_ready(&self) -> Result<InspectorState, CoreError> {
        let start = Instant::now();
        
        loop {
            {
                let state = self.state.read();
                if state.is_ready {
                    return Ok(state.clone());
                }
            }
            
            // Check timeout
            if start.elapsed() > self.config.max_wait_time {
                warn!("Timeout waiting for page ready");
                return Err(CoreError::Inspector("Timeout waiting for page ready".to_string()));
            }
            
            // Check network idle
            {
                let state = self.state.read();
                if state.active_requests == 0 {
                    // Network is idle, check DOM
                    if let Some(last_mutation) = state.last_mutation {
                        if last_mutation.elapsed() >= self.config.dom_idle_threshold {
                            let mut state = self.state.write();
                            state.is_ready = true;
                            return Ok(state.clone());
                        }
                    } else {
                        // No mutations, consider ready
                        let mut state = self.state.write();
                        state.is_ready = true;
                        return Ok(state.clone());
                    }
                }
            }
            
            // Wait before checking again
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get current state
    pub fn get_state(&self) -> InspectorState {
        self.state.read().clone()
    }

    /// Reset inspector state
    pub fn reset(&self) {
        let mut state = self.state.write();
        state.active_requests = 0;
        state.last_mutation = None;
        state.is_ready = false;
        info!("Inspector state reset");
    }

    /// Check if network is idle
    pub fn is_network_idle(&self) -> bool {
        self.state.read().active_requests == 0
    }

    /// Get active request count
    pub fn active_request_count(&self) -> u32 {
        self.state.read().active_requests
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
