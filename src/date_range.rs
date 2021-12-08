use chrono::naive::NaiveDate;

pub struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        DateRange { start, end }
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        self.start <= date && date < self.end
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

pub struct DateRangeIterator {
    cur: NaiveDate,
    end: NaiveDate,
}

impl Iterator for DateRangeIterator {
    type Item = NaiveDate;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.end {
            None
        } else {
            let ret = self.cur;
            self.cur = self.cur.succ();
            Some(ret)
        }
    }
}

impl IntoIterator for DateRange {
    type Item = NaiveDate;
    type IntoIter = DateRangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        DateRangeIterator {
            cur: self.start,
            end: self.end,
        }
    }
}

impl IntoIterator for &DateRange {
    type Item = NaiveDate;
    type IntoIter = DateRangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        DateRangeIterator {
            cur: self.start,
            end: self.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_bounds() {
        let dr = DateRange {
            start: NaiveDate::from_ymd(2021, 1, 1),
            end: NaiveDate::from_ymd(2021, 2, 1),
        };

        assert!(!dr.is_empty());
        assert!(dr.contains(NaiveDate::from_ymd(2021, 1, 1)));
        assert!(dr.contains(NaiveDate::from_ymd(2021, 1, 15)));
        assert!(!dr.contains(NaiveDate::from_ymd(2021, 2, 1)));
    }

    #[test]
    fn test_range_iteration() {
        let dr = DateRange {
            start: NaiveDate::from_ymd(2021, 1, 1),
            end: NaiveDate::from_ymd(2021, 2, 1),
        };

        let mut cnt = 0;
        for _ in dr {
            cnt += 1;
        }
        assert_eq!(cnt, 31);
    }
}
