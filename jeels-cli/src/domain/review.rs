use crate::domain::value_objects::{Rating, Interval};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    id: Ulid,
    rating: Rating,
    timestamp: DateTime<Utc>,
    interval: Interval,
}

impl Review {
    pub fn new(rating: Rating, interval: Interval) -> Self {
        Self {
            id: Ulid::new(),
            rating,
            timestamp: Utc::now(),
            interval,
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn rating(&self) -> Rating {
        self.rating
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn interval(&self) -> Interval {
        self.interval
    }
}
