//! Performance Optimization Module
//!
//! Provides performance-related utilities for the core engine.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;

/// Cache entry with TTL support
pub struct CacheEntry<T> {
    pub value: T,
    pub expires_at: Instant,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl_secs: u64) -> Self {
        Self {
            value,
            expires_at: Instant::now() + Duration::from_secs(ttl_secs),
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Simple in-memory cache with TTL
pub struct TimedCache<K, V> {
    map: RwLock<HashMap<K, CacheEntry<V>>>,
    default_ttl_secs: u64,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> TimedCache<K, V> {
    pub fn new(ttl_secs: u64) -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            default_ttl_secs: ttl_secs,
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let map = self.map.read().await;
        map.get(key)
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.value.clone())
    }

    pub async fn set(&self, key: K, value: V) {
        let entry = CacheEntry::new(value, self.default_ttl_secs);
        let mut map = self.map.write().await;
        map.insert(key, entry);
    }

    pub async fn set_with_ttl(&self, key: K, value: V, ttl_secs: u64) {
        let entry = CacheEntry::new(value, ttl_secs);
        let mut map = self.map.write().await;
        map.insert(key, entry);
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut map = self.map.write().await;
        map.remove(key).map(|e| e.value)
    }

    pub async fn clear(&self) {
        let mut map = self.map.write().await;
        map.clear();
    }

    pub async fn cleanup_expired(&self) {
        let mut map = self.map.write().await;
        map.retain(|_, entry| !entry.is_expired());
    }
}

/// Connection pool for HTTP clients
pub struct HttpPool {
    client: reqwest::Client,
}

impl HttpPool {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl Default for HttpPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter for API calls
pub struct RateLimiter {
    requests: RwLock<Vec<Instant>>,
    max_requests: usize,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: RwLock::new(Vec::new()),
            max_requests,
            window_secs,
        }
    }

    pub async fn check(&self) -> bool {
        let now = Instant::now();
        let mut requests = self.requests.write().await;

        // Remove old requests outside the window
        let window_start = now - Duration::from_secs(self.window_secs);
        requests.retain(|&t| t > window_start);

        if requests.len() < self.max_requests {
            requests.push(now);
            true
        } else {
            false
        }
    }

    pub async fn wait_for_slot(&self) {
        while !self.check().await {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

/// Performance metrics
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub request_count: u64,
    pub error_count: u64,
    pub total_latency_ms: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl Metrics {
    pub fn record_request(&mut self, latency_ms: u64) {
        self.request_count += 1;
        self.total_latency_ms += latency_ms;
    }

    pub fn record_error(&mut self) {
        self.error_count += 1;
    }

    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    pub fn avg_latency_ms(&self) -> u64 {
        if self.request_count > 0 {
            self.total_latency_ms / self.request_count
        } else {
            0
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total > 0 {
            self.cache_hits as f64 / total as f64
        } else {
            0.0
        }
    }
}

/// Thread-safe metrics wrapper
pub struct MetricsCollector {
    metrics: RwLock<Metrics>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(Metrics::default()),
        }
    }

    pub async fn record_request(&self, latency_ms: u64) {
        let mut m = self.metrics.write().await;
        m.record_request(latency_ms);
    }

    pub async fn record_error(&self) {
        let mut m = self.metrics.write().await;
        m.record_error();
    }

    pub async fn record_cache_hit(&self) {
        let mut m = self.metrics.write().await;
        m.record_cache_hit();
    }

    pub async fn record_cache_miss(&self) {
        let mut m = self.metrics.write().await;
        m.record_cache_miss();
    }

    pub async fn get(&self) -> Metrics {
        self.metrics.read().await.clone()
    }

    pub async fn reset(&self) {
        let mut m = self.metrics.write().await;
        *m = Metrics::default();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Lazy initialization helper
pub struct Lazy<T> {
    cell: RwLock<Option<T>>,
    init: fn() -> T,
}

impl<T: Clone> Lazy<T> {
    pub fn new(init: fn() -> T) -> Self {
        Self {
            cell: RwLock::new(None),
            init,
        }
    }

    pub async fn get(&self) -> T {
        if let Some(v) = self.cell.read().await.clone() {
            return v;
        }

        let v = (self.init)();
        let mut cell = self.cell.write().await;
        *cell = Some(v.clone());
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache() {
        let cache: TimedCache<String, String> = TimedCache::new(1);
        
        cache.set("key1".to_string(), "value1".to_string()).await;
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, 1);
        
        assert!(limiter.check().await);
        assert!(limiter.check().await);
        assert!(!limiter.check().await);
    }

    #[tokio::test]
    async fn test_metrics() {
        let collector = MetricsCollector::new();
        
        collector.record_request(100).await;
        collector.record_cache_hit().await;
        
        let m = collector.get().await;
        assert_eq!(m.request_count, 1);
        assert_eq!(m.cache_hits, 1);
    }
}
