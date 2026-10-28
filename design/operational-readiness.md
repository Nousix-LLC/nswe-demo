# Operational readiness

This document is the operational-readiness area of the chirp product design: what must be true in production **beyond "it runs"** for chirp to be operated, not just started. It states durable data, a backup and recovery path, limits and quotas, observability, and a status and incident posture — each as a testable `REQ-OPS-*` release requirement whose acceptance criteria a person who never met the author can check against the live product. It is **requirements, not implementation**: every engineering "how" (which backup tool, which metrics backend, which rate-limiter) is stated as a property the operated product must have and deferred to later code work.

It builds on, and does not restate, the domain persistence model (`REQ-DOM-100..118`) and the product-level operational needs (`REQ-PROD-037..041`, operator journey J9); it references those ids at the seam. The register below holds 75 requirements (`REQ-OPS-001..095`), 59 of them release-gating.

The seven sections below are, in order: `durable_data`, `backups_and_recovery`, `limits_and_quotas`, `observability`, `status_and_incident`, `existing_followups`, and `requirements_and_acceptance_criteria`.

---

## durable_data

*Durable data in production.*


This is the `durable_data` section of `design/operational-readiness.md`: the **operational posture**
around chirp's persistence model, stated as testable `REQ-OPS-*` requirements. The domain design already
fixed the persistence *model* — authoritative state in a durable store distinct from the serving process,
acknowledged writes survive, invariants hold across restart/redeploy/restore, readiness reflects store
reachability, upgrades preserve data (`REQ-DOM-100..118`). This section does not restate that model. It
states what an **operator and a user are owed in production** once that model is in place: that the
product's authoritative state lives durably as a standing property of an operated deployment, survives the
normal operations of running a service, and is **retained for as long as the product promises and no
longer**. The backups-and-recovery section (`REQ-OPS-020..039`) builds its backup, restore, and
recovery-target requirements on the durability posture fixed here; the readiness *signal* itself is the
observability section's (`REQ-OPS-060..079`) — this section states only that durability requires the store
to be run as a reachable, persistent component.

### What this section fixes, and what it defers

- **In this section**: the production durability expectation (a running deployment loses no user data to a
  restart, update, or redeploy); acknowledged-write durability as an operated property; the integrity
  properties that must still hold after an operational restart and after a restore; the requirement that the
  durable store be operated as a reachable, persistent component distinct from the serving process; and a
  stated **data-retention posture** — how long live user data, deleted-account data, sessions, and private
  identity data are kept, consistent with the defined deletion outcome (`REQ-DOM-037`, `REQ-DOM-112`) and
  with export (`REQ-DOM-117`).
- **Referenced, owned elsewhere**: the persistence *model* and its invariants (`REQ-DOM-100..118`, domain
  design) — consumed here by reference, not restated; the backup cadence, retention schedule,
  recovery-point / recovery-time targets, and the restore procedure (`REQ-OPS-020..039`,
  backups-and-recovery), which build on this section; the readiness *signal*, the health-vs-readiness
  distinction, and the audit trail (`REQ-OPS-060..079`, observability); the structure and handling of the
  account **Credential** and **Verified contact** (`REQ-ID-*`, identity/security); and the user-facing
  export surface and portable format (`REQ-UX-*` / `REQ-ID-*`).
- **Deferred to the code work (the "how")**: which store engine, replication or snapshot mechanism, schema,
  migrations, connection handling, and serialization. Every requirement below is a property the operated
  product must have, not a mechanism; where a mechanism is tempting it is stated as the property and the
  choice handed to the engineering code DAGs.

Entities are named by the domain model's canonical vocabulary (Account, Chirp, Follow, Like, Session,
Credential, Verified contact); this section introduces none the model does not define. The product and the
in-memory backing it closes are named by repository identity — `Nousix-LLC/nswe-demo @ c805525`, the
`ChirpRepository` seam in the `chirp-server` crate — never by any host, cluster, or deployment address.

### The posture in prose

**Durability is a standing property of an operated deployment, not a one-time property of a fresh start.**
The domain design requires that authoritative state survives a restart (`REQ-DOM-100`). The operational
requirement is stronger in a different direction: a deployment that an operator runs — restarting it,
updating it, redeploying it, rolling the serving process to a fresh instance — loses no user data in the
course of those routine operations, every time, for the life of the deployment. The operator does not have
to treat a restart as a risky event; it is a normal one. This is the operational face of `REQ-DOM-113/114`
(authoritative state in a durable store distinct from the serving process) and `REQ-DOM-118` (an upgrade
preserves existing data).

**The store is a component the deployment runs, not a convenience of the process.** For durability to be a
standing property, the durable store named by `REQ-DOM-113` must be operated as a persistent component
whose lifecycle is independent of any single serving process. An operator can replace the serving process
without touching the store, and the store outlives the process. A deployment that keeps authoritative state
only in the serving process's memory — the `c805525` blocker — does not meet this posture no matter how
rarely it restarts.

**Integrity survives the events of operating the product.** Recovery that brings the rows back but breaks
the model's invariants has not recovered the product (`REQ-DOM-110`). The operational posture is that after
an operational restart *and* after a restore from backup, every reference still resolves, ids are still
unique and never reused, counts still equal their edges, and an applied account deletion stays applied —
and that an interrupted write (a crash or restart mid-operation) never leaves half-applied state visible.
The backups-and-recovery section inherits this: a restore is "correct" only when these properties hold, and
that is the bar its restore-verification requirements are written against.

**Retention is a promise with a stated length, in both directions.** A releasable product owes its users a
plain answer to two questions: *how long do you keep my data?* and *when I delete my account, when is my
data actually gone?* The posture here is that live user data is retained for as long as the account exists —
no silent expiry, no background deletion of a live account's content — and that deleted-account data is
removed or de-identified per the stated deletion outcome (`REQ-DOM-037`, `REQ-DOM-112`) within a stated,
bounded window rather than lingering indefinitely, including in backups. Retention and export name the same
set: while an account exists, the data the product keeps for it is the data it can hand back (`REQ-DOM-117`).
The product states this posture where a user and an operator can find it.

The formal requirements are **REQ-OPS-001..016** in the register below.

### Requirements and acceptance criteria (`REQ-OPS-001..019`)

Each requirement carries an `id` (within this section's assigned range `REQ-OPS-001..019`), a `statement`
(what must be true in production, from the operator's or user's side), testable `acceptance_criteria`, a
`verification_method` from the contract vocabulary (`manual_user_check`, `automated_test`, `inspection`,
`operational_check`), a `release_gating` flag, and `traces_to` the product/domain requirement it builds on.
The range is generous; it is not filled, and the gaps are deliberate.

#### Production durability as an operated property

##### REQ-OPS-001 — A running deployment loses no user data to restart, update, or redeploy
- **statement**: An operated chirp deployment does not lose any user data — Accounts, Chirps, Follows, Likes, Sessions — to the normal operations of running the service: restarting the serving process, updating it, and redeploying it are routine operations after which all user data is present and unchanged. This is the operational standing of the domain model's survival requirement, held as a property of the operated service rather than of a single fresh start.
- **acceptance_criteria**: On an operated deployment, create Accounts and post Chirps, Follows, and Likes; perform each of a process restart, a redeploy, and an update as an operator would; after each, every Account and its Chirps, Follows, and Likes are present and unchanged, with no manual intervention to recover them.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-100

##### REQ-OPS-002 — Authoritative state is operated in a durable store distinct from the serving process
- **statement**: The deployment runs the product's authoritative state in a durable store that is a separate, persistent component from the serving process, with its own lifecycle; the serving process holds no authoritative state that exists only in its memory. The in-memory backing of the `ChirpRepository` seam at `Nousix-LLC/nswe-demo @ c805525` does not meet this and is the blocker this posture closes.
- **acceptance_criteria**: Inspect the deployment: the durable store is a component whose lifecycle is independent of the serving process (stopping or replacing the serving process does not stop or reset the store). Confirm `REQ-DOM-113/114` hold against the operated product. No deployment configuration keeps authoritative state solely in serving-process memory.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-113, REQ-DOM-114

##### REQ-OPS-003 — Replacing the serving process is a routine operation that touches no user data
- **statement**: An operator can replace or roll the serving process to a fresh instance, with no shared process memory, as a normal operation; all user data remains intact because it lives in the durable store, not the process.
- **acceptance_criteria**: On a populated deployment, replace the serving process with a fresh instance without touching the durable store; afterward all Accounts, Chirps, Follows, Likes, and active Sessions are intact, demonstrating the authoritative state did not live in the replaced process.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-113

##### REQ-OPS-004 — An operated upgrade preserves existing user data
- **statement**: Performing an upgrade in production — including a redeploy that changes how the data is shaped — preserves the user data that existed before it: afterward the prior Accounts, Chirps, Follows, and Likes are all present and still satisfy their invariants. The migration mechanics are the code work's; the preservation property across an operated upgrade is the bar's.
- **acceptance_criteria**: On a populated deployment, perform an upgrade that changes the data's shape; afterward the Accounts, Chirps, Follows, and Likes created before the upgrade are all present and their references still resolve. No data created before the upgrade is lost or corrupted by it.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-118

#### Acknowledged-write durability as an operated property

##### REQ-OPS-005 — An acknowledged write survives an immediate restart in production
- **statement**: In the operated product, any create or change the product has confirmed to a person — a posted Chirp, a Follow or unfollow, a Like or unlike, a profile edit, an account deletion — is durable at the moment it is acknowledged, so a restart or process replacement happening immediately after the success response does not lose it. Durability is a correctness property of the acknowledgement, not a tendency observed over time.
- **acceptance_criteria**: On an operated deployment, perform a write and receive the product's success response; with no further action, restart or replace the serving process; the written state is present afterward. Repeated across each write type, no acknowledged write is lost by a restart in the window immediately following its acknowledgement.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-109

#### Integrity across operational events

##### REQ-OPS-006 — Domain invariants hold after an operational restart
- **statement**: After the serving process is restarted or replaced in production, the domain model's invariants still hold against the durable data: every reference resolves, ids are unique and never reused, each Account's follower/following counts and each Chirp's like count equal their current edges, and any applied account deletion stays applied. A restart never introduces a dangling reference, a reused id, or a count that disagrees with its edges.
- **acceptance_criteria**: On a populated deployment, record a sample of references, ids, and counts; restart the serving process; traverse Chirps, Replies, Follows, and Likes — every referenced Account and Chirp resolves, no recorded id has moved or been reused, and the counts equal the current edges (add and remove a Follow and a Like after the restart and confirm the counts track by exactly one each way).
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-107, REQ-DOM-110, REQ-DOM-111, REQ-DOM-112

##### REQ-OPS-007 — Domain invariants hold after a restore from backup (the posture backups build on)
- **statement**: After a restore from backup, the durable data satisfies the same invariants required after a restart (`REQ-OPS-006`): references resolve, ids are unchanged and never reused, counts equal their edges, and applied deletions remain applied. A restore that brings rows back but breaks these invariants has not recovered a correct product. This is the durability posture the backups-and-recovery section's restore-correctness requirements (`REQ-OPS-020..039`) are written against.
- **acceptance_criteria**: On a store restored from a backup of a populated deployment, traverse Chirps, Replies, Follows, and Likes — every reference resolves, recorded ids are unchanged and unreused, counts equal their edges, and any account deleted before the backup is still deleted with no dangling reference. A failing invariant is reported as an unsuccessful restore, never silently accepted.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-110, REQ-DOM-116

##### REQ-OPS-008 — An interrupted write never leaves half-applied state visible
- **statement**: A restart, crash, or process replacement that happens while a write is in progress leaves the durable data in a consistent state: the operation is observable afterward as either fully applied or not applied, never as a half-applied change with a dangling reference or a count that disagrees with its edges. Account deletion in particular is all-or-nothing across an interruption.
- **acceptance_criteria**: Interrupt the serving process during representative writes (posting a Chirp, following, deleting an account that has Chirps, Follows, and Likes); after recovery, each interrupted operation is observably either fully applied or fully absent, and no partially applied state (a Chirp without a resolvable author, a half-deleted Account with dangling references, a count out of step with its edges) is observable.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-110, REQ-DOM-112

#### The durable store's reachability as a durability property

##### REQ-OPS-009 — Durability requires the store to be operated as a reachable, persistent component
- **statement**: Because the serving process depends on the durable store to serve, durability in production requires the store to be run as a reachable, persistent component for the life of the deployment: a serving process that cannot reach its durable state cannot serve, and a deployment that does not keep its store reachable and persistent does not meet the durability posture. This section states the dependency as a durability requirement; the readiness *signal* that reports store reachability is the observability section's (`REQ-OPS-060..079`), building on `REQ-DOM-115` / `REQ-PROD-039`.
- **acceptance_criteria**: Confirm the deployment runs the durable store as a persistent, reachable component independent of the serving process. With the store made unreachable to the serving process, the product does not serve user writes or reads from process memory as if durable (it does not silently fall back to a non-durable store). The readiness signal's behaviour under this condition is verified by the observability section.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-039, REQ-DOM-115

#### Data-retention posture

##### REQ-OPS-010 — Live user data is retained for the life of the account
- **statement**: While an account exists, its user data — the Account and its profile, its Chirps, its Follows, and its Likes — is retained in full: the product does not silently expire, age out, or delete a live account's content as a side effect of operating the service. Any retention limit the product chooses to place on a live account's content is stated, not silent.
- **acceptance_criteria**: Create an account with Chirps, Follows, and Likes; over a sustained operating period with ordinary restarts and redeploys, the account's content remains present and complete without the holder re-creating it. If any component of a live account's data is subject to an age-out, that limit is found stated in the product's retention posture (`REQ-OPS-016`) rather than observed as an unexplained loss.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037

##### REQ-OPS-011 — Deleted-account data is removed within a stated, bounded window
- **statement**: When an account is deleted, its data is removed or de-identified per the stated deletion outcome (`REQ-DOM-037`, `REQ-DOM-112`) within a stated, bounded retention window after the deletion is applied, rather than being retained indefinitely. The product states the window; after it elapses, the deleted account's data is not retained in the live durable store.
- **acceptance_criteria**: Find the product's stated deleted-account retention window. Delete an account that has Chirps, Follows, and Likes; after the deletion is applied the stated outcome holds immediately (the account cannot sign in, no dangling reference remains per `REQ-DOM-112`), and after the stated window the deleted account's data is no longer present in the live durable store. The behaviour matches the stated window.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-037, REQ-DOM-112

##### REQ-OPS-012 — Backups do not retain deleted-account data beyond a stated bound
- **statement**: Deleted-account data does not persist indefinitely in backups: a stated maximum backup-retention bound limits how long a deleted account's data can remain recoverable in backups, after which the backups holding it have aged out. This section fixes that such a bound exists and is consistent with the deletion posture; the backup cadence, retention schedule, and ageing mechanics are the backups-and-recovery section's (`REQ-OPS-020..039`), which builds on this.
- **acceptance_criteria**: Find the product's stated maximum backup-retention bound and confirm it is consistent with the deleted-account retention posture (`REQ-OPS-011`). Confirm that no backup older than that bound is retained, so deleted-account data cannot be recovered from backups indefinitely. The concrete cadence and schedule are verified against the backups-and-recovery requirements.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-DOM-037, REQ-PROD-038

##### REQ-OPS-013 — Retention and export cover the same per-account set
- **statement**: The data the product retains for a live account is the data it can hand that person back: while an account exists, its retained profile, full set of Chirps, and full set of followed Accounts are enumerable for the holder's own export (`REQ-DOM-117`). A person cannot be told their data is kept while being unable to obtain it, and export reflects the committed durable state at the time of the request.
- **acceptance_criteria**: For a signed-in account with Chirps and Follows, the data the product retains (per `REQ-OPS-010`) and the data a self-service export yields (per `REQ-DOM-117`) are the same set — the account's profile, its complete Chirps, and its complete followed Accounts — matching the committed durable state. No retained category of a person's own data is absent from what they can export.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-DOM-117, REQ-PROD-019

##### REQ-OPS-014 — Sessions are retained only for their bounded lifetime
- **statement**: Active Sessions are retained durably so a signed-in person is not silently signed out by a restart or redeploy (`REQ-DOM-106`), but only until the Session ends (sign-out) or passes its lifetime (`REQ-DOM-030`, `REQ-DOM-031`); an ended or expired Session is not retained as usable and is never revived by a restart or a restore.
- **acceptance_criteria**: Sign in to obtain a Session; restart or redeploy the service — a request carrying that Session's token is still accepted as the same account. Separately, end a Session (or let it pass its lifetime), then restart and restore from a backup taken while it was active — a request carrying that token is refused in both cases, not revived.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-106, REQ-DOM-030, REQ-DOM-031

##### REQ-OPS-015 — Private identity data is retained durably, kept non-public, and removed with the account
- **statement**: The account-associated Credential and Verified contact are retained durably as part of the Account for the life of the account so a person can still sign in and still recover after a restart (`REQ-DOM-102`), are never exposed in any public response, and are removed or de-identified with the account under the stated deletion outcome and within the deleted-account retention window (`REQ-OPS-011`). Their structure and handling are the identity/security design's (`REQ-ID-*`); this requirement is that they persist, stay non-public, and are not retained past the account.
- **acceptance_criteria**: After a restart and a restore, a person can still sign in with their Credential and the recovery path still recognises their Verified contact, and neither value ever appears in any public response. After an account is deleted and the retention window elapses, the deleted account's Credential and Verified contact are no longer retained in the live durable store.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-102, REQ-DOM-037

##### REQ-OPS-016 — The retention and deletion posture is stated where users and operators can find it
- **statement**: The product states, in a place a user and an operator can find, what user data it keeps, for how long, and what account deletion removes and when — covering live-account retention (`REQ-OPS-010`), the deleted-account window (`REQ-OPS-011`), the backup bound (`REQ-OPS-012`), and session and private-identity-data handling (`REQ-OPS-014`, `REQ-OPS-015`). A releasable product does not leave its retention and deletion promise implicit.
- **acceptance_criteria**: Locate the product's stated data-retention and deletion posture as a user/operator could. It names what is retained, the live-account retention, the deleted-account retention window, the backup-retention bound, and session/private-data handling, and these statements match the behaviour verified by `REQ-OPS-010..015`. A discrepancy between the stated posture and the observed behaviour is a defect.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-037

---

## backups_and_recovery

*Backups and recovery.*


This is the `backups_and_recovery` section of `design/operational-readiness.md`: the **backup and
recovery path** a release of chirp requires, stated as testable `REQ-OPS-*` requirements. The domain
design fixed that the capability must exist — the durable user data can be backed up and restored, and a
restore brings the Accounts, Chirps, Follows, and Likes back consistent with every model invariant
(`REQ-DOM-116`), and the product-level need is a release-gating product requirement (`REQ-PROD-038`). The
durable-data section (`REQ-OPS-001..019`) fixed the operational durability posture this path protects:
authoritative state lives in a durable store distinct from the serving process, and the invariants the
model requires after a restart must hold again after a restore (`REQ-OPS-007`). This section does not
restate either. It states what an **operator is owed** so that durable data, once it exists, can actually
be recovered: that it is backed up on a known cadence, kept for a known period, recoverable to stated loss
and downtime targets, restorable by a defined procedure that returns a correct product, recoverable after a
bad deploy, and proven restorable rather than merely produced. This is the operational face of the operator
journey's unmet need — *backup and recovery for durable data* (J9) — which the product requirements record
as a release gap.

### What this section fixes, and what it defers

- **In this section**: that durable user data is backed up on a stated, regular cadence; that backups are
  retained for a stated, bounded period; that each backup is an internally consistent point-in-time image
  held independently of the primary store; a recovery-point target (tolerable data loss) and a
  recovery-time target (tolerable downtime); a defined, operator-performable restore procedure that returns
  the Accounts, Chirps, Follows, and Likes intact and consistent; recovery from a failed or data-damaging
  deploy; and the requirement that backups are periodically proven restorable, that backup failures are
  surfaced rather than silent, and that the whole posture is written where an operator can find it.
- **Referenced, owned elsewhere**: the durability posture these requirements protect and the
  invariants-after-restore bar they are written against (`REQ-OPS-001..019`, especially `REQ-OPS-007`;
  durable-data) — consumed here by reference, not restated; the backup-retention bound as it constrains
  **deleted-account** data (`REQ-OPS-012`, durable-data), which this section's retention requirement builds
  on rather than redefines; the persistence model and its invariants (`REQ-DOM-100..118`, domain design);
  the alerting *signal* that carries a backup failure to an operator, which is the observability section's
  (`REQ-OPS-060..079`) — this section states that a failure must be surfaced, not how it is signalled.
- **Deferred to the code work (the "how")**: which backup tool or mechanism, which storage target, the
  snapshot or replication method, the restore scripting, and the deploy/rollback machinery. Every
  requirement below is a property the operated product must have, not a mechanism; where a mechanism is
  tempting it is stated as the property and the choice is handed to the engineering code DAGs.

Entities are named by the domain model's canonical vocabulary (Account, Chirp, Follow, Like, Session); this
section introduces none the model does not define. The product and the durable store are named by
repository identity — `Nousix-LLC/nswe-demo @ c805525`, the `ChirpRepository` seam in the `chirp-server`
crate — never by any host, cluster, or deployment address.

### The posture in prose

**A backup is only real once it has brought a correct product back.** The domain design already requires
that the capability exists (`REQ-DOM-116`); the operational requirement is that the capability is run as a
standing part of operating the service and is trusted because it has been exercised. A deployment that
produces backups it has never restored from has a file, not a recovery path. The bar this section is
written to is that an operator who has lost the durable store can, following a written procedure, bring the
product back to a known-good point within a stated time, and that what comes back satisfies the same
invariants a restart must satisfy (`REQ-OPS-007`, `REQ-DOM-110/111/107`): references resolve, ids are
unchanged and never reused, counts equal their edges, and applied account deletions stay applied.

**Cadence and retention set the two numbers a person's recovery depends on.** How often a backup is taken
bounds how much a person can lose — the work done since the last good backup. How long backups are kept
bounds how far back an operator can reach, and, in the other direction, how long a deleted account's data
can still be recovered from a backup. The durable-data section already fixed that a maximum backup-retention
bound exists and is consistent with the deletion posture (`REQ-OPS-012`); this section states the cadence
and the retention period themselves, and keeps them consistent with that bound rather than redefining it.

**Loss and downtime are commitments, not hopes.** A releasable product that can be operated states how much
data it is willing to lose in a disaster (a recovery-point objective) and how long it is willing to be down
while recovering (a recovery-time objective), and it sets them so they can be met — the recovery-point
objective no tighter than the backup cadence allows, the recovery-time objective no tighter than a
rehearsed restore achieves. A target nobody has measured a restore against is an aspiration; the restore
drill is what turns it into a commitment.

**A bad deploy is the disaster most likely to happen.** Hardware loss is rare; a deploy that breaks the
product or damages its data is routine enough that recovering from one is part of operating the service. A
failed deploy must be reversible to the last working version without losing user data, and a deploy that
has already damaged data must be recoverable by restoring from backup — consistent with the domain
requirement that an upgrade preserves existing data (`REQ-DOM-118`). The rollback and restore are the same
recovery path this section defines, applied to the most common cause of needing it.

The formal requirements are **REQ-OPS-020..033** in the register below.

### Requirements and acceptance criteria (`REQ-OPS-020..039`)

Each requirement carries an `id` (within this section's assigned range `REQ-OPS-020..039`), a `statement`
(what must be true in production, from the operator's side), testable `acceptance_criteria`, a
`verification_method` from the contract vocabulary (`manual_user_check`, `automated_test`, `inspection`,
`operational_check`), a `release_gating` flag, and `traces_to` the product/domain requirement it builds on.
The range is generous; it is not filled, and the gaps are deliberate.

#### Backup posture: cadence, retention, consistency, independence

##### REQ-OPS-020 — Durable user data is backed up on a stated, regular cadence
- **statement**: An operated chirp deployment backs up its durable user data — Accounts, Chirps, Follows, Likes — on a stated, regular cadence, as a standing part of operating the service rather than as an occasional manual act. The product states what the cadence is; backups are produced on it without an operator having to remember to take them.
- **acceptance_criteria**: Find the product's stated backup cadence. On an operated deployment with user data, observe over a period spanning several cadence intervals that a backup is produced each interval without manual intervention, and that each produced backup is recorded where an operator can see it was taken. Backups appear on the stated cadence, not ad hoc.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-DOM-116

##### REQ-OPS-021 — Backups are retained for a stated, bounded period
- **statement**: Backups are kept for a stated retention period that bounds how far back an operator can recover, and that period is bounded above so backups do not accumulate indefinitely. The retention period is consistent with the maximum backup-retention bound the durable-data section fixed for deleted-account data (`REQ-OPS-012`): a backup older than the stated period has aged out and is no longer retained.
- **acceptance_criteria**: Find the product's stated backup-retention period and confirm it is consistent with the deleted-account backup bound (`REQ-OPS-012`). On an operated deployment, confirm that backups within the period are available to restore from and that backups older than the period are no longer retained. The oldest available backup is no older than the stated period.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-DOM-037

##### REQ-OPS-022 — Each backup is an internally consistent point-in-time image
- **statement**: A single backup captures the durable user data as of one point in time, consistent within itself: restoring it yields a state in which the model's invariants hold, not a smear of rows captured at different moments with references that no longer resolve. A backup taken while the product is serving does not record a Chirp without its author or a count out of step with its edges.
- **acceptance_criteria**: Take a backup of a populated deployment while writes are occurring; restore it to a separate store and traverse Chirps, Replies, Follows, and Likes — every reference resolves, ids are unchanged and unreused, and counts equal their edges, with no partially captured write visible. A backup that restores to a state violating an invariant is treated as an unusable backup, not accepted.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-116, REQ-DOM-110

##### REQ-OPS-023 — Backups are stored independently of the primary durable store
- **statement**: Backups are held in a location whose loss is independent of the primary durable store, so that the failure a backup protects against — losing the store — does not also take the backups. A backup kept only alongside the data it backs up does not meet this posture.
- **acceptance_criteria**: Inspect where backups are stored relative to the primary durable store; the two do not share a single point of failure such that losing the primary store also loses its backups. Confirm that, with the primary store made unavailable, a retained backup is still reachable to restore from.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038

#### Recovery objectives

##### REQ-OPS-024 — A stated recovery-point objective (tolerable data loss)
- **statement**: The product states a recovery-point objective: the maximum amount of recent durable data, measured as a time window of work, that a recovery is permitted to lose in a disaster. The objective is set no tighter than the backup cadence (`REQ-OPS-020`) can actually achieve, so it is a target a recovery can meet rather than an aspiration.
- **acceptance_criteria**: Find the product's stated recovery-point objective and confirm it is consistent with the backup cadence (a recovery-point objective of N cannot be met by a cadence coarser than N). The stated objective names a bounded quantity of tolerable data loss, not "none" without a mechanism that would achieve none.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038

##### REQ-OPS-025 — A stated recovery-time objective (tolerable downtime)
- **statement**: The product states a recovery-time objective: the maximum time the product may be unavailable to users while an operator recovers it from backup after a disaster. The objective is a commitment the restore procedure is built and rehearsed to meet, bounding the downtime a person experiences in a recovery.
- **acceptance_criteria**: Find the product's stated recovery-time objective. It names a bounded recovery duration and is consistent with the availability the product commits to (`REQ-PROD-040`). The objective is the one the restore drill (`REQ-OPS-031`) measures the restore against.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-PROD-040

#### Restore procedure

##### REQ-OPS-026 — A restore is a defined, operator-performable procedure
- **statement**: Recovering the product from a backup is a defined procedure an operator can follow, not an improvisation invented during an incident: the steps to restore the durable store from a retained backup and return the product to service are written down and performable by an operator who did not design the system. The scripting and tooling are the code work's; that a defined, followable procedure exists is the bar's.
- **acceptance_criteria**: Locate the product's stated restore procedure. An operator who did not build the system can follow it to restore a deployment from a retained backup and return it to service, without needing knowledge that is not in the procedure. The procedure names what is restored (Accounts, Chirps, Follows, Likes) and how the product is returned to serving.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-DOM-116

##### REQ-OPS-027 — A restore brings back accounts, chirps, follows, and likes intact and consistent
- **statement**: A restore from a retained backup returns the durable user data intact and consistent: the Accounts, Chirps, Follows, and Likes present at the backup's point in time are all present afterward, and they satisfy the same invariants a restart must satisfy (`REQ-OPS-007`) — references resolve, ids are unchanged and never reused, counts equal their edges, and any account deleted before the backup stays deleted with no dangling reference. A restore that brings rows back but breaks these invariants has not recovered a correct product.
- **acceptance_criteria**: On a store restored from a backup of a populated deployment, every Account, Chirp, Follow, and Like present at the backup point is present; traverse Chirps, Replies, Follows, and Likes — every reference resolves, recorded ids are unchanged and unreused, counts equal their edges, and an account deleted before the backup is still deleted with no dangling reference. A failing invariant is reported as an unsuccessful restore, never silently accepted.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-116, REQ-DOM-110, REQ-DOM-111, REQ-DOM-107

##### REQ-OPS-028 — An exercised restore meets the stated recovery-point and recovery-time objectives
- **statement**: When the restore procedure is exercised, it recovers the product within the stated recovery-time objective (`REQ-OPS-025`) and loses no more than the stated recovery-point objective (`REQ-OPS-024`): the downtime and data loss a real recovery incurs are within the commitments the product has made, not merely asserted by them.
- **acceptance_criteria**: Perform a timed restore from a backup on a representative deployment: measure the elapsed time from starting the restore to the product serving users again — it is within the stated recovery-time objective — and measure the data lost relative to the disaster point — it is within the stated recovery-point objective. A drill that exceeds either objective is a defect in the objective or the procedure, reported rather than ignored.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-038

#### Recovery from a bad deploy

##### REQ-OPS-029 — A failed deploy can be rolled back to the last working version with no user-data loss
- **statement**: A deploy that fails or ships a broken product can be rolled back to the last working version as a routine operation, and the rollback loses no user data: the Accounts, Chirps, Follows, and Likes present before the deploy are all present after the rollback. Rolling back is a recovery operation an operator can perform, not a reason to lose data.
- **acceptance_criteria**: On a populated deployment, perform a deploy and then roll it back to the prior version as an operator would; afterward the product serves on the prior version and every Account, Chirp, Follow, and Like that existed before the deploy is present and unchanged. The rollback requires no manual reconstruction of user data.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-118

##### REQ-OPS-030 — A data-damaging deploy can be recovered from by restore
- **statement**: A deploy that damages durable data — corrupting it or losing part of it despite the preserve-existing-data requirement (`REQ-DOM-118`) — can be recovered from by restoring from a backup taken before the deploy, returning the product to a correct state consistent with `REQ-OPS-027`. The preserve-data property is the first line; a restore path for when it is breached is the second.
- **acceptance_criteria**: Simulate a deploy that damages or loses durable data on a populated deployment; recover by restoring from a pre-deploy backup following the restore procedure (`REQ-OPS-026`); afterward the durable data is intact and consistent per `REQ-OPS-027`, recovered to the pre-deploy point within the stated objectives (`REQ-OPS-028`). The product does not depend on the damaging deploy being reversible in place.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-118, REQ-DOM-116

#### Backup verifiability

##### REQ-OPS-031 — Backups are periodically proven restorable
- **statement**: Backups are proven restorable on a recurring basis, not merely produced: a restore from a recent backup is exercised periodically against a store where its correctness can be checked, and the exercise verifies the restored data satisfies its invariants (`REQ-OPS-027`) and meets the stated objectives (`REQ-OPS-028`). A backup that has never been restored is not yet known to be a backup.
- **acceptance_criteria**: Find the product's stated restore-drill cadence and the record of its most recent drills. Confirm a restore from a recent backup has been exercised within that cadence, that the drill checked the restored data's invariants and timing, and that a failed drill is recorded and acted on rather than passed over. The drill is recurring, not a one-time pre-release event.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-DOM-116

##### REQ-OPS-032 — A failed or missing backup is surfaced, not silent
- **statement**: When a scheduled backup fails or does not run, that fact is surfaced to operators rather than passing silently, so a deployment cannot drift into having no recent usable backup without anyone knowing. This section requires that the failure is detectable and raised; the alerting signal that carries it to an operator is the observability section's (`REQ-OPS-060..079`), building on the same posture.
- **acceptance_criteria**: Induce a backup failure or a missed scheduled backup on an operated deployment; the condition is surfaced to operators (a visible failure state or alert), not absorbed silently. Confirm that a run of successful backups and a failed backup are distinguishable to an operator. The signal's shape is verified against the observability requirements.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-038

##### REQ-OPS-033 — The backup and recovery posture is stated where an operator can find it
- **statement**: The product states, in a place an operator can find, its backup cadence and retention (`REQ-OPS-020`, `REQ-OPS-021`), its recovery-point and recovery-time objectives (`REQ-OPS-024`, `REQ-OPS-025`), and the restore and rollback procedures (`REQ-OPS-026`, `REQ-OPS-029`). A releasable product that can be operated does not leave its recovery procedure to be reconstructed under incident pressure.
- **acceptance_criteria**: Locate the product's stated backup-and-recovery posture as an operator could. It names the cadence, the retention period, the recovery-point and recovery-time objectives, and the restore and rollback procedures, and these statements match the behaviour verified by `REQ-OPS-020..032`. A discrepancy between the stated posture and the observed behaviour is a defect.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038

---

## limits_and_quotas

*Limits and quotas.*


chirp is a public, internet-facing consumer web application: anyone can reach it, and the write surface —
posting a chirp, following an account, liking a chirp — is open to any signed-in account, with an
unauthenticated surface (the landing page, sign-up, sign-in) open to anyone at all. A surface like that,
released with no limits, can be driven past what the service can carry by a single script, and can be used
by one account to bury everyone else's timeline. This section states the rate, size, and volume limits a
release needs so that neither happens: the operational first line against overload and trivial abuse.

These are requirements, not an implementation. Where a limit belongs (the edge, a gateway, the application),
which algorithm enforces it (token bucket, leaky bucket, fixed window), and the exact numeric thresholds are
the later code work's to decide. What a release owes is that the limits exist, that exceeding one is handled
by refusing or slowing the excess rather than serving it without bound, and that the caller is told. The
numbers themselves are deliberately left to enforcement: a requirement that hard-codes "N requests per
minute" ages badly and tells a tester nothing they can check against intent. The requirements below fix the
*behaviour* and leave the *threshold* to be set — and re-set — operationally.

**What this section covers and what it leaves to its neighbours.**

- **Rate and volume.** A per-client limit on the overall request rate, tighter limits on the actions that
  create state, and a bounded volume of created content per account over a longer window. Together these are
  the two halves of "cannot overwhelm the service" and "cannot flood others" (`REQ-PROD-041`).
- **Size, at the edge.** A cap on request body size, and the page-size cap on list reads. The per-field
  content bounds are already fixed by the domain design and are **referenced, not restated** here: a chirp
  body is 1–280 characters (`REQ-DOM-012`) and a handle is 1–15 characters (`REQ-DOM-002`). The operational
  requirement is that oversized input is refused at the edge, before it reaches a handler, consistent with
  those bounds. Timeline reads are already required to come back a bounded page at a time behind an opaque
  cursor with a server-enforced maximum (`REQ-DOM-027`, `REQ-DOM-218`); the operational requirement is that
  this cap is enforced on every list surface so no client can ask for an unbounded page.
- **The over-limit response.** When a limit is hit, the product rejects or throttles the excess and tells the
  caller with a typed, machine-readable error, consistent with the error contract (`REQ-DOM-220`). A limit
  that silently drops work, or that answers with an ad-hoc string the frontend cannot branch on, is not a
  finished limit.

**Boundaries — what this section references but does not own.**

- **Account-level abuse controls belong to identity and trust (`REQ-ID-*`).** The limits here are coarse and
  content-blind: they bound how fast and how much any client may do, not *who* a client is or whether a
  specific account is behaving abusively. Blocking and muting another account, reporting abuse, stated rules,
  and the operator's supported action on an abusive account are the identity/security and product-safety
  designs' (`REQ-ID-*`, and `REQ-PROD-025` / `REQ-PROD-028`). Rate and volume limiting is the first line that
  holds while those controls do their slower, account-specific work; it is not a substitute for them.
- **Operational access control (board follow-up #24).** Follow-up **#24** (least-privilege delivery RBAC and
  supply-chain hardening) concerns who and what may change the *running system* through the delivery and
  control plane — a privileged, operator-and-automation-facing access posture, distinct in subject from the
  public, client-facing rate and volume limits owned here. Its substance is an operational access/limits
  matter, but it bounds privileged control-plane actors rather than public clients. It is **referenced here
  for the synthesis gate to place**, not re-filed and not owned as a `REQ-OPS-04x` limit; the synthesis gate
  owns the final placement of #24 across the operational-readiness sections.

#### Requirements

##### REQ-OPS-040 — The public request rate is bounded per client
- **statement**: The product bounds the rate of requests it will accept from a single client, so one client cannot drive the service past what it can serve for everyone else by issuing requests as fast as it can. The threshold, the client-identification basis, and the enforcement point are set operationally; the requirement is that a bound exists and is applied per client rather than only in aggregate.
- **acceptance_criteria**: From a single client, issue requests far faster than a person using the product would; past a point the product stops serving the excess (it refuses or slows further requests) rather than accepting every request without limit, and other clients continue to be served normally meanwhile.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041

##### REQ-OPS-041 — State-creating actions are rate-limited more tightly than reads
- **statement**: The actions that create state — posting a chirp, following an account, liking a chirp — are subject to their own, tighter rate limit than ordinary reads, so a single account cannot flood timelines or follow/like en masse at machine speed. The specific limits are operational; the requirement is that a write-rate bound exists on each state-creating action.
- **acceptance_criteria**: Signed in, perform one state-creating action (for example posting) repeatedly, far faster than a person would; past a point the product refuses or throttles the further writes rather than accepting them without limit. Reading remains possible within its own limit while writes are being throttled.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041

##### REQ-OPS-042 — Created volume per account is bounded over a longer window
- **statement**: Beyond the short-term write rate, the volume of content a single account can create over a longer rolling window is bounded, so an account cannot flood others by sustaining an at-the-limit rate indefinitely. This is the "volume" half of the product's rate-and-volume requirement, distinct from the burst limit of REQ-OPS-041. The window and the cap are operational; the requirement is that a sustained-volume bound exists per account.
- **acceptance_criteria**: With one account, create content continuously at or just under the per-action rate limit for a sustained period; the product eventually bounds the account's further creation for that window rather than allowing unbounded accumulation, and resumes in a later window.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041

##### REQ-OPS-043 — Oversized request bodies are refused at the edge
- **statement**: The product enforces a maximum request body size and refuses a request whose body exceeds it before the body is processed, so a client cannot consume unbounded memory or processing by sending an enormous payload. The cap is consistent with the already-fixed content bounds — a chirp body of 1–280 characters (REQ-DOM-012) and a handle of 1–15 characters (REQ-DOM-002) — and exists to stop payloads far larger than any valid request, not to re-enforce those per-field rules.
- **acceptance_criteria**: Send a request whose body is far larger than any valid chirp or form submission (for example a multi-megabyte body to a write endpoint); the product refuses it with an error and does not attempt to process the oversized body. A normal, within-bounds request to the same endpoint succeeds.
- **verification_method**: automated_test
- **release_gating**: true
- **traces_to**: REQ-PROD-041, REQ-DOM-012, REQ-DOM-002

##### REQ-OPS-044 — List reads are served only as bounded pages
- **statement**: Every endpoint that returns a list enforces the server's maximum page size, so no client can force the product to assemble and return an unbounded result in one response. This is the operational enforcement of the domain requirement that timelines are read a bounded page at a time behind an opaque cursor (REQ-DOM-027, REQ-DOM-218); the operational requirement is that the cap holds on every list surface, not only the timeline.
- **acceptance_criteria**: Request a list with no page size, and request one with a page size well above the server maximum; in both cases the response carries at most the server's maximum number of items, with a continuation cursor when more remain, rather than the full result set.
- **verification_method**: automated_test
- **release_gating**: true
- **traces_to**: REQ-PROD-041, REQ-DOM-218, REQ-DOM-027

##### REQ-OPS-045 — Exceeding a limit refuses or throttles the excess and says so
- **statement**: When any limit in this section is exceeded, the product rejects or throttles the excess rather than serving it without bound, and tells the caller with a typed, machine-readable error carrying a distinct category for being rate- or size-limited, consistent with the error contract (REQ-DOM-220). The caller can tell "you were limited" apart from other failures and can act on it (for example back off and retry later).
- **acceptance_criteria**: Drive any one of the limits above past its threshold; the response is an error whose machine-readable category identifies it as a limit being hit — distinct from a validation error or a not-found — and the excess work is not performed. The error is the typed contract category, not an ad-hoc string.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041, REQ-DOM-220

##### REQ-OPS-046 — The unauthenticated surface is limited too
- **statement**: The request-rate limit applies to the unauthenticated surface — the landing page, sign-up, and sign-in — identified by client rather than by account, so a client cannot escape limiting simply by not signing in, and the pre-sign-in surface cannot be used as an unbounded overwhelm or credential-guessing vector. This bounds request volume on that surface; account-specific abuse handling and credential-attempt lockout belong to the identity design (REQ-ID-*) and are referenced, not owned here.
- **acceptance_criteria**: Without signing in, issue requests to a public, unauthenticated endpoint far faster than a person would; past a point the product refuses or throttles the excess, the same as for an authenticated client.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041

##### REQ-OPS-047 — Under aggregate overload, the product sheds rather than collapses
- **statement**: When total load exceeds what the product can serve — even from many within-limit clients at once — the product sheds or throttles the excess and keeps serving the traffic it can, rather than failing wholesale for everyone. This protects the product's reachability and reasonable availability (REQ-PROD-040) and is the aggregate counterpart to the per-client limits above. The capacity point and the shedding policy are operational.
- **acceptance_criteria**: Apply aggregate load beyond the product's serving capacity; the product continues to serve a bounded share of requests (and reports the rest as limited or unavailable per REQ-OPS-045) rather than becoming unresponsive to all callers, and it recovers to normal service when the load subsides.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040, REQ-PROD-041

##### REQ-OPS-048 — Limits sit above ordinary use
- **statement**: The limits in this section are set above the rate and volume of ordinary legitimate use, so a person using the product normally — reading a timeline, posting through the day, following and liking as they browse — does not hit them. Limiting targets abuse and overload, not use; a limit that blocks a normal person is a worse product than a more generous one. The thresholds are operational, but the requirement that they clear ordinary use is a release requirement.
- **acceptance_criteria**: As one person, use the product the way a normal user would over a normal session — browse, post several times, follow and like a reasonable number of accounts — without being rate- or volume-limited. Only usage well beyond what a person does by hand triggers a limit.
- **verification_method**: manual_user_check
- **release_gating**: false
- **traces_to**: REQ-PROD-041

---

## observability

*Observability.*

To operate chirp an operator has to be able to answer two questions at any moment: **is it serving
users, and what is it doing?** This section states what the product must expose so those questions can
be answered — a health and readiness signal, operational logs, serving metrics, and an audit trail of
operator actions. Each is stated as a requirement on what the product makes visible, not as a choice of
tooling: which log shipper, which metrics backend, which dashboard, and which alert rules are deferred to
later code work. The requirements here describe the *signals*; the status-and-incident posture
(`REQ-OPS-080..099`) builds detection, alerting, and user communication on top of them.

Three things anchor this section. First, chirp already answers a liveness probe (`/healthz` at `c805525`)
that reports only that the process is running; the release needs a **readiness** meaning that reflects
whether the product can serve users — including that its durable store is reachable
(`REQ-PROD-039`, `REQ-DOM-115`, `REQ-DOM-226`). Second, chirp holds private account data — credentials,
session tokens, verified contact — that is never public (`REQ-DOM-035`); that same rule binds everything
the product logs or emits as telemetry, so operational visibility is never bought with a privacy leak.
Third, the operator has a supported administrative surface (`REQ-PROD-028`, operator journey J9) that can
reach into accounts and content; actions taken through it must leave a record.

> **Note for the synthesis fold (issue #23):** the open follow-up #23 (exercise and harden the native
> install path) touches operational visibility — exercising that path is where the readiness signal, the
> logs, and the metrics below are first confirmed wired and observable in a real deployment. It is
> referenced here so the `existing_followups` fold can place it against the operational-visibility
> requirements it exercises, rather than being re-filed as a new requirement.

#### Health and readiness

The product must distinguish **liveness** ("the process is running") from **readiness** ("the process can
serve users"). An orchestrator uses liveness to decide whether to restart a crashed process and readiness
to decide whether to send it traffic; conflating them either kills a process that is merely waiting on a
recoverable dependency, or sends users to a process that cannot serve them. Because chirp's
authoritative state now lives in a durable store distinct from the serving process, a process that cannot
reach that store is running but not ready — and readiness must say so.

#### Structured operational logs

The product must record what it is doing in a form an operator can read and search after the fact,
covering the events needed to explain normal operation and to investigate a failure — and it must do so
without ever writing private account data. Logs are the operator's primary account of "what happened"
when something goes wrong.

#### Serving metrics

The product must expose enough quantitative signal to tell, at a glance and over time, whether it is
serving users well: how much traffic it is handling, how much of that is failing, how slow it is, and how
close it is to overload. These are the signals that make "the product is degraded" something an operator
(and the status-and-incident posture) can detect rather than infer from user complaints.

#### Operator and administrative audit trail

Because the supported administrative surface (`REQ-PROD-028`) can assist users, remove content, and action
accounts, the product must record who did what, to what, and when. An audit trail makes operator action
accountable and reviewable, and it is held to the same no-secrets/no-PII rule as the logs.

---

#### Requirements: `REQ-OPS-060..079`

##### REQ-OPS-060 — Readiness is a distinct signal meaning "serving users"
- **statement**: The product exposes a readiness signal, distinct from liveness, whose meaning is that the
  product can serve users — not merely that the process is running. An operator or orchestrator
  can obtain readiness separately from liveness.
- **acceptance_criteria**: With the product fully able to serve, the readiness signal reports ready. With
  the product running but unable to serve its core loop, the readiness signal reports not-ready while the
  liveness signal still reports alive; the two are separately observable and can disagree.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-039, REQ-DOM-226

##### REQ-OPS-061 — Readiness reflects durable-store reachability
- **statement**: The readiness signal reports not-ready when the product's durable store is unreachable
  from the serving process, and ready when the store is reachable and the product can serve. Readiness is
  not satisfied by process liveness alone.
- **acceptance_criteria**: Make the durable store unreachable to a running serving process; the readiness
  signal reports not-ready. Restore reachability so the product can serve; the readiness signal reports
  ready.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-039, REQ-DOM-115

##### REQ-OPS-062 — Liveness reports process health only, not dependency health
- **statement**: The liveness signal reports whether the serving process itself is alive and able to make
  progress; it does not report not-alive solely because a recoverable dependency (such as the durable
  store) is temporarily unreachable. A dependency outage is reflected in readiness, not liveness, so an
  orchestrator restarts a genuinely crashed process but does not kill one that is alive and waiting on its
  store.
- **acceptance_criteria**: With the durable store unreachable but the serving process otherwise healthy,
  the liveness signal reports alive while the readiness signal reports not-ready. Liveness reports
  not-alive only when the process itself cannot make progress.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-226

##### REQ-OPS-063 — The health/readiness endpoint is unauthenticated and exposes no private data
- **statement**: The health and readiness signal can be queried without an account or session, so an
  operator or orchestrator can reach it, and its response discloses no private account data and no
  internal secret — it conveys serving status, not the contents of the system.
- **acceptance_criteria**: Query the health/readiness endpoint with no session; it responds with the
  serving status. Inspect the response; it contains no credential, session token, verified-contact value,
  or other private account data, and no secret such as a connection string.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-DOM-226, REQ-DOM-035

##### REQ-OPS-064 — Readiness is machine-consumable
- **statement**: The readiness signal is expressed so that an orchestrator or load balancer can
  mechanically distinguish ready from not-ready without interpreting prose — a decidable ready/not-ready
  outcome an automated caller can branch on.
- **acceptance_criteria**: Inspect the readiness signal while ready and while not-ready; in each state the
  signal carries a machine-distinguishable indication of the state (such that an automated caller routes
  traffic only when ready) without requiring a human to read a message.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-039

##### REQ-OPS-065 — Readiness tracks current state and recovers without a restart
- **statement**: The readiness signal reflects the product's current ability to serve, continuously: when
  the condition that made it not-ready clears (for example the durable store becomes reachable again), the
  signal returns to ready on its own, without restarting or redeploying the serving process.
- **acceptance_criteria**: Drive readiness to not-ready by making the durable store unreachable, then
  restore reachability without restarting the process; the readiness signal returns to ready on its own.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-039

##### REQ-OPS-066 — Operational events are recorded in a structured, parseable form
- **statement**: The product records its operational events as structured, machine-parseable records with
  consistent fields (such as a timestamp, a severity, an event category, and an outcome), not as
  free-form prose, so an operator can search and filter them rather than reading them line by line.
- **acceptance_criteria**: Exercise the product and inspect its log output; operational events appear as
  structured records with consistent, named fields that a log tool can parse and filter.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**:

##### REQ-OPS-067 — Logs let one request be followed end to end
- **statement**: Log records carry a correlation identifier that ties together the events of a single
  request or user action, so an operator can follow one action through the product's logs from receipt to
  outcome.
- **acceptance_criteria**: Perform a single user action that touches more than one logged step; the log
  records for that action share a correlation identifier by which they can be selected together and read in
  order.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**:

##### REQ-OPS-068 — Logs cover the events needed to explain operation and failure
- **statement**: The log record covers the operational events an operator needs to answer "what is it
  doing, and what went wrong": request handling and its outcome, errors and failures with a cause
  category, process start and stop, changes in durable-store connectivity, and authentication outcomes
  (success and failure) without the secret material involved.
- **acceptance_criteria**: Provoke, in a running product, a handled request, an error, a process restart, a
  loss and recovery of durable-store connectivity, and a failed and a successful sign-in; each is present
  in the logs as a distinct, categorized event with enough context to tell what happened.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-069 — Logs never contain credentials or private account data
- **statement**: Credentials, session tokens, verified-contact values, and any other private account data
  (`REQ-DOM-035`) are never written to the product's logs, in any field, at any severity. Operational
  visibility is never obtained at the cost of exposing private data in the logs.
- **acceptance_criteria**: Exercise sign-up, sign-in, recovery, and session use so that private data flows
  through the product, then inspect the logs produced; no credential, session token, verified-contact
  value, or other private account field appears in any log record. References to an account in a log use a
  non-secret identifier, not its private data.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-DOM-035

##### REQ-OPS-070 — Severity is applied consistently
- **statement**: Log records carry a severity that is applied consistently, so an operator can separate
  normal operation from warnings and errors by filtering on severity rather than by reading every line.
- **acceptance_criteria**: Inspect logs from a period that includes both normal operation and at least one
  error; normal events and error events carry distinct severities consistent with their nature, and
  filtering to error severity surfaces the error events.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**:

##### REQ-OPS-071 — Errors are logged with enough context to investigate
- **statement**: When the product logs an error it includes enough diagnostic context — what operation
  failed, a cause category, and identifiers needed to locate the affected request or resource — that an
  operator can investigate without reproducing the failure, subject always to the no-secrets/no-PII rule
  (`REQ-OPS-069`).
- **acceptance_criteria**: Provoke a representative failure; its log record names the failing operation and
  a cause category and carries a correlation or resource identifier sufficient to investigate, and contains
  no private account data or secret.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**: REQ-DOM-035

##### REQ-OPS-072 — The product exposes request-rate and error-rate metrics
- **statement**: The product exposes serving metrics that report the rate of requests it is handling and
  the rate that are failing, broken down enough to see which class of surface is affected (for example
  reads versus writes, or by outcome category). Metric labels carry no private account data.
- **acceptance_criteria**: Generate a known mix of successful and failing requests; the metrics reflect the
  request rate and the error rate, distinguishable by surface class or outcome. Inspect the metric labels;
  they contain no handle-linked private data, credential, or contact value.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040, REQ-DOM-035

##### REQ-OPS-073 — Request latency is observable as a distribution
- **statement**: The product exposes request-handling latency as a distribution (not only an average), so
  that a product which is up but slow — tail latency degrading while the mean looks acceptable — is
  visible.
- **acceptance_criteria**: Drive traffic including some slow requests; the latency metric reflects the
  spread of response times (for example percentiles or buckets), not a single averaged value, and the slow
  requests are visible in the distribution.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

##### REQ-OPS-074 — A saturation/headroom signal is observable
- **statement**: The product exposes a saturation or headroom signal — a measure of how close it is to its
  serving capacity, such as in-flight work against available capacity — so that approaching overload is
  detectable before it turns into errors or timeouts.
- **acceptance_criteria**: Drive load toward the product's capacity; the saturation signal rises as
  headroom shrinks, giving an observable indication of approaching overload ahead of widespread failures.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

##### REQ-OPS-075 — A serving/availability signal detects a degraded product
- **statement**: The product exposes an availability or serving signal sufficient to detect that it is
  degraded or not serving its core loop — the signal on which the status-and-incident posture
  (`REQ-OPS-080..099`) builds incident detection. It makes "the product is not serving users" an
  observable fact rather than an inference from complaints.
- **acceptance_criteria**: Put the product into a degraded state in which the core loop does not work for
  users; the serving/availability signal reflects the degradation. Return it to serving; the signal
  reflects recovery. The signal is suitable to drive automated detection.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-076 — Durable-store interaction is observable
- **statement**: Because serving now depends on the durable store, the product exposes the health of its
  interaction with that store — the rate of store errors and the latency of store operations — so an
  operator can tell a store problem from an application problem.
- **acceptance_criteria**: Induce elevated store latency or store errors; the product's store-interaction
  metrics reflect it, distinguishably from application-level request errors.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-DOM-115

##### REQ-OPS-077 — Administrative actions are recorded in an audit trail
- **statement**: Every action taken through the supported administrative surface (`REQ-PROD-028`) —
  assisting a user's recovery, removing content, and disabling or actioning an account — is recorded in an
  audit trail that captures the acting operator/administrator identity, the action taken, the target
  account or content, and the time.
- **acceptance_criteria**: Through the supported administrative surface, assist a user's recovery, remove a
  named chirp, and action an account; each action produces an audit entry naming who acted, what they did,
  which account or content was affected, and when. An administrative action that reaches accounts or
  content leaves no untraced path.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-028

##### REQ-OPS-078 — Audit records are durable and not silently editable
- **statement**: Audit-trail entries are durable — they survive restart and redeploy like the rest of the
  product's authoritative state — and are not silently editable or erasable through the product's ordinary
  surfaces, so the trail remains trustworthy when it is reviewed after the fact.
- **acceptance_criteria**: Produce audit entries, then restart or redeploy the product; the entries are
  still present and unchanged. No ordinary product surface offers a way to alter or delete an existing
  audit entry without trace.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-028

##### REQ-OPS-079 — The audit trail carries no secrets or private data
- **statement**: The audit trail records who acted, what they did, and on which account or content using
  stable, non-secret identifiers, and never stores credentials, session tokens, or verified-contact values
  (`REQ-DOM-035`). The same no-secrets/no-PII rule that binds the logs binds the audit trail.
- **acceptance_criteria**: Inspect audit entries produced by representative administrative actions; each
  names the actor, action, and target by non-secret identifier and contains no credential, session token,
  or verified-contact value.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-DOM-035

---

## status_and_incident

*Status and incident.*


Observability (`REQ-OPS-060..079`) gives an operator the signals to see that chirp is degraded. This
section states what has to happen next: that a degradation visible in those signals is **noticed and acted
on**, that **users are told** when the product is down or going down for maintenance, and that the product
owes a person a usable experience while it is impaired. It also states, as an operated commitment, what
"reasonably available" (`REQ-PROD-040`) means for this product in a number someone can measure against.

The posture here is sized for what chirp is: a small hosted service run by the people who build it
(operator journey J9), not a large product with a staffed on-call rotation. So the requirements ask for a
detection path, a named person who is responsible when something fires, a way to reach users that does not
itself go down with the product, and a stated availability target — and they stop there. The incident
*tooling* (which alerting backend, which status-page service), the paging schedule, and the written
runbooks are deferred to later code work; this section states the requirements they have to satisfy.

Three boundaries keep this section from redrawing its neighbours. Incident **detection** builds directly on
the observability serving/availability signal (`REQ-OPS-075`) and the request, saturation, and
store-interaction metrics (`REQ-OPS-072`, `REQ-OPS-074`, `REQ-OPS-076`) — it does not define new signals.
The **user-facing error and degraded screens** — what a person actually sees when an action fails or a list
will not load — are the experience design's (`REQ-UX-*`, and the product-level `REQ-PROD-033` and
`REQ-PROD-034`); this section requires that an incident lands the user *in* those designed states rather
than in a blank page or a crash, and does not specify the states themselves. The **data** recovery-point
and recovery-time targets belong to backups and recovery (`REQ-OPS-020..039`); the time target here is
time-to-restore-serving during an incident, which is a different thing from restoring lost data.

#### Incident detection and response

A product is not being operated if the first notice of an outage is a user complaint. The serving signal
that observability exposes has to be watched, and a degradation in it has to reach a person who is
responsible for acting. For a product this size that means a detection path from the signals to an alert, a
light severity split so the response is proportionate to what broke, and a named expectation of who
responds — not an organisation chart.

#### Telling users: status and maintenance communication

When chirp is down or degraded, a person trying to use it should be able to find out that it is chirp that
is affected and not their own connection, without guessing. That calls for a status surface or an
in-product notice — and, critically, one that does not travel down the same path that just failed, because
a status page served by the same process that is down tells a user nothing. The same surface carries
advance notice of planned maintenance, and reflects recovery when service returns. Whatever it shows is
held to the product's no-secrets/no-private-data rule (`REQ-DOM-035`), the same rule that binds the logs
and the audit trail.

#### The degraded-state experience boundary

What a person sees on a failed action or an unreachable list — the explained error, the preserved draft,
the routed-to-sign-in on an expired session — is designed by the experience area (`REQ-UX-*`,
`REQ-PROD-033`, `REQ-PROD-034`). The operational requirement is the join: when a dependency the product
needs is unavailable in production (for example the durable store, per `REQ-DOM-115`), the operated product
fails *into* those designed states rather than into a blank screen or a process crash, and contains a
partial failure to the capability that is actually affected where its design allows.

#### Availability as an operated commitment

"Reasonably available" (`REQ-PROD-040`) is only testable if it is a number. This product states an
availability target proportionate to a small hosted service, measures it from the serving/availability
signal (`REQ-OPS-075`) over a stated window rather than asserting it, and accounts for announced
maintenance so a routine deploy is not counted as an unplanned outage. Incident response aims to restore
serving within a stated time that is consistent with that target.

---

#### Requirements: `REQ-OPS-080..099`

##### REQ-OPS-080 — A serving degradation is detected, not waited for
- **statement**: The product's serving/availability signal (`REQ-OPS-075`) is watched in production so that
  a degradation — the product not serving its core loop, or serving it with elevated errors — is detected
  as an operational event and does not depend on a user reporting it. Detection reaches a responsible
  operator (`REQ-OPS-083`).
- **acceptance_criteria**: Put the product into a state where the core loop does not work for users; within
  a bounded time the configured detection raises a notification to the responsible operator, without a user
  having had to report the problem first. Return the product to serving; no further alert for that incident
  is raised.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-081 — Detection is driven by the observability signals
- **statement**: Incident detection is built on the observability signals — the serving/availability signal
  (`REQ-OPS-075`), the request- and error-rate metrics (`REQ-OPS-072`), the saturation/headroom signal
  (`REQ-OPS-074`), and the durable-store interaction health (`REQ-OPS-076`) — rather than on a person
  watching a screen. The conditions that constitute an incident are expressed against those signals.
- **acceptance_criteria**: Inspect what drives detection; each alerting condition names an observability
  signal and a threshold or state on it. Induce an elevated error rate, approaching saturation, and a
  store-interaction fault in turn; each is capable of being surfaced as a detected condition from its
  signal.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-082 — Incidents carry a proportionate severity
- **statement**: A detected incident is classified by a small severity scheme that at least distinguishes
  "the product is not serving its core loop" (a major incident) from "the product is degraded or partially
  impaired but still serving" (a minor incident), so that response and communication are proportionate to
  what is actually broken. The scheme is kept to the scale of a small hosted product.
- **acceptance_criteria**: Find the stated severity scheme; it defines at least a major and a minor level
  with a criterion for each expressed against the serving signal. Classify a total-outage case and a
  partial-degradation case; each maps to the intended level.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**: REQ-PROD-040

##### REQ-OPS-083 — A responsible operator is identified for incident response
- **statement**: There is a stated expectation of who is responsible for responding when an incident is
  detected — a role, not a staffed rotation — so that a fired alert has an owner rather than reaching no
  one. This is the operator of operator journey J9.
- **acceptance_criteria**: Find the stated responsibility; it names the role accountable for acknowledging
  and acting on a detected incident. A detected major incident has a defined responder; no detected incident
  is left with no one responsible for it.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-084 — An incident has a defined response path
- **statement**: A detected incident has a defined response expectation — acknowledge, investigate using the
  observability signals and logs, communicate to users per the status requirements, and resolve — recorded
  so the responder follows a known path rather than improvising. The detailed step-by-step runbooks are
  deferred to later operational work; this requires that the response path exist and be written down.
- **acceptance_criteria**: Find the recorded incident-response expectation; it covers acknowledgement,
  investigation against the observability signals, user communication, and resolution. Walk a representative
  incident through it; each step has a defined owner and outcome.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**: REQ-PROD-040

##### REQ-OPS-085 — Users can learn that the product is down or degraded
- **statement**: When the product is unavailable or degraded, a person has a way to learn that chirp itself
  is affected — a status surface or an in-product notice — rather than being left to infer it from a
  failed action or a spinner. The notice states, at least, that the product is experiencing a problem.
- **acceptance_criteria**: With the product in an outage or degraded state, a user consulting the status
  surface or in-product notice sees that chirp is affected and is not left only with an unexplained failure.
  With the product serving normally, the surface shows normal operation.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-086 — The status surface does not depend on the failed product
- **statement**: The surface users consult to learn of an outage is reachable when the product itself is
  down — it does not depend on the serving path, the durable store, or the process that may be the thing
  that has failed. A status page that goes down with the product conveys nothing at the moment it is needed.
- **acceptance_criteria**: Take the serving product fully offline, including its durable store; the status
  surface is still reachable and reports the product as unavailable. The surface's availability does not
  require the product's serving path to be up.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-087 — Planned maintenance is announced in advance
- **statement**: When the operator plans maintenance that will make the product unavailable or degraded,
  users are given advance notice through the same status surface or in-product notice, stating the expected
  window, so a planned interruption is distinguishable to a user from an unexpected outage.
- **acceptance_criteria**: Schedule a maintenance window and publish the notice; before the window a user
  consulting the status surface sees the planned maintenance and its expected timing. During the window the
  interruption is presented as planned, not as an unexplained failure.
- **verification_method**: manual_user_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

##### REQ-OPS-088 — Status and incident communication carries no private or secret data
- **statement**: Nothing published on the status surface, in an in-product outage notice, or in a
  maintenance announcement contains private account data or secret material — no credential, session token,
  verified-contact value (`REQ-DOM-035`), nor internal secret such as a connection string or a stack trace
  that exposes them. Telling users about an outage never leaks the data the outage is around.
- **acceptance_criteria**: Inspect the content published for a representative outage and a representative
  maintenance notice; it describes product status in terms a user needs and contains no credential, session
  token, verified-contact value, connection string, or raw internal error detail that discloses private or
  secret data.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-DOM-035

##### REQ-OPS-089 — Recovery is communicated when service returns
- **statement**: When a communicated incident is resolved and the product is serving again, the status
  surface or in-product notice reflects the recovery, so a user who saw the outage can tell that it is over
  rather than being left unsure whether the problem persists.
- **acceptance_criteria**: Communicate an outage, then resolve it and restore serving; the status surface
  updates to show the product operating normally. A user who saw the outage notice can see that service has
  recovered.
- **verification_method**: manual_user_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

##### REQ-OPS-090 — The product fails into its designed degraded states
- **statement**: When a dependency the product needs is unavailable in production — in particular the
  durable store (`REQ-DOM-115`) — the operated product presents the user the degraded and error experiences
  designed by the experience area (`REQ-UX-*`, `REQ-PROD-033`, `REQ-PROD-034`): an explained error, a
  preserved in-progress draft, a routed-to-sign-in on an expired session. This requirement is the
  operational join to those states; it does not define them.
- **acceptance_criteria**: In production, make the durable store unreachable while the serving process stays
  up; a user exercising the core loop meets an explained degraded or error state, not a blank page or a
  crashed process. The states a user sees are the ones the experience design specifies.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-PROD-033, REQ-PROD-034

##### REQ-OPS-091 — A partial failure is contained to the affected capability
- **statement**: When one capability is degraded rather than the whole product — for example the durable
  store is slow but reachable — the parts of the product that can still serve continue to do so, to the
  extent the product's design allows, rather than a single fault turning into a total outage.
- **acceptance_criteria**: Induce degradation in one capability (for example elevated store latency); the
  product continues to serve the surfaces that do not depend on the degraded capability, and the impact a
  user sees is scoped to the affected capability rather than a blanket failure.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040, REQ-DOM-115

##### REQ-OPS-092 — A stated, proportionate availability target
- **statement**: The product states an availability target that defines what "reasonably available"
  (`REQ-PROD-040`) means for it — a figure proportionate to a small hosted service, expressed over a stated
  window (for example a monthly serving-availability target). The target is written down, not left implicit.
- **acceptance_criteria**: Find the stated availability target; it gives a specific availability figure and
  the window it is measured over, and it is proportionate to a small hosted product rather than an
  enterprise commitment.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-093 — Availability is measured from the serving signal, not asserted
- **statement**: The availability target is measured in production from the serving/availability signal
  (`REQ-OPS-075`) over the stated window, so that whether the product met its target is a measured fact
  rather than a claim. Unavailability is counted when the serving signal shows the product not serving its
  core loop.
- **acceptance_criteria**: Over a measurement window that includes at least one serving interruption,
  compute availability from the serving signal; the result reflects the observed interruption and can be
  compared against the stated target to say whether it was met.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

##### REQ-OPS-094 — Announced maintenance is accounted for in availability
- **statement**: Availability measurement distinguishes announced planned maintenance (`REQ-OPS-087`) from
  unplanned outage, so that routine, pre-announced deploys and maintenance do not register as unplanned
  unavailability against the target.
- **acceptance_criteria**: Take the product down during an announced maintenance window and, separately,
  during an unplanned incident; the availability accounting attributes the first to planned maintenance and
  the second to unplanned unavailability, per the stated rule.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**: REQ-PROD-040

##### REQ-OPS-095 — A stated time-to-restore-serving objective for incidents
- **statement**: Incident response works to a stated objective for how quickly serving is restored after a
  detected outage, consistent with the availability target (`REQ-OPS-092`). This is time-to-restore
  serving; it is distinct from the data recovery-time objective owned by backups and recovery
  (`REQ-OPS-020..039`), which concerns restoring lost data rather than resuming service.
- **acceptance_criteria**: Find the stated time-to-restore-serving objective; it gives a target duration
  from detection to restored serving and is consistent with the availability target. For a representative
  resolved incident, the time from detection to restored serving is measurable against it.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

---

## existing_followups

Three open board follow-ups predate this design and are non-release-gating hardening or
polish. They are folded here into the operational-readiness concern each belongs to and are
**not** minted as new `REQ-OPS-*`; where an existing requirement already covers the substance,
that requirement is named. None gates the release.

| Issue | Summary | Folded into | Covered in substance by | Release-gating |
|-------|---------|-------------|--------------------------|----------------|
| #23 | Exercise and harden the native install path | `observability` | `REQ-OPS-060`, `REQ-OPS-061`, `REQ-OPS-066`, `REQ-OPS-068`, `REQ-OPS-072`, `REQ-OPS-075` — exercising the install path is where the readiness signal, the structured logs, and the serving metrics are first confirmed wired and observable in a real deployment | No |
| #24 | Least-privilege delivery RBAC + supply-chain hardening | `limits_and_quotas` | Adjacent to the operational-security posture: the operator/administrative audit trail (`REQ-OPS-077`, `REQ-OPS-079`) records privileged action, but least-privilege RBAC on the delivery/control plane is a distinct, privileged-actor access concern not fully covered by any release-gating `REQ-OPS-*`. It is control-plane access control, distinct in subject from the public, client-facing rate and volume limits (`REQ-OPS-040..048`), and remains issue #24 as hardening | No |
| #25 | Containerize follow-ups (non-blocking polish) | `durable_data` | Relates to the operated-deploy posture — a running deployment loses no user data to restart, redeploy, or upgrade (`REQ-OPS-001`, `REQ-OPS-003`, `REQ-OPS-004`); the containerization items are non-blocking polish on the deploy artifact, not a new durability requirement | No |

**Placement rationale.** #23 is folded into `observability` because its substance — proving the
operational signals are wired in a real deployment — is exactly what the observability requirements
assert. #24 is folded into `limits_and_quotas` as its operational-security neighbour: it is an
access-and-limits matter, but it bounds privileged control-plane actors rather than public clients,
so it sits beside the public limits without being restated as one. #25 is folded into `durable_data`
against the operated-deploy posture it polishes. Each issue appears exactly once above, is covered by
or adjacent to a named existing requirement, and is not duplicated as a new `REQ-OPS-*`.


---

## requirements_and_acceptance_criteria

This is the consolidated `REQ-OPS-*` register: the verbatim union of the five concern sections' requirements, in id order. Each requirement is also stated in its concern section above; this register is the single flat index of the operational-readiness requirements and is followed by the OPS traceability table. Every requirement carries a `verification_method` from the vocabulary (`manual_user_check`, `automated_test`, `inspection`, `operational_check`), a `release_gating` flag, and `traces_to` the product/domain requirement it refines (empty only for an operational-internal requirement).

#### REQ-OPS-001 — A running deployment loses no user data to restart, update, or redeploy
- **statement**: An operated chirp deployment does not lose any user data — Accounts, Chirps, Follows, Likes, Sessions — to the normal operations of running the service: restarting the serving process, updating it, and redeploying it are routine operations after which all user data is present and unchanged. This is the operational standing of the domain model's survival requirement, held as a property of the operated service rather than of a single fresh start.
- **acceptance_criteria**: On an operated deployment, create Accounts and post Chirps, Follows, and Likes; perform each of a process restart, a redeploy, and an update as an operator would; after each, every Account and its Chirps, Follows, and Likes are present and unchanged, with no manual intervention to recover them.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-100

#### REQ-OPS-002 — Authoritative state is operated in a durable store distinct from the serving process
- **statement**: The deployment runs the product's authoritative state in a durable store that is a separate, persistent component from the serving process, with its own lifecycle; the serving process holds no authoritative state that exists only in its memory. The in-memory backing of the `ChirpRepository` seam at `Nousix-LLC/nswe-demo @ c805525` does not meet this and is the blocker this posture closes.
- **acceptance_criteria**: Inspect the deployment: the durable store is a component whose lifecycle is independent of the serving process (stopping or replacing the serving process does not stop or reset the store). Confirm `REQ-DOM-113/114` hold against the operated product. No deployment configuration keeps authoritative state solely in serving-process memory.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-113, REQ-DOM-114

#### REQ-OPS-003 — Replacing the serving process is a routine operation that touches no user data
- **statement**: An operator can replace or roll the serving process to a fresh instance, with no shared process memory, as a normal operation; all user data remains intact because it lives in the durable store, not the process.
- **acceptance_criteria**: On a populated deployment, replace the serving process with a fresh instance without touching the durable store; afterward all Accounts, Chirps, Follows, Likes, and active Sessions are intact, demonstrating the authoritative state did not live in the replaced process.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-113

#### REQ-OPS-004 — An operated upgrade preserves existing user data
- **statement**: Performing an upgrade in production — including a redeploy that changes how the data is shaped — preserves the user data that existed before it: afterward the prior Accounts, Chirps, Follows, and Likes are all present and still satisfy their invariants. The migration mechanics are the code work's; the preservation property across an operated upgrade is the bar's.
- **acceptance_criteria**: On a populated deployment, perform an upgrade that changes the data's shape; afterward the Accounts, Chirps, Follows, and Likes created before the upgrade are all present and their references still resolve. No data created before the upgrade is lost or corrupted by it.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-118

#### REQ-OPS-005 — An acknowledged write survives an immediate restart in production
- **statement**: In the operated product, any create or change the product has confirmed to a person — a posted Chirp, a Follow or unfollow, a Like or unlike, a profile edit, an account deletion — is durable at the moment it is acknowledged, so a restart or process replacement happening immediately after the success response does not lose it. Durability is a correctness property of the acknowledgement, not a tendency observed over time.
- **acceptance_criteria**: On an operated deployment, perform a write and receive the product's success response; with no further action, restart or replace the serving process; the written state is present afterward. Repeated across each write type, no acknowledged write is lost by a restart in the window immediately following its acknowledgement.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-109

#### REQ-OPS-006 — Domain invariants hold after an operational restart
- **statement**: After the serving process is restarted or replaced in production, the domain model's invariants still hold against the durable data: every reference resolves, ids are unique and never reused, each Account's follower/following counts and each Chirp's like count equal their current edges, and any applied account deletion stays applied. A restart never introduces a dangling reference, a reused id, or a count that disagrees with its edges.
- **acceptance_criteria**: On a populated deployment, record a sample of references, ids, and counts; restart the serving process; traverse Chirps, Replies, Follows, and Likes — every referenced Account and Chirp resolves, no recorded id has moved or been reused, and the counts equal the current edges (add and remove a Follow and a Like after the restart and confirm the counts track by exactly one each way).
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-107, REQ-DOM-110, REQ-DOM-111, REQ-DOM-112

#### REQ-OPS-007 — Domain invariants hold after a restore from backup (the posture backups build on)
- **statement**: After a restore from backup, the durable data satisfies the same invariants required after a restart (`REQ-OPS-006`): references resolve, ids are unchanged and never reused, counts equal their edges, and applied deletions remain applied. A restore that brings rows back but breaks these invariants has not recovered a correct product. This is the durability posture the backups-and-recovery section's restore-correctness requirements (`REQ-OPS-020..039`) are written against.
- **acceptance_criteria**: On a store restored from a backup of a populated deployment, traverse Chirps, Replies, Follows, and Likes — every reference resolves, recorded ids are unchanged and unreused, counts equal their edges, and any account deleted before the backup is still deleted with no dangling reference. A failing invariant is reported as an unsuccessful restore, never silently accepted.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-110, REQ-DOM-116

#### REQ-OPS-008 — An interrupted write never leaves half-applied state visible
- **statement**: A restart, crash, or process replacement that happens while a write is in progress leaves the durable data in a consistent state: the operation is observable afterward as either fully applied or not applied, never as a half-applied change with a dangling reference or a count that disagrees with its edges. Account deletion in particular is all-or-nothing across an interruption.
- **acceptance_criteria**: Interrupt the serving process during representative writes (posting a Chirp, following, deleting an account that has Chirps, Follows, and Likes); after recovery, each interrupted operation is observably either fully applied or fully absent, and no partially applied state (a Chirp without a resolvable author, a half-deleted Account with dangling references, a count out of step with its edges) is observable.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-110, REQ-DOM-112

#### REQ-OPS-009 — Durability requires the store to be operated as a reachable, persistent component
- **statement**: Because the serving process depends on the durable store to serve, durability in production requires the store to be run as a reachable, persistent component for the life of the deployment: a serving process that cannot reach its durable state cannot serve, and a deployment that does not keep its store reachable and persistent does not meet the durability posture. This section states the dependency as a durability requirement; the readiness *signal* that reports store reachability is the observability section's (`REQ-OPS-060..079`), building on `REQ-DOM-115` / `REQ-PROD-039`.
- **acceptance_criteria**: Confirm the deployment runs the durable store as a persistent, reachable component independent of the serving process. With the store made unreachable to the serving process, the product does not serve user writes or reads from process memory as if durable (it does not silently fall back to a non-durable store). The readiness signal's behaviour under this condition is verified by the observability section.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-039, REQ-DOM-115

#### REQ-OPS-010 — Live user data is retained for the life of the account
- **statement**: While an account exists, its user data — the Account and its profile, its Chirps, its Follows, and its Likes — is retained in full: the product does not silently expire, age out, or delete a live account's content as a side effect of operating the service. Any retention limit the product chooses to place on a live account's content is stated, not silent.
- **acceptance_criteria**: Create an account with Chirps, Follows, and Likes; over a sustained operating period with ordinary restarts and redeploys, the account's content remains present and complete without the holder re-creating it. If any component of a live account's data is subject to an age-out, that limit is found stated in the product's retention posture (`REQ-OPS-016`) rather than observed as an unexplained loss.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037

#### REQ-OPS-011 — Deleted-account data is removed within a stated, bounded window
- **statement**: When an account is deleted, its data is removed or de-identified per the stated deletion outcome (`REQ-DOM-037`, `REQ-DOM-112`) within a stated, bounded retention window after the deletion is applied, rather than being retained indefinitely. The product states the window; after it elapses, the deleted account's data is not retained in the live durable store.
- **acceptance_criteria**: Find the product's stated deleted-account retention window. Delete an account that has Chirps, Follows, and Likes; after the deletion is applied the stated outcome holds immediately (the account cannot sign in, no dangling reference remains per `REQ-DOM-112`), and after the stated window the deleted account's data is no longer present in the live durable store. The behaviour matches the stated window.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-037, REQ-DOM-112

#### REQ-OPS-012 — Backups do not retain deleted-account data beyond a stated bound
- **statement**: Deleted-account data does not persist indefinitely in backups: a stated maximum backup-retention bound limits how long a deleted account's data can remain recoverable in backups, after which the backups holding it have aged out. This section fixes that such a bound exists and is consistent with the deletion posture; the backup cadence, retention schedule, and ageing mechanics are the backups-and-recovery section's (`REQ-OPS-020..039`), which builds on this.
- **acceptance_criteria**: Find the product's stated maximum backup-retention bound and confirm it is consistent with the deleted-account retention posture (`REQ-OPS-011`). Confirm that no backup older than that bound is retained, so deleted-account data cannot be recovered from backups indefinitely. The concrete cadence and schedule are verified against the backups-and-recovery requirements.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-DOM-037, REQ-PROD-038

#### REQ-OPS-013 — Retention and export cover the same per-account set
- **statement**: The data the product retains for a live account is the data it can hand that person back: while an account exists, its retained profile, full set of Chirps, and full set of followed Accounts are enumerable for the holder's own export (`REQ-DOM-117`). A person cannot be told their data is kept while being unable to obtain it, and export reflects the committed durable state at the time of the request.
- **acceptance_criteria**: For a signed-in account with Chirps and Follows, the data the product retains (per `REQ-OPS-010`) and the data a self-service export yields (per `REQ-DOM-117`) are the same set — the account's profile, its complete Chirps, and its complete followed Accounts — matching the committed durable state. No retained category of a person's own data is absent from what they can export.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-DOM-117, REQ-PROD-019

#### REQ-OPS-014 — Sessions are retained only for their bounded lifetime
- **statement**: Active Sessions are retained durably so a signed-in person is not silently signed out by a restart or redeploy (`REQ-DOM-106`), but only until the Session ends (sign-out) or passes its lifetime (`REQ-DOM-030`, `REQ-DOM-031`); an ended or expired Session is not retained as usable and is never revived by a restart or a restore.
- **acceptance_criteria**: Sign in to obtain a Session; restart or redeploy the service — a request carrying that Session's token is still accepted as the same account. Separately, end a Session (or let it pass its lifetime), then restart and restore from a backup taken while it was active — a request carrying that token is refused in both cases, not revived.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-106, REQ-DOM-030, REQ-DOM-031

#### REQ-OPS-015 — Private identity data is retained durably, kept non-public, and removed with the account
- **statement**: The account-associated Credential and Verified contact are retained durably as part of the Account for the life of the account so a person can still sign in and still recover after a restart (`REQ-DOM-102`), are never exposed in any public response, and are removed or de-identified with the account under the stated deletion outcome and within the deleted-account retention window (`REQ-OPS-011`). Their structure and handling are the identity/security design's (`REQ-ID-*`); this requirement is that they persist, stay non-public, and are not retained past the account.
- **acceptance_criteria**: After a restart and a restore, a person can still sign in with their Credential and the recovery path still recognises their Verified contact, and neither value ever appears in any public response. After an account is deleted and the retention window elapses, the deleted account's Credential and Verified contact are no longer retained in the live durable store.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-102, REQ-DOM-037

#### REQ-OPS-016 — The retention and deletion posture is stated where users and operators can find it
- **statement**: The product states, in a place a user and an operator can find, what user data it keeps, for how long, and what account deletion removes and when — covering live-account retention (`REQ-OPS-010`), the deleted-account window (`REQ-OPS-011`), the backup bound (`REQ-OPS-012`), and session and private-identity-data handling (`REQ-OPS-014`, `REQ-OPS-015`). A releasable product does not leave its retention and deletion promise implicit.
- **acceptance_criteria**: Locate the product's stated data-retention and deletion posture as a user/operator could. It names what is retained, the live-account retention, the deleted-account retention window, the backup-retention bound, and session/private-data handling, and these statements match the behaviour verified by `REQ-OPS-010..015`. A discrepancy between the stated posture and the observed behaviour is a defect.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-037

#### REQ-OPS-020 — Durable user data is backed up on a stated, regular cadence
- **statement**: An operated chirp deployment backs up its durable user data — Accounts, Chirps, Follows, Likes — on a stated, regular cadence, as a standing part of operating the service rather than as an occasional manual act. The product states what the cadence is; backups are produced on it without an operator having to remember to take them.
- **acceptance_criteria**: Find the product's stated backup cadence. On an operated deployment with user data, observe over a period spanning several cadence intervals that a backup is produced each interval without manual intervention, and that each produced backup is recorded where an operator can see it was taken. Backups appear on the stated cadence, not ad hoc.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-DOM-116

#### REQ-OPS-021 — Backups are retained for a stated, bounded period
- **statement**: Backups are kept for a stated retention period that bounds how far back an operator can recover, and that period is bounded above so backups do not accumulate indefinitely. The retention period is consistent with the maximum backup-retention bound the durable-data section fixed for deleted-account data (`REQ-OPS-012`): a backup older than the stated period has aged out and is no longer retained.
- **acceptance_criteria**: Find the product's stated backup-retention period and confirm it is consistent with the deleted-account backup bound (`REQ-OPS-012`). On an operated deployment, confirm that backups within the period are available to restore from and that backups older than the period are no longer retained. The oldest available backup is no older than the stated period.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-DOM-037

#### REQ-OPS-022 — Each backup is an internally consistent point-in-time image
- **statement**: A single backup captures the durable user data as of one point in time, consistent within itself: restoring it yields a state in which the model's invariants hold, not a smear of rows captured at different moments with references that no longer resolve. A backup taken while the product is serving does not record a Chirp without its author or a count out of step with its edges.
- **acceptance_criteria**: Take a backup of a populated deployment while writes are occurring; restore it to a separate store and traverse Chirps, Replies, Follows, and Likes — every reference resolves, ids are unchanged and unreused, and counts equal their edges, with no partially captured write visible. A backup that restores to a state violating an invariant is treated as an unusable backup, not accepted.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-116, REQ-DOM-110

#### REQ-OPS-023 — Backups are stored independently of the primary durable store
- **statement**: Backups are held in a location whose loss is independent of the primary durable store, so that the failure a backup protects against — losing the store — does not also take the backups. A backup kept only alongside the data it backs up does not meet this posture.
- **acceptance_criteria**: Inspect where backups are stored relative to the primary durable store; the two do not share a single point of failure such that losing the primary store also loses its backups. Confirm that, with the primary store made unavailable, a retained backup is still reachable to restore from.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038

#### REQ-OPS-024 — A stated recovery-point objective (tolerable data loss)
- **statement**: The product states a recovery-point objective: the maximum amount of recent durable data, measured as a time window of work, that a recovery is permitted to lose in a disaster. The objective is set no tighter than the backup cadence (`REQ-OPS-020`) can actually achieve, so it is a target a recovery can meet rather than an aspiration.
- **acceptance_criteria**: Find the product's stated recovery-point objective and confirm it is consistent with the backup cadence (a recovery-point objective of N cannot be met by a cadence coarser than N). The stated objective names a bounded quantity of tolerable data loss, not "none" without a mechanism that would achieve none.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038

#### REQ-OPS-025 — A stated recovery-time objective (tolerable downtime)
- **statement**: The product states a recovery-time objective: the maximum time the product may be unavailable to users while an operator recovers it from backup after a disaster. The objective is a commitment the restore procedure is built and rehearsed to meet, bounding the downtime a person experiences in a recovery.
- **acceptance_criteria**: Find the product's stated recovery-time objective. It names a bounded recovery duration and is consistent with the availability the product commits to (`REQ-PROD-040`). The objective is the one the restore drill (`REQ-OPS-031`) measures the restore against.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-PROD-040

#### REQ-OPS-026 — A restore is a defined, operator-performable procedure
- **statement**: Recovering the product from a backup is a defined procedure an operator can follow, not an improvisation invented during an incident: the steps to restore the durable store from a retained backup and return the product to service are written down and performable by an operator who did not design the system. The scripting and tooling are the code work's; that a defined, followable procedure exists is the bar's.
- **acceptance_criteria**: Locate the product's stated restore procedure. An operator who did not build the system can follow it to restore a deployment from a retained backup and return it to service, without needing knowledge that is not in the procedure. The procedure names what is restored (Accounts, Chirps, Follows, Likes) and how the product is returned to serving.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-DOM-116

#### REQ-OPS-027 — A restore brings back accounts, chirps, follows, and likes intact and consistent
- **statement**: A restore from a retained backup returns the durable user data intact and consistent: the Accounts, Chirps, Follows, and Likes present at the backup's point in time are all present afterward, and they satisfy the same invariants a restart must satisfy (`REQ-OPS-007`) — references resolve, ids are unchanged and never reused, counts equal their edges, and any account deleted before the backup stays deleted with no dangling reference. A restore that brings rows back but breaks these invariants has not recovered a correct product.
- **acceptance_criteria**: On a store restored from a backup of a populated deployment, every Account, Chirp, Follow, and Like present at the backup point is present; traverse Chirps, Replies, Follows, and Likes — every reference resolves, recorded ids are unchanged and unreused, counts equal their edges, and an account deleted before the backup is still deleted with no dangling reference. A failing invariant is reported as an unsuccessful restore, never silently accepted.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-116, REQ-DOM-110, REQ-DOM-111, REQ-DOM-107

#### REQ-OPS-028 — An exercised restore meets the stated recovery-point and recovery-time objectives
- **statement**: When the restore procedure is exercised, it recovers the product within the stated recovery-time objective (`REQ-OPS-025`) and loses no more than the stated recovery-point objective (`REQ-OPS-024`): the downtime and data loss a real recovery incurs are within the commitments the product has made, not merely asserted by them.
- **acceptance_criteria**: Perform a timed restore from a backup on a representative deployment: measure the elapsed time from starting the restore to the product serving users again — it is within the stated recovery-time objective — and measure the data lost relative to the disaster point — it is within the stated recovery-point objective. A drill that exceeds either objective is a defect in the objective or the procedure, reported rather than ignored.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-038

#### REQ-OPS-029 — A failed deploy can be rolled back to the last working version with no user-data loss
- **statement**: A deploy that fails or ships a broken product can be rolled back to the last working version as a routine operation, and the rollback loses no user data: the Accounts, Chirps, Follows, and Likes present before the deploy are all present after the rollback. Rolling back is a recovery operation an operator can perform, not a reason to lose data.
- **acceptance_criteria**: On a populated deployment, perform a deploy and then roll it back to the prior version as an operator would; afterward the product serves on the prior version and every Account, Chirp, Follow, and Like that existed before the deploy is present and unchanged. The rollback requires no manual reconstruction of user data.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-037, REQ-DOM-118

#### REQ-OPS-030 — A data-damaging deploy can be recovered from by restore
- **statement**: A deploy that damages durable data — corrupting it or losing part of it despite the preserve-existing-data requirement (`REQ-DOM-118`) — can be recovered from by restoring from a backup taken before the deploy, returning the product to a correct state consistent with `REQ-OPS-027`. The preserve-data property is the first line; a restore path for when it is breached is the second.
- **acceptance_criteria**: Simulate a deploy that damages or loses durable data on a populated deployment; recover by restoring from a pre-deploy backup following the restore procedure (`REQ-OPS-026`); afterward the durable data is intact and consistent per `REQ-OPS-027`, recovered to the pre-deploy point within the stated objectives (`REQ-OPS-028`). The product does not depend on the damaging deploy being reversible in place.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-118, REQ-DOM-116

#### REQ-OPS-031 — Backups are periodically proven restorable
- **statement**: Backups are proven restorable on a recurring basis, not merely produced: a restore from a recent backup is exercised periodically against a store where its correctness can be checked, and the exercise verifies the restored data satisfies its invariants (`REQ-OPS-027`) and meets the stated objectives (`REQ-OPS-028`). A backup that has never been restored is not yet known to be a backup.
- **acceptance_criteria**: Find the product's stated restore-drill cadence and the record of its most recent drills. Confirm a restore from a recent backup has been exercised within that cadence, that the drill checked the restored data's invariants and timing, and that a failed drill is recorded and acted on rather than passed over. The drill is recurring, not a one-time pre-release event.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-038, REQ-DOM-116

#### REQ-OPS-032 — A failed or missing backup is surfaced, not silent
- **statement**: When a scheduled backup fails or does not run, that fact is surfaced to operators rather than passing silently, so a deployment cannot drift into having no recent usable backup without anyone knowing. This section requires that the failure is detectable and raised; the alerting signal that carries it to an operator is the observability section's (`REQ-OPS-060..079`), building on the same posture.
- **acceptance_criteria**: Induce a backup failure or a missed scheduled backup on an operated deployment; the condition is surfaced to operators (a visible failure state or alert), not absorbed silently. Confirm that a run of successful backups and a failed backup are distinguishable to an operator. The signal's shape is verified against the observability requirements.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-038

#### REQ-OPS-033 — The backup and recovery posture is stated where an operator can find it
- **statement**: The product states, in a place an operator can find, its backup cadence and retention (`REQ-OPS-020`, `REQ-OPS-021`), its recovery-point and recovery-time objectives (`REQ-OPS-024`, `REQ-OPS-025`), and the restore and rollback procedures (`REQ-OPS-026`, `REQ-OPS-029`). A releasable product that can be operated does not leave its recovery procedure to be reconstructed under incident pressure.
- **acceptance_criteria**: Locate the product's stated backup-and-recovery posture as an operator could. It names the cadence, the retention period, the recovery-point and recovery-time objectives, and the restore and rollback procedures, and these statements match the behaviour verified by `REQ-OPS-020..032`. A discrepancy between the stated posture and the observed behaviour is a defect.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-038

#### REQ-OPS-040 — The public request rate is bounded per client
- **statement**: The product bounds the rate of requests it will accept from a single client, so one client cannot drive the service past what it can serve for everyone else by issuing requests as fast as it can. The threshold, the client-identification basis, and the enforcement point are set operationally; the requirement is that a bound exists and is applied per client rather than only in aggregate.
- **acceptance_criteria**: From a single client, issue requests far faster than a person using the product would; past a point the product stops serving the excess (it refuses or slows further requests) rather than accepting every request without limit, and other clients continue to be served normally meanwhile.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041

#### REQ-OPS-041 — State-creating actions are rate-limited more tightly than reads
- **statement**: The actions that create state — posting a chirp, following an account, liking a chirp — are subject to their own, tighter rate limit than ordinary reads, so a single account cannot flood timelines or follow/like en masse at machine speed. The specific limits are operational; the requirement is that a write-rate bound exists on each state-creating action.
- **acceptance_criteria**: Signed in, perform one state-creating action (for example posting) repeatedly, far faster than a person would; past a point the product refuses or throttles the further writes rather than accepting them without limit. Reading remains possible within its own limit while writes are being throttled.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041

#### REQ-OPS-042 — Created volume per account is bounded over a longer window
- **statement**: Beyond the short-term write rate, the volume of content a single account can create over a longer rolling window is bounded, so an account cannot flood others by sustaining an at-the-limit rate indefinitely. This is the "volume" half of the product's rate-and-volume requirement, distinct from the burst limit of REQ-OPS-041. The window and the cap are operational; the requirement is that a sustained-volume bound exists per account.
- **acceptance_criteria**: With one account, create content continuously at or just under the per-action rate limit for a sustained period; the product eventually bounds the account's further creation for that window rather than allowing unbounded accumulation, and resumes in a later window.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041

#### REQ-OPS-043 — Oversized request bodies are refused at the edge
- **statement**: The product enforces a maximum request body size and refuses a request whose body exceeds it before the body is processed, so a client cannot consume unbounded memory or processing by sending an enormous payload. The cap is consistent with the already-fixed content bounds — a chirp body of 1–280 characters (REQ-DOM-012) and a handle of 1–15 characters (REQ-DOM-002) — and exists to stop payloads far larger than any valid request, not to re-enforce those per-field rules.
- **acceptance_criteria**: Send a request whose body is far larger than any valid chirp or form submission (for example a multi-megabyte body to a write endpoint); the product refuses it with an error and does not attempt to process the oversized body. A normal, within-bounds request to the same endpoint succeeds.
- **verification_method**: automated_test
- **release_gating**: true
- **traces_to**: REQ-PROD-041, REQ-DOM-012, REQ-DOM-002

#### REQ-OPS-044 — List reads are served only as bounded pages
- **statement**: Every endpoint that returns a list enforces the server's maximum page size, so no client can force the product to assemble and return an unbounded result in one response. This is the operational enforcement of the domain requirement that timelines are read a bounded page at a time behind an opaque cursor (REQ-DOM-027, REQ-DOM-218); the operational requirement is that the cap holds on every list surface, not only the timeline.
- **acceptance_criteria**: Request a list with no page size, and request one with a page size well above the server maximum; in both cases the response carries at most the server's maximum number of items, with a continuation cursor when more remain, rather than the full result set.
- **verification_method**: automated_test
- **release_gating**: true
- **traces_to**: REQ-PROD-041, REQ-DOM-218, REQ-DOM-027

#### REQ-OPS-045 — Exceeding a limit refuses or throttles the excess and says so
- **statement**: When any limit in this section is exceeded, the product rejects or throttles the excess rather than serving it without bound, and tells the caller with a typed, machine-readable error carrying a distinct category for being rate- or size-limited, consistent with the error contract (REQ-DOM-220). The caller can tell "you were limited" apart from other failures and can act on it (for example back off and retry later).
- **acceptance_criteria**: Drive any one of the limits above past its threshold; the response is an error whose machine-readable category identifies it as a limit being hit — distinct from a validation error or a not-found — and the excess work is not performed. The error is the typed contract category, not an ad-hoc string.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041, REQ-DOM-220

#### REQ-OPS-046 — The unauthenticated surface is limited too
- **statement**: The request-rate limit applies to the unauthenticated surface — the landing page, sign-up, and sign-in — identified by client rather than by account, so a client cannot escape limiting simply by not signing in, and the pre-sign-in surface cannot be used as an unbounded overwhelm or credential-guessing vector. This bounds request volume on that surface; account-specific abuse handling and credential-attempt lockout belong to the identity design (REQ-ID-*) and are referenced, not owned here.
- **acceptance_criteria**: Without signing in, issue requests to a public, unauthenticated endpoint far faster than a person would; past a point the product refuses or throttles the excess, the same as for an authenticated client.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-041

#### REQ-OPS-047 — Under aggregate overload, the product sheds rather than collapses
- **statement**: When total load exceeds what the product can serve — even from many within-limit clients at once — the product sheds or throttles the excess and keeps serving the traffic it can, rather than failing wholesale for everyone. This protects the product's reachability and reasonable availability (REQ-PROD-040) and is the aggregate counterpart to the per-client limits above. The capacity point and the shedding policy are operational.
- **acceptance_criteria**: Apply aggregate load beyond the product's serving capacity; the product continues to serve a bounded share of requests (and reports the rest as limited or unavailable per REQ-OPS-045) rather than becoming unresponsive to all callers, and it recovers to normal service when the load subsides.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040, REQ-PROD-041

#### REQ-OPS-048 — Limits sit above ordinary use
- **statement**: The limits in this section are set above the rate and volume of ordinary legitimate use, so a person using the product normally — reading a timeline, posting through the day, following and liking as they browse — does not hit them. Limiting targets abuse and overload, not use; a limit that blocks a normal person is a worse product than a more generous one. The thresholds are operational, but the requirement that they clear ordinary use is a release requirement.
- **acceptance_criteria**: As one person, use the product the way a normal user would over a normal session — browse, post several times, follow and like a reasonable number of accounts — without being rate- or volume-limited. Only usage well beyond what a person does by hand triggers a limit.
- **verification_method**: manual_user_check
- **release_gating**: false
- **traces_to**: REQ-PROD-041

#### REQ-OPS-060 — Readiness is a distinct signal meaning "serving users"
- **statement**: The product exposes a readiness signal, distinct from liveness, whose meaning is that the
  product can serve users — not merely that the process is running. An operator or orchestrator
  can obtain readiness separately from liveness.
- **acceptance_criteria**: With the product fully able to serve, the readiness signal reports ready. With
  the product running but unable to serve its core loop, the readiness signal reports not-ready while the
  liveness signal still reports alive; the two are separately observable and can disagree.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-039, REQ-DOM-226

#### REQ-OPS-061 — Readiness reflects durable-store reachability
- **statement**: The readiness signal reports not-ready when the product's durable store is unreachable
  from the serving process, and ready when the store is reachable and the product can serve. Readiness is
  not satisfied by process liveness alone.
- **acceptance_criteria**: Make the durable store unreachable to a running serving process; the readiness
  signal reports not-ready. Restore reachability so the product can serve; the readiness signal reports
  ready.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-039, REQ-DOM-115

#### REQ-OPS-062 — Liveness reports process health only, not dependency health
- **statement**: The liveness signal reports whether the serving process itself is alive and able to make
  progress; it does not report not-alive solely because a recoverable dependency (such as the durable
  store) is temporarily unreachable. A dependency outage is reflected in readiness, not liveness, so an
  orchestrator restarts a genuinely crashed process but does not kill one that is alive and waiting on its
  store.
- **acceptance_criteria**: With the durable store unreachable but the serving process otherwise healthy,
  the liveness signal reports alive while the readiness signal reports not-ready. Liveness reports
  not-alive only when the process itself cannot make progress.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-DOM-226

#### REQ-OPS-063 — The health/readiness endpoint is unauthenticated and exposes no private data
- **statement**: The health and readiness signal can be queried without an account or session, so an
  operator or orchestrator can reach it, and its response discloses no private account data and no
  internal secret — it conveys serving status, not the contents of the system.
- **acceptance_criteria**: Query the health/readiness endpoint with no session; it responds with the
  serving status. Inspect the response; it contains no credential, session token, verified-contact value,
  or other private account data, and no secret such as a connection string.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-DOM-226, REQ-DOM-035

#### REQ-OPS-064 — Readiness is machine-consumable
- **statement**: The readiness signal is expressed so that an orchestrator or load balancer can
  mechanically distinguish ready from not-ready without interpreting prose — a decidable ready/not-ready
  outcome an automated caller can branch on.
- **acceptance_criteria**: Inspect the readiness signal while ready and while not-ready; in each state the
  signal carries a machine-distinguishable indication of the state (such that an automated caller routes
  traffic only when ready) without requiring a human to read a message.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-039

#### REQ-OPS-065 — Readiness tracks current state and recovers without a restart
- **statement**: The readiness signal reflects the product's current ability to serve, continuously: when
  the condition that made it not-ready clears (for example the durable store becomes reachable again), the
  signal returns to ready on its own, without restarting or redeploying the serving process.
- **acceptance_criteria**: Drive readiness to not-ready by making the durable store unreachable, then
  restore reachability without restarting the process; the readiness signal returns to ready on its own.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-039

#### REQ-OPS-066 — Operational events are recorded in a structured, parseable form
- **statement**: The product records its operational events as structured, machine-parseable records with
  consistent fields (such as a timestamp, a severity, an event category, and an outcome), not as
  free-form prose, so an operator can search and filter them rather than reading them line by line.
- **acceptance_criteria**: Exercise the product and inspect its log output; operational events appear as
  structured records with consistent, named fields that a log tool can parse and filter.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**:

#### REQ-OPS-067 — Logs let one request be followed end to end
- **statement**: Log records carry a correlation identifier that ties together the events of a single
  request or user action, so an operator can follow one action through the product's logs from receipt to
  outcome.
- **acceptance_criteria**: Perform a single user action that touches more than one logged step; the log
  records for that action share a correlation identifier by which they can be selected together and read in
  order.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**:

#### REQ-OPS-068 — Logs cover the events needed to explain operation and failure
- **statement**: The log record covers the operational events an operator needs to answer "what is it
  doing, and what went wrong": request handling and its outcome, errors and failures with a cause
  category, process start and stop, changes in durable-store connectivity, and authentication outcomes
  (success and failure) without the secret material involved.
- **acceptance_criteria**: Provoke, in a running product, a handled request, an error, a process restart, a
  loss and recovery of durable-store connectivity, and a failed and a successful sign-in; each is present
  in the logs as a distinct, categorized event with enough context to tell what happened.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-069 — Logs never contain credentials or private account data
- **statement**: Credentials, session tokens, verified-contact values, and any other private account data
  (`REQ-DOM-035`) are never written to the product's logs, in any field, at any severity. Operational
  visibility is never obtained at the cost of exposing private data in the logs.
- **acceptance_criteria**: Exercise sign-up, sign-in, recovery, and session use so that private data flows
  through the product, then inspect the logs produced; no credential, session token, verified-contact
  value, or other private account field appears in any log record. References to an account in a log use a
  non-secret identifier, not its private data.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-DOM-035

#### REQ-OPS-070 — Severity is applied consistently
- **statement**: Log records carry a severity that is applied consistently, so an operator can separate
  normal operation from warnings and errors by filtering on severity rather than by reading every line.
- **acceptance_criteria**: Inspect logs from a period that includes both normal operation and at least one
  error; normal events and error events carry distinct severities consistent with their nature, and
  filtering to error severity surfaces the error events.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**:

#### REQ-OPS-071 — Errors are logged with enough context to investigate
- **statement**: When the product logs an error it includes enough diagnostic context — what operation
  failed, a cause category, and identifiers needed to locate the affected request or resource — that an
  operator can investigate without reproducing the failure, subject always to the no-secrets/no-PII rule
  (`REQ-OPS-069`).
- **acceptance_criteria**: Provoke a representative failure; its log record names the failing operation and
  a cause category and carries a correlation or resource identifier sufficient to investigate, and contains
  no private account data or secret.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**: REQ-DOM-035

#### REQ-OPS-072 — The product exposes request-rate and error-rate metrics
- **statement**: The product exposes serving metrics that report the rate of requests it is handling and
  the rate that are failing, broken down enough to see which class of surface is affected (for example
  reads versus writes, or by outcome category). Metric labels carry no private account data.
- **acceptance_criteria**: Generate a known mix of successful and failing requests; the metrics reflect the
  request rate and the error rate, distinguishable by surface class or outcome. Inspect the metric labels;
  they contain no handle-linked private data, credential, or contact value.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040, REQ-DOM-035

#### REQ-OPS-073 — Request latency is observable as a distribution
- **statement**: The product exposes request-handling latency as a distribution (not only an average), so
  that a product which is up but slow — tail latency degrading while the mean looks acceptable — is
  visible.
- **acceptance_criteria**: Drive traffic including some slow requests; the latency metric reflects the
  spread of response times (for example percentiles or buckets), not a single averaged value, and the slow
  requests are visible in the distribution.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

#### REQ-OPS-074 — A saturation/headroom signal is observable
- **statement**: The product exposes a saturation or headroom signal — a measure of how close it is to its
  serving capacity, such as in-flight work against available capacity — so that approaching overload is
  detectable before it turns into errors or timeouts.
- **acceptance_criteria**: Drive load toward the product's capacity; the saturation signal rises as
  headroom shrinks, giving an observable indication of approaching overload ahead of widespread failures.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

#### REQ-OPS-075 — A serving/availability signal detects a degraded product
- **statement**: The product exposes an availability or serving signal sufficient to detect that it is
  degraded or not serving its core loop — the signal on which the status-and-incident posture
  (`REQ-OPS-080..099`) builds incident detection. It makes "the product is not serving users" an
  observable fact rather than an inference from complaints.
- **acceptance_criteria**: Put the product into a degraded state in which the core loop does not work for
  users; the serving/availability signal reflects the degradation. Return it to serving; the signal
  reflects recovery. The signal is suitable to drive automated detection.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-076 — Durable-store interaction is observable
- **statement**: Because serving now depends on the durable store, the product exposes the health of its
  interaction with that store — the rate of store errors and the latency of store operations — so an
  operator can tell a store problem from an application problem.
- **acceptance_criteria**: Induce elevated store latency or store errors; the product's store-interaction
  metrics reflect it, distinguishably from application-level request errors.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-DOM-115

#### REQ-OPS-077 — Administrative actions are recorded in an audit trail
- **statement**: Every action taken through the supported administrative surface (`REQ-PROD-028`) —
  assisting a user's recovery, removing content, and disabling or actioning an account — is recorded in an
  audit trail that captures the acting operator/administrator identity, the action taken, the target
  account or content, and the time.
- **acceptance_criteria**: Through the supported administrative surface, assist a user's recovery, remove a
  named chirp, and action an account; each action produces an audit entry naming who acted, what they did,
  which account or content was affected, and when. An administrative action that reaches accounts or
  content leaves no untraced path.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-028

#### REQ-OPS-078 — Audit records are durable and not silently editable
- **statement**: Audit-trail entries are durable — they survive restart and redeploy like the rest of the
  product's authoritative state — and are not silently editable or erasable through the product's ordinary
  surfaces, so the trail remains trustworthy when it is reviewed after the fact.
- **acceptance_criteria**: Produce audit entries, then restart or redeploy the product; the entries are
  still present and unchanged. No ordinary product surface offers a way to alter or delete an existing
  audit entry without trace.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-028

#### REQ-OPS-079 — The audit trail carries no secrets or private data
- **statement**: The audit trail records who acted, what they did, and on which account or content using
  stable, non-secret identifiers, and never stores credentials, session tokens, or verified-contact values
  (`REQ-DOM-035`). The same no-secrets/no-PII rule that binds the logs binds the audit trail.
- **acceptance_criteria**: Inspect audit entries produced by representative administrative actions; each
  names the actor, action, and target by non-secret identifier and contains no credential, session token,
  or verified-contact value.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-DOM-035

#### REQ-OPS-080 — A serving degradation is detected, not waited for
- **statement**: The product's serving/availability signal (`REQ-OPS-075`) is watched in production so that
  a degradation — the product not serving its core loop, or serving it with elevated errors — is detected
  as an operational event and does not depend on a user reporting it. Detection reaches a responsible
  operator (`REQ-OPS-083`).
- **acceptance_criteria**: Put the product into a state where the core loop does not work for users; within
  a bounded time the configured detection raises a notification to the responsible operator, without a user
  having had to report the problem first. Return the product to serving; no further alert for that incident
  is raised.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-081 — Detection is driven by the observability signals
- **statement**: Incident detection is built on the observability signals — the serving/availability signal
  (`REQ-OPS-075`), the request- and error-rate metrics (`REQ-OPS-072`), the saturation/headroom signal
  (`REQ-OPS-074`), and the durable-store interaction health (`REQ-OPS-076`) — rather than on a person
  watching a screen. The conditions that constitute an incident are expressed against those signals.
- **acceptance_criteria**: Inspect what drives detection; each alerting condition names an observability
  signal and a threshold or state on it. Induce an elevated error rate, approaching saturation, and a
  store-interaction fault in turn; each is capable of being surfaced as a detected condition from its
  signal.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-082 — Incidents carry a proportionate severity
- **statement**: A detected incident is classified by a small severity scheme that at least distinguishes
  "the product is not serving its core loop" (a major incident) from "the product is degraded or partially
  impaired but still serving" (a minor incident), so that response and communication are proportionate to
  what is actually broken. The scheme is kept to the scale of a small hosted product.
- **acceptance_criteria**: Find the stated severity scheme; it defines at least a major and a minor level
  with a criterion for each expressed against the serving signal. Classify a total-outage case and a
  partial-degradation case; each maps to the intended level.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**: REQ-PROD-040

#### REQ-OPS-083 — A responsible operator is identified for incident response
- **statement**: There is a stated expectation of who is responsible for responding when an incident is
  detected — a role, not a staffed rotation — so that a fired alert has an owner rather than reaching no
  one. This is the operator of operator journey J9.
- **acceptance_criteria**: Find the stated responsibility; it names the role accountable for acknowledging
  and acting on a detected incident. A detected major incident has a defined responder; no detected incident
  is left with no one responsible for it.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-084 — An incident has a defined response path
- **statement**: A detected incident has a defined response expectation — acknowledge, investigate using the
  observability signals and logs, communicate to users per the status requirements, and resolve — recorded
  so the responder follows a known path rather than improvising. The detailed step-by-step runbooks are
  deferred to later operational work; this requires that the response path exist and be written down.
- **acceptance_criteria**: Find the recorded incident-response expectation; it covers acknowledgement,
  investigation against the observability signals, user communication, and resolution. Walk a representative
  incident through it; each step has a defined owner and outcome.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**: REQ-PROD-040

#### REQ-OPS-085 — Users can learn that the product is down or degraded
- **statement**: When the product is unavailable or degraded, a person has a way to learn that chirp itself
  is affected — a status surface or an in-product notice — rather than being left to infer it from a
  failed action or a spinner. The notice states, at least, that the product is experiencing a problem.
- **acceptance_criteria**: With the product in an outage or degraded state, a user consulting the status
  surface or in-product notice sees that chirp is affected and is not left only with an unexplained failure.
  With the product serving normally, the surface shows normal operation.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-086 — The status surface does not depend on the failed product
- **statement**: The surface users consult to learn of an outage is reachable when the product itself is
  down — it does not depend on the serving path, the durable store, or the process that may be the thing
  that has failed. A status page that goes down with the product conveys nothing at the moment it is needed.
- **acceptance_criteria**: Take the serving product fully offline, including its durable store; the status
  surface is still reachable and reports the product as unavailable. The surface's availability does not
  require the product's serving path to be up.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-087 — Planned maintenance is announced in advance
- **statement**: When the operator plans maintenance that will make the product unavailable or degraded,
  users are given advance notice through the same status surface or in-product notice, stating the expected
  window, so a planned interruption is distinguishable to a user from an unexpected outage.
- **acceptance_criteria**: Schedule a maintenance window and publish the notice; before the window a user
  consulting the status surface sees the planned maintenance and its expected timing. During the window the
  interruption is presented as planned, not as an unexplained failure.
- **verification_method**: manual_user_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

#### REQ-OPS-088 — Status and incident communication carries no private or secret data
- **statement**: Nothing published on the status surface, in an in-product outage notice, or in a
  maintenance announcement contains private account data or secret material — no credential, session token,
  verified-contact value (`REQ-DOM-035`), nor internal secret such as a connection string or a stack trace
  that exposes them. Telling users about an outage never leaks the data the outage is around.
- **acceptance_criteria**: Inspect the content published for a representative outage and a representative
  maintenance notice; it describes product status in terms a user needs and contains no credential, session
  token, verified-contact value, connection string, or raw internal error detail that discloses private or
  secret data.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-DOM-035

#### REQ-OPS-089 — Recovery is communicated when service returns
- **statement**: When a communicated incident is resolved and the product is serving again, the status
  surface or in-product notice reflects the recovery, so a user who saw the outage can tell that it is over
  rather than being left unsure whether the problem persists.
- **acceptance_criteria**: Communicate an outage, then resolve it and restore serving; the status surface
  updates to show the product operating normally. A user who saw the outage notice can see that service has
  recovered.
- **verification_method**: manual_user_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

#### REQ-OPS-090 — The product fails into its designed degraded states
- **statement**: When a dependency the product needs is unavailable in production — in particular the
  durable store (`REQ-DOM-115`) — the operated product presents the user the degraded and error experiences
  designed by the experience area (`REQ-UX-*`, `REQ-PROD-033`, `REQ-PROD-034`): an explained error, a
  preserved in-progress draft, a routed-to-sign-in on an expired session. This requirement is the
  operational join to those states; it does not define them.
- **acceptance_criteria**: In production, make the durable store unreachable while the serving process stays
  up; a user exercising the core loop meets an explained degraded or error state, not a blank page or a
  crashed process. The states a user sees are the ones the experience design specifies.
- **verification_method**: manual_user_check
- **release_gating**: true
- **traces_to**: REQ-PROD-033, REQ-PROD-034

#### REQ-OPS-091 — A partial failure is contained to the affected capability
- **statement**: When one capability is degraded rather than the whole product — for example the durable
  store is slow but reachable — the parts of the product that can still serve continue to do so, to the
  extent the product's design allows, rather than a single fault turning into a total outage.
- **acceptance_criteria**: Induce degradation in one capability (for example elevated store latency); the
  product continues to serve the surfaces that do not depend on the degraded capability, and the impact a
  user sees is scoped to the affected capability rather than a blanket failure.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040, REQ-DOM-115

#### REQ-OPS-092 — A stated, proportionate availability target
- **statement**: The product states an availability target that defines what "reasonably available"
  (`REQ-PROD-040`) means for it — a figure proportionate to a small hosted service, expressed over a stated
  window (for example a monthly serving-availability target). The target is written down, not left implicit.
- **acceptance_criteria**: Find the stated availability target; it gives a specific availability figure and
  the window it is measured over, and it is proportionate to a small hosted product rather than an
  enterprise commitment.
- **verification_method**: inspection
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-093 — Availability is measured from the serving signal, not asserted
- **statement**: The availability target is measured in production from the serving/availability signal
  (`REQ-OPS-075`) over the stated window, so that whether the product met its target is a measured fact
  rather than a claim. Unavailability is counted when the serving signal shows the product not serving its
  core loop.
- **acceptance_criteria**: Over a measurement window that includes at least one serving interruption,
  compute availability from the serving signal; the result reflects the observed interruption and can be
  compared against the stated target to say whether it was met.
- **verification_method**: operational_check
- **release_gating**: true
- **traces_to**: REQ-PROD-040

#### REQ-OPS-094 — Announced maintenance is accounted for in availability
- **statement**: Availability measurement distinguishes announced planned maintenance (`REQ-OPS-087`) from
  unplanned outage, so that routine, pre-announced deploys and maintenance do not register as unplanned
  unavailability against the target.
- **acceptance_criteria**: Take the product down during an announced maintenance window and, separately,
  during an unplanned incident; the availability accounting attributes the first to planned maintenance and
  the second to unplanned unavailability, per the stated rule.
- **verification_method**: inspection
- **release_gating**: false
- **traces_to**: REQ-PROD-040

#### REQ-OPS-095 — A stated time-to-restore-serving objective for incidents
- **statement**: Incident response works to a stated objective for how quickly serving is restored after a
  detected outage, consistent with the availability target (`REQ-OPS-092`). This is time-to-restore
  serving; it is distinct from the data recovery-time objective owned by backups and recovery
  (`REQ-OPS-020..039`), which concerns restoring lost data rather than resuming service.
- **acceptance_criteria**: Find the stated time-to-restore-serving objective; it gives a target duration
  from detection to restored serving and is consistent with the availability target. For a representative
  resolved incident, the time from detection to restored serving is measurable against it.
- **verification_method**: operational_check
- **release_gating**: false
- **traces_to**: REQ-PROD-040

### OPS traceability table

`REQ-OPS-*` → the product/domain requirements it refines, with its release-gating flag. An empty `traces_to` marks an operational-internal requirement that refines no single product/domain requirement.

| REQ-OPS | release_gating | traces_to |
|---------|----------------|-----------|
| REQ-OPS-001 | true | REQ-PROD-037, REQ-DOM-100 |
| REQ-OPS-002 | true | REQ-PROD-037, REQ-DOM-113, REQ-DOM-114 |
| REQ-OPS-003 | true | REQ-PROD-037, REQ-DOM-113 |
| REQ-OPS-004 | true | REQ-PROD-037, REQ-DOM-118 |
| REQ-OPS-005 | true | REQ-PROD-037, REQ-DOM-109 |
| REQ-OPS-006 | true | REQ-DOM-107, REQ-DOM-110, REQ-DOM-111, REQ-DOM-112 |
| REQ-OPS-007 | true | REQ-DOM-110, REQ-DOM-116 |
| REQ-OPS-008 | true | REQ-DOM-110, REQ-DOM-112 |
| REQ-OPS-009 | true | REQ-PROD-039, REQ-DOM-115 |
| REQ-OPS-010 | true | REQ-PROD-037 |
| REQ-OPS-011 | true | REQ-DOM-037, REQ-DOM-112 |
| REQ-OPS-012 | true | REQ-DOM-037, REQ-PROD-038 |
| REQ-OPS-013 | true | REQ-DOM-117, REQ-PROD-019 |
| REQ-OPS-014 | true | REQ-DOM-106, REQ-DOM-030, REQ-DOM-031 |
| REQ-OPS-015 | true | REQ-DOM-102, REQ-DOM-037 |
| REQ-OPS-016 | true | REQ-PROD-037, REQ-DOM-037 |
| REQ-OPS-020 | true | REQ-PROD-038, REQ-DOM-116 |
| REQ-OPS-021 | true | REQ-PROD-038, REQ-DOM-037 |
| REQ-OPS-022 | true | REQ-DOM-116, REQ-DOM-110 |
| REQ-OPS-023 | true | REQ-PROD-038 |
| REQ-OPS-024 | true | REQ-PROD-038 |
| REQ-OPS-025 | true | REQ-PROD-038, REQ-PROD-040 |
| REQ-OPS-026 | true | REQ-PROD-038, REQ-DOM-116 |
| REQ-OPS-027 | true | REQ-DOM-116, REQ-DOM-110, REQ-DOM-111, REQ-DOM-107 |
| REQ-OPS-028 | true | REQ-PROD-038 |
| REQ-OPS-029 | true | REQ-PROD-037, REQ-DOM-118 |
| REQ-OPS-030 | true | REQ-DOM-118, REQ-DOM-116 |
| REQ-OPS-031 | true | REQ-PROD-038, REQ-DOM-116 |
| REQ-OPS-032 | true | REQ-PROD-038 |
| REQ-OPS-033 | true | REQ-PROD-038 |
| REQ-OPS-040 | true | REQ-PROD-041 |
| REQ-OPS-041 | true | REQ-PROD-041 |
| REQ-OPS-042 | true | REQ-PROD-041 |
| REQ-OPS-043 | true | REQ-PROD-041, REQ-DOM-012, REQ-DOM-002 |
| REQ-OPS-044 | true | REQ-PROD-041, REQ-DOM-218, REQ-DOM-027 |
| REQ-OPS-045 | true | REQ-PROD-041, REQ-DOM-220 |
| REQ-OPS-046 | true | REQ-PROD-041 |
| REQ-OPS-047 | false | REQ-PROD-040, REQ-PROD-041 |
| REQ-OPS-048 | false | REQ-PROD-041 |
| REQ-OPS-060 | true | REQ-PROD-039, REQ-DOM-226 |
| REQ-OPS-061 | true | REQ-PROD-039, REQ-DOM-115 |
| REQ-OPS-062 | true | REQ-DOM-226 |
| REQ-OPS-063 | true | REQ-DOM-226, REQ-DOM-035 |
| REQ-OPS-064 | true | REQ-PROD-039 |
| REQ-OPS-065 | true | REQ-PROD-039 |
| REQ-OPS-066 | true | — |
| REQ-OPS-067 | false | — |
| REQ-OPS-068 | true | REQ-PROD-040 |
| REQ-OPS-069 | true | REQ-DOM-035 |
| REQ-OPS-070 | false | — |
| REQ-OPS-071 | false | REQ-DOM-035 |
| REQ-OPS-072 | true | REQ-PROD-040, REQ-DOM-035 |
| REQ-OPS-073 | false | REQ-PROD-040 |
| REQ-OPS-074 | false | REQ-PROD-040 |
| REQ-OPS-075 | true | REQ-PROD-040 |
| REQ-OPS-076 | false | REQ-DOM-115 |
| REQ-OPS-077 | true | REQ-PROD-028 |
| REQ-OPS-078 | false | REQ-PROD-028 |
| REQ-OPS-079 | true | REQ-DOM-035 |
| REQ-OPS-080 | true | REQ-PROD-040 |
| REQ-OPS-081 | true | REQ-PROD-040 |
| REQ-OPS-082 | false | REQ-PROD-040 |
| REQ-OPS-083 | true | REQ-PROD-040 |
| REQ-OPS-084 | false | REQ-PROD-040 |
| REQ-OPS-085 | true | REQ-PROD-040 |
| REQ-OPS-086 | true | REQ-PROD-040 |
| REQ-OPS-087 | false | REQ-PROD-040 |
| REQ-OPS-088 | true | REQ-DOM-035 |
| REQ-OPS-089 | false | REQ-PROD-040 |
| REQ-OPS-090 | true | REQ-PROD-033, REQ-PROD-034 |
| REQ-OPS-091 | false | REQ-PROD-040, REQ-DOM-115 |
| REQ-OPS-092 | true | REQ-PROD-040 |
| REQ-OPS-093 | true | REQ-PROD-040 |
| REQ-OPS-094 | false | REQ-PROD-040 |
| REQ-OPS-095 | false | REQ-PROD-040 |
