-- 0001_baseline.sql — baseline migration for the chirp durable store (ADR-0001).
--
-- This is the first, forward-only migration. It establishes the migration *mechanism* and a small
-- provenance/metadata surface; it deliberately creates NO domain tables (users, chirps, follows,
-- likes, sessions, credentials). The domain DDL and all subsequent schema content are added by
-- downstream, forward-only migrations (EPIC-PERSIST SUBTASK_durable_impl) as new NNNN_*.sql files —
-- an applied migration is immutable and is never edited, and there are no down-migrations.
--
-- The applied schema version is tracked by sqlx in its `_sqlx_migrations` table; the greatest
-- applied `version` there is the inspectable schema version (see PostgresRepository::schema_version).
--
-- Every statement is idempotent (IF NOT EXISTS / ON CONFLICT DO NOTHING) so re-running the migrator
-- against an up-to-date store changes nothing.

CREATE TABLE IF NOT EXISTS chirp_store_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT INTO chirp_store_meta (key, value)
    VALUES ('baseline', 'ADR-0001')
    ON CONFLICT (key) DO NOTHING;
