# Operational readiness

*`design/operational-readiness.md` — owner: operational-readiness concern of the `Nousix-LLC/nswe-demo`
(`chirp`) release design. Requirement scheme: `REQ-OPS-*`.*

## Overview

This document states **what must be true for `chirp` in production, beyond "it runs"** — the operational
bar a product must clear to be *operated*, not merely *started*. It is written as **requirements and
acceptance criteria, not operational implementation**: every topic with an operational "how" (which durable
store, which backup tool, which metrics backend, which alerting pipeline) is stated here as a requirement
with a testable bar, and the implementation is deferred to later code work. Each `REQ-OPS-*` carries a
statement from the operator's/user's side, acceptance criteria a stranger could check against the live
product, a `release_gating` flag, the `REQ-DOM-*`/`REQ-PROD-*` it builds on, and a `verification_method`
from the design's shared vocabulary (`manual_user_check` | `automated_test` | `inspection` |
`operational_check`).

Everything here **builds on the durable-persistence model the domain design already fixed** — it does not
redesign persistence; it designs the operations around it. The baseline it stands on, referenced by id and
never restated:

- **`REQ-DOM-012`** — a user's data endures across process restart, redeploy, and reschedule.
- **`REQ-DOM-013`** — durable state lives outside any single process's ephemeral memory/filesystem.
- **`REQ-DOM-014`** — all durable state crosses the one `ChirpRepository` seam.
- **`REQ-DOM-015` / `REQ-DOM-016`** — counts never diverge across the durability boundary; the seam is safe
  for concurrent access.
- **`REQ-DOM-021`** — the health signal can report **unhealthy when the durable store is unreachable**
  (explicitly handed from the domain design to `REQ-OPS-*`).
- **`REQ-DOM-022`** — the durable schema has a defined initial version and a forward migration path.
- **`REQ-DOM-005` / `REQ-DOM-006`** — durable credentials and durable sessions behind the seam.

**Grounding.** Observations of the product as it stands are grounded in `Nousix-LLC/nswe-demo` at commit
`c805525`, which may be exercised read-only to ground acceptance criteria. At that commit `chirp` runs on an
in-memory repository with a single replica and no persistent volume, no backups, no rate/size limits, a
liveness-only `/healthz`, and no metrics or incident posture — so most gating requirements below are
conditions today's product fails. Each is written so the same yes/no can be reached against the running
product.

## Document index — required sections, in order

This deliverable contains every required section of `operational-readiness.md` from the design's
`design_artifact_map.yaml`, in order. The five area sections are composed from the per-area work without
rewriting their requirements; the final two sections are authored at integration.

| # | Section | REQ-OPS range | Source |
|---|---------|---------------|--------|
| 1 | `durable_data` | `REQ-OPS-001..019` | durable-data area |
| 2 | `backups_and_recovery` | `REQ-OPS-020..039` | backups-&-recovery area |
| 3 | `limits_and_quotas` | `REQ-OPS-040..059` | limits-&-quotas area |
| 4 | `observability` | `REQ-OPS-060..079` | observability area |
| 5 | `status_and_incident` | `REQ-OPS-080..099` | status-&-incident area |
| 6 | `existing_followups` | — | integration (placement of #23/#24/#25) |
| 7 | `requirements_and_acceptance_criteria` | all `REQ-OPS-*` | integration (consolidated register) |

---

## durable_data

*(section: `durable_data` of `design/operational-readiness.md`)*

This section states **what durability and data retention `chirp` requires in production** — the
operational face of the durable-persistence model that domain & application design already fixed. It does
**not** redesign persistence or choose a store: *which* durable store is used, *how* it is provisioned,
the schema-migration tooling, and the backup/restore mechanism are owned elsewhere (domain design's
`REQ-DOM-*` and later code work for the store; `backups_and_recovery` for backup, restore, rollback and
migration safety; `observability` for the health signal, metrics and logs; `limits_and_quotas` for
rate/size limits and per-user quotas). This section pins the **outcome a user and an operator must be
able to observe**: that a user's data provably survives normal operation, that the product commits to a
stated retention for each kind of state, and that durability can be demonstrated against the live product.

Every requirement here **builds on** the domain persistence model and references it by id rather than
restating it. The Inherited Baseline it stands on:

- **`REQ-DOM-012`** — a user's data endures across process restart, redeploy, and reschedule.
- **`REQ-DOM-013`** — durable state lives outside any single process's ephemeral memory and filesystem.
- **`REQ-DOM-014`** — all durable state crosses the one `ChirpRepository` seam (a durable store is a
  second implementor; the in-memory store is a baseline to replace).
- **`REQ-DOM-015` / `REQ-DOM-016`** — counts never diverge across the durability boundary; the seam is
  safe for concurrent access.
- **`REQ-DOM-004`** — ids are monotonic, non-reused, and stable across restarts.
- **`REQ-DOM-005` / `REQ-DOM-006`** — durable credentials and durable sessions behind the seam.
- **`REQ-DOM-009`** — account deletion has defined cascade semantics.
- **`REQ-DOM-022`** — the durable schema carries a defined version (its migration path is
  `backups_and_recovery`'s concern; this section requires only that the version is itself durable state).

**Grounding.** Observations of the product as it stands are grounded in `Nousix-LLC/nswe-demo` at commit
`c805525`. At that commit `chirp-server` is backed by an in-memory repository and
`deploy/chirp/10-deployment.yaml` runs `replicas: 1` with a read-only root filesystem and a single
writable `emptyDir` scratch mount — **no database and no persistent volume**. The consequence, stated
from the user's side, is that every account, chirp, follow, like, credential, and session is lost on any
restart, redeploy, or reschedule. Every requirement below is a condition that today's product fails, and
each acceptance criterion is written so a stranger can reach the same yes/no against the running product.

---

### What "data endures" must mean in operation

"Durable" is not one property; in operation it is five distinct guarantees, each with its own failure
mode and its own test. A releasable `chirp` must hold all five:

1. **The medium is durable and pod-independent** — data lives on storage whose lifecycle is not tied to
   any one pod (not process memory, not the ephemeral root or scratch mount). *(REQ-OPS-001)*
2. **No loss across a process restart** — the same workload restarting keeps everything. *(REQ-OPS-002)*
3. **No loss across a redeploy** — rolling out a new revision keeps everything. *(REQ-OPS-003)*
4. **No loss across a reschedule** — a pod deleted and recreated, possibly on a different node, keeps
   everything. *(REQ-OPS-004)*
5. **Safe under more than one replica** — either replicas share one durable store (no split-brain state),
   or running more than one replica is prevented until the store supports it. *(REQ-OPS-005)*

Around those five sit the guarantees that make durability trustworthy rather than nominal: a confirmed
write is durable *before* it is confirmed (REQ-OPS-006); *every* kind of state is covered, not just the
obvious ones (REQ-OPS-007); and derived projections stay correct across the durability boundary
(REQ-OPS-008).

### Retention & lifetime commitments

Durability says data *can* survive; retention says *how long the product commits to keeping it*. The
product commits to a stated lifetime for each kind of state:

- **User content is retained for the life of the account** and is never silently expired, evicted, or
  truncated by the store (REQ-OPS-009).
- **Intentionally time-bounded durable state** — sessions, recovery factors, and any materialised export
  artefacts — has a defined, documented expiry so it does not accumulate forever (REQ-OPS-010).
- **Deletion is durable and bounded** — once a user or operator deletes data, it is purged from the live
  store within a short, defined window and is not served again (REQ-OPS-011).
- **Capacity honours the retention commitment** — the store has headroom for expected growth, and running
  out of space fails writes honestly rather than silently losing or corrupting committed data
  (REQ-OPS-012).

### Demonstrating durability on the live product

Durability is worthless if it cannot be checked. The product ships (or documents) a repeatable durability
drill that a stranger can run against the deployment to confirm, rather than assert, that nothing a user
created is lost (REQ-OPS-013). This is the `operational_check` that most `REQ-OPS-*` in this section share.

---

### Requirements & acceptance criteria

Ids are from this section's allocated range (`REQ-OPS-001 .. REQ-OPS-019`); `REQ-OPS-014 .. 019` are left
as amendment headroom. Each requirement carries a `statement`, testable `acceptance_criteria`, a
`release_gating` flag, `traces_to` the baseline it builds on, and a `verification_method` from the parent
vocabulary (`manual_user_check` | `automated_test` | `inspection` | `operational_check`).

#### REQ-OPS-001 — Durable state lives on pod-independent storage
- **Statement:** All durable domain state is held on storage whose lifecycle is independent of any single
  pod — a persistent volume or an external managed store — and never in process memory or the pod's
  ephemeral filesystem (the read-only root or the writable scratch mount).
- **Acceptance criteria:** Inspecting the running deployment shows application data bound to storage that
  is not destroyed when the pod is destroyed (not process memory, not an `emptyDir`); deleting and
  recreating the pod preserves all data. At `c805525` this fails — `chirp-server` uses an in-memory
  repository and `deploy/chirp/10-deployment.yaml` mounts only an `emptyDir` scratch — and passes only
  when a pod-independent store backs the `ChirpRepository` seam.
- **Release gating:** true
- **Traces to:** `REQ-DOM-013`, `REQ-DOM-012`, `REQ-DOM-014`, `REQ-PROD-031`.
- **Verification method:** inspection

#### REQ-OPS-002 — Data survives a process restart
- **Statement:** Restarting the service process (same workload, same node) loses no durable state.
- **Acceptance criteria:** Create an account, post ≥1 chirp, and follow ≥1 account; restart the service;
  then the account still signs in and its chirps, follows, likes, and counts are all present and correct —
  nothing created is lost.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-PROD-026`, `REQ-PROD-031`.
- **Verification method:** operational_check

#### REQ-OPS-003 — Data survives a redeploy / rolling update
- **Statement:** Rolling out a new revision of the service loses no durable state, including across the
  window in which old and new pods briefly overlap during a rolling update.
- **Acceptance criteria:** With user data present, roll the deployment to a new revision; afterwards every
  account, chirp, follow, like, count, and signed-in session is intact, and no data written during the
  rollout is lost or served inconsistently by the overlapping pods.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-DOM-016`, `REQ-PROD-026`, `REQ-PROD-031`.
- **Verification method:** operational_check

#### REQ-OPS-004 — Data survives a reschedule to another node
- **Statement:** A pod that is deleted and recreated — possibly on a different node — loses no durable
  state, because the durable store is reachable independently of where the pod runs.
- **Acceptance criteria:** With user data present, delete the pod and let it be recreated (including onto
  a different node); afterwards all user data is still present and the account still signs in. Storage
  bound to a single node or to the pod's lifecycle fails this; node-independent durable storage passes it.
- **Release gating:** true
- **Traces to:** `REQ-DOM-013`, `REQ-DOM-012`, `REQ-PROD-031`.
- **Verification method:** operational_check

#### REQ-OPS-005 — Durability holds under more than one replica, or single-writer is enforced
- **Statement:** Either all replicas share one durable store so no replica holds independent state, or the
  deployment prevents running more than one replica until the store supports shared access — so scaling can
  never silently fork user data across pods. The current `replicas: 1` is recorded as a **store** property,
  not a property of the `ChirpRepository` seam.
- **Acceptance criteria:** Documentation states whether the durable store supports more than one replica.
  If it does, running two replicas and writing through either one shows a single consistent dataset (a
  follow made on one replica is visible on the other). If it does not, the deployment constrains the
  workload to one replica by configuration, and that constraint is documented as a store limitation — not
  the undocumented, data-splitting `replicas: 1` of `c805525`.
- **Release gating:** true
- **Traces to:** `REQ-DOM-016`, `REQ-DOM-014`, `REQ-PROD-031`.
- **Verification method:** inspection

#### REQ-OPS-006 — A confirmed write is durable before it is confirmed
- **Statement:** Any action the product reports to the user as succeeded — account creation, posting a
  chirp, following/liking, editing a profile, signing in — is committed to durable storage before success
  is returned, so an immediate crash cannot lose an action the product already confirmed.
- **Acceptance criteria:** Perform an action and observe its success in the product, then immediately
  restart or kill the service before any graceful drain; after recovery the confirmed action is present
  (the chirp exists, the follow holds, the profile edit persisted). No action the UI reported as succeeded
  is absent after the crash.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-PROD-026`, `REQ-PROD-017`.
- **Verification method:** operational_check

#### REQ-OPS-007 — Every kind of domain state is durable (coverage)
- **Statement:** Durability covers **all** domain state, not only the obvious entities: accounts and
  profiles, durable credentials, sessions, chirps, follow edges, likes, the denormalised counts, the
  id/sequence counters, moderation state, and the schema version. None of these is held only in process
  memory.
- **Acceptance criteria:** For each listed kind of state, exercise it and then restart the service; after
  restart it is still present and correct — in particular a signed-in session still resolves (today it
  does not: sessions are an in-process map) and a credentialed account still authenticates (today no
  durable credential exists). An inspection confirms no listed kind of state is backed only by process
  memory.
- **Release gating:** true
- **Traces to:** `REQ-DOM-005`, `REQ-DOM-006`, `REQ-DOM-004`, `REQ-DOM-011`, `REQ-DOM-022`, `REQ-DOM-012`,
  `REQ-PROD-006`, `REQ-PROD-026`.
- **Verification method:** operational_check

#### REQ-OPS-008 — Derived state stays correct across the durability boundary
- **Statement:** The invariants domain design requires of any store (`REQ-DOM-015`, `REQ-DOM-004`) continue
  to hold **in operation** across a restart/redeploy/reschedule: denormalised counts still agree with the
  edge sets, timelines remain reverse-chronological, and newly assigned ids remain monotonic and never
  reuse a value assigned before the restart.
- **Acceptance criteria:** After a restart, displayed follower/following and like counts equal the actual
  edge counts; a fetched timeline is strictly newest-first; and entities created after the restart receive
  ids strictly greater than every id assigned before it, with cursor paging returning each chirp exactly
  once. (This is the operational face of `REQ-DOM-015`/`REQ-DOM-004`, not a restatement of them.)
- **Release gating:** true
- **Traces to:** `REQ-DOM-015`, `REQ-DOM-004`, `REQ-PROD-020`, `REQ-PROD-021`, `REQ-PROD-026`.
- **Verification method:** operational_check

#### REQ-OPS-009 — User content is retained for the life of the account
- **Statement:** User-created durable state — accounts, profiles, chirps, follows, and likes — is retained
  until the user or an authorized operator removes it; the store does not silently expire, evict, time-out,
  or truncate user content.
- **Acceptance criteria:** Content created and then left untouched for an extended period, and content left
  across multiple restarts/redeploys, is still present and unchanged; the product (or its documentation)
  states no automatic expiry or eviction of user content, and none is observed. Any configured data-lifetime
  behaviour that would remove user content is documented and is off by default.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-PROD-026`.
- **Verification method:** operational_check

#### REQ-OPS-010 — Time-bounded durable state has a defined, enforced lifetime
- **Statement:** Durable state that is intentionally transient — sessions, recovery factors/tokens, and
  any materialised data-export artefacts — carries a defined, documented retention/expiry and is cleaned
  up when it expires, so it does not accumulate indefinitely. (The session *lifetime value* and its
  security properties are owned by identity & trust, `REQ-ID-*`; this requirement is that the store
  **honours** expiry and cleanup.)
- **Acceptance criteria:** An expired or invalidated session no longer resolves and its record is not
  retained beyond its documented lifetime; a recovery token cannot be used past its documented validity; a
  materialised export artefact, if the product produces one, is removed within its documented retention
  window and is not left readable indefinitely. The retention window for each is documented.
- **Release gating:** true
- **Traces to:** `REQ-DOM-006`, `REQ-DOM-005`, `REQ-DOM-010`, `REQ-PROD-005`, `REQ-PROD-006`.
- **Verification method:** inspection

#### REQ-OPS-011 — Deletion is durable and bounded
- **Statement:** When a user deletes their account (or an operator removes content), the data is purged
  from the live durable store within a short, defined window and is not served again; removal from any
  offline/derived copies is bounded and documented too. (The cascade *semantics* are `REQ-DOM-009`; the
  *authorization* is `REQ-ID-*`; purge from **backups** is owned by `backups_and_recovery`, which this
  requirement cross-references for the backup-copy window.)
- **Acceptance criteria:** After a confirmed deletion, the account cannot be signed into, its profile
  returns not-found, and its chirps, follow/like edges, sessions, and credential are gone from the live
  store and do not reappear after a subsequent restart or redeploy; the maximum time for the live-store
  purge is documented and observed to hold, and the documentation states the bound for removal from backup
  copies.
- **Release gating:** true
- **Traces to:** `REQ-DOM-009`, `REQ-PROD-010`, `REQ-PROD-012`.
- **Verification method:** operational_check

#### REQ-OPS-012 — Capacity honours the retention commitment; exhaustion fails safe
- **Statement:** Durable storage is provisioned with headroom for expected growth so the retention
  commitments above can be met, and if the store nears exhaustion the product fails writes with an honest
  error rather than silently dropping, truncating, or corrupting committed data. (Capacity *monitoring and
  alerting* are owned by `observability`; per-user size limits and quotas are owned by `limits_and_quotas`;
  this requirement is only that capacity pressure can never destroy committed data or be hidden from the
  user.)
- **Acceptance criteria:** Already-committed data is never lost or corrupted as the store fills; a write
  that cannot be durably stored is reported to the user as failed (not falsely confirmed), and no
  previously confirmed data is affected. The provisioned capacity and the expected-growth assumption are
  documented.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-PROD-026`, `REQ-PROD-017`.
- **Verification method:** operational_check

#### REQ-OPS-013 — Durability is demonstrable by a repeatable drill
- **Statement:** The product ships, or its repository documents, a repeatable durability drill that a
  person who never met the authors can run against the deployment to confirm that no user data is lost
  across restart, redeploy, and reschedule.
- **Acceptance criteria:** Following the documented steps, a stranger creates an account, posts a chirp,
  and follows an account; triggers each of a restart, a redeploy, and a reschedule; and confirms after
  each that the account still signs in and its content and counts are intact — reaching the same pass/fail
  the authors would, using only what the repository provides.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-PROD-031`, `REQ-PROD-034`.
- **Verification method:** operational_check

---

### Traceability summary

| REQ-OPS | Durability/retention guarantee | Release gating | Traces to | Verification |
|---|---|---|---|---|
| 001 | Durable state on pod-independent storage | true | REQ-DOM-013/012/014, REQ-PROD-031 | inspection |
| 002 | Survives process restart | true | REQ-DOM-012, REQ-PROD-026/031 | operational_check |
| 003 | Survives redeploy / rolling update | true | REQ-DOM-012/016, REQ-PROD-026/031 | operational_check |
| 004 | Survives reschedule to another node | true | REQ-DOM-013/012, REQ-PROD-031 | operational_check |
| 005 | Safe under >1 replica, or single-writer enforced | true | REQ-DOM-016/014, REQ-PROD-031 | inspection |
| 006 | Confirmed write is durable before confirmed | true | REQ-DOM-012, REQ-PROD-026/017 | operational_check |
| 007 | Every kind of domain state is durable | true | REQ-DOM-005/006/004/011/022/012, REQ-PROD-006/026 | operational_check |
| 008 | Derived state correct across the boundary | true | REQ-DOM-015/004, REQ-PROD-020/021/026 | operational_check |
| 009 | User content retained for life of account | true | REQ-DOM-012, REQ-PROD-026 | operational_check |
| 010 | Time-bounded state has enforced lifetime | true | REQ-DOM-006/005/010, REQ-PROD-005/006 | inspection |
| 011 | Deletion is durable and bounded | true | REQ-DOM-009, REQ-PROD-010/012 | operational_check |
| 012 | Capacity honours retention; fails safe | true | REQ-DOM-012, REQ-PROD-026/017 | operational_check |
| 013 | Durability demonstrable by a drill | true | REQ-DOM-012, REQ-PROD-031/034 | operational_check |

### Boundaries and cross-references

- **Not designed here (named, deferred):** the concrete durable store and its provisioning → domain design
  (`REQ-DOM-013/014`) + later code work; backup, restore, rollback, and schema-migration safety →
  `backups_and_recovery` (`REQ-OPS-020..039`, building on `REQ-DOM-022`); the health signal, capacity
  monitoring, logs, and metrics → `observability` (`REQ-OPS-060..079`, building on `/healthz` and
  `REQ-DOM-021`); rate/size limits and per-user quotas → `limits_and_quotas` (`REQ-OPS-040..059`);
  authentication, session lifetime, recovery, deletion/export authorization → identity & trust
  (`REQ-ID-*`).


---

## backups_and_recovery

*Section of `design/operational-readiness.md`. Requirement ids
`REQ-OPS-020..039`. These are **requirements and acceptance criteria**, not operational implementation:
they describe the recovery outcome and how a stranger could test it on the live product, not which backup
tool, storage target, or CD mechanism delivers it — those are deferred to the implementing code works.*

### What this section covers

A product that "runs" is not a product you can operate. The question this section answers is the one every
serious operator and every first customer eventually asks: **when something goes wrong, is there a defined
way back to a working service with user data intact?** `chirp` must answer yes, and must be able to *prove*
it. Three failure stories define the scope:

1. **Data loss** — the durable store is corrupted, deleted, or lost. Recovery means restoring user data to
   a recent, known-good state.
2. **A bad deploy** — a release ships that is broken or regressed. Recovery means returning the running
   service to the previously-known-good version without losing user data.
3. **A schema change gone wrong** — a forward migration fails partway or corrupts data. Recovery means
   getting back to a consistent store at a known schema version, with the data it held.

Backup, restore, deploy rollback, and migration safety are four faces of this one concern. They are stated
here as `REQ-OPS-*` requirements, each with a testable bar.

### What this builds on (consumed, not redesigned)

This section **consumes** the durable-persistence model fixed by the domain/application design and does not
restate or alter it:

- **`REQ-DOM-012` / `REQ-DOM-013` / `REQ-DOM-014`** — all domain state is durable, lives **outside** any
  single pod's ephemeral memory and filesystem, and crosses the one `ChirpRepository` seam. This is what
  makes "back up the data" and "roll back the code without touching the data" meaningful at all.
- **`REQ-DOM-015` / `REQ-DOM-016`** — counts never diverge from edges across the durability boundary; the
  seam is safe for concurrent access. A restore must land on a store that still satisfies these.
- **`REQ-DOM-022`** — the durable schema has a **defined initial version and a forward migration path**.
  Recovery and migration safety are two faces of the same concern: a migration is a deliberate, risky
  change to the very thing backups protect, so this section pairs every migration requirement with a
  recovery requirement.
- **`REQ-DOM-021`** — the health signal can report unhealthy when the durable store is unreachable. Backup
  and restore health build on this; the probe/alert wiring itself is the `observability` area
  (`REQ-OPS-060..079`), referenced here, not duplicated.

### Honest statement of the starting point

At the measured baseline (`Nousix-LLC/nswe-demo` @ `c805525`), `chirp-server` uses an **in-memory
repository** — `deploy/chirp/10-deployment.yaml` runs a single replica with only an `emptyDir` scratch
volume and a read-only root filesystem, and no persistent volume or database. **There is therefore nothing
to back up yet, and all user data is lost on every restart** (the defect `REQ-DOM-012` exists to remove).
These requirements describe the **releasable bar once the durable store required by `REQ-DOM-012..014`
exists**; each backup/restore requirement is predicated on that store being present, and becomes testable at
the moment it lands. Until then, the only "recovery" available is redeploying the code from Git — which
restores the *service* but cannot restore *data that was never durable*. This is called out so the gap is
declared, not discovered.

Delivery is GitOps: the Argo CD `Application` at `deploy/argocd/application.yaml` tracks the `deploy/chirp`
path on the default branch with `prune`, `selfHeal`, and `allowEmpty: false`, and the workload image is
pinned by digest. That model — Git as the source of truth, image pinned by digest — is what makes a
deterministic deploy rollback possible, and the rollback requirements below are grounded in it.

---

### Requirements (`REQ-OPS-020..039`)

#### REQ-OPS-020 — The durable store is backed up on a defined, recent schedule
- **Statement:** Once durable persistence exists (`REQ-DOM-012..014`), the product's durable data is backed
  up automatically on a defined schedule, such that a current backup always exists without an operator
  having to remember to make one.
- **Acceptance criteria:** An operator can point to the most recent backup and observe that its age is
  within the stated schedule interval; no manual step was required to produce it; and the schedule and its
  interval are documented in the repository (`Nousix-LLC/nswe-demo`). The *mechanism* (which tool, which
  storage target) is out of scope here.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-DOM-013`, `REQ-PROD-031`, `REQ-PROD-032`
- **Verification method:** operational_check

#### REQ-OPS-021 — A backup is complete: it captures everything the durable model holds
- **Statement:** A backup captures the **entire** durable domain state — accounts, durable credentials,
  sessions, chirps, follows, likes, the denormalized counts, the id counters, and moderation state — plus
  the durable schema version, so that a restore reconstructs a self-consistent store and not a partial one.
- **Acceptance criteria:** Restoring a backup into a clean target yields a store in which every entity class
  listed is present, the counts agree with the edges (`REQ-DOM-015`), and the schema version is recorded;
  no entity class required by the durable model is absent from the backup.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-DOM-015`, `REQ-DOM-022`
- **Verification method:** operational_check

#### REQ-OPS-022 — A recovery point objective (maximum tolerable data loss) is defined and met
- **Statement:** The product declares how much recent data it may lose in a worst-case recovery — a recovery
  point objective (RPO) expressed as a time window — and the backup schedule is set so that objective is
  actually met.
- **Acceptance criteria:** A stated RPO exists in the repository documentation; the backup interval is no
  longer than the RPO; and after a simulated loss, the amount of data missing from the restored store is no
  greater than the RPO window. A recovery path with no stated data-loss objective does not meet this bar.
- **Release gating:** true
- **Traces to:** `REQ-PROD-031`, `REQ-PROD-032`
- **Verification method:** operational_check

#### REQ-OPS-023 — A recovery time objective (maximum tolerable time to recover) is defined and met
- **Statement:** The product declares how long a full recovery of service and data may take — a recovery
  time objective (RTO) — and the documented recovery path can actually be completed within it.
- **Acceptance criteria:** A stated RTO exists in the repository documentation; a timed restore drill
  (`REQ-OPS-024`) completes — service back up with data restored and verified — within the stated RTO.
- **Release gating:** true
- **Traces to:** `REQ-PROD-032`, `REQ-PROD-034`
- **Verification method:** operational_check

#### REQ-OPS-024 — Restore is *proven*, not merely configured (periodic restore drill)
- **Statement:** The product's restore path is exercised end-to-end on a regular cadence — a backup is
  restored into a clean, separate target and the result is verified to be a known-good state — so that "we
  have backups" is never assumed and only ever demonstrated. A backup that has never been restored is
  treated as no backup.
- **Acceptance criteria:** On a documented cadence, an operator (or an automated job) restores the latest
  backup into a target that is not the live store, brings a service up against it, and confirms a
  **known-good state**: a representative account can be signed into, and its chirps, follows, likes, and
  counts are all present and correct — i.e. the `REQ-DOM-012` restart-survival bar holds against the
  *restored* data. The date and outcome of the most recent successful drill are recorded.
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-DOM-015`, `REQ-PROD-032`
- **Verification method:** operational_check

#### REQ-OPS-025 — Backup success and freshness are monitored; a broken backup is detectable before it is needed
- **Statement:** The success, recency, and completion of backups are observable, so that a failed or stale
  backup raises a signal **before** a recovery is needed — the product never discovers at restore time that
  its backups were empty or absent.
- **Acceptance criteria:** When a scheduled backup fails or no backup has completed within the RPO window
  (`REQ-OPS-022`), that condition is observable to an operator through the product's operational signals;
  when backups are current and succeeding, that is also observable. (The signal/alert wiring itself is the
  `observability` area, `REQ-OPS-060..079`, and the store-reachability signal `REQ-DOM-021`; this
  requirement states only that backup health must be among what is visible.)
- **Release gating:** true
- **Traces to:** `REQ-DOM-021`, `REQ-PROD-032`
- **Verification method:** operational_check

#### REQ-OPS-026 — Backups are stored durably and separately from the primary store
- **Statement:** Backups are kept on a target whose failure is independent of the primary durable store, so
  that a loss which destroys the primary store does not also destroy its backups.
- **Acceptance criteria:** Inspection of the documented backup configuration shows the backup target is not
  the same volume/storage whose loss the backup is meant to protect against; a simulated loss of the primary
  store leaves at least one usable backup intact and restorable (`REQ-OPS-024`).
- **Release gating:** true
- **Traces to:** `REQ-DOM-013`, `REQ-PROD-032`
- **Verification method:** inspection

#### REQ-OPS-027 — Backups are access-controlled and their retention is defined
- **Statement:** Because a backup is a full copy of users' personal data, access to backups is restricted,
  and backups are retained for a defined period and then disposed of, rather than kept indefinitely with no
  stated policy.
- **Acceptance criteria:** A documented retention period exists; backups older than the retention period are
  removed; and access to backup storage is restricted rather than world-readable. (The user-facing privacy
  statement of what data is held, including in backups, is owned by identity & trust, `REQ-ID-*`, and by
  `REQ-PROD-029`; this requirement states the operational handling only.)
- **Release gating:** false — *a correct, monitored, restorable, separately-stored backup is the hard
  release bar; the exact retention duration and access policy are expected and must exist, but are tunable
  policy rather than a gate on first release. Recorded here so the surface is designed, not forgotten.*
- **Traces to:** `REQ-PROD-029`, `REQ-PROD-032`
- **Verification method:** inspection

#### REQ-OPS-028 — A documented recovery runbook exists
- **Statement:** There is a written recovery procedure in the repository that an operator who did not build
  the product can follow to recover a working service and restore user data after a failure, and to verify
  afterward that accounts and content are present — the operator-facing face of `REQ-PROD-032`.
- **Acceptance criteria:** The repository (`Nousix-LLC/nswe-demo`) contains documented recovery steps
  covering at least: restoring the durable store from a backup, bringing the service up against the restored
  store, and verifying a known-good state; a reader following only those steps and what the repository
  provides can complete a recovery without undocumented knowledge (`REQ-PROD-034`).
- **Release gating:** true
- **Traces to:** `REQ-PROD-032`, `REQ-PROD-034`
- **Verification method:** inspection

#### REQ-OPS-029 — A bad deploy can be rolled back to the previously-known-good running state
- **Statement:** When a release is broken or regressed, an operator can return the running service to the
  previously-known-good version through the repository's GitOps delivery path, deterministically and without
  hand-editing the cluster.
- **Acceptance criteria:** Reverting the tracked source (`deploy/chirp` on the branch the
  `deploy/argocd/application.yaml` `Application` tracks) to a previously-known-good commit causes the
  reconciler to restore that version in the cluster; because the workload image is pinned by digest, the
  rolled-back state is byte-for-byte the prior release; and `allowEmpty: false` prevents a bad revision from
  pruning the app to zero. An operator can demonstrate a rollback returns the service to health
  (`/healthz`, `REQ-DOM-021`) on the prior version. *(Hardening of the deploy/CD path this rollback relies
  on is tracked by follow-ups #23 (native k3s install path) and #24 (Argo CD least-privilege RBAC +
  supply-chain) — referenced here for relevance; their placement across the areas is left to the synthesis
  gate and not duplicated as a requirement here.)*
- **Release gating:** true
- **Traces to:** `REQ-DOM-019`, `REQ-PROD-031`, `REQ-PROD-032`
- **Verification method:** operational_check

#### REQ-OPS-030 — A deploy rollback does not destroy user data
- **Statement:** Rolling the application version back (or forward) is decoupled from the durable store:
  because durable state lives outside any single pod (`REQ-DOM-013`), changing the deployed code version
  does not drop, reset, or reinitialize user data.
- **Acceptance criteria:** After a rollback of the application version that does **not** cross a schema
  migration, all user accounts, chirps, follows, likes, and counts present before the rollback are still
  present and correct afterward; no user data is lost as a side effect of the version change. (The case
  where a rollback *does* cross a schema migration is governed by `REQ-OPS-032`.)
- **Release gating:** true
- **Traces to:** `REQ-DOM-012`, `REQ-DOM-013`, `REQ-PROD-031`
- **Verification method:** operational_check

#### REQ-OPS-031 — Forward schema migrations are applied safely
- **Statement:** A forward schema migration (`REQ-DOM-022`) is applied so that existing user data is
  preserved and the schema version advances; a fresh deploy initializes the schema from empty; and a
  migration is always preceded by a backup so a known-good pre-migration state exists.
- **Acceptance criteria:** Applying the documented migration path to a store holding data preserves that
  data and advances the recorded schema version (`REQ-DOM-022`); deploying against an empty store
  initializes the schema cleanly; and a backup satisfying `REQ-OPS-020`/`REQ-OPS-021` is captured
  immediately before the migration runs and is restorable afterward. The migration *tooling and DDL* are a
  `software`-domain how, deferred to the implementing code work.
- **Release gating:** true
- **Traces to:** `REQ-DOM-022`, `REQ-PROD-026`, `REQ-PROD-034`
- **Verification method:** operational_check

#### REQ-OPS-032 — A failed or bad schema migration has a defined recovery path
- **Statement:** If a schema migration fails partway, corrupts data, or produces a version that must be
  abandoned, there is a defined path back to a consistent store at a known-good schema version — and a
  half-migrated store is never left serving users.
- **Acceptance criteria:** Given a deliberately failed migration, an operator can follow the documented
  recovery path — restore the pre-migration backup (`REQ-OPS-031`) into a clean target and bring the
  matching service version up against it, or apply a defined roll-forward fix — and reach a consistent
  store whose schema version and data are known-good and verified (`REQ-OPS-024`); at no point is a store
  stuck between schema versions left answering user requests.
- **Release gating:** true
- **Traces to:** `REQ-DOM-022`, `REQ-PROD-032`
- **Verification method:** operational_check

#### REQ-OPS-033 — The recovery path works on the self-hostable surface
- **Statement:** A person self-hosting `chirp` from the repository can back up and restore their own
  deployment using only what the repository provides, so recovery is not an ability reserved to the original
  operator.
- **Acceptance criteria:** Following the repository's documented steps (`REQ-OPS-028`, `REQ-PROD-034`), a
  self-hoster can produce a backup of their durable store and restore it into a working deployment, and
  verify a known-good state afterward, with no hidden prerequisites beyond the documented durable-store
  configuration.
- **Release gating:** true
- **Traces to:** `REQ-PROD-034`, `REQ-PROD-032`
- **Verification method:** operational_check

---


---

## limits_and_quotas

*(section: `limits_and_quotas`)*

chirp is a **publicly reachable** product: its read endpoints (timeline, profile) are served without
authentication, and anyone on the internet can call them. A public surface with no limits is a product
one impatient script can take down for everyone. This section states, as testable `REQ-OPS-*`
requirements, the limits a releasable public chirp needs so that **no single caller can degrade the
service for everyone** and **resource use stays bounded**: request-rate limits, request/payload size
limits, pagination bounds, and the per-account quotas the product scope warrants. For each limit it
states *what* is limited, the *observable behaviour a client sees when the limit is hit*, and *how the
limit is verified*.

These are **requirements and acceptance criteria, not a limiting mechanism.** Where a limit is enforced
(in-process middleware, an ingress/edge proxy, the store itself) and the exact numeric thresholds are a
*how* deferred to the implementing code work and operational tuning; what is fixed here is the
user-/operator-observable property that must hold at the public surface, with a sane default and a
documented, configurable value. All limits build on the single-service, single-origin entry points the
domain design fixed (`REQ-DOM-019`: the JSON API under `/api/…`, the unauthenticated `GET /healthz`
probe, and the SPA fallback) and on the durable model (`REQ-DOM-012`/`REQ-DOM-013`), which is what makes
bounding resource use meaningful: a durable store *accumulates*, so unbounded creation grows real
storage, not just process memory.

**Current state (grounded in `Nousix-LLC/nswe-demo` at `c805525`).** The server mounts only a
request-tracing layer across the whole stack — there is **no rate limiting of any kind**, so a single
client may call any endpoint, including the unauthenticated reads, as fast as it can open connections.
There is **no explicitly configured maximum request body size** (only the HTTP framework's implicit
default applies, undocumented and not a deliberate product limit). **Pagination is partially bounded
already**: the home timeline clamps its page size to `1..=100` and defaults to `20` when the caller omits
`limit`, and a malformed or foreign opaque cursor fails safe to the first page — but this is the only
paginated collection today, and the account-management list endpoints the release adds
(following/followers, data export) do not yet exist and must inherit the same bound. There are **no
per-account quotas**. The requirements below close the rate-limit and size-limit gaps, generalise the
pagination bound to every collection the release introduces, and add the per-account quota the product
scope warrants.

**Boundary with identity & trust (`REQ-ID-*`).** Abuse *policy* — who may be blocked or muted, how
reports are handled, moderation and suspension of accounts, and the security properties of credential
and session handling — is identity & trust's area (`REQ-PROD-027`/`REQ-PROD-028`; `REQ-ID-*`). This
section owns the **operational limit mechanics** (rate, size, quota) stated as release requirements.
Where a limit *enforces* an abuse policy — rate-limiting credential attempts, throttling sign-up to
blunt mass account creation — the requirement references the identity & trust area that owns the policy
rather than restating it; the limit is the operational floor beneath that policy, valuable on its own as
availability protection even before the policy exists.

---

### Rate limits

#### REQ-OPS-040 — The public API enforces a per-caller request-rate limit
- **Statement:** Every client-facing HTTP endpoint under `/api/…` is subject to a request-rate limit
  applied per caller, so that one caller cannot consume the service's request capacity without bound.
  The limit applies to the **unauthenticated read endpoints** as well as the authenticated ones, because
  the public reads are the surface any anonymous caller can reach.
- **Acceptance criteria:** A single caller issuing requests to a public endpoint (for example repeated
  `GET /api/timeline` or `GET /api/users/{id}`) faster than the configured limit begins receiving a
  rejected, non-success response once the limit is exceeded, while a caller staying within the limit is
  served normally; the service remains responsive to *other* callers throughout. Exercised against the
  live product, sustained single-caller load does not deny service to a second caller.
- **Release gating:** true — a publicly reachable product with no request-rate limit can be degraded for
  everyone by one caller, which is exactly what this concern exists to prevent.
- **Traceability:** `REQ-PROD-013` (public reads), `REQ-PROD-030`/`REQ-PROD-031` (the product keeps
  serving users); builds on `REQ-DOM-019` (the `/api/…` surface). Anti-abuse policy → `REQ-ID-*`.
- **Verification method:** operational_check

#### REQ-OPS-041 — Rate limits are scoped to the caller and tightened for writes and sensitive endpoints
- **Statement:** The rate limit distinguishes callers so one caller's traffic does not exhaust another's
  allowance: **authenticated** requests are limited per account (keyed by the session identity), and
  **unauthenticated** requests are limited per request source. State-changing (write) endpoints —
  `POST /api/chirps`, `POST /api/users`, the follow/like mutations, and the account-management mutations
  the release adds — are bounded **more tightly** than reads, and the **sensitive authentication
  endpoints** (account creation `POST /api/users` and sign-in `POST /api/sessions`) carry a specifically
  tighter limit to blunt mass account creation and credential-guessing.
- **Acceptance criteria:** Two distinct callers each get their own allowance (one caller hitting its
  limit does not cause the other to be throttled); a write endpoint rejects sustained load at a lower
  request rate than a read endpoint does; and repeated rapid calls to `POST /api/sessions` or
  `POST /api/users` from one source are throttled sooner than ordinary reads. The configured per-scope
  and per-endpoint-class values are documented and inspectable.
- **Release gating:** true
- **Traceability:** `REQ-PROD-004`/`REQ-PROD-007` (credentialed sign-in and recovery — throttling guards
  the credential surface), `REQ-PROD-001`/`REQ-PROD-003` (self-service sign-up and handle uniqueness —
  throttling guards against mass account creation). Credential-attempt and account-abuse *policy* →
  `REQ-ID-*`. Builds on `REQ-DOM-019`.
- **Verification method:** automated_test

#### REQ-OPS-042 — A rate-limited request gets a well-formed, non-destructive, recoverable response
- **Statement:** When a request is rejected for exceeding a rate limit, the client sees a **distinct,
  machine-readable response** — HTTP **429 (Too Many Requests)** carrying the product's standard error
  body and an indication of when to retry — the rejected request causes **no state change**, and the
  caller is served again once the window resets. The error is rendered consistently with the wire error
  contract (`ApiError` with a machine-readable `code`), which, because the current `ErrorCode` set has
  no rate-limit category, requires an **additive** wire error category per `REQ-DOM-017` (the specific
  category name and the retry-signalling mechanism are deferred to the implementing code work).
- **Acceptance criteria:** A caller driven past the limit receives HTTP 429 with a parseable error body
  whose code identifies the condition as rate limiting (not a generic 400/500) and a retry indication
  (for example a `Retry-After` value); a *write* that is rejected for rate limiting leaves no chirp,
  follow, like, account, or session created or changed (verifiable by reading state afterward); and after
  waiting the indicated interval the same caller's request succeeds. No rejected request returns an
  ad-hoc plain-text body.
- **Release gating:** true
- **Traceability:** `REQ-DOM-017` (wire contract extended only, additively, through `chirp-types`);
  `REQ-PROD-006` (failures are surfaced honestly, not silently). Builds on the `ApiError`/`ErrorCode`
  contract.
- **Verification method:** automated_test

---

### Size limits

#### REQ-OPS-043 — Request payload size is bounded, and oversized requests are rejected early
- **Statement:** The product enforces a **documented maximum size for request bodies** on the `/api/…`
  surface; a request whose body exceeds the maximum is rejected before the handler does any work, with an
  observable error, so a client cannot force the service to buffer or process an arbitrarily large
  payload. This is the transport-level envelope; it is **distinct from and in addition to** the domain
  field validation already owned by `REQ-DOM-002` (`Username` ≤ 15 characters, `ChirpText` ≤ 280
  characters), which this requirement references and does not restate.
- **Acceptance criteria:** Sending a request body larger than the configured maximum to a `POST`
  endpoint (for example `POST /api/chirps`) is rejected with a clear, non-success response and creates
  no resource, without the service attempting to fully buffer or validate the body first; a request body
  within the maximum is processed normally. The configured maximum is documented and inspectable; a
  body within the size limit but violating a field rule still fails as a `VALIDATION_ERROR` per
  `REQ-DOM-002` (the two limits compose, neither replaces the other).
- **Release gating:** true — an unbounded accepted payload is a trivial resource-exhaustion vector on a
  public endpoint; the current implicit framework default is not a deliberate, documented product limit.
- **Traceability:** `REQ-DOM-002` (field-level content validation — referenced, not restated),
  `REQ-PROD-016` (compose within the limit), `REQ-PROD-030`/`REQ-PROD-031` (the product keeps serving).
- **Verification method:** automated_test

---

### Pagination bounds

#### REQ-OPS-044 — Every collection endpoint is bounded by a maximum page size with a default
- **Statement:** No endpoint returns an **unbounded** collection. Every list/collection response —
  today's home timeline, and the following/followers listing and data export the release adds
  (`REQ-DOM-010`, `REQ-DOM-018`) — is paginated with a **maximum page size** and a **default page size**
  used when the caller omits an explicit limit, so a single request can never be made to return the
  entire corpus.
- **Acceptance criteria:** For each collection endpoint, a request that asks for more than the maximum
  returns at most the maximum number of items and a continuation cursor for the rest; a request that
  omits a limit returns the default page size; and no request parameter causes the endpoint to return an
  unbounded number of items in one response. (Grounded: the home timeline already enforces a maximum of
  100 and a default of 20 via its clamped `limit`; the release holds every new collection endpoint to
  the same bounded-page contract.)
- **Release gating:** true — bounded responses are a release property of a public collection API; new
  list endpoints must not ship unbounded.
- **Traceability:** `REQ-PROD-018` (timeline paging), `REQ-DOM-004` (opaque monotonic cursor the timeline
  pages on), `REQ-DOM-010`/`REQ-DOM-018` (the follow-list and export collections the release adds).
- **Verification method:** automated_test

#### REQ-OPS-045 — Pagination parameters clamp rather than error, and bad cursors fail safe
- **Statement:** A page-size parameter outside the allowed range is **clamped** to the valid range rather
  than rejected as an error, and a malformed, expired, or foreign (server-un-minted) cursor **fails safe
  to the first page** rather than erroring or leaking, so routine pagination is robust against client
  quirks and cannot be used to probe the service.
- **Acceptance criteria:** Requesting a page size of 0, a negative value, or far above the maximum
  returns a valid bounded page (clamped into range) rather than an error; supplying a cursor the server
  did not mint returns the first page rather than an error or an internal failure. (Grounded: the home
  timeline already clamps `limit` into `1..=100` and treats an unparseable cursor as "from the newest";
  the release holds every new collection endpoint to the same robustness.)
- **Release gating:** false — the primary collection (the timeline) already meets this and it is a
  robustness/quality property rather than a release blocker; it is stated so every *new* collection
  endpoint inherits the same behaviour, and is a recorded expectation, not an omission.
- **Traceability:** `REQ-PROD-018`, `REQ-DOM-004`.
- **Verification method:** automated_test

---

### Per-account quotas

#### REQ-OPS-046 — Per-account quotas bound runaway content creation and storage growth
- **Statement:** Because durable state accumulates (`REQ-DOM-012`/`REQ-DOM-013`), the product can bound
  the rate at which a single account creates durable resources — chirps, follows, likes — with a
  per-account quota over a rolling window, so one account (human or automated) cannot grow the store or
  the social graph without limit. This is the sustained-use bound that complements the short-window
  request-rate limit of `REQ-OPS-040`/`REQ-OPS-041`.
- **Acceptance criteria:** An account that creates chirps (or follows) past its configured per-account
  quota within the window is prevented from creating more until the window resets, observing the same
  well-formed, non-destructive rejection as `REQ-OPS-042` (HTTP 429, no resource created); an account
  within the quota is unaffected; the configured quota values are documented and inspectable.
- **Release gating:** false — the per-caller request-rate limits (`REQ-OPS-040`/`REQ-OPS-041`) already
  bound sustained creation for a first releasable version, so a dedicated per-account content quota is a
  recorded, recommended improvement rather than a release blocker. This is an honest cut, not an
  oversight: it is called out so it is scoped deliberately and can be gated up if abuse is observed.
- **Traceability:** `REQ-DOM-012`/`REQ-DOM-013` (durable store the quota protects), `REQ-PROD-031`
  (operations keep user data/service healthy). Account-abuse *policy* → `REQ-ID-*`.
- **Verification method:** automated_test

---

### Cross-cutting

#### REQ-OPS-047 — Limits are configurable with documented defaults, and operability surfaces are exempt
- **Statement:** Every limit above has a **safe default** and is **configurable through documented
  settings** (consistent with `REQ-DOM-019`'s documented-configuration rule and `REQ-PROD-034`'s
  self-hostable surface), so an operator can tune limits for their environment without code changes. The
  **operability surfaces are exempt** from the API rate limits: the unauthenticated `GET /healthz`
  liveness/readiness probe and the static SPA assets must remain reachable so that rate limiting cannot
  make the product appear unhealthy or unservable to its own operator.
- **Acceptance criteria:** The repository documents each limit's default value and the setting that
  overrides it; changing a setting changes the enforced limit without rebuilding application logic;
  and health probing `GET /healthz` and loading the SPA shell are not throttled by the API rate limits
  even under sustained load that would rate-limit `/api/…` calls.
- **Release gating:** true — an operator who cannot keep the health probe reachable, or cannot tune
  limits for their deployment, cannot operate the product; this is the self-hostable operability bar.
- **Traceability:** `REQ-DOM-019` (documented configuration; `/healthz` and SPA entry points),
  `REQ-PROD-034` (reproducible, documented, operable), `REQ-PROD-030` (truthful health signal).
- **Verification method:** inspection

---

### Deferred "how" (named here, built elsewhere)

To keep this section to *what, not how*, the following are stated above as requirements and their
implementation is deferred:

- The **enforcement point and mechanism** of rate limiting (in-process middleware vs. ingress/edge), the
  **token-bucket/window algorithm**, and the **exact numeric thresholds** → the implementing code work and
  operational tuning, realising `REQ-OPS-040`–`REQ-OPS-042`, `REQ-OPS-046`.
- The **additive wire error category** for the 429 response and its retry-signalling → the implementing
  code work, additively through `chirp-types` per `REQ-DOM-017` (`REQ-OPS-042`).
- The **abuse/credential/account policy** a limit enforces (who is blocked, credential-attempt handling,
  moderation) → identity & trust (`REQ-ID-*`), with the limits here as the operational floor beneath it.

---

### Register summary — `REQ-OPS-040..059` (limits & quotas)

| REQ-OPS | Concern | Release gating | Traces to | Verification |
|---|---|---|---|---|
| 040 | Per-caller request-rate limit on the public API | true | REQ-PROD-013/030/031, REQ-DOM-019, REQ-ID-* | operational_check |
| 041 | Rate limits scoped (per-account/per-source); writes + auth endpoints tighter | true | REQ-PROD-001/003/004/007, REQ-DOM-019, REQ-ID-* | automated_test |
| 042 | 429 breach response: well-formed, non-destructive, recoverable | true | REQ-DOM-017, REQ-PROD-006 | automated_test |
| 043 | Bounded, documented request payload size; rejected early | true | REQ-DOM-002, REQ-PROD-016/030/031 | automated_test |
| 044 | Every collection bounded by a max page size + default | true | REQ-PROD-018, REQ-DOM-004/010/018 | automated_test |
| 045 | Pagination params clamp; bad cursors fail safe | false | REQ-PROD-018, REQ-DOM-004 | automated_test |
| 046 | Per-account quota bounds runaway creation / storage growth | false | REQ-DOM-012/013, REQ-PROD-031, REQ-ID-* | automated_test |
| 047 | Limits configurable with documented defaults; `/healthz` + SPA exempt | true | REQ-DOM-019, REQ-PROD-034/030 | inspection |

`REQ-OPS-048..059` are unused, reserved as amendment headroom within this area's allocated range.


---

## observability

### What this section is, and is not

This is the `observability` section of `design/operational-readiness.md`. It defines **what must be
visible to operate chirp** — the signals an operator must be able to observe to know the product is
working, to tell *why* it is not when it breaks, and to see it degrade before users do — stated as
testable `REQ-OPS-*` requirements of the **product**, not as a choice of monitoring stack.

Three things must be observable: a **health signal** that is honest about whether the product can
serve users; **logs** that let an operator reconstruct what happened and why; and **metrics** that
make traffic, errors, latency, and saturation measurable over time. Each requirement states the
signal that must exist and what it must *truthfully reflect* — never which backend produces it. Which
database health is read from, which log collector ships stdout, and which time-series system scrapes
the metrics are a *how* owned by later code work and the implementing deployment, not by this design.

**The defect this section exists to prevent** is a signal that lies: a health check that is green
while the product cannot serve users, a 5xx that leaves no trace, a slowdown nobody could see coming.
The guiding principle throughout is **truthfulness** — a signal is worth having only if an operator
can trust it.

### Where this builds on what already exists

Grounded in `Nousix-LLC/nswe-demo` @ `c805525` (read-only):

- The service exposes `GET /healthz` (`crates/chirp-server/src/api/router.rs`), unauthenticated,
  returning `200` with a small JSON body (`status`, `service`, `version`). **It touches no
  application state**, so by construction it reports *process liveness only* — it stays green when
  the product cannot actually serve users. The deployment currently points **all three** Kubernetes
  probes (startup, liveness, readiness) at this same `/healthz` (`deploy/chirp/10-deployment.yaml`),
  so "ready for traffic" and "process is up" are not distinguishable today.
- Logging uses `tracing` + `tracing-subscriber` with a `RUST_LOG` env-filter and a `tower-http`
  `TraceLayer` wrapping the whole HTTP stack, plus lifecycle logs on start/stop
  (`crates/chirp-server/src/main.rs`). So request-level and lifecycle logging exist, but there is no
  per-request correlation id and no guaranteed machine-parseable format or field set.
- There are **no operational metrics**: no request/error/latency counters or exposition endpoint
  exist in the server today.

These requirements build on the `/healthz` surface and especially on **`REQ-DOM-021`** — the domain
design fixed that the model must expose a durable-store reachability check and *explicitly handed the
probe wiring and the readiness/liveness policy to `REQ-OPS-*`*. This section discharges that hand-off.

### The seam with status & incident (kept clean)

Observability **defines the signals and what they must reflect**. The sibling `status_and_incident`
section **defines how people are told** — alert thresholds, on-call expectations, the status posture,
and user-facing degradation messaging. The boundary: this section names the signals and the product
SLIs (`REQ-OPS-074`) and requires they be truthful and collectable; it does **not** set alerting
thresholds, page anyone, or specify the status page. Those consume the signals defined here.

---

### observability requirements

#### REQ-OPS-060 — Health reports unhealthy when the product cannot serve users
- **Statement:** The product's health signal MUST be able to report **unhealthy when the durable
  store is unreachable**, so that "the process is up" cannot be mistaken for "the product works for
  users". The signal reflects the product's actual ability to serve, not merely process liveness.
- **Acceptance criteria:** With the product running and able to serve users, the health signal
  reports healthy. When the durable store is made unreachable while the process stays alive, the
  health signal reports unhealthy (not `200`/healthy). A reader who never met the author can perform
  this check against the live product and get the same result.
- **release_gating:** true
- **traces_to:** `REQ-DOM-021`, `REQ-PROD-030`
- **verification_method:** operational_check

#### REQ-OPS-061 — Liveness and readiness are distinct signals with distinct meanings
- **Statement:** The product MUST expose a **liveness** signal (is the process able to make progress,
  such that restarting it would help) and a **readiness** signal (is the product able to serve user
  requests right now, including its dependence on durable data) as **separately observable** signals,
  so that a dependency outage takes the instance out of rotation (readiness) without triggering a
  restart loop (liveness).
- **Acceptance criteria:** Two signals are observable and documented. When the durable store is
  unreachable, the readiness signal reports not-ready while the liveness signal still reports alive
  (so an orchestrator stops routing traffic but does not kill and restart the process for a
  dependency it cannot fix by restarting). Inspection of the deployment shows the readiness decision
  is driven by the readiness signal, not by a liveness-only probe.
- **release_gating:** true
- **traces_to:** `REQ-DOM-021`, `REQ-PROD-030`
- **verification_method:** operational_check

#### REQ-OPS-062 — The readiness check actually exercises its dependency, bounded, and fails closed
- **Statement:** The readiness signal MUST **actually verify the durable-store dependency** (not
  assume it), complete within a bounded time, and **fail closed** — report unhealthy on timeout or
  error rather than hanging or defaulting to healthy.
- **Acceptance criteria:** When the store is reachable, readiness returns healthy within a small,
  documented bound. When the store is slow past that bound or returns an error, readiness reports
  unhealthy within the bound rather than blocking indefinitely or reporting healthy. The check does
  not itself mutate product state.
- **release_gating:** true
- **traces_to:** `REQ-DOM-021`, `REQ-DOM-014`
- **verification_method:** operational_check

#### REQ-OPS-063 — Health recovers on its own when the dependency returns
- **Statement:** When a dependency that made the product unhealthy is restored, the health signal
  MUST return to healthy **without requiring a restart or manual intervention**.
- **Acceptance criteria:** Starting from an unhealthy readiness caused by an unreachable store,
  restoring store reachability causes the readiness signal to report healthy again on its next
  evaluation, with no process restart and no operator action beyond fixing the dependency.
- **release_gating:** true
- **traces_to:** `REQ-DOM-021`, `REQ-PROD-031`
- **verification_method:** operational_check

#### REQ-OPS-064 — The health endpoint is cheap, safe, and self-identifying
- **Statement:** The health signal MUST be **unauthenticated, side-effect-free, and inexpensive
  enough to poll frequently**, and MUST return a machine-readable body identifying the service and
  the running version, so an operator (and an automated probe) can consume it and know *which build*
  is answering.
- **Acceptance criteria:** The health endpoint is reachable without credentials, returns a
  machine-readable body naming the service and version, does not alter product state when called, and
  can be polled on the deployment's probe cadence without measurable user-facing cost. (Preserves the
  existing `status`/`service`/`version` body at `/healthz`.)
- **release_gating:** false
- **traces_to:** `REQ-PROD-034`
- **verification_method:** inspection

#### REQ-OPS-065 — Every request produces a log record an operator can read traffic from
- **Statement:** Each handled HTTP request MUST produce a log record carrying at least the **method,
  the route (not raw user content), the response status, and the latency**, so an operator can see
  traffic volume, error rate, and slow requests from logs alone.
- **Acceptance criteria:** After exercising the live product with a mix of successful and failing
  requests, the logs contain one record per request with method, route, status, and a latency
  measurement; an operator can distinguish successful from failing requests and identify slow ones
  without access to anything but the logs.
- **release_gating:** true
- **traces_to:** `REQ-PROD-030`, `REQ-PROD-034`
- **verification_method:** operational_check

#### REQ-OPS-066 — Logs are structured, parseable, and emitted for platform collection
- **Statement:** Logs MUST be emitted as **structured, machine-parseable records** (one event per
  line, with documented fields) to the process's standard output/error streams, so the surrounding
  platform can collect, search, and filter them without bespoke parsing. Log verbosity MUST be
  configurable at deploy time without a rebuild.
- **Acceptance criteria:** Log output is parseable by field (not only free-form prose), each line is
  a single self-contained event, and the field set is documented. Logs go to stdout/stderr so a
  collector can capture them. Changing the configured log level (e.g. via the documented `RUST_LOG`
  setting) changes verbosity on the next start without recompiling.
- **release_gating:** true
- **traces_to:** `REQ-PROD-034`
- **verification_method:** inspection

#### REQ-OPS-067 — Server faults are logged with enough context to diagnose, never silently
- **Statement:** Every server-side fault (a `5xx`, an unhandled error, a failed dependency call) MUST
  produce a log record with enough context to tell **what failed and why** — distinct from an
  ordinary request line — so that no server error is silent.
- **Acceptance criteria:** Inducing a server-side failure (for example, a store error) produces a log
  record that identifies the failing operation and the error cause, correlatable to the request that
  triggered it; there is no case where the product returns a `5xx` or drops an operation with no
  corresponding diagnostic log entry.
- **release_gating:** true
- **traces_to:** `REQ-PROD-030`
- **verification_method:** operational_check

#### REQ-OPS-068 — A request can be followed across log lines by a correlation id
- **Statement:** Each request's log records MUST carry a **correlation/request identifier** so an
  operator can follow a single request across all the lines it produced (and, where a client reports
  an error, tie the client's report to the server-side trace).
- **Acceptance criteria:** For a single request that emits multiple log lines, all those lines share
  one identifier that is absent from unrelated requests; given that identifier an operator can
  retrieve exactly the lines for that one request. (Closes the current gap: no per-request id exists
  today.)
- **release_gating:** true
- **traces_to:** `REQ-PROD-030`
- **verification_method:** inspection

#### REQ-OPS-069 — Logs never expose secrets or personal data
- **Statement:** Logs MUST NOT contain **secrets or sensitive personal data** — no passwords,
  authentication credentials, session or bearer tokens, or request/response bodies carrying personal
  content. What is logged is metadata about the request, not its sensitive contents.
- **Acceptance criteria:** Inspection of logs produced across authentication, posting, and account
  operations shows no credential, token, or password values and no personal message bodies; where an
  identifier is needed it is a non-sensitive reference, not a secret. A reviewer can confirm this
  from the logs and the documented field set.
- **release_gating:** true
- **traces_to:** `REQ-DOM-005`, `REQ-DOM-006`, `REQ-PROD-029`
- **verification_method:** inspection

#### REQ-OPS-070 — Lifecycle and fatal-startup events are observable in logs
- **Statement:** The product MUST log its **lifecycle events** — startup (with an effective,
  secret-free configuration summary), receipt of shutdown signals and graceful drain, and clean
  stop — and MUST log a clear, specific record when it **fails to start**, so an operator can tell
  why an instance will not come up.
- **Acceptance criteria:** A normal start/stop produces log records marking listening-start, signal
  receipt, drain, and stop. A deliberately broken configuration (for example, an unreachable store or
  a bad bind setting) produces a specific startup-failure log that names the cause, rather than an
  instance that exits or crash-loops with no explanation. (Builds on the existing start/stop
  lifecycle logs.)
- **release_gating:** true
- **traces_to:** `REQ-PROD-034`, `REQ-DOM-019`
- **verification_method:** operational_check

#### REQ-OPS-071 — Quantitative request metrics are exposed for collection
- **Statement:** The product MUST expose **quantitative operational metrics** for collection — at
  least **request rate, error rate, and a latency distribution** broken down by route-class — so an
  operator can measure how the product behaves over time rather than inferring it by reading logs.
- **Acceptance criteria:** A collector can retrieve, from the running product, counters/measurements
  sufficient to compute request rate, the proportion of error responses, and a latency distribution
  (not just an average) per route-class. After driving known traffic, the retrieved values reflect
  that traffic. (Closes the current gap: no metrics exist today.)
- **release_gating:** true
- **traces_to:** `REQ-PROD-030`, `REQ-PROD-034`
- **verification_method:** operational_check

#### REQ-OPS-072 — Degradation is visible before users feel it
- **Statement:** The product MUST expose a **saturation / degradation signal** — at least in-flight
  request load and the **health and latency of the durable store as seen by the application** — so an
  operator can see the product heading toward trouble (a slowing or struggling dependency, rising
  concurrency) *before* it turns into user-visible failures.
- **Acceptance criteria:** With the store artificially slowed but not yet failing, the degradation
  signal moves observably (store latency as seen by the app rises, and/or in-flight load climbs)
  while user requests are still largely succeeding — demonstrating early warning rather than
  after-the-fact detection. The signal is retrievable by the same collection path as `REQ-OPS-071`.
- **release_gating:** true
- **traces_to:** `REQ-DOM-021`, `REQ-DOM-016`, `REQ-PROD-030`
- **verification_method:** operational_check

#### REQ-OPS-073 — Metrics exposition is separable from the user surface and safe to scrape
- **Statement:** The metrics exposition MUST be **separable from the public user-facing surface**
  (served off the user path and/or access-restricted) and cheap enough to scrape on a normal cadence
  without degrading the product; it MUST NOT leak secrets or personal data.
- **Acceptance criteria:** Inspection shows the metrics surface is not part of the normal user
  request path and is either network-restricted or otherwise protected; scraping it on the intended
  cadence has no measurable user-facing cost; the exposed metrics contain no credentials, tokens, or
  personal content.
- **release_gating:** false
- **traces_to:** `REQ-PROD-029`, `REQ-PROD-034`
- **verification_method:** inspection

#### REQ-OPS-074 — The product's health is defined as named, quantitative SLIs
- **Statement:** "Working" MUST be defined **quantitatively**, as a small set of named service-level
  indicators the signals above make measurable — at least **successful-request ratio**, a **latency
  indicator (e.g. p95) for the core read/write paths**, and **durable-store availability** — so that
  the release bar and the `status_and_incident` posture can set thresholds and alerts against a shared
  definition rather than inventing their own. This section **defines and names** the SLIs; it does not
  set their thresholds or alerting (that is `status_and_incident` and the release bar).
- **Acceptance criteria:** A named SLI set exists, each SLI states precisely what it measures and from
  which signal (`REQ-OPS-060..072`) it is derived, and each is computable from the exposed
  signals/metrics against the live product. A stranger can compute each SLI and get a defensible
  number without further definition work.
- **release_gating:** true
- **traces_to:** `REQ-PROD-030`, `REQ-DOM-021`
- **verification_method:** operational_check

#### REQ-OPS-075 — Observability is documented for whoever operates the product
- **Statement:** The product's observability MUST be **documented for an operator**: what signals
  exist (health, logs, metrics, SLIs), where to find each, and what each one means — consistent with
  the self-hostable/operable surface, so someone who did not build chirp can operate it from what the
  repository provides.
- **Acceptance criteria:** Following only the repository's documentation, a person who did not build
  chirp can locate the health signal, find and interpret the structured logs and the request
  correlation id, retrieve the metrics, and identify the named SLIs and what each reflects — without
  reading the source to discover that these signals exist.
- **release_gating:** true
- **traces_to:** `REQ-PROD-034`
- **verification_method:** manual_user_check

---

### Scope notes and deferrals (requirements only — the "how" is a later code work)

- **Which backends** produce these signals — the metrics/time-series system, the log collector, the
  tracing system, the exact probe wiring in the deployment manifests — are deliberately **out of
  scope** here and deferred to the implementing code work; this section fixes only *what must be
  observable and what it must truthfully reflect*.
- **Alert thresholds, on-call, the status page, and user-facing degradation messaging** are out of
  scope here by the clean seam with `status_and_incident`; this section hands them the truthful
  signals and the named SLIs (`REQ-OPS-074`) to act on.
- **Distributed tracing across multiple services** is not required at this product's single-service
  scope (`REQ-DOM-019`); the per-request correlation id (`REQ-OPS-068`) is the proportionate form of
  request-level traceability for one service.


---

## status_and_incident

*(section: `status_and_incident` — owns `REQ-OPS-080..099`)*

This section designs **how operators and users learn when chirp is in trouble, and the minimum a
releasable product commits to when it is.** It is deliberately a *status and incident posture*, not an
SRE programme: the bar is that a degradation or outage is **noticed by a machine, reaches a responsible
human, is shown honestly to the people using the product, and is handled by someone who owns it** — a
credible first-release commitment, with the heavier discipline (noise-tuned alerting, status channels,
post-incident review) named and tracked rather than pretended.

It states **required outcomes, not tools**: it names no on-call vendor, no hosted status-page service, no
alerting pipeline. Those are an operational "how" deferred to the implementing code/infra work; this
section fixes *what must be true* so a stranger could check it against the live product.

### What this section builds on, and the seam it does not cross

- **Detection rides on the observability signals, which this section does not define.** The
  observability area (its own `REQ-OPS-060..079` register) owns the product-health signal, logs, and
  metrics — including a health check that can report **unhealthy when the durable store is unreachable**
  (`REQ-DOM-021`, the domain design's hand-off to operational readiness). This section begins *after* a
  signal exists and designs **how a signal becomes a human being told, and a user being informed** — the
  monitoring condition, the notification, the user-facing failure, and the incident handling. Where a
  requirement here needs a health/metric signal, it references the observability area by role, not by
  redefining it.
- **The user-facing side meets experience design.** Experience design (`REQ-UX-*`) owns the error,
  empty, and degraded screen states and their copy; identity & trust (`REQ-ID-*`) owns session
  behaviour. This section fixes the **operational requirement** that, during a real degradation or
  outage, those states are *reached honestly* — the product fails openly instead of appearing to work —
  and leaves the pixels and wording to experience design.

### The product as it stands (grounded at `Nousix-LLC/nswe-demo` @ `c805525`)

Two facts set the shape of the gap, both observable today:

1. **"Healthy" today means only "the process answers."** The liveness, readiness, and startup probes in
   `deploy/chirp/10-deployment.yaml` all target `GET /healthz`, and that endpoint
   (`crates/chirp-server/src/api/router.rs`) "touches no application state … reports process liveness
   only." So the product can be failing for users while every probe still reports healthy — there is no
   condition on which an incident could be *detected*, and nothing that would *tell anyone*.
2. **A restart is a silent, total user-facing outage of state.** The service runs a single replica of an
   in-memory store (`replicas: 1`, read-only root filesystem, `emptyDir` scratch — deliberate, per the
   deployment's own comment), so every restart, redeploy, or reschedule discards all accounts, chirps,
   follows, and sessions and silently signs everyone out (`REQ-DOM-012/013`, `REQ-PROD-026/031`,
   `REQ-PROD-006`). Today that catastrophic event produces **no operator alert, no user-facing notice,
   and no record**. The durable-persistence work removes the data loss; this section ensures that *if*
   the product is ever degraded or down, people are not left to discover it in silence.

There is, today, **no status surface, no alerting, no incident ownership, and no incident record** of any
kind. The requirements below define the minimum that changes.

### The posture, in three parts

**1 — Detection → a human is told.** A monitored condition over the product-health signal defines "the
product is in trouble," and crossing it notifies a responsible human within a bounded time — not merely
lighting up a dashboard nobody is watching.

**2 — Honest user-facing failure.** When chirp cannot serve users, the people using it see an honest,
recoverable "something is wrong on our side" state and are never misled into thinking a failed action
succeeded; and they have a way to learn whether a current problem is already known.

**3 — A minimal incident commitment.** Someone owns the response, incidents are recorded, and users are
kept informed of significant impact — proportionate to a first release, with recurrence-prevention
tracked.

---

### Requirements & acceptance criteria (`REQ-OPS-080..099`)

Each requirement states what must be true from the operator's or user's side; each acceptance criterion
is written so a stranger could check it against the live product and reach the same yes/no. `traces_to`
records the baseline or product requirement it builds on; `verification_method` is drawn from the parent
schema vocabulary (`manual_user_check`, `automated_test`, `inspection`, `operational_check`).

#### Detection and operator notification

#### REQ-OPS-080 — Product-trouble is detected on a product-health condition, not on process liveness
- **Statement:** A monitored condition exists that distinguishes "the product can serve users" from "the
  process is up," and defines when chirp is **in trouble** (degraded or down). The condition is evaluated
  against the observability area's product-health signal — the one that can report unhealthy when the
  durable store is unreachable (`REQ-DOM-021`) — not against a state-less liveness probe. "The pod is
  Ready" MUST NOT be sufficient to conclude the product is healthy.
- **Acceptance criteria:** With the product running normally, the monitored product-health condition
  reads healthy. When the product is made unable to serve users (for example, its durable store is made
  unreachable) while the process stays alive, the monitored condition reads **in trouble** within a
  defined, documented evaluation window — i.e. a reader can point at a signal that went bad, not only at a
  probe that stayed green. Documentation names the condition and its threshold.
- **Release gating:** true
- **Traceability:** `REQ-DOM-021`, `REQ-PROD-030`; consumes the observability health signal
  (`REQ-OPS-060..079`).
- **Verification method:** operational_check

#### REQ-OPS-081 — When the product is in trouble, a responsible human is notified within a bounded time
- **Statement:** When the detection condition (`REQ-OPS-080`) holds for a defined duration, a responsible
  human is **actively notified** through a defined channel, rather than the problem being visible only to
  whoever happens to look. The required outcome is "someone is told, within a stated time"; the specific
  notification tool is out of scope.
- **Acceptance criteria:** Inducing a sustained in-trouble condition causes a notification to reach the
  documented responsible party within the documented time bound; clearing the condition is likewise
  observable to that party. The notification identifies *which* product and *what* is wrong well enough to
  begin a response. The channel and time bound are documented in the repository.
- **Release gating:** true
- **Traceability:** `REQ-PROD-030`, `REQ-PROD-033`; builds on `REQ-OPS-080`.
- **Verification method:** operational_check

#### REQ-OPS-082 — Alerts are actionable and tuned not to page a healthy product
- **Statement:** Each alert names what is wrong, where, and where to look next (a pointer to the relevant
  signal or a short runbook), and the alerting threshold is set so that a healthy, normally-operating
  product does not routinely generate alerts.
- **Acceptance criteria:** A sample alert, inspected, states the affected product, the failing condition,
  and a next step or runbook reference; over a normal operating period with no real degradation, the
  product does not emit spurious in-trouble alerts (no routine paging on healthy operation).
- **Release gating:** false — a first release can ship with the plain detection-and-notification of
  `REQ-OPS-080/081`; actionability and noise-tuning are a real improvement, not a release blocker.
- **Traceability:** `REQ-PROD-030`; refines `REQ-OPS-081`.
- **Verification method:** inspection

#### User-facing degradation and status

#### REQ-OPS-083 — When chirp cannot serve users, users see an honest degraded/outage state, never a broken page
- **Statement:** When the backend is unavailable or failing (for example, the API is down or the durable
  store is unreachable), the product presents an honest, human, recoverable "something is wrong on our
  side — please try again" state that is distinct from ordinary user-input/validation errors, rather than
  a blank screen, a raw error, an infinite spinner, or a silent nothing.
- **Acceptance criteria:** With the backend made unavailable, a visitor or signed-in user loading chirp
  sees a clear system-side problem message (visibly different from a "that handle is taken"-style
  validation message) and an obvious way to retry; they do not see a blank/broken page or an unexplained
  spinner that never resolves. When the backend recovers, retrying returns the user to normal use. (The
  exact screen and copy are experience design's, `REQ-UX-*`.)
- **Release gating:** true
- **Traceability:** `REQ-PROD-014`, `REQ-PROD-018`, `REQ-PROD-019`, `REQ-PROD-030`; meets experience
  design (`REQ-UX-*`).
- **Verification method:** manual_user_check

#### REQ-OPS-084 — During a degradation the product fails honestly and never loses the user's work silently
- **Statement:** While chirp is degraded or down, no user action may *appear* to succeed when it did not,
  and no degraded read may present stale or empty data as if it were current and complete. A write that
  cannot be durably applied (post, follow/unfollow, like/unlike, profile edit) is reported as failed and
  the user's input is preserved so it can be retried — the product does not accept work it is about to
  drop.
- **Acceptance criteria:** With writes made to fail (backend degraded), composing a chirp and attempting
  to post it surfaces an honest failure and leaves the typed text intact for retry (never a false
  "posted"); a follow/like attempted during the degradation either visibly succeeds or visibly fails, and
  the state shown after recovery matches what actually happened — no action silently vanishes and none is
  shown as done when it was not.
- **Release gating:** true
- **Traceability:** `REQ-PROD-006`, `REQ-PROD-017`, `REQ-PROD-020`, `REQ-PROD-021`, `REQ-PROD-031`.
- **Verification method:** manual_user_check

#### REQ-OPS-085 — Users have a way to learn whether a current problem is known
- **Statement:** When chirp is having a known problem, a user can tell that it is known and being handled,
  rather than being left to wonder whether the fault is theirs. For a first release the minimum is an
  honest in-product degraded indication during an incident (satisfied together with `REQ-OPS-083`); a
  separate, independently-hosted status channel is **not** required at the first-release bar.
- **Acceptance criteria:** During an induced incident, a user can observe from the product that there is
  a current, known problem (for example, the degraded-state indication of `REQ-OPS-083`) rather than only
  a generic per-action error; documentation records where users are expected to learn status. If a
  distinct status channel is later provided, it reflects a real incident when one is active and "all
  clear" otherwise.
- **Release gating:** false — the honest in-app degraded state (`REQ-OPS-083`) is the gating minimum; a
  dedicated status channel is a near-term addition, recorded not required.
- **Traceability:** `REQ-PROD-030`; builds on `REQ-OPS-083`.
- **Verification method:** manual_user_check

#### Minimal incident handling

#### REQ-OPS-086 — Incident-response responsibility is defined and documented
- **Statement:** The releasable product names who is responsible for responding when chirp is in trouble,
  so an incident has an owner instead of falling to whoever happens to notice. The responsibility is
  documented in the repository; the specific rota/tooling is out of scope.
- **Acceptance criteria:** The repository documents who is responsible for incident response for chirp and
  how they are reached (by role or named point of contact), such that a stranger reading the docs can say
  who would handle an outage. The notification channel of `REQ-OPS-081` reaches this responsible party.
- **Release gating:** true
- **Traceability:** `REQ-PROD-033`, `REQ-PROD-034`; pairs with `REQ-OPS-081`.
- **Verification method:** inspection

#### REQ-OPS-087 — Incidents are recorded durably
- **Statement:** When an incident occurs, it is recorded in a durable place — at minimum what happened,
  when it was detected, the user-facing impact, and when it was resolved — so there is a history and a
  basis for preventing recurrence. A lightweight record (for example, a tracked issue in the repository's
  board) satisfies this; a dedicated incident-management system is not required.
- **Acceptance criteria:** A past (or drill) incident has a durable record containing detection time,
  impact summary, and resolution; the record is reachable by the responsible party from the repository or
  its board. The format is documented so the next incident is recorded the same way.
- **Release gating:** true
- **Traceability:** `REQ-PROD-032`, `REQ-PROD-033`.
- **Verification method:** inspection

#### REQ-OPS-088 — Users are kept informed of a significant incident's impact and resolution
- **Statement:** For an incident with meaningful user impact — notably one where user data was lost or the
  product was unavailable for an extended period — users are informed of the impact and of its resolution
  through a channel the product controls, rather than left to infer it.
- **Acceptance criteria:** For a defined "significant incident" (impact threshold documented), there is a
  recorded instance of users being informed of the impact and the resolution; the threshold for informing
  users and the channel used are documented.
- **Release gating:** false — a near-term commitment above the first-release gating minimum
  (`REQ-OPS-083/085`); recorded, not required for the first release.
- **Traceability:** `REQ-PROD-030`; extends `REQ-OPS-085`.
- **Verification method:** manual_user_check

#### REQ-OPS-089 — Each incident yields a recurrence-prevention follow-up
- **Statement:** After an incident, its cause and at least one concrete follow-up action to reduce the
  chance of recurrence are captured and routed to the delivery backlog, so repeated failures get fixed
  rather than re-lived.
- **Acceptance criteria:** An incident record (`REQ-OPS-087`) carries, or links to, a cause note and at
  least one follow-up action filed where the backlog lives, with enough detail that someone could act on
  it; a stranger can trace from the incident to the follow-up.
- **Release gating:** false — a maturity step beyond the first-release bar; tracked for the operability
  backlog.
- **Traceability:** `REQ-PROD-032`; builds on `REQ-OPS-087`.
- **Verification method:** inspection

---

### Section register summary

| REQ-OPS | Concern | Release gating | Verification | Traces to |
|---|---|---|---|---|
| 080 | Detect product-trouble on a product-health condition (not liveness) | true | operational_check | REQ-DOM-021, REQ-PROD-030 |
| 081 | Notify a responsible human within a bounded time | true | operational_check | REQ-PROD-030, REQ-PROD-033 |
| 082 | Alerts actionable; not noisy on a healthy product | false | inspection | REQ-PROD-030 |
| 083 | Honest user-facing degraded/outage state, never broken | true | manual_user_check | REQ-PROD-014, 018, 019, 030 |
| 084 | Fail honestly during degradation; never lose work silently | true | manual_user_check | REQ-PROD-006, 017, 020, 021, 031 |
| 085 | Users can learn a current problem is known | false | manual_user_check | REQ-PROD-030 |
| 086 | Incident-response responsibility defined & documented | true | inspection | REQ-PROD-033, 034 |
| 087 | Incidents recorded durably | true | inspection | REQ-PROD-032, 033 |
| 088 | Users informed of significant-incident impact/resolution | false | manual_user_check | REQ-PROD-030 |
| 089 | Recurrence-prevention follow-up per incident | false | inspection | REQ-PROD-032 |

**Gating summary:** five release-gating requirements form the first-release commitment — a product in
trouble is **detected** on a real health condition (`080`), a **human is told** (`081`), users see an
**honest failure** (`083`) that **never loses their work** (`084`), and an incident has a **named owner**
(`086`) and is **recorded** (`087`). The non-gating five (`082`, `085`, `088`, `089`) are honest,
recorded cuts: alert-noise tuning, a dedicated status channel, proactive user communications, and
post-incident follow-up discipline — real maturity steps tracked for the operability backlog, not
first-release blockers.

**Baseline consistency:** every durability-dependent requirement here references the relevant `REQ-DOM-*`
(chiefly `REQ-DOM-021` for the health signal, and `REQ-DOM-012/013` for the restart-outage context) and
neither restates nor alters it. Detection depends on the observability area's signals
(`REQ-OPS-060..079`) and is written to that seam without redefining them.

---

## existing_followups

Three follow-ups are already open on the `Nousix-LLC/nswe-demo` board. Operational readiness touches each,
so each is **placed where it belongs** — folded into the area it most affects (or recorded here as backlog)
with a one-line rationale — rather than re-minted as a new `REQ-OPS-*` or duplicated. None is dropped; none
is placed under more than one home. All three are open at the time of writing.

| Issue | Title (as on the board) | Placed under | Gating | Rationale |
|-------|-------------------------|--------------|--------|-----------|
| **#23** | infra follow-up: harden & exercise the native k3s install path | `backups_and_recovery` | non-gating (hardening) | Deterministic recovery and deploy-rollback (`REQ-OPS-029`) depend on a sound, exercised install/delivery path; hardening that path strengthens the recovery guarantee rather than adding a new requirement. |
| **#24** | argocd follow-up: least-privilege RBAC + supply-chain hardening before the first Application | `backups_and_recovery` | non-gating (hardening) | The GitOps rollback path (`REQ-OPS-029`) runs through the Argo CD `Application`; least-privilege RBAC and supply-chain hardening protect the integrity of the delivery/rollback mechanism the recovery story relies on. |
| **#25** | containerize follow-ups: non-blocking polish | operability backlog (no gating area) | non-gating (polish) | Explicitly non-blocking polish with no operational-readiness bar attached; tracked for the operability backlog so it is not forgotten, and intentionally not gated on first release. |

**Why these are placements, not requirements.** #23 and #24 harden the delivery/recovery path that
`REQ-OPS-029` (roll a bad deploy back to the previously-known-good running state) and `REQ-OPS-028` (a
recovery runbook) already require; the requirement is the testable bar, and these issues are the hardening
work that makes meeting it durable. #25 is quality polish with no readiness consequence. Each remains a live
board item; this section records *where each sits relative to the operational-readiness design*, consistent
with the delivery backlog's own handling of the same issues.


---

## requirements_and_acceptance_criteria

This is the consolidated register of every `REQ-OPS-*` across the five operational areas. The full
statement, acceptance criteria, and cross-references for each requirement live in its area section above;
this table is the single traceable index — id, concern, release-gating flag, what it builds on, and how it
is verified — assembled from the area registers without altering them. Ids are unique and each sits within
its area's allocated range (`durable_data` 001–019, `backups_and_recovery` 020–039, `limits_and_quotas`
040–059, `observability` 060–079, `status_and_incident` 080–099).

| REQ-OPS | Concern | Release gating | Traces to | Verification |
|---|---|---|---|---|
| REQ-OPS-001 | Durable state on pod-independent storage | true | REQ-DOM-013/012/014, REQ-PROD-031 | inspection |
| REQ-OPS-002 | Survives process restart | true | REQ-DOM-012, REQ-PROD-026/031 | operational_check |
| REQ-OPS-003 | Survives redeploy / rolling update | true | REQ-DOM-012/016, REQ-PROD-026/031 | operational_check |
| REQ-OPS-004 | Survives reschedule to another node | true | REQ-DOM-013/012, REQ-PROD-031 | operational_check |
| REQ-OPS-005 | Safe under >1 replica, or single-writer enforced | true | REQ-DOM-016/014, REQ-PROD-031 | inspection |
| REQ-OPS-006 | Confirmed write is durable before it is confirmed | true | REQ-DOM-012, REQ-PROD-026/017 | operational_check |
| REQ-OPS-007 | Every kind of domain state is durable | true | REQ-DOM-005/006/004/011/022/012, REQ-PROD-006/026 | operational_check |
| REQ-OPS-008 | Derived state correct across the durability boundary | true | REQ-DOM-015/004, REQ-PROD-020/021/026 | operational_check |
| REQ-OPS-009 | User content retained for life of account | true | REQ-DOM-012, REQ-PROD-026 | operational_check |
| REQ-OPS-010 | Time-bounded durable state has an enforced lifetime | true | REQ-DOM-006/005/010, REQ-PROD-005/006 | inspection |
| REQ-OPS-011 | Deletion is durable and bounded | true | REQ-DOM-009, REQ-PROD-010/012 | operational_check |
| REQ-OPS-012 | Capacity honours retention; exhaustion fails safe | true | REQ-DOM-012, REQ-PROD-026/017 | operational_check |
| REQ-OPS-013 | Durability demonstrable by a repeatable drill | true | REQ-DOM-012, REQ-PROD-031/034 | operational_check |
| REQ-OPS-020 | Durable store backed up on a defined, recent schedule | true | REQ-DOM-012/013, REQ-PROD-031/032 | operational_check |
| REQ-OPS-021 | A backup is complete: captures the whole durable model | true | REQ-DOM-012/015/022 | operational_check |
| REQ-OPS-022 | Recovery point objective (RPO) defined and met | true | REQ-PROD-031/032 | operational_check |
| REQ-OPS-023 | Recovery time objective (RTO) defined and met | true | REQ-PROD-032/034 | operational_check |
| REQ-OPS-024 | Restore is proven by a periodic restore drill | true | REQ-DOM-012/015, REQ-PROD-032 | operational_check |
| REQ-OPS-025 | Backup success and freshness are monitored | true | REQ-DOM-021, REQ-PROD-032 | operational_check |
| REQ-OPS-026 | Backups stored durably and separately from the primary | true | REQ-DOM-013, REQ-PROD-032 | inspection |
| REQ-OPS-027 | Backups access-controlled; retention defined | false | REQ-PROD-029/032 | inspection |
| REQ-OPS-028 | A documented recovery runbook exists | true | REQ-PROD-032/034 | inspection |
| REQ-OPS-029 | A bad deploy can be rolled back to known-good | true | REQ-DOM-019, REQ-PROD-031/032 | operational_check |
| REQ-OPS-030 | A deploy rollback does not destroy user data | true | REQ-DOM-012/013, REQ-PROD-031 | operational_check |
| REQ-OPS-031 | Forward schema migrations are applied safely | true | REQ-DOM-022, REQ-PROD-026/034 | operational_check |
| REQ-OPS-032 | A failed/bad schema migration has a recovery path | true | REQ-DOM-022, REQ-PROD-032 | operational_check |
| REQ-OPS-033 | The recovery path works on the self-hostable surface | true | REQ-PROD-034/032 | operational_check |
| REQ-OPS-040 | Per-caller request-rate limit on the public API | true | REQ-PROD-013/030/031, REQ-DOM-019, REQ-ID-* | operational_check |
| REQ-OPS-041 | Rate limits scoped; writes + auth endpoints tighter | true | REQ-PROD-001/003/004/007, REQ-DOM-019, REQ-ID-* | automated_test |
| REQ-OPS-042 | 429 breach response: well-formed, non-destructive, recoverable | true | REQ-DOM-017, REQ-PROD-006 | automated_test |
| REQ-OPS-043 | Bounded, documented request payload size; rejected early | true | REQ-DOM-002, REQ-PROD-016/030/031 | automated_test |
| REQ-OPS-044 | Every collection bounded by a max page size + default | true | REQ-PROD-018, REQ-DOM-004/010/018 | automated_test |
| REQ-OPS-045 | Pagination params clamp; bad cursors fail safe | false | REQ-PROD-018, REQ-DOM-004 | automated_test |
| REQ-OPS-046 | Per-account quota bounds runaway creation / storage growth | false | REQ-DOM-012/013, REQ-PROD-031, REQ-ID-* | automated_test |
| REQ-OPS-047 | Limits configurable with documented defaults; `/healthz` + SPA exempt | true | REQ-DOM-019, REQ-PROD-034/030 | inspection |
| REQ-OPS-060 | Health reports unhealthy when the product cannot serve users | true | REQ-DOM-021, REQ-PROD-030 | operational_check |
| REQ-OPS-061 | Liveness and readiness are distinct signals | true | REQ-DOM-021, REQ-PROD-030 | operational_check |
| REQ-OPS-062 | Readiness exercises its dependency, bounded, fails closed | true | REQ-DOM-021/014 | operational_check |
| REQ-OPS-063 | Health recovers on its own when the dependency returns | true | REQ-DOM-021, REQ-PROD-031 | operational_check |
| REQ-OPS-064 | Health endpoint cheap, safe, self-identifying | false | REQ-PROD-034 | inspection |
| REQ-OPS-065 | Every request produces a readable log record | true | REQ-PROD-030/034 | operational_check |
| REQ-OPS-066 | Logs structured, parseable, emitted for collection | true | REQ-PROD-034 | inspection |
| REQ-OPS-067 | Server faults logged with context, never silently | true | REQ-PROD-030 | operational_check |
| REQ-OPS-068 | A request can be followed by a correlation id | true | REQ-PROD-030 | inspection |
| REQ-OPS-069 | Logs never expose secrets or personal data | true | REQ-DOM-005/006, REQ-PROD-029 | inspection |
| REQ-OPS-070 | Lifecycle and fatal-startup events observable in logs | true | REQ-PROD-034, REQ-DOM-019 | operational_check |
| REQ-OPS-071 | Quantitative request metrics exposed for collection | true | REQ-PROD-030/034 | operational_check |
| REQ-OPS-072 | Degradation visible before users feel it | true | REQ-DOM-021/016, REQ-PROD-030 | operational_check |
| REQ-OPS-073 | Metrics exposition separable from user surface, safe to scrape | false | REQ-PROD-029/034 | inspection |
| REQ-OPS-074 | Product health defined as named, quantitative SLIs | true | REQ-PROD-030, REQ-DOM-021 | operational_check |
| REQ-OPS-075 | Observability documented for whoever operates the product | true | REQ-PROD-034 | manual_user_check |
| REQ-OPS-080 | Detect product-trouble on a product-health condition | true | REQ-DOM-021, REQ-PROD-030 | operational_check |
| REQ-OPS-081 | Notify a responsible human within a bounded time | true | REQ-PROD-030/033 | operational_check |
| REQ-OPS-082 | Alerts actionable; not noisy on a healthy product | false | REQ-PROD-030 | inspection |
| REQ-OPS-083 | Honest user-facing degraded/outage state, never broken | true | REQ-PROD-014/018/019/030 | manual_user_check |
| REQ-OPS-084 | Fail honestly during degradation; never lose work silently | true | REQ-PROD-006/017/020/021/031 | manual_user_check |
| REQ-OPS-085 | Users can learn whether a current problem is known | false | REQ-PROD-030 | manual_user_check |
| REQ-OPS-086 | Incident-response responsibility defined and documented | true | REQ-PROD-033/034 | inspection |
| REQ-OPS-087 | Incidents recorded durably | true | REQ-PROD-032/033 | inspection |
| REQ-OPS-088 | Users informed of significant-incident impact/resolution | false | REQ-PROD-030 | manual_user_check |
| REQ-OPS-089 | Recurrence-prevention follow-up per incident | false | REQ-PROD-032 | inspection |

**Register totals.** 61 requirements across the five areas: **52 release-gating**, **9 non-gating** (honest,
recorded cuts — `REQ-OPS-027`, `045`, `046`, `064`, `073`, `082`, `085`, `088`, `089`). The non-gating items
are expected and designed-for, scoped out of the first-release gate deliberately rather than forgotten. Ids
`REQ-OPS-014..019`, `034..039`, `048..059`, `076..079`, and `090..099` are unused, reserved as per-area
amendment headroom.

**Verification-method mix.** Every requirement names a method from the design's closed vocabulary
(`manual_user_check` | `automated_test` | `inspection` | `operational_check`); durability, backup/restore,
and health behaviours are predominantly `operational_check` (a property observed against the running
product), limit mechanics are predominantly `automated_test`, and configuration/structural properties are
`inspection`.

**Traceability to the release bar.** Every release-gating `REQ-OPS-*` here is a candidate the release bar
(`BAR-*`) must cover and the delivery backlog must carry to an issue; this register is the operational-area
input to that cross-document traceability matrix.
