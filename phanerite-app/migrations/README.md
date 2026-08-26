# Phanerite-app/migrations

This directory contains migration SQL files for the embedded Turso (SQLite) database in the app.

## The migration system

The migration system is custom-built and relatively simple. A custom build script ([build.rs](../build.rs)) scans this directory and embeds the migration scripts into the binary. At runtime, the app reads them and applies them in order. ([migration.rs](../src/db/migration.rs))

## Migration file format

Migration files are stored in this directory and MUST comply with the following rules:

- Must be UTF-8-encoded plain-text files.
- Must be written in the valid SQLite dialect. (Turso extensions are supported.)
- Each SQL statement must end with a semicolon (`;`).
- File names must be valid (see below).

### File name rules

Each migration file MUST have the following filename:

```
[id]-[slug].sql
```

Where:

- `[id]` is an integer. It must not exceed 65,535. The order of migrations is determined **SOLELY** by this number. You **MUST** ensure that newer migrations have larger IDs than all older migrations.
- `[slug]` is the human-readable name of the migration. It must be in `kebab-case`. (Underscores are allowed but not recommended.)

## Note

**NEVER modify past migrations that have been committed to the version tree.**

If you want to change existing table structures, simply create a new migration and write some SQL.
