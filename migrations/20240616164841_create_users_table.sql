CREATE TABLE IF NOT EXISTS users
(
    id BIGSERIAL PRIMARY KEY,
    external_id VARCHAR(36),
    name VARCHAR(255) NOT NULL,
    UNIQUE (external_id)
);