# chirp — release bar

The release bar for chirp (`Nousix-LLC/nswe-demo`), measured against the current product at commit `c805525`. It consolidates the five area designs into one register of checkable release conditions, draws the line between what a first usable version must meet and what the public release requires on top, and records a deliberate judgment — in scope or out, with a reason — for every surface a product of this kind is expected to have.

Each criterion (`BAR-NNN`) is a condition someone who was not in the design room can check against the live product. It enforces one or more requirements (`REQ-<AREA>-NNN`) from the area designs and names how it is verified — `manual_user_check` (a person exercises the product), `automated_test` (a test in the repository asserts it), `inspection` (a declared property is read from a config, manifest, or code artifact), or `operational_check` (a property of the running service is observed, such as data surviving a restart). A criterion states what must be observably true; it does not prescribe how the product is built.

The requirements themselves live in the area designs and are not restated here: `design/product-requirements.md`, `design/domain-application-design.md`, `design/experience-design.md`, `design/identity-security.md`, `design/operational-readiness.md`.

---

## MVP-vs-releasable scoping

The register is staged on one line, applied the same way across all five areas.

**`usable_early`** marks a condition the first version a real person could actually adopt must already meet:

- the **core loop** works — a signed-in person can post a chirp, read a home timeline of themselves and the people they follow, reply, like, follow, and view a public profile;
- the person's **identity is real and owner-only** — they get in only with a secret they hold, carry a session that is safe to expose on the public internet, and can act on their own account and data and no one else's;
- **what they create survives** a restart, redeploy, or process replacement — the in-memory store at `c805525`, which loses every account, chirp, follow, and like on restart, is the headline release blocker, and closing it is part of the first usable version, not a later polish;
- the product stands on the **module and contract structure** it will keep (the four-crate partition, the one typed client/server contract, a single durable-persistence seam, one web entry point);
- the core loop **never shows a blank screen, an endless spinner, or a silent failure**, completes on a phone, and is reachable by keyboard and a screen reader; and
- the service can **run under an orchestrator and be diagnosed** — a readiness signal that reports whether it can actually serve users, a separate liveness signal, structured logs, and no secrets written to them — and it **refuses the obvious abuse** that threatens even an early build (oversized request bodies, unbounded list reads).

**`releasable`** marks what the public release must add on top:

- the rest of the **account lifecycle** — self-service recovery, the settings hub, viewing and changing the verified contact, the sessions-and-devices list, data export, and self-service account deletion;
- the **safety and trust surface** — reporting against stated in-product rules, the rules/terms screen, help and support, and the provisioned, least-privilege operator surface with an audit trail;
- **backups and recovery** — scheduled backups with stated retention, stated recovery-point and recovery-time objectives, a written restore procedure, and rehearsed restore drills;
- the **full limits and quotas** posture beyond the early floor;
- the **depth of observability** (serving metrics, a serving/availability signal, the administrative audit trail) and the **incident, status, and availability** apparatus; the finished-feeling experience beyond the core loop; and the **WCAG 2.1 AA** completion with its verified, no-open-failure release gate.

### What each stage contains, by area

| Area | `usable_early` | `releasable` | Total |
|------|---------------:|-------------:|------:|
| Product | 14 | 21 | 35 |
| Domain & application | 50 | 10 | 60 |
| Experience | 51 | 43 | 94 |
| Identity, security & trust | 18 | 22 | 40 |
| Operational readiness | 15 | 34 | 49 |
| **All areas** | **148** | **130** | **278** |

### Where the areas were reconciled

Each area proposed a stage for its own criteria; the one cross-cutting line is reconciled here. Seventeen criteria were moved from the area proposal to `usable_early` so the line holds together across areas. The cuts are stated, not hidden:

- **Durable persistence is part of the first usable version, not a later addition (14 criteria).** `BAR-130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 141, 143, 144, 145` — the survival of accounts, chirps, follows, likes, sessions, counts, and the reconstructable timeline across a restart or redeploy, durable acknowledged writes, stable ids, authoritative state in a store separate from the serving process, and a readiness signal that reflects the store — were proposed `releasable` by the domain area but are the headline `c805525` blocker. They move to `usable_early` to agree with the product area (`BAR-013`) and the operational-readiness durable-data floor (`BAR-400`–`BAR-404`, `BAR-409`, `BAR-410`), and with the plain meaning of a usable version: what a person creates is still there when they come back.
- **A safe, endable session is part of real identity (2 criteria).** `BAR-016` (sign-out ends the session on the server) and `BAR-113` (a session can be ended and does not live forever) move to `usable_early` to agree with the identity area's session floor (`BAR-306`, `BAR-307`): a session that cannot be ended or never expires leaves the next person at a shared device signed in as someone else, which is not acceptable even in an early build.
- **The sign-in signal is part of real authentication (1 criterion).** `BAR-010` (a person can tell "no such account" from "wrong secret" without leaking who exists) moves to `usable_early` to agree with the identity area's `BAR-304`.

Five domain persistence criteria were deliberately **left `releasable`**, because each rests on a release-layer capability rather than the restart-survival floor: `BAR-140` (integrity across a *restore from backup* — the restart half of its guarantee is already held early by `BAR-403`), `BAR-142` (a *deletion* stays applied durably — deletion itself is a releasable surface), `BAR-146` (*backup and restore*), `BAR-147` (the per-account enumeration behind *data export*), and `BAR-148` (an *upgrade* that changes the data shape preserves data). Staging these `releasable` hides nothing: the restart-survival floor they build on is held at `usable_early` by the criteria listed above.

### What the early version leaves for the release

A person on the **usable-early** version can find chirp, sign up with a real credential, post and read a timeline, reply, like, follow, and view profiles, on a phone or a desktop, by pointer or by keyboard and screen reader — and what they create is still there tomorrow. What they do **not** yet have until the **release** is: recovering a lost credential by themselves, a settings area, changing their contact, seeing and ending their other sessions, exporting their data, deleting their own account, reporting abuse against stated rules, and the operator, backup, incident, and status apparatus behind a service people can trust. Those are the release's to add; none of them is pretended to be present earlier.

---

## Release bar

The consolidated register, by area, in id order. Each line gives the final stage, the verification method, the release condition, and the requirement(s) it enforces.

### Product (`BAR-001..099`) — enforces `design/product-requirements.md`

_35 criteria — 14 `usable_early`, 21 `releasable`._

**BAR-001** (`usable_early` · `automated_test`) — A signed-in person can publish a chirp of 280 characters or fewer, attributed to their handle; an empty or over-limit message is refused and no chirp is created. _Enforces REQ-PROD-001._

**BAR-002** (`usable_early` · `automated_test`) — A signed-in person's home timeline returns their own chirps and those of the people they follow, newest first, in bounded pages reached by a cursor — never the entire history in one response. _Enforces REQ-PROD-002._

**BAR-003** (`usable_early` · `automated_test`) — A reply records the chirp it answers; a reply to a chirp that no longer exists is reported as not-found rather than accepted as though it succeeded. _Enforces REQ-PROD-003._

**BAR-004** (`usable_early` · `automated_test`) — Liking a chirp increases its like count by one; liking it again does not increase the count further; unliking returns the count — the like is idempotent and the displayed count matches the result. _Enforces REQ-PROD-005._

**BAR-005** (`usable_early` · `automated_test`) — Following a user brings that user's new chirps into the follower's timeline and is idempotent; unfollowing removes them; a person cannot follow their own account. _Enforces REQ-PROD-006._

**BAR-006** (`usable_early` · `automated_test`) — A user's public profile — handle, display name, bio, and follower/following counts — is viewable with no session. _Enforces REQ-PROD-007._

**BAR-007** (`usable_early` · `automated_test`) — Sign-up establishes a secret the person later uses to prove the account is theirs; an account created at sign-up cannot be entered by knowledge of its handle alone. _Enforces REQ-PROD-008._

**BAR-008** (`usable_early` · `automated_test`) — Sign-up enforces handle format (up to 15 letters, digits, or underscores) and uniqueness, reporting a taken handle as a conflict naming it and an ill-formed handle as a field-level error; a valid, unused handle is accepted. _Enforces REQ-PROD-009._

**BAR-009** (`usable_early` · `automated_test`) — Presenting only another account's handle, without its credential, grants no session; the legitimate owner presenting the credential set at sign-up or recovery is signed in. _Enforces REQ-PROD-010._

**BAR-010** (`usable_early` · `automated_test`) — A sign-in attempt lets the person tell "no such account" from "credential did not match" — enough to distinguish a typo from a wrong secret — without revealing which accounts exist beyond what the product already makes public. _Enforces REQ-PROD-011._

**BAR-011** (`usable_early` · `automated_test`) — Signed in as one account, every attempt through the product's surfaces to act as another account — post, reply, like, or follow as them, edit their profile or settings, or delete their content — is refused. _Enforces REQ-PROD-023._

**BAR-012** (`usable_early` · `automated_test`) — A person's contact, credential, and session data are never returned to another user or in any public response; only the public fields (handle, display name, bio, public chirps, counts) are exposed. _Enforces REQ-PROD-024._

**BAR-013** (`usable_early` · `operational_check`) — After the service is restarted, updated, or redeployed, an account created beforehand still exists and its chirps, follows, and likes are still present on the next sign-in. The in-memory store at `c805525`, which loses everything on restart, does not meet this. _Enforces REQ-PROD-037._

**BAR-014** (`releasable` · `manual_user_check`) — An account has a point of contact that the product confirms the person controls (a verification step) before it is relied on for recovery or important notices; the contact is kept minimal. _Enforces REQ-PROD-012._

**BAR-015** (`releasable` · `manual_user_check`) — A person who has lost their credential can regain access themselves by proving control of the verified contact, with no one acting on their behalf; a person without that verified contact cannot recover the account. _Enforces REQ-PROD-013._

**BAR-016** (`usable_early` · `automated_test`) — After a person signs out, the session that was in use can no longer be used — replaying the prior request with it is rejected — so the next person at a shared device is not signed in as them. _Enforces REQ-PROD-014._

**BAR-017** (`releasable` · `automated_test`) — A signed-in person can change their display name and bio, and the change persists and shows on their profile on the next view. _Enforces REQ-PROD-016._

**BAR-018** (`releasable` · `manual_user_check`) — A signed-in person has a settings surface where they can view and update their verified contact and view and end their active sessions or devices. _Enforces REQ-PROD-017._

**BAR-019** (`releasable` · `manual_user_check`) — A signed-in person can obtain their own data — their chirps, profile, and follow list — in a portable form they can open and read outside chirp. _Enforces REQ-PROD-019._

**BAR-020** (`releasable` · `automated_test`) — A signed-in person can delete their own account; afterward the account can no longer sign in and the person's content is removed per the stated deletion outcome. _Enforces REQ-PROD-020._

**BAR-021** (`releasable` · `inspection`) — The product states a definite outcome for deletion — what becomes of the person's chirps that others replied to or liked, and whether the freed handle can be re-registered — and its behavior after a deletion matches that stated outcome. _Enforces REQ-PROD-021._

**BAR-022** (`releasable` · `manual_user_check`) — Deletion is confirmed before it happens and explains what will and will not survive — including that export is the person's chance to keep a copy; cancelling at the confirmation deletes nothing. _Enforces REQ-PROD-022._

**BAR-023** (`releasable` · `manual_user_check`) — A signed-in person can report a chirp and report an account, and each submission is acknowledged as received. _Enforces REQ-PROD-025._

**BAR-024** (`releasable` · `inspection`) — The product states the rules — what is and is not allowed — within the product, and a report can point to them. _Enforces REQ-PROD-026._

**BAR-025** (`releasable` · `manual_user_check`) — Through a supported interface, the operator can assist a user's recovery, remove a named chirp, and disable or action an abusive account, none of which requires editing stored data by hand. _Enforces REQ-PROD-028._

**BAR-026** (`releasable` · `manual_user_check`) — A visitor arriving with no account and no session sees an explanation of what chirp is and a clear path to sign up, rather than being stopped at a sign-in wall. _Enforces REQ-PROD-030._

**BAR-027** (`releasable` · `manual_user_check`) — Immediately after sign-up, with no follows and no posts, the person sees guidance explaining the empty state and at least one actionable way toward a non-empty timeline, not a blank page. _Enforces REQ-PROD-032._

**BAR-028** (`releasable` · `manual_user_check`) — An empty list, a loading list, and a failed action each present a state the person can understand and act on, rather than a blank area or a silent failure. _Enforces REQ-PROD-033._

**BAR-029** (`releasable` · `manual_user_check`) — An expired or invalid session routes the person to sign-in with their place explained rather than failing silently; a transient server error while posting can be recovered by retry without losing the drafted chirp. _Enforces REQ-PROD-034._

**BAR-030** (`releasable` · `manual_user_check`) — On a typical phone viewport, a person can read the timeline, compose and post, reply, like, follow, and view a profile with no broken layout or unreachable controls. _Enforces REQ-PROD-035._

**BAR-031** (`releasable` · `manual_user_check`) — Using keyboard navigation and a screen reader, a person can complete the core loop (read the timeline, compose and post, reply, like, follow, view a profile), and the controls expose accessible names and roles, to the baseline the experience design sets. _Enforces REQ-PROD-036._

**BAR-032** (`releasable` · `operational_check`) — An operator can produce a backup of user data and restore from it, bringing back accounts, chirps, and follows. _Enforces REQ-PROD-038._

**BAR-033** (`releasable` · `operational_check`) — The health/readiness signal reports not-ready when the product cannot actually serve users (for example its durable store is unavailable) and ready when it can — it means "serving users," not merely "the process is running." _Enforces REQ-PROD-039._

**BAR-034** (`releasable` · `operational_check`) — Over a sustained period the product responds at its address with the core loop working; unavailability is the exception, measured against the availability target the operational-readiness design sets. _Enforces REQ-PROD-040._

**BAR-035** (`releasable` · `operational_check`) — Posting or requesting beyond a reasonable rate is throttled or rejected rather than served without limit, so one actor cannot trivially overwhelm the service or flood others. _Enforces REQ-PROD-041._

---

### Domain & application (`BAR-100..199`) — enforces `design/domain-application-design.md`

_60 criteria — 50 `usable_early`, 10 `releasable`._

**BAR-100** (`usable_early` · `manual_user_check`) — Registering a person yields exactly one account carrying an opaque id, a handle, a display name, an optional bio (absent and empty-string are distinguishable), a creation time, and follower/following counts; no second "profile" entity exists, and that single entity is both the public profile others see and the account its holder manages. _Enforces REQ-DOM-001, REQ-DOM-004, REQ-DOM-005._

**BAR-101** (`usable_early` · `manual_user_check`) — A handle of 1–15 characters, each an ASCII letter, digit, or underscore, is accepted; an empty, over-length, or otherwise invalid handle is refused against the handle field; and registering a handle already in use is refused as a conflict, so at most one account holds any handle. _Enforces REQ-DOM-002, REQ-DOM-003._

**BAR-102** (`usable_early` · `manual_user_check`) — Account ids and chirp ids are assigned by the product and are opaque to clients; no create surface accepts a client-supplied id, and a client only echoes back an id the product issued. _Enforces REQ-DOM-006._

**BAR-103** (`usable_early` · `automated_test`) — Once assigned to an entity, an account id or chirp id is never later observed on a different entity, including after the original entity is deleted. _Enforces REQ-DOM-007._

**BAR-104** (`usable_early` · `manual_user_check`) — Every account and chirp carries a creation time in UTC epoch-milliseconds that does not change after creation, and two entities created in sequence carry times whose order matches the order they were created, so timestamps are directly comparable for ordering such as the newest-first timeline. _Enforces REQ-DOM-008, REQ-DOM-039._

**BAR-105** (`usable_early` · `manual_user_check`) — A signed-in holder can change their display name and bio and see the change on the next view, and no product surface ever changes an account's id or creation time. _Enforces REQ-DOM-009._

**BAR-106** (`usable_early` · `manual_user_check`) — Posting a chirp of 1–280 characters (counted by Unicode scalar value) attributed to the signed-in account yields a chirp carrying an id, that author, the body as written, a creation time, a like count of zero, and no reply target; an empty or over-length body is refused and no chirp is created; and after posting, the chirp's body and author never change. _Enforces REQ-DOM-011, REQ-DOM-012, REQ-DOM-013, REQ-DOM-014._

**BAR-107** (`usable_early` · `manual_user_check`) — A reply records the id of exactly one chirp that exists at reply time; replying to a chirp id that does not exist or has been removed is refused as not-found, and no reply is created with a dangling target. _Enforces REQ-DOM-015._

**BAR-108** (`usable_early` · `manual_user_check`) — A follow is recorded in one direction from one existing account to another; following one's own account is refused; following an account already followed changes nothing and does not duplicate the edge or double a count; and unfollowing removes the edge so the followee's chirps no longer enter the follower's timeline (unfollowing an account not followed changes nothing). _Enforces REQ-DOM-017, REQ-DOM-018, REQ-DOM-019, REQ-DOM-020._

**BAR-109** (`usable_early` · `manual_user_check`) — An account's follower count equals the number of accounts currently following it and its following count equals the number it currently follows; one follow raises the count by exactly one and an unfollow returns it. _Enforces REQ-DOM-021._

**BAR-110** (`usable_early` · `manual_user_check`) — Liking a chirp records one like from the account to that existing chirp; liking a chirp already liked changes nothing and does not add a second like; unliking removes it; and the chirp's like count equals the number of distinct accounts that currently like it, moving by exactly one on a like or unlike. _Enforces REQ-DOM-022, REQ-DOM-023, REQ-DOM-024._

**BAR-111** (`usable_early` · `manual_user_check`) — A viewer's home timeline contains the viewer's own chirps and the chirps of every account the viewer currently follows and no others; it is ordered newest-first; it is returned a bounded page at a time with an opaque continuation cursor when more remain (a client cannot force every chirp into one response); and because it is derived, following or unfollowing an account changes what the next read returns with no stale materialized timeline. _Enforces REQ-DOM-025, REQ-DOM-026, REQ-DOM-027, REQ-DOM-028._

**BAR-112** (`usable_early` · `manual_user_check`) — A session authenticates requests as exactly one account, and every action taken under it is attributed to that account and no other; the session is presented as an opaque bearer token whose internal structure no client surface interprets. _Enforces REQ-DOM-029, REQ-DOM-032._

**BAR-113** (`usable_early` · `manual_user_check`) — A signed-in person can end their session (sign out) after which a request carrying that session's token is refused, and a session presented after its bounded lifetime has elapsed is refused and the person is returned to sign-in. Sign-out that invalidates the session is a release surface the product does not provide at `c805525`. _Enforces REQ-DOM-030, REQ-DOM-031._

**BAR-114** (`usable_early` · `manual_user_check`) — Each account has an associated authentication credential the holder proves themselves with, distinct from the handle, so that knowing only an account's handle never establishes a session for it; the credential value is never returned in any profile or public response. _Enforces REQ-DOM-033._

**BAR-115** (`releasable` · `manual_user_check`) — Each account has an associated point of contact the product treats as verified before relying on it for account recovery; that contact value is never returned to another account or in a public response. A verified contact and the recovery it supports are a release surface the product does not provide at `c805525`. _Enforces REQ-DOM-034._

**BAR-116** (`usable_early` · `manual_user_check`) — An account's credential, verified contact, and session data never appear in any public response or in another account's view; a profile fetched with no session or with a different account's session returns only the handle, display name, bio, follower/following counts, and public chirps. _Enforces REQ-DOM-035._

**BAR-117** (`usable_early` · `manual_user_check`) — Through every product surface that creates a relationship, a chirp's author, a reply's target, a follow's two endpoints, and a like's account and chirp all name entities that exist or the action is refused; no surface produces a chirp, follow, reply, or like that names a nonexistent entity. _Enforces REQ-DOM-036._

**BAR-118** (`releasable` · `manual_user_check`) — The product states a single deletion outcome — what becomes of the account's chirps (including those others replied to or liked), its follow edges, and its likes, and whether its freed handle may be re-registered — and deleting an account applies exactly that outcome: the account can no longer sign in, the product's state matches the stated outcome, and no remaining entity references the deleted account in a way the outcome did not define. Account deletion is a release surface the product does not provide at `c805525`. _Enforces REQ-DOM-037._

**BAR-119** (`releasable` · `manual_user_check`) — A signed-in person can obtain their own data — their profile, the full set of their chirps, and the full set of accounts they follow — because that data is enumerable per account. A self-service export is a release surface the product does not provide at `c805525`. _Enforces REQ-DOM-038._

**BAR-120** (`usable_early` · `inspection`) — An account's follower and following counts and a chirp's like count are maintained by the product and are never honored as client input; a create or update request carrying such a count has no effect on the stored value. _Enforces REQ-DOM-040._

**BAR-130** (`usable_early` · `operational_check`) — On a running deployment, an account created with chirps, follows, and likes is still present and unchanged — account, chirps, follows, likes, and active session — after the serving process is restarted and, separately, after it is redeployed; a person's content is intact on their next visit. This closes the `c805525` release blocker: the in-memory store loses everything on restart. _Enforces REQ-DOM-100._

**BAR-131** (`usable_early` · `operational_check`) — After a restart, an account reads back with the same id, handle, display name, bio (preserved as given, absent or present), and creation time, and its follower/following counts are unchanged; an account registered with no bio still reads back with its bio absent rather than an empty string. _Enforces REQ-DOM-101._

**BAR-132** (`usable_early` · `operational_check`) — An account's credential and verified contact persist across a restart so the holder can still sign in and still recover their account afterward, and that data never appears in any public response before or after the restart. _Enforces REQ-DOM-102._

**BAR-133** (`usable_early` · `operational_check`) — After a restart, each chirp reads back with the same id, author, body text, and creation time; a reply still records the id of the chirp it answers; and like counts are unchanged. _Enforces REQ-DOM-103._

**BAR-134** (`usable_early` · `operational_check`) — After a restart, every follow edge is still recorded in the same direction, the followee's chirps still enter the follower's timeline, and both accounts' follower/following counts are unchanged. _Enforces REQ-DOM-104._

**BAR-135** (`usable_early` · `operational_check`) — After a restart, each like is still recorded from the same account to the same chirp and the chirp's like count is unchanged. _Enforces REQ-DOM-105._

**BAR-136** (`usable_early` · `operational_check`) — A session obtained before a restart is still accepted afterward and still acts as the same account, so a redeploy does not silently sign a person out; a session that was ended (signed out) or has passed its lifetime is refused after a restart and is never revived by one. _Enforces REQ-DOM-106._

**BAR-137** (`usable_early` · `operational_check`) — After a restart, each account's follower and following counts equal its current follow edges and each chirp's like count equals the distinct accounts that currently like it; a follow or like added or removed after the restart moves the relevant count by exactly one. _Enforces REQ-DOM-107._

**BAR-138** (`usable_early` · `operational_check`) — A timeline noted before a restart returns the same chirps in the same newest-first order after the restart with no further writes, reconstructed from the durable chirps and the durable follow graph; no separately stored timeline is required. _Enforces REQ-DOM-108._

**BAR-139** (`usable_early` · `operational_check`) — A write the product confirms to a person — posting a chirp, following or unfollowing, liking or unliking, editing a profile, deleting an account — is present after a restart that occurs immediately after the acknowledgement, with no further action in between. _Enforces REQ-DOM-109._

**BAR-140** (`releasable` · `operational_check`) — After a restart and, separately, after a restore from backup, traversing chirps, replies, follows, and likes resolves every referenced account and chirp; no stopping, restarting, or recovering of the product introduces a reference to an entity that is absent. _Enforces REQ-DOM-110._

**BAR-141** (`usable_early` · `operational_check`) — After a restart, ids already issued are never reassigned to a different entity and id generation does not reset to a point that collides with existing ids; accounts and chirps created after the restart carry ids that collide with none recorded before it, and no existing entity's id has changed. _Enforces REQ-DOM-111._

**BAR-142** (`releasable` · `operational_check`) — Once applied, an account deletion's stated outcome survives a restart (the account still cannot sign in, the stated outcome still holds, the freed-handle policy still holds), and a restart during a deletion leaves the product with the deletion either fully applied or not applied — never a partially deleted account with dangling references. _Enforces REQ-DOM-112._

**BAR-143** (`usable_early` · `operational_check`) — Replacing the serving process entirely — rolling it to a fresh instance with no shared memory — without touching the data store leaves all accounts, chirps, follows, likes, and active sessions intact afterward, demonstrating that the authoritative state did not live in the replaced process. _Enforces REQ-DOM-113._

**BAR-144** (`usable_early` · `operational_check`) — The persistence seam the product reads and writes through (the `ChirpRepository` boundary in `chirp-server`, or its equivalent) is backed by a durable store rather than process memory on the released product, confirmed by the product's state surviving a restart. The in-memory backing at `Nousix-LLC/nswe-demo @ c805525` is the named release blocker this closes; the store engine, schema, and migrations are deferred to the code work and are not fixed here. _Enforces REQ-DOM-114._

**BAR-145** (`usable_early` · `operational_check`) — The product reports ready to serve only when its durable store is reachable: queried while the store is unreachable to the serving process it reports not-ready, and queried while the store is reachable and the product can serve it reports ready. _Enforces REQ-DOM-115._

**BAR-146** (`releasable` · `operational_check`) — A backup produced from a populated deployment can be restored, and on the restored store the accounts, chirps, follows, and likes are present and satisfy every model invariant — references resolve, ids are unchanged, counts equal the edges. _Enforces REQ-DOM-116._

**BAR-147** (`releasable` · `operational_check`) — For a signed-in account, the product can produce that account's profile, its complete set of chirps, and its complete set of followed accounts, matching the committed durable state at the time of the request. _Enforces REQ-DOM-117._

**BAR-148** (`releasable` · `operational_check`) — After an upgrade that changes the shape of the data (a redeploy with a data-model change), the accounts, chirps, follows, and likes that existed before the upgrade are all present afterward and their references still resolve. _Enforces REQ-DOM-118._

**BAR-160** (`usable_early` · `inspection`) — The product is exactly a client-side framework layer (`ferric` with `ferric-http` and `ferric-router`) and a product layer of three crates (`chirp-types`, `chirp-server`, `chirp-frontend`), with no fifth product crate silently carrying responsibility, and for every entity its wire shape, its lifecycle and persistence, and its presentation each resolve to exactly one part (no entity owned by two parts, none owned by the frontend). _Enforces REQ-DOM-200, REQ-DOM-208._

**BAR-161** (`usable_early` · `inspection`) — The `ferric` crates reference no chirp entity (account, chirp, follow, like, timeline, session) and do not depend on `chirp-types`; `chirp-frontend` depends on the `ferric` crates and never the reverse. _Enforces REQ-DOM-201._

**BAR-162** (`usable_early` · `inspection`) — Every identifier, domain entity and value object, request/response shape, pagination envelope, session token, and error contract that crosses between frontend and backend is defined once in `chirp-types`; the crate carries no I/O, framework, or async dependency, so both the native backend and the WebAssembly frontend compile against the identical definitions, and no wire shape is defined anywhere else. _Enforces REQ-DOM-202._

**BAR-163** (`usable_early` · `manual_user_check`) — Against the running product, actions that would violate a domain invariant (following oneself, liking a chirp twice, posting as a nonexistent account) are refused with a categorized error and no violating state is stored; `chirp-server` is the authority that enforces these invariants. _Enforces REQ-DOM-203._

**BAR-164** (`usable_early` · `automated_test`) — Submitting directly to the API, bypassing the frontend, a chirp over the length limit, a self-follow, and a chirp attributed to a nonexistent account is refused by the server regardless of any client-side checking. _Enforces REQ-DOM-207._

**BAR-165** (`usable_early` · `inspection`) — A single `ChirpRepository` boundary in `chirp-server` mediates every durable operation and enumerates exactly the durable operations the product uses; no handler or frontend code reaches around it to the store, and a durable store added as a second implementation requires no change to any handler. _Enforces REQ-DOM-204, REQ-DOM-223._

**BAR-166** (`usable_early` · `manual_user_check`) — From one address, the running service serves the REST API, loads the single-page app, and answers the health endpoint; no second process or second deployable is required to serve the app. _Enforces REQ-DOM-205._

**BAR-167** (`usable_early` · `inspection`) — The only state `chirp-frontend` owns across views is the current session (the authenticated account and its bearer token); every account, chirp, follow, like, and timeline it shows is fetched from the backend through the typed API client, and reloading the app re-fetches state so no entity data persists only in the client. _Enforces REQ-DOM-206._

**BAR-168** (`usable_early` · `inspection`) — The frontend and server build requests and decode responses from the same `chirp-types` definitions, with no second, hand-maintained copy of any wire shape on either side. _Enforces REQ-DOM-215._

**BAR-169** (`usable_early` · `manual_user_check`) — The REST surface lives under a reserved path prefix reached on the same origin that served the app, so no cross-origin configuration is part of normal operation; a path that is not an API path resolves to the single-page app rather than an API handler. _Enforces REQ-DOM-216._

**BAR-170** (`usable_early` · `manual_user_check`) — Create an account, fetch an account's public profile, open a session, post a chirp (including a reply), read a timeline, follow and unfollow an account, and like and unlike a chirp are each served end to end from the app and each has a defined request and response type in the contract. _Enforces REQ-DOM-217._

**BAR-171** (`usable_early` · `manual_user_check`) — A timeline read over the API returns a bounded page with an opaque continuation cursor when more chirps remain, passing the cursor returns the next older page, and a requested size above the server's maximum is clamped so a client cannot pull every chirp in one response. _Enforces REQ-DOM-218._

**BAR-172** (`usable_early` · `manual_user_check`) — A signed-in request proves itself by presenting the session as an opaque bearer token the server issued, and only an account's public profile fields and public chirps cross the contract outward — credential, verified contact, and session data never appear in any response. _Enforces REQ-DOM-219._

**BAR-173** (`usable_early` · `manual_user_check`) — A failed request returns a typed error carrying a machine-readable category (for example validation, not-found, conflict, unauthorized, forbidden) the frontend can branch on — distinguishing, for instance, a taken handle from a malformed one — rather than an ad-hoc string. _Enforces REQ-DOM-220._

**BAR-174** (`usable_early` · `inspection`) — JSON field names are stable regardless of internal renames, opaque identifiers and tokens serialize as their bare value, validated value objects reject invalid input on the way in, and error categories are machine-readable constants; the contract's public surface grows only by addition, and changing or removing an existing wire shape is out of bounds. _Enforces REQ-DOM-221._

**BAR-175** (`releasable` · `inspection`) — Each new surface the release adopts — among them ending a session, editing a profile, account settings, recovery, data export, account deletion, reporting and blocking/muting, notifications, an operator/administrative surface, and a way to find accounts to follow — has its request/response types defined in `chirp-types` and its operations mounted under the one namespaced API alongside the existing ones, not in a separate contract or service. _Enforces REQ-DOM-222._

**BAR-176** (`usable_early` · `manual_user_check`) — `chirp-server` serves the frontend as static assets from a configurable root, a request to an in-app route reached directly loads the app and resolves that route client-side, and a request for an unknown path returns the app's entry document rather than a server error. _Enforces REQ-DOM-224._

**BAR-177** (`usable_early` · `manual_user_check`) — A person reaches the product at one web address that serves the single-page app, with nothing to install and no second place to go, and the in-app route set reaches the release's surfaces (sign-up, sign-in, recovery, settings, deletion, and the rest) rather than adding a separate app. _Enforces REQ-DOM-225._

**BAR-178** (`releasable` · `operational_check`) — An operator can query an unauthenticated health endpoint with no session; with the durable store made unavailable it reports not-ready, and with the service able to serve users it reports ready, so an orchestrator can tell "serving" from "merely running." _Enforces REQ-DOM-226._

**BAR-179** (`usable_early` · `inspection`) — The service's network bind address and static-asset root are read from the environment with sensible defaults rather than hard-coded, and the product ships as one self-contained artifact serving both the API and the app that deploys to a fresh environment by setting configuration only, without editing source. _Enforces REQ-DOM-227._

---

### Experience (`BAR-200..299`) — enforces `design/experience-design.md`

_94 criteria — 51 `usable_early`, 43 `releasable`._

**BAR-200** (`usable_early` · `manual_user_check`) — With no session, every signed-out screen (at minimum the landing and a public profile opened by direct link) carries the chirp wordmark, a Sign in entry, a prominent Sign up entry, and footer access to the rules and to help; from each, a person can act (sign in, sign up, read the rules, reach help) without first being signed in and without reaching a dead end. _Enforces REQ-UX-001._

**BAR-201** (`usable_early` · `manual_user_check`) — Signed in, a persistent shell exposes Home, Explore, Notifications, Profile, Settings, and a Compose affordance, and all six are reachable from every signed-in screen — verified from the timeline, a profile, a conversation, and a settings sub-section — without using the browser back control. _Enforces REQ-UX-002._

**BAR-202** (`usable_early` · `manual_user_check`) — On every screen the product shows where the person is: navigating to Home, Explore, Notifications, Profile, and a settings sub-section in turn, the active primary-nav destination is visibly marked and the browser tab title (and the screen's main heading) names the current screen. _Enforces REQ-UX-003._

**BAR-203** (`releasable` · `inspection`) — Every screen the product presents, signed-out and signed-in, matches a published screen-inventory entry (route, exactly one archetype, declared auth state); no presented screen is absent from the inventory, and every signed-in-reachable inventory screen can in fact be reached. _Enforces REQ-UX-004._

**BAR-204** (`usable_early` · `manual_user_check`) — Signed in and starting from the home timeline, a person reaches each of compose, their own profile, notifications, settings, explore, help, and the rules in no more than two steps through the product, without holding an external link. _Enforces REQ-UX-005._

**BAR-205** (`usable_early` · `automated_test`) — A public profile and an individual chirp's conversation each resolve from a stable, shareable URL that, opened directly in a fresh browser context, lands on that exact profile / that exact chirp — not on a generic home or a redirect to sign-in for public content. _Enforces REQ-UX-006._

**BAR-206** (`releasable` · `manual_user_check`) — Opening a signed-in-only URL (for example a settings sub-section) with no session routes the person to sign-in, and after a successful sign-in lands them on that originally requested screen rather than on a default screen such as the home timeline. _Enforces REQ-UX-007._

**BAR-207** (`releasable` · `manual_user_check`) — An action taken with an expired or invalid session (for example opening the timeline or submitting a drafted chirp) routes the person to sign-in with the reason stated, does not lose their in-progress input, and returns them to where they were after they sign back in. _Enforces REQ-UX-008._

**BAR-208** (`releasable` · `manual_user_check`) — The browser back control returns to the previous screen without re-submitting the prior action and without losing the person's place in a scrolled timeline, and a dialog counts as one step so that back dismisses it by closing it; pressing back after posting a chirp does not re-post it. _Enforces REQ-UX-009._

**BAR-209** (`usable_early` · `manual_user_check`) — A list with more items than one page loads a bounded first page rather than every item at once, offers an explicit way to load older items, and — on the timeline — offers a way to pull in chirps posted since the person opened it. _Enforces REQ-UX-010._

**BAR-210** (`releasable` · `manual_user_check`) — Every irreversible or destructive action — delete account, delete a chirp, block or mute, end a session, sign out of all sessions — presents an explicit confirmation that names the consequence and offers a cancel before it proceeds, and cancelling changes nothing; the account-deletion confirmation additionally states what is and is not retained and points to data export. _Enforces REQ-UX-011._

**BAR-211** (`usable_early` · `manual_user_check`) — Every completed action (for example publishing a chirp, following an account, saving a setting) shows brief, non-blocking confirmation, and every failed action is shown at the point it was taken — against the field for a form, against the control for an in-place action — with enough to understand and retry; no action completes or fails silently. _Enforces REQ-UX-012._

**BAR-212** (`usable_early` · `manual_user_check`) — A signed-in person can start composing a chirp from any screen through a persistent compose affordance without navigating away first (verified from the timeline, a profile, and notifications), and opening the dedicated compose URL directly opens the compose surface. _Enforces REQ-UX-013._

**BAR-213** (`usable_early` · `manual_user_check`) — A completed sign-up establishes the session and lands the person on onboarding — not on a repeated sign-in prompt and not on a blank timeline; and signing out ends the session, returns the person to the signed-out landing, and leaves the browser back control unable to re-expose a protected screen's content. _Enforces REQ-UX-014, REQ-UX-016._

**BAR-214** (`releasable` · `manual_user_check`) — Account-level choices live under one settings area with labelled sub-sections (edit profile, account & contact, notification preferences, sessions & devices, blocked & muted, export, delete), each opening directly by its own URL and each returning to the settings hub; and the rules/terms and a user-facing help & support entry are reachable from both the signed-out shell and the signed-in shell. _Enforces REQ-UX-017, REQ-UX-018._

**BAR-215** (`releasable` · `manual_user_check`) — On three screens of different archetypes on the live product (for example the home timeline, a profile, and a settings sub-section), body text is the same size and leading, the gutters and spacing rhythm match, buttons share one shape and one primary colour, and no screen introduces a colour or control style absent from the others. _Enforces REQ-UX-020._

**BAR-216** (`usable_early` · `manual_user_check`) — On the live product, chirp and reading text renders at no less than 15px with visibly generous line spacing; on any screen the main heading is visibly larger and heavier than its body text, and meta text (timestamps, handles) is visibly quieter than body text. _Enforces REQ-UX-021._

**BAR-217** (`usable_early` · `manual_user_check`) — On the live product, sample body text, secondary text, link text, primary-button text, and a field border and measure each with a contrast tool: normal text reaches ≥ 4.5:1, large text ≥ 3:1, and control and meaningful-icon boundaries ≥ 3:1. The measurement passes in each theme the product ships. _Enforces REQ-UX-022._

**BAR-218** (`usable_early` · `manual_user_check`) — Using only the keyboard on the live product, tabbing through a screen's links, buttons, fields, and menu items shows a distinct focus indicator on each that stands out from its surroundings, and no focusable control takes focus without a visible indicator. _Enforces REQ-UX-025._

**BAR-219** (`releasable` · `manual_user_check`) — On the live product, the reading column (timeline, conversation, profile chirps, a form) is held to a bounded measure on a wide window rather than spanning the whole width; on a phone viewport the same content fills the width with a consistent edge gutter; and the spacing between items is visibly uniform within and across these screens. _Enforces REQ-UX-026._

**BAR-220** (`releasable` · `manual_user_check`) — On the live product, opening a menu places it above content; opening a dialog places it above content on a dimmed scrim with the menu not showing through; triggering a toast places it above the dialog. At no point does a lower layer render on top of a higher one. _Enforces REQ-UX-027._

**BAR-221** (`releasable` · `manual_user_check`) — On the live product, icons come from one consistent set; the current primary-nav destination reads as active by more than colour, confirmed still identifiable in greyscale or a colour-blind simulation; and every icon-only control (compose, like, menu) exposes an accessible name to a screen reader and a label on hover. _Enforces REQ-UX-028._

**BAR-222** (`releasable` · `manual_user_check`) — On the live product, opening and closing a menu and a dialog and triggering a toast each complete a brief transition (well under half a second) that settles, with nothing animating or blinking continuously apart from a bounded loading indicator; with the system reduce-motion setting on, the same actions remove or reduce animation to a short fade and no information is lost. _Enforces REQ-UX-029._

**BAR-223** (`usable_early` · `manual_user_check`) — On the live product, action buttons across several screens read as sentence-case verbs (for example "Post", "Follow", "Save changes") rather than "OK"/"Submit" or Title Case; and a forced error (an invalid form entry, then a failed submit) shows a plain-language message with a way forward and no raw error code, stack text, or internal identifier. _Enforces REQ-UX-032._

**BAR-224** (`releasable` · `manual_user_check`) — On three screens of different archetypes on the live product, a chirp, buttons, text fields, and error messages each render from one shared treatment with no bespoke control standing in for a library component; and inspecting a chirp card, a primary button, a text field, and a dialog in each shipped theme shows colours matching the token roles (a primary action is the one primary colour, a destructive one the danger colour), text sizes matching the type scale, and no colour or size that is absent from the token set. _Enforces REQ-UX-033, REQ-UX-034._

**BAR-225** (`usable_early` · `manual_user_check`) — On the live product, typing past the character limit in compose turns the counter to the danger treatment and disables Post; an empty draft leaves Post disabled and posts nothing; a valid chirp posts and appears as a chirp card showing avatar, author, handle, relative time, text, and the like / reply / more-actions row; and liking a chirp shows a filled icon and a "Liked" label together, still identifiable as liked in greyscale. _Enforces REQ-UX-035._

**BAR-226** (`usable_early` · `manual_user_check`) — On the live product, a brand-new account's empty timeline shows an explanatory block with a next step rather than a blank area; a list while loading shows skeleton rows at the real item size rather than a spinner on a blank screen; a forced page-load failure shows an inline message with a retry in place of the missing items while keeping any items already shown; and a failed action shows an inline error in the product's danger styling at the point of action. None of these renders a blank area, a frozen screen, or a raw failure. _Enforces REQ-UX-031, REQ-UX-036._

**BAR-227** (`usable_early` · `manual_user_check`) — On the live product sign-up, submitting a handle already in use shows a plain-language error against the handle field naming the handle (not in a distant summary and not as a code); submitting a mis-formatted handle shows a field-level validation error that explains the rule; and while a handle's availability is being checked the field shows a pending state rather than flicking prematurely to valid or invalid. _Enforces REQ-UX-037._

**BAR-228** (`usable_early` · `manual_user_check`) — On the live product, initiating deletion of a chirp, blocking an account, and deleting the account each present a confirmation that names what will happen and offers a plain cancel, and nothing changes on cancel; the account-deletion confirmation additionally states what is and is not retained and points to data export before proceeding. _Enforces REQ-UX-038._

**BAR-229** (`usable_early` · `manual_user_check`) — On the live product, publishing a chirp, following an account, and saving a setting each show a brief, non-blocking confirmation; an action in flight shows its control in a busy state that cannot be re-triggered; and a forced in-place failure (acting while offline, or a transient server error while posting) shows the failure at the control with a way to retry and does not lose the drafted text. _Enforces REQ-UX-039._

**BAR-230** (`usable_early` · `manual_user_check`) — With no session, opening the site root reaches a Landing that says what chirp is and shows both a Sign up and a Sign in action, each reachable without first signing in. Where a public sample is served it shows real public chirps; where none is served the Landing still explains chirp and offers both actions rather than a blank area. _Enforces REQ-UX-040, REQ-PROD-030, REQ-PROD-031._

**BAR-231** (`usable_early` · `manual_user_check`) — The Sign up screen collects a handle, a display name, an optional bio marked optional, a credential, and a point of contact — and no fields beyond these — and on submit with valid values creates the account and signs the person in. The contact is shown as pending verification and is not treated as confirmed until it is verified. _Enforces REQ-UX-041, REQ-UX-043, REQ-PROD-008, REQ-PROD-012, REQ-PROD-031._

**BAR-232** (`usable_early` · `automated_test`) — On the Sign up screen a handle that is already taken, or that breaks the rule (more than 15 characters, or anything other than letters, numbers, or underscores), is reported inline against the handle field so the person can fix it before submitting, and a valid unused handle resolves to accepted. The check does not disclose which other handles exist beyond the conflict it reports. _Enforces REQ-UX-042, REQ-PROD-009._

**BAR-233** (`usable_early` · `automated_test`) — Signing in requires the credential set at sign-up or recovery — a known handle alone never grants a session. A handle that matches no account returns a not-found result with a path to Sign up; a real handle with a wrong credential returns a failed sign-in the person can retry or recover from, with a path to Recover access. Neither result reveals account existence beyond what a public profile already shows, and neither shows a system code. _Enforces REQ-UX-044, REQ-UX-045, REQ-PROD-010, REQ-PROD-011._

**BAR-234** (`releasable` · `automated_test`) — A person who cannot sign in can start recovery themselves from a Recover access screen that runs through the account's verified contact. Submitting a contact returns the same neutral acknowledgement whether or not it matches an account, so the response discloses no account existence. Following a valid recovery step opens a Set new credential screen, and after setting a new credential the person is routed to Sign in and can sign in with it. An expired or already-used recovery link is explained with a path back to Recover access rather than a dead end. _Enforces REQ-UX-046, REQ-UX-047, REQ-PROD-013, REQ-PROD-012._

**BAR-235** (`usable_early` · `automated_test`) — Signing out is confirmed before it proceeds; the confirmation names the consequence and offers both a cancel and an option to end sessions on all other devices. Confirming ends the current session on the server — the prior session can no longer be used — and lands the person on the signed-out Landing. Cancelling changes nothing. A sign-out that fails is reported and leaves the person signed in rather than appearing to succeed. _Enforces REQ-UX-048, REQ-PROD-014._

**BAR-236** (`releasable` · `manual_user_check`) — Each of the Landing, Sign up, Sign in, Recover access, Set new credential, and Sign out surfaces presents a designed state for empty, loading, and error — a field-level message, a busy control, a skeleton, or an explained error with a next step — rather than a blank area, a hang, or a silent failure. Where a state does not apply to a surface, that is stated rather than left undesigned. _Enforces REQ-UX-049, REQ-PROD-033, REQ-PROD-034._

**BAR-237** (`releasable` · `automated_test`) — A completed sign-up lands the person on the onboarding screen, not a sign-in prompt or a blank page. A person routed from a signed-in-only screen to Sign in lands back on the screen they asked for after signing in. After signing out, pressing back does not show the protected screen's content. (The three-way links among Sign in, Sign up, and Recover access ride the non-gating REQ-UX-015 and are not part of this release condition; a first release may omit them provided Recover access stays reachable from Help & support.) _Enforces REQ-UX-050, REQ-PROD-032, REQ-PROD-014._

**BAR-238** (`usable_early` · `manual_user_check`) — A signed-in person who opens the home destination sees a timeline carrying their own chirps and the chirps of the accounts they follow, ordered newest first, and the signed-in shell marks Home as the current destination. The timeline arrives as a bounded first page with a visible control to load older chirps; using that control brings in a further bounded page, and at no point does the product load the entire timeline in one request. Following at least one account that has posted, the person sees those chirps interleaved with their own in newest-first order. _Enforces REQ-UX-051, REQ-PROD-002._

**BAR-239** (`releasable` · `manual_user_check`) — When chirps have been posted since the person last viewed the timeline, a catch-up control at the top of the list states how many new chirps are waiting and, when used, brings them in as a single bounded step that returns the reader to the newest chirps — not an open-ended scroll back through everything missed. Opening a chirp's conversation and returning to the timeline leaves the reader at the position they left, not reset to the top of the list. _Enforces REQ-UX-052, REQ-PROD-002._

**BAR-240** (`releasable` · `manual_user_check`) — The home timeline shows a designed state for empty, for failure, and for loading in place of a blank region or an indefinite spinner. An account that follows no one and has posted nothing sees an empty state that explains why the timeline is empty and offers a route to Explore and to composing a first chirp. A failed first page shows a plain-language message with a retry in place of the list, visibly distinct from the empty state; a failed load-older or failed catch-up is shown at the boundary while the chirps already on screen stay loaded. A first load shows skeleton rows at the size of real chirp cards while the signed-in shell stays visible, and no region of the timeline continues to spin without resolving. _Enforces REQ-UX-053, REQ-PROD-002._

**BAR-241** (`usable_early` · `automated_test`) — The compose surface shows a remaining-character count that warns as the 280-character limit nears and marks the draft as over the limit once it is exceeded. The post action is not activatable while the draft is empty or over the limit, and an empty or over-limit submission reached by any path creates no chirp. While a publish is in flight the post action shows a busy state and the same draft cannot be submitted a second time; on success the chirp is published, the field clears, and a brief non-blocking confirmation is shown; on failure the draft text is kept exactly as written with an inline error and a retry, so the person can post again without retyping. No successful post is duplicated and no failed post loses its draft. _Enforces REQ-UX-054, REQ-UX-055, REQ-PROD-001._

**BAR-242** (`releasable` · `manual_user_check`) — The compose surface is reachable inline at the top of the home timeline, as the Compose dialog (opened from the signed-in shell's compose affordance and directly at `/compose`), and in the reply position of a chirp's conversation screen, and it behaves the same way in each. In every placement an untouched field shows a placeholder prompt, a remaining-character count at the full 280 budget, and a disabled primary action; the action is labelled for its placement — "Post" when writing a chirp, "Reply" when answering one. Dismissing the Compose dialog with text still in the field and reopening it does not lose the draft. _Enforces REQ-UX-056, REQ-PROD-001, REQ-PROD-003._

**BAR-243** (`usable_early` · `manual_user_check`) — Signed in on a chirp's conversation, a person can write a reply within the character limit and publish it from the conversation's reply control; the published reply then appears in that conversation shown as answering the chirp, and its publication is confirmed. Attempting to reply to a chirp that no longer exists reports not-found at the reply control and keeps the typed draft, rather than appearing to send. Opened without a session, the conversation presents a visible way to sign in or sign up in place of the reply control, not a dead or silent one. _Enforces REQ-UX-058, REQ-PROD-003._

**BAR-244** (`usable_early` · `manual_user_check`) — A chirp's like control toggles between not-liked and liked, with the liked state distinguished by more than colour (a filled heart and a "Liked" label). Liking an already-liked chirp, or reloading the screen, does not raise the count a second time; unliking returns the count to its earlier value. When a like or unlike cannot be completed, the control returns to its prior state, the count is left unchanged (it never shows a like that did not take), and the failure is reported at the control with a way to retry. The control looks and behaves the same wherever the chirp is shown — in the timeline, in its conversation, and on a profile. _Enforces REQ-UX-059, REQ-PROD-005._

**BAR-245** (`usable_early` · `manual_user_check`) — On another person's profile header or in a person row, the follow control toggles between "Follow" and "Following"; following the same account again, or reloading, leaves exactly one follow in place with the control reading "Following", and unfollowing returns it to "Follow". A person's own identity never presents a Follow control — the own-identity action is "Edit profile" — so a self-follow cannot be started, and any attempt is refused and shown at the control. When a follow or unfollow cannot be completed, the control returns to its prior label, any follower count is unchanged, and the failure is reported at the control with a way to retry. The control looks and behaves the same wherever it appears (profile header and person rows). _Enforces REQ-UX-060, REQ-PROD-006._

**BAR-246** (`usable_early` · `manual_user_check`) — A person's profile at its handle URL shows their avatar, display name, handle, bio, and follower and following counts, above that person's own chirps in newest-first bounded pages with a way to load older ones, and loads both without a session and with one; signed in, the header also carries the follow control. When the signed-in owner opens their own profile, they see the same public face a visitor sees, with an "Edit profile" entry in place of the follow control that opens the edit-profile screen, and no follow control is shown on their own profile. _Enforces REQ-UX-061, REQ-UX-062, REQ-PROD-007._

**BAR-247** (`releasable` · `manual_user_check`) — A profile's follower and following counts open the followers list and the following list, each at its own URL. Each list shows one person per row with enough to recognise the account (avatar, display name, handle, bio) in a bounded page with a way to load more, a labelled way back to the profile, and — for a signed-in viewer — a follow control on each row. Each list's URL, opened directly, lands on that list. _Enforces REQ-UX-063, REQ-PROD-007, REQ-PROD-006._

**BAR-248** (`usable_early` · `manual_user_check`) — After a fresh sign-up, a brand-new account lands on the onboarding screen and sees, in plain language, why its timeline is empty and at least one working control that opens compose to write a first chirp (and, wherever discovery is present, a control to find people to follow). The explanation and the compose route appear immediately and stay usable while any suggestions preview is still loading — shown as placeholders, not a blank region — and after that preview fails or comes back empty. The screen is never a blank page and never leaves the person without a next step. _Enforces REQ-UX-065, REQ-UX-066, REQ-PROD-032, REQ-PROD-033._

**BAR-249** (`releasable` · `manual_user_check`) — A signed-in person can reach edit profile from the settings hub and from the edit action on their own profile, change their display name and bio, and save; the saved values then show on their profile. An empty required display name or a rejected value is reported against the field in plain words and no save is made; a save that fails is reported at the form with the person's edited values still present so they can retry; a save that succeeds is confirmed. The screen never reports success or failure silently and never shows a raw system or error code. _Enforces REQ-UX-067, REQ-UX-068, REQ-PROD-016, REQ-PROD-033, REQ-PROD-034._

**BAR-250** (`releasable` · `manual_user_check`) — A signed-in person opening settings sees a labelled row for every account section in the release — edit profile, account & contact, sessions & devices, blocked & muted, export your data, and delete account (the notification-preferences row appears only where notifications are in the release). Each row opens its section both by selection and by entering the section's own URL directly in a fresh tab, and each section offers a labelled way back to the hub that works alongside the browser back control. The delete-account row is shown as destructive and opens a confirmation rather than deleting when selected. If the account summary fails to load, every section row is still reachable. _Enforces REQ-UX-069, REQ-PROD-017._

**BAR-251** (`releasable` · `manual_user_check`) — On the account & contact screen a signed-in person sees their current point of contact with a status of either "Verified" or "Not verified yet". They can change the contact and save; the change is confirmed, the new contact shows as not-yet-verified, and the screen offers a step that confirms the person controls it before it is used for account recovery. An unverified contact is presented as a caution, not an error, and is not treated as verified until that step is completed. A failed save is shown against the contact field with the typed value preserved for retry. _Enforces REQ-UX-070, REQ-PROD-012, REQ-PROD-017._

**BAR-252** (`releasable` · `operational_check`) — A signed-in person opening sessions & devices sees every place the account is currently signed in, each with enough to recognise it — device and browser, approximate location, last-active time — and the current session marked. Ending another session goes through a confirmation that names the effect; after confirming, that session leaves the list and the next action attempted from the ended session on its own device is refused and requires a fresh sign-in. _Enforces REQ-UX-073, REQ-PROD-017, REQ-PROD-014._

**BAR-253** (`usable_early` · `operational_check`) — A signed-in person can sign out from sessions & devices through a confirmation that, where the account is signed in elsewhere, offers to end those other sessions too. After sign-out the person is on the signed-out landing; the browser back control does not re-expose any protected screen; and replaying the signed-out session's request is refused, so the next person at that device is not signed in as them. _Enforces REQ-UX-074, REQ-PROD-014._

**BAR-254** (`releasable` · `manual_user_check`) — From the export screen a signed-in person sees what the export contains and a single request action. After requesting, the screen shows a preparing state rather than freezing or blanking, and when ready offers a way to download the file; the downloaded file opens outside chirp and contains the person's chirps, their profile, and who they follow. A preparation that fails is explained in plain words with a retry, and no broken file is offered. _Enforces REQ-UX-075, REQ-PROD-019._

**BAR-255** (`releasable` · `operational_check`) — The delete-account confirmation, shown before anything is deleted, states plainly that the account and its chirps are removed and cannot be recovered, shows what is and is not retained and what happens to the handle (matching the product's stated deletion outcome), and points the person to export their data first. The delete action stays unavailable until the person gives the explicit confirmation it asks for. Cancelling at any point returns to settings with the account and its content unchanged and still able to sign in. Giving the confirmation and deleting signs the person out to the signed-out landing, after which the account can no longer sign in and the browser back control does not re-expose the deleted account's screens. _Enforces REQ-UX-076, REQ-UX-077, REQ-PROD-020, REQ-PROD-021, REQ-PROD-022._

**BAR-256** (`releasable` · `manual_user_check`) — Signed in, a person can open the report action from both a chirp's and an account's more-actions menu, choose a reason, add optional detail, and submit, and sees an in-product confirmation that the report was received; and when no reason is chosen the submit control does not send the report. No report path ends in a silent no-op. _Enforces REQ-UX-079, REQ-PROD-025, REQ-PROD-026._

**BAR-257** (`releasable` · `manual_user_check`) — From both the signed-out and the signed-in shell, a person can reach a Rules & terms screen that lists the product's rules as named categories; and the reasons offered in the report action correspond to those same named categories, so a submitted report points at a stated rule rather than an unlabelled choice. _Enforces REQ-UX-083, REQ-PROD-026._

**BAR-258** (`releasable` · `manual_user_check`) — A Help & support entry is reachable with no session from the signed-out shell and, signed in, from the account menu; from it a person can reach the recover-access path ("Can't get in?") and a way to contact the operator ("Contact support") for what self-service does not cover. _Enforces REQ-UX-082, REQ-PROD-028, REQ-PROD-013._

**BAR-259** (`usable_early` · `manual_user_check`) — Opening an address that does not resolve — a mistyped path, a deleted chirp, an unavailable account — shows a not-found screen that states the resource is missing (consistent with the in-place tombstone used elsewhere), reveals no private detail about a removed or restricted resource, and offers a route to a known place (the home timeline when signed in, the landing when signed out) rather than a broken page. _Enforces REQ-UX-084, REQ-PROD-033._

**BAR-260** (`usable_early` · `manual_user_check`) — On each surface in this area a person can force an empty result, a failed load, and a first load and sees, respectively, an explained empty treatment carrying at least one next step (never a blank region), a distinct "couldn't load — try again" treatment with a retry, and a loading treatment — with the empty treatment and the failed treatment visibly different from each other; a surface to which one of these states does not apply says so rather than showing nothing. Checked across, for example, Notifications, the report and block/mute dialogs on a failed submit (the dialog stays open with its input preserved and an inline error), the blocked & muted screen when empty, a brand-new home timeline, and an empty followers list. _Enforces REQ-UX-085, REQ-UX-086, REQ-PROD-033, REQ-PROD-034._

**BAR-261** (`usable_early` · `manual_user_check`) — On a slow connection a first load of a content surface shows skeletons sized to the real content — not a spinner on a blank page — while the persistent navigation frame stays visible; an incremental load, such as loading older items, shows a bounded indicator at the point of the action while the already-loaded content stays on screen; and a load that cannot resolve stops and becomes an error with a retry rather than spinning without end. _Enforces REQ-UX-087, REQ-PROD-033._

**BAR-262** (`usable_early` · `manual_user_check`) — A person who triggers a field error (for example a taken handle at sign-up), a failed in-place action (for example a like or follow that fails), and a failed list page sees in each case a plain-language message at the point of the problem — no system or error code, never a silent no-op and never a blank — with a way to retry or correct and their input preserved; an optimistic in-place action that fails reverts to its prior state rather than showing a result that did not happen. _Enforces REQ-UX-088, REQ-PROD-033._

**BAR-263** (`releasable` · `manual_user_check`) — With a session expired, a person who drafts a chirp and tries to post is routed to sign-in with the reason stated — not a silent failure — and, after signing in, is returned to where they were with the drafted text intact; and a transient server error during the same post can be retried in place and succeeds without re-typing the draft. This observes, and does not redefine, the information architecture's session-expiry (REQ-UX-008) and auth-gated-return (REQ-UX-007) conventions. _Enforces REQ-UX-089, REQ-PROD-034._

**BAR-265** (`usable_early` · `manual_user_check`) — At a 360×640 CSS-px portrait viewport, signed in, a person completes the whole core loop end to end — open and scroll the home timeline, open Compose and post a chirp, reply to a chirp, like then unlike a chirp, follow then unfollow an account, and open a public profile — and at no step is there horizontal page scrolling, a control hidden off-screen, or an action target that cannot be tapped. _Enforces REQ-UX-090._

**BAR-266** (`usable_early` · `automated_test`) — The content region is a single reading column at every width. At a 360px-wide viewport it fills the width inside the 16px gutter with no horizontal scrollbar; at a 1280px-wide viewport it is held to the 600px measure and horizontally centred, not stretched across the window. Neither width requires pinch-zoom or sideways scrolling to read a chirp. _Enforces REQ-UX-091._

**BAR-267** (`usable_early` · `manual_user_check`) — Signed in, starting from the timeline, a person reaches Home, Explore, Notifications, Profile, Settings, and a Compose affordance, and sees the current destination marked, at both a compact (< 640px) and a wide (≥ 1024px) viewport: at compact width the primary destinations sit in a bottom tab bar and Compose is reachable without opening a menu; at wide width the same destinations appear in a persistent side navigation. _Enforces REQ-UX-092._

**BAR-268** (`usable_early` · `manual_user_check`) — With no session, the Landing and a public profile link each show a Sign up and a Sign in entry without opening a menu, and keep Rules & terms and Help & support reachable (at minimum by scrolling to the footer), at both a compact (< 640px) and a wide (≥ 1024px) viewport. No viewport presents an authentication wall or hides the path to join. _Enforces REQ-UX-093._

**BAR-269** (`releasable` · `manual_user_check`) — On a compact (< 640px) viewport, a feed/list screen longer than one page (home timeline, a chirp's conversation, notifications, search results, followers, following) keeps its load-older control and its "N new chirps" catch-up reachable without horizontal scrolling and keeps each chirp card's reply, like, and more controls tappable; opening an item and pressing back restores the list at its prior scroll position. _Enforces REQ-UX-094._

**BAR-270** (`usable_early` · `manual_user_check`) — On a compact (< 640px) viewport with the on-screen keyboard open, a form screen (sign up, sign in, recover access, set new credential, edit profile) shows the focused field, its label, and any inline error together; keeps its primary submit reachable by scrolling the form without first dismissing the keyboard; and never requires horizontal scrolling or a side-by-side field layout. An invalid field's error shows with the field, not in a summary elsewhere. _Enforces REQ-UX-095._

**BAR-271** (`releasable` · `manual_user_check`) — On a compact (< 640px) viewport a public profile shows the header's avatar, display name, handle, bio, counts, and its context action (Follow, or Edit profile on one's own profile) without horizontal scrolling, with the chirps list in one column below; the Settings hub and each sub-section are full screens, and a labelled back returns from a sub-section to the hub. _Enforces REQ-UX-096._

**BAR-272** (`releasable` · `manual_user_check`) — On a compact (< 640px) viewport, Compose and Report present as a full-screen or bottom-sheet surface whose input and primary action stay visible and operable with the keyboard open, and every destructive confirmation (delete account, delete chirp, block or mute, end a session, sign out) shows both its confirm and its cancel fully visible and tappable without clipping at the screen edge; the browser back control dismisses the overlay without leaving the underlying screen. _Enforces REQ-UX-097._

**BAR-273** (`usable_early` · `automated_test`) — On a touch viewport every interactive control renders a tap target of at least 44×44 CSS px with at least 8 CSS px between adjacent targets, and the core-loop primary actions (the Compose affordance, a chirp card's like and reply controls, and the primary navigation) render within the lower portion of the screen rather than only at the top. _Enforces REQ-UX-098._

**BAR-274** (`usable_early` · `manual_user_check`) — Walking the signed-in core loop at a supported phone viewport in portrait and then in landscape, the page never scrolls horizontally and never needs pinch-zoom, and focusing a sign-in or compose text field does not trigger the browser's automatic zoom; content reflows to the narrower dimension rather than overflowing it. _Enforces REQ-UX-099._

**BAR-275** (`releasable` · `inspection`) — The product states its accessibility standard as WCAG 2.1 Level AA by name (not "accessible" in the abstract) and names its gated conformance set — the core loop and the get-in screens (sign-up, sign-in, recovery, onboarding) with their empty, loading, and error states. A reviewer can find the stated standard and the enumerated gated set in the product's accessibility statement. _Enforces REQ-UX-100, REQ-PROD-036._

**BAR-276** (`usable_early` · `manual_user_check`) — With the pointer unused, a keyboard-only user completes every core-loop and get-in action: sign in, read the timeline, operate a chirp's reply/like/more controls, compose and post a chirp, follow an account, open a profile, and operate a settings sub-section. No action is reachable by pointer only. _Enforces REQ-UX-101._

**BAR-277** (`usable_early` · `manual_user_check`) — Tabbing through the timeline, a form, and a settings screen, focus moves in each screen's reading order and the focused control always shows a visible focus indicator. Opening a dialog moves focus into it; closing it returns focus to the opening control; changing route lands focus on the new screen rather than on a removed control. _Enforces REQ-UX-102, REQ-UX-025._

**BAR-278** (`usable_early` · `manual_user_check`) — No screen leaves keyboard focus stuck with no way out. A modal dialog (compose, a destructive confirmation) holds focus while open, cycles focus within itself on Tab, closes on Esc and on its close/Cancel control, and returns focus to the control that opened it. _Enforces REQ-UX-103._

**BAR-279** (`releasable` · `manual_user_check`) — On a signed-in screen, one Tab from the top reveals a visible "Skip to content" control that moves focus past the navigation shell to the main content, and a screen reader can jump to the main landmark directly. _Enforces REQ-UX-104._

**BAR-280** (`usable_early` · `manual_user_check`) — With a screen reader, each interactive component — chirp card, compose box, text field, toggle, button, icon button, dialog — is announced with a sensible accessible name, its correct role (button, switch, text box, dialog), and its current state (a like as pressed or not, a toggle as on or off, a busy or disabled control as such). _Enforces REQ-UX-106._

**BAR-281** (`releasable` · `manual_user_check`) — With a screen reader, the landing, the timeline, and a settings sub-section each expose a navigation landmark, exactly one main landmark, and (on the signed-out shell) a footer landmark, and the user can jump between them. _Enforces REQ-UX-107._

**BAR-282** (`releasable` · `manual_user_check`) — With a screen reader, the heading list for the timeline, a profile, and a settings sub-section each shows exactly one top-level heading that names the screen and matches the browser tab title, with sub-headings descending in order and no skipped levels. _Enforces REQ-UX-108._

**BAR-283** (`releasable` · `manual_user_check`) — With a screen reader on the timeline, the feed is navigable item by item — each chirp is a distinguishable list item the reader can tell from the next — and the "load older" and "N new chirps" catch-up affordances are reported as named buttons. The conversation, notifications, search-results, and followers/following lists expose the same item structure. _Enforces REQ-UX-109._

**BAR-284** (`usable_early` · `manual_user_check`) — With a screen reader, each icon-only control (the compose affordance, a like control, the current nav destination) reports a text name; the like reports pressed or not-pressed and the current nav destination reports its selected state as a programmatic state, not by colour or fill alone. _Enforces REQ-UX-110, REQ-UX-028._

**BAR-285** (`usable_early` · `automated_test`) — An automated contrast scan run against the live product in each shipped theme finds normal text at or above 4.5:1, large text at or above 3:1 (WCAG 1.4.3), and meaningful control and icon boundaries at or above 3:1 (WCAG 1.4.11), with no reported failure. _Enforces REQ-UX-111, REQ-UX-022._

**BAR-286** (`releasable` · `manual_user_check`) — At 200% browser zoom the timeline and a form show no clipped content and keep every control reachable (WCAG 1.4.4); at a 320-CSS-pixel viewport the reading content reflows to one column with no horizontal scrolling (WCAG 1.4.10); and an applied text-spacing override clips or overlaps nothing (WCAG 1.4.12). _Enforces REQ-UX-112._

**BAR-287** (`usable_early` · `manual_user_check`) — Viewed in greyscale or a colour-blindness simulation, a field error, an active like, and the current nav destination are each still identifiable without their colour — carried by text, an icon, or a label as well (WCAG 1.4.1). _Enforces REQ-UX-113, REQ-UX-022._

**BAR-288** (`releasable` · `manual_user_check`) — Through normal use of the product, no content flashes more than three times in any one second (WCAG 2.3.1). _Enforces REQ-UX-113._

**BAR-289** (`releasable` · `manual_user_check`) — With the system reduce-motion setting on, opening and closing a menu and a dialog, triggering a toast, and loading a list remove or shorten motion to at most a brief fade; nothing animates continuously except a loading indicator that is also announced; and no information is conveyed only by movement. _Enforces REQ-UX-114, REQ-UX-029._

**BAR-290** (`releasable` · `manual_user_check`) — With a screen reader, posting a chirp, following an account, and saving a setting are each announced as a status message through a polite live region without moving focus off the user's current control; with the timeline open, the arrival of new chirps behind the catch-up control is announced politely (WCAG 4.1.3). _Enforces REQ-UX-115._

**BAR-291** (`usable_early` · `manual_user_check`) — With a screen reader on sign-up, submitting a taken handle announces the error, re-reads it when the handle field is focused, reports the field invalid, and names the problem and a fix in plain words with no error code (WCAG 3.3.1, 3.3.2, 3.3.3); a forced submit failure is announced, not silent. _Enforces REQ-UX-116, REQ-UX-032._

**BAR-292** (`usable_early` · `manual_user_check`) — With a screen reader, opening a list announces "loading" while its skeletons are hidden from the reader; a forced load failure announces an error with a reachable retry; a brand-new empty timeline presents a heading and a next-step action rather than silence; and acting with an expired session announces the reason and moves focus to the sign-in screen. _Enforces REQ-UX-117, REQ-UX-008._

**BAR-293** (`releasable` · `inspection`) — For a release, three recorded checks over the gated conformance set are on file — an automated scan in each shipped theme, a keyboard-only walk, and a screen-reader walk of the core loop — together with the support matrix they ran on (at least one screen reader per major platform with its common browser), and every gated screen appears in each check. _Enforces REQ-UX-118._

**BAR-294** (`releasable` · `inspection`) — The release record shows the gated conformance set has no open Level A or Level AA failure, every accessibility failure found on the gated set was fixed rather than waived, and any failure recorded against a non-gating surface is tracked with that surface rather than blocking the release. _Enforces REQ-UX-119._

---

### Identity, security & trust (`BAR-300..399`) — enforces `design/identity-security.md`

_40 criteria — 18 `usable_early`, 22 `releasable`._

**BAR-300** (`usable_early` · `manual_user_check`) — Signing in to any account requires presenting the secret established at sign-up; knowing only a handle (public information) grants no session and no authenticated action on any product surface. There is no debug route, handle-only fallback, or default/shared secret that issues a session without verifying the presented secret against the account. _Enforces REQ-ID-002, REQ-ID-030, REQ-ID-031, REQ-ID-032._

**BAR-301** (`usable_early` · `automated_test`) — A trivially weak secret (a short or well-known value) is rejected with guidance and a secret meeting the stated policy is accepted; repeated failed sign-in attempts against one account or from one source are throttled, delayed, or capped rather than answered at full speed indefinitely, and the throttled response does not distinguish "almost right" from "wrong." _Enforces REQ-ID-034, REQ-ID-035._

**BAR-302** (`usable_early` · `inspection`) — No product surface — API response, data export, settings screen, or operator tool — returns any account's secret or holds it in a form from which the original value can be read back; the only supported way back in after losing a secret is to set a new one through recovery. _Enforces REQ-ID-036._

**BAR-303** (`usable_early` · `manual_user_check`) — Sign-up accepts a handle only if it is well-formed (up to 15 letters, digits, or underscores) and not already taken; a taken or ill-formed handle is reported as a field-level error on the handle so the person can correct it, and a valid unused handle is accepted. _Enforces REQ-ID-003._

**BAR-304** (`usable_early` · `manual_user_check`) — A sign-in with an unknown handle lets the person conclude the account does not exist (so they can proceed to sign-up); a sign-in with a known handle and a wrong secret is a retryable, recoverable failure; and neither path exposes a private attribute of any account (such as a recovery contact) beyond what the product already makes public. _Enforces REQ-ID-033._

**BAR-305** (`usable_early` · `automated_test`) — A successful sign-in issues a session credential separate from the account secret, so routine authenticated requests never carry the secret; whether a session is valid and until when is decided server-side, and tampering with the presented session credential — altering its contents, substituting another account's identifier, or extending an encoded expiry — yields no valid or longer-lived session for any account. _Enforces REQ-ID-037, REQ-ID-040._

**BAR-306** (`usable_early` · `operational_check`) — Every session has a bounded, defined maximum lifetime after which it is rejected and the person is returned to sign-in; an expired, signed-out, or unrecognized session credential is each rejected the same way (treated as unauthenticated, with no partial or half-authenticated state); and a session accepted before a server restart behaves consistently with the expiry and invalidation rules after one. _Enforces REQ-ID-038, REQ-ID-044._

**BAR-307** (`usable_early` · `operational_check`) — A person can sign out, and sign-out invalidates the session server-side so the prior session credential can no longer act as the account — verified by replaying the prior request, which is rejected — and the next person at that device is not signed in as the previous one. _Enforces REQ-ID-039._

**BAR-308** (`releasable` · `operational_check`) — A person's sessions are distinguishable from one another, so that ending one device's session rejects it on its next use while the others keep working, and the set of a person's active sessions can be listed for the account-settings surface to present. _Enforces REQ-ID-045._

**BAR-309** (`usable_early` · `automated_test`) — Signing in establishes a new session identity; a session identifier carried before authentication does not become an authenticated session and carries no authenticated privileges after sign-in. _Enforces REQ-ID-043._

**BAR-310** (`usable_early` · `inspection`) — The session credential is exchanged only over an encrypted transport — a request carrying it over an unencrypted channel is refused or redirected, never served — and it never appears in URLs, server or client logs, or storage readable by page scripts, so a network observer or an injected script cannot capture and replay it as the owner. _Enforces REQ-ID-041, REQ-ID-042._

**BAR-311** (`usable_early` · `inspection`) — Every private field the product exchanges with a person — the account secret at sign-up, sign-in, and recovery; the session credential; the verified contact; and the data-export payload — crosses the network only over an encrypted transport, and any request that would carry such a field in the clear is refused or redirected rather than served. _Enforces REQ-ID-100._

**BAR-312** (`usable_early` · `automated_test`) — Every action that creates, changes, or removes data, and every read restricted to an owner, requires a valid authenticated session and is refused without one; any action not explicitly permitted by the authority model is refused rather than allowed by default. _Enforces REQ-ID-060._

**BAR-313** (`usable_early` · `automated_test`) — An account's public fields (handle, display name, bio, follower/following counts, public chirps with their reply and like counts) are readable by any viewer; its owner-only fields (verified contact, credential, active session/device list, notification preferences, settings, and any export payload) and its owner-scoped views (home timeline, account-management views) are returned only to the authenticated owner and never to another account or an unauthenticated viewer. _Enforces REQ-ID-001, REQ-ID-061, REQ-ID-062, REQ-ID-063._

**BAR-314** (`usable_early` · `automated_test`) — Posting, replying, liking and unliking, and following and unfollowing are authorized for and performed as the signed-in actor and cannot be directed to take effect under another account's identity; the author recorded on any chirp, reply, like, or follow is set by the product from the authenticated session, not from actor-supplied input, and no surface lets an actor create or alter content attributed to another account. _Enforces REQ-ID-064, REQ-ID-065, REQ-ID-068._

**BAR-315** (`usable_early` · `automated_test`) — A signed-in person can change only their own profile, contact, notification preferences, and account settings, and can delete only their own chirps and their own account; every attempt to change or delete another account or its content through any surface is refused and leaves the other account unchanged. _Enforces REQ-ID-066, REQ-ID-067._

**BAR-316** (`usable_early` · `manual_user_check`) — When the authority model refuses a read or action, the response carries no owner-only field of the target account and reveals nothing about it beyond its already-public profile, and the refusal is consistent whether or not the target account exists, beyond what public lookup already discloses. _Enforces REQ-ID-069._

**BAR-317** (`usable_early` · `manual_user_check`) — A signed-in person can change their display name and bio under the same format rules as at sign-up; a valid change persists and shows on their public profile, and an invalid value is reported as a field-level error and not saved. _Enforces REQ-ID-007._

**BAR-318** (`releasable` · `manual_user_check`) — A signed-in person has one settings surface, reachable from the product's navigation, that presents in one place the verified contact, the active sessions/devices, and — where notifications exist — notification preferences. _Enforces REQ-ID-008._

**BAR-319** (`releasable` · `manual_user_check`) — From settings a person can view their current verified contact and enter a new one; the new contact becomes the recovery/notification basis only after its confirmation step is completed, and until then the prior confirmed contact remains in force. _Enforces REQ-ID-009._

**BAR-320** (`releasable` · `operational_check`) — From settings a person can see the sessions/devices currently signed in to their account and end any of them; an ended session can no longer act as the account. _Enforces REQ-ID-010._

**BAR-321** (`releasable` · `manual_user_check`) — Sign-up captures one point of contact, stores it as a private field never shown to other users, and the product confirms the person controls it before using it for recovery or notifications; an unconfirmed contact is not accepted as a basis for regaining access. _Enforces REQ-ID-004, REQ-ID-005._

**BAR-322** (`releasable` · `manual_user_check`) — A person who lost their credential can regain access themselves by proving control of the verified contact, is then prompted to set a new credential, and afterward signs in with it; the credential-reset step cannot be reached without proving contact control, and someone who does not control the contact cannot complete recovery. _Enforces REQ-ID-013, REQ-ID-014._

**BAR-323** (`releasable` · `manual_user_check`) — The recovery surface does not reveal whether a given handle or contact has an account beyond what the product already makes public; submitting a handle or contact with no account does not disclose account existence in a way that enumerates accounts. _Enforces REQ-ID-015._

**BAR-324** (`releasable` · `operational_check`) — Completing recovery and setting a new credential ends the account's sessions that predate the reset (or offers the owner the option to end them), and the old secret no longer signs in, so a recovery prompted by a possible compromise can eject an attacker. _Enforces REQ-ID-016, REQ-ID-046._

**BAR-325** (`releasable` · `manual_user_check`) — A signed-in person can obtain their own data — their chirps, profile, and follow list — through the product without contacting an operator, in a form they can open outside chirp without special tooling; the export contains only the requesting account's data and no other account's records. _Enforces REQ-ID-017, REQ-ID-018._

**BAR-326** (`releasable` · `inspection`) — A data export never includes the account's credential or session data, and never includes another person's private contact. _Enforces REQ-ID-019._

**BAR-327** (`releasable` · `manual_user_check`) — A signed-in person can delete their own account after a confirmation that explains what will and will not survive and points to export as their chance to keep a copy; cancelling at the confirmation leaves the account and all content intact, and after deletion the account can no longer sign in. _Enforces REQ-ID-020, REQ-ID-022._

**BAR-328** (`releasable` · `manual_user_check`) — Deletion applies the product's stated outcome: the person's chirps leave public view; a reply that pointed at a now-deleted chirp shows a "chirp unavailable" placeholder with like/reply counts adjusted rather than breaking the conversation; and the freed handle is retired — not immediately re-registerable — so a departed account cannot be impersonated by someone claiming its handle. _Enforces REQ-ID-021._

**BAR-329** (`releasable` · `operational_check`) — Deleting an account erases its private data — verified contact, credential, and session records — so none remains on any product surface or in an export, and immediately ends all of the account's active sessions so none can continue to act as the deleted account. _Enforces REQ-ID-023, REQ-ID-024, REQ-ID-088._

**BAR-330** (`releasable` · `manual_user_check`) — The product states, where a person can find it, a data-rights baseline — access and a portable copy of their own data, and erasure by deleting their account — and every part of it is satisfied through the account's own surfaces, with no step requiring the person to contact an operator by hand. _Enforces REQ-ID-090._

**BAR-331** (`releasable` · `inspection`) — The authority to perform the supported administrative actions — assist recovery, remove content, action an abusive account — is provisioned outside the ordinary sign-up/sign-in path and is carried by no ordinary session; an ordinary account attempting any operator action is refused, and the operator surface exposes no authority beyond the supported set. _Enforces REQ-ID-070, REQ-ID-071._

**BAR-332** (`releasable` · `inspection`) — No operator action posts, replies, likes, or follows as a user, and no operator action returns any account's credential; assisting a recovery routes the user through their own verified contact rather than exposing or resetting the credential to the operator. _Enforces REQ-ID-072._

**BAR-333** (`releasable` · `operational_check`) — Each operator action on an account or on content records which operator performed it, what was done, the target, and when, in a retained record not editable through the ordinary product surfaces; and operator content removal and account actioning apply the product's defined outcome (consistent with the account-deletion outcome), with the applied outcome recorded. _Enforces REQ-ID-073, REQ-ID-074._

**BAR-334** (`releasable` · `manual_user_check`) — A signed-in person can report a chirp or an account, receives an acknowledgement that the report was received, and can point the report at a specific rule; the rules that say what is and is not allowed are reachable from within the product without an external link. _Enforces REQ-ID-080, REQ-ID-081._

**BAR-335** (`releasable` · `operational_check`) — A filed report is recorded and made available to the authorized operator/support surface with what was reported, by whom, against whom, and which rule — enough to act on — rather than merely acknowledged to the reporter and discarded. _Enforces REQ-ID-082._

**BAR-336** (`releasable` · `operational_check`) — The product throttles or rejects writes and requests from one account or client that exceed a reasonable rate rather than serving them without limit, while a legitimate user at a normal rate is not blocked. _Enforces REQ-ID-085._

**BAR-337** (`releasable` · `operational_check`) — The product slows, challenges, or rejects automated mass account creation at the public sign-up surface and resists automated high-volume writes from many fresh accounts, while a legitimate person signing up once is not blocked. _Enforces REQ-ID-101._

**BAR-338** (`usable_early` · `manual_user_check`) — Through every surface available to another user, to an unauthenticated visitor, and through the report/block/mute flows, only an account's public fields appear; its verified contact, credential, and session data are never returned to anyone else, and the credential is never returned to anyone in readable form. _Enforces REQ-ID-087._

**BAR-339** (`releasable` · `inspection`) — Every field that sign-up and the account surfaces collect and store maps to a stated account function — identity, authentication, recovery/contact, or profile — and no field is collected that no function needs. _Enforces REQ-ID-086._

---

### Operational readiness (`BAR-400..499`) — enforces `design/operational-readiness.md`

_49 criteria — 15 `usable_early`, 34 `releasable`._

**BAR-400** (`usable_early` · `operational_check`) — The deployment runs the product's authoritative state in a durable store that is a separate, persistent component from the serving process, with its own lifecycle; stopping or replacing the serving process does not stop or reset the store, and no deployment configuration keeps authoritative state only in serving-process memory. The in-memory backing of the `ChirpRepository` seam at `Nousix-LLC/nswe-demo @ c805525` does not meet this; closing it is the blocker this bar turns on. _Enforces REQ-OPS-002, REQ-OPS-009._

**BAR-401** (`usable_early` · `operational_check`) — On a populated deployment, each of a serving-process restart, a redeploy, and a replacement of the serving process with a fresh instance leaves every Account and its Chirps, Follows, and Likes present and unchanged, with no manual step to recover them — demonstrating the authoritative state did not live in the process that was replaced. _Enforces REQ-OPS-001, REQ-OPS-003._

**BAR-402** (`usable_early` · `operational_check`) — A create or change the product has confirmed to a person — a posted Chirp, a Follow or unfollow, a Like or unlike, a profile edit, an account deletion — is present after a restart or process replacement that happens immediately after the success response, with no further action taken. No acknowledged write is lost in the window right after its acknowledgement. _Enforces REQ-OPS-005._

**BAR-403** (`usable_early` · `operational_check`) — After the serving process is restarted or replaced, the durable data still satisfies the domain invariants: every reference resolves, ids are unchanged and never reused, each Account's follower/following counts and each Chirp's like count equal their current edges, and any applied account deletion stays applied. Adding and removing a Follow and a Like after the restart moves the counts by exactly one each way. _Enforces REQ-OPS-006._

**BAR-404** (`usable_early` · `operational_check`) — A restart, crash, or process replacement during a write leaves the durable data consistent: after recovery each interrupted operation — posting a Chirp, following, deleting an account that has Chirps, Follows, and Likes — is observably either fully applied or fully absent, with no half-applied state (a Chirp with no resolvable author, a half-deleted Account with dangling references, a count out of step with its edges). _Enforces REQ-OPS-008._

**BAR-405** (`releasable` · `operational_check`) — An upgrade performed in production that changes how the data is shaped preserves the user data that existed before it: afterward the Accounts, Chirps, Follows, and Likes created before the upgrade are all present and their references still resolve, with none lost or corrupted by the upgrade. _Enforces REQ-OPS-004._

**BAR-406** (`usable_early` · `manual_user_check`) — While an account exists, its data — the Account and profile, its Chirps, its Follows, its Likes — remains present and complete over a sustained operating period with ordinary restarts and redeploys, without the holder re-creating it. Any age-out the product applies to a live account's content is found stated in the retention posture, not met as an unexplained loss. _Enforces REQ-OPS-010._

**BAR-407** (`releasable` · `operational_check`) — When an account is deleted, the stated deletion outcome holds immediately (the account cannot sign in and no dangling reference remains), and after the product's stated, bounded retention window the deleted account's data is no longer present in the live durable store. The observed behaviour matches the stated window. _Enforces REQ-OPS-011._

**BAR-408** (`releasable` · `inspection`) — The product states a maximum backup-retention bound that limits how long a deleted account's data can remain recoverable in backups, and that bound is consistent with the deleted-account retention posture; no backup older than the bound is retained, so deleted-account data cannot be recovered from backups indefinitely. _Enforces REQ-OPS-012._

**BAR-409** (`usable_early` · `operational_check`) — An active Session survives a restart or redeploy — a request carrying its token is still accepted as the same account afterward — while a Session that has ended (sign-out) or passed its lifetime is refused after a restart and after a restore from a backup taken while it was active, never revived. _Enforces REQ-OPS-014._

**BAR-410** (`usable_early` · `operational_check`) — The account's Credential and Verified contact are retained durably — a person can still sign in and the recovery path still recognises the contact after a restart and a restore — never appear in any public response, and are no longer retained in the live durable store once the account is deleted and its retention window elapses. _Enforces REQ-OPS-015._

**BAR-411** (`releasable` · `manual_user_check`) — For a signed-in account with Chirps and Follows, the data the product retains and the data a self-service export yields are the same set — the account's profile, its complete Chirps, and its complete followed Accounts — matching the committed durable state. No retained category of a person's own data is missing from what they can export. _Enforces REQ-OPS-013._

**BAR-412** (`releasable` · `inspection`) — The product's data-retention and deletion posture is published where a user and an operator can find it, and it names what is retained, the live-account retention, the deleted-account retention window, the backup-retention bound, and session and private-identity-data handling. The stated posture matches the behaviour the retention and deletion criteria verify; a discrepancy is a defect. _Enforces REQ-OPS-016._

**BAR-420** (`releasable` · `operational_check`) — An operated deployment with user data produces a backup of its durable data (Accounts, Chirps, Follows, Likes) on the product's stated, regular cadence without an operator remembering to take it, and each produced backup is recorded where an operator can see it was taken. Observed over several cadence intervals, a backup appears each interval, not ad hoc. _Enforces REQ-OPS-020._

**BAR-421** (`releasable` · `inspection`) — Backups are kept for the product's stated, bounded retention period and no longer: a backup within the period is available to restore from, a backup older than the period is no longer retained, and the period is consistent with the deleted-account backup bound (BAR-408). The oldest available backup is no older than the stated period. _Enforces REQ-OPS-021._

**BAR-422** (`releasable` · `operational_check`) — A backup taken while the product is serving restores to a state in which the model's invariants hold — every reference resolves, ids are unchanged and unreused, counts equal their edges — with no partially captured write visible. A backup that restores to an invariant-violating state is treated as unusable, not accepted. _Enforces REQ-OPS-022._

**BAR-423** (`releasable` · `inspection`) — Backups are held so that losing the primary durable store does not also lose them: the two do not share a single point of failure, and with the primary store made unavailable a retained backup is still reachable to restore from. _Enforces REQ-OPS-023._

**BAR-424** (`releasable` · `inspection`) — The product states a recovery-point objective — a bounded quantity of recent durable data a recovery may lose, no tighter than the backup cadence can achieve — and a recovery-time objective — a bounded duration the product may be unavailable during recovery, consistent with its availability commitment. Both name a bounded figure, not "none" without a mechanism to achieve it. _Enforces REQ-OPS-024, REQ-OPS-025._

**BAR-425** (`releasable` · `inspection`) — The restore procedure is written down and followable by an operator who did not build the system: following it, that operator restores a deployment from a retained backup and returns it to service without knowledge that is not in the procedure, and the procedure names what is restored (Accounts, Chirps, Follows, Likes) and how the product is returned to serving. _Enforces REQ-OPS-026._

**BAR-426** (`releasable` · `operational_check`) — A restore from a retained backup returns the Accounts, Chirps, Follows, and Likes present at the backup point intact and consistent — references resolve, ids are unchanged and unreused, counts equal their edges, and an account deleted before the backup stays deleted with no dangling reference. A failing invariant is reported as an unsuccessful restore, never silently accepted. _Enforces REQ-OPS-027, REQ-OPS-007._

**BAR-427** (`releasable` · `operational_check`) — A timed restore from a backup on a representative deployment recovers the product within the stated recovery-time objective and loses no more than the stated recovery-point objective, both measured rather than asserted. A drill that exceeds either objective is reported as a defect in the objective or the procedure, not ignored. _Enforces REQ-OPS-028._

**BAR-428** (`releasable` · `operational_check`) — A deploy can be rolled back to the last working version as a routine operator operation with no user-data loss: after the rollback the product serves on the prior version and every Account, Chirp, Follow, and Like that existed before the deploy is present and unchanged, with no manual reconstruction. _Enforces REQ-OPS-029._

**BAR-429** (`releasable` · `operational_check`) — A deploy that damages or loses durable data can be recovered from by restoring from a pre-deploy backup following the restore procedure, returning the durable data to an intact and consistent state (as BAR-426) within the stated objectives. The product does not depend on the damaging deploy being reversible in place. _Enforces REQ-OPS-030._

**BAR-430** (`releasable` · `operational_check`) — A restore from a recent backup is exercised on the product's stated recurring drill cadence against a store where its correctness can be checked; the drill verifies the restored data's invariants and timing, and a failed drill is recorded and acted on rather than passed over. The drill is recurring, not a one-time pre-release event. _Enforces REQ-OPS-031._

**BAR-431** (`releasable` · `operational_check`) — A scheduled backup that fails or does not run is surfaced to operators as a visible failure state or alert, not absorbed silently, so a deployment cannot drift into having no recent usable backup unnoticed. A run of successful backups and a failed backup are distinguishable to an operator. _Enforces REQ-OPS-032._

**BAR-432** (`releasable` · `inspection`) — The product's backup-and-recovery posture is published where an operator can find it, naming the backup cadence and retention, the recovery-point and recovery-time objectives, and the restore and rollback procedures; the stated posture matches the behaviour the backup-and-recovery criteria verify. A discrepancy is a defect. _Enforces REQ-OPS-033._

**BAR-440** (`releasable` · `operational_check`) — The product bounds the request rate it accepts from a single client, applied on both the authenticated surface and the unauthenticated surface (landing, sign-up, sign-in) and identified by client rather than only by account. A client issuing requests far faster than a person would is refused or slowed past a point — and cannot escape the bound by not signing in — while other clients continue to be served normally. _Enforces REQ-OPS-040, REQ-OPS-046._

**BAR-441** (`releasable` · `operational_check`) — State-creating actions — posting a Chirp, following, liking — are held to their own, tighter rate limit than reads: performing one repeatedly far faster than a person would is refused or throttled past a point, while reading remains possible within its own limit meanwhile. _Enforces REQ-OPS-041._

**BAR-442** (`releasable` · `operational_check`) — The volume of content one account can create over a longer rolling window is bounded: creating content continuously at or just under the per-action rate limit for a sustained period is eventually bounded for that window rather than accumulating without limit, and resumes in a later window. _Enforces REQ-OPS-042._

**BAR-443** (`usable_early` · `automated_test`) — A request whose body is far larger than any valid chirp or form submission (for example a multi-megabyte body to a write endpoint) is refused with an error before the body is processed, while a normal, within-bounds request to the same endpoint succeeds. _Enforces REQ-OPS-043._

**BAR-444** (`usable_early` · `automated_test`) — Every endpoint that returns a list serves at most the server's maximum page size: a request with no page size and a request with a page size well above the maximum both return at most the maximum number of items, with a continuation cursor when more remain, rather than the full result set. _Enforces REQ-OPS-044._

**BAR-445** (`releasable` · `operational_check`) — Exceeding any limit above refuses or throttles the excess rather than serving it, and returns a typed, machine-readable error whose category identifies it as a limit being hit — distinct from a validation error or a not-found — so a caller can tell it was limited and back off. The error is the typed contract category, not an ad-hoc string. _Enforces REQ-OPS-045._

**BAR-460** (`usable_early` · `operational_check`) — The product exposes a readiness signal, distinct from liveness, whose meaning is that it can serve users: it reports ready when the durable store is reachable and the product can serve, reports not-ready when the store is unreachable from the serving process, carries a machine-distinguishable ready/not-ready indication an orchestrator can branch on without reading prose, and returns to ready on its own — without a restart or redeploy — once the condition that made it not-ready clears. _Enforces REQ-OPS-060, REQ-OPS-061, REQ-OPS-064, REQ-OPS-065._

**BAR-461** (`usable_early` · `operational_check`) — The liveness signal reports the serving process's own health only: with the durable store unreachable but the process otherwise healthy, liveness reports alive while readiness reports not-ready, so an orchestrator restarts a genuinely crashed process but does not kill one that is alive and waiting on its store. Liveness reports not-alive only when the process itself cannot make progress. _Enforces REQ-OPS-062._

**BAR-462** (`usable_early` · `manual_user_check`) — The health/readiness endpoint responds when queried with no account or session, and its response carries no credential, session token, verified-contact value, or other private account data, and no secret such as a connection string — it conveys serving status, not the contents of the system. _Enforces REQ-OPS-063._

**BAR-463** (`usable_early` · `inspection`) — The product records its operational events as structured, machine-parseable records with consistent, named fields (such as timestamp, severity, event category, and outcome) that a log tool can parse and filter, not as free-form prose. _Enforces REQ-OPS-066._

**BAR-464** (`releasable` · `operational_check`) — The product's logs cover the operational events needed to explain operation and failure: request handling and its outcome, errors and failures with a cause category, process start and stop, changes in durable-store connectivity, and successful and failed authentication outcomes. Provoking each in a running product produces a distinct, categorized log event with enough context to tell what happened. _Enforces REQ-OPS-068._

**BAR-465** (`usable_early` · `inspection`) — No credential, session token, verified-contact value, or other private account data appears in the product's logs or its audit trail, in any field at any severity: exercising sign-up, sign-in, recovery, session use, and representative administrative actions and then inspecting the logs and audit entries shows accounts referenced by non-secret identifier only, with no private data or secret present. _Enforces REQ-OPS-069, REQ-OPS-079._

**BAR-466** (`releasable` · `operational_check`) — The product exposes serving metrics that report the rate of requests handled and the rate failing, broken down enough to see which class of surface is affected (for example reads versus writes, or by outcome category); the metric labels carry no handle-linked private data, credential, or contact value. _Enforces REQ-OPS-072._

**BAR-467** (`releasable` · `operational_check`) — The product exposes a serving/availability signal that makes "the product is not serving its core loop" an observable fact: put into a degraded state in which the core loop does not work for users, the signal reflects the degradation; returned to serving, it reflects recovery. The signal is suitable to drive automated detection. _Enforces REQ-OPS-075._

**BAR-468** (`releasable` · `operational_check`) — Every action taken through the supported administrative surface — assisting a user's recovery, removing content, disabling or actioning an account — produces an audit entry naming the acting operator identity, the action taken, the target account or content, and the time. An administrative action that reaches accounts or content leaves no untraced path. _Enforces REQ-OPS-077._

**BAR-480** (`releasable` · `operational_check`) — A serving degradation — the product not serving its core loop, or serving it with elevated errors — is detected from the observability signals (the serving/availability signal, request- and error-rate metrics, saturation/headroom, durable-store interaction health) and raises a notification to the responsible operator within a bounded time, without a user having to report it first. Each alerting condition names an observability signal and a threshold or state on it; returning the product to serving raises no further alert for that incident. _Enforces REQ-OPS-080, REQ-OPS-081._

**BAR-481** (`releasable` · `inspection`) — There is a stated role accountable for acknowledging and acting on a detected incident, so a fired alert has an owner; a detected major incident has a defined responder and no detected incident is left with no one responsible for it. _Enforces REQ-OPS-083._

**BAR-482** (`releasable` · `manual_user_check`) — When the product is unavailable or degraded, a person consulting a status surface or in-product notice sees that chirp itself is affected rather than being left to infer it from a failed action or a spinner; when the product is serving normally, the surface shows normal operation. _Enforces REQ-OPS-085._

**BAR-483** (`releasable` · `operational_check`) — The surface users consult to learn of an outage is reachable when the serving product and its durable store are fully offline: with the product taken fully down, the status surface is still reachable and reports the product as unavailable. Its availability does not require the product's serving path, store, or process to be up. _Enforces REQ-OPS-086._

**BAR-484** (`releasable` · `inspection`) — Nothing published on the status surface, in an in-product outage notice, or in a maintenance announcement contains private account data or secret material — no credential, session token, verified-contact value, connection string, or raw internal error detail that discloses them. Inspected for a representative outage and a maintenance notice, the content describes product status in the terms a user needs and nothing more. _Enforces REQ-OPS-088._

**BAR-485** (`releasable` · `manual_user_check`) — With the durable store made unreachable while the serving process stays up, a user exercising the core loop meets the explained degraded or error experience the experience design specifies — an explained error, a preserved in-progress draft, a routed-to-sign-in on an expired session — not a blank page or a crashed process. _Enforces REQ-OPS-090._

**BAR-486** (`releasable` · `inspection`) — The product states an availability target that defines what "reasonably available" means for it — a specific figure over a stated window (for example a monthly serving-availability target), proportionate to a small hosted service rather than an enterprise commitment — written down, not left implicit. _Enforces REQ-OPS-092._

**BAR-487** (`releasable` · `operational_check`) — Availability is computed in production from the serving/availability signal over the stated window, counting the product as unavailable when the signal shows it not serving its core loop, so whether the target was met is a measured fact rather than a claim. Over a window that includes at least one serving interruption, the measured result reflects the interruption and can be compared against the stated target. _Enforces REQ-OPS-093._

---

## Completeness and gap review

A deliberate sweep over the surfaces a product of this kind — a public, consumer, individual-account social service — is expected to have. For each, a judgment is recorded: in scope (designed, and where it is held in the bar) or out of scope (with the reason), so an absence is a decision and not an oversight. The sweep covers the account and operational surfaces the engagement named, and the trust, safety, and discovery surfaces this product kind adds.

### Account and operational surfaces

| Surface | Judgment | Where it is held / why it is out |
|---------|----------|-----------------------------------|
| Sign-up | In scope | Real credential and a point of contact captured; handle format and uniqueness enforced with field-level feedback. `BAR-007`, `BAR-008`, `BAR-231`, `BAR-232`, `BAR-303`, `BAR-321`. |
| Sign-in | In scope | Owner-only sign-in with a held secret; a clear not-found vs wrong-credential signal that leaks no private existence. `BAR-009`, `BAR-010`, `BAR-233`, `BAR-300`, `BAR-304`, `BAR-305`. |
| Recovery | In scope (releasable) | Self-service recovery gated on proven control of the verified contact, ending sessions that predate the reset. `BAR-015`, `BAR-234`, `BAR-321`–`BAR-324`. |
| Account deletion | In scope (releasable) | Self-service, confirmed, with a stated and applied outcome for content and handle; private data erased and sessions ended; durable. `BAR-020`–`BAR-022`, `BAR-118`, `BAR-255`, `BAR-327`–`BAR-329`, `BAR-407`. |
| Data export | In scope (releasable) | A person can obtain their own chirps, profile, and follow list in a portable form, excluding secrets and others' data. `BAR-019`, `BAR-119`, `BAR-147`, `BAR-254`, `BAR-325`, `BAR-326`, `BAR-411`. |
| Onboarding | In scope | A fresh sign-up lands on an onboarding screen that explains the empty state and offers a first action, never a blank page. `BAR-027`, `BAR-213`, `BAR-237`, `BAR-248`. |
| Settings | In scope | One settings hub with labelled sub-sections — edit profile, account & contact, sessions & devices, export, delete (and notification preferences only if notifications ship). `BAR-017`, `BAR-018`, `BAR-214`, `BAR-249`–`BAR-252`, `BAR-317`–`BAR-320`. |
| Notifications | Out of scope for v1 (deferred, designed) | A baseline notifications story is designed and expected but is marked non-gating by every area (`REQ-UX-019/072/078`, `REQ-ID-011`); the release does not gate on it. Recorded as a near-term fast-follow rather than a v1 release condition, so its absence at release is a stated decision. No notifications criterion is gated; settings shows the preferences row only where notifications ship. |
| Admin / support | In scope (releasable) | A provisioned, least-privilege operator surface (assist recovery, remove content, action an abusive account) that cannot impersonate users or read credentials, with an audit trail; plus user-facing help & support. `BAR-025`, `BAR-258`, `BAR-331`–`BAR-333`, `BAR-335`, `BAR-468`. |
| Billing | Out of scope (by product kind) | chirp is a single public consumer product of individual accounts where no money changes hands; tenancy, team/seat management, org-admin, and billing are an explicit non-goal (`need: no-tenancy-or-billing`). There is no billing surface to design, so none is gated. |

### Trust, safety, and discovery surfaces this product kind adds

| Surface | Judgment | Where it is held / why it is out |
|---------|----------|-----------------------------------|
| Public first visit / landing | In scope | A visitor with no account sees what chirp is and a clear path to sign up, not a sign-in wall. `BAR-026`, `BAR-230`, `BAR-268`. |
| Reporting abuse | In scope (releasable) | A signed-in person can report a chirp or account against stated rules and is acknowledged; reports are routed to the operator surface with enough context to act. `BAR-023`, `BAR-256`, `BAR-334`, `BAR-335`. |
| Rules / content policy | In scope (releasable) | The product states its rules as named categories, reachable in-product from both shells, and a report points at one. `BAR-024`, `BAR-257`. |
| Block / mute | Out of scope for v1 (deferred, designed) | Designed and expected unwanted-interaction controls, marked non-gating by the product and identity areas (`REQ-PROD`'s unwanted-interaction requirement, `REQ-ID-083/084`). The gated safety floor for v1 is reporting plus operator action, both of which ARE gated; block/mute is recorded as a near-term fast-follow. This is a stated decision, with the residual risk accepted in the identity design. |
| Discovery / Explore (find people to follow) | Out of scope for v1 (deferred, designed) | Marked non-gating (`REQ-UX-064`); the area designs flag it as the main social-completeness gap. The bar's explicit call: v1 gates the ability to follow from any profile and from a person row (`BAR-005`, `BAR-245`), so a person can build a timeline without a discovery surface, and a dedicated Explore/discovery surface is a near-term follow-up rather than a v1 gate. Absence is a recorded decision, not a default. |
| Sessions & devices management | In scope (releasable) | A person can see where they are signed in and end any session; an ended session can no longer act. `BAR-018`, `BAR-252`, `BAR-308`, `BAR-320`. |
| Accessibility statement & conformance | In scope (releasable) | The product states WCAG 2.1 AA and its gated set by name, and the release record shows the gated set has no open A/AA failure. `BAR-275`, `BAR-293`, `BAR-294`. |
| Status / incident surface | In scope (releasable) | A status surface reachable when the product itself is down tells a person chirp is affected rather than leaving them to guess. `BAR-482`, `BAR-483`. |
| Privacy / data-rights & retention posture | In scope (releasable) | A stated data-rights baseline satisfied through the account's own surfaces, and a published retention and deletion posture that matches observed behaviour. `BAR-330`, `BAR-412`. |

### Coverage and findings

- **Every release-gating requirement is enforced.** Across the five areas, all 342 requirements marked `release_gating: true` (Product 35, Domain 79, Experience 106, Identity 63, Operational 59) are each referenced by at least one `BAR-*`. No release-gating requirement is left without a criterion.
- **No cross-cutting criterion was required.** The reserved cross-cutting range `BAR-900..999` is held for a gap-review finding that needs a release condition no single area already holds. The sweep found none: every expected surface is either held by an area criterion or recorded out of scope above, and the three deferred surfaces (notifications, block/mute, discovery) rest on requirements the area designs mark non-gating, so minting a bar for them would either duplicate an area criterion or gate something the designs deliberately did not. The reserved range is therefore unused.
- **The product can be adopted end to end.** A stranger can find chirp, sign up, post and read, follow and be followed, get help, report what breaks the rules, export their data, and delete their account — each of those is a gated release condition above — and what they create survives a restart. The one structural blocker in the current product at `c805525`, the in-memory store, is closed by the durable-persistence criteria at `usable_early`.
- **The gaps at release are named, not hidden.** Notifications, block/mute, and a discovery surface are real parts of a finished social product and are deliberately deferred as near-term follow-ups; the safety floor that covers for block/mute (reporting plus operator action) and the ability to follow without a discovery surface are gated, so the deferral is a scoped decision rather than a hole.

