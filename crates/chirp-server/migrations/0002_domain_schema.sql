-- 0002_domain_schema.sql — the full durable domain schema for the chirp store (EPIC-PERSIST, #64/#65/#66).
--
-- Second forward-only migration (it follows 0001_baseline.sql). It establishes the COMPLETE durable
-- domain schema the capability_set persists against: the identity/account tables, the content
-- tables, the social-graph edge tables, the denormalized count columns, the id/sequence counter
-- infrastructure, moderation/suspension state, and the intentionally-transient (session / recovery /
-- export) tables with their enforced-lifetime columns. It represents the chirp-types domain model
-- (crates/chirp-types) faithfully in PostgreSQL.
--
-- Conventions — FROZEN by ADR-0001 §3 and established by 0001_baseline.sql; this migration applies
-- them, it does not re-decide them:
--   * Forward-only & immutable. Once applied this file is never edited; a later schema change is a
--     NEW NNNNN_*.sql file. There are no down-migrations (REQ-DOM-022 requires only a forward path).
--   * Applying it ADVANCES the inspectable schema version to 2 — sqlx records it in
--     `_sqlx_migrations`, and PostgresRepository::schema_version() reports MAX(version) = 2.
--   * Every statement is idempotent (IF NOT EXISTS / ON CONFLICT DO NOTHING) so re-running the
--     migrator against an up-to-date store changes nothing, matching 0001_baseline.sql's house style.
--   * Ids are BIGINT, server-assigned from the SEQUENCE counters below. Those sequences are the id
--     counter INFRASTRUCTURE this migration owns; the allocation MECHANISM (when/how nextval is
--     called and mapped onto the opaque chirp_types ids) is the store-access-substrate sibling's, so
--     the id columns deliberately carry NO `DEFAULT nextval(...)`.
--   * Timestamps are BIGINT epoch-MILLISECONDS (UTC), matching chirp_types::ids::Timestamp (an i64).
--   * The denormalized counts (follower_count / following_count / like_count) are columns kept in
--     lockstep with their edge sets by the store's transaction mechanism (a later sibling); this
--     migration declares the columns with a zeroed default, never the maintenance logic (REQ-DOM-003,
--     REQ-DOM-015).
--   * Domain-boundary VALIDATION (REQ-DOM-002) is enforced in Rust by the mapping/validation sibling.
--     The length / invariant CHECKs below mirror the chirp_types invariants in-store as defence in
--     depth so entity invariants hold in ANY backing store (REQ-DOM-003) — they do not replace the
--     Rust validation pattern.

-- ===========================================================================
-- Id / sequence counters (REQ-DOM-004).
-- A PostgreSQL SEQUENCE provides exactly the id guarantees the bar requires: opaque, server-assigned,
-- MONOTONIC, STABLE across restart (its state is durable), and NEVER reused. These mirror the
-- in-memory next_user_id / next_chirp_id / next_token counters. The store-access substrate allocates
-- an id by calling nextval() on the matching sequence; this migration owns only the counters.
-- ===========================================================================
CREATE SEQUENCE IF NOT EXISTS user_id_seq  AS BIGINT MINVALUE 1 START WITH 1;
CREATE SEQUENCE IF NOT EXISTS chirp_id_seq AS BIGINT MINVALUE 1 START WITH 1;
CREATE SEQUENCE IF NOT EXISTS token_seq    AS BIGINT MINVALUE 1 START WITH 1;

-- ===========================================================================
-- Identity / account.
-- ===========================================================================

-- users — accounts and public profiles (chirp_types::domain::User).
--   * id and username are IMMUTABLE identity; display_name and bio are mutable (REQ-DOM-008).
--   * follower_count / following_count are denormalized projections of the follows edge set.
--   * suspended / suspended_at are moderation state (account_suspension, REQ-DOM-011); a suspended
--     account cannot act, honoured on every read by the capability-method sibling.
CREATE TABLE IF NOT EXISTS users (
    id              BIGINT  PRIMARY KEY,
    username        TEXT    NOT NULL,
    display_name    TEXT    NOT NULL,
    bio             TEXT,
    created_at      BIGINT  NOT NULL,
    follower_count  BIGINT  NOT NULL DEFAULT 0 CHECK (follower_count  >= 0),
    following_count BIGINT  NOT NULL DEFAULT 0 CHECK (following_count >= 0),
    suspended       BOOLEAN NOT NULL DEFAULT FALSE,
    suspended_at    BIGINT,
    CONSTRAINT users_username_len CHECK (char_length(username) BETWEEN 1 AND 15)
);

-- The unique-handle index: handles are globally unique, so create_user returns Conflict on a
-- duplicate handle and login/verify_credential resolve a handle to exactly one account.
CREATE UNIQUE INDEX IF NOT EXISTS users_username_key ON users (username);

-- credentials — the durable authentication credential, distinct from the public handle
-- (REQ-DOM-005; credential_set_verify). One per user. `secret` is the AT-REST credential
-- representation; its exact format and the verification protocol are owned by identity & trust
-- (REQ-ID-*) — this is only the durable slot, and the secret is NEVER served on the wire.
CREATE TABLE IF NOT EXISTS credentials (
    user_id    BIGINT PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    secret     TEXT   NOT NULL,
    updated_at BIGINT NOT NULL
);

-- sessions — durable sessions (REQ-DOM-006; login / resolve_token / invalidate_session). The opaque
-- SessionToken string is the primary key; resolve_token maps it to the authenticated user.
-- `expires_at` is the enforced-lifetime column for this intentionally-transient state (REQ-OPS-010):
-- the lifetime VALUE and the sweep are owned downstream; the column (and its index) exist so a
-- bounded lifetime can be enforced and expired rows swept. NULL means no expiry has been assigned.
CREATE TABLE IF NOT EXISTS sessions (
    token      TEXT   PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at BIGINT NOT NULL,
    expires_at BIGINT
);
CREATE INDEX IF NOT EXISTS sessions_user_id_idx    ON sessions (user_id);
CREATE INDEX IF NOT EXISTS sessions_expires_at_idx ON sessions (expires_at);

-- recovery_tokens — account-recovery challenges (REQ-DOM-005; begin/complete_account_recovery).
-- Intentionally-transient durable state with a REQUIRED, enforced lifetime (REQ-OPS-010): a recovery
-- token cannot be usable past its documented validity, so expires_at is NOT NULL.
CREATE TABLE IF NOT EXISTS recovery_tokens (
    token      TEXT   PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS recovery_tokens_user_id_idx    ON recovery_tokens (user_id);
CREATE INDEX IF NOT EXISTS recovery_tokens_expires_at_idx ON recovery_tokens (expires_at);

-- ===========================================================================
-- Content.
-- ===========================================================================

-- chirps — posts (chirp_types::domain::Chirp).
--   * author_id references the author; account deletion cascades the author's chirps (REQ-DOM-009).
--   * reply_to is a same-store self-reference for threaded replies and MUST reference a chirp that
--     exists (REQ-DOM-007 referential integrity); ON DELETE SET NULL keeps a reply valid when its
--     target is removed (no dangling reference).
--   * like_count is the denormalized projection of the likes edge set.
--   * removed / removed_at are moderation soft-removal (chirp_removal, REQ-DOM-011); removed content
--     is not served, honoured by the capability-method sibling. Distinct from account-deletion, which
--     hard-removes rows via the cascade above.
CREATE TABLE IF NOT EXISTS chirps (
    id         BIGINT  PRIMARY KEY,
    author_id  BIGINT  NOT NULL REFERENCES users (id)  ON DELETE CASCADE,
    text       TEXT    NOT NULL,
    created_at BIGINT  NOT NULL,
    like_count BIGINT  NOT NULL DEFAULT 0 CHECK (like_count >= 0),
    reply_to   BIGINT  REFERENCES chirps (id) ON DELETE SET NULL,
    removed    BOOLEAN NOT NULL DEFAULT FALSE,
    removed_at BIGINT,
    CONSTRAINT chirps_text_len CHECK (char_length(text) BETWEEN 1 AND 280)
);
-- Author lookup, and the reverse-chronological home-timeline scan: ordering by id DESC is the stable
-- reverse-chronological proxy (a larger id is always a later chirp), which opaque-cursor paging walks.
CREATE INDEX IF NOT EXISTS chirps_author_id_idx ON chirps (author_id);
CREATE INDEX IF NOT EXISTS chirps_timeline_idx  ON chirps (author_id, id DESC);
CREATE INDEX IF NOT EXISTS chirps_reply_to_idx  ON chirps (reply_to);

-- export_artifacts — materialised data-export documents (REQ-DOM-010; data_export). `content` is the
-- serialized export document the export capability returns. Intentionally-transient durable state
-- with a REQUIRED retention window (REQ-OPS-010): a materialised export is removed within its
-- documented window and not left readable indefinitely, so expires_at is NOT NULL.
CREATE TABLE IF NOT EXISTS export_artifacts (
    id         BIGINT PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    content    TEXT   NOT NULL,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL
);
CREATE INDEX IF NOT EXISTS export_artifacts_user_id_idx    ON export_artifacts (user_id);
CREATE INDEX IF NOT EXISTS export_artifacts_expires_at_idx ON export_artifacts (expires_at);

-- ===========================================================================
-- Social graph (the edge sets; their denormalized counts live on users/chirps above).
-- ===========================================================================

-- follows — directed follow edges (follower → followee). The primary key enforces at-most-one edge
-- per pair (follow is idempotent); the CHECK enforces the no-self-follow invariant (REQ-DOM-003).
-- Deleting either account cascades the edge (account_deletion_cascade, REQ-DOM-009).
CREATE TABLE IF NOT EXISTS follows (
    follower_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    followee_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    created_at  BIGINT NOT NULL,
    PRIMARY KEY (follower_id, followee_id),
    CONSTRAINT follows_no_self CHECK (follower_id <> followee_id)
);
-- list_followers (accounts following a user) and list_following (accounts a user follows), each
-- newest-first with opaque-cursor paging.
CREATE INDEX IF NOT EXISTS follows_followee_idx ON follows (followee_id, created_at DESC);
CREATE INDEX IF NOT EXISTS follows_follower_idx ON follows (follower_id, created_at DESC);

-- likes — like edges (user → chirp). The primary key enforces at-most-one like per pair (like/unlike
-- are idempotent). Deleting the user or the chirp cascades the like.
CREATE TABLE IF NOT EXISTS likes (
    user_id    BIGINT NOT NULL REFERENCES users (id)   ON DELETE CASCADE,
    chirp_id   BIGINT NOT NULL REFERENCES chirps (id)  ON DELETE CASCADE,
    created_at BIGINT NOT NULL,
    PRIMARY KEY (user_id, chirp_id)
);
-- Reverse lookup: who liked a chirp / recomputing a chirp's like_count from its edges.
CREATE INDEX IF NOT EXISTS likes_chirp_idx ON likes (chirp_id);

-- Record the domain-schema milestone in the provenance surface established by 0001_baseline.sql.
INSERT INTO chirp_store_meta (key, value)
    VALUES ('domain_schema', '0002')
    ON CONFLICT (key) DO NOTHING;
