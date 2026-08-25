CREATE TABLE IF NOT EXISTS storage_registry (
    storage TEXT NOT NULL,
    hash    BLOB NOT NULL,
    path    TEXT NOT NULL UNIQUE,
    PRIMARY KEY (storage, hash)
);
