CREATE TYPE contact_type AS ENUM ('email', 'phone');

CREATE TABLE users_contacts
(
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    contact VARCHAR(255) NOT NULL,
    type contact_type NOT NULL,
    UNIQUE (user_id, contact)
);
