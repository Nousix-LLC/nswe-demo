# chirp — Domain & Application Design

This document states **what the chirp application and its domain must be** — the domain model, how the
product's parts divide, what durable persistence the release bar requires, and the interfaces and entry
points between the parts — expressed as **requirements and structure**, concrete enough to build against
and held to the crates that already exist in `Nousix-LLC/nswe-demo`. It does **not** settle *how* any of
it is built: the database engine, the migration tooling, the authentication protocol, and the backup
mechanism are named as design requirements here and their implementation is deferred to later code work
and to the sibling design areas.

**What this document owns.** It is the sole owner of the `REQ-DOM-*` register. Where a requirement's
detailed mechanism belongs to a sibling area, the requirement is stated from the user's or operator's
side and the area is named: authentication-protocol internals, session lifetime, deletion authorization,
and export authorization sit with identity & trust (`REQ-ID-*`); the concrete store, backups, restore,
health probes, and scaling sit with operational readiness (`REQ-OPS-*`), which **consumes the persistence
model defined here**; screens and interaction sit with experience design (`REQ-UX-*`). This document
fixes the *what* at each of those seams.

**Grounding.** Observations of "the product as it stands" are grounded in the repository at commit
`c805525`. They are stated so the gaps and the structure to build against are real, not hypothetical. The
crates referenced are `ferric` / `ferric-http` / `ferric-router` (the frontend framework substrate),
`chirp-types` (the shared domain + wire contract), `chirp-server` (the axum REST backend, currently
backed by an in-memory repository), and `chirp-frontend` (the WebAssembly single-page app).

**Traceability.** Every `REQ-DOM-*` carries a testable acceptance criterion, a release-gating flag, and —
where it realises a product-level need — a trace to one or more `REQ-PROD-*` from
`product-requirements.md`. Identifier conventions follow `contracts/id_and_traceability_schema.yaml`; the
`chirp-types` schema is used as-is and is **not** redefined here.

The required sections of this artifact follow in order: `domain_model`, `modules_and_responsibilities`,
`persistence_requirement`, `interfaces_and_entry_points`, and `requirements_and_acceptance_criteria`.

---

## 1. Domain model

*(section: `domain_model`)*

### 1.1 The model as it exists, and the model the release needs

chirp's domain is already defined, once, in `chirp-types` (`crates/chirp-types/src/domain.rs`,
`ids.rs`, `api.rs`) and is shared verbatim by the server and the frontend, so the two ends of every
request cannot drift. That crate is the **single source of truth for the types that cross the wire**, and
this design treats it as the canonical domain model: the release is built by *extending* it additively,
never by re-declaring or forking it.

The entities and value objects that exist today:

| Element | Kind | Shape (authoritative at `c805525`) | Invariant(s) recorded by the type |
|---|---|---|---|
| `User` | entity | `id: UserId`, `username: Username`, `display_name: String`, `bio: Option<String>`, `created_at: Timestamp`, `follower_count: u64`, `following_count: u64` | counts are server-maintained, read-only to clients |
| `Chirp` | entity | `id: ChirpId`, `author_id: UserId`, `text: ChirpText`, `created_at: Timestamp`, `like_count: u64`, `reply_to: Option<ChirpId>` | `like_count` is server-maintained; `reply_to` is `Some` only for a reply |
| `Follow` | edge | `follower_id`, `followee_id`, `created_at` | `follower_id != followee_id` (no self-follow) |
| `Like` | edge | `user_id`, `chirp_id`, `created_at` | at most one `Like` per `(user_id, chirp_id)` |
| `Timeline` | aggregate | `chirps: Vec<Chirp>` | reverse-chronological (newest first) |
| `Username` | value object | newtype over `String` | 1..=15 chars; ASCII letter, digit, or `_` only; validated on construction **and** deserialization |
| `ChirpText` | value object | newtype over `String` | 1..=280 characters (counted by Unicode scalar value, not bytes); validated likewise |
| `UserId` / `ChirpId` | identifier | opaque newtype over `u64`, server-assigned, `transparent` on the wire | distinct types; a client never mints one |
| `Timestamp` | value object | `i64` epoch-milliseconds (UTC) | ordering is total |

These are the facts a releasable chirp must keep true. They are registered as requirements
**REQ-DOM-001** (canonical model preserved), **REQ-DOM-002** (validated value objects enforced), and
**REQ-DOM-003** (entity invariants hold in *any* backing store, not only the in-memory one).

### 1.2 Identity and ordering

Ids are assigned from **monotonically increasing counters** today (`in_memory.rs`: `next_user_id`,
`next_chirp_id`), so a larger id denotes a later creation, and timelines order by **id descending** — a
stable proxy for reverse-chronological order that is immune to two chirps sharing a millisecond. This
ordering rule is part of the domain contract, not an implementation accident of the in-memory store: any
durable backing MUST preserve monotonic, non-reused, non-regressing id assignment so ordering and the
opaque-cursor pagination remain correct across restarts. This is **REQ-DOM-004**.

### 1.3 What the release bar adds to the domain

The product requirements surface several needs whose *data* belongs in this domain even though their
*mechanism* belongs to a sibling area. Each is modelled here as durable domain state so the rest of the
design (persistence, interfaces) has something concrete to build against:

- **A real account credential (and recovery factor).** Today an account is identified by handle alone
  and sign-in is handle-only — anyone who types a handle becomes that user (`LoginRequest` carries only a
  `username`; `login` issues a token for any known handle). A releasable product requires that an account
  carry a **durable authentication credential distinct from its public handle**, and optionally a
  recovery factor, so that knowing the handle cannot authenticate. The credential is domain state that
  must persist and must never be served on the wire; the *protocol* (what the credential is, how it is
  verified and recovered) is owned by identity & trust. This is **REQ-DOM-005** (trace `REQ-PROD-004`,
  `REQ-PROD-007`).
- **Sessions as first-class, durable state.** Sessions exist today only as an in-process
  `tokens: HashMap<String, UserId>` behind the repository, and are discarded on restart — which is
  exactly why a restart silently signs everyone out. The release models a **session** explicitly (its
  token identity, the user it authenticates, and enough lifecycle state to express issuance and
  invalidation) as durable state behind the persistence seam. This is **REQ-DOM-006** (trace
  `REQ-PROD-006`); session *lifetime and security properties* are owned by identity & trust.
- **Mutable profile fields.** `display_name` and `bio` are mutable account fields a user can change;
  `username` (the handle) and `id` are immutable identity. This is **REQ-DOM-008** (trace `REQ-PROD-008`).
- **Account-deletion data semantics.** Deletion removes the account and its personal data — its chirps,
  its follow edges in *both* directions, its likes, its sessions, and its credential — and defines what
  happens to references others hold (a reply whose target chirp was deleted, a follow edge pointing at
  the deleted account). This is **REQ-DOM-009** (trace `REQ-PROD-010`, `REQ-PROD-012`); the *consequence
  copy* shown to the user and the *authorization* to delete are owned by experience design and identity &
  trust respectively.
- **A defined "a user's data" for export.** Export needs a precise definition of what constitutes a
  user's own data — at minimum profile, authored chirps, and follow list — in a documented
  machine-readable shape built from `chirp-types`. This is **REQ-DOM-010** (trace `REQ-PROD-011`).
- **Moderation state.** A public posting product must be able to represent an **abusive chirp as removed**
  and an **abusive account as suspended/removed**, and must honour that state on every read (removed
  content is not served; a suspended account cannot act). This adds a small amount of lifecycle state to
  `Chirp` and `User` (or an adjacent moderation record). This is **REQ-DOM-011** (trace `REQ-PROD-027`,
  `REQ-PROD-028`, `REQ-PROD-033`); the moderation *tooling and authorization* are owned by identity &
  trust and operational readiness.
- **Reply/thread integrity (only if surfaced).** `reply_to` already exists on `Chirp`. If replies are
  part of the release, a `reply_to` MUST reference a chirp that exists (referential integrity), and a
  thread is the transitive set of chirps linked by `reply_to`; if replies are *not* surfaced, the field
  is persisted but no reply affordance is exposed. This is **REQ-DOM-007** (non-gating, trace
  `REQ-PROD-023`).

### 1.4 Relationships and invariants (the model as a whole)

- A `User` **authors** many `Chirp`s (`Chirp.author_id → User.id`).
- A `Chirp` **may reply to** at most one other `Chirp` (`reply_to → Chirp.id`, same-store referential
  integrity when replies are surfaced).
- `Follow` is a directed many-to-many edge over `User` with the no-self-follow invariant; the reverse
  traversal yields followers.
- `Like` is a many-to-many edge between `User` and `Chirp` with at-most-one-per-pair.
- `User.follower_count` / `User.following_count` and `Chirp.like_count` are **denormalized projections**
  of the edge sets and MUST always agree with them (`REQ-DOM-003`, and, across the durability boundary
  and under concurrency, `REQ-DOM-015`).
- A `User` **has** a durable credential (`REQ-DOM-005`) and zero or more `Session`s (`REQ-DOM-006`).
- The home `Timeline` of a viewer is the reverse-chronological run of chirps authored by the viewer or by
  anyone the viewer follows (the `timeline` seam already defines this).

---

## 2. Modules & responsibilities

*(section: `modules_and_responsibilities`)*

The product's parts and the responsibility each carries are **already divided** by the existing crates,
and that division is sound and must be held to. The release adds capability *within* this structure; it
does not re-lay the boundaries.

### 2.1 The division of responsibilities (held to the crates)

| Part | Crate(s) | Responsibility (what it owns) | Must stay true |
|---|---|---|---|
| **Shared contract** | `chirp-types` | The domain entities, value objects, ids, and the client↔server wire DTOs + error contract. Pure data + validation; no I/O, no framework, no async. | Single source of truth; server and frontend both depend on it and never re-declare a wire type. Growth is **additive-only**. |
| **Transport** | `chirp-server` `api/` (router, handlers, extractors, state) | Translating HTTP ⇄ DTO, authentication extraction (`AuthUser`), request validation (`ValidatedJson`), error rendering (`IntoResponse`), route composition (`build_router`), the `/healthz` probe, and the static/SPA fallback. | Holds persistence only as `Arc<dyn ChirpRepository>`; depends on the repository *capability*, never a concrete store. |
| **Error seam** | `chirp-server` `error.rs` | The one `ServerError` enum and its mapping to the `chirp-types` `ErrorCode`/`ApiError` and to HTTP status. | Variant ↔ code ↔ status mapping is frozen; `#[non_exhaustive]` so new categories are additive. |
| **Persistence seam** | `chirp-server` `repository/` (`ChirpRepository` trait) | The single boundary for all durable state, expressed in `chirp-types` terms; object-safe, async, `Send + Sync`, `&self`. | Every durable operation goes through this trait; it is the designed swap point for a real store. |
| **Default backing** | `chirp-server` `repository/in_memory.rs` | The infrastructure-free `InMemoryRepository` used for `cargo test` and local runs. | Remains available for tests; is **not** the release backing (see §3). |
| **Client** | `chirp-frontend` (on `ferric` / `ferric-router` / `ferric-http`) | The WebAssembly SPA: reactive state, routing, the typed REST client, and the route views (timeline, compose, profile). | Talks to the backend only through `chirp-types` DTOs; the `ferric` framework itself is a dependency, not chirp's to design here. |

### 2.2 What this division makes easy, and the one responsibility that must move

The `ChirpRepository` trait is the architectural keystone: because the transport layer holds
`Arc<dyn ChirpRepository>` and every handler delegates through it, **a durable store is added as a second
implementor of the existing trait without touching a single handler** (`REQ-DOM-014`). This is the
mechanism by which the release's headline change — durable persistence — is contained to one module.

The one responsibility that must **move into durable state** is session and credential storage. Today it
lives (as `tokens` and as the absence of any credential) inside `InMemoryRepository`; it must become part
of the durable model behind the same seam (`REQ-DOM-005`, `REQ-DOM-006`). No new top-level module is
required: the responsibilities are the same; their *backing* changes from process memory to a durable
store, which is exactly what the seam was designed to allow.

### 2.3 New capabilities, placed on the existing parts

The release-bar needs below are each placed on the part that already owns that responsibility — no new
layer is introduced:

- **New wire DTOs** (account update, logout, account deletion, data export, follow-list, moderation
  actions) are added to `chirp-types` `api` (`REQ-DOM-017`).
- **New repository capabilities** (session invalidation, account update, account deletion with cascade,
  data export, follow/follower listing, credential set/verify, chirp removal, account suspension) are
  added to the `ChirpRepository` trait (`REQ-DOM-018`).
- **New routes and handlers** for those capabilities are added to `api/router.rs` + `handlers.rs`
  (`REQ-DOM-017`, `REQ-DOM-019`).
- **New SPA views/flows** (sign-up, sign-out control, account-management area) are the frontend's and are
  owned by experience design; this design only fixes the DTOs and endpoints they consume.
- **Authorization checks** (actor == owner for account/content mutations; operator/moderator role for
  moderation) are expressed in the model as `REQ-DOM-020` and enforced per identity & trust's `REQ-ID-*`.

---

## 3. Persistence & state — the durable-persistence requirement

*(section: `persistence_requirement`)*

This is the single most important requirement in this design and the clearest release blocker.

### 3.1 The blocker, stated precisely

chirp keeps **all** application state in process memory. `InMemoryRepository` holds one `Store` behind a
`Mutex` containing `users`, `username_to_id`, `chirps`, `follows`, `likes`, `tokens`, and the id/token
counters; nothing is written outside the process. The deployment confirms and compounds this: the
workload runs `replicas: 1` **deliberately** (a second replica would hold independent state and serve
inconsistent data), the container root filesystem is read-only, and the only writable mount is an
`emptyDir` `/tmp`. There is **no database and no persistent volume**. The consequence is total: on every
restart, redeploy, or reschedule, **every account, chirp, follow, like, and session is gone**. For a
product whose purpose is to let people build a presence over time, this is a defining defect, not an
implementation detail.

The in-memory store is therefore a **baseline to replace, not to build on**. The release bar requires
durable persistence; the `InMemoryRepository` remains only as the test/local backing.

### 3.2 The durable-persistence requirement (the what)

**REQ-DOM-012** — *A user's data endures.* All domain state — accounts, their durable credentials,
sessions, chirps, follows, likes, the denormalized counts, the id counters, and moderation state —
persists in durable storage that **survives process restart, redeploy, and rescheduling**. This is the
domain/application face of `REQ-PROD-026` and `REQ-PROD-031`; its operational face (backups, restore,
health, scaling) is owned by operational readiness (`REQ-OPS-*`), which consumes this model.

The requirement has a single, testable bar input stated so anyone can check it:

> **Restart-survival bar.** Create an account, post at least one chirp, and follow at least one other
> account; then restart (or redeploy) the service; then confirm the account can still be signed into and
> that its chirps, follows, likes, and counts are all still present and correct. The product passes only
> when **nothing a user created is lost** across the restart. (Today this test fails completely.)

### 3.3 Where state lives

**REQ-DOM-013** — *State lives outside any single process's ephemeral memory and filesystem.* The durable
store MUST be external to the container's in-process memory and to its ephemeral filesystem (the
read-only root and the `emptyDir` `/tmp`), so that the lifecycle of the data is independent of the
lifecycle of any one pod. This is what makes `REQ-DOM-012` true in operation and what makes more than one
replica even conceivable. The **concrete store** (an embedded database on a persistent volume, or an
external managed database), its provisioning, and its backups are a *how* owned by operational readiness
and the implementation; this requirement fixes only that durable state must not be bound to a
single pod.

### 3.4 Consistency, atomicity, and concurrency

**REQ-DOM-015** — *Counts never diverge from edges, across the durability boundary.* Operations that
mutate an edge set together with a denormalized count — `follow`/`unfollow` with `follower_count`/
`following_count`, `like`/`unlike` with `like_count`, `create_chirp`, `create_user` with the handle
index, and account deletion's cascade — are applied so that a reader never observes a count that
disagrees with the edges, even under concurrent requests and even if the process fails mid-operation. The
in-memory store gets this for free from one global `Mutex`; a durable store MUST provide the equivalent
guarantee (a transaction, or equivalent atomic unit). The *mechanism* is deferred to the implementing
implementation; the *guarantee* is required here.

**REQ-DOM-016** — *The persistence seam is safe for concurrent access.* The `ChirpRepository` contract is
already `Send + Sync` with `&self` methods and interior mutability, so the transport layer can share one
`Arc<dyn ChirpRepository>` across concurrent requests. A durable implementor MUST honour that contract
(concurrent calls are correct). Whether the store additionally supports **multiple replicas** is an
operational-scaling decision owned by `REQ-OPS-*`; this design requires only that single-instance
concurrency is correct and that the current single-replica constraint is a consequence of the *store*,
not of the seam.

### 3.5 Schema versioning and evolution (the "how" named, deferred)

**REQ-DOM-022** — *The durable schema has a defined initial version and a forward migration path.* Because
durable data outlives any one deployment, the design requires that the durable representation carry a
defined schema version and that schema changes be applied through a repeatable migration path, so an
upgrade does not lose or corrupt existing user data. This is where an architecture "how" (the migration
tool, the DDL) is tempting: it is stated here as a **design requirement** and its mechanics are deferred
to the engineering implementation. (Trace `REQ-PROD-026`,
`REQ-PROD-034`.)

---

## 4. Interfaces & entry points

*(section: `interfaces_and_entry_points`)*

### 4.1 The internal seams between parts

There are three internal seams, all of which already exist and must be preserved:

1. **`chirp-types` as the wire seam.** The server and frontend meet only through `chirp-types` DTOs
   (camelCase JSON, `transparent` ids/tokens, value objects validated on deserialization, `ErrorCode`
   as `SCREAMING_SNAKE_CASE`, `ApiError` as the error body, `Page<T>` + opaque `Cursor` for pagination).
   New capability is added by adding DTOs here, never by a parallel shape. This is **REQ-DOM-017**.
2. **`ChirpRepository` as the persistence seam.** All durable state crosses this one object-safe async
   trait; the durable store plugs in here as a second implementor. This is **REQ-DOM-014**.
3. **`ServerError` as the error seam.** Every layer returns one typed error that maps deterministically
   to `ErrorCode` → HTTP status; the mapping (`VALIDATION_ERROR`/400, `UNAUTHORIZED`/401, `FORBIDDEN`/403,
   `NOT_FOUND`/404, `CONFLICT`/409, `INTERNAL`/500) is frozen and `#[non_exhaustive]` for additive
   growth. New failure modes reuse these categories.

### 4.2 The capabilities the persistence seam must carry at the release bar

The `ChirpRepository` trait today covers `create_user`, `get_user`, `login`, `resolve_token`,
`create_chirp`, `timeline`, `follow`, `unfollow`, `like`, `unlike`. The release bar requires the seam to
additionally express (names illustrative; the trait is the contract, the method set is the *what*):

| New capability | Why (product need) | Owning REQ-DOM / sibling seam |
|---|---|---|
| Set / verify a real credential | credentialed sign-in replaces handle-only | `REQ-DOM-005`; protocol → `REQ-ID-*` |
| Invalidate a session (sign-out) + durable sessions | deliberate sign-out; no silent sign-out on restart | `REQ-DOM-006`, `REQ-DOM-018`; → `REQ-ID-*` |
| Begin / complete account recovery | regain access after credential loss | `REQ-DOM-005`; flow → `REQ-ID-*` |
| Update profile (`display_name`, `bio`) | manage own account | `REQ-DOM-008`, `REQ-DOM-018` |
| Delete account (cascade) | leave cleanly; deletion honoured | `REQ-DOM-009`, `REQ-DOM-018` |
| Export a user's data | take data out | `REQ-DOM-010`, `REQ-DOM-018` |
| List following / followers | manage follows; export completeness | `REQ-DOM-018` |
| Remove a chirp / suspend an account | act on abuse | `REQ-DOM-011`, `REQ-DOM-018`; authz → `REQ-ID-*` |
| Report store health | health reflects whether the product works | `REQ-DOM-021`; probe → `REQ-OPS-*` |

These capabilities, and the DTOs and routes that expose them, are **REQ-DOM-017** / **REQ-DOM-018**. Each
new repository capability maps to exactly one endpoint on the existing `/api/…` surface, rendered with the
`chirp-types` error contract.

### 4.3 Authorization boundaries in the model

**REQ-DOM-020** — Every state-changing operation names the **authenticated actor** and the **resource
owner**. A user may act only on their own account and content — post as self, edit and delete *their own*
account, export *their own* data, follow/unfollow/like/unlike as self — while **moderation** operations
(remove a chirp authored by another, suspend another account) are reserved to an operator/moderator role.
This design fixes the *boundary* in the model (which actor may touch which resource); the *enforcement
detail and role definition* are owned by identity & trust (`REQ-ID-*`). Reading the public timeline and
profiles requires no actor (trace `REQ-PROD-013`).

### 4.4 Product entry points

**REQ-DOM-019** — chirp is a **single self-contained service** that serves the SPA and the REST API from
one origin, and this stays true at the release bar:

- **HTTP surface:** the JSON API under `/api/…`, the unauthenticated `GET /healthz` liveness probe, and a
  static/SPA fallback that serves `index.html` for any unmatched path so the single-page app owns
  client-side routing. API routes are namespaced under `/api/…` so they never collide with the SPA path
  space.
- **Read vs. write:** read endpoints (timeline, profile) are reachable without authentication; write
  endpoints require an authenticated actor via the `Authorization: Bearer` session token resolved by
  `AuthUser` (trace `REQ-PROD-013`, `REQ-PROD-004`).
- **Process entry point:** the `chirp-server` binary constructs the shared `AppState` over a
  `ChirpRepository` implementor and serves `build_router(state)` — the same composition root the
  integration tests use — with graceful shutdown on `SIGINT`/`SIGTERM`.
- **Configuration:** the service is configured through documented environment settings only (today
  `CHIRP_BIND_ADDR`, `CHIRP_STATIC_DIR`, `RUST_LOG`); the durable store adds its own documented
  configuration (e.g. a connection/location setting). No configuration is hard-coded. (Trace
  `REQ-PROD-034`; the concrete config surface is owned with operational readiness.)

**REQ-DOM-021** — The product's health signal MUST be able to report **unhealthy when the durable store is
unreachable**, so that "the process is up" cannot be mistaken for "the product works for users". The
model/interface must therefore expose a store-reachability check that a health endpoint can consult; the
probe wiring and the readiness/liveness policy are owned by operational readiness (`REQ-OPS-*`). (Trace
`REQ-PROD-030`.)

---

## 5. Requirements & acceptance criteria

*(section: `requirements_and_acceptance_criteria`)*

The register below is the authoritative `REQ-DOM-*` list. Each requirement states what must be true of the
domain or application; each acceptance criterion is written so a person who never met the author can check
it against the built product (a release-bar criterion can attach a verification method from the schema
vocabulary — `manual_user_check`, `automated_test`, `inspection`, `operational_check`). Each carries a
`release_gating` flag (`true` where a releasable chirp genuinely depends on it) and, where applicable, a
trace to `REQ-PROD-*`.

### A. Domain model

#### REQ-DOM-001 — The `chirp-types` domain model is the canonical model, preserved
- **Statement:** The releasable product represents exactly the entities, edges, and aggregates of the
  `chirp-types` domain (`User`, `Chirp`, `Follow`, `Like`, `Timeline`) with their documented fields; the
  shared schema is extended additively and never re-declared or forked.
- **Acceptance criteria:** The wire shapes produced and consumed match the `chirp-types` definitions
  byte-for-byte on field names and types (camelCase, transparent ids); a round-trip of each entity
  through the API preserves every field. No second definition of any wire entity exists in the server or
  frontend. *(inspection; automated_test)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-016`, `REQ-PROD-018`, `REQ-PROD-019`, `REQ-PROD-020`, `REQ-PROD-021`.

#### REQ-DOM-002 — Validated value objects are enforced at the domain boundary
- **Statement:** `Username` (1..=15 chars; ASCII letter/digit/underscore) and `ChirpText` (1..=280
  characters) are validated on every path into the domain, including deserialization, so an invalid value
  is never stored or served.
- **Acceptance criteria:** Submitting a handle that is empty, longer than 15 characters, or containing a
  disallowed character is rejected with a `VALIDATION_ERROR`; submitting chirp text longer than 280
  characters is rejected likewise; a valid value is accepted; no stored `User`/`Chirp` ever carries a
  value that violates these rules. *(automated_test; manual_user_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-002`, `REQ-PROD-016`.

#### REQ-DOM-003 — Entity invariants hold in any backing store
- **Statement:** Regardless of the store, the domain invariants hold: denormalized counts
  (`follower_count`, `following_count`, `like_count`) agree with the edge sets; no user follows
  themselves; at most one like exists per `(user, chirp)`; a timeline is reverse-chronological.
- **Acceptance criteria:** After any sequence of follow/unfollow and like/unlike, the displayed counts
  equal the actual edge counts; a self-follow attempt is rejected with `FORBIDDEN`; liking an
  already-liked chirp leaves the count unchanged; a fetched timeline is strictly newest-first. These hold
  against the durable backing, not only the in-memory one. *(automated_test; operational_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-018`, `REQ-PROD-020`, `REQ-PROD-021`.

#### REQ-DOM-004 — Ids are opaque, server-assigned, monotonic, and stable across restarts
- **Statement:** Entity ids are server-assigned opaque values that increase monotonically (larger id =
  later creation) and are never reused or regressed, including across restarts; timeline ordering and
  cursor pagination rely on this.
- **Acceptance criteria:** Ids returned by the API are opaque to the client; two chirps created in order
  have strictly increasing ids; after a restart, newly created entities receive ids greater than all
  previously assigned ids (no reuse), and cursor-based paging returns each chirp exactly once with no
  gaps or duplicates. *(automated_test; operational_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-018`, `REQ-PROD-026`.

#### REQ-DOM-005 — An account carries a durable credential distinct from its public handle
- **Statement:** An account carries a durable authentication credential (and optionally a recovery
  factor) that is distinct from its public handle and is never served on the wire; handle knowledge alone
  cannot authenticate or act as the account.
- **Acceptance criteria:** A persisted account has a credential field that is not returned by any read
  endpoint; a sign-in presenting only a handle (no valid credential) cannot obtain a session; the
  legitimate owner presenting the credential can. (The authentication protocol itself is `REQ-ID-*`.)
  *(inspection; manual_user_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-004`, `REQ-PROD-007`.

#### REQ-DOM-006 — Sessions are first-class durable state
- **Statement:** A session is modelled explicitly (its token identity, the user it authenticates, and
  lifecycle state sufficient to express issuance and invalidation) and is stored durably behind the
  persistence seam, so sessions are not lost on a restart.
- **Acceptance criteria:** A signed-in user remains signed in across a service restart (their presented
  token still resolves), rather than being silently signed out; an invalidated (signed-out) session no
  longer resolves. (Session lifetime and security are `REQ-ID-*`.) *(operational_check; manual_user_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-006`, `REQ-PROD-026`.

#### REQ-DOM-007 — Reply/thread model is coherent if surfaced
- **Statement:** If replies are part of the release, a `Chirp.reply_to` references an existing chirp
  (referential integrity) and a thread is the set of chirps linked by `reply_to`; if replies are not
  surfaced, `reply_to` is persisted but no reply affordance is exposed.
- **Acceptance criteria:** Either (a) a reply can only be created against an existing chirp and a thread
  read returns the linked chirps in order; or (b) no endpoint accepts a reply and no stored chirp has a
  non-null `reply_to` created through the product. No dangling `reply_to` is served. *(automated_test)*
- **Release gating:** false *(matches `REQ-PROD-023`; the release may defer full replies but must not ship
  a dead reply affordance)*
- **Traceability:** `REQ-PROD-023`.

#### REQ-DOM-008 — Profile fields are mutable; identity is immutable
- **Statement:** `display_name` and `bio` are mutable account fields; `username` (handle) and `id` are
  immutable identity.
- **Acceptance criteria:** A signed-in user can change their own `display_name` and `bio` and the change
  persists and is shown on their profile on a later visit; no operation changes an account's `id`, and
  handle immutability is enforced (or any handle-change path is a deliberate, separately specified
  capability). *(automated_test; manual_user_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-008`.

#### REQ-DOM-009 — Account deletion has defined cascade semantics
- **Statement:** Deleting an account removes the account and its personal data — its chirps, its follow
  edges in both directions, its likes, its sessions, and its credential — and defines the treatment of
  references others hold.
- **Acceptance criteria:** After deletion, the account cannot be signed into, its profile returns
  `NOT_FOUND`, its authored chirps and its follow/like edges are removed, its sessions no longer resolve,
  and the defined handling of others' references (e.g. a reply to a now-deleted chirp) is applied
  consistently and does not leave a dangling reference that errors on read. *(automated_test;
  operational_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-010`, `REQ-PROD-012`.

#### REQ-DOM-010 — "A user's data" is defined for export
- **Statement:** The data a user can export is defined as, at minimum, their profile, their authored
  chirps, and their follow list, serialized in a documented machine-readable format built from
  `chirp-types` shapes.
- **Acceptance criteria:** A user can obtain an export that contains their profile, all their chirps, and
  their follow list, in a documented format that parses outside chirp and whose contents match what the
  product shows for that account. *(automated_test; manual_user_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-011`.

#### REQ-DOM-011 — Moderation state is representable and honoured on reads
- **Statement:** The model can represent a chirp as removed and an account as suspended/removed, and every
  read honours that state (removed content is not served; a suspended account cannot act).
- **Acceptance criteria:** After an authorized actor removes a chirp, that chirp no longer appears in any
  timeline or profile read; after an account is suspended, it cannot post, follow, or like, and its
  state is observable to an operator. (Authorization and tooling are `REQ-ID-*` / `REQ-OPS-*`.)
  *(automated_test; manual_user_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-027`, `REQ-PROD-028`, `REQ-PROD-033`.

### B. Persistence & state

#### REQ-DOM-012 — Durable persistence of all domain state (in-memory store is a release blocker)
- **Statement:** All domain state — accounts, durable credentials, sessions, chirps, follows, likes, the
  denormalized counts, the id counters, and moderation state — persists in durable storage that survives
  process restart, redeploy, and rescheduling. The in-memory store is not an acceptable release backing.
- **Acceptance criteria (restart-survival bar):** Create an account, post ≥1 chirp, and follow ≥1
  account; restart or redeploy the service; then the account can still be signed into and its chirps,
  follows, likes, and counts are all present and correct — **nothing a user created is lost**.
  *(operational_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-026`, `REQ-PROD-031`. Operational face → `REQ-OPS-*`.

#### REQ-DOM-013 — Durable state lives outside any single pod's ephemeral memory and filesystem
- **Statement:** The durable store is external to the container's in-process memory and ephemeral
  filesystem (the read-only root and `emptyDir` `/tmp`), so data outlives any one pod.
- **Acceptance criteria:** Inspection of the running configuration shows application data is held in a
  store whose lifecycle is independent of the pod (not process memory, not the pod's ephemeral volumes);
  deleting and recreating the pod does not lose data. (The concrete store and its provisioning are
  `REQ-OPS-*`.) *(inspection; operational_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-031`, `REQ-PROD-026`.

#### REQ-DOM-014 — Durable storage plugs into the existing `ChirpRepository` seam
- **Statement:** Durable persistence is provided as a second implementor of the existing
  `ChirpRepository` trait, preserving its object-safety, async signatures, `Send + Sync` bound,
  `&self`/interior-mutability shape, and `ServerError` return contract — so no handler or transport code
  changes to adopt it.
- **Acceptance criteria:** The durable backing implements the `ChirpRepository` trait and is selected
  where `AppState` is constructed; the handler and router code are unchanged relative to `c805525` except
  for additive new endpoints; the integration tests run against the trait, not a concrete store.
  *(inspection; automated_test)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-026`.

#### REQ-DOM-015 — Multi-write operations keep counts consistent with edges, atomically
- **Statement:** Operations that mutate an edge set together with a denormalized count (follow/unfollow,
  like/unlike, create_chirp, create_user + handle index, account-deletion cascade) are applied as an
  atomic unit so a reader never observes a count that disagrees with the edges, even under concurrency or
  a mid-operation failure.
- **Acceptance criteria:** Concurrent and interleaved follow/like operations never leave a count that
  disagrees with the edge set; an operation interrupted partway leaves the store in a consistent
  pre- or post-state, never a half-applied one. *(automated_test; operational_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-020`, `REQ-PROD-021`, `REQ-PROD-026`.

#### REQ-DOM-016 — The persistence seam is safe for concurrent access
- **Statement:** A durable `ChirpRepository` implementor is correct under concurrent requests on a single
  instance (honouring the `Send + Sync`, `&self` contract); the current single-replica constraint is a
  property of the store, not of the seam.
- **Acceptance criteria:** Concurrent requests against a single instance produce correct, consistent
  results; documentation states whether the store supports more than one replica, and if it does not, the
  single-replica constraint is recorded as a store property. (Scale-out itself is `REQ-OPS-*`.)
  *(automated_test; inspection)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-031`.

#### REQ-DOM-022 — The durable schema is versioned with a forward migration path
- **Statement:** The durable representation carries a defined schema version and schema changes are
  applied through a repeatable migration path, so upgrading the product does not lose or corrupt existing
  user data. (The migration tooling and DDL are an engineering "how", deferred.)
- **Acceptance criteria:** The durable store declares an initial schema version; applying the documented
  migration path to a store holding data preserves that data and advances the version; a fresh deploy
  initialises the schema from empty. *(inspection; operational_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-026`, `REQ-PROD-034`.

### C. Interfaces & entry points

#### REQ-DOM-017 — The REST/wire contract is preserved and extended only through `chirp-types`
- **Statement:** Existing endpoints and their DTOs are preserved; new capabilities are expressed by adding
  DTOs to `chirp-types` (camelCase, `transparent` ids/tokens, value-object validation, `Page<T>`/`Cursor`
  pagination, `ErrorCode`/`ApiError` errors) and never by a parallel or divergent shape.
- **Acceptance criteria:** Every request/response for a new capability is a `chirp-types` type; no
  endpoint returns an ad-hoc error body (all failures render as `ApiError` with a valid `ErrorCode`); the
  frontend and server share each new type with no second definition. *(inspection; automated_test)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-001`, `REQ-PROD-005`, `REQ-PROD-008`, `REQ-PROD-010`, `REQ-PROD-011`.

#### REQ-DOM-018 — The `ChirpRepository` seam covers the release-bar capabilities
- **Statement:** The persistence seam expresses, in addition to today's operations, the capabilities the
  release needs: set/verify credential, invalidate session, begin/complete recovery, update profile,
  delete account (cascade), export user data, list following/followers, remove a chirp, and suspend an
  account — each returning the typed `ServerError`.
- **Acceptance criteria:** The trait (or its documented successor) declares a method for each listed
  capability; each maps to exactly one API endpoint; each failure mode returns the correct `ErrorCode`
  category. *(inspection; automated_test)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-005`, `REQ-PROD-007`, `REQ-PROD-008`, `REQ-PROD-010`, `REQ-PROD-011`,
  `REQ-PROD-027`, `REQ-PROD-028`.

#### REQ-DOM-019 — chirp remains a single self-contained service with defined entry points
- **Statement:** One service serves the REST API under `/api/…`, the `GET /healthz` probe, and the
  static/SPA fallback from a single origin; read endpoints need no authentication, write endpoints require
  a bearer session token; the service is configured through documented environment settings only, and the
  durable store adds its own documented configuration.
- **Acceptance criteria:** A single deployed service answers both the API and the SPA; a logged-out client
  can read the timeline and a profile; a write without a valid token is rejected with `UNAUTHORIZED`; all
  runtime configuration (including the store location/connection) is supplied through documented settings,
  none hard-coded. *(manual_user_check; inspection)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-013`, `REQ-PROD-004`, `REQ-PROD-034`.

#### REQ-DOM-020 — Authorization boundaries are fixed in the model
- **Statement:** Every state-changing operation names an authenticated actor and the resource owner; a
  user may act only on their own account and content, and moderation operations are reserved to an
  operator/moderator role.
- **Acceptance criteria:** A user cannot edit, delete, export, or post as another account; a user cannot
  remove another user's chirp or suspend another account through the ordinary user surface; an authorized
  operator/moderator can. (Role definition and enforcement detail are `REQ-ID-*`.) *(automated_test;
  manual_user_check)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-004`, `REQ-PROD-010`, `REQ-PROD-028`.

#### REQ-DOM-021 — The health interface can reflect store reachability
- **Statement:** The model/interface exposes a durable-store reachability check that the product's health
  signal can consult, so health reports unhealthy when the store is unreachable even if the process is
  alive.
- **Acceptance criteria:** When the durable store is made unreachable, the product's health signal reports
  unhealthy; when the store is reachable and the product can serve users, it reports healthy. (Probe
  wiring and policy are `REQ-OPS-*`.) *(operational_check; inspection)*
- **Release gating:** true
- **Traceability:** `REQ-PROD-030`.

### Register summary

| REQ-DOM | Concern | Release gating | Traces to REQ-PROD |
|---|---|---|---|
| 001 | Canonical domain model preserved | true | 016, 018, 019, 020, 021 |
| 002 | Validated value objects enforced | true | 002, 016 |
| 003 | Entity invariants hold in any store | true | 018, 020, 021 |
| 004 | Opaque monotonic ids stable across restarts | true | 018, 026 |
| 005 | Durable credential distinct from handle | true | 004, 007 |
| 006 | Sessions as durable state | true | 006, 026 |
| 007 | Reply/thread coherent if surfaced | false | 023 |
| 008 | Mutable profile, immutable identity | true | 008 |
| 009 | Account-deletion cascade semantics | true | 010, 012 |
| 010 | "A user's data" defined for export | true | 011 |
| 011 | Moderation state representable/honoured | true | 027, 028, 033 |
| 012 | Durable persistence of all state (blocker) | true | 026, 031 |
| 013 | State lives outside ephemeral pod memory/fs | true | 031, 026 |
| 014 | Durable store plugs into `ChirpRepository` | true | 026 |
| 015 | Atomic multi-write; counts consistent | true | 020, 021, 026 |
| 016 | Seam safe for concurrent access | true | 031 |
| 017 | Wire contract preserved/extended via `chirp-types` | true | 001, 005, 008, 010, 011 |
| 018 | Seam covers release-bar capabilities | true | 005, 007, 008, 010, 011, 027, 028 |
| 019 | Single service; defined entry points/config | true | 013, 004, 034 |
| 020 | Authorization boundaries in the model | true | 004, 010, 028 |
| 021 | Health reflects store reachability | true | 030 |
| 022 | Durable schema versioned + migration path | true | 026, 034 |

### Deferred "how" (named here, built elsewhere)

To keep this design to *what, not how*, the following are stated above as requirements and their
implementation is deferred — to engineering and later code work, or to the
named sibling design area:

- The **concrete durable store** (embedded DB on a volume vs. external database), its provisioning, its
  **backups and restore**, and the **health-probe policy** and **scaling** → operational readiness
  (`REQ-OPS-*`), which consumes `REQ-DOM-012`–`REQ-DOM-016`, `REQ-DOM-021`, `REQ-DOM-022`.
- The **authentication protocol**, **session lifetime/security**, **recovery flow**, **deletion/export
  authorization**, and **moderation role & tooling** → identity & trust (`REQ-ID-*`), composing with
  `REQ-DOM-005`, `REQ-DOM-006`, `REQ-DOM-009`, `REQ-DOM-010`, `REQ-DOM-011`, `REQ-DOM-020`.
- The **migration tooling and DDL** → the implementation, realising `REQ-DOM-022`.
- All **screens and interaction** for sign-up, sign-out, account management, export, and moderation →
  experience design (`REQ-UX-*`), consuming the DTOs and endpoints fixed by `REQ-DOM-017`–`REQ-DOM-019`.
