CREATE TABLE schedules
(
    id BIGSERIAL PRIMARY KEY,
    recurrence_id BIGINT REFERENCES recurrences(id) ON DELETE CASCADE,
    event_id BIGINT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP NOT NULL
);