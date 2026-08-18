CREATE TABLE transfer (
    id UUID PRIMARY KEY,
    operation TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    file_size_unit TEXT NOT NULL,
    num_files INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);