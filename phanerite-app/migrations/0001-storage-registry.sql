CREATE TABLE IF NOT EXISTS storage_registry (
    storage TEXT NOT NULL,
    hash    BLOB NOT NULL,
    path    TEXT NOT NULL UNIQUE,
    ref_count INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (storage, hash)
);
