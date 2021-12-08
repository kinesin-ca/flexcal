Introduction
============

A flexible calendar structure to support aribtrary and calculated
off days.

Calendars
=========

Definition
----------

Calendars consist of:

- Day of week list
- Date Exclusion List
  - Dates are defined as:
    - One of
      - YYYY-MM-DD
        - Add a specific date to a calendar's exclusion list
      - Month and Day
      - Month, Day of week, and an offset (e.g. 2nd Friday)
        - Number can be relative, so 1 is first, -1 is last
    - Optional `description`
    - Optional `observed` attribute that is `Next`, `Prev`, `Closest`, `NoAdjustment` (default)
      that is in the day of week mask and not also a holiday
- A name, which can contain letters, numbers, dashes, and underscores
  - They cannot contain dots / periods
