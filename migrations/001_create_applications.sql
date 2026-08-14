CREATE TABLE applications (
    id UUID PRIMARY KEY,
    operation TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    file_size_unit TEXT NOT NULL,
    num_files INTEGER NOT NULL
);