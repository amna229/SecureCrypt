CREATE TABLE evaluation (
    id UUID PRIMARY KEY,
    crypto_mode TEXT NOT NULL,
    num_clients INTEGER NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    status TEXT NOT NULL
);