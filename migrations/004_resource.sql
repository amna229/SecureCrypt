CREATE TABLE resource (
    resource_id UUID PRIMARY KEY,
    evaluation_id UUID NOT NULL REFERENCES evaluation(id),
    role TEXT NOT NULL,
    client_id INTEGER,
    timestamp TIMESTAMPTZ NOT NULL,
    cpu_avg_percent DOUBLE PRECISION NOT NULL,
    cpu_peak_percent DOUBLE PRECISION NOT NULL,
    memory_peak_bytes BIGINT NOT NULL
);