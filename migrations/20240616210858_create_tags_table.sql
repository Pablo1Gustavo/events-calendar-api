CREATE TABLE tags
(
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(6) NOT NULL CHECK (color ~* '^[0-9A-F]{6}$')
);