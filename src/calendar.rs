use chrono::naive::NaiveDate;
use chrono::{Datelike, Month, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/*
  There are a few gaping holes here, and some deficiencies:

    - Holidays are only calculated within a year. If a holiday in a prior
      year is bumped to the next year, it won't be considered.
    - Holiday impact is searched forward. If there is a mix of AdjustmentPolicy's
      then some weird stuff can happen (Holidays occur A, B, but end up getting
      observed B, A)
    - No support for holiday ranges (e.g. Golden Week)
*/

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
enum AdjustmentPolicy {
    Prev,
    Next,
    Closest,
    NoAdjustment,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum DateSpec {
    SpecificDate {
        date: NaiveDate,
        description: String,
    },
    MonthDay {
        month: chrono::Month,
        day: u32,
        observed: AdjustmentPolicy,
        description: String,
        since: Option<NaiveDate>,
        until: Option<NaiveDate>,
    },
    MonthNthDay {
        month: Month,
        dow: Weekday,
        offset: i8,
        observed: AdjustmentPolicy,
        description: String,
        since: Option<NaiveDate>,
        until: Option<NaiveDate>,
    },
}

impl DateSpec {
    /// Get exact date of event and its policy, if it occured in that year
    fn resolve(&self, year: i32) -> Option<(NaiveDate, AdjustmentPolicy)> {
        use DateSpec::*;

        match self {
            SpecificDate { date, .. } => Some((*date, AdjustmentPolicy::NoAdjustment)),
            MonthDay {
                month,
                day,
                since,
                until,
                observed,
                ..
            } => {
                if since.is_some() && since.unwrap().year() > year {
                    None
                } else if until.is_some() && until.unwrap().year() < year {
                    None
                } else {
                    Some((
                        NaiveDate::from_ymd(year, month.number_from_month(), *day),
                        *observed,
                    ))
                }
            }
            MonthNthDay {
                month,
                dow,
                offset,
                since,
                until,
                observed,
                ..
            } => {
                if since.is_some() && since.unwrap().year() > year {
                    None
                } else if until.is_some() && until.unwrap().year() < year {
                    None
                } else {
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
                        Some((date, *observed))
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
                        Some((date, *observed))
                    }
                }
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
struct Calendar {
    description: String,
    dow: HashSet<Weekday>,
    public: bool,
    excluded: Vec<DateSpec>,
    inherits: Vec<String>,
}

impl Calendar {
    fn adjust_holidays(&self, holidays: &Vec<(NaiveDate, AdjustmentPolicy)>) -> HashSet<NaiveDate> {
        let mut observed = HashSet::new();

        for (date, policy) in holidays.iter() {
            match self.adjust_holiday(*date, policy, &observed) {
                Some(holiday) => {
                    observed.insert(holiday);
                }
                None => {}
            }
        }

        observed
    }

    /// Adjust a date given existing holidays and adjustment policy
    fn adjust_holiday(
        &self,
        date: NaiveDate,
        policy: &AdjustmentPolicy,
        holidays: &HashSet<NaiveDate>,
    ) -> Option<NaiveDate> {
        let mut actual = date.clone();

        println!("Adjusting {:?}", date);

        let is_blocked =
            |x: NaiveDate| -> bool { (!self.dow.contains(&x.weekday())) || holidays.contains(&x) };

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
                    .adjust_holiday(date, &AdjustmentPolicy::Prev, holidays)
                    .unwrap();
                let next = self
                    .adjust_holiday(date, &AdjustmentPolicy::Next, holidays)
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

    /// Get the set of all holidays in a given year
    pub fn get_holidays(&self, date: NaiveDate) -> HashSet<NaiveDate> {
        let holidays: Vec<(NaiveDate, AdjustmentPolicy)> = self
            .excluded
            .iter()
            .map(|x| x.resolve(date.year()))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();
        self.adjust_holidays(&holidays)
    }

    /// Returns true if the given date is a holiday / non-business day
    fn is_holiday(&self, date: NaiveDate) -> bool {
        self.get_holidays(date).contains(&date)
    }

    /// Returns the set of valid calendar dates within the specified range
    pub fn date_range(&self, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
        let mut result = Vec::new();
        let mut cur = from.pred();
        let year = from.year();
        let mut holidays = self.get_holidays(cur);

        while cur <= to {
            cur = cur.succ();
            if cur.year() != year {
                holidays = self.get_holidays(cur);
            }
            if !self.dow.contains(&cur.weekday()) || holidays.contains(&cur) {
                continue;
            }
            result.push(cur);
        }
        result
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
            excluded: vec![
                DateSpec::MonthDay {
                    month: December,
                    day: 25u32,
                    observed: AdjustmentPolicy::Next,
                    description: "Christmas".to_owned(),
                    since: None,
                    until: None,
                },
                DateSpec::MonthDay {
                    month: December,
                    day: 26u32,
                    observed: AdjustmentPolicy::Next,
                    description: "Boxing Day".to_owned(),
                    since: None,
                    until: None,
                },
            ],
            inherits: vec![],
        };

        // Christmas falls on a Saturday, observed was Monday
        assert!(cal.is_holiday(NaiveDate::from_ymd(2021, 12, 27)));

        // Boxing Day falls on a Sunday, observed is a Tuesday
        assert!(cal.is_holiday(NaiveDate::from_ymd(2021, 12, 28)));

        assert!(!cal.is_holiday(NaiveDate::from_ymd(2021, 12, 24)));
    }

    #[test]
    fn check_crossing_years() {
        use chrono::Month::*;
        use chrono::Weekday::*;

        let cal = Calendar {
            description: "Test description".to_owned(),
            dow: HashSet::from([Mon, Tue, Wed, Thu, Fri]),
            public: false,
            excluded: vec![
                DateSpec::MonthDay {
                    month: December,
                    day: 25u32,
                    observed: AdjustmentPolicy::Next,
                    description: "Christmas".to_owned(),
                    since: None,
                    until: None,
                },
                DateSpec::MonthDay {
                    month: December,
                    day: 26u32,
                    observed: AdjustmentPolicy::Next,
                    description: "Boxing Day".to_owned(),
                    since: None,
                    until: None,
                },
                DateSpec::MonthDay {
                    month: January,
                    day: 1u32,
                    observed: AdjustmentPolicy::Next,
                    description: "New Years Day".to_owned(),
                    since: None,
                    until: None,
                },
            ],
            inherits: vec![],
        };

        let myrange = cal.date_range(
            NaiveDate::from_ymd(2021, 12, 15),
            NaiveDate::from_ymd(2022, 01, 15),
        );
        assert_eq!(myrange.len(), 20);
    }
}
