use crate::date_range::DateRange;
use chrono::naive::NaiveDate;
use chrono::{Datelike, Month, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/*
    - offday impact is searched forward. If there is a mix of AdjustmentPolicies
      then some weird stuff can happen (offdays occur A, B, but end up getting
      observed B, A)
*/

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum AdjustmentPolicy {
    Prev,
    Next,
    Closest,
    NoAdjustment,
}

impl Default for AdjustmentPolicy {
    fn default() -> AdjustmentPolicy {
        AdjustmentPolicy::NoAdjustment
    }
}

fn default_earliest_date() -> NaiveDate {
    chrono::naive::MIN_DATE
}
fn default_latest_date() -> NaiveDate {
    chrono::naive::MAX_DATE
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum DateSpec {
    SpecificDate {
        date: NaiveDate,
        #[serde(default)]
        description: String,
    },
    DayOfMonth {
        month: chrono::Month,
        day: u32,
        #[serde(default)]
        observed: AdjustmentPolicy,
        #[serde(default)]
        description: String,
        #[serde(default = "default_earliest_date")]
        valid_since: NaiveDate,
        #[serde(default = "default_latest_date")]
        valid_until: NaiveDate,
    },
    NthDayOccurance {
        month: Month,
        dow: Weekday,
        offset: i8,
        #[serde(default)]
        observed: AdjustmentPolicy,
        #[serde(default)]
        description: String,
        #[serde(default = "default_earliest_date")]
        valid_since: NaiveDate,
        #[serde(default = "default_latest_date")]
        valid_until: NaiveDate,
    },
}

impl DateSpec {
    /// Get exact date of event and its policy, if it occured in that year
    fn resolve(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Option<(Vec<NaiveDate>, AdjustmentPolicy)> {
        use DateSpec::*;

        match self {
            SpecificDate { date, .. } => {
                if *date < start || end < *date {
                    None
                } else {
                    Some((vec![*date], AdjustmentPolicy::NoAdjustment))
                }
            }
            DayOfMonth {
                month,
                day,
                valid_since,
                valid_until,
                observed,
                ..
            } => {
                if *valid_since > end || *valid_until < start {
                    None
                } else {
                    let s = if *valid_since < start {
                        start
                    } else {
                        *valid_since
                    }
                    .year();

                    let e = if *valid_until < end {
                        *valid_until
                    } else {
                        end
                    }
                    .year();

                    let mut result = Vec::new();
                    for year in s..(e + 1) {
                        result.push(NaiveDate::from_ymd(year, month.number_from_month(), *day));
                    }
                    Some((result, *observed))
                }
            }
            NthDayOccurance {
                month,
                dow,
                offset,
                valid_since,
                valid_until,
                observed,
                ..
            } => {
                if *valid_since > end || *valid_until < start {
                    None
                } else {
                    let s = if *valid_since < start {
                        start
                    } else {
                        *valid_since
                    }
                    .year();

                    let e = if *valid_until < end {
                        *valid_until
                    } else {
                        end
                    }
                    .year();

                    let mut result = Vec::new();
                    for year in s..(e + 1) {
                        let month_num = month.number_from_month();
                        if *offset < 0i8 {
                            let mut date = if month_num == 12 {
                                NaiveDate::from_ymd(year, 12, 31)
                            } else {
                                NaiveDate::from_ymd(year, month_num + 1, 1).pred()
                            };
                            while date.weekday() != *dow {
                                date = date.pred()
                            }
                            let mut off = offset + 1;
                            while off < 0 {
                                date = date.checked_sub_signed(chrono::Duration::days(7)).unwrap();
                                off += 1;
                            }
                            result.push(date);
                        } else {
                            let mut date = NaiveDate::from_ymd(year, month_num, 1);
                            while date.weekday() != *dow {
                                date = date.succ()
                            }
                            let mut off = offset - 1;
                            while off > 0 {
                                date = date.checked_add_signed(chrono::Duration::days(7)).unwrap();
                                off -= 1;
                            }
                            result.push(date);
                        }
                    }

                    Some((result, *observed))
                }
            }
        }
    }
}

fn default_dow_set() -> HashSet<Weekday> {
    use Weekday::*;
    HashSet::from([Mon, Tue, Wed, Thu, Fri])
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct Calendar {
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_dow_set")]
    pub dow: HashSet<Weekday>,
    #[serde(default)]
    pub public: bool,
    #[serde(default)]
    pub exclude: Vec<DateSpec>,
    #[serde(default)]
    pub inherits: Vec<String>,
}

impl Calendar {
    fn adjust_special_offdays(
        &self,
        offdays: &Vec<(Vec<NaiveDate>, AdjustmentPolicy)>,
    ) -> HashSet<NaiveDate> {
        let mut observed = HashSet::new();

        for (dates, policy) in offdays.iter() {
            for date in dates.iter() {
                match self.adjust_special_offday(*date, policy, &observed) {
                    Some(offday) => {
                        observed.insert(offday);
                    }
                    None => {}
                }
            }
        }

        observed
    }

    /// Adjust a date given existing offday and adjustment policy
    fn adjust_special_offday(
        &self,
        date: NaiveDate,
        policy: &AdjustmentPolicy,
        offdays: &HashSet<NaiveDate>,
    ) -> Option<NaiveDate> {
        let mut actual = date.clone();

        println!("Adjusting {:?}", date);

        let is_blocked =
            |x: NaiveDate| -> bool { (!self.dow.contains(&x.weekday())) || offdays.contains(&x) };

        use AdjustmentPolicy::*;
        match policy {
            Next => {
                while is_blocked(actual) {
                    actual = actual.succ();
                }
                Some(actual)
            }
            Prev => {
                while is_blocked(actual) {
                    actual = actual.pred();
                }
                Some(actual)
            }
            Closest => {
                let prev = self
                    .adjust_special_offday(date, &AdjustmentPolicy::Prev, offdays)
                    .unwrap();
                let next = self
                    .adjust_special_offday(date, &AdjustmentPolicy::Next, offdays)
                    .unwrap();
                if (date - prev) < (next - date) {
                    Some(prev)
                } else {
                    Some(next)
                }
            }
            NoAdjustment => {
                if is_blocked(actual) {
                    None
                } else {
                    Some(actual)
                }
            }
        }
    }

    /// Get the set of all offdays in a given year
    pub fn get_special_offdays(&self, start: NaiveDate, end: NaiveDate) -> HashSet<NaiveDate> {
        let offdays: Vec<(Vec<NaiveDate>, AdjustmentPolicy)> = self
            .exclude
            .iter()
            .map(|x| x.resolve(start, end))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();
        self.adjust_special_offdays(&offdays)
    }

    /// Returns true if the given date is a offday / non-business day
    fn is_offday(&self, date: NaiveDate) -> bool {
        !self.dow.contains(&date.weekday()) || self.get_special_offdays(date, date).contains(&date)
    }

    /// Returns the set of non-offday calendar dates within the specified range
    pub fn date_range(&self, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
        let offdays = self.get_special_offdays(from, to);

        // Expand by 2 weeks on each side to allow for adjustments in
        // out-of-scope periods to affect in-scope dates
        let dr = DateRange::new(from - chrono::Duration::days(14), to.succ());

        dr.into_iter()
            .filter(|x| self.dow.contains(&x.weekday()) && !offdays.contains(x))
            .filter(|x| *x >= from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_double_observed() {
        use chrono::Month::*;
        use chrono::Weekday::*;

        let cal = Calendar {
            description: "Test description".to_owned(),
            dow: HashSet::from([Mon, Tue, Wed, Thu, Fri]),
            public: false,
            exclude: vec![
                DateSpec::DayOfMonth {
                    month: December,
                    day: 25u32,
                    observed: AdjustmentPolicy::Next,
                    description: "Christmas".to_owned(),
                    valid_since: default_earliest_date(),
                    valid_until: default_latest_date(),
                },
                DateSpec::DayOfMonth {
                    month: December,
                    day: 26u32,
                    observed: AdjustmentPolicy::Next,
                    description: "Boxing Day".to_owned(),
                    valid_since: default_earliest_date(),
                    valid_until: default_latest_date(),
                },
            ],
            inherits: vec![],
        };

        // Christmas falls on a Saturday, observed was Monday
        assert!(cal.is_offday(NaiveDate::from_ymd(2021, 12, 27)));

        // Boxing Day falls on a Sunday, observed is a Tuesday
        assert!(cal.is_offday(NaiveDate::from_ymd(2021, 12, 28)));

        assert!(!cal.is_offday(NaiveDate::from_ymd(2021, 12, 24)));
    }

    #[test]
    fn check_crossing_years() {
        use chrono::Month::*;
        use chrono::Weekday::*;

        let cal = Calendar {
            description: "Test description".to_owned(),
            dow: HashSet::from([Mon, Tue, Wed, Thu, Fri]),
            public: false,
            exclude: vec![
                DateSpec::DayOfMonth {
                    month: December,
                    day: 25u32,
                    observed: AdjustmentPolicy::Next,
                    description: "Christmas".to_owned(),
                    valid_since: default_earliest_date(),
                    valid_until: default_latest_date(),
                },
                DateSpec::DayOfMonth {
                    month: December,
                    day: 26u32,
                    observed: AdjustmentPolicy::Next,
                    description: "Boxing Day".to_owned(),
                    valid_since: default_earliest_date(),
                    valid_until: default_latest_date(),
                },
                DateSpec::DayOfMonth {
                    month: January,
                    day: 1u32,
                    observed: AdjustmentPolicy::Next,
                    description: "New Years Day".to_owned(),
                    valid_since: default_earliest_date(),
                    valid_until: default_latest_date(),
                },
            ],
            inherits: vec![],
        };

        assert!(cal.is_offday(NaiveDate::from_ymd(2021, 12, 25)));

        let myrange = cal.date_range(
            NaiveDate::from_ymd(2021, 12, 15),
            NaiveDate::from_ymd(2022, 01, 15),
        );
        assert_eq!(myrange.len(), 20);
    }

    #[test]
    fn test_deserialization() {
        let data = r#"
            {
            "description": "Long description",
            "dow": ["Mon","Tue","Wed","Thu","Fri", "Sat", "Sun"],
            "public": true,
            "exclude": [
                {
                    "type": "SpecificDate",
                    "date": "2021-01-01",
                    "description": "New Years Day"
                },
                {
                    "type": "DayOfMonth",
                    "month": "January",
                    "day": 17,
                    "observed": "Closest",
                    "description": "Martin Luther King Day"
                },
                {
                    "type": "NthDayOccurance",
                    "month": "January",
                    "dow": "Mon",
                    "offset": -1,
                    "observed": "Closest",
                    "description": "Final Monday Margarita Day"
                }
            ]
            }"#;
        let res: serde_json::Result<Calendar> = serde_json::from_str(data);
        assert!(res.is_ok());
    }
}
