//! Subscription registry and helpers.

use std::collections::{HashMap, HashSet};

use crate::web::stream::filters::{
    AuditLogFilter, ScrapeJobsFilter, ScraperStatsFilter, ScraperTimeseriesFilter,
    parse_audit_log_filter, parse_scrape_jobs_filter, parse_scraper_stats_filter,
    parse_scraper_timeseries_filter,
};
use crate::web::stream::protocol::{StreamError, StreamFilter, StreamKind};

pub enum Subscription {
    ScrapeJobs {
        filter: ScrapeJobsFilter,
        known_ids: HashSet<i32>,
    },
    AuditLog {
        filter: AuditLogFilter,
    },
    ScraperStats {
        filter: ScraperStatsFilter,
    },
    ScraperTimeseries {
        filter: ScraperTimeseriesFilter,
    },
    ScraperSubjects,
}

impl Subscription {
    pub fn kind(&self) -> StreamKind {
        match self {
            Subscription::ScrapeJobs { .. } => StreamKind::ScrapeJobs,
            Subscription::AuditLog { .. } => StreamKind::AuditLog,
            Subscription::ScraperStats { .. } => StreamKind::ScraperStats,
            Subscription::ScraperTimeseries { .. } => StreamKind::ScraperTimeseries,
            Subscription::ScraperSubjects => StreamKind::ScraperSubjects,
        }
    }

    pub fn is_computed(&self) -> bool {
        matches!(
            self,
            Self::ScraperStats { .. } | Self::ScraperTimeseries { .. } | Self::ScraperSubjects
        )
    }
}

pub struct SubscriptionRegistry {
    subscriptions: HashMap<String, Subscription>,
    next_id: u64,
}

impl Default for SubscriptionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SubscriptionRegistry {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn allocate_id(&mut self) -> String {
        let id = self.next_id.to_string();
        self.next_id += 1;
        id
    }

    pub fn insert(&mut self, id: String, subscription: Subscription) {
        self.subscriptions.insert(id, subscription);
    }

    pub fn remove(&mut self, id: &str) -> Option<Subscription> {
        self.subscriptions.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&Subscription> {
        self.subscriptions.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Subscription> {
        self.subscriptions.get_mut(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Subscription)> {
        self.subscriptions.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Subscription)> {
        self.subscriptions.iter_mut()
    }

    pub fn ids_for_kind(&self, kind: StreamKind) -> Vec<String> {
        self.subscriptions
            .iter()
            .filter_map(|(id, sub)| {
                if sub.kind() == kind {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

pub fn build_subscription(
    kind: StreamKind,
    filter: Option<StreamFilter>,
) -> Result<Subscription, StreamError> {
    match kind {
        StreamKind::ScrapeJobs => {
            let filter = parse_scrape_jobs_filter(filter)?;
            Ok(Subscription::ScrapeJobs {
                filter,
                known_ids: HashSet::new(),
            })
        }
        StreamKind::AuditLog => {
            let filter = parse_audit_log_filter(filter)?;
            Ok(Subscription::AuditLog { filter })
        }
        StreamKind::ScraperStats => {
            let filter = parse_scraper_stats_filter(filter)?;
            Ok(Subscription::ScraperStats { filter })
        }
        StreamKind::ScraperTimeseries => {
            let filter = parse_scraper_timeseries_filter(filter)?;
            Ok(Subscription::ScraperTimeseries { filter })
        }
        StreamKind::ScraperSubjects => Ok(Subscription::ScraperSubjects),
    }
}
