use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use serde::Serialize;
use ts_rs::TS;

/// Health status of a service.
#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ServiceStatus {
    Starting,
    Active,
    Connected,
    Disabled,
    Error,
}

/// A timestamped status entry for a service.
#[derive(Debug, Clone)]
pub struct StatusEntry {
    pub status: ServiceStatus,
    pub updated_at: Instant,
}

/// Thread-safe registry for services to self-report their health status.
#[derive(Debug, Clone, Default)]
pub struct ServiceStatusRegistry {
    inner: Arc<DashMap<String, StatusEntry>>,
}

impl ServiceStatusRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or updates the status for a named service.
    pub fn set(&self, name: &str, status: ServiceStatus) {
        self.inner.insert(
            name.to_owned(),
            StatusEntry {
                status,
                updated_at: Instant::now(),
            },
        );
    }

    /// Returns the current status of a named service, if present.
    pub fn get(&self, name: &str) -> Option<ServiceStatus> {
        self.inner.get(name).map(|entry| entry.status.clone())
    }

    /// Returns a snapshot of all service statuses.
    pub fn all(&self) -> Vec<(String, ServiceStatus)> {
        self.inner
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().status.clone()))
            .collect()
    }
}
