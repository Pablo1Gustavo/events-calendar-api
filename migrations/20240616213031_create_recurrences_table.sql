CREATE TYPE recurrence_type AS ENUM ('daily', 'weekly', 'monthly', 'yearly');

CREATE TABLE recurrences
(
    schedule_id BIGINT NOT NULL REFERENCES schedules(id) ON DELETE CASCADE,
    type recurrence_type NOT NULL,
    step SMALLINT NOT NULL DEFAULT 1,
    repetitions SMALLINT NOT NULL,
    end_date TIMESTAMP NOT NULL,
    PRIMARY KEY (schedule_id)
)