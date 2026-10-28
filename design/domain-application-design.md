# chirp — Domain & application design

This document states chirp's domain and application design **as the requirements and structure the
product must have** — concrete enough to build and test against, held to the crates that already exist
in the repository `Nousix-LLC/nswe-demo` at commit `c805525` (`ferric` with `ferric-http` and
`ferric-router`, `chirp-types`, `chirp-server`, `chirp-frontend`). It says *what must be true*, not
*how* to build it: wherever an engineering decision is tempting — a database, a schema, a migration
mechanism, a session format — the requirement is stated and the decision is left to the later code
work. It owns the `REQ-DOM-*` requirements and marks each with a `release_gating` flag so the release
bar can consume it.

It has five sections:

1. **Domain model** — the entities a reader and poster deal with, their relationships, and their
   invariants, and the canonical entity vocabulary the rest of the design draws on.
2. **Modules and responsibilities** — how the running system divides into parts and what each owes the
   others, held to the four crates.
3. **Persistence requirement** — what of the product's state must survive a restart, where it lives,
   and the durability, backup, and integrity posture a release depends on (the current in-memory store
   is a release blocker, not a baseline).
4. **Interfaces and entry points** — the seams between the parts and the entry points a person or an
   operator reaches.
5. **Requirements and acceptance criteria** — the consolidated `REQ-DOM-*` register (the union of the
   requirements the four sections above establish), each with testable acceptance criteria, a
   `release_gating` flag, and a trace to the product requirement (`REQ-PROD-*`) it refines where one
   applies.

---

## Domain model

### What this section fixes

This section states *what is true of chirp's domain* — the entities a reader and poster deals with,
how they relate, and the invariants that must hold — as requirements something can be built and
tested against. It says nothing about *how* the data is stored or served: persistence is the
`persistence_requirement` section (`REQ-DOM-100..149`), and the division of the running system into
parts is the `modules_and_responsibilities` / `interfaces_and_entry_points` sections
(`REQ-DOM-200..249`). Where an engineering decision is tempting here — a schema, an index, a session
format — it is stated as a requirement on the model and the decision is left to the later code work.

The model is held to what already exists in the repository at commit `c805525`: the shared
`crates/chirp-types` crate (the single source of truth for the types that cross the wire) and the
`crates/chirp-server` domain (the `ChirpRepository` seam and its in-memory backing). Every entity below
names the `chirp-types` type that already carries it, so the model refines the running product rather
than inventing a parallel one. Where the model requires something the product does not yet have (a
session a person can end, a defined outcome when an account is deleted), that is called out as a gap the
requirement closes, traced to the product requirement (`REQ-PROD-*`) it serves.

#### Scope of this section

- **In this section**: the core social domain — accounts and their public profile, the handle, the
  chirp and its authorship, replies, the follow graph, likes, the timeline a reader sees, and the
  session that identifies a signed-in person — with their relationships and invariants.
- **Referenced, owned elsewhere**: the authentication *credential* and how a session is proven
  (identity/security design, `REQ-ID-*`); the *verified point of contact* mechanism and verification
  (identity/privacy, `REQ-ID-*`); safety entities — *report*, *block*, *mute* — and moderation
  (identity/abuse, `REQ-ID-*`); *notifications* (experience and product, `REQ-UX-*` / `REQ-PROD-029`);
  durability, backups, and limits (operational readiness, `REQ-OPS-*`). These are named in the entity
  vocabulary where the core model must reference them, so the picture is complete and persistence knows
  what account data exists, but their internals are not modelled here.
- **Not in the product** (recorded so the absence is a decision, not an oversight): **mentions** and
  mention-based notification. The product requirements settle rich social features beyond the core loop
  as out of scope; replies (`REQ-PROD-003`) are the only chirp-to-chirp relationship the product
  carries, so the model defines replies and does **not** introduce a mention entity or a mention edge.
  If a later phase adopts mentions, it extends this vocabulary; it is absent today by decision.

### Entity vocabulary (canonical)

The rest of the design references entities **only** by the canonical names below — the persistence
requirement, the module and interface design, and the consolidated requirements all draw on this one
vocabulary. Anything that needs an entity this vocabulary does not name has found a gap to report, not
a licence to coin a new name.

| Canonical name | What it is | `chirp-types` type (at `c805525`) |
|---|---|---|
| **Account** (a.k.a. **User**) | A registered person: their identity, their public profile, and the account-level data an account-based product carries. "User" and "Account" name the same entity — *User* is the public/profile face, *Account* the owned-and-managed face. | `chirp_types::domain::User` |
| **Handle** | The unique, human-chosen identifier others know an account by and follow. | `chirp_types::domain::Username` |
| **Display name** | The free-form, non-unique name an account shows under. | `User::display_name` (field) |
| **Bio** | An optional short free-form profile description. | `User::bio` (field, `Option`) |
| **Chirp** | A single short public post, attributed to one authoring account. | `chirp_types::domain::Chirp` |
| **Chirp text** | The validated body of a chirp. | `chirp_types::domain::ChirpText` |
| **Reply** | A chirp that answers another chirp; the relationship, not a separate entity — a chirp whose *reply target* is set. | `Chirp::reply_to` (field, `Option<ChirpId>`) |
| **Follow** | A directed "follows" relationship from one account (the follower) to another (the followee). | `chirp_types::domain::Follow` |
| **Like** | A relationship recording that an account likes a chirp. | `chirp_types::domain::Like` |
| **Timeline** | The reverse-chronological run of chirps a viewer sees: their own chirps and those of the accounts they follow. A derived view, not stored state. | `chirp_types::domain::Timeline`, paged as `chirp_types::api::Page<Chirp>` |
| **Session** | A proof that a request is acting as a particular account, carried as an opaque bearer token. | `chirp_types::api::SessionToken`, issued in `AuthResponse` |
| **Account id / Chirp id** | Opaque, server-assigned identifiers for an account and a chirp. | `chirp_types::ids::UserId`, `chirp_types::ids::ChirpId` |
| **Timestamp** | A point in time on an entity (created-at), UTC. | `chirp_types::ids::Timestamp` |
| **Credential** *(referenced; owned by `REQ-ID-*`)* | The secret an account proves itself with at sign-in. Named so the model and persistence know account-secret data exists and must be protected; its structure and handling are the identity/security design's. | — (not in `chirp-types` at `c805525`) |
| **Verified contact** *(referenced; owned by `REQ-ID-*`)* | A confirmed point of contact for an account, the basis for recovery and important notifications. Named so persistence carries it as part of the account record; what contact is collected and how it is verified is the identity/privacy design's. | — (not in `chirp-types` at `c805525`) |

**Relationships at a glance.** An Account **authors** Chirps (one author per chirp; many chirps per
account). A Chirp **replies to** at most one other Chirp. An Account **follows** other Accounts (the
social graph: a set of directed Follow edges). An Account **likes** Chirps (a set of Like edges). A
Timeline is **derived** for a viewing Account from its own chirps plus the chirps of the accounts it
follows. A Session **belongs to** exactly one Account.

### The entities and their invariants

The requirements (`REQ-DOM-001..049`) carry the detail; this prose states the model's spine so a
reader has the shape before the register.

**Accounts and identity.** An account is identified two ways: by an **opaque, server-assigned id**
that never changes and is never reused, and by a **handle** the person chooses, which is unique across
the product and well-formed (1–15 characters of ASCII letters, digits, or underscore — the
`Username` rule already enforced at `c805525`). An account carries a mutable display name and optional
bio, an immutable creation time, and server-maintained follower/following counts that clients read but
do not set. The account record also carries identity-area data — a credential and a verified contact —
whose internals the identity design owns but which the model acknowledges so they are persisted and
protected as part of the account.

**Chirps, authorship, and replies.** A chirp is a short public post with a validated body of 1–280
characters (the `ChirpText` rule at `c805525`), attributed to exactly one authoring account that must
exist. A chirp may be a **reply**, in which case it records the id of the chirp it answers, and that
target must exist when the reply is posted; a chirp that is not a reply records no target. The parent
linkage is what lets a reply be read in the context of its conversation. A posted chirp's body and
authorship are immutable; only its server-maintained like count and the existence of replies to it
change over time.

**The social graph.** Following is a set of directed edges: *follower → followee*. A person cannot
follow themselves, there is at most one edge per ordered pair (following is idempotent), and unfollowing
removes the edge idempotently. The follower/following counts on an account always equal the number of
edges into and out of it.

**Engagement.** A like is an edge *account → chirp*. There is at most one like per (account, chirp)
pair; liking is idempotent and unliking removes the like idempotently. A chirp's like count always
equals the number of distinct accounts that currently like it.

**The timeline.** A viewer's timeline is **derived**, not stored: it is the reverse-chronological
(newest-first) run of the viewer's own chirps and the chirps of the accounts the viewer currently
follows, returned a bounded page at a time behind an opaque cursor. Because it is derived from the
current follow graph and chirp set, it reflects follows and unfollows without any separate
materialized state the model must keep consistent.

**Sessions.** A session identifies the account a request acts as. From the person's side the model
requires three things of a session regardless of how authentication is built: it authenticates exactly
one account; it can be **ended** so it can no longer be used (sign-out, which the product does not
support at `c805525`); and it does not live forever. How a session is established and proven — the
credential exchange and the token's internals — is the identity/security design's (`REQ-ID-*`); the
`SessionToken` type already treats the token as opaque.

**Integrity across the model.** Every reference resolves: a chirp's author, a reply's target, a
follow's two endpoints, and a like's account and chirp all name entities that exist. This referential
integrity is what makes **account deletion** a domain question and not only a privacy one: the model
must define a consistent outcome for a deleted account's chirps, follows, likes, and whether its freed
handle may be registered again. The model fixes that a defined, applied outcome is required and that no
dangling reference may result; the specific policy (what is retained, whether the handle returns to the
pool) is settled with the identity/privacy design against `REQ-PROD-021`.

---

## Modules and responsibilities

### What this section fixes

This section states *how chirp's running system divides into parts, what each part is responsible
for, and the contracts at the seams between them* — as requirements and structure, not as code. It
is held to the four crates that already exist at commit `c805525`, so the design refines the running
product rather than proposing a parallel one. Where an engineering *how* is implied — which HTTP
framework, how WebAssembly is bundled, how a datastore is wired — the requirement is stated and the
decision is left to the later code work.

The division is described from the outside: what each part owes the others, and what crosses each
seam, so that a reader can tell whether a given change belongs in the shared types, the server, or
the frontend, and so the parts compose into one product. Entities are named only by the canonical
vocabulary the domain model establishes (Account/User, Handle, Chirp, Reply, Follow, Like, Timeline,
Session); this section assigns responsibility over those entities, it does not redefine them.

### The parts at a glance

chirp is four crates in one repository, in two layers:

- **`ferric`** — the in-house client-side UI framework (reactive signals and a virtual DOM), with
  **`ferric-router`** (client-side routing) and **`ferric-http`** (a browser HTTP client) beside
  it. `ferric` is the engagement's *other* deliverable; chirp is the product that exercises it.
  `ferric` carries **no** chirp domain knowledge.
- **`chirp-types`** — the single source of truth for every type that crosses the wire: the
  identifiers, the domain entities and their validated value objects, and the request/response
  shapes, pagination envelope, and error contract. Pure data and validation — no I/O, no framework,
  no async — so both a native server and a WebAssembly frontend import it unchanged.
- **`chirp-server`** — the backend: a REST service that implements the API over the shared types,
  enforces the domain invariants, owns the persistence seam and the data behind it, serves the
  single-page app, and exposes the health endpoint.
- **`chirp-frontend`** — the single-page web application the person uses: it presents the screens,
  holds the signed-in Session in reactive state, and talks to the backend through the shared
  contract. It holds **no** durable state of its own.

The module decomposition is stated as requirements `REQ-DOM-200..208` in the register below: the
four-crate partition and its two layers (`REQ-DOM-200`), `ferric` as a domain-free framework
(`REQ-DOM-201`), `chirp-types` as the single wire contract (`REQ-DOM-202`), `chirp-server` owning the
REST API and the domain invariants (`REQ-DOM-203`) and the persistence seam (`REQ-DOM-204`) and serving
the app and health endpoint (`REQ-DOM-205`), `chirp-frontend` presenting the product and holding only
session state (`REQ-DOM-206`), server-side enforcement of domain rules (`REQ-DOM-207`), and one-part
ownership of each entity's lifecycle (`REQ-DOM-208`).

---

## Persistence requirement

### What this section fixes

This section states the **durable-persistence requirement** for chirp: what of the product's state must
survive the serving process going away and coming back, where that state has to live for the survival to
hold, and the backup, recovery, and data-integrity posture a release depends on. It states all of this as
requirements the release bar can be checked against — what must be true from the operator's and the user's
side — and it leaves the engineering *how* to the later code work.

It says nothing about which database, which schema, which migration tool, or how connections are pooled.
Those are real decisions, but they are decisions about *how* to meet a requirement, not the requirement
itself. Wherever one is tempting below, it is written as the property the product must have and the choice is
handed to the later code work. The one place the current implementation is named is the place it
is the problem: chirp at `c805525` keeps its state in process memory behind the `ChirpRepository` seam in the
`chirp-server` crate, so a restart, update, or redeploy empties the product. The product requirements call
this out as a release **blocker**, not a baseline (`REQ-PROD-037`), and every requirement here exists to
close it.

The vocabulary is the domain model's. "Account," "Chirp," "Follow," "Like," "Session," "Timeline,"
"Credential," and "Verified contact" are used below exactly as the domain model defines them
(`REQ-DOM-001..049`); this section introduces no entity the model does not name. Where persistence has a
consequence the model already fixed — referential integrity (`REQ-DOM-036`), ids never reused
(`REQ-DOM-007`), counts that match their edges (`REQ-DOM-021`, `REQ-DOM-024`), a defined deletion outcome
(`REQ-DOM-037`) — the requirement here is that the fixed property keeps holding across a restart and a
restore, not a restatement of the property.

#### Scope of this section

- **In this section**: the concrete set of state that must survive a restart and redeploy; the requirement
  that the product's authoritative state lives in a durable store distinct from the serving process; the
  durability property that an acknowledged write is not lost; the integrity properties that must hold after a
  restart and after a restore; and the requirement that the durable data can be backed up and recovered and
  is enumerable for a person's own export.
- **Referenced, owned elsewhere**: the operational *mechanics* of durability — backup cadence and retention,
  the restore runbook, a recovery-point / recovery-time target, and the availability target — are the
  operational-readiness design's (`REQ-OPS-*`); this section states the capability the bar needs and hands
  the operational targets to that design, which consumes this section as its `durable_data` input. The
  structure and handling of the account **Credential** and **Verified contact** are the identity/security
  design's (`REQ-ID-*`); this section requires only that they persist durably and stay non-public. The
  **how** — database engine, schema, migrations, connection handling, serialization — is the later code
  work's.
- **Not required to be durable** (recorded so the absence is a decision): the **Timeline**. The domain model
  fixes the timeline as a *derived* view, not stored state (`REQ-DOM-025`, `REQ-DOM-028`); it therefore does
  not need to survive a restart as stored data. What must survive is the data it is derived from — the
  viewer's chirps and the follow graph — so that the same timeline is reconstructable after a restart
  (`REQ-DOM-108`). Durably materializing a timeline is neither required nor prohibited here; it is a `how`
  left to the later code work, constrained only by the derived-view invariants the model already fixed.

### The requirement in prose

**What must survive.** A person who creates an account, posts, follows, and likes must find all of it still
there the next time they visit, and an operator must be able to restart, update, and redeploy the service as
normal operations without losing any of it. Concretely, the state that MUST survive a restart and a redeploy
is: every **Account** (its profile fields, its server-assigned **Account id**, its creation **Timestamp**,
and its follower/following counts); the account-associated **Credential** and **Verified contact** that the
identity area owns but that live on the account record; every **Chirp** (its body, its author, its creation
time, its reply linkage, and its like count); the entire **Follow** graph; every **Like**; and every active
**Session**. This set is named by `REQ-PROD-037`, which lists accounts, chirps, follows, likes, and sessions
explicitly; the account's private identity data is added here because the model places it on the account
(`REQ-DOM-033`, `REQ-DOM-034`).

**Durability is a correctness property, not only a presence one.** It is not enough that data *tends* to be
there after a restart. A write the product has acknowledged to a person — a posted chirp, a new follow, a
like, a profile edit — must not be lost by a restart that happens immediately afterward; once the product
tells someone "done," that fact is durable (`REQ-DOM-109`). And the invariants the domain model fixes must
still hold after the process comes back and after a restore from backup: every reference still resolves
(`REQ-DOM-110`), ids are still unique and still never reused (`REQ-DOM-111`), the counts still equal their
edges (`REQ-DOM-107`), and an applied account deletion stays applied (`REQ-DOM-112`). Recovery that brings the
rows back but breaks these invariants has not recovered the product.

**Where state lives.** For any of the above to hold, the product's authoritative state cannot live in the
serving process. The requirement is that it lives in a **durable store distinct from the serving process**,
so that replacing, restarting, or redeploying the process does not touch the data (`REQ-DOM-113`). The
`ChirpRepository` seam in `chirp-server` is the right place for this to resolve: today that seam is backed by
an in-memory store, which is exactly the blocker; the requirement is that the seam (or its equivalent) be
backed by the durable store instead (`REQ-DOM-114`). Which store, and how it is reached, is the later code
work's to decide. Because serving now depends on that store, the product's readiness signal must reflect
whether the store is reachable — a process that cannot reach its durable state is not ready to serve, even
though it is running (`REQ-DOM-115`), which is the persistence-side half of the readiness requirement the
operational-readiness design owns (`REQ-PROD-039`).

**Backup, recovery, and a person's own copy.** Once data is durable there is something to protect, and the
product must be able to protect it: the durable data can be backed up and restored, and a restore brings the
accounts, chirps, follows, and likes back intact and consistent (`REQ-DOM-116`). The operational shape of
this — how often a backup is taken, how long it is kept, how much data loss is tolerable, and the restore
procedure — is the operational-readiness design's (`REQ-OPS-*`), which consumes this requirement; this
section fixes only that the capability exists and recovers a correct product. Separately, a person must be
able to obtain their own data, so the durable store must be enumerable per account — a person's profile,
their full set of chirps, and the accounts they follow can all be read out for export (`REQ-DOM-117`),
feeding the export surface the experience and identity designs own.

**Change over time.** Redeploys are named alongside restarts in the product requirement, and a redeploy may
carry a change to how data is shaped. The requirement is that evolving the product does not lose or corrupt
existing durable data: after an upgrade that changes the data's shape, the accounts, chirps, follows, and
likes that existed before are still present and still satisfy their invariants (`REQ-DOM-118`). The mechanics
of schema and data migration are the later code work's; the property that an upgrade preserves existing
data is the bar's.

---

## Interfaces and entry points

### What this section fixes

This section states the **seams between the parts** and the **entry points into the product** as
requirements and structure, from the outside: what the REST API must offer the single-page app as a
contract (not route-by-route code), what the shared types boundary must carry so the server and
frontend agree, how the persistence seam is shaped so a durable store drops in, how the frontend is
delivered, and the entry points a person or an operator reaches. Every engineering *how* — the exact
HTTP framework, URL strings, the serialization library, the datastore wiring, the WebAssembly build
— is deferred to the later code work. Entities are named only by the canonical vocabulary.

Three seams hold the product together, and they are the backbone of this section:

1. **The REST contract** — the typed request/response surface the frontend calls and the server
   implements, with `chirp-types` as its single source of truth (`REQ-DOM-215..222`).
2. **The persistence seam** — the one boundary (`ChirpRepository`) through which all durable state
   flows, so the store is replaceable (`REQ-DOM-223`).
3. **The delivery seam** — how the single-page app is served, and the entry points (the app URL,
   the in-app routes, the health endpoint, the service's configuration) a person or operator meets
   (`REQ-DOM-224..227`).

---

## Requirements and acceptance criteria

This is the consolidated `REQ-DOM-*` register: the union of the requirements the four sections above
establish, carried verbatim. Each requirement carries an `id`, a `statement` (what must be true), an
`acceptance_criteria` a person who never met the author can check — against the running product, or by
inspection of the shared contract in the repository — a `release_gating` flag (`true` when a releasable
product depends on it), and `traces_to` the product requirement (`REQ-PROD-*`) it refines where one
applies. The three parts own disjoint number ranges — the domain model `001..049`, persistence
`100..149`, modules and interfaces `200..249` — so no id collides. The consolidated traceability matrix
follows the register.

### Domain model (`REQ-DOM-001..049`)

Acceptance criteria that read "inspect `crates/chirp-types` …" are checkable by inspection of the
shared contract; the rest are checkable against the running product.

#### Accounts, handles, and profile

#### REQ-DOM-001 — The account is the one entity for a registered person
- **statement**: A registered person is represented by exactly one account entity carrying an opaque id, a handle, a display name, an optional bio, a creation time, and follower/following counts; the same entity is the account's public profile (what others see) and the owned account (what its holder manages).
- **acceptance_criteria**: Register an account and fetch it; a single profile is returned carrying a handle, a display name, a bio field (possibly empty), a creation time, and follower/following counts. Inspect `crates/chirp-types/src/domain.rs`; `User` carries exactly these fields and no second "profile" type exists.
- **release_gating**: true
- **traces_to**: REQ-PROD-008

#### REQ-DOM-002 — Handles are well-formed
- **statement**: A handle is 1 to 15 characters, each an ASCII letter, digit, or underscore; anything else is rejected against the handle field.
- **acceptance_criteria**: Create an account with handle `alice_01`; it is accepted. Attempt handles that are empty, 16 characters, contain a space, or contain `!`; each is rejected with a field-level validation error naming the handle. Inspect `chirp_types::domain::Username`; `USERNAME_MAX_CHARS` is 15 and the rule matches.
- **release_gating**: true
- **traces_to**: REQ-PROD-009

#### REQ-DOM-003 — Handles are unique
- **statement**: At most one account exists for a given handle; registering a handle already in use is refused as a conflict.
- **acceptance_criteria**: Register handle `bob`; register `bob` again — the second attempt is refused as a conflict naming the handle, and only one account holds `bob`.
- **release_gating**: true
- **traces_to**: REQ-PROD-009

#### REQ-DOM-004 — An account has a display name
- **statement**: An account carries a free-form display name that need not be unique and is shown wherever the account is presented.
- **acceptance_criteria**: Register two accounts with the same display name and different handles; both are created, and each profile shows the display name given.
- **release_gating**: true

#### REQ-DOM-005 — Bio is optional
- **statement**: An account's bio is optional; an account with no bio is well-formed and its bio is explicitly absent, not an empty-string stand-in that cannot be told from "unset".
- **acceptance_criteria**: Register an account without a bio and fetch it; the bio is absent (null), distinguishable from an account whose bio is the empty string. Inspect `User::bio`; it is an `Option`.
- **release_gating**: true

#### REQ-DOM-006 — Account and chirp ids are opaque and server-assigned
- **statement**: Account ids and chirp ids are assigned by the product, are opaque to clients, and a client never supplies one on creation; a client only echoes back an id the product issued.
- **acceptance_criteria**: Register an account and post a chirp without supplying any id; the product assigns both ids. Inspect `chirp_types::ids`; `UserId`/`ChirpId` are server-minted newtypes and the create-request types (`CreateUserRequest`, `CreateChirpRequest`) carry no id field.
- **release_gating**: true

#### REQ-DOM-007 — Ids are never reused
- **statement**: Once assigned, an account id or chirp id is never reassigned to a different entity, including after the original entity is deleted.
- **acceptance_criteria**: Record an account's id; delete the account (per REQ-DOM-037); over the product's lifetime no other account is ever observed carrying that id.
- **release_gating**: true
- **traces_to**: REQ-PROD-020

#### REQ-DOM-008 — Creation time is recorded and immutable
- **statement**: Each account and each chirp records the UTC time it was created, and that time does not change after creation.
- **acceptance_criteria**: Fetch an account and a chirp; each carries a creation timestamp. Re-fetch later; the timestamp is unchanged. Inspect `chirp_types::ids::Timestamp`; it is epoch-milliseconds UTC.
- **release_gating**: true

#### REQ-DOM-009 — Display name and bio are editable; id and creation time are not
- **statement**: An account holder can change their display name and bio after sign-up; an account's id and creation time are immutable.
- **acceptance_criteria**: Signed in, change your display name and bio; the change shows on your profile on the next view. No product surface changes an account's id or creation time.
- **release_gating**: true
- **traces_to**: REQ-PROD-016

#### REQ-DOM-010 — Handle mutability follows the product's stated policy
- **statement**: Whether a handle may change after sign-up is governed by a single stated policy, and the model behaves consistently with it; if handles are fixed, no path changes one, and if a handle change is offered, the uniqueness and format invariants (REQ-DOM-002, REQ-DOM-003) continue to hold after the change and following relationships remain attached to the account, not to the old handle string.
- **acceptance_criteria**: Find the product's stated handle-change position. If fixed, confirm no surface changes a handle. If changes are offered, change a followed account's handle and confirm its followers still follow the account and the new handle is validated and unique.
- **release_gating**: false
- **traces_to**: REQ-PROD-018

#### Chirps, authorship, and replies

#### REQ-DOM-011 — A chirp is a short post with one author
- **statement**: A chirp is a single public post carrying an opaque id, the id of exactly one authoring account that exists, a validated body, a creation time, a like count, and an optional reply target.
- **acceptance_criteria**: Post a chirp; it is returned with an id, your account as author, the body you wrote, a creation time, a like count of zero, and no reply target. Inspect `chirp_types::domain::Chirp`; it carries exactly these fields.
- **release_gating**: true
- **traces_to**: REQ-PROD-001

#### REQ-DOM-012 — Chirp body length is bounded
- **statement**: A chirp body is 1 to 280 characters, counted by Unicode scalar value (not bytes); an empty or over-length body is rejected and no chirp is created.
- **acceptance_criteria**: Post a 280-character body (including one of 280 multi-byte emoji); it is accepted. Post a 281-character or empty body; it is rejected against the body field and no chirp appears. Inspect `chirp_types::domain::ChirpText`; `CHIRP_TEXT_MAX_CHARS` is 280 and length is by character.
- **release_gating**: true
- **traces_to**: REQ-PROD-001

#### REQ-DOM-013 — Every chirp is attributed to an existing author
- **statement**: A chirp is always attributed to one account that exists at post time; a chirp with no author or an author that does not exist cannot be created.
- **acceptance_criteria**: Post a chirp while signed in; it is attributed to your account. There is no product path to post a chirp attributed to a nonexistent account.
- **release_gating**: true
- **traces_to**: REQ-PROD-001

#### REQ-DOM-014 — A posted chirp's body and author are immutable
- **statement**: After a chirp is posted, its body text and author do not change; only its server-maintained like count and whether other chirps reply to it change over time.
- **acceptance_criteria**: Post a chirp and re-fetch it over time; its body and author are unchanged. No product surface edits a chirp's text or reassigns its author.
- **release_gating**: true

#### REQ-DOM-015 — A reply records the chirp it answers, and that chirp must exist
- **statement**: A chirp may be a reply, in which case it records the id of exactly one existing chirp it answers; replying to a chirp that does not exist is refused as not-found rather than posted with a dangling target.
- **acceptance_criteria**: Reply to a visible chirp; the reply carries that chirp's id as its reply target. Attempt to reply to a nonexistent or removed chirp id; it is refused as not-found and no reply is created. Inspect `Chirp::reply_to`; it is `Option<ChirpId>`.
- **release_gating**: true
- **traces_to**: REQ-PROD-003

#### REQ-DOM-016 — The reply linkage supports reading a conversation
- **statement**: The reply target linkage is sufficient to move from a reply to the chirp it answers, so a conversation can be reconstructed from the model without any separate thread entity.
- **acceptance_criteria**: Given a reply, use its recorded reply target to fetch the parent chirp and present the two in relation. No entity beyond `Chirp` and its `reply_to` linkage is required to do so.
- **release_gating**: false
- **traces_to**: REQ-PROD-004

#### The social graph (follow)

#### REQ-DOM-017 — Following is a directed relationship between two accounts
- **statement**: A follow is a directed relationship from a follower account to a followee account, both of which exist, recording when it was created.
- **acceptance_criteria**: Follow another account; the relationship is recorded from you to them. Inspect `chirp_types::domain::Follow`; it carries `follower_id`, `followee_id`, and a creation time.
- **release_gating**: true
- **traces_to**: REQ-PROD-006

#### REQ-DOM-018 — No account follows itself
- **statement**: An account cannot follow itself; a self-follow is refused.
- **acceptance_criteria**: Attempt to follow your own account; it is refused and no self-edge exists.
- **release_gating**: true
- **traces_to**: REQ-PROD-006

#### REQ-DOM-019 — Following is idempotent
- **statement**: At most one follow edge exists per ordered (follower, followee) pair; following an account already followed succeeds and changes nothing, and unfollowing an account not followed succeeds and changes nothing.
- **acceptance_criteria**: Follow an account twice; the second call succeeds and the follow is not duplicated and counts do not double. Unfollow an account you do not follow; it succeeds and nothing changes.
- **release_gating**: true
- **traces_to**: REQ-PROD-006

#### REQ-DOM-020 — Unfollowing removes the relationship
- **statement**: Unfollowing an account removes the follow edge so the followee's chirps no longer enter the follower's timeline.
- **acceptance_criteria**: Follow an account and see their chirps in your timeline; unfollow them; their chirps no longer appear in your timeline.
- **release_gating**: true
- **traces_to**: REQ-PROD-006

#### REQ-DOM-021 — Follower and following counts match the edges
- **statement**: An account's follower count equals the number of accounts currently following it, and its following count equals the number it currently follows; the counts are maintained by the product and are read-only to clients.
- **acceptance_criteria**: Note an account's follower count; have one account follow it; the follower count increases by exactly one. Unfollow; it returns. A client cannot set these counts directly (they are not inputs to any create/update request).
- **release_gating**: true

#### Engagement (like)

#### REQ-DOM-022 — A like relates an account to a chirp
- **statement**: A like records that one account likes one chirp, both of which exist, and when the like was recorded.
- **acceptance_criteria**: Like a chirp; the like is recorded from your account to that chirp. Inspect `chirp_types::domain::Like`; it carries `user_id`, `chirp_id`, and a creation time.
- **release_gating**: true
- **traces_to**: REQ-PROD-005

#### REQ-DOM-023 — At most one like per account and chirp
- **statement**: There is at most one like for a given (account, chirp) pair; liking a chirp already liked succeeds and does not add a second like, and unliking a chirp not liked succeeds and changes nothing.
- **acceptance_criteria**: Like a chirp twice; the second call succeeds and the like count does not increase again. Unlike twice; the first returns the count, the second changes nothing.
- **release_gating**: true
- **traces_to**: REQ-PROD-005

#### REQ-DOM-024 — Like count matches the likes
- **statement**: A chirp's like count equals the number of distinct accounts that currently like it; it is maintained by the product and read-only to clients.
- **acceptance_criteria**: Note a chirp's like count; have one account like it; the count increases by exactly one. Unlike; it returns. A client cannot set the count directly.
- **release_gating**: true
- **traces_to**: REQ-PROD-005

#### The timeline

#### REQ-DOM-025 — The timeline is derived from the viewer's own chirps and those they follow
- **statement**: A viewer's home timeline is the set of the viewer's own chirps together with the chirps of every account the viewer currently follows; it is a derived view, not stored state the model keeps separately.
- **acceptance_criteria**: Follow one account and post yourself; your timeline contains your chirps and theirs, and contains no chirps from accounts you neither are nor follow.
- **release_gating**: true
- **traces_to**: REQ-PROD-002

#### REQ-DOM-026 — The timeline is newest-first
- **statement**: Chirps in a timeline are ordered reverse-chronologically, newest first.
- **acceptance_criteria**: Post two chirps a moment apart; in the timeline the later one appears before the earlier one. Inspect `chirp_types::domain::Timeline`; its ordering invariant is reverse-chronological.
- **release_gating**: true
- **traces_to**: REQ-PROD-002

#### REQ-DOM-027 — The timeline is returned in bounded pages
- **statement**: A timeline is read a bounded page at a time, each page carrying an opaque continuation cursor when more chirps remain; a reader cannot force the product to return every chirp in one response.
- **acceptance_criteria**: Request a timeline; a bounded page is returned with a continuation cursor when more remain. Pass the cursor; the next, older page is returned. A page carries no more than the product's maximum, independent of any requested size. Inspect `chirp_types::api::Page` and `TimelineQuery`; the page carries `items` and an optional `next_cursor`, and the query's limit is optional.
- **release_gating**: true
- **traces_to**: REQ-PROD-002

#### REQ-DOM-028 — The timeline reflects the current follow graph
- **statement**: Because the timeline is derived, following or unfollowing an account changes what a subsequent timeline read returns, with no separate materialized timeline to reconcile.
- **acceptance_criteria**: Read your timeline; follow a new account that has posted; read again — their chirps now appear. Unfollow; read again — their chirps are gone.
- **release_gating**: true
- **traces_to**: REQ-PROD-002

#### Sessions

#### REQ-DOM-029 — A session identifies exactly one account
- **statement**: A session authenticates requests as exactly one account; actions taken under a session are attributed to that account and no other.
- **acceptance_criteria**: Sign in and post, follow, and like; each action is attributed to the signed-in account. Inspect `chirp_types::api::AuthResponse`; a session is issued for one authenticated `User`.
- **release_gating**: true
- **traces_to**: REQ-PROD-010

#### REQ-DOM-030 — A session can be ended
- **statement**: A session can be ended so that it can no longer be used to act as the account — the model supports sign-out that invalidates the session, which the product does not provide at `c805525`.
- **acceptance_criteria**: Sign in, then end the session (sign out); replay a request with the ended session's token — it is refused. The account is no longer acted as under that session.
- **release_gating**: true
- **traces_to**: REQ-PROD-014

#### REQ-DOM-031 — A session does not live forever
- **statement**: A session has a bounded lifetime after which it no longer authenticates; the specific duration and renewal behavior are the identity/security design's to set.
- **acceptance_criteria**: A session presented after its lifetime has elapsed is refused and the person is returned to sign-in. (The exact lifetime is defined by `REQ-ID-*`; this requirement is that a lifetime exists and is enforced.)
- **release_gating**: true
- **traces_to**: REQ-PROD-015

#### REQ-DOM-032 — The session token is opaque
- **statement**: A session is carried as an opaque bearer token; its internal structure is not part of the domain contract and clients treat it as an opaque string to present, not to interpret.
- **acceptance_criteria**: Inspect `chirp_types::api::SessionToken`; it is a transparent string wrapper with no interpreted structure, and no client surface parses its contents. (How the token is formed and proven is the identity/security design's, `REQ-ID-*`.)
- **release_gating**: true

#### Account-associated data owned by the identity area

#### REQ-DOM-033 — An account carries an authentication credential
- **statement**: An account has an associated authentication credential — a secret the holder proves themselves with — distinct from the handle, so that knowing a handle alone does not establish a session. The credential's structure and handling are the identity/security design's; the model requires that such data exists on the account and is never part of any public profile.
- **acceptance_criteria**: Knowing only another account's handle, attempt to obtain a session for it; it is refused (the handle is not the credential). A credential value is never returned in any profile or public response. (The credential mechanism is `REQ-ID-*`.)
- **release_gating**: true
- **traces_to**: REQ-PROD-008, REQ-PROD-010

#### REQ-DOM-034 — An account carries a verified point of contact
- **statement**: An account has an associated, confirmed point of contact that is the basis for account recovery and important notifications; the model requires it as part of the account record and as non-public data. What contact is collected and how it is verified is the identity/privacy design's.
- **acceptance_criteria**: An account has a contact value that the product treats as verified before relying on it for recovery, and that value is never returned to another account or in a public response. (The contact and verification mechanism is `REQ-ID-*`.)
- **release_gating**: true
- **traces_to**: REQ-PROD-012

#### REQ-DOM-035 — Private account data is never public
- **statement**: An account's credential, verified contact, and session data are never exposed to other accounts or in public responses; only the public profile fields — handle, display name, bio, follower/following counts, and public chirps — are visible to others.
- **acceptance_criteria**: Fetch another account's profile and any public response with no session and with a different account's session; only handle, display name, bio, and counts appear. Credential, contact, and session data never appear.
- **release_gating**: true
- **traces_to**: REQ-PROD-024

#### Integrity, deletion, and derivable facts

#### REQ-DOM-036 — Every reference resolves
- **statement**: Every reference between entities resolves to an entity that exists: a chirp's author, a reply's target (when set), a follow's follower and followee, and a like's account and chirp all name existing entities; no dangling reference is observable.
- **acceptance_criteria**: Through every product surface that creates a relationship, the referenced entities must exist or the action is refused (REQ-DOM-013, REQ-DOM-015, REQ-DOM-017, REQ-DOM-022). No surface produces a chirp, follow, or like naming an entity that does not exist.
- **release_gating**: true

#### REQ-DOM-037 — Account deletion has a defined, applied outcome
- **statement**: Deleting an account produces a single defined outcome that the product applies and states: what becomes of the account's chirps (including those others have replied to or liked), its follow edges, and its likes, and whether its freed handle may be registered again — with no dangling reference left behind (REQ-DOM-036 continues to hold after a deletion).
- **acceptance_criteria**: Find the stated deletion outcome. Delete an account that has chirps (some replied-to and liked), follows, and likes; afterward the account cannot sign in, the product's state matches the stated outcome, and no remaining chirp, follow, or like references the deleted account in a way the outcome did not define. Whether the handle can be re-registered matches the stated policy.
- **release_gating**: true
- **traces_to**: REQ-PROD-020, REQ-PROD-021

#### REQ-DOM-038 — A person's own data is enumerable for export
- **statement**: The data belonging to an account — its profile, its chirps, and the set of accounts it follows — is enumerable from the model so the holder can obtain their own data in a portable form.
- **acceptance_criteria**: For a signed-in account, the model yields that account's profile, the full set of its chirps, and the full set of accounts it follows — the data a data-export surface returns. (The export surface and format are the experience/identity designs'; this requirement is that the data is enumerable per account.)
- **release_gating**: true
- **traces_to**: REQ-PROD-019

#### REQ-DOM-039 — Timestamps are UTC and totally ordered
- **statement**: Every creation timestamp is milliseconds since the Unix epoch in UTC, so timestamps across accounts and chirps are directly and unambiguously comparable for ordering (for example, the newest-first timeline of REQ-DOM-026).
- **acceptance_criteria**: Inspect `chirp_types::ids::Timestamp`; it is epoch-milliseconds UTC and ordered. Two chirps posted in sequence carry timestamps whose order matches the order they were posted.
- **release_gating**: true

#### REQ-DOM-040 — Server-maintained counts are read-only to clients
- **statement**: The denormalized counts in the model — an account's follower and following counts and a chirp's like count — are maintained by the product and are never accepted as client input; a client that sends a count on a create or update has it ignored, not honored.
- **acceptance_criteria**: Inspect the create/update request types in `chirp_types::api`; none carries a follower, following, or like count. Any attempt to set a count directly has no effect on the stored value.
- **release_gating**: true

### Persistence (`REQ-DOM-100..149`)

Durability criteria prefer an **operational_check** — observe the property on the running product
across a real restart, redeploy, or restore — because that is the only check that actually proves
durability; a few criteria that fix a boundary or name the current blocker are checkable by
**inspection** of the repository at `c805525`.

#### What must survive a restart

#### REQ-DOM-100 — The product's user state survives restart, update, and redeploy
- **statement**: The product's user-visible state — every Account, Chirp, Follow, Like, and active Session — survives the serving process being restarted, updated, and redeployed; a person's account and content are still present and unchanged on their next visit, for any deployment of chirp.
- **acceptance_criteria**: On a running deployment, create an Account, post Chirps, follow accounts, and like Chirps; restart (and separately, redeploy) the serving process; sign in again — the Account exists and its Chirps, Follows, and Likes are all present and unchanged. Observed as an operational_check across a real process restart and a real redeploy.
- **release_gating**: true — the in-memory store at `c805525` loses everything on restart; this is the release blocker, not a style choice.
- **traces_to**: REQ-PROD-037

#### REQ-DOM-101 — Accounts persist in full
- **statement**: An Account survives a restart carrying all of its state: its opaque Account id, its Handle, its Display name, its Bio (absent or present, preserved as given), its creation Timestamp, and its follower/following counts.
- **acceptance_criteria**: Register an Account with a Handle, Display name, and Bio; restart the service; fetch the Account — the same Account id, Handle, Display name, Bio, and creation Timestamp are returned, and the counts are unchanged. An Account registered with no Bio still reads back with its Bio absent (not an empty string).
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-001

#### REQ-DOM-102 — Account private identity data persists durably and stays non-public
- **statement**: The account-associated Credential and Verified contact persist durably as part of the Account across a restart, so a person can still sign in and still recover their account after a restart; this data is never returned in any public response. Its structure and handling are the identity/security design's; this requirement is that it is durable and non-public.
- **acceptance_criteria**: Register an Account and establish its Credential and Verified contact; restart the service; the person can still sign in with their Credential, and the recovery path still recognises the Verified contact. Fetch the Account's public profile before and after the restart with another account's Session and with no Session — the Credential, Verified contact, and Session data never appear.
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-033, REQ-DOM-034, REQ-DOM-035

#### REQ-DOM-103 — Chirps persist in full
- **statement**: A Chirp survives a restart carrying its Chirp id, its author's Account id, its Chirp text, its creation Timestamp, its like count, and its reply target when it is a Reply.
- **acceptance_criteria**: Post a Chirp and a Reply to it; restart the service; fetch both — each returns with the same id, author, text, and creation time; the Reply still records the id of the Chirp it answers; the like counts are unchanged.
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-011, REQ-DOM-015

#### REQ-DOM-104 — The follow graph persists
- **statement**: Every Follow edge survives a restart with its follower and followee Account ids intact, so who follows whom is unchanged across a restart.
- **acceptance_criteria**: Have one Account follow another; restart the service; the follow relationship is still recorded in the same direction, the followee's chirps still enter the follower's Timeline, and both accounts' follower/following counts are unchanged.
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-017

#### REQ-DOM-105 — Likes persist
- **statement**: Every Like survives a restart with its Account id and Chirp id intact, so what each person has liked is unchanged across a restart.
- **acceptance_criteria**: Like a Chirp; restart the service; the Like is still recorded from the same Account to the same Chirp, and the Chirp's like count is unchanged.
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-022

#### REQ-DOM-106 — Active sessions survive a restart, and ended sessions stay ended
- **statement**: An active Session remains valid across a restart so a signed-in person is not silently signed out by a redeploy; a Session that has been ended (sign-out) or has passed its lifetime stays unusable across a restart and is never revived by one.
- **acceptance_criteria**: Sign in to obtain a Session; restart the service; a request carrying that Session's token is still accepted and still acts as the same Account. Separately, end a Session (sign out) and then restart; a request carrying the ended Session's token is refused, not revived.
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-029, REQ-DOM-030, REQ-DOM-031

#### REQ-DOM-107 — Server-maintained counts are correct after a restart
- **statement**: After a restart, each Account's follower and following counts still equal its current Follow edges and each Chirp's like count still equals the distinct Accounts that like it — whether the counts are stored durably or recomputed from the durable edges, the result observed by a client is consistent with the edges.
- **acceptance_criteria**: Record an Account's follower count and a Chirp's like count; restart the service; both counts read back equal to the number of current edges (add and remove a Follow and a Like after the restart and confirm the counts track the edges by exactly one each way).
- **release_gating**: true
- **traces_to**: REQ-DOM-021, REQ-DOM-024

#### REQ-DOM-108 — The timeline is reconstructable from durable data after a restart
- **statement**: Because the Timeline is a derived view and not stored state, it need not itself be durable; the data it is derived from — the viewer's Chirps and the current Follow graph — must be durable, so the same Timeline is produced after a restart as before it.
- **acceptance_criteria**: Build a Timeline for a viewer who follows several accounts; note its contents and order; restart the service; read the Timeline again with no further writes — it returns the same Chirps in the same newest-first order. No separate stored Timeline is required for this to hold.
- **release_gating**: true
- **traces_to**: REQ-PROD-002, REQ-DOM-025, REQ-DOM-028

#### Durability as a correctness property

#### REQ-DOM-109 — An acknowledged write is durable before it is acknowledged
- **statement**: Any create or change the product confirms to a person — posting a Chirp, following or unfollowing, liking or unliking, editing a profile, deleting an account — is durable at the moment it is acknowledged, so a restart immediately after the acknowledgement does not lose it.
- **acceptance_criteria**: Perform a write and receive the product's success response; without any further action, restart the service; the written state is present after the restart. Observed as an operational_check: the window between acknowledgement and the next restart does not lose acknowledged writes.
- **release_gating**: true
- **traces_to**: REQ-PROD-037

#### REQ-DOM-110 — Referential integrity holds across restart and restore
- **statement**: After a restart and after a restore from backup, every reference in the model still resolves — each Chirp's author, each Reply's target, each Follow's two endpoints, and each Like's Account and Chirp name entities that exist — with no dangling reference introduced by stopping, restarting, or recovering the product.
- **acceptance_criteria**: On a populated deployment, restart (and separately, restore from a backup); then traverse chirps, replies, follows, and likes — every referenced Account and Chirp resolves, and no Chirp, Follow, or Like references an entity that is absent.
- **release_gating**: true
- **traces_to**: REQ-DOM-036

#### REQ-DOM-111 — Ids stay unique and are never reused across a restart
- **statement**: Account id and Chirp id assignment survives a restart such that ids already issued are never reassigned to a different entity after a restart; id generation does not reset to a point that would collide with existing ids.
- **acceptance_criteria**: Record several existing Account ids and Chirp ids; restart the service; create new Accounts and Chirps — the newly assigned ids collide with none of the recorded ids, and no existing entity's id has changed. Over the deployment's life, no id is ever observed on two entities.
- **release_gating**: true
- **traces_to**: REQ-DOM-006, REQ-DOM-007

#### REQ-DOM-112 — An applied account deletion is durable and leaves no half-applied state
- **statement**: Once an account deletion has been applied, its defined outcome (what becomes of the account's Chirps, Follows, and Likes, and whether the freed Handle may be re-registered) survives a restart; a restart during a deletion leaves the product in a consistent state — the deletion is either fully applied or not applied, never half-applied with dangling references.
- **acceptance_criteria**: Delete an Account that has Chirps (some replied-to and liked), Follows, and Likes; restart the service; the Account still cannot sign in, the stated deletion outcome is still reflected, and the freed-Handle policy still holds. Across restarts the product's state matches the stated outcome and never shows a partially deleted Account with dangling references.
- **release_gating**: true
- **traces_to**: REQ-PROD-020, REQ-PROD-021, REQ-DOM-037

#### Where state lives

#### REQ-DOM-113 — Authoritative state lives in a durable store distinct from the serving process
- **statement**: The product's authoritative state lives in a durable store that is distinct from the serving process, so that stopping, restarting, replacing, or redeploying the serving process does not alter or lose any of the state in REQ-DOM-100. The serving process holds no authoritative state that only exists in its memory.
- **acceptance_criteria**: Replace the serving process entirely (for example, roll it to a fresh instance with no shared memory) without touching the data store; all Accounts, Chirps, Follows, Likes, and active Sessions are intact afterward — demonstrating the authoritative state did not live in the replaced process. Observed as an operational_check.
- **release_gating**: true
- **traces_to**: REQ-PROD-037

#### REQ-DOM-114 — The repository seam is backed by the durable store; the in-memory backing is the blocker this closes
- **statement**: The persistence seam the product reads and writes through — the `ChirpRepository` in the `chirp-server` crate, or its equivalent — is backed by the durable store of REQ-DOM-113 rather than by process memory. The in-memory backing at `c805525` is named as the release blocker this requirement closes. The choice of store engine, the schema, migrations, and connection handling are deferred to the later code work.
- **acceptance_criteria**: Inspect `crates/chirp-server` at `Nousix-LLC/nswe-demo @ c805525`; the `ChirpRepository` is backed by an in-memory store (the blocker). On the released product, the same seam is backed by a durable store, confirmed by REQ-DOM-100 holding across a restart. No requirement here names a specific engine.
- **release_gating**: true
- **traces_to**: REQ-PROD-037

#### REQ-DOM-115 — Readiness reflects durable-store availability
- **statement**: The product reports itself ready to serve only when its durable store is reachable; a serving process that is running but cannot reach its durable state reports not-ready rather than ready.
- **acceptance_criteria**: Query the readiness signal while the durable store is unavailable to the serving process; it reports not-ready. Query it while the store is reachable and the product can serve; it reports ready. (The readiness signal's shape and the liveness-vs-readiness distinction are the operational-readiness design's; this requirement is that readiness depends on store reachability.)
- **release_gating**: true
- **traces_to**: REQ-PROD-039

#### Backup, recovery, and a person's own copy

#### REQ-DOM-116 — Durable data can be backed up and restored intact
- **statement**: The durable user data can be backed up and restored, and a restore brings back the Accounts, Chirps, Follows, and Likes consistent with every model invariant (REQ-DOM-110, REQ-DOM-111, REQ-DOM-107). The backup cadence, retention, recovery-point/recovery-time targets, and restore procedure are the operational-readiness design's; this requirement is that the capability exists and recovers a correct product.
- **acceptance_criteria**: On a populated deployment, produce a backup of the user data; then, on a store restored from that backup, the Accounts, Chirps, Follows, and Likes are present and the invariants hold (references resolve, ids are unchanged, counts match edges). Observed as an operational_check (backup then restore).
- **release_gating**: true
- **traces_to**: REQ-PROD-038

#### REQ-DOM-117 — Durable data is enumerable per account for export
- **statement**: The durable store is enumerable per Account so a person's own data — their profile, the full set of their Chirps, and the full set of Accounts they follow — can be read out for a self-service export, reflecting the committed durable state at the time of the request.
- **acceptance_criteria**: For a signed-in Account with Chirps and Follows, the product can produce that Account's profile, its complete set of Chirps, and its complete set of followed Accounts, matching what is in the durable store. (The export surface and portable format are the experience/identity designs'; this requirement is that the durable data is enumerable per Account.)
- **release_gating**: true
- **traces_to**: REQ-PROD-019, REQ-DOM-038

#### Change over time

#### REQ-DOM-118 — Upgrading the product preserves existing durable data
- **statement**: Evolving the product — including a redeploy that changes how the data is shaped — preserves the durable data that existed before: after an upgrade, the prior Accounts, Chirps, Follows, and Likes are still present and still satisfy their invariants. The mechanics of schema and data migration are the later code work's; the preservation property is the bar's.
- **acceptance_criteria**: On a populated deployment, perform an upgrade that changes the data's shape (a redeploy with a data-model change); afterward the Accounts, Chirps, Follows, and Likes created before the upgrade are all present and their references still resolve. Observed as an operational_check across an upgrade.
- **release_gating**: true
- **traces_to**: REQ-PROD-037

### Modules and interfaces (`REQ-DOM-200..249`)

Each requirement's `acceptance_criteria` is checkable either by inspecting the named crate in the
shared contract or by exercising the running product.

#### The module decomposition

#### REQ-DOM-200 — The product is exactly these four crates, in two layers
- **statement**: chirp is partitioned into a client-side framework layer (`ferric`, with `ferric-http` and `ferric-router`) and a product layer of three crates — `chirp-types` (shared wire contract), `chirp-server` (REST backend), and `chirp-frontend` (the single-page app) — with no fifth product crate silently carrying responsibility. Every responsibility in this design is assigned to exactly one of these parts.
- **acceptance_criteria**: Inspect the repository's `crates/` directory at `c805525`; it contains `ferric`, `ferric-http`, `ferric-router`, `chirp-types`, `chirp-server`, and `chirp-frontend`, and no product responsibility in this design is left unassigned or assigned to two parts.
- **release_gating**: true

#### REQ-DOM-201 — `ferric` is the framework, and carries no chirp domain knowledge
- **statement**: `ferric` (with `ferric-router` and `ferric-http`) provides the client-side UI framework — reactive state, a virtual DOM, client-side routing, and a browser HTTP client — and nothing in it names a chirp entity or encodes a chirp rule; chirp builds *on* `ferric`, and `ferric` could serve a different product unchanged.
- **acceptance_criteria**: Inspect `crates/ferric`, `crates/ferric-router`, and `crates/ferric-http`; none references an Account, Chirp, Follow, Like, Timeline, or Session, nor depends on `chirp-types`. The dependency direction is one-way: `chirp-frontend` depends on the `ferric` crates, never the reverse.
- **release_gating**: true

#### REQ-DOM-202 — `chirp-types` is the single source of truth for everything that crosses the wire
- **statement**: Every type that passes between the frontend and the backend — identifiers, the domain entities and their validated value objects, the request and response shapes, the pagination envelope, the Session token, and the error contract — is defined once in `chirp-types`; neither side defines a parallel shape for data that crosses the seam. `chirp-types` is pure data and validation: it performs no I/O, embeds no framework, and is not async, so both the native backend and the WebAssembly frontend compile against the identical definitions.
- **acceptance_criteria**: Inspect `crates/chirp-types`; its `ids`, `domain`, and `api` modules carry the identifiers, entities/value objects, and request/response DTOs respectively, and both `chirp-server` and `chirp-frontend` depend on it. `chirp-types` has no I/O, framework, or async dependency. No DTO that crosses the wire is defined anywhere but `chirp-types`.
- **release_gating**: true

#### REQ-DOM-203 — `chirp-server` owns the REST backend and enforces the domain invariants
- **statement**: `chirp-server` implements the REST API over the shared types and is the authority for the domain invariants the model fixes — a Chirp's author exists, Follow is not self-directed and is idempotent, a Like is at most one per (Account, Chirp) and idempotent, server-maintained counts stay consistent with their edges, and a Timeline is newest-first and bounded. It maps each failure to a machine-readable category the transport layer turns into a status; a client cannot cause the server to persist a state that violates an invariant.
- **acceptance_criteria**: Through the running product, attempt actions that would violate an invariant (follow yourself; like a chirp twice; post a chirp as a nonexistent Account); each is refused with a categorized error and no violating state is stored. Inspect `crates/chirp-server`; the `ChirpRepository` boundary documents these invariants as obligations on every implementation.
- **release_gating**: true

#### REQ-DOM-204 — `chirp-server` owns the persistence seam and the data behind it
- **statement**: All of chirp's durable state lives behind one boundary owned by `chirp-server` — the `ChirpRepository` seam — and no other part reaches around it to the store. Which store backs the seam is an engineering choice deferred to the later code work and the persistence requirement; the structural requirement is that exactly one replaceable boundary owns durable state.
- **acceptance_criteria**: Inspect `crates/chirp-server/src/repository`; a single `ChirpRepository` boundary mediates every durable operation, with the store behind it chosen by configuration. No handler or frontend code reads or writes the store except through this boundary. (Which store and its durability guarantees are the sibling persistence requirement, REQ-DOM-100..149.)
- **release_gating**: true
- **traces_to**: REQ-PROD-037

#### REQ-DOM-205 — `chirp-server` serves the single-page app and the health endpoint
- **statement**: The same `chirp-server` process that serves the API also serves the frontend's static assets and exposes the health endpoint, so chirp is one running service and not a pair that must be deployed and reconciled separately.
- **acceptance_criteria**: Start the service and, from one address, both call the API and load the single-page app; query the health endpoint at that same address. No second process or second deployable is required to serve the app.
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-DOM-206 — `chirp-frontend` presents the product and holds only session state
- **statement**: `chirp-frontend` is the single-page web application: it renders the screens and the persistent navigation, holds the signed-in Session (the authenticated Account and its bearer token) in reactive state, and issues requests and decodes responses through the shared contract. It holds no durable state — every Account, Chirp, Follow, Like, and Timeline it shows is the backend's, fetched through the API — so clearing the browser loses nothing but the local Session.
- **acceptance_criteria**: Inspect `crates/chirp-frontend`; the only state it owns across views is the current Session, and all entity data is fetched through its typed API client rather than stored locally. Reloading the app in the browser re-fetches state from the backend; no entity data persists only in the client.
- **release_gating**: true

#### REQ-DOM-207 — Domain rules are enforced on the server, not trusted from the client
- **statement**: The frontend may validate inputs for a fast, kind experience (for example the Handle and Chirp-text rules), but every domain invariant is re-checked and enforced by `chirp-server`; the product never depends on the client to uphold a rule, because a request can reach the API without the frontend.
- **acceptance_criteria**: Bypassing the frontend, submit to the API a Chirp over the length limit, a self-Follow, and a Chirp attributed to a nonexistent Account; each is refused by the server regardless of any client-side checking.
- **release_gating**: true
- **traces_to**: REQ-PROD-023

#### REQ-DOM-208 — Responsibility over each entity is assigned to exactly one part
- **statement**: For every entity in the canonical vocabulary, `chirp-types` carries its wire *shape*, `chirp-server` owns its *lifecycle and persistence* (creating it, enforcing its invariants, storing it), and `chirp-frontend` only *presents* it; no entity's lifecycle is owned by two parts, and none is owned by the frontend.
- **acceptance_criteria**: For each of Account, Chirp, Follow, Like, Timeline, and Session, trace where its shape is defined (`chirp-types`), where it is created and validated (`chirp-server`, behind `ChirpRepository`), and where it is shown (`chirp-frontend`); each role resolves to exactly one crate and the frontend owns no lifecycle.
- **release_gating**: true

#### The REST contract (the frontend↔server seam)

#### REQ-DOM-215 — There is exactly one typed client↔server contract, and `chirp-types` is it
- **statement**: The frontend and the server communicate through a single REST contract whose request and response types are the `chirp-types` definitions; the two sides build requests and decode responses from the same definitions, so they cannot drift. There is no second, hand-maintained copy of any wire shape on either side.
- **acceptance_criteria**: Inspect `crates/chirp-types/src/api.rs`; it defines the request/response DTOs, the pagination envelope (`Page` with an opaque `Cursor`), the `SessionToken`, and the error contract. Confirm `chirp-server` accepts and returns these types and `chirp-frontend` sends and decodes them, with no parallel DTO defined in either crate.
- **release_gating**: true

#### REQ-DOM-216 — The API is namespaced and same-origin with the app
- **statement**: The REST surface lives under a reserved path prefix so it never collides with the app's own client-side routes, and the frontend reaches it on the same origin that served the app, so no cross-origin configuration is part of the product's normal operation.
- **acceptance_criteria**: Load the app and observe its API calls; they target the reserved API prefix on the same origin as the page. A path that is not an API path resolves to the single-page app, not to an API handler.
- **release_gating**: true

#### REQ-DOM-217 — The contract covers the core-loop operations the app depends on
- **statement**: The REST contract offers, as a surface the frontend can rely on, the operations the core loop needs: create an Account, fetch an Account's public profile, open a Session, post a Chirp (including a Reply), read a Timeline, follow and unfollow an Account, and like and unlike a Chirp. The design fixes the *operations and their types*, not their exact URLs.
- **acceptance_criteria**: Against the running product, exercise each operation end to end from the app and confirm it is served. Inspect `chirp-types::api`; each operation has a defined request and response type. (The live surface at `c805525` already provides these; inspect the server's router to confirm each is mounted.)
- **release_gating**: true
- **traces_to**: REQ-PROD-001, REQ-PROD-002, REQ-PROD-005, REQ-PROD-006, REQ-PROD-007

#### REQ-DOM-218 — Timeline reads cross the contract as bounded pages behind an opaque cursor
- **statement**: A Timeline is returned over the contract one bounded page at a time, each page carrying an opaque continuation cursor when more Chirps remain; the server enforces a maximum page size, so a client cannot force the contract to return every Chirp in one response.
- **acceptance_criteria**: Request a Timeline over the API; a bounded page returns with a continuation cursor when more remain, and passing the cursor returns the next, older page. A requested size above the server's maximum is clamped. Inspect `chirp-types::api`; the page envelope carries the items and an optional next cursor, and the cursor is opaque.
- **release_gating**: true
- **traces_to**: REQ-PROD-002

#### REQ-DOM-219 — Authentication crosses the contract as an opaque bearer Session token; private data never does
- **statement**: A signed-in request proves itself by presenting the Session as an opaque bearer token the server issued; the token's internals are not part of the contract. Only an Account's public profile fields (Handle, display name, bio, follower/following counts) and public Chirps cross the contract outward — an Account's credential, verified contact, and Session data never appear in any response.
- **acceptance_criteria**: Sign in, obtain a token, and present it on an authenticated request; the request acts as that Account. Inspect responses for another Account's profile and any public response; only public fields appear, and no credential, contact, or Session data is ever returned. Inspect `chirp-types::api`; the Session token is a transparent string wrapper with no interpreted structure.
- **release_gating**: true
- **traces_to**: REQ-PROD-010, REQ-PROD-024

#### REQ-DOM-220 — Errors cross the contract as a typed, machine-readable category
- **statement**: A failed request returns a typed error carrying a machine-readable category (for example validation, not-found, conflict, unauthorized, forbidden), not an ad-hoc string, so the frontend can present it meaningfully and distinguish, for example, a taken Handle from a malformed one.
- **acceptance_criteria**: Provoke a validation failure, a conflict (a duplicate Handle), and an unauthorized request; each returns a response carrying a distinct, machine-readable category the frontend can branch on. Inspect `chirp-types::api`; the error contract defines a closed set of categories.
- **release_gating**: true
- **traces_to**: REQ-PROD-009, REQ-PROD-011

#### REQ-DOM-221 — The contract's wire conventions are stable and grow additively
- **statement**: The contract's serialization conventions are fixed and load-bearing — JSON field names are stable regardless of internal renames, opaque identifiers and tokens serialize as their bare value, validated value objects (the Handle, the Chirp text) reject invalid input on the way in, and error categories are machine-readable constants — and the contract's public surface grows only by addition, never by changing or removing an existing shape.
- **acceptance_criteria**: Inspect `crates/chirp-types`; field naming, the transparent identifier/token wrappers, the validate-on-deserialize value objects, and the machine-readable error codes are present as conventions. A change that renamed or removed an existing wire field (rather than adding one) would be a breaking change to this contract and is out of bounds.
- **release_gating**: true

#### REQ-DOM-222 — New product surfaces extend the one contract, not a second API
- **statement**: The release requires surfaces the live API at `c805525` does not yet carry — among them ending a Session (sign-out), editing a profile, account settings, recovery, data export, account deletion, reporting and blocking/muting, notifications, a supported operator/administrative surface, and a way to find Accounts to follow. Each is added by extending the one `chirp-types` contract and the one namespaced API, additively; the product does not grow a second, parallel API. Which of these surfaces exist, and their detailed behavior, are owned by the product, identity, experience, and operational-readiness designs and the later code work; the structural requirement here is that they attach to the single existing contract.
- **acceptance_criteria**: For each new surface the release adopts, confirm its request/response types are defined in `chirp-types` and its operations are mounted under the one namespaced API alongside the existing ones — not in a separate contract or service. (The set of surfaces and their behavior trace to the product and area designs named; this requirement checks only that they extend the one contract.)
- **release_gating**: true
- **traces_to**: REQ-PROD-008, REQ-PROD-014, REQ-PROD-016, REQ-PROD-017, REQ-PROD-019, REQ-PROD-020, REQ-PROD-028

#### The persistence seam (the server↔store boundary)

#### REQ-DOM-223 — All durable state flows through one replaceable boundary, and a durable store is a drop-in
- **statement**: Every durable operation the server performs on an Account, Chirp, Follow, Like, Session, or Timeline passes through the single `ChirpRepository` boundary; replacing the in-memory backing at `c805525` with a durable store is introducing a new implementation of that one boundary, touching no request handler and no frontend code. The capability set the boundary exposes is the whole of what the product needs from durable state, so a durable implementation has a closed, known surface to satisfy.
- **acceptance_criteria**: Inspect `crates/chirp-server/src/repository`; the `ChirpRepository` boundary enumerates exactly the durable operations the product uses, the default in-memory backing implements it, and the handlers depend on the boundary rather than on a concrete store. A durable store added as a second implementation requires no change to any handler. (The durability guarantees themselves — what survives a restart — are the sibling persistence requirement, REQ-DOM-100..149.)
- **release_gating**: true
- **traces_to**: REQ-PROD-037

#### Delivery and entry points

#### REQ-DOM-224 — The app is served as static assets with a single-page fallback, and owns its client-side routing
- **statement**: `chirp-server` serves the frontend as static assets from a configured root, and any request path that is neither an API path nor a real asset resolves to the app's entry document, so the single-page app owns its own client-side routing and a deep link to an in-app route loads the app rather than failing.
- **acceptance_criteria**: Request an in-app route directly (not the app root) with no prior navigation; the app loads and resolves that route client-side. Request an unknown asset path; the app's entry document is returned rather than a server error. The static root is configurable for deployment.
- **release_gating**: true
- **traces_to**: REQ-PROD-035

#### REQ-DOM-225 — The product has one user entry point: a single web address
- **statement**: A person reaches chirp at one web address that serves the single-page app; there is nothing to install and no second place to go. The in-app routes reached from that entry are the Timeline view, the compose-and-sign-in view, and a profile view (`c805525`), and this route set must grow to reach the release's new surfaces (sign-up, sign-in, recovery, settings, deletion, and the rest) rather than adding a separate app.
- **acceptance_criteria**: Open the single web address with no account; the app loads and offers a path onward. From the app, reach the Timeline, compose/sign-in, and a profile. Confirm there is no separate installed client. (The specific screens and their flows are the experience design's, REQ-UX-*; this requirement fixes that there is one web entry and the app owns the route set.)
- **release_gating**: true
- **traces_to**: REQ-PROD-030, REQ-PROD-040

#### REQ-DOM-226 — The product exposes a health/readiness entry point for the operator
- **statement**: The service exposes an unauthenticated health endpoint an operator can query without a Session. Today it reports process liveness only; the release requires that it report *readiness* — that the service can actually serve users, including that its durable store is reachable — so an orchestrator can tell "serving" from "merely running."
- **acceptance_criteria**: Query the health endpoint with no Session; it responds. With the durable store made unavailable, the readiness signal reports not-ready; with the service able to serve users, it reports ready. (The detailed observability and the readiness semantics are the operational-readiness design's, REQ-OPS-*; the `how` is deferred to the later code work.)
- **release_gating**: true
- **traces_to**: REQ-PROD-039

#### REQ-DOM-227 — The service's runtime entry points are configured, not hard-coded, and it ships as one artifact
- **statement**: An operator reaches the service through configuration rather than code changes: the network address it binds and the static-asset root it serves are set from the environment, and chirp ships as one self-contained artifact that serves both the API and the app and is operated as a hosted service. No part of the product requires editing source to deploy it to a new environment.
- **acceptance_criteria**: Inspect the server binary's configuration; the bind address and static root are read from the environment with sensible defaults. Deploy the one artifact to a fresh environment by setting configuration only — without editing source — and confirm it serves both the API and the app at its address.
- **release_gating**: true
- **traces_to**: REQ-PROD-040

### Traceability matrix

Every `REQ-DOM-*` that refines a product requirement cites it; the matrix is the checkable summary.
"Builds on" records the intra-`DOM` dependencies the persistence requirements rest on. All `REQ-PROD-*`
targets resolve to the product-requirements register (`REQ-PROD-001..042`).

| REQ-DOM | Section | Traces to REQ-PROD | Builds on REQ-DOM | release_gating |
|---|---|---|---|---|
| 001 | domain_model | 008 | — | true |
| 002 | domain_model | 009 | — | true |
| 003 | domain_model | 009 | — | true |
| 004 | domain_model | — | — | true |
| 005 | domain_model | — | — | true |
| 006 | domain_model | — | — | true |
| 007 | domain_model | 020 | — | true |
| 008 | domain_model | — | — | true |
| 009 | domain_model | 016 | — | true |
| 010 | domain_model | 018 | — | false |
| 011 | domain_model | 001 | — | true |
| 012 | domain_model | 001 | — | true |
| 013 | domain_model | 001 | — | true |
| 014 | domain_model | — | — | true |
| 015 | domain_model | 003 | — | true |
| 016 | domain_model | 004 | — | false |
| 017 | domain_model | 006 | — | true |
| 018 | domain_model | 006 | — | true |
| 019 | domain_model | 006 | — | true |
| 020 | domain_model | 006 | — | true |
| 021 | domain_model | — | — | true |
| 022 | domain_model | 005 | — | true |
| 023 | domain_model | 005 | — | true |
| 024 | domain_model | 005 | — | true |
| 025 | domain_model | 002 | — | true |
| 026 | domain_model | 002 | — | true |
| 027 | domain_model | 002 | — | true |
| 028 | domain_model | 002 | — | true |
| 029 | domain_model | 010 | — | true |
| 030 | domain_model | 014 | — | true |
| 031 | domain_model | 015 | — | true |
| 032 | domain_model | — | — | true |
| 033 | domain_model | 008, 010 | — | true |
| 034 | domain_model | 012 | — | true |
| 035 | domain_model | 024 | — | true |
| 036 | domain_model | — | — | true |
| 037 | domain_model | 020, 021 | — | true |
| 038 | domain_model | 019 | — | true |
| 039 | domain_model | — | — | true |
| 040 | domain_model | — | — | true |
| 100 | persistence_requirement | 037 | — | true |
| 101 | persistence_requirement | 037 | 001 | true |
| 102 | persistence_requirement | 037 | 033, 034, 035 | true |
| 103 | persistence_requirement | 037 | 011, 015 | true |
| 104 | persistence_requirement | 037 | 017 | true |
| 105 | persistence_requirement | 037 | 022 | true |
| 106 | persistence_requirement | 037 | 029, 030, 031 | true |
| 107 | persistence_requirement | — | 021, 024 | true |
| 108 | persistence_requirement | 002 | 025, 028 | true |
| 109 | persistence_requirement | 037 | — | true |
| 110 | persistence_requirement | — | 036 | true |
| 111 | persistence_requirement | — | 006, 007 | true |
| 112 | persistence_requirement | 020, 021 | 037 | true |
| 113 | persistence_requirement | 037 | — | true |
| 114 | persistence_requirement | 037 | — | true |
| 115 | persistence_requirement | 039 | — | true |
| 116 | persistence_requirement | 038 | — | true |
| 117 | persistence_requirement | 019 | 038 | true |
| 118 | persistence_requirement | 037 | — | true |
| 200 | modules_and_responsibilities | — | — | true |
| 201 | modules_and_responsibilities | — | — | true |
| 202 | modules_and_responsibilities | — | — | true |
| 203 | modules_and_responsibilities | — | — | true |
| 204 | modules_and_responsibilities | 037 | — | true |
| 205 | modules_and_responsibilities | 040 | — | true |
| 206 | modules_and_responsibilities | — | — | true |
| 207 | modules_and_responsibilities | 023 | — | true |
| 208 | modules_and_responsibilities | — | — | true |
| 215 | interfaces_and_entry_points | — | — | true |
| 216 | interfaces_and_entry_points | — | — | true |
| 217 | interfaces_and_entry_points | 001, 002, 005, 006, 007 | — | true |
| 218 | interfaces_and_entry_points | 002 | — | true |
| 219 | interfaces_and_entry_points | 010, 024 | — | true |
| 220 | interfaces_and_entry_points | 009, 011 | — | true |
| 221 | interfaces_and_entry_points | — | — | true |
| 222 | interfaces_and_entry_points | 008, 014, 016, 017, 019, 020, 028 | — | true |
| 223 | interfaces_and_entry_points | 037 | — | true |
| 224 | interfaces_and_entry_points | 035 | — | true |
| 225 | interfaces_and_entry_points | 030, 040 | — | true |
| 226 | interfaces_and_entry_points | 039 | — | true |
| 227 | interfaces_and_entry_points | 040 | — | true |

**Release-gating summary.** Of the 81 `REQ-DOM-*`, 79 are `release_gating: true`; the two non-gating
are `REQ-DOM-010` (handle-change policy — a stated-policy-and-consistency requirement, not a v1 blocker)
and `REQ-DOM-016` (reading a reply in conversation context — an enhancement over the recorded reply
linkage). The core durability gate the release bar consumes from this design is `REQ-DOM-100`,
`REQ-DOM-109`, and `REQ-DOM-113` (state survives a real restart and redeploy, acknowledged writes are
not lost, no authoritative state lives in the serving process), with `REQ-DOM-116` the backup/restore
gate and `REQ-DOM-115` the readiness-reflects-the-store gate.
