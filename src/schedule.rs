use chrono::naive::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
struct TimeSpan {
    start: NaiveTime,
    end: NaiveTime,
    #[serde(default)]
    description: String,
}

impl TimeSpan {
    fn intersection(&self, other: &TimeSpan) -> Option<TimeSpan> {
        if self.end < other.start {
            None
        } else {
            Some(TimeSpan {
                start: if self.start < other.start {
                    other.start
                } else {
                    self.start
                },
                end: if self.end < other.end {
                    self.end
                } else {
                    other.end
                },
                description: format!(
                    "Intersection of {} and {}",
                    self.description, other.description
                ),
            })
        }
    }
}

impl PartialEq for TimeSpan {
    fn eq(&self, other: &TimeSpan) -> bool {
        self.start == other.start && self.end == other.end
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ScheduleOverride {
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    schedule: Vec<TimeSpan>,

    #[serde(default)]
    description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Schedule {
    default: Vec<TimeSpan>,
    overrides: Vec<ScheduleOverride>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timespan_intersections() {
        {
            let a = TimeSpan {
                start: NaiveTime::from_hms(0, 0, 0),
                end: NaiveTime::from_hms(2, 0, 0),
                description: "".to_owned(),
            };

            let b = TimeSpan {
                start: NaiveTime::from_hms(1, 0, 0),
                end: NaiveTime::from_hms(3, 0, 0),
                description: "".to_owned(),
            };

            let c = TimeSpan {
                start: NaiveTime::from_hms(1, 0, 0),
                end: NaiveTime::from_hms(2, 0, 0),
                description: "".to_owned(),
            };

            assert_eq!(a.intersection(&b), Some(c));
        }
    }
}
