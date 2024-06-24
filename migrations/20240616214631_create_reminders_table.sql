CREATE TYPE reminder_type AS ENUM (
    'email', 'sms', 'whatsapp', 'telegram', 'notification'
);

CREATE TABLE IF NOT EXISTS reminders
(
    id BIGSERIAL PRIMARY KEY,
    user_contact_id BIGINT NOT NULL REFERENCES users_contacts(id) ON DELETE CASCADE,
    event_id BIGINT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    type reminder_type NOT NULL,
    minutes_before INTEGER NOT NULL
);