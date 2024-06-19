CREATE TABLE users
(
    id BIGSERIAL PRIMARY KEY,
    external_id VARCHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    UNIQUE (external_id)
);