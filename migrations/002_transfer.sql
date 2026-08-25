CREATE TABLE transfer (
    execution_id TEXT PRIMARY KEY,
    transfer_id UUID NOT NULL,
    evaluation_id UUID NOT NULL REFERENCES evaluation(id),

    client_id INTEGER NOT NULL,
    operation TEXT NOT NULL,

    file_size BIGINT NOT NULL,
    file_size_unit TEXT NOT NULL,
    num_files INTEGER NOT NULL,

    bytes_transferred BIGINT,
    duration_ms BIGINT,
    throughput_mbps DOUBLE PRECISION,

    crypto_mode TEXT,
    kx_group TEXT,
    cipher_suite TEXT,

    success BOOLEAN,
    error_type TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ
);