Introduction
============

Easier job scheduling, supporting:

- Schedules in a particular timezone
- Calendars

Authorization
=============

ezsched relies on PAM authentication. By default, all objects are owned
by the user that submitted them.

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
      - Month, Day of week, and Number (e.g. 2nd Friday)
        - Number can be relative, so 1 is first, -1 is last
    - Optional `description`
    - Optional `observed` attribute that is `next`, `prev`, `closest`, `none` (default)
      that is in the day of week mask and not also a holiday
- A public flag, which if true will make the calendar publicly available
- A name, which can contain letters, numbers, dashes, and underscores
  - They cannot contain dots / periods

By default, calendars are referred to by their name, but are scoped by
the owning user in the format `user.calendar`.

Users can refer to their own calendars with just the name, other users
must access unowned calendars using the fulle `user.calendar` notation.

Endpoints
---------

```
/api/v1
 /calendars
  GET    -- Retrieve list of calendars and descriptions
 /calendar/:id
  GET    -- Get the current definition
  POST   -- Create a new calendar
  PATCH  -- Update an existing calendar
  DELETE -- Delete the named calendar
 /calendar/:id/:date
  GET    -- Returns true if the date is in the calendar
 /calendar/:id/:from/:to
  GET    -- Return the list of valid dates between :from and :to, inclusive.
            :from and :to are in the ISO-8601 date format (YYYY-MM-DD)
```

Calendar `:id`s are alphanumeric strings without spaces (e.g. `USHolidays`).

Encoding
--------

A JSON definition for a calendar looks as follows:

```json
{
  "description": "Long description",
  "dow_list": ["M","T","W","R","F"],
  "public": true,
  "exclude": [
    {
      "date": "2021-01-01",
      "description": "New Years Day"
      },
    {
      "month": "January",
      "day": 17,
      "observed": "closest",
      "description": "Martin Luther King Day"
      },
    {
      "month": "January",
      "dow": "M",
      "-1",
      "observed": "closest",
      "description": "Final Monday Margarita Day"
      },
    ]
  ]
}
```

Letter codes are:

| Day of Week | Letter |
|-------------|--------|
| Monday      | M      |
| Tuesday     | T      |
| Wednesday   | W      |
| Thursday    | **R**  |
| Friday      | F      |
| Saturday    | S      |
| Sunday      | U      |

Inheritance
-----------

Calendars can inherit from other calendars. The result will be a union
of the sets of days. For instance, a calendar of vacations could inherit
from a calendar of holidays.

```json
{
  "description": "Personal calendar days",
  "inherits": [
    "company/USHolidays",
    "company/CompanyHolidays"
  ],
  "exclude": [
    {
      "date": "2021-05-01",
      "description": "My gerbil's birthday"
    }
  ]
}
```

Job
===

Jobs are tasks that must be run on a schedule. Schedules are defined
using a calendar, a start time, and an optional frequency.

Jobs are identified by a name that can contain letters, numbers, dashes,
and underscores. Just like calendars, jobs can be reffered to using a
`user.job` notation.

By default, Jobs are run as the user that submitted them.

```json
{
  "description":  "Description of job",
  "calendar": "US Holidays",
  "timezone": "Atlantic/Reykjavik",
  "schedule": {
    "start": {
      "minute": "5",
      "hour": "*",
    },
    "frequency": "15m"
  },
  "command": {
  },
  "environment": {
  },
  "mailto": [
    "oncall@company.com"
  ],
  "public_acl": "rwx"
}
```

The `public_acl` is a string that defines how the non-owning users can
access the job: `r`ead it, `w`rite its defintiion, or e`x`ecute it.

Endpoints
---------

```
/api/v1
 /jobs
  GET    -- Retrieve list of jobs and descriptions
 /job/:id
  GET    -- Get the current definition
  POST   -- Create a new job
  PATCH  -- Update an existing job
  DELETE -- Delete the job
 /job/:id/run
  GET    -- Force a run of the job immediately
 /job/:id/history
  GET    -- Retrieve the run history and results
```
