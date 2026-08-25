CREATE TABLE handshake (
    handshake_id UUID PRIMARY KEY,
    evaluation_id UUID NOT NULL REFERENCES evaluation(id),
    client_id INTEGER NOT NULL,
    handshake_duration_ms BIGINT,
    kx_group TEXT,
    cipher_suite TEXT,
    success BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);