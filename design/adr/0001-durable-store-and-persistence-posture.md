# ADR-0001: Durable store, replica/single-writer posture, and schema-versioning approach

## Status

Accepted

## Date

2026-10-13

## Context

`chirp` keeps **all** domain state in process memory: accounts, durable-credential and session
state, chirps, follows, likes, the denormalized counts, and the id/sequence counters. The
deployment runs a single replica whose only writable mount is an `emptyDir` scratch volume, so
every restart, redeploy, or reschedule wipes the product clean. This is the single largest gap
between the current product and the release bar (EPIC-PERSIST, issue #56; issues #64–#67).

The durable-persistence release conditions this decision is weighed against:

- **BAR-019 / REQ-OPS-001..004, REQ-OPS-006, REQ-OPS-007, REQ-DOM-012/013** — every kind of domain
  state is durable on **pod-independent** storage and survives process restart, rolling redeploy,
  and **reschedule to another node**; a confirmed write is durable *before* success is returned;
  coverage is total (sessions and credentials included).
- **BAR-020 / REQ-DOM-015/016, REQ-OPS-008** — multi-write operations that change an edge set and a
  denormalized count stay **atomic under concurrency and mid-operation failure**; the seam is safe
  for concurrent access on a single instance.
- **BAR-021 / REQ-DOM-014/018/022, REQ-OPS-005** — durable storage plugs into the **existing
  `ChirpRepository` seam** as a second implementor; durability holds under **more than one replica
  with a single consistent dataset, OR a single writer is enforced and documented**; the durable
  schema carries a **defined, inspectable version with a repeatable forward-migration path**.
- **BAR-022 / REQ-OPS-009/010/012/013** — user content is retained for the life of the account
  (no silent expiry); intentionally transient durable state has an enforced, documented lifetime;
  capacity **fails safe** on exhaustion (never a false-confirmed or corrupted write); durability is
  demonstrable by a repeatable drill.

The decision is also bounded by the existing architecture (the inherited seam, `../../contracts`):
`chirp-server` is a Rust/`axum`/`tokio` service whose persistence boundary is the **object-safe,
`async`, `Send + Sync`, `&self`** trait `ChirpRepository` (returning the typed `ServerError`), held
by the transport layer as `Arc<dyn ChirpRepository>`; `chirp-types` is the canonical domain model;
`InMemoryRepository` is the default backing and must remain so `cargo test` runs store-free. The
runtime image is distroless-nonroot with a **read-only root filesystem** (only `/tmp` is a writable
`emptyDir`), deployed on Kubernetes under ArgoCD GitOps.

This ADR **freezes** three coupled keystone decisions — the concrete store, the
replica-vs-single-writer posture, and the schema-versioning approach — that the rest of the
durable-persistence work (the additive seam extension, the durable-backend scaffold, the durable
implementation, and the operational storage binding under EPIC-PERSIST) reads as a **settled**
input rather than re-deciding. It records the *decision and approach*; the concrete schema DDL, the
durable method bodies, the migration content, and the live-deployment storage binding are
explicitly later work.

## Decision

### 1. Store — a PostgreSQL-compatible SQL database, reached over the network via `sqlx`

The durable store is a **PostgreSQL database**, accessed from `chirp-server` with the async Rust
driver **`sqlx`** (PostgreSQL backend, runtime-`tokio`, `rustls`). It is reached over the network
through a single connection pool configured from a `DATABASE_URL` connection string — so the store
is an independently-addressable service whose lifecycle is decoupled from any `chirp-server` pod.

The store is *pod-independent by construction*: because `chirp-server` reaches it over the network,
a `chirp-server` restart, redeploy, or reschedule to another node never touches the data. In this
engagement's cluster the database is provisioned in-cluster as a single-primary `StatefulSet` bound
to its own `PersistentVolumeClaim`; other environments may point `DATABASE_URL` at an external
managed PostgreSQL. **This ADR fixes "a PostgreSQL database reached via `sqlx` over `DATABASE_URL`";
the concrete provisioning of that database is owned by the later operational-readiness work** (and,
for backup/recovery, EPIC-DATAOPS) and is out of scope here.

Why PostgreSQL + `sqlx`:

- **Node-independent durability (REQ-OPS-004).** A networked database survives a `chirp-server`
  reschedule to another node trivially — the data does not move with the pod. An embedded
  file-database on a node-local volume cannot make this guarantee in a multi-node cluster.
- **Native transactional atomicity (REQ-DOM-015, BAR-020).** Edge-plus-count mutations
  (follow/unfollow, like/unlike, create-chirp, create-user + handle index, account-deletion
  cascade) map directly onto SQL transactions: a reader never sees a count disagreeing with its
  edges, and a mid-operation failure leaves a clean pre- or post-state. This is the boring,
  well-understood mechanism for the correctness the bar demands, rather than one hand-built.
- **Concurrency-safe seam on a single handle (REQ-DOM-016).** A shared `sqlx::PgPool` is
  `Send + Sync` and used through `&self`, so it drops into the existing
  `Arc<dyn ChirpRepository>` shape with no change to the trait's object-safety or signatures.
- **Multi-replica posture is *available* (BAR-021 / REQ-OPS-005).** Several `chirp-server` replicas
  can share one PostgreSQL dataset as a single consistent source of truth — satisfying the
  multi-replica arm of the bar — without forking state across pods the way the current in-memory
  `replicas: 1` silently would.
- **First-class schema versioning + migrations (REQ-DOM-022).** `sqlx` ships an embedded migration
  runner and a version-tracking table (see §3), so the versioned-schema requirement is met with the
  store's own standard tooling rather than a bespoke mechanism.
- **Stack fit.** `sqlx` is async (matching the `async` seam), compile-time-checks queries against
  the schema, and is the ecosystem-standard async SQL layer — preserving `chirp-types` as canonical
  and leaving the transport layer untouched. The existing crate docs already named a SQL backing as
  the intended durable alternative; this ADR makes it concrete and justifies it against the bar.
- **Fail-safe capacity (REQ-OPS-012).** A write that cannot be durably committed (including on disk
  exhaustion) surfaces as a database error the durable implementor maps to `ServerError::Internal`
  and reports as a **failed** action — never a false confirmation, and committed rows are untouched.

### 2. Posture — multi-replica-capable application tier over a single-primary database

The posture is a **single consistent dataset** held by **one PostgreSQL primary**, with the
**application tier (`chirp-server`) replica-capable** against it:

- At the seam/store level the **multi-replica arm of BAR-021 / REQ-OPS-005 is satisfied**: running
  more than one `chirp-server` replica is *safe* because every replica reads and writes the one
  shared PostgreSQL dataset — there is no per-pod state to fork. This replaces the current
  undocumented, state-splitting `replicas: 1`.
- The **PostgreSQL primary is a single instance** in this release: there is **no automatic failover
  / HA and no read-replica fan-out**. This is a deliberately **documented limitation** — it bounds
  *availability* of the database (a primary outage is downtime), not *consistency* (there is never
  more than one writable copy, so data can never silently fork). Durability of committed data rests
  on the primary's persistent volume plus the backups/recovery work owned downstream (EPIC-DATAOPS).
- Because a single writable primary is what makes this safe, the posture is honestly a
  **single-writer database** posture at the *database* tier, combined with a **horizontally
  scalable, stateless application tier** at the *seam* tier. The demo deployment defaults
  `chirp-server` to `replicas: 1` for simplicity, but scaling it out is correctness-safe; the only
  limitation that must be recorded is the absence of database-primary HA.

This posture is **frozen**: the later persistence work treats "single consistent PostgreSQL
dataset; app tier scalable; database primary single-instance (no HA yet), documented" as given.

### 3. Schema-versioning approach — `sqlx` embedded, forward-only, version-tracked migrations

Schema evolution uses **`sqlx`'s embedded migration runner** over an **append-only, forward-only**
set of ordered SQL migration files:

- **Migration source.** A repository directory (`crates/chirp-server/migrations/`, exact path for
  the scaffolding work to settle) of `NNNN_description.sql` files, applied in ascending order. The files
  are **immutable once committed and forward-only** — a schema change is a *new* migration, never an
  edit to an applied one; there are no down-migrations (REQ-DOM-022 requires a forward path).
- **Embedded + run at startup.** The migrations are embedded into the binary via
  `sqlx::migrate!(...)` and applied by the durable backend at startup
  (`migrator.run(&pool).await`) *before* the service begins serving. A fresh database initializes
  from empty by applying every migration in order; an up-to-date database applies none.
- **Inspectable version + idempotent re-run.** `sqlx` records applied migrations (version +
  checksum + timestamp) in its `_sqlx_migrations` tracking table. The **current schema version** is
  the greatest applied version there; it is readable for inspection and is the natural datum behind
  a `store_health` capability. Re-running the migrator when all migrations are already applied is a
  **no-op** (applying the migration path to an up-to-date store changes nothing — the idempotency
  the versioned-schema requirement calls for).
- **Scope of this decision.** This ADR freezes the *mechanism and conventions* (embedded `sqlx`
  migrations, forward-only, version-tracked, startup-applied). The **initial schema DDL and the
  per-state migration content are later work**: the durable-backend scaffolding stands up the runner
  and an inspectable version surface (migration content may be skeletal); the durable implementation
  writes the real DDL/migration content against it.

### 4. Backend selection — config-driven, in-memory default for test/local

Backend selection is **config-driven at the composition root** (`main.rs` / `AppState`
construction), *not* a code edit:

- When a PostgreSQL connection string is configured (e.g. `CHIRP_DATABASE_URL` / `DATABASE_URL`
  set — exact variable name for the scaffolding work to finalize and document), the **production path**
  constructs the PostgreSQL-backed `ChirpRepository`, runs migrations, and injects it into
  `AppState`.
- When it is **absent/empty** (the **test/local default**), the server constructs
  `InMemoryRepository`, so `cargo build` and `cargo test` need no database and stay store-free.
- Only the backend *construction* at the composition root varies; the transport layer, router,
  handlers, and `AppState`'s `Arc<dyn ChirpRepository>` shape are **unchanged**.

## Consequences

### Positive

- Satisfies the hard durability bar (BAR-019): a networked store is pod-independent and survives
  restart, redeploy, and reschedule-to-another-node by construction; confirmed-write-before-confirm
  is a transaction commit before the `Ok(..)` return.
- SQL transactions give the atomic, concurrency-correct multi-write behaviour (BAR-020 /
  REQ-DOM-015/016) directly, without a hand-rolled consistency mechanism.
- The durable store is a *second implementor* of the unchanged `ChirpRepository` trait — the seam
  stays additive, `chirp-types` stays canonical, `InMemoryRepository` is retained, and the
  transport layer is untouched (BAR-021 / REQ-DOM-014).
- The application tier can scale horizontally against one consistent dataset (REQ-OPS-005), removing
  the current state-splitting single-replica hazard.
- Standard, inspectable, forward-only schema versioning via the store's own tooling (REQ-DOM-022).
- Retention and fail-safe capacity map cleanly onto SQL: content rows never auto-expire; transient
  rows carry an `expires_at` swept on a schedule; a store that cannot commit returns an error that
  is reported as a failed write (BAR-022 / REQ-OPS-009/010/012).

### Negative

- Adds an operational dependency: a PostgreSQL instance must be provisioned, connected (a
  `DATABASE_URL` secret), and operated — new infrastructure and a new failure mode versus the
  zero-dependency in-memory store. (The provisioning/binding is downstream ops work.)
- The single PostgreSQL primary is a **single point of availability** in this release: a primary
  outage is downtime. HA/failover and read replicas are explicitly deferred; durability rests on the
  primary's persistent volume plus the backups epic.
- `sqlx` compile-time query verification against a live/offline schema adds a build-time
  consideration (an offline query cache or a reachable dev database); the scaffolding work decides
  between `query!`-macro checking and runtime `query` to keep `cargo build`/CI self-contained.
- Introduces eventual operational chores (connection-pool sizing, migration review discipline,
  capacity monitoring) that the in-memory store never required.

### Neutral

- The exact PostgreSQL hosting (in-cluster single-primary `StatefulSet` + PVC for the demo vs an
  external managed database) is intentionally left to the later operational-readiness work; this ADR fixes only
  "PostgreSQL via `sqlx` over `DATABASE_URL`".
- `sqlx` was chosen over other async SQL layers (see alternatives); a team already fluent in an
  async ORM could revisit the driver without disturbing this ADR's store/posture/versioning
  decisions, since those are expressed behind the `ChirpRepository` trait.

## Alternatives Considered

### Alternative 1: Embedded SQLite on a persistent volume

- **Pros:** Simplest operations — one PVC, no separate database pod, no connection secret; lowest
  resource footprint; `sqlx` supports it; on a single-node cluster the reschedule case is moot.
- **Cons:** On a **node-local** volume (the cluster's default `local-path` storage class) the data
  is pinned to a node, so a reschedule to **another** node loses it — directly failing REQ-OPS-004
  in any multi-node posture. SQLite is a **single-writer file database**: the multi-replica arm of
  BAR-021 / REQ-OPS-005 is impossible (the app tier can never scale out), and concurrent writers
  must be serialized in-process rather than by the store. A read-only root filesystem further means
  the database file must live on a dedicated writable mount, making the write path a careful special
  case.
- **Why rejected:** It forces the weaker single-writer arm *and* carries a real reschedule-fragility
  the release bar explicitly tests. Freezing it would bake a limitation into the foundation that
  later work could not undo without re-opening this keystone decision. PostgreSQL meets the
  same simplicity need at modest additional operational cost while satisfying REQ-OPS-004 and the
  multi-replica arm.

### Alternative 2: A non-relational / key-value store (e.g. Redis-persistence, a document store)

- **Pros:** Simple data access for the key-addressed entities; some options offer easy horizontal
  scaling.
- **Cons:** The hard requirement is **atomic multi-write consistency across an edge set and a
  denormalized count under concurrency** (REQ-DOM-015). Relational transactions express this
  directly; a KV/document store would push multi-key atomicity, secondary indexing (the unique
  handle index), and cursor-stable ordering into hand-built application logic — exactly the
  "draw the rest of the owl" correctness risk the bar is written to prevent. Monotonic, restart-
  stable id/sequence allocation is also a native SQL `SEQUENCE`/identity concern.
- **Why rejected:** The denormalized-count-vs-edge-set invariant is the core correctness bar, and a
  transactional SQL store is the lowest-risk way to hold it. The product's shape is relational.

### Alternative 3: A distributed / HA SQL cluster now (e.g. multi-node PostgreSQL HA, CockroachDB)

- **Pros:** Removes the single-primary availability limitation; distributed-SQL options are
  PostgreSQL-wire-compatible.
- **Cons:** Significant operational complexity (failover orchestration, quorum, backup/restore of a
  cluster) paid before the product has the scale or the operational maturity to justify it. The
  release bar requires a *single consistent dataset* and *documented* posture — not HA.
- **Why rejected:** Solves an availability problem the release does not yet have, at a cost the
  engagement cannot yet operate. Recorded as the natural **trigger for a future ADR**: revisit
  database HA when an availability SLO or real load makes single-primary downtime unacceptable.
  Choosing standard PostgreSQL keeps that door open (the HA options are wire-compatible).

### Alternative 4: Three separate ADRs (store, posture, schema-versioning)

- **Pros:** Finer-grained decision records.
- **Cons:** The three facets are not independent — the posture options and the viable
  schema-versioning tooling are *consequences* of the store choice. Splitting them would fragment
  one interlocked decision and force cross-references for context that belongs together.
- **Why rejected:** This is one keystone decision; recording it as one ADR keeps the rationale and
  the coupling visible in a single place, which is exactly what the later work reads.

## References

- `design/release-bar.md` — BAR-019, BAR-020, BAR-021, BAR-022 (durable persistence & data
  integrity).
- `design/domain-application-design.md` — REQ-DOM-005/012/013/014/015/016/018/022 (persistence
  requirement, domain model, the `ChirpRepository` seam and its capability set).
- `design/operational-readiness.md` — REQ-OPS-001..013 (pod-independent storage, restart/redeploy/
  reschedule survival, confirmed-write durability, retention, lifetime, capacity fail-safe, drill).
- EPIC-PERSIST issue #56 and child issues #64 (BAR-019), #65 (BAR-020), #66 (BAR-021), #67
  (BAR-022).
- `crates/chirp-server/src/repository/mod.rs` — the `ChirpRepository` seam this store plugs into as
  a second implementor; `crates/chirp-types` — the canonical, preserved domain model.
