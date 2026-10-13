# chirp — Release Bar

This document consolidates the five concern designs into the single, testable definition of what
"releasable" means for chirp. It has three parts:

1. **MVP-vs-releasable scoping** — the honest line between what makes chirp *usable early* and what the
   *releasable* product requires on top. No hidden cuts.
2. **The release bar** — `BAR-*` criteria. Each one is concrete, names a verification method a stranger
   could run, references the `REQ-*` it enforces, and is placed in a stage (`usable_early` | `releasable`).
3. **Completeness & gap review** — a deliberate sweep over the surfaces a product of chirp's kind is
   expected to have, with an in-/out-of-scope judgment and a reason recorded for each, so any absence is a
   decision rather than an oversight.

The bar is a *bar*, not an aspiration: every criterion is checkable against the live product by someone who
was not in the room. "It deploys / is reachable / CI is green / a working skeleton" is **not** a release
bar and does not appear below.

This file composes against `../contracts/`: `BAR-*` ids and the `stage` field follow
`id_and_traceability_schema.yaml`; every `verification_method` is drawn from that schema's closed
vocabulary (`manual_user_check`, `automated_test`, `inspection`, `operational_check`); the `REQ-*` ids are
owned and defined by the five concern designs and are not redefined here.

---

## 1. MVP vs releasable scoping

*(section: `mvp_vs_releasable_scoping`)*

### 1.1 The two stages, and the principle that separates them

Every `BAR-*` is tagged with the stage at which it must hold:

- **`usable_early`** — the floor at which chirp first becomes a product a real person can actually use for
  its core purpose. This is **not a skeleton and not a demo.** The usable-early line is drawn so that a
  stranger can sign up with a real credential, have their account and content survive a restart, read the
  public timeline, post within the limit, build a timeline by following people, like and unlike, act only
  as themselves, and get honest loading/empty/error feedback on a phone or a desktop. A product that loses
  all data on restart, lets anyone act as anyone by typing a handle, or shows a blank screen on error is
  **below** this floor, not within it.
- **`releasable`** — the full bar. Everything in `usable_early` **plus** the surfaces that make a public,
  account-holding product responsible to put online and hand to strangers: the complete account lifecycle
  (recovery, deletion, data export), the trust/safety/privacy/legal surface, admin & support, abuse limits
  and quotas, durable-data operations (backups, recovery drills, rollback, migration safety), observability
  and incident response, accessibility to WCAG 2.1 AA across the whole inventory, responsive layout across
  the breakpoint set, and a reproducible self-hostable build/run.

**The principle.** A criterion is `usable_early` when its *absence makes the core loop unusable or unsafe
to use at all* (no real accounts, no durable data, impersonation possible, authenticated mutations forgeable,
a dead screen on the common error). A criterion is `releasable` when the core loop works without it but a
product of chirp's kind **cannot responsibly be released** without it (a user who loses their credential is
stranded forever; a user cannot close their account or take their data; there is no way to respond to abuse;
a disk loss is unrecoverable; the product is unusable to someone on a keyboard or screen reader). Security
properties that make *authenticated* use safe at all — session confidentiality, server-side authorization,
protection against cross-site request forgery and stored script injection — are placed in `usable_early`,
because shipping real accounts without them is not a smaller product, it is a dangerous one.

### 1.2 What is in `usable_early` (the early-usable floor)

- **Real credentialed accounts & sessions**: self-service sign-up with handle validation/uniqueness,
  credentialed sign-in that proves ownership, durable sessions, deliberate sign-out, a truthful
  signed-in/logged-out indicator, and no silent sign-out. (Closes today's handle-only, impersonable sign-in
  and restart-lost sessions.)
- **Durable persistence** of accounts, chirps, follows, and likes across restart/redeploy/reschedule, with
  the domain invariants enforced in the backing store. (Closes today's in-memory store.)
- **The core loop**: read the public timeline and profiles without an account; a useful logged-out landing
  (with its empty/error states) and an obvious join route; compose and post within the limit without losing
  a draft; a reverse-chronological home timeline with honest loading/content/empty/error/end-of-list states
  and paging; profile views that handle not-found and error; reliable follow/unfollow and like/unlike with
  truthful counts.
- **Authorization & safe authenticated use**: a user acts only on their own resources and only as their own
  identity (no acting-as-another); every mutation is authorized on the server and default-denied; the
  session secret is confidential and resists fixation; authenticated mutations resist cross-site forgery;
  user-submitted content cannot execute as script in another reader's browser; credentials are never
  revealed, displayed, or logged.
- **Own-profile management**: view and edit your own display name and bio, persisted.
- **Honest interaction** for the above: inline validation at/before submit, busy/disabled submission state
  with no duplicate submits, optimistic actions that reconcile to the truth on failure, session-expiry that
  prompts re-sign-in rather than failing silently or falsely succeeding.
- **Usable on a phone**: one readable content column that reflows without horizontal scroll, compose and the
  core screens usable at a small portrait width without losing state.
- **A health signal** that reflects whether the product can actually serve users (not merely that the
  process is up).

### 1.3 What `releasable` adds on top

- **Full account lifecycle**: account recovery for a locked-out owner; self-service account deletion with
  stated consequences and honoured removal; portable data export of one's own data; a reachable
  account-management hub gathering these.
- **Trust, safety, privacy & legal**: report a chirp or account; reduce exposure (block); moderation whose
  effect is visible; a reachable, accurate privacy-and-terms surface; erasure and access rights honoured.
- **Abuse limits & quotas**: baseline limits on account creation and high-volume actions; per-caller
  request-rate limits tightened for writes; bounded request payloads and bounded, clamped pagination.
- **Admin & support**: a segregated operator surface to look up an account and act on abuse, with
  least-privilege elevated authority, attributable elevated actions, and a privacy limit (data
  minimisation) on the operator view.
- **Durable-data operations**: scheduled, complete, monitored, separately-stored backups; a defined-and-met
  RPO and RTO; a *proven* restore drill; a documented recovery runbook; deploy rollback that does not
  destroy data; safe forward schema migration with a recovery path; all working on the self-hostable
  surface.
- **Observability & incident response**: distinct liveness/readiness that exercise the durable dependency
  and fail closed; structured logs with correlation ids and no secrets/PII; request metrics and named SLIs;
  detection on a product-health condition that notifies a responsible human; an honest user-facing degraded
  state that never loses work; defined incident responsibility and durable incident records.
- **Accessibility** to WCAG 2.1 AA across the whole screen inventory.
- **Responsive layout** across the defined breakpoint set for every surface (not only the mobile floor).
- **The full visual design system** and its product-wide consistency.
- **A reproducible, documented self-hostable build/run**.

### 1.4 Tracked but deliberately out of the bar (non-gating)

These are real improvements the design records as **non-release-gating**; they are not part of the bar at
either stage. They are listed here so the cut is explicit, not silent:

- New-user onboarding beyond a functional empty first session (`REQ-PROD-022`, `REQ-UX-014`, `REQ-UX-049`).
- A full replies/thread experience — provided no dead reply affordance ships (`REQ-PROD-023`, `REQ-DOM-007`,
  `REQ-UX-013`, `REQ-UX-047`, `REQ-UX-048`).
- Search/discovery beyond on-screen links, and its navigation entry and wide-desktop multi-region layout
  (`REQ-PROD-024`, `REQ-UX-008`, `REQ-UX-050`, `REQ-UX-051`, `REQ-UX-104`).
- "What's new" continuity for returning users (`REQ-PROD-025`).
- Mute as a softer complement to block (`REQ-ID-062`); self-service session listing/revocation
  (`REQ-ID-036`); MFA (`REQ-ID-038`); a dark theme (`REQ-UX-089`).
- Operability maturity beyond the first-release floor: backup access-control/retention policy
  (`REQ-OPS-027`), pagination clamp niceties and per-account storage quotas already covered by rate limits
  (`REQ-OPS-045`, `REQ-OPS-046`), the self-identifying health endpoint nicety (`REQ-OPS-064`), scrape-safe
  metrics separation nicety (`REQ-OPS-073`), alert tuning (`REQ-OPS-082`), a public status page
  (`REQ-OPS-085`), proactive user incident communication (`REQ-OPS-088`), and post-incident
  recurrence-prevention follow-ups (`REQ-OPS-089`).

> **Not-a-prototype note.** Unless the engagement explicitly reduces it, the bar is a *releasable product*.
> The non-gating list above is the only sanctioned way to ship less, and every item on it is a recorded
> decision with its owning requirement's stated reason — never a quiet omission.

---

## 2. The release bar

*(section: `release_bar`)*

Each criterion: **statement** (the release condition) · **references** (the `REQ-*` it enforces) ·
**verification_method** (from the schema vocabulary) · **stage**. A criterion is met only when its
verification, run against the live product, passes. The bar is met when every `releasable`-stage criterion
passes; the `usable_early` subset is the earlier gate a usable product clears first.

### A. Accounts, authentication & sessions

#### BAR-001 — Self-service, validated sign-up creates one owned, credentialed account
A logged-out visitor can create a chirp account entirely in the web UI, reached from the first screen and
the sign-in screen; the handle is validated inline (length/characters) and handle-uniqueness conflicts are
surfaced clearly with no duplicate created; submission yields exactly one complete account signed in as that
new user, anchored to a credential and with a recovery factor established — all without touching the raw API.
- **references:** REQ-PROD-001, REQ-PROD-002, REQ-PROD-003, REQ-ID-001, REQ-ID-002, REQ-ID-003, REQ-ID-004, REQ-ID-005, REQ-UX-033, REQ-UX-034, REQ-UX-035
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-002 — Sign-in proves ownership; knowing a handle is never enough
Signing in requires proof of a secret only the owner holds; a credential is verified against the account on
every sign-in; a person who knows another user's handle but not their credential cannot sign in as, or act
as, that account. Authentication failure is uniform and non-enumerating; online guessing is throttled.
- **references:** REQ-PROD-004, REQ-ID-020, REQ-ID-021, REQ-ID-022, REQ-ID-023, REQ-UX-036, REQ-UX-037
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-003 — Sessions are durable, bounded, and never silently dropped
Authentication establishes a session (the credential is not re-sent per action); the session secret is
confidential, unguessable, and protected in transit; sessions have a bounded lifetime and survive normal
operation or end visibly; the signed-in/logged-out state is always shown truthfully; expiry or invalidation
surfaces a re-sign-in prompt on the next action rather than failing silently or appearing to succeed.
- **references:** REQ-PROD-006, REQ-ID-026, REQ-ID-027, REQ-ID-028, REQ-ID-029, REQ-ID-030, REQ-ID-035, REQ-DOM-006, REQ-UX-015, REQ-UX-026, REQ-UX-042, REQ-UX-060
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-004 — Deliberate sign-out ends the session at the server
A signed-in user can find and use a sign-out control in a consistent place; after it the server treats the
session as ended (a second person on a shared device cannot act as the first user), and the user must
re-authenticate to act again.
- **references:** REQ-PROD-005, REQ-ID-031, REQ-UX-005, REQ-UX-059
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-005 — Session integrity: fixation-resistant, re-proof for sensitive actions, revoked on credential change
The session issued at authentication resists fixation; destructive or identity-changing actions require
fresh proof; a credential change or recovery invalidates all existing sessions; a session acts only as its
own account and every account action requires one.
- **references:** REQ-ID-025, REQ-ID-032, REQ-ID-033, REQ-ID-034
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-006 — Account recovery returns a locked-out owner to the same account, without becoming a bypass
A user who cannot sign in can start recovery from the sign-in screen, re-prove control via a pre-established
factor, and regain access to the same account (same handle, posts, follows); a failed attempt changes
nothing; recovery is honestly observable and does not enumerate accounts; recovery is not an authentication
bypass and supersedes the lost credential.
- **references:** REQ-PROD-007, REQ-ID-006, REQ-ID-007, REQ-ID-008, REQ-ID-009, REQ-ID-037, REQ-UX-038, REQ-UX-039
- **verification_method:** manual_user_check
- **stage:** releasable

### B. Account lifecycle: profile, management hub, deletion, export

#### BAR-007 — A user can view and edit their own profile, persisted
A signed-in user can view their handle, display name, and bio and change the display name and bio; after
saving, changes show on their profile and persist on a later visit; profile identity (handle/id) is
immutable while profile fields are mutable; editing validates inline and never loses the edit.
- **references:** REQ-PROD-008, REQ-DOM-008, REQ-UX-053, REQ-UX-054
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-008 — A reachable account-management hub gathers the lifecycle surfaces
From the signed-in navigation a user reaches an account-management/settings hub that gathers, at minimum,
profile editing, account deletion, data export, sign-out, and the privacy & terms link — all without the
raw API.
- **references:** REQ-PROD-009, REQ-UX-004, REQ-UX-006, REQ-UX-052
- **verification_method:** manual_user_check
- **stage:** releasable

#### BAR-009 — Self-service account deletion is confirmed, honoured, and bounded
A signed-in user can delete their own account from the UI; the product states what will be removed and that
deletion is permanent, and requires explicit confirmation; after deletion the user is signed out and the
account and its personal data are removed (defined cascade), not-found thereafter, within a bounded,
durable time; only the owner (or an authorized administrator) can trigger it.
- **references:** REQ-PROD-010, REQ-PROD-012, REQ-ID-010, REQ-ID-011, REQ-ID-012, REQ-ID-013, REQ-ID-014, REQ-ID-043, REQ-DOM-009, REQ-OPS-011, REQ-UX-055, REQ-UX-056
- **verification_method:** manual_user_check
- **stage:** releasable

#### BAR-010 — A user can export their own complete data in a portable form
A signed-in user can export their own data; the export is complete (everything the durable model holds for
that user, per a defined "a user's data"), portable, documented, and machine-readable; it returns only the
requesting user's own data; the UI shows honest progress and resolves to a real download or an honest error;
only the owner (or an authorized administrator on their behalf) can obtain it.
- **references:** REQ-PROD-011, REQ-ID-015, REQ-ID-016, REQ-ID-017, REQ-ID-044, REQ-DOM-010, REQ-UX-057, REQ-UX-058
- **verification_method:** manual_user_check
- **stage:** releasable

### C. The core product loop

#### BAR-011 — Reading never requires an account; the logged-out landing is useful even when empty
A logged-out visitor can read the public timeline and the profiles reached from it without signing in; the
logged-out landing says what chirp is, offers an obvious join route, and handles its loading, empty, error,
and end-of-list states honestly (no blank or broken screen when there is nothing yet).
- **references:** REQ-PROD-013, REQ-PROD-014, REQ-PROD-015, REQ-UX-002, REQ-UX-030, REQ-UX-031, REQ-UX-032
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-012 — Compose and post a chirp within the limit, without losing the draft
A signed-in user can compose and post a chirp, sees the character limit, gets honest non-optimistic
busy/error/success feedback, and never silently loses entered text on failure or session expiry; the new
chirp appears afterward.
- **references:** REQ-PROD-016, REQ-PROD-017, REQ-UX-012, REQ-UX-043, REQ-UX-044, REQ-UX-078
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-013 — The home timeline is a reverse-chronological feed with honest states and paging
A signed-in user sees a reverse-chronological home feed with honest loading/content/empty/error/
end-of-list states and working paging; like, follow, and compose are reachable from the feed; an action on
an expired session prompts re-sign-in and never fails silently.
- **references:** REQ-PROD-018, REQ-UX-040, REQ-UX-041
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-014 — Profiles present the user honestly and handle their real states
A profile view presents the user, distinguishes own-profile from another's (exposing edit/settings on one's
own), and handles not-found and error states; follow/unfollow from a profile is optimistic, truthful, and
reconciles on failure.
- **references:** REQ-PROD-019, REQ-UX-045, REQ-UX-046
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-015 — Follow/unfollow and like/unlike are reliable and truthful
Follow/unfollow and like/unlike reflect the true server state with truthful counts; optimistic updates
reconcile to the truth on failure; the controls reflect the real current state everywhere they appear.
- **references:** REQ-PROD-020, REQ-PROD-021, REQ-UX-081, REQ-UX-082
- **verification_method:** manual_user_check
- **stage:** usable_early

### D. Honest state, feedback & navigation (product-wide)

#### BAR-016 — One consistent, honest state model across the product
The product uses one consistent state model: loading is shown in place and never traps the user; empty
states explain why and offer the next action; errors are honest, recoverable in place, and never lose input;
not-found is a designed state with a way back; end-of-list is explicit and distinct from empty; the state
vocabulary is surfaced consistently product-wide.
- **references:** REQ-UX-019, REQ-UX-020, REQ-UX-021, REQ-UX-022, REQ-UX-023, REQ-UX-024, REQ-UX-025
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-017 — Input, submission & action feedback is honest and loss-free
Validation is inline, actionable, and at or before submit; submissions show a busy state, prevent duplicate
submits, and reconcile on failure; success is honest, observable, and never false; optimistic updates
reconcile to the true state on failure; destructive actions confirm consequences while reversible ones do
not over-confirm; auth-gating happens at the point of action, never as a silent fail.
- **references:** REQ-UX-016, REQ-UX-017, REQ-UX-018, REQ-UX-027, REQ-UX-028, REQ-UX-029
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-018 — Navigation reaches every product surface in both signed-in and logged-out states
Persistent signed-in primary navigation covers the core loop; logged-out navigation makes reading and
joining obvious; every inventory surface is reachable through the product; sign-out has a single
discoverable consistent placement; privacy & terms is reachable in both states; a coherent screen map with
stable destinations exists; the logged-out→signed-in transition is designed.
- **references:** REQ-UX-001, REQ-UX-003, REQ-UX-007, REQ-UX-010, REQ-UX-011
- **verification_method:** manual_user_check
- **stage:** usable_early

### E. Durable persistence & data integrity

#### BAR-019 — All domain state is durable and survives restart, redeploy, and reschedule
Every kind of domain state (accounts, chirps, follows, likes, sessions, moderation state) is persisted on
pod-independent storage and survives a process restart, a rolling redeploy, and a reschedule to another
node; a confirmed write is durable before it is confirmed; the in-memory store is replaced. (This is the
single largest gap between the current product and the bar.)
- **references:** REQ-PROD-026, REQ-DOM-012, REQ-DOM-013, REQ-OPS-001, REQ-OPS-002, REQ-OPS-003, REQ-OPS-004, REQ-OPS-006, REQ-OPS-007
- **verification_method:** operational_check
- **stage:** usable_early

#### BAR-020 — Domain invariants and derived counts hold in the backing store, under concurrency
The canonical `chirp-types` model is preserved and validated at the domain boundary; entity invariants hold
in any backing store; ids are opaque, server-assigned, monotonic, and stable across restarts; multi-write
operations keep counts consistent with edges atomically; the persistence seam is safe for concurrent access;
derived state stays correct across the durability boundary.
- **references:** REQ-DOM-001, REQ-DOM-002, REQ-DOM-003, REQ-DOM-004, REQ-DOM-015, REQ-DOM-016, REQ-OPS-008
- **verification_method:** automated_test
- **stage:** usable_early

#### BAR-021 — The persistence seam, single-writer safety, and schema versioning are sound
Durable storage plugs into the existing `ChirpRepository` seam, which covers the release-bar capabilities;
durability holds under more than one replica or single-writer is enforced; the durable schema is versioned
with a forward migration path.
- **references:** REQ-DOM-005, REQ-DOM-014, REQ-DOM-018, REQ-DOM-022, REQ-OPS-005
- **verification_method:** inspection
- **stage:** releasable

#### BAR-022 — Retention honours the account's lifetime and capacity fails safe
User content is retained for the life of the account; time-bounded durable state has a defined, enforced
lifetime; capacity honours the retention commitment and exhaustion fails safe; durability is demonstrable by
a repeatable drill.
- **references:** REQ-OPS-009, REQ-OPS-010, REQ-OPS-012, REQ-OPS-013
- **verification_method:** operational_check
- **stage:** releasable

### F. Authorization & application security

#### BAR-023 — A user acts only on their own resources and only as their own identity
A user may act only on resources they own and manage only their own social graph and likes; every action is
attributed to and authorized against the actor's own identity — no acting-as-another; holding an account
grants ownership-scoped authority only.
- **references:** REQ-ID-040, REQ-ID-041, REQ-ID-042, REQ-ID-045, REQ-PROD-004, REQ-DOM-020
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-024 — Authorization is server-side, default-deny, and consistent across every entry point
Every mutating action is authorized on the server and cannot be bypassed; public reads are explicitly
allowed without a session; mutations and unrecognised actions are default-denied; a verified identity is
necessary but not sufficient for a mutation; authorization is consistent across every entry point.
- **references:** REQ-ID-053, REQ-ID-054, REQ-ID-055, REQ-ID-056, REQ-ID-057
- **verification_method:** automated_test
- **stage:** usable_early

#### BAR-025 — Credentials are protected; authenticated mutations resist CSRF and stored injection
Credentials are never revealed, displayed, or logged; mutating actions cannot be forged from another site
(CSRF resistance); user-submitted content cannot execute as code in another user's browser (stored-injection
resistance).
- **references:** REQ-ID-024, REQ-ID-080, REQ-ID-081
- **verification_method:** automated_test
- **stage:** usable_early

### G. Trust, safety, privacy & the legal surface

#### BAR-026 — Users can respond to unwanted content and accounts
A signed-in user can report a chirp or an account and is told it was filed, and can reduce their own
exposure by blocking an account (with stated consequences); report and block have honest states and never
fail silently or falsely succeed; a public product that lets anyone post does not release without these.
- **references:** REQ-PROD-027, REQ-ID-060, REQ-ID-061, REQ-ID-063, REQ-ID-064, REQ-UX-062, REQ-UX-063, REQ-UX-064, REQ-DOM-011
- **verification_method:** manual_user_check
- **stage:** releasable

#### BAR-027 — A reachable, accurate privacy & terms surface states what data is held
A privacy statement of what personal data is held, and a terms-of-use statement, are reachable in both
navigation states without an account, presented plainly and readably; what is public is clear to the user;
the privacy statement is accurate to what is actually held; access/export and erasure rights are honoured.
- **references:** REQ-PROD-029, REQ-ID-068, REQ-ID-069, REQ-ID-070, REQ-ID-071, REQ-ID-073, REQ-ID-074, REQ-UX-061, REQ-UX-065, REQ-UX-066
- **verification_method:** manual_user_check
- **stage:** releasable

### H. Abuse limits & quotas

#### BAR-028 — Baseline anti-abuse limits exist and are enforced safely
A baseline limit on account creation and on posting/high-volume actions exists; a per-caller request-rate
limit is enforced and tightened for writes and sensitive endpoints; a rate-limited request gets a
well-formed, non-destructive, recoverable response; limits are communicated, never silent; limits are
configurable with documented defaults and operability surfaces are exempt.
- **references:** REQ-ID-065, REQ-ID-066, REQ-ID-067, REQ-OPS-040, REQ-OPS-041, REQ-OPS-042, REQ-OPS-047
- **verification_method:** operational_check
- **stage:** releasable

#### BAR-029 — Request payloads and collection responses are bounded
Request payload size is bounded and oversized requests are rejected early; every collection endpoint is
bounded by a maximum page size with a default.
- **references:** REQ-OPS-043, REQ-OPS-044
- **verification_method:** automated_test
- **stage:** releasable

### I. Admin & support capability

#### BAR-030 — A segregated operator surface can look up and support an account
A minimal admin/support capability exists on a surface segregated from consumer navigation: an operator can
look up a user account by handle or id and see enough detail to answer a support request, subject to a
privacy limit (data minimisation) on what the operator view exposes.
- **references:** REQ-PROD-033, REQ-ID-048, REQ-ID-072, REQ-UX-009, REQ-UX-067, REQ-UX-068
- **verification_method:** manual_user_check
- **stage:** releasable

#### BAR-031 — An operator can act on abuse, with least-privilege, attributable, revocable authority
Abuse can be acted on by whoever runs the product; named elevated authorities (moderator, operator) are
defined, least-privilege (no impersonation, no over-reach, no credential access), explicitly assigned and
revocable, and every elevated action is attributable to an identified elevated principal; a moderated
account loses authority consistent with the action, and the effect becomes visible to users.
- **references:** REQ-PROD-028, REQ-ID-046, REQ-ID-047, REQ-ID-049, REQ-ID-050, REQ-ID-051, REQ-ID-052, REQ-UX-069
- **verification_method:** manual_user_check
- **stage:** releasable

### J. Durable-data operations (backup, recovery, rollback, migration)

#### BAR-032 — Backups are scheduled, complete, monitored, and stored separately
The durable store is backed up on a defined, recent schedule; a backup captures everything the durable model
holds; backup success and freshness are monitored so a broken backup is detectable before it is needed;
backups are stored durably and separately from the primary store.
- **references:** REQ-OPS-020, REQ-OPS-021, REQ-OPS-025, REQ-OPS-026
- **verification_method:** operational_check
- **stage:** releasable

#### BAR-033 — Recovery objectives are defined and met, and restore is proven by drill
A recovery point objective (max tolerable data loss) and a recovery time objective (max tolerable time to
recover) are defined and met; restore is proven by a periodic restore drill, not merely configured; a
documented recovery runbook exists; the recovery path works on the self-hostable surface.
- **references:** REQ-OPS-022, REQ-OPS-023, REQ-OPS-024, REQ-OPS-028, REQ-OPS-033
- **verification_method:** operational_check
- **stage:** releasable

#### BAR-034 — Normal operations never destroy user data; rollback and migration are safe
Normal operations (restart/redeploy/scale) do not destroy user data; a bad deploy can be rolled back to the
previously-known-good running state without destroying user data; forward schema migrations apply safely and
a failed/bad migration has a defined recovery path; a defined recovery path exists for the service and its
data.
- **references:** REQ-PROD-031, REQ-PROD-032, REQ-OPS-029, REQ-OPS-030, REQ-OPS-031, REQ-OPS-032
- **verification_method:** operational_check
- **stage:** releasable

### K. Observability

#### BAR-035 — Health reflects whether the product can serve users, with distinct liveness/readiness
The health signal reports unhealthy when the product cannot serve users (e.g., durable store unreachable),
not merely process liveness; liveness and readiness are distinct signals with distinct meanings; readiness
actually exercises the durable dependency, is bounded, and fails closed; health recovers on its own when the
dependency returns.
- **references:** REQ-PROD-030, REQ-DOM-021, REQ-DOM-019, REQ-OPS-060, REQ-OPS-061, REQ-OPS-062, REQ-OPS-063
- **verification_method:** operational_check
- **stage:** usable_early

#### BAR-036 — Logs are structured, correlated, safe, and diagnostic
Every request produces a readable log record; logs are structured, parseable, and emitted for platform
collection; server faults are logged with enough context to diagnose and never silently; a request can be
followed across log lines by a correlation id; logs never expose secrets or personal data; lifecycle and
fatal-startup events are observable in logs.
- **references:** REQ-OPS-065, REQ-OPS-066, REQ-OPS-067, REQ-OPS-068, REQ-OPS-069, REQ-OPS-070
- **verification_method:** inspection
- **stage:** releasable

#### BAR-037 — Quantitative metrics and named SLIs make degradation visible
Quantitative request metrics are exposed for collection; degradation is visible before users feel it; the
product's health is defined as named, quantitative SLIs; observability is documented for whoever operates
the product.
- **references:** REQ-OPS-071, REQ-OPS-072, REQ-OPS-074, REQ-OPS-075
- **verification_method:** operational_check
- **stage:** releasable

### L. Incident response & honest degradation

#### BAR-038 — Trouble is detected on product health and reaches a responsible human
Product-trouble is detected on a product-health condition (not process liveness); when the product is in
trouble, a responsible human is notified within a bounded time.
- **references:** REQ-OPS-080, REQ-OPS-081
- **verification_method:** operational_check
- **stage:** releasable

#### BAR-039 — Degradation is honest to users and never loses their work
When chirp cannot serve users, users see an honest degraded/outage state rather than a broken page; during
a degradation the product fails honestly and never loses the user's work silently.
- **references:** REQ-OPS-083, REQ-OPS-084
- **verification_method:** manual_user_check
- **stage:** releasable

#### BAR-040 — Incident responsibility is defined and incidents are recorded durably
Incident-response responsibility is defined and documented; incidents are recorded durably.
- **references:** REQ-OPS-086, REQ-OPS-087
- **verification_method:** inspection
- **stage:** releasable

### M. Visual design system & product-wide consistency

#### BAR-041 — One consistent visual language, typography, colour, and spacing
The product uses one consistent visual language across all surfaces with a readable typographic hierarchy, a
colour system with sufficient contrast, and consistent spacing and a readable content column; status is
never carried by colour alone.
- **references:** REQ-UX-070, REQ-UX-071, REQ-UX-072, REQ-UX-073, REQ-UX-074
- **verification_method:** inspection
- **stage:** releasable

#### BAR-042 — A consistent component system with full interaction and feedback states
A button system with full interaction states; destructive actions are distinct and confirmed; inputs with
inline validation styling and character-limit feedback that never loses the draft; one consistent chirp card
everywhere; consistent identity presentation with a placeholder avatar; follow and like controls reflect the
true state and count; standard state blocks and an honest recoverable error state with retry; uniform
submission feedback; standard feedback surfaces (dialog, toast, inline alert); consistent, labelled
iconography; visible focus on every interactive element.
- **references:** REQ-UX-075, REQ-UX-076, REQ-UX-077, REQ-UX-079, REQ-UX-080, REQ-UX-083, REQ-UX-084, REQ-UX-085, REQ-UX-086, REQ-UX-087, REQ-UX-088
- **verification_method:** inspection
- **stage:** releasable

### N. Responsive layout

#### BAR-043 — The product is usable on a small phone without losing state (mobile floor)
One readable content column reflows without horizontal scroll; the timeline reflows to a usable single-column
feed; compose is reachable and usable at every breakpoint; the product is usable on a small phone in
portrait and landscape without losing state; forms and compose remain usable with the on-screen keyboard;
touch targets meet the minimum size and spacing.
- **references:** REQ-UX-091, REQ-UX-093, REQ-UX-094, REQ-UX-098, REQ-UX-099, REQ-UX-103
- **verification_method:** manual_user_check
- **stage:** usable_early

#### BAR-044 — Every surface reflows correctly across the defined breakpoint set
A defined responsive breakpoint set exists that the whole product reflows against; navigation reflows while
keeping every surface reachable; profile, forms/settings, and modals/dialogs reflow to usable layouts;
hover-only affordances have a touch-accessible equivalent; the admin/support surface reflows without breaking
the app layout; avatars and media scale to the layout and never overflow.
- **references:** REQ-UX-090, REQ-UX-092, REQ-UX-095, REQ-UX-096, REQ-UX-097, REQ-UX-100, REQ-UX-101, REQ-UX-102
- **verification_method:** manual_user_check
- **stage:** releasable

### O. Accessibility (WCAG 2.1 AA)

#### BAR-045 — Full keyboard operability and managed, visible focus
Full keyboard operability with no keyboard trap; a visible focus indicator meeting non-text contrast on
every background; logical focus order and a skip-to-content link; managed focus for overlays, async
surfaces, and route changes.
- **references:** REQ-UX-106, REQ-UX-107, REQ-UX-108, REQ-UX-109
- **verification_method:** manual_user_check
- **stage:** releasable

#### BAR-046 — Correct structure, names, and announcements for assistive technology
Landmark regions and a correct heading hierarchy; accurate name, role, and value for every control;
asynchronous state changes are announced; accessible forms with labels, instructions, and actionable
announced errors; text alternatives for non-text content.
- **references:** REQ-UX-110, REQ-UX-111, REQ-UX-112, REQ-UX-116, REQ-UX-117
- **verification_method:** inspection
- **stage:** releasable

#### BAR-047 — Perceivable content: contrast, not-colour-alone, text resize, and motion respect
Text and UI contrast meet AA; information is never conveyed by colour alone; text resizes and content
reflows without loss; motion and timing respect user needs; the conformance target is stated as WCAG 2.1 AA.
- **references:** REQ-UX-105, REQ-UX-113, REQ-UX-114, REQ-UX-115, REQ-UX-118
- **verification_method:** manual_user_check
- **stage:** releasable

#### BAR-048 — Accessibility is verified across the whole gating screen inventory
Accessibility conformance is verified across every gating screen in the inventory, not sampled on one or two.
- **references:** REQ-UX-119
- **verification_method:** manual_user_check
- **stage:** releasable

### P. Domain & application structure

#### BAR-049 — chirp remains one self-contained service with a preserved wire contract
chirp remains a single self-contained service with defined entry points; the REST/wire contract is preserved
and extended only through `chirp-types`.
- **references:** REQ-DOM-017, REQ-DOM-019
- **verification_method:** inspection
- **stage:** usable_early

### Q. Self-hostable surface

#### BAR-050 — chirp builds reproducibly and runs from documented steps
chirp is distributed as source plus a self-contained image with a build-run-deploy path; it can be built
reproducibly and run from documented steps, and configured and operated from what the repository provides.
- **references:** REQ-PROD-034
- **verification_method:** inspection
- **stage:** releasable

---

### 2.1 Coverage of release-gating requirements by the bar

Every release-gating `REQ-*` in the five concern designs is enforced by at least one `BAR-*` above. The map
below is the evidence that every release-gating requirement is covered by the bar; the inverse map (BAR → REQ) is the `references` list on each criterion and is the evidence that each criterion is testable. Each `BAR-*` carries only real,
schema-conformant `REQ-*` ids in its `references` line.

**REQ-PROD (gating: 001–021, 026–034):**
001→BAR-001; 002→BAR-001; 003→BAR-001; 004→BAR-002/023; 005→BAR-004; 006→BAR-003; 007→BAR-006;
008→BAR-007; 009→BAR-008; 010→BAR-009; 011→BAR-010; 012→BAR-009; 013→BAR-011; 014→BAR-011; 015→BAR-011;
016→BAR-012; 017→BAR-012; 018→BAR-013; 019→BAR-014; 020→BAR-015; 021→BAR-015; 026→BAR-019; 027→BAR-026;
028→BAR-031; 029→BAR-027; 030→BAR-035; 031→BAR-034; 032→BAR-034; 033→BAR-030; 034→BAR-050.

**REQ-DOM (gating: 001–006, 008–022):**
001→BAR-020; 002→BAR-020; 003→BAR-020; 004→BAR-020; 005→BAR-021; 006→BAR-003; 008→BAR-007; 009→BAR-009;
010→BAR-010; 011→BAR-026; 012→BAR-019; 013→BAR-019; 014→BAR-021; 015→BAR-020; 016→BAR-020; 017→BAR-049;
018→BAR-021; 019→BAR-035/049; 020→BAR-023; 021→BAR-035; 022→BAR-021.

**REQ-UX (gating: 001–007, 009–012, 015–046, 052–088, 090–103, 105–119):**
001→BAR-018; 002→BAR-011; 003→BAR-018; 004→BAR-008; 005→BAR-004; 006→BAR-008;
007→BAR-018; 009→BAR-030; 010→BAR-018; 011→BAR-018; 012→BAR-012; 015→BAR-003; 016→BAR-017; 017→BAR-017; 018→BAR-017;
019→BAR-016; 020→BAR-016; 021→BAR-016; 022→BAR-016; 023→BAR-016; 024→BAR-016; 025→BAR-016; 026→BAR-003;
027→BAR-017; 028→BAR-017; 029→BAR-017; 030→BAR-011; 031→BAR-011; 032→BAR-011; 033→BAR-001; 034→BAR-001;
035→BAR-001; 036→BAR-002; 037→BAR-002; 038→BAR-006; 039→BAR-006; 040→BAR-013; 041→BAR-013; 042→BAR-003;
043→BAR-012; 044→BAR-012; 045→BAR-014; 046→BAR-014; 052→BAR-008; 053→BAR-007; 054→BAR-007; 055→BAR-009;
056→BAR-009; 057→BAR-010; 058→BAR-010; 059→BAR-004; 060→BAR-003; 061→BAR-027; 062→BAR-026; 063→BAR-026;
064→BAR-026; 065→BAR-027; 066→BAR-027; 067→BAR-030; 068→BAR-030; 069→BAR-031; 070→BAR-041; 071→BAR-041;
072→BAR-041; 073→BAR-041; 074→BAR-041; 075→BAR-042; 076→BAR-042; 077→BAR-042; 078→BAR-012; 079→BAR-042;
080→BAR-042; 081→BAR-015; 082→BAR-015; 083→BAR-042; 084→BAR-042; 085→BAR-042; 086→BAR-042; 087→BAR-042;
088→BAR-042; 090→BAR-044; 091→BAR-043; 092→BAR-044; 093→BAR-043; 094→BAR-043; 095→BAR-044; 096→BAR-044;
097→BAR-044; 098→BAR-043; 099→BAR-043; 100→BAR-044; 101→BAR-044; 102→BAR-044; 103→BAR-043; 105→BAR-047;
106→BAR-045; 107→BAR-045; 108→BAR-045; 109→BAR-045; 110→BAR-046; 111→BAR-046; 112→BAR-046; 113→BAR-047;
114→BAR-047; 115→BAR-047; 116→BAR-046; 117→BAR-046; 118→BAR-047; 119→BAR-048.

**REQ-ID (gating: 001–017, 020–035, 037, 040–061, 063–074, 080–081):**
001→BAR-001; 002→BAR-001; 003→BAR-001; 004→BAR-001; 005→BAR-001; 006→BAR-006; 007→BAR-006; 008→BAR-006;
009→BAR-006; 010→BAR-009; 011→BAR-009; 012→BAR-009; 013→BAR-009; 014→BAR-009; 015→BAR-010; 016→BAR-010;
017→BAR-010; 020→BAR-002; 021→BAR-002; 022→BAR-002; 023→BAR-002; 024→BAR-025; 025→BAR-005; 026→BAR-003;
027→BAR-003; 028→BAR-003; 029→BAR-003; 030→BAR-003; 031→BAR-004; 032→BAR-005; 033→BAR-005; 034→BAR-005;
035→BAR-003; 037→BAR-006; 040→BAR-023; 041→BAR-023; 042→BAR-023; 043→BAR-009; 044→BAR-010; 045→BAR-023;
046→BAR-031; 047→BAR-031; 048→BAR-030; 049→BAR-031; 050→BAR-031; 051→BAR-031; 052→BAR-031; 053→BAR-024;
054→BAR-024; 055→BAR-024; 056→BAR-024; 057→BAR-024; 060→BAR-026; 061→BAR-026; 063→BAR-026; 064→BAR-026;
065→BAR-028; 066→BAR-028; 067→BAR-028; 068→BAR-027; 069→BAR-027; 070→BAR-027; 071→BAR-027; 072→BAR-030;
073→BAR-027; 074→BAR-027; 080→BAR-025; 081→BAR-025.

**REQ-OPS (gating: 001–013, 020–026, 028–033, 040–044, 047, 060–063, 065–072, 074–075, 080–081, 083–084, 086–087):**
001→BAR-019; 002→BAR-019; 003→BAR-019; 004→BAR-019; 005→BAR-021; 006→BAR-019; 007→BAR-019; 008→BAR-020;
009→BAR-022; 010→BAR-022; 011→BAR-009; 012→BAR-022; 013→BAR-022; 020→BAR-032; 021→BAR-032; 022→BAR-033;
023→BAR-033; 024→BAR-033; 025→BAR-032; 026→BAR-032; 028→BAR-033; 029→BAR-034; 030→BAR-034; 031→BAR-034;
032→BAR-034; 033→BAR-033; 040→BAR-028; 041→BAR-028; 042→BAR-028; 043→BAR-029; 044→BAR-029; 047→BAR-028;
060→BAR-035; 061→BAR-035; 062→BAR-035; 063→BAR-035; 065→BAR-036; 066→BAR-036; 067→BAR-036; 068→BAR-036;
069→BAR-036; 070→BAR-036; 071→BAR-037; 072→BAR-037; 074→BAR-037; 075→BAR-037; 080→BAR-038; 081→BAR-038;
083→BAR-039; 084→BAR-039; 086→BAR-040; 087→BAR-040.

No release-gating `REQ-*` is left uncovered. Non-gating requirements (listed in §1.4) are intentionally not
enforced by any `BAR-*`.

---

## 3. Completeness & gap review

*(section: `completeness_and_gap_review`)*

This is a deliberate sweep over the surfaces a product of chirp's kind — a public, account-holding consumer
web application with a self-hostable surface — is expected to have. For each surface the review records
whether it is **covered** (and where), **out of scope** (with a reason), or a **gap** that must be closed.
The sweep is the Product-Surfaces lens applied to the *whole* design, independent of what any one concern
file happened to cover, so that an absence is a recorded decision rather than an oversight.

### 3.1 The expected-surface register

| # | Surface | Verdict | Where it lives (or the reason) |
|---|---------|---------|-------------------------------|
| 1 | **Sign-up** (self-service, in-product) | **Covered** | BAR-001; REQ-PROD-001–003, REQ-ID-001–005, REQ-UX-033–035. Closes today's API-only creation. |
| 2 | **Sign-in** (credentialed) | **Covered** | BAR-002; REQ-PROD-004, REQ-ID-020–023, REQ-UX-036–037. Closes today's handle-only sign-in. |
| 3 | **Sign-out** | **Covered** | BAR-004; REQ-PROD-005, REQ-ID-031, REQ-UX-005/059. |
| 4 | **Account recovery** (lost credential) | **Covered** | BAR-006; REQ-PROD-007, REQ-ID-006–009/037, REQ-UX-038–039. |
| 5 | **Sessions** (durable, bounded, honest) | **Covered** | BAR-003/005; REQ-ID-026–035, REQ-DOM-006. Closes today's restart-lost sessions. |
| 6 | **SSO / federated identity** | **Out of scope** | REQ-ID-039: a deliberate scope decision tied to the product kind (a single shared consumer product, not B2B SaaS). Recorded, not omitted. |
| 7 | **Multi-factor authentication** | **Out of scope (non-gating)** | REQ-ID-038: deferred-not-precluded; a recorded scope decision for this product kind. |
| 8 | **Profile view & edit** | **Covered** | BAR-007; REQ-PROD-008, REQ-DOM-008, REQ-UX-053–054. |
| 9 | **Settings / account-management hub** | **Covered** | BAR-008; REQ-PROD-009, REQ-UX-004/006/052. |
| 10 | **Account deletion** (self-service, honoured) | **Covered** | BAR-009; REQ-PROD-010/012, REQ-ID-010–014, REQ-DOM-009, REQ-OPS-011, REQ-UX-055–056. |
| 11 | **Data export / portability** | **Covered** | BAR-010; REQ-PROD-011, REQ-ID-015–017, REQ-DOM-010, REQ-UX-057–058. |
| 12 | **Right to erasure / access** | **Covered** | BAR-027/009; REQ-ID-070–071. |
| 13 | **Onboarding / first-run** | **Out of scope (non-gating)** | REQ-PROD-022, REQ-UX-014/049: a first version may ship a functional-but-empty first session; onboarding is a tracked improvement, not a blocker. The empty *state* itself is gating and is covered (BAR-016). |
| 14 | **The core loop** (read/post/timeline/follow/like) | **Covered** | BAR-011–015; REQ-PROD-013–021. |
| 15 | **Honest empty/error/loading/not-found states** | **Covered** | BAR-016; REQ-UX-019–025. |
| 16 | **Search / discovery** | **Out of scope (non-gating)** | REQ-PROD-024, REQ-UX-008/050/051/104: the core loop launches with on-screen-links discovery; search is a tracked near-term addition. |
| 17 | **Replies / conversations** | **Out of scope (non-gating)** | REQ-PROD-023, REQ-DOM-007, REQ-UX-013/047/048: may be deferred provided no dead reply affordance ships (that no-dead-affordance condition is itself held by BAR-016/018). |
| 18 | **User-facing notifications** (replies, new followers) | **Out of scope (non-gating)** | REQ-PROD (richer-social cut, §5): a natural Twitter-style extension, not required for the first releasable loop; the most likely near-term addition. (Distinct from *operator* notification/alerting, which **is** gating — BAR-038.) |
| 19 | **Report / flag abuse** | **Covered** | BAR-026; REQ-PROD-027, REQ-ID-060/063/064, REQ-UX-062/064. |
| 20 | **Block / mute** | **Covered (block gating; mute non-gating)** | BAR-026; block = REQ-ID-061/REQ-UX-063 (gating); mute = REQ-ID-062 (recorded non-gating complement). |
| 21 | **Moderation & its visible effect** | **Covered** | BAR-031/026; REQ-PROD-028, REQ-ID-046–052/063, REQ-DOM-011, REQ-UX-069. |
| 22 | **Privacy policy & terms of use** | **Covered** | BAR-027; REQ-PROD-029, REQ-ID-068–074, REQ-UX-065–066. |
| 23 | **Admin / support capability** | **Covered** | BAR-030; REQ-PROD-033, REQ-ID-048/072, REQ-UX-009/067–068. |
| 24 | **Billing / payments / paid tiers** | **Out of scope** | REQ-PROD (§5), REQ-ID-1062: chirp has no commercial model — nothing to charge for, no entitlement/billing authority to design. Revisit only if monetization is introduced. |
| 25 | **Rate limits / quotas / payload bounds** | **Covered** | BAR-028/029; REQ-ID-065–067, REQ-OPS-040–044/047. |
| 26 | **Durable persistence** | **Covered** | BAR-019–022; REQ-DOM-012–016/022, REQ-OPS-001–013. The current in-memory store is the headline blocker. |
| 27 | **Backups & restore (proven)** | **Covered** | BAR-032/033; REQ-OPS-020–026/028/033. |
| 28 | **Deploy rollback & schema migration safety** | **Covered** | BAR-034; REQ-OPS-029–032, REQ-PROD-031–032. |
| 29 | **Observability** (health/logs/metrics/SLIs) | **Covered** | BAR-035–037; REQ-OPS-060–063/065–075, REQ-PROD-030, REQ-DOM-021. |
| 30 | **Incident response & honest degradation** | **Covered** | BAR-038–040; REQ-OPS-080–081/083–084/086–087. |
| 31 | **Authorization** (own-resource, server-side, default-deny) | **Covered** | BAR-023/024; REQ-ID-040–045/053–057, REQ-DOM-020. |
| 32 | **Application security** (CSRF, XSS, credential handling, fixation) | **Covered** | BAR-005/025; REQ-ID-024/025/032/033/080/081. |
| 33 | **Threat model** | **Covered** | The identity-security design states the threat model that BAR-023–025/028 mitigate; it is a design artifact, not a separately testable live surface. |
| 34 | **Accessibility** (WCAG 2.1 AA) | **Covered** | BAR-045–048; REQ-UX-105–119. |
| 35 | **Responsive layout** | **Covered** | BAR-043/044; REQ-UX-090–103. |
| 36 | **Visual design system & consistency** | **Covered** | BAR-041/042; REQ-UX-070–088. |
| 37 | **Self-hostable build/run/operate** | **Covered** | BAR-050; REQ-PROD-034. |
| 38 | **Multi-tenancy / organizations / teams** | **Out of scope** | REQ-PROD (§5): chirp is a single shared consumer product with no tenant/team notion. Watch-point if org accounts are ever added. |
| 39 | **Open-source community governance** (contributor guide, license, triage) | **Out of scope** | REQ-PROD (§5): chirp is a closed demonstration that does not accept contributions; the outward reproducible-build shape is retained via the self-hostable surface (BAR-050). |
| 40 | **COTS licensing / entitlement / paid upgrades / commercial support** | **Out of scope** | REQ-PROD (§5): chirp is not sold, licensed, or commercially supported; the self-hostable build/operate expectations still apply (BAR-050). |
| 41 | **Federation / interoperability with other networks** | **Out of scope** | REQ-PROD (§5): not part of chirp's product aim. |
| 42 | **Direct messages / media upload / hashtags & trending** | **Out of scope (non-gating)** | REQ-PROD (§5): richer-social extensions beyond the core loop; later work. |

### 3.2 Findings of the sweep

- **No uncovered expected surface for this product kind remains.** Every surface a public, account-holding
  consumer web app is expected to have is either enforced by a `BAR-*` (surfaces 1–5, 8–12, 14–15, 19–23,
  25–37) or recorded out of scope with a reason (6–7, 13, 16–18, 24, 38–42). There is no silent absence.
- **The single largest gap between the current product and the bar is durable persistence** (surface 26,
  BAR-019). The repository runs an in-memory store, so on the live product today
  accounts, chirps, follows, likes, and sessions do not survive a restart. This is a `usable_early`-stage
  criterion: chirp is not even *usable-early* until it is closed.
- **The second structural gap is real identity**: today sign-in is handle-only and anyone who types a handle
  becomes that user (REQ-PROD-004 acceptance note). BAR-001/002/023/024 close this; it is also
  `usable_early`.
- **Operational, trust/safety, and accessibility surfaces are the bulk of the `releasable` work** on top of
  a usable-early product: recovery/deletion/export, report/block, privacy & terms, admin/support, rate
  limits, backups/restore/rollback/migration, observability/incident, WCAG AA, and full responsive. None of
  these is optional for a responsible release; each is a recorded `releasable` criterion above.
- **Every out-of-scope verdict carries a reason and, where relevant, a watch-point** (e.g., multi-tenancy
  returns if org accounts are added; billing returns if monetization is introduced), so a future reviewer
  can tell a deliberate cut from an oversight.

### 3.3 How this bar is used downstream

- The **delivery backlog** turns each `BAR-*` (and the `REQ-*` it references) into issues whose acceptance
  criteria cite them, grouped into epics that reach a *usable-early* product first and the full *releasable*
  bar after; the `stage` field is the natural cut line between the early epics and the rest.
- **Release-readiness measurement** evaluates the live product at the pinned commit against each `BAR-*` using its named
  `verification_method`, records a `live_status` (met/partial/missing/not_applicable) per the traceability
  schema, and maps every gap to a filed issue — with the two `usable_early` structural gaps (durable
  persistence, real identity) expected to dominate the first measurement.
