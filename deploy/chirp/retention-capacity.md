# Retention and capacity

This document states what `chirp` keeps, for how long, and what happens when the durable store runs
out of room. It covers three operational commitments for the PostgreSQL store deployed under
`deploy/chirp`:

- user content is never silently expired (REQ-OPS-009);
- intentionally transient state has a defined, enforced lifetime and its expired rows are swept on a
  schedule (REQ-OPS-010);
- storage is provisioned with headroom, and capacity exhaustion fails writes honestly instead of
  losing or corrupting committed data (REQ-OPS-012).

The server already models transient-state expiry and already fails capacity exhaustion safe (see the
code references below). The operational pieces this document adds are the schedule that runs the
sweep and the written retention and capacity commitments.

## User content is retained for the life of the account

Accounts, profiles, chirps, follows, and likes have **no automatic expiry or eviction**. These tables
carry no `expires_at` column and are reached by no lifetime, expiry, or sweep path. User content
leaves the store only when a user or an authorized operator deletes it; account deletion cascades to
the account's content and transient state in one durable transaction. There is no background job,
TTL, or size cap that removes user content, and none is configured.

## Transient state has a defined, enforced lifetime

Three kinds of durable state are intentionally transient. Each carries its lifetime in an `expires_at`
column (`BIGINT` epoch-milliseconds, UTC), added in migration `0002_domain_schema.sql`.

| State | Window | Enforcement |
|-------|--------|-------------|
| Session | 30 days | `SESSION_LIFETIME_MILLIS` stamped on each issued session; an expired session resolves as absent at serve time and its row is swept. |
| Account-recovery token | 24 hours | `RECOVERY_TOKEN_VALIDITY_MILLIS`; `expires_at` is `NOT NULL`; an expired token is rejected by recovery completion and its row is swept. |
| Materialised data-export artefact | swept once past its stamped `expires_at` | The `export_artifacts` table and the sweep enforce the window. This release materialises no export artefacts (the download-your-data feature is not yet built), so no such rows are produced yet; the lifetime is enforced for any that a future release stamps. |

The session lifetime value and its security properties are owned by identity and trust (`REQ-ID-*`).
This store honours and documents the window; it does not set the credential or session crypto.

Expiry is honoured at read time independently of the sweep: an expired session resolves as absent and
an expired recovery token is rejected whether or not the sweep has run yet. The sweep's job is to
reclaim the storage those expired rows occupy so transient tables do not accumulate indefinitely.

### The sweep runs on a schedule

`ChirpRepository::sweep_expired_transient_state()` deletes expired `sessions`, `recovery_tokens`, and
`export_artifacts` rows (`crates/chirp-server/src/repository/postgres.rs`). The deployed system invokes
it on a schedule through a Kubernetes CronJob, `deploy/chirp/50-transient-sweep-cronjob.yaml`:

- **Schedule:** hourly (`0 * * * *`, UTC). The shortest window is the 24-hour recovery token, so an
  hourly cadence keeps expired rows from lingering more than about an hour past expiry. The three
  deletes are index-supported (`sessions_expires_at_idx`, `recovery_tokens_expires_at_idx`), so each
  run is seconds of work.
- **What it runs:** the same three `DELETE ... WHERE expires_at <= now` statements the repository
  sweep runs, in one transaction, against the store over the `chirp-postgres` Service. It reads its
  database credentials from the same out-of-band `chirp-db-credentials` Secret the server and the
  StatefulSet use; nothing secret is committed.
- **Why a CronJob and not an in-process timer:** the sweep code is held unchanged, no server image
  rebuild is required, and delivery stays GitOps (the `batch/CronJob` kind is whitelisted in the
  Argo CD AppProject). The CronJob triggers the sweep's reclamation on a cadence; which rows expire is
  decided entirely by the schema and the lifetime values above.
- **Concurrency and cleanup:** `concurrencyPolicy: Forbid` (a run never overlaps itself),
  `activeDeadlineSeconds` bounds a hung run, and finished Jobs are reaped (`ttlSecondsAfterFinished`,
  history limits), so the sweep never piles up completed Jobs.

## Capacity

### Provisioned size and growth

The store's data volume is a per-replica `PersistentVolumeClaim` (`data-chirp-postgres-0`, access mode
`ReadWriteOnce`) minted by the StatefulSet's `volumeClaimTemplates`, with a storage request of
**2Gi**. Its retention policy is `Retain` on both scale-down and deletion, so the data volume is not
reclaimed out from under the store.

Growth assumption: `chirp`'s domain rows are small — integer ids and counts, handles of at most 15
characters, chirps of at most 280 characters, plus follow and like edges and their denormalised
counts. A demo-scale corpus on the order of tens of thousands of accounts and up to a few hundred
thousand chirps, with indexes and write-ahead log, sits in the low hundreds of megabytes. The
transient tables are bounded by the sweep above rather than growing without limit. 2Gi therefore
holds the expected working set several times over.

Expansion caveat: the target uses the default `local-path` StorageClass, which does not support online
volume expansion. Growing the store means provisioning a larger volume and migrating the data, or
moving to a StorageClass that allows expansion. This is a known limitation of the demo target, not an
assumption that 2Gi is permanent; capacity monitoring and alerting that would signal approaching the
limit are owned by the observability area and are out of scope here.

Single-writer posture: there is one PostgreSQL primary (StatefulSet `replicas: 1`, no HA this release
per ADR-0001), holding one consistent dataset. The application tier is safe to scale against that one
store because all domain state lives behind it, not in any pod; it defaults to one replica. This is a
store property, documented as such, not an undocumented single replica.

### Exhaustion fails safe

A write that cannot be durably committed is reported as a **failed action**, never as a false
confirmation, and already-committed data is left intact. The path is in the server code:

- `PostgresRepository::with_transaction` returns a success value **only after** the transaction
  commits. If the commit fails, the write is not confirmed; if the work fails earlier, the transaction
  rolls back and no partial write becomes durable.
- A write failure caused by the store being out of capacity — disk full, out of memory, too many
  connections (PostgreSQL SQLSTATE class `53`, *insufficient_resources*), or a connection-pool acquire
  timeout — is classified by `classify_write_error` / `is_capacity_exhaustion` as
  `ServerError::Unavailable`; any other failure is `ServerError::Internal`. Both map to the `INTERNAL`
  error code and HTTP 500, so the caller sees the action fail rather than appear to succeed.

This is the behaviour the `confirmed_write_durable` and `capacity_fail_safe_never_false_confirms`
tests assert. The effect for a user is that capacity pressure can delay or fail new writes but can
never silently drop or corrupt data the product already confirmed.

## Scope

Out of scope here and owned elsewhere: capacity monitoring and alerting (observability); per-user
rate limits and quotas (limits and quotas); backups, restore, and migration-rollback safety; and the
authoritative session-lifetime value and credential crypto (identity and trust). This document covers
the live store's retention and capacity only.
