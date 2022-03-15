CREATE TABLE IF NOT EXISTS scores
(
    id        BIGSERIAL    PRIMARY KEY,
    score     BIGINT       NOT NULL,
    username  VARCHAR(255) NOT NULL,
    scored_at TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
