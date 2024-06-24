CREATE TYPE recurrence_type AS ENUM (
    'daily', 'weekly', 'monthly', 'yearly'
);

CREATE TABLE IF NOT EXISTS recurrences
(
    id BIGSERIAL PRIMARY KEY,
    type recurrence_type NOT NULL,
    step SMALLINT NOT NULL DEFAULT 1,
    repetitions SMALLINT NOT NULL,
    end_date TIMESTAMP NOT NULL
);