use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub operator: String,
    pub provider: String,
    pub service: String,
    pub action: String,
    pub target: String,
    pub status: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct AuditTrail {
    events: Arc<RwLock<VecDeque<AuditEvent>>>,
    max_capacity: usize,
}

impl AuditTrail {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(VecDeque::with_capacity(capacity))),
            max_capacity: capacity,
        }
    }

    pub fn record(
        &self,
        operator: &str,
        provider: &str,
        service: &str,
        action: &str,
        target: &str,
        status: &str,
        details: &str,
    ) {
        let mut list = self.events.write().unwrap();
        if list.len() >= self.max_capacity {
            list.pop_back();
        }
        let event = AuditEvent {
            id: format!("audit_{}_{}", Utc::now().timestamp_millis(), list.len()),
            timestamp: Utc::now(),
            operator: operator.to_string(),
            provider: provider.to_string(),
            service: service.to_string(),
            action: action.to_string(),
            target: target.to_string(),
            status: status.to_string(),
            details: details.to_string(),
        };
        list.push_front(event);
    }

    pub fn list(&self, limit: usize) -> Vec<AuditEvent> {
        let list = self.events.read().unwrap();
        list.iter().take(limit).cloned().collect()
    }
}

impl Default for AuditTrail {
    fn default() -> Self {
        Self::new(1000)
    }
}
