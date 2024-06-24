CREATE TYPE week_day AS ENUM (
    'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday', 'sunday'
);

CREATE TABLE IF NOT EXISTS recurrences_week_days
(
    recurrence_id BIGINT NOT NULL REFERENCES recurrences(id) ON DELETE CASCADE,
    week_day week_day NOT NULL,
    PRIMARY KEY (recurrence_id, week_day)
);