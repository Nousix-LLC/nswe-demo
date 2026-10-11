# Identity, Security & Trust — chirp

<!-- design/identity-security.md — owner: identity & trust; requirement area: REQ-ID-*. -->

This document specifies the **identity, security and trust surface** a releasable, publicly-exposed
chirp must have: how a person gets an account and proves it is theirs, how their session stays safe on
the public internet, what each actor may see and change, the abuse-resistance and privacy/data-rights
basics a public product owes its users, the **threat model** this surface implies, and the consolidated
register of testable `REQ-ID-*` requirements with their acceptance criteria and traceability.

It states **what must be true** — from the user's, operator's, and attacker's side — not how any of it is
built. Every implementation "how" (the authentication protocol, session and token mechanics,
cryptography, the rate-limiting mechanism, the storage schema) is stated here as a requirement with a
testable bar and deferred, by prose, to later code work in the `software` domain and to the sibling design
areas that own a mechanism — experience design (`REQ-UX-*`), domain & application design (`REQ-DOM-*`),
and operational readiness (`REQ-OPS-*`).

**Grounding.** The product being designed is `Nousix-LLC/nswe-demo` at commit `c805525`. Its current
surface, confirmed against the source at that commit, is: **handle-only sign-in** — the sign-in request
(`POST /api/sessions`) carries only a username and no credential of any kind, so anyone who types a known
handle is accepted as that account; **sessions and all data held in memory** — a successful sign-in
returns a bearer token resolved from an in-memory store, with no expiry, no server-side invalidation, and
no sign-out route, and every session and record is lost on restart; and **no account recovery, no account
deletion, no data export, no moderation or admin surface, no rate or volume limits, and no privacy/terms
surface**. Public reads (the timeline and profiles) require no account. Every requirement and threat below
is grounded in that real surface, so each gap it closes is concrete, not hypothetical.

**How to read this document.** The six required sections are `accounts_and_lifecycle`,
`authentication_and_sessions`, `authorization`, `abuse_and_privacy_basics`, `threat_model`, and
`requirements_and_acceptance_criteria`. Requirements are partitioned by concern so that no two sections
mint the same id: accounts & lifecycle owns `REQ-ID-001..019`; authentication & sessions
`REQ-ID-020..039`; authorization `REQ-ID-040..059`; abuse & privacy `REQ-ID-060..079`; and the threat
model's net-new, cross-cutting requirements `REQ-ID-080..099`. Each requirement states what must be true,
gives acceptance criteria a person who never met the author can check against the live product, carries a
`release_gating` flag, and traces to at least one product requirement (`REQ-PROD-*`). The final section
consolidates the whole register and resolves every cross-section coupling into an explicit
`REQ-ID ↔ REQ-ID` reference.

---

---

## accounts_and_lifecycle

<!-- Section: accounts_and_lifecycle of design/identity-security.md. The account as a thing that is created, lived in, and left: creation & credential establishment, credential recovery, account deletion (semantics + honoured guarantee), and data export. Requirements only (REQ-ID-001..017) — no implementation "how". -->

<!--
accounts_and_lifecycle — a section of design/identity-security.md.
The identity-side of the account as a thing that is created, lived in, and left.
Consolidated requirements REQ-ID-001..017 across four surfaces: account creation & credential
establishment, credential recovery, account deletion (semantics + honoured guarantee), and data export.
Requirements only — the engineering "how" is deferred by prose to the areas and code DAGs that own it.
Intra-section couplings are resolved into explicit REQ-ID cross-references below; cross-section couplings
to authentication_and_sessions / authorization / abuse_and_privacy_basics are carried forward for the
identity & trust synthesis to resolve — they are NOT resolved here.
-->

### accounts_and_lifecycle

An account on a public social product is not a single moment — it is a life. A person **creates** an
account, **lives in** it day to day, and eventually **leaves** it. This section designs the identity-side
of that whole life as testable requirements: the account is created *with* a real ownership credential;
a locked-out owner has a safe way back in; an owner can delete the account and have that deletion actually
honoured; and an owner can take their own data out in a portable form. These are the account surfaces a
releasable consumer product of this kind is expected to carry, and they are the ones products most often
ship without.

**Grounded in the product as it stands.** In `chirp` today, sign-in is **handle-only**: a visitor becomes
a given user simply by presenting that user's handle, because no credential is ever established — an account
is a public handle, a display name, and an optional bio, nothing more. There is **no credential**, **no
recovery journey**, **no account-deletion capability** (no UI control and no API route), and **no
data-export capability**. The requirements below close those gaps at the lifecycle level; each gap is real,
not hypothetical.

**Requirements only — what, not how.** Every requirement states what must be true from the user's or
operator's side, with acceptance criteria a person who never met the author can check against the live
product, a `release_gating` flag, and a trace to at least one `REQ-PROD-*`. No requirement chooses a
library, protocol, token format, export serialization, or erasure mechanism; each such "how" is deferred by
prose to the section or the code DAGs (`software` domain) that own it.

**What this section defers to sibling identity & trust sections (carried forward, not resolved here):**

- The **authentication mechanism** — what *verifying* a credential means, how a session is issued, held,
  and invalidated — is owned by `authentication_and_sessions` (`REQ-ID-020..039`). This section establishes
  that there is a credential to verify and what recovery must re-prove; it does not define the verification.
- The **authority** of who-may-delete and who-may-export-on-behalf, and moderation/admin authority, is owned
  by `authorization` (`REQ-ID-040..059`). This section states the lifecycle; the authority is referenced.
- The **privacy / data-rights framing** — what personal data is held and the data-right that export and
  erasure satisfy — is owned by `abuse_and_privacy_basics` (`REQ-ID-060..079`). This section states the
  mechanism-side requirement; the right is referenced.
- The **screens** (sign-up, recovery-entry, deletion confirmation, export control) are experience design
  (`REQ-UX-*`); the **durable-removal operational proof** is operational readiness (`REQ-OPS-*`); the
  export **format internals** and erasure **mechanics** are code DAGs (`software` domain) / domain design
  (`REQ-DOM-*`). Each is named at the point it applies.

---

#### Surface 1 — Account creation & credential establishment (REQ-ID-001..005)

This surface designs the single moment at which an account comes into existence: that it is created **with a
real credential** that can later prove it belongs to a specific person and can be recovered, and that its
**handle** is valid and unique. It closes the handle-only gap at the lifecycle level — for the rest of the
account's life the thing that proves ownership is that credential, never the public handle.

##### REQ-ID-001 — An account is created with a credential that proves ownership
- **Statement:** Creating a chirp account establishes a **credential** — a secret or factor held only by the
  account's owner — bound to that account at the moment of creation. From then on, control of that
  credential, not knowledge of the public handle, is what establishes that someone is the account's owner.
- **Acceptance criteria:**
  - Completing account creation results in an account for which an ownership credential has been established;
    a creation attempt that establishes no credential does not produce an account.
  - A person who knows only the account's public handle is not treated as its owner: knowing the handle alone
    is not sufficient to be recognised as the account for any owner-only action. (The mechanism that
    *verifies* the credential and issues a session is defined in `authentication_and_sessions`; this
    requirement establishes that there is a credential to verify.)
  - The credential established is distinct from, and not derivable from, the account's public handle or
    display name.
- **Release gating:** true
- **Traceability:** REQ-PROD-001, REQ-PROD-004; journeys J2 (sign-up), J3 (sign-in). Credential verification
  & session issuance → `authentication_and_sessions` (`REQ-ID-020..039`); credential storage/verification
  "how" → `software` domain.
- **Resolved intra-section cross-references:** re-established by recovery — **REQ-ID-006**, **REQ-ID-008**;
  removed by deletion — **REQ-ID-012**, **REQ-ID-005**.

##### REQ-ID-002 — A recovery factor is established at creation, so the account is recoverable
- **Statement:** At account creation the product establishes at least one **recovery factor** — something the
  owner controls — bound to the account, so that an owner who later loses their credential has a basis on
  which to re-prove ownership. This factor is set up as part of creating the account, not bolted on
  afterwards.
- **Acceptance criteria:**
  - Completing account creation results in an account that has at least one recovery factor on record,
    established or confirmed by the creating user during creation.
  - During creation the product makes clear that this factor is what will be used to regain access if the
    credential is later lost.
  - The product does not silently create an account for which no recovery is possible: either a recovery
    factor is established, or the user is clearly told, at creation, that the account cannot be recovered.
    (The recovery path — how the factor is challenged and access re-granted — is defined by REQ-ID-006..009;
    this requirement only establishes the factor at creation.)
- **Release gating:** true
- **Traceability:** REQ-PROD-004, REQ-PROD-001; journey J2, feeding J4 (recovery). What recovery must
  re-prove / auth mechanism → `authentication_and_sessions` (`REQ-ID-020..039`).
- **Resolved intra-section cross-references:** re-proved by recovery — **REQ-ID-008** (and **REQ-ID-007**'s
  no-bypass safety property rests on this factor).

##### REQ-ID-003 — Handle validity is enforced at account creation
- **Statement:** A chosen handle must satisfy the product's handle rules — **no more than 15 characters** and
  **only the allowed character set** — and account creation enforces these rules, explaining any rejection in
  terms the user can act on, before an account is created.
- **Acceptance criteria:**
  - Attempting to create an account with a handle longer than 15 characters, or containing characters outside
    the allowed set, is refused, and no account is created.
  - The refusal states specifically which rule was broken (too long, or a disallowed character) in language a
    user can act on, without requiring them to read a raw API error to understand it.
  - A handle that satisfies every rule is accepted.
- **Release gating:** true
- **Traceability:** REQ-PROD-002; journey J2. Inline-feedback presentation on the sign-up screen →
  experience design (`REQ-UX-*`).

##### REQ-ID-004 — Handle uniqueness is enforced; a taken handle is surfaced clearly
- **Statement:** Each account has a **unique** handle. Attempting to create an account with a handle that is
  already in use is refused clearly; no duplicate account is created and no existing account is altered.
- **Acceptance criteria:**
  - Attempting to create an account with a handle that already belongs to an existing account yields a clear
    "that handle is taken" result.
  - No second account is created for that handle, and the existing account — its owner, credential, recovery
    factor, profile, and content — is not overwritten, merged, or otherwise changed.
  - Choosing a different, available handle then allows creation to succeed.
- **Release gating:** true
- **Traceability:** REQ-PROD-003; journey J2.

##### REQ-ID-005 — Creation yields one complete, owned account, and the credential is its durable ownership anchor
- **Statement:** Account creation either produces a **single complete account** — a unique valid handle, its
  profile (display name and optional bio), the established ownership credential, and a recovery factor, bound
  together as one account owned from the moment of creation — or it fails and creates nothing. The credential
  established at creation remains the account's **ownership anchor for the rest of its life**, so that what
  proves ownership at sign-in, at recovery, and for every owner-only action is always that credential and
  never the public handle.
- **Acceptance criteria:**
  - A successful creation yields exactly one account that has all of: a unique, valid handle (REQ-ID-003,
    REQ-ID-004); a display name; the established ownership credential (REQ-ID-001); and a recovery factor
    (REQ-ID-002). A creation that cannot establish all of these produces no account, and no partially created,
    handle-only, or credential-less account is left behind.
  - For the life of the account, owner-only actions are gated on the credential established at creation, not
    on the public handle — observable by a second person who knows the handle but not the credential being
    unable to act as that account.
  - The account that deletion later removes, and that recovery later restores access to, is this same bound
    account established here.
- **Release gating:** true
- **Traceability:** REQ-PROD-001, REQ-PROD-004; journeys J2, J3, J7. Durable, honoured removal of this
  account → REQ-ID-012/REQ-ID-013 (below) and operational readiness (`REQ-OPS-*`).
- **Resolved intra-section cross-references:** the bound account (and its credential) removed by deletion —
  **REQ-ID-012**; restored-access target of recovery — **REQ-ID-006**.

---

#### Surface 2 — Credential recovery (REQ-ID-006..009)

Once an account is created **with a real credential** (REQ-ID-001..005), a new failure becomes possible that
does not exist in a handle-only product: a legitimate owner can **lose that credential** and be locked out of
an account that is genuinely theirs. A consumer product of this kind is expected to give that person a way
back in. The whole surface is governed by one tension: recovery must be **easy enough for the real owner when
they are locked out**, and **hard enough that it never becomes a second way in for someone who is not the
owner**. The no-bypass property (REQ-ID-007) is not an add-on to recovery — it is what makes recovery
releasable. The credential recovery re-establishes is the one set up at creation (REQ-ID-001); the recovery
factor it relies on is the one established at creation (REQ-ID-002).

##### REQ-ID-006 — A locked-out owner has a defined path back to the same account
- **Statement:** A user who holds a credentialed account but can no longer sign in has a defined, in-product
  path that returns them to **the same account** — the same handle, chirps, follows, and likes — rather than
  forcing them to abandon it and start over.
- **Acceptance criteria:** Acting as a user who owns a credentialed account but cannot sign in (the credential
  is unknown/lost), a person can, starting from the sign-in surface, find and follow a visible recovery
  control ("forgot access" / "recover account") into a recovery flow; on completing that flow they reach a
  signed-in state **on the original account**, verified by the handle, existing chirps, follow list, and likes
  being those of the pre-existing account and not a new or empty one. Recovery can be started using only the
  product UI in a browser, without the raw API.
- **Release gating:** true — without it, a lost credential is a permanent, unrecoverable loss of the user's
  account and content.
- **Traceability:** REQ-PROD-007 (account recovery path), REQ-PROD-004 (credentialed sign-in that makes a
  credential, and therefore its loss, possible); journey J4. Recovery-entry screen → experience design
  (`REQ-UX-*`); what recovery re-proves / the recovery mechanism → `authentication_and_sessions`
  (`REQ-ID-020..039`).
- **Resolved intra-section cross-references:** returns the owner to the account established at creation —
  **REQ-ID-005**, re-proving the ownership credential of **REQ-ID-001**.

##### REQ-ID-007 — Recovery is not an authentication bypass
- **Statement:** Completing recovery requires the person to **demonstrate control of a recovery factor that
  was established for the account before recovery is attempted**; supplying only public information (such as
  the account's handle) is never sufficient. A person who does not control the account's recovery factor
  cannot gain access, or learn whether the account exists, through the recovery flow.
- **Acceptance criteria:** Given two people — the account's owner, who controls the account's established
  recovery factor, and a second person who knows only the public handle but does not control that factor —
  (a) the second person, following the recovery flow with public information alone, **cannot reach a signed-in
  state** and cannot post, follow, like, or change settings as the account; (b) the legitimate owner,
  demonstrating control of the recovery factor, **can** complete recovery and regain access; and (c)
  initiating recovery for a handle that exists and for one that does not yields responses the initiator cannot
  tell apart, so the recovery entry point does not disclose whether a given account exists to someone who
  cannot complete recovery. (What the recovery factor is and how control of it is demonstrated are defined by
  `authentication_and_sessions`; this requirement fixes only that such control is **required** and that public
  identifiers alone are **insufficient**.)
- **Release gating:** true — a recovery path completable without a pre-established factor is a second, weaker
  way into every account and defeats REQ-PROD-004.
- **Traceability:** REQ-PROD-007, REQ-PROD-004; journey J4. What control of the recovery factor must prove,
  and the factor itself → `authentication_and_sessions` (`REQ-ID-020..039`).
- **Resolved intra-section cross-references:** the required recovery factor is the one established at creation
  — **REQ-ID-002**.

##### REQ-ID-008 — Successful recovery restores access and supersedes what was lost; a failed attempt changes nothing
- **Statement:** On successful recovery the legitimate owner regains **working authenticated access** to their
  account, and whatever recovery supersedes (the lost or compromised means of access) **no longer grants
  access**. A recovery attempt that is cancelled, fails, or expires without completion leaves the account, its
  credential, and its access **exactly as they were before** — it neither grants new access nor weakens the
  account.
- **Acceptance criteria:** (a) After a user completes recovery, they can perform an authenticated action (for
  example, post a chirp or change a profile setting) as the account — recovery ends in genuine, usable access,
  not merely a confirmation screen. (b) Any means of access that recovery supersedes no longer works
  afterward: where recovery results in a replacement credential, the superseded credential is rejected on a
  subsequent sign-in attempt, so a person who held the old (lost or compromised) credential cannot continue to
  use it. (c) After a recovery attempt that is abandoned, fails, or is left to expire, the account can still be
  used exactly as before (its existing credential still signs the owner in) and no access has been granted to
  anyone through the incomplete attempt. (The mechanics of replacing a credential and invalidating prior
  access/sessions are defined by `authentication_and_sessions`; this requirement fixes the user-observable
  outcomes.)
- **Release gating:** true — recovery that does not restore usable access is not recovery, and recovery that
  leaves a lost/compromised credential working reintroduces the bypass risk REQ-ID-007 exists to remove.
- **Traceability:** REQ-PROD-007, REQ-PROD-004; journey J4. Credential replacement and session invalidation
  mechanics → `authentication_and_sessions` (`REQ-ID-020..039`).
- **Resolved intra-section cross-references:** re-establishes the credential and recovery factor set up at
  creation — **REQ-ID-001**, **REQ-ID-002**.

##### REQ-ID-009 — Recovery is honestly observable from the user's side
- **Statement:** A user can always tell what state their recovery is in — that it has started, that it is
  pending any step they must complete, and whether it ultimately **succeeded** or **failed/expired** — through
  clear in-product feedback. Recovery never dead-ends silently and never reports success it did not achieve.
- **Acceptance criteria:** A user who initiates recovery (a) sees clear feedback that recovery has started and
  what, if anything, they must do next; (b) on completion sees an unambiguous success state that leaves them
  signed in on their account (per REQ-ID-006 / REQ-ID-008), **or** an honest failure/expiry message that
  explains the attempt did not succeed and how to try again — never a blank screen, a silent failure, or a
  success indication that is not backed by actual access. The feedback is understandable without reading an API
  error. (The wording and screens of these states are experience design, `REQ-UX-*`; this requirement fixes
  that the states exist and are honest.)
- **Release gating:** true — a release-gating recovery flow that fails or stalls silently strands the
  locked-out owner and is indistinguishable, from their side, from no recovery at all.
- **Traceability:** REQ-PROD-007, REQ-PROD-006 (no silent failure in the identity/session surface); journey
  J4. Recovery screens and state copy → experience design (`REQ-UX-*`).

---

#### Surface 3 — Account deletion: semantics and the honoured guarantee (REQ-ID-010..014)

An account is something a person must be able to **leave**, not only create and live in. This surface states
what deletion must do from the user's side — how a user starts it, what they are told before it happens, what
it removes, and the separately verifiable guarantee that, once done, the account and its personal data are
actually gone. These are requirements, not deletion mechanics: the removal cascade, how removal is made
durable across restart and backup, and the deletion screen are deferred by prose to the areas that own them.
As the product stands today there is **no account-deletion capability anywhere** (journey J7, "Leaving").

##### REQ-ID-010 — Self-service account deletion from within the product
- **Statement:** A signed-in user can initiate deletion of their **own** account from within the product's UI
  — reachable from the account-management home — without resorting to the raw API.
- **Acceptance criteria:** Starting signed in, a user can navigate to the account-management area
  (REQ-PROD-009) and find and use a clearly labelled control that begins deletion of their own account, using
  only the product UI in a browser; the control initiates deletion of the signed-in user's account and no
  other. (A person who is not signed in, or who does not hold the authority to delete a given account, cannot
  reach a control that deletes it — the authority to delete is owned by `authorization` and referenced here.)
- **Release gating:** true — a consumer account a user cannot close is an account they cannot properly own.
- **Traceability:** REQ-PROD-010 (self-service deletion), REQ-PROD-009 (account-management home); J7. The
  deletion/account-management **screen** → experience design (`REQ-UX-*`); the **authority** to delete →
  `authorization` (`REQ-ID-040..059`).
- **Resolved intra-section cross-references:** shares the account-management home with the export control —
  **REQ-ID-015** (both are "leaving" controls, REQ-PROD-009).

##### REQ-ID-011 — Confirmation of consequences before deletion
- **Statement:** Before an account is deleted, the product shows the user a clear statement of exactly what
  deletion will remove — the account itself, the user's chirps, the user's follows, and the user's likes — and
  deletion proceeds only on an explicit confirming action.
- **Acceptance criteria:** When a user initiates deletion, then *before* anything is removed they are shown a
  confirmation that names, in plain terms, what will be removed: the account, the user's chirps, the user's
  follow relationships, and the user's likes. Deletion proceeds **only** after an explicit confirming action
  distinct from the initiating action; a user who cancels (or leaves) at this step keeps their account and all
  of its data intact and remains signed in. A third party can verify both that the consequences are stated
  before removal and that nothing is removed without the explicit confirmation.
- **Release gating:** true — deletion is destructive and irreversible (REQ-ID-014); letting a user destroy
  their account without first understanding and confirming the consequences is not a responsible default.
- **Traceability:** REQ-PROD-010 (clear statement of what deletion removes); J7. The **copy and layout** of
  the confirmation → experience design (`REQ-UX-*`).

##### REQ-ID-012 — Deletion removes the account and its personal data (removal scope)
- **Statement:** A confirmed deletion removes the account and all of its personal data from the product — at
  minimum the account record and its ownership credential, the user's chirps, the user's follow relationships
  (both the accounts they followed and their presence in other accounts' follower lists), and the user's
  likes.
- **Acceptance criteria:** After a user confirms deletion of their account, then: the account's chirps no
  longer appear anywhere in the product (not on any timeline, profile, or other listing); the account no
  longer appears in any other account's follower or following list, and the accounts it had followed no longer
  count it as a follower; and likes made by the account are no longer shown or counted. Each of these is
  checkable, by a person who never met the author, against the live product after a deletion. *(This
  requirement states the user-observable scope of removal; the erasure **mechanics** — the cascade
  implementation and how references from other users' content are handled — are deferred to later code DAGs in
  the `software` domain and to domain & application design, `REQ-DOM-*`.)*
- **Release gating:** true — a deletion that leaves the user's account or content still present and reachable
  has not deleted the account.
- **Traceability:** REQ-PROD-010 (what deletion removes), REQ-PROD-012 (personal data gone); J7. Erasure
  **mechanics** → `software`-domain code DAGs / domain & application design (`REQ-DOM-*`).
- **Resolved intra-section cross-references:** removes the bound account and the credential established at
  creation — **REQ-ID-005**, **REQ-ID-001**; removes the same personal-data inventory that export takes out —
  **REQ-ID-015**.

##### REQ-ID-013 — Deletion is honoured (the separately testable after-deletion guarantee)
- **Statement:** Once an account's deletion is confirmed, the deletion is actually honoured: the account
  cannot be signed into, its profile returns not-found, the user who deleted it is signed out, and the
  account's personal data is not served again through the product.
- **Acceptance criteria:** After a confirmed deletion, a third party can independently verify **all four** of
  the following against the live product: (a) an attempt to sign in as the deleted account fails; (b)
  requesting that account's profile returns a not-found result (not a stale or empty-but-present profile); (c)
  the session of the user who performed the deletion no longer acts as that account — they are signed out and
  must sign in as some other account to continue; and (d) the deleted account's chirps, follows, and likes are
  not returned through any product surface. Each check is a yes/no an evaluator can reach without having met
  the author. *(This is the honoured guarantee as the live product exposes it. That the removal is also
  **durable** — that it survives a service restart and is not re-served from a backup or replica — is an
  operational property owned by operational readiness, `REQ-OPS-*`, and is referenced here, not specified in
  this section.)*
- **Release gating:** true — "deletion is honoured" is the trust the whole leaving journey rests on.
- **Traceability:** REQ-PROD-012 (deletion is honoured), REQ-PROD-010; J7. **Durable** removal across restart
  / backup → operational readiness (`REQ-OPS-*`). Satisfies the honoured-guarantee bar — a separately testable
  guarantee distinct from the removal scope in REQ-ID-012.

##### REQ-ID-014 — Deletion is disclosed as permanent, and its completion is confirmed
- **Statement:** The product tells the user, before they confirm, that deleting the account is permanent and
  cannot be undone, and confirms to the user once the deletion has completed.
- **Acceptance criteria:** The pre-deletion confirmation (REQ-ID-011) states plainly that deletion is
  permanent and irreversible — that the account and its data cannot be recovered afterward — so the user is
  not led to believe the account can be restored; and after the user confirms, the product gives an
  unambiguous indication that the account has been deleted, rather than leaving the user on a screen that
  still appears signed in or in an uncertain state. A third party can verify both that the permanence is
  disclosed before confirmation and that a completion indication is shown afterward.
- **Release gating:** true — for an irreversible destructive action, permanence disclosure and honest
  completion feedback are part of not misleading the user.
- **Traceability:** REQ-PROD-010 (understand what deletion does); J7. Builds on the honoured guarantee in
  REQ-ID-013 (there is nothing to recover because deletion is honoured). The confirmation **screen and copy**
  → experience design (`REQ-UX-*`).

---

#### Surface 4 — Data export (REQ-ID-015..017)

"I can take my data with me when I leave" is one of the defining promises a consumer web application makes to
the people who trust it with their content. This surface designs the data-export surface of the account
lifecycle: a signed-in user can request and obtain **their own data** — their profile, their chirps, their
follow list, and their likes — in a usable, portable, documented, machine-readable form they can download and
open outside chirp. It states what a user can take out and in what shape, as requirements from the user's
side — not how the export is serialized, scheduled, or delivered. In the product as it stands, **no
data-export capability exists**. The export format internals are deferred to code DAGs (`software` domain) and
domain design (`REQ-DOM-*`); the export control/screen to experience design (`REQ-UX-*`); and the *data-right*
this capability satisfies is owned by `abuse_and_privacy_basics` (referenced, not decided here).

##### REQ-ID-015 — A user can export their own data, and it is complete
- **Statement:** A signed-in user can request an export of their own account data and obtain it, and the
  export includes the personal data chirp holds about their account — at minimum their profile (handle,
  display name, bio), their chirps, their follow list, and their likes. The control to request the export is
  reachable from the account-management home without using the raw API.
- **Acceptance criteria:**
  1. A signed-in user can initiate a data export from the account-management area (the home of REQ-PROD-009)
     using only the product UI in a browser, and obtain the resulting export.
  2. The obtained export contains the requesting user's **profile** (at least handle, display name, and bio),
     **every chirp they authored** that the product still holds, **the list of accounts they follow**, and
     **the chirps they have liked**. A reviewer can confirm each of these four categories is present by
     comparing the export against what that account shows in the product.
  3. If any category is deliberately omitted or limited, the export's accompanying documentation states that
     clearly (so "complete" is testable against a stated scope rather than an unstated one).
- **Release gating:** true
- **Traceability:** REQ-PROD-011 (export your own data), REQ-PROD-009 (account-management home); journey J7.
  Export control & screen → experience design (`REQ-UX-*`).
- **Resolved intra-section cross-references:** the exported inventory is the same inventory deletion removes —
  **REQ-ID-012**; shares the account-management home with the deletion control — **REQ-ID-010** (both are
  "leaving" controls, REQ-PROD-009).

##### REQ-ID-016 — The export is portable, documented, and machine-readable
- **Statement:** The exported data is delivered in a portable, documented, machine-readable form that the user
  can download and open outside chirp, without needing access to chirp to make sense of it.
- **Acceptance criteria:**
  1. The user can **download** the export as a self-contained artifact (not merely view it inside chirp).
  2. The downloaded artifact is **machine-readable**: it is in a structured, parseable format (not an image,
     screenshot, or opaque blob), such that a program other than chirp can read the user's chirps, profile,
     follows, and likes out of it.
  3. The export is **documented**: it is accompanied by a description of what it contains and what its fields
     mean, sufficient for a competent person who does not know chirp's internals to interpret it.
  4. A reviewer can open the export on a machine that has no access to chirp and recover the user's data from
     it using only the export and its documentation.
- **Release gating:** true
- **Traceability:** REQ-PROD-011 (usable, portable, documented, machine-readable, openable outside chirp);
  journey J7. Export **format internals / schema / serialization** → code DAGs (`software` domain) and domain
  & application design (`REQ-DOM-*`) — this requirement fixes the properties the result must have, not the
  encoding.

##### REQ-ID-017 — An export returns only the requesting user's own data
- **Statement:** A data export returns only the data of the authenticated user who requested it; the export
  capability cannot be used to obtain another user's personal data, and requires the requester to be
  authenticated as the account being exported.
- **Acceptance criteria:**
  1. An export requested by a signed-in user contains **that user's own** profile, chirps, follows, and likes,
     and does **not** contain another user's private data (another account's credential, settings, or any data
     that account's owner has not made public).
  2. A person who is not signed in, or who is signed in as a different account, cannot obtain account X's
     export; only the authenticated owner of account X can export account X's data.
  3. Where the export necessarily references other accounts to be meaningful (for example, the identifier or
     handle of an account the user follows), it includes only what is already visible to the requesting user
     about those accounts, and no more.
- **Release gating:** true
- **Traceability:** REQ-PROD-011 (export of **their own** data); journey J7. The authority of who-may-export
  (ordinary owner here; any operator/admin-on-behalf authority) → `authorization` (`REQ-ID-040..059`),
  referenced in prose; the data-right this mechanism satisfies → `abuse_and_privacy_basics`
  (`REQ-ID-060..079`), carried forward for the identity & trust synthesis.

---

#### Intra-section coupling cross-references (resolved)

The four surfaces above are parts of one account lifecycle and share a real seam. The couplings between them
are resolved here into explicit `REQ-ID ↔ REQ-ID` cross-references (also carried inline on each requirement
above):

1. **Recovery re-establishes what creation set up.** Recovery must re-prove the same ownership, and return the
   owner to the same account, that creation established.
   → **REQ-ID-006 ↔ REQ-ID-005** (same account), **REQ-ID-006 ↔ REQ-ID-001** and **REQ-ID-008 ↔ REQ-ID-001 /
   REQ-ID-002** (same credential and recovery factor), **REQ-ID-007 ↔ REQ-ID-002** (the required factor is the
   one established at creation).
2. **Deletion removes exactly what creation established.** What deletion honour-ably removes is the bound
   account and its ownership credential.
   → **REQ-ID-012 ↔ REQ-ID-005** and **REQ-ID-012 ↔ REQ-ID-001**.
3. **Export and deletion describe one personal-data inventory.** Export takes out the same inventory (profile,
   chirps, follows, likes) that deletion removes; the two must not drift apart.
   → **REQ-ID-015 ↔ REQ-ID-012**.
4. **Deletion and export are the two "leaving" controls in one account-management home** (REQ-PROD-009).
   → **REQ-ID-010 ↔ REQ-ID-015**.

#### Cross-section couplings (carried forward — resolved by the identity & trust synthesis, not here)

These couplings join this section to sibling identity & trust sections. They are stated here in prose, naming
the owning section; this section does **not** resolve them — the identity & trust synthesis resolves each into
an explicit `REQ-ID ↔ REQ-ID` cross-reference once all sections are assembled.

- **Recovery must not become an authentication bypass.** REQ-ID-007 (and the access-supersession part of
  REQ-ID-008) couples to `authentication_and_sessions` (`REQ-ID-020..039`), which defines what recovery must
  re-prove, what "the credential" is, and the session/credential-invalidation mechanics recovery relies on.
  This section fixes the safety property and defers the mechanism.
- **Account deletion: lifecycle semantics vs. who-may-delete authority.** REQ-ID-010 and REQ-ID-012 depend on
  the authority that only the account's owner (and an authorized administrator) may delete an account, owned by
  `authorization` (`REQ-ID-040..059`). This section names the dependency; the authority model owns the roles
  and the per-action check.
- **Data export / erasure: mechanism vs. the data-right it satisfies.** REQ-ID-015 and REQ-ID-017 (export),
  together with the erasure in REQ-ID-012 (deletion), are the *mechanism* by which a user exercises a **data
  right** owned by `abuse_and_privacy_basics` (`REQ-ID-060..079`). The requirement here and the right there
  describe the same promise from two sides and must be reconciled by the identity & trust synthesis.

---

#### Consolidated requirement register — accounts_and_lifecycle (REQ-ID-001..017)

| REQ-ID | Surface | Release-gating | Traces to REQ-PROD | Journey |
|--------|---------|----------------|--------------------|---------|
| REQ-ID-001 | Creation & credential | true | REQ-PROD-001, REQ-PROD-004 | J2, J3 |
| REQ-ID-002 | Creation & credential | true | REQ-PROD-004, REQ-PROD-001 | J2 → J4 |
| REQ-ID-003 | Creation & credential | true | REQ-PROD-002 | J2 |
| REQ-ID-004 | Creation & credential | true | REQ-PROD-003 | J2 |
| REQ-ID-005 | Creation & credential | true | REQ-PROD-001, REQ-PROD-004 | J2, J3, J7 |
| REQ-ID-006 | Credential recovery | true | REQ-PROD-007, REQ-PROD-004 | J4 |
| REQ-ID-007 | Credential recovery | true | REQ-PROD-007, REQ-PROD-004 | J4 |
| REQ-ID-008 | Credential recovery | true | REQ-PROD-007, REQ-PROD-004 | J4 |
| REQ-ID-009 | Credential recovery | true | REQ-PROD-007, REQ-PROD-006 | J4 |
| REQ-ID-010 | Account deletion | true | REQ-PROD-010, REQ-PROD-009 | J7 |
| REQ-ID-011 | Account deletion | true | REQ-PROD-010 | J7 |
| REQ-ID-012 | Account deletion | true | REQ-PROD-010, REQ-PROD-012 | J7 |
| REQ-ID-013 | Account deletion | true | REQ-PROD-012, REQ-PROD-010 | J7 |
| REQ-ID-014 | Account deletion | true | REQ-PROD-010 | J7 |
| REQ-ID-015 | Data export | true | REQ-PROD-011, REQ-PROD-009 | J7 |
| REQ-ID-016 | Data export | true | REQ-PROD-011 | J7 |
| REQ-ID-017 | Data export | true | REQ-PROD-011 | J7 |

*All seventeen requirements in this section are release-gating. REQ-ID-018 and REQ-ID-019 are reserved for
this section and are unused: assembly surfaced no net-new cross-facet requirement that no surface owns — the
one-account invariant is owned by REQ-ID-005 and the one-inventory invariant by the REQ-ID-015 ↔ REQ-ID-012
cross-reference, so no new requirement is warranted.*

---

## authentication_and_sessions

<!-- Section: authentication_and_sessions of design/identity-security.md. Real authentication (closing the handle-only gap) and sessions safe to expose on the public internet, plus the MFA/SSO scoping decision. Requirements only (REQ-ID-020..039). -->

### authentication_and_sessions

This section defines what **"real authentication, not a stub"** must mean for chirp, and what makes a
**session safe to expose on the public internet**. It is the identity bar for the whole identity-and-trust
subtree: a releasable chirp proves who a signer is before letting them act, keeps that proof alive safely
while they use the product, and ends it cleanly and visibly when they leave.

Everything here is a **requirement** — what must be true from the user's and the attacker's side — not a
design for how it is built. No authentication protocol, credential format, cryptographic primitive, or
session-store technology is named; those "how" decisions belong to later code work in the `software` domain
and are reached by prose below. Requirements carry `REQ-ID-020 .. REQ-ID-039`, owned by this spoke.

#### What the product does today, and the gap this closes

Grounded in `Nousix-LLC/nswe-demo` at commit `c805525`, described from the user's and attacker's side:

- **Sign-in is handle-only.** `POST /api/sessions` authenticates by presenting a handle and nothing else
  (`crates/chirp-server/src/api/auth.rs`, `crates/chirp-server/src/api/handlers.rs`): an unknown handle is
  rejected, but any known handle is accepted with **no secret of any kind**. Because every handle is already
  public (any profile is viewable), the product today *cannot tell the real owner of a handle from anyone
  who types it*. This is the central gap `REQ-ID-020..025` close.
- **Sessions are a bearer credential resolved from memory.** A successful sign-in returns a bearer session
  credential that later requests present; the server resolves it to an account through an in-memory store.
  There is **no expiry, no server-side invalidation, and no sign-out route** (the session route exposes only
  creation). When the service restarts, every session is silently dropped and every user is signed out with
  no notice. `REQ-ID-026..036` make these sessions safe to expose publicly.
- **There is no recovery, no multi-factor option, and no federated sign-in.** `REQ-ID-037` states what
  recovery must re-prove (coupled to the account lifecycle), and `REQ-ID-038..039` record the scoping
  decision on MFA and SSO.

#### Scope decision — SSO and MFA (recorded with a reason)

chirp is a **single, shared consumer product** with no organizations, teams, or tenants (see the product
requirements' out-of-scope on multi-tenancy). Against that product kind:

- **MFA (a second authentication factor) is deferred** from the first release bar, not excluded — see
  `REQ-ID-038`. A single-factor credential that meets `REQ-ID-020..024` is an adequate first bar for a
  consumer microblog; MFA is the first strengthening to add. The requirement set MUST NOT be written in a
  way that precludes adding a second factor later.
- **SSO / federated sign-in is out of scope** for this release — see `REQ-ID-039`. SSO earns its place when
  there are organizations whose identity provider should govern sign-in; chirp has none. The condition that
  would bring it back is the same condition that would reintroduce tenancy.

#### Couplings to resolve in synthesis

Two couplings from the subtree coupling register are owned jointly by this spoke; stated here in prose for
the section-assembly step to turn into explicit `REQ-ID ↔ REQ-ID` cross-references:

1. **Recovery must not become an authentication bypass.** The recovery *lifecycle* is owned by
   the accounts & lifecycle section (`REQ-ID-001..019`). This spoke owns *what recovery must re-prove and must
   invalidate* — `REQ-ID-037` (with `REQ-ID-024` on never revealing a credential and `REQ-ID-032` on
   invalidating existing sessions). Synthesis links `REQ-ID-037` to the accounts-lifecycle recovery
   requirement.
2. **Public-internet session safety depends on an authorization check on every mutating action.** This spoke
   owns *that a valid session is required and what its scope is* — `REQ-ID-034`. The per-action *authority*
   model (who may act on what) is owned by the authorization section (`REQ-ID-040..059`). Synthesis links `REQ-ID-034`
   to the authz per-action-authorization requirement.

A third, lighter coupling: credential-guessing resistance (`REQ-ID-023`) relies on the baseline rate/volume
limits owned by the abuse & privacy section (`REQ-ID-060..079`); and step-up re-authentication for destructive
actions (`REQ-ID-025`) references account deletion, owned by the accounts & lifecycle section and authorized by
the authorization section. These are noted for synthesis to cross-reference.

---

#### Requirements — real authentication

##### REQ-ID-020 — Signing in requires proof of a secret only the owner holds
- **Statement:** Authentication MUST require the signer to prove possession of a secret credential bound to
  the account. Presenting a public identifier (a handle) MUST NOT by itself authenticate anyone or authorize
  any action.
- **Acceptance criteria:** A person who knows another account's handle but does not hold its credential
  cannot obtain an authenticated session for that account and cannot post, follow, like, edit the profile, or
  change settings as it. The legitimate owner, presenting the correct credential together with the handle,
  can. A sign-in attempt that supplies only a handle (no credential) is rejected.
- **Release gating:** true
- **Traces to:** REQ-PROD-004.

##### REQ-ID-021 — A credential is verified against the account on every sign-in
- **Statement:** Each sign-in MUST verify the presented credential against the one bound to the named account
  and succeed only on a match; the account's credential is established when the account is created (that
  establishment is owned by the account lifecycle).
- **Acceptance criteria:** Signing in with the correct handle and correct credential succeeds; signing in
  with a correct handle and an incorrect credential fails; the credential is checked every time, not only on
  the first sign-in of a session. There exists no account that can be signed into with no credential.
- **Release gating:** true
- **Traces to:** REQ-PROD-004. (Credential *establishment* at sign-up → the accounts & lifecycle section.)

##### REQ-ID-022 — Authentication failure is uniform and non-informative
- **Statement:** A failed authentication MUST report only that authentication failed. It MUST NOT disclose
  which element was wrong, MUST NOT confirm that a guessed credential was "close," and MUST NOT behave in a
  way (including a timing difference) that lets an attacker distinguish a wrong credential for a real account
  from one for a non-existent account.
- **Acceptance criteria:** Across (a) a real handle with a wrong credential and (b) a handle no account
  holds, the product's failure response is indistinguishable in message and observable behavior. No failure
  message states whether the handle exists or whether the credential was partly correct.
- **Release gating:** true
- **Traces to:** REQ-PROD-004.

##### REQ-ID-023 — Credentials resist guessing, and online guessing is throttled
- **Statement:** The product MUST require a credential strong enough to resist casual guessing and MUST limit
  the rate of online authentication attempts so that automated guessing against an account is impractical.
- **Acceptance criteria:** A credential that is trivially weak by a stated, documented rule is rejected at
  the time it is set. Repeated failed sign-in attempts against an account are slowed or blocked after a
  bounded number of tries within a window, and this is observable (a rapid sequence of wrong-credential
  attempts stops being accepted). The legitimate owner retains a path back in after a lockout.
- **Release gating:** true
- **Traces to:** REQ-PROD-004. (The baseline rate/volume-limit mechanism is coupled to
  the abuse & privacy section, `REQ-ID-060..079`.)

##### REQ-ID-024 — A credential is never revealed, displayed, or logged
- **Statement:** The product MUST never display, return, or record a user's credential in a recoverable form.
  Losing a credential MUST be resolved by re-establishing a new one, never by the product revealing the old.
- **Acceptance criteria:** No product screen, API response, export, notification, operator/admin view, or log
  shows a user's credential. A recovery path (REQ-ID-037) lets a user set a new credential; at no point does
  the product tell anyone the existing one.
- **Release gating:** true
- **Traces to:** REQ-PROD-004. (Recovery lifecycle → the accounts & lifecycle section; see REQ-ID-037.)

##### REQ-ID-025 — Destructive or identity-changing actions require fresh proof
- **Statement:** Actions that change the account's identity or are irreversible — changing the credential or
  the recovery factor, and deleting the account — MUST require the signer to re-prove the credential in the
  current session, not rely on the session alone.
- **Acceptance criteria:** Attempting to change the credential/recovery factor or to delete the account, from
  an already-signed-in session, prompts for the credential again and proceeds only on success; an attacker
  who obtains an unlocked signed-in session but not the credential cannot complete these actions.
- **Release gating:** true
- **Traces to:** REQ-PROD-004. (Account deletion is owned by the accounts & lifecycle section; the authority to
  delete by the authorization section — synthesis cross-references.)

---

#### Requirements — sessions safe on the public internet

##### REQ-ID-026 — Authentication establishes a session; the credential is not re-sent per action
- **Statement:** A successful authentication MUST establish a session that authorizes the account's
  subsequent actions for its lifetime, so the credential itself is presented only at sign-in (and at the
  step-up points of REQ-ID-025), not on every request.
- **Acceptance criteria:** After a successful sign-in, a user can perform authenticated actions without
  re-entering their credential until the session ends; the credential is not required on each ordinary action.
- **Release gating:** true
- **Traces to:** REQ-PROD-004, REQ-PROD-006.

##### REQ-ID-027 — The session secret is confidential, unguessable, and protected in transit
- **Statement:** The session secret MUST be infeasible to guess and MUST remain confidential; all exchange of
  the credential and the session secret MUST occur over a confidential, integrity-protected channel so it
  cannot be read or altered in transit by a network observer.
- **Acceptance criteria:** Session secrets are not predictable from other sessions or from account data. The
  product refuses to transmit the credential or session secret over an unprotected channel. Capturing a
  valid session secret is the only practical way to act as the account without the credential (there is no
  weaker path).
- **Release gating:** true
- **Traces to:** REQ-PROD-006.

##### REQ-ID-028 — Sessions have a bounded lifetime
- **Statement:** A session MUST expire — both after a bounded period of inactivity and after a bounded
  absolute age — so a session cannot remain valid indefinitely.
- **Acceptance criteria:** A session left idle beyond the inactivity bound no longer authorizes actions; a
  session older than the absolute bound no longer authorizes actions even if used continuously. In both cases
  the next action triggers the explicit re-authenticate prompt of REQ-ID-029 rather than silently working.
- **Release gating:** true
- **Traces to:** REQ-PROD-006.

##### REQ-ID-029 — No silent sign-out: expiry and invalidation are always surfaced
- **Statement:** When a session has expired or been invalidated, the user's next action MUST surface a clear
  prompt to sign in again. The product MUST NOT fail silently and MUST NOT appear to succeed when the session
  is no longer valid.
- **Acceptance criteria:** With an expired or invalidated session, the next action a user attempts produces a
  visible "your session has ended, please sign in again" outcome — not a silent no-op, not a generic
  unexplained error, and not a false success. The user is never left believing an action happened when it did
  not because their session had ended.
- **Release gating:** true
- **Traces to:** REQ-PROD-006.

##### REQ-ID-030 — Signed-in versus logged-out state is always shown truthfully
- **Statement:** The product MUST, at all times, show the user whether they are currently signed in and as
  which account, and this indication MUST reflect the real session state.
- **Acceptance criteria:** A signed-in user always sees that they are signed in (and as whom); a logged-out
  visitor always sees that they are logged out. The indicator never shows "signed in" for a session that is
  no longer valid.
- **Release gating:** true
- **Traces to:** REQ-PROD-006.

##### REQ-ID-031 — Deliberate sign-out ends the session at the server
- **Statement:** A signed-in user MUST be able to deliberately end their session, and doing so MUST
  invalidate that session at the server — not merely discard it on the client. After sign-out the prior
  session MUST NOT authorize any further action.
- **Acceptance criteria:** After a user signs out, re-presenting the just-ended session secret is rejected;
  on a shared device, a second person cannot act as the first user's account using the prior session. The
  user must authenticate again before acting.
- **Release gating:** true
- **Traces to:** REQ-PROD-005.

##### REQ-ID-032 — Credential change and recovery invalidate all existing sessions
- **Statement:** When an account's credential is changed or recovered, all of that account's existing
  sessions (other than the one performing the change, at most) MUST be invalidated, so that a session an
  attacker may hold does not survive the owner regaining control.
- **Acceptance criteria:** After the owner changes or recovers their credential, any session established
  before that moment no longer authorizes actions and triggers the re-authenticate prompt of REQ-ID-029. A
  pre-existing session held by someone else cannot continue to act on the account.
- **Release gating:** true
- **Traces to:** REQ-PROD-005. (Recovery lifecycle → the accounts & lifecycle section, enforcing REQ-PROD-007;
  see REQ-ID-037.)

##### REQ-ID-033 — The session issued at authentication resists fixation
- **Statement:** The session a signer ends up with MUST be one that could only have been established by that
  authentication — an attacker MUST NOT be able to pre-plant or fix a session value that becomes
  authenticated when the victim signs in.
- **Acceptance criteria:** A session secret observed or supplied before authentication does not become a
  valid authenticated session for the account after the owner signs in; authentication yields a fresh
  authenticated session that a prior observer does not hold.
- **Release gating:** true
- **Traces to:** REQ-PROD-004, REQ-PROD-006.

##### REQ-ID-034 — A session acts only as its account, and every account action requires one
- **Statement:** A session MUST authorize actions only as the account it was issued to and only for that
  account's own resources; every action that creates, changes, or deletes data, or that acts as the account,
  MUST require a valid session. Reading public content (the public timeline and public profiles) MUST NOT
  require a session.
- **Acceptance criteria:** Without a valid session, no post/follow/like/profile-edit/settings/delete action
  succeeds; with a valid session, those actions succeed only as the signed-in account and only on its own
  resources. A logged-out visitor can still read the public timeline and open public profiles.
- **Release gating:** true
- **Traces to:** REQ-PROD-004, REQ-PROD-006. (Also supports REQ-PROD-013 "reading never requires an
  account." The per-action *authority* model is owned by the authorization section, `REQ-ID-040..059` — synthesis
  cross-references this requirement to it.)

##### REQ-ID-035 — Sessions survive normal operation, or end visibly — never silently
- **Statement:** Routine operation of the service — restart, redeploy, or scaling — MUST NOT silently sign
  users out. A user's session MUST either continue to work across such events or, if it genuinely cannot be
  honored, present as the explicit re-authenticate prompt of REQ-ID-029 rather than a silent, unexplained
  loss of sign-in.
- **Acceptance criteria:** After a routine restart or redeploy, a user with a still-valid session either
  remains signed in and able to act, or is clearly told their session has ended and prompted to sign in
  again. No user is silently and unexplainedly logged out by ordinary operations. (Today, in-memory sessions
  vanish on restart with no notice; this requirement removes that silent failure.)
- **Release gating:** true
- **Traces to:** REQ-PROD-006. (Durable session continuity depends on durable data, owned by
  the operational-readiness design / domain design, enforcing REQ-PROD-026 — synthesis cross-references.)

##### REQ-ID-036 — A user can see and end their own active sessions
- **Statement:** A signed-in user SHOULD be able to see that their account has one or more active sessions and
  end sessions other than the current one, so that a user who suspects their account is being used elsewhere
  can cut off access without changing their credential.
- **Acceptance criteria:** From the signed-in account area, a user can view that active sessions exist and
  invoke an action that ends other sessions; after doing so, those other sessions no longer authorize actions
  (per REQ-ID-031). The current session remains valid.
- **Release gating:** false — a real improvement to account safety that a first releasable version can ship
  without, since credential change (REQ-ID-032) already gives a user a way to cut off all other access. It
  is the first self-service hijack-response control to add.
- **Traces to:** REQ-PROD-005, REQ-PROD-006.

---

#### Requirements — recovery coupling and scope decisions

##### REQ-ID-037 — Recovery re-proves control and is not an authentication bypass
- **Statement:** Any account-recovery path MUST re-prove the recoverer's control of the account through a
  factor the account established in advance, MUST result in a newly set credential (never the disclosure of
  the old one, per REQ-ID-024), and MUST invalidate pre-existing sessions (per REQ-ID-032). Recovery MUST NOT
  provide an easier route to act as an account than normal authentication does.
- **Acceptance criteria:** A user who controls the account's pre-established recovery factor can complete
  recovery and regain access by setting a new credential; a person who does not control that factor cannot
  gain access through recovery. Completing recovery invalidates sessions that existed before it. There is no
  recovery step that signs the user in without re-proving control.
- **Release gating:** true — once real authentication exists (REQ-ID-020), a user who loses their credential
  must have a safe way back, and that way must not become the weakest link.
- **Traces to:** REQ-PROD-007, REQ-PROD-004. (The recovery *lifecycle/flow* is owned by
  the accounts & lifecycle section, `REQ-ID-001..019`; this requirement is the authentication constraint on it —
  synthesis cross-references the two.)

##### REQ-ID-038 — Multi-factor authentication: deferred, not precluded
- **Statement:** A second authentication factor (MFA) is **not required** for the first release bar of this
  single, shared consumer product; single-factor authentication meeting REQ-ID-020..024 is the release bar.
  The requirement set MUST NOT be written so as to preclude adding an optional second factor later.
- **Acceptance criteria:** The release bar is met without MFA, provided REQ-ID-020..024 hold. The decision and
  its reason are recorded (this requirement). Nothing in the authentication requirements forbids a future
  optional second factor. **Revisit condition:** add MFA (at least optionally) if the product begins holding
  higher-sensitivity data, sees credential-stuffing abuse in practice, or adds the organization/tenant
  accounts that are currently out of scope.
- **Release gating:** false — a recorded, deliberate scope decision for this product kind, not a shipped
  capability.
- **Traces to:** REQ-PROD-004.

##### REQ-ID-039 — Single sign-on / federated identity: out of scope
- **Statement:** Federated sign-in through an external identity provider (SSO) is **out of scope** for this
  release. chirp is a single shared consumer product with no organizations whose identity provider would
  govern sign-in; each account authenticates with its own credential (REQ-ID-020).
- **Acceptance criteria:** The release bar is met with first-party credential authentication only; no SSO
  capability is required. The decision and its reason are recorded (this requirement). **Revisit condition:**
  reconsider SSO if and when chirp introduces organizations, teams, or tenants (the same condition that would
  reintroduce the multi-tenant product kind the product requirements place out of scope).
- **Release gating:** false — a recorded, deliberate scope decision tied to the product kind.
- **Traces to:** REQ-PROD-004.

---

#### Section coverage note

`REQ-ID-020..025` specify real authentication (closing the handle-only gap: proof of a held secret, uniform
failure, guessing resistance, credential confidentiality, and fresh proof for destructive actions).
`REQ-ID-026..036` make sessions safe to expose on the public internet (session issuance; confidential,
bounded, fixation-resistant sessions; no silent sign-out; truthful state; deliberate and global
invalidation; scoped authority; continuity across operations; and owner-visible session control).
`REQ-ID-037` ties recovery to authentication without creating a bypass. `REQ-ID-038..039` record the MFA and
SSO scoping decisions with reasons and revisit conditions. Every requirement states *what must be true* and
defers *how* to later `software`-domain code work. Couplings to the accounts & lifecycle section, the authorization section,
and the abuse & privacy section are called out above for the section-assembly step to resolve into explicit `REQ-ID`
cross-references.

---

## authorization

<!-- Section: authorization of design/identity-security.md. The authority model: ownership-based user access and moderator/operator/administrator authority, with the per-action server-side authorization check. Requirements only (REQ-ID-040..057). -->

### authorization

*(section: `authorization` of `design/identity-security.md`)*

#### What this section settles, and what it defers

This section defines chirp's **authority model**: who each actor is, and **what each actor may see and
change**. It states the authorization requirements as testable `REQ-ID-*` requirements — *what must be
true* from the acting user's or operator's side — and deliberately **defers the "how"** (policy-check
placement, middleware, how roles are stored and assigned, how a decision is computed) to later
implementation work in the `software` domain.

**Grounding.** Today chirp has **no authorization model at all**: identity is handle-only (anyone who
types a handle becomes that account — see `authentication_and_sessions`), there are **no roles**, and
there is **no moderation or admin surface** (grounded in the product as it stands at `c805525`). This
section defines the authority model the product does not yet have. It is the **authority** companion to
authentication: authentication establishes *who you are* (owned by `authentication_and_sessions`); this
section establishes *what you are allowed to do once you are known*.

Authorization is distinct from capability. Where the user-facing *ability to report, block, or mute* and
the *visible effect* of moderation live in `abuse_and_privacy_basics`, this section owns only the
**authority** to act — who is permitted to perform an action — which those capabilities and the account
lifecycle depend on.

---

#### A. Principals and authorities (the role model)

chirp recognises four kinds of principal. Authority is **additive and least-privilege**: every principal
starts with the authority of the tier below it and gains only what is explicitly named. Having an account
grants **ownership** authority only; elevated authority is never implied by signing in.

| Principal | How recognised | May do | May **not** do |
|---|---|---|---|
| **Anonymous visitor** | No session | Read public content (public timeline, public profiles, public chirps) | Any state-changing (mutating) action |
| **Account holder (user)** | An authenticated session proving ownership of the account | Everything an anonymous visitor may do, plus create/edit/delete **their own** chirps, edit **their own** profile, manage **their own** follows and likes, delete **their own** account, and export **their own** data | Act on another account's content, account, follows, or likes; exercise any elevated authority |
| **Moderator** (trust & safety) | An explicitly assigned elevated authority | Everything a user may do with their own account, plus act on **abusive** content and accounts across the product (remove/hide a chirp; suspend, restore, or remove an account) | Impersonate a user; read personal data beyond what the moderation action requires; read or recover credentials |
| **Operator / Administrator** | An explicitly assigned elevated authority | Everything a moderator may do, plus **look up** an account and see the support-relevant information needed to answer a support request, and perform account-administration actions through a provided operator surface | Impersonate a user; read personal data beyond what the support action requires; read or recover credentials; manipulate storage directly as a substitute for the provided surface |

**Moderator** and **operator/administrator** are **distinct authorities** even though, in a small
single-operator deployment, one person may hold both. Keeping them distinct lets the authority to act on
abuse be reasoned about (and later granted) separately from the authority to see a user's account for
support. These roles are defined for a **single shared consumer product**; organisation/team/tenant
roles are explicitly out of scope (§F).

The authority to act is the only thing settled here. The *mechanism* by which a decision is enforced on
every request — and the *way* roles are assigned — is a `software`-domain concern.

---

#### B. Ownership-based user access

##### REQ-ID-040 — A user may act only on resources they own
- **Statement:** A signed-in user may create, edit, or delete only resources that belong to their own account — their own chirps, their own profile, and their own account — and may not create, edit, or delete another account's content, profile, or account.
- **Acceptance criteria:** With real authentication in place, a person signed in as user A cannot, through the web UI **or** by calling the API directly, edit or delete a chirp authored by user B, edit B's profile, or delete B's account; every such attempt is refused and changes nothing. The same person can edit and delete their own chirps and edit their own profile.
- **Release gating:** true
- **Traceability:** REQ-PROD-004, REQ-PROD-010; persona P3; journeys J3, J5, J7. Enforcement mechanism → `software` domain.

##### REQ-ID-041 — Every action is attributed to, and authorized against, the actor's own identity (no acting-as-another)
- **Statement:** Every state-changing action a user takes is attributed to, and authorized against, the identity of the account whose session performed it; a user cannot post, follow, like, edit, or delete in a way that is attributed to or performed as another account.
- **Acceptance criteria:** A chirp, follow, or like created while signed in as account A is attributed to A and to no other account; there is no input, parameter, or UI path by which a person signed in as A can cause an action to be recorded as performed by account B. Knowing another user's public handle does not let a person act as that account.
- **Release gating:** true
- **Traceability:** REQ-PROD-004; persona P3; journey J3. (Closes the authority side of the handle-only gap; the proof-of-ownership mechanism is owned by `authentication_and_sessions`.)

##### REQ-ID-042 — A user manages only their own social graph and likes
- **Statement:** Following/unfollowing and liking/unliking are authorized as changes to the acting user's own relationships; a user cannot create or remove follows or likes that belong to another account.
- **Acceptance criteria:** A person signed in as A can follow/unfollow accounts and like/unlike chirps as A, and these change only A's own following set and A's own likes; no UI or direct API call lets A add or remove a follow or like on behalf of account B.
- **Release gating:** true
- **Traceability:** REQ-PROD-004; persona P3; journey J5. Enforcement mechanism → `software` domain.

##### REQ-ID-043 — Only the account owner (or an authorized administrator) may delete an account
- **Statement:** The authority to delete an account rests with the account's owner, and with an authorized operator/administrator acting through the provided surface; no other principal may delete an account.
- **Acceptance criteria:** A signed-in user can initiate deletion of **their own** account; a user cannot initiate deletion of another account; an unauthenticated request cannot delete any account. An authorized administrator can delete an account through the operator surface. Each unauthorized attempt is refused and removes nothing. (What deletion removes and the guarantee it is honoured are owned by the accounts & lifecycle requirements.)
- **Release gating:** true
- **Traceability:** REQ-PROD-010, REQ-PROD-012; persona P3; journey J7. Deletion semantics & guarantee → `accounts_and_lifecycle`; enforcement → `software` domain.

##### REQ-ID-044 — Only the account owner (or an authorized administrator on the owner's behalf) may export an account's data
- **Statement:** The authority to export an account's data rests with the account's owner, and with an authorized administrator acting on the owner's behalf through the provided surface; one user cannot export another user's data.
- **Acceptance criteria:** A signed-in user can request an export of **their own** data and obtains only their own account's data; a user cannot obtain another account's export; an unauthenticated request obtains no export. Each unauthorized attempt is refused and discloses nothing.
- **Release gating:** true
- **Traceability:** REQ-PROD-011; persona P3; journey J7. Export mechanism/shape → `accounts_and_lifecycle`; the data-right it satisfies → `abuse_and_privacy_basics`; enforcement → `software` domain.

##### REQ-ID-045 — Holding an account grants ownership-scoped authority only (no elevated authority by default)
- **Statement:** Creating or signing into an ordinary account grants ownership-scoped authority only; it never grants moderator or operator/administrator authority. Elevated authority exists only where it has been explicitly assigned.
- **Acceptance criteria:** A freshly created, ordinary signed-in account cannot remove another user's chirp, suspend another account, or look up another account through an operator surface; each such attempt is refused. Elevated actions succeed only for an account that has been explicitly granted the corresponding elevated authority.
- **Release gating:** true
- **Traceability:** REQ-PROD-004, REQ-PROD-033; security baseline for a public product. Role assignment mechanism → `software` domain.

---

#### C. Elevated authority — moderator, operator / administrator

##### REQ-ID-046 — Named elevated authorities are defined
- **Statement:** The product defines two named elevated authorities — **moderator** (act on abusive content and accounts) and **operator/administrator** (support lookup and account administration) — each with an explicitly stated set of permitted actions. The two are distinct authorities, though one principal may hold both.
- **Acceptance criteria:** The design and the running product distinguish the moderator authority from the operator/administrator authority; it is possible to describe, for any elevated action, which named authority permits it; a principal can hold one, the other, or both.
- **Release gating:** true
- **Traceability:** REQ-PROD-028, REQ-PROD-033; persona P4 and the deliberately-absent moderator persona; journey J8.

##### REQ-ID-047 — Moderator authority over abusive content and accounts
- **Statement:** A principal holding moderator authority may act on abusive content and accounts across the product — remove or hide a chirp, and suspend, restore, or remove an account — and the effect of the action is visible to users.
- **Acceptance criteria:** An account **without** moderator authority cannot remove another user's chirp or suspend another account (the action is refused); an account **with** moderator authority can, and after it acts the removed/hidden chirp no longer appears to users and a suspended account is treated as acted-upon. (The user-facing report/block capability and the exact presentation of the effect are owned by `abuse_and_privacy_basics`.)
- **Release gating:** true
- **Traceability:** REQ-PROD-028; the absent moderator persona; journey J8. Capability & visible effect → `abuse_and_privacy_basics`; enforcement → `software` domain.

##### REQ-ID-048 — Operator/administrator support-lookup and account-administration authority
- **Statement:** A principal holding operator/administrator authority may look up an account and see the information needed to answer a support request, and may perform account-administration actions through a provided operator surface rather than by manipulating storage directly.
- **Acceptance criteria:** An account **without** operator authority cannot look up another account through the operator surface (the attempt is refused); an account **with** operator authority can look up an account by its handle/identifier and see the support-relevant fields sufficient to answer requests such as "my account is gone" or "I can't sign in," performed through the provided surface. The lookup never exposes the account's credential.
- **Release gating:** true
- **Traceability:** REQ-PROD-033; persona P4; journey J8. The privacy limit on *what* support may see → `abuse_and_privacy_basics`; operator surface → `operational_readiness`; enforcement → `software` domain.

##### REQ-ID-049 — Elevated authority is least-privilege: no impersonation, no over-reach, no credential access
- **Statement:** Elevated authority is bounded. No elevated role may impersonate a user (perform an action attributed to a user's account), read a user's personal data beyond what the specific moderation or support action requires, or read or recover any account's credential.
- **Acceptance criteria:** No moderator or operator/administrator action can produce a chirp, follow, or like attributed to a user as its author; content acted on via moderation is never re-attributed to an operator as the author. An elevated lookup returns only the information the support/moderation action needs and never returns a credential. An attempt by an elevated principal to view personal data unrelated to a legitimate action is refused.
- **Release gating:** true
- **Traceability:** REQ-PROD-004, REQ-PROD-028, REQ-PROD-033; persona P4. The scope of "personal data support may see" → `abuse_and_privacy_basics`; enforcement → `software` domain.

##### REQ-ID-050 — Elevated actions are attributable to an identified elevated principal
- **Statement:** Every moderation or administration action is performed under, and attributable to, a distinct, identified elevated principal — not an anonymous, shared, or un-attributed "god mode."
- **Acceptance criteria:** For any moderation or admin action that has taken effect (a removed chirp, a suspended account, a support lookup), it is possible to determine which elevated principal performed it; elevated actions cannot be performed by an unidentified or shared super-user with no attributable identity.
- **Release gating:** true
- **Traceability:** REQ-PROD-028, REQ-PROD-033; persona P4; trust baseline for elevated power over user data. *A richer, reviewable action history (an audit trail an operator can browse and review over time) is a real improvement beyond this attribution baseline and is recorded as a non-gating enhancement, not a release blocker.* Recording mechanism → `software` domain / `operational_readiness`.

##### REQ-ID-051 — Elevated authority is explicitly assigned, revocable, and minimal
- **Statement:** A principal holds an elevated authority only by explicit assignment; the assignment can be removed; and the number of principals holding elevated authority is kept to the minimum needed to operate the product.
- **Acceptance criteria:** There is a defined way (even if operational rather than in-product, for a first release) to grant and to remove moderator and operator/administrator authority; after an assignment is removed, the former holder can no longer perform the corresponding elevated actions; an ordinary account that was never assigned an elevated authority holds none.
- **Release gating:** true
- **Traceability:** REQ-PROD-033; persona P4; least-privilege baseline. Assignment/revocation mechanism → `software` domain / `operational_readiness`.

##### REQ-ID-052 — An account acted on by moderation loses authority consistent with the action
- **Statement:** When moderation suspends or removes an account, that account's authority is curtailed consistent with the action: a suspended or removed account cannot authenticate or perform mutating actions while in that state, enforced on the server.
- **Acceptance criteria:** After an account is suspended or removed by an authorized moderator, attempts to sign in as, or act as, that account are refused server-side (not merely hidden in the UI); a restored account regains its ordinary authority. This holds whether the action is attempted through the web UI or directly against the API.
- **Release gating:** true
- **Traceability:** REQ-PROD-028, REQ-PROD-012; the absent moderator persona; journey J8. The user-visible effect of moderation → `abuse_and_privacy_basics`; durable effect across restart → `operational_readiness`; enforcement → `software` domain.

---

#### D. The authorization check (cross-cutting)

These requirements bind every surface of the product. They are the invariant that governs sections B and
C: the role model only means something if it is checked, on the server, on every attempt.

##### REQ-ID-053 — Every mutating action is authorized on the server and cannot be bypassed
- **Statement:** Every state-changing action is authorized on the server against the acting principal's identity and its specific authority for the target resource. Authorization is never enforced by the client alone, and cannot be bypassed by calling the API directly.
- **Acceptance criteria:** For a representative set of mutating actions (post, edit, delete chirp; edit profile; follow/unfollow; like/unlike; delete account; export; each elevated action), an unauthorized attempt made **directly against the API** — bypassing the web UI — is refused exactly as it is in the UI; no mutating action succeeds merely because the client allowed it.
- **Release gating:** true
- **Traceability:** REQ-PROD-004, REQ-PROD-013; personas P3, P4. Depends on safe sessions from `authentication_and_sessions`; enforcement → `software` domain.

##### REQ-ID-054 — Public reads are explicitly allowed without a session
- **Statement:** Reading public content — the public/home timeline, public profiles, and public chirps — requires no account and no session. The authorization model's default-deny applies to mutating actions, not to public reads.
- **Acceptance criteria:** A visitor with no session can load the public timeline and open any public profile and read its chirps without being asked to sign in; no public-read path is denied for lack of a session.
- **Release gating:** true
- **Traceability:** REQ-PROD-013; persona P1 and the reader-only sub-case of P3; journeys J1, J5.

##### REQ-ID-055 — Default-deny for mutations and unrecognised actions
- **Statement:** Any state-changing request that does not carry a granting authority — no valid session, or a session that lacks the required ownership or elevated authority for the target — is denied, not silently allowed, partially applied, or ignored.
- **Acceptance criteria:** A mutating request with no session, an expired/invalid session, or a session lacking the required authority results in a clear refusal and no change to any data; there is no action that "falls open" to success when its authority cannot be established.
- **Release gating:** true
- **Traceability:** REQ-PROD-004, REQ-PROD-013; personas P3, P4. Session validity → `authentication_and_sessions`; enforcement → `software` domain.

##### REQ-ID-056 — A verified identity is necessary but not sufficient for a mutation
- **Statement:** Performing a mutating action requires both an authenticated identity (a valid session proving who the actor is) **and** an authority check establishing that this actor may perform this action on this resource. A valid session alone never authorizes an action; the per-resource authority must also hold.
- **Acceptance criteria:** A user with a perfectly valid session is still refused when they attempt an action outside their authority (e.g. deleting another user's chirp, or performing a moderator action without moderator authority); authority is evaluated per action and per target, not granted wholesale by the mere existence of a session.
- **Release gating:** true
- **Traceability:** REQ-PROD-004; personas P3, P4. Session issuance/safety → `authentication_and_sessions`; enforcement → `software` domain.

##### REQ-ID-057 — Authorization is consistent across every entry point
- **Statement:** The same authority rules apply to every entry point into the product. The REST API is not a privileged back door: an action refused in the web UI for lack of authority is refused when attempted directly against the API, and vice versa.
- **Acceptance criteria:** For the representative mutating actions, the authorization outcome (allowed/refused) is the same whether the action is attempted through the SPA or directly against the API, for the same principal and target; no entry point grants authority another denies.
- **Release gating:** true
- **Traceability:** REQ-PROD-004, REQ-PROD-013; personas P3, P4. Enforcement → `software` domain.

---

#### E. Cross-spoke couplings (for the synthesis to resolve into explicit cross-references)

The authority model is coupled to the other identity/security concerns. Each coupling below names the two
facets so the synthesis gate can resolve it into an explicit `REQ-ID ↔ REQ-ID` cross-reference in the
assembled document.

1. **Deletion — authority vs. lifecycle semantics.** The *authority* "only the owner or an authorized
   administrator may delete an account" (**REQ-ID-043**) is coupled to the *deletion semantics and the
   guarantee that deletion is honoured* owned by the **`accounts_and_lifecycle`** section
   (`REQ-ID-001..019`). Synthesis to cross-reference REQ-ID-043 ↔ the accounts-lifecycle deletion
   requirement(s).
2. **Export/erasure — authority vs. mechanism vs. data-right.** The *authority* to export
   (**REQ-ID-044**) is coupled to the *export mechanism* in **`accounts_and_lifecycle`**
   (`REQ-ID-001..019`) and to the *data-right it satisfies* in **`abuse_and_privacy_basics`**
   (`REQ-ID-060..079`). Synthesis to cross-reference REQ-ID-044 ↔ both.
3. **Moderation — authority vs. capability and visible effect.** The *authority* to act on abuse
   (**REQ-ID-047**) and the *loss of a suspended/removed account's authority* (**REQ-ID-052**) are
   coupled to the *user-facing report/block capability and the visible moderation effect* owned by
   **`abuse_and_privacy_basics`** (`REQ-ID-060..079`). Synthesis to cross-reference REQ-ID-047 and
   REQ-ID-052 ↔ the abuse/privacy capability-and-effect requirement(s).
4. **Admin/support lookup — authority vs. privacy limit.** The operator *authority* to look up an
   account (**REQ-ID-048**) and the least-privilege *bound* on it (**REQ-ID-049**) are coupled to the
   *privacy limit on what personal data support may see* owned by **`abuse_and_privacy_basics`**
   (`REQ-ID-060..079`). Synthesis to cross-reference REQ-ID-048 and REQ-ID-049 ↔ the privacy-limit
   requirement(s).
5. **Public-internet session safety — session safety vs. per-action authorization.** The cross-cutting
   *server-side authorization on every mutation* (**REQ-ID-053**) and *verified-identity-is-necessary-
   but-not-sufficient* (**REQ-ID-056**) are coupled to the *public-internet session-safety* properties
   owned by **`authentication_and_sessions`** (`REQ-ID-020..039`): safe sessions provide the verified
   identity; per-action authorization is what makes that identity safe to expose. Synthesis to
   cross-reference REQ-ID-053 and REQ-ID-056 ↔ the session-safety requirement(s).

---

#### F. Watch-points and out of scope

- **Organisation / team / tenant roles — out of scope.** chirp is a single shared consumer product with
  no notion of a tenant, team, or organisation; this authority model has exactly the roles above (user,
  moderator, operator/administrator) and no per-tenant isolation or org-admin tier. **Watch-point:** if
  chirp ever grew organisation or team accounts, multi-tenant roles, per-tenant isolation, and an
  org-admin authority would return and would need to be scoped in. (Consistent with the product
  requirements' out-of-scope on multi-tenancy.)
- **Billing / entitlement authority — out of scope.** chirp has no commercial model, so there is no
  paid-tier entitlement or billing-administration authority to design.
- **Enforcement "how" — deferred, not omitted.** How each authorization decision is computed and
  enforced on every request (check placement, middleware, policy representation), and how elevated roles
  are stored and assigned, are **`software`-domain** concerns. This section fixes *what must be true*; the
  code DAGs own *how it is made true*.
- **Richer elevated-action audit — non-gating enhancement.** Beyond the attribution baseline
  (REQ-ID-050), a browsable, reviewable history of moderation/admin actions is a real improvement a first
  releasable version may ship without.

---

## abuse_and_privacy_basics

<!-- Section: abuse_and_privacy_basics of design/identity-security.md. The minimum abuse-resistance and privacy/data-rights surface a public product owes its users. Requirements only (REQ-ID-060..075). -->

### Abuse-resistance & privacy/data-rights basics

*(section: `abuse_and_privacy_basics`)*

chirp is a public product that lets anyone with an account post in public and lets anyone — signed in or
not — read. That openness is the point, and it is also what makes a trust-and-safety and privacy surface
non-optional: a public, **anonymous-signup** social product released with no way for a user to respond to
abuse, no baseline resistance to flooding, and no statement of what it does with people's data is not a
releasable product of its kind. This section states the **minimum** that must be true on three fronts —
how a user responds to unwanted content and accounts, how the product resists trivial flooding, and the
privacy and data-rights surface the product owes its users — as testable `REQ-ID-*` requirements.

**Grounded starting point.** As the product stands today (`Nousix-LLC/nswe-demo` @ `c805525`) there is
**no reporting, no block or mute, no rate or volume limits on sign-up or posting, and no privacy or terms
surface** of any kind. Everything below is a real gap against a releasable bar, not a hypothetical.

**Boundary — requirements, not mechanism.** This section specifies *what must be true* from the user's
and operator's side. It does **not** choose a spam filter, a rate-limiter, a consent-management library, a
moderation-queue design, or an export file format — those "how" decisions belong to later code work in the
`software` domain, and to the operational-readiness area for the enforcement of limits and quotas
(`REQ-OPS-*`, section `limits_and_quotas`). Where a requirement here depends on an **authority** to act,
that authority is owned by the authorization model (`authorization`, `REQ-ID-040..059`); where it depends
on an account-lifecycle **mechanism** (export, erasure), that mechanism is owned by
`accounts_and_lifecycle` (`REQ-ID-001..019`). Those couplings are listed explicitly at the end of this
section for the synthesis to resolve into cross-references.

This section owns `REQ-ID-060 .. REQ-ID-079` and uses `REQ-ID-060 .. REQ-ID-075`.

---

#### 1. User-facing abuse response

A public posting product owes every user two things when they meet content or an account they do not
want: a way to **report** it to whoever runs the product, and a way to **reduce their own exposure**
directly, without waiting for anyone. Both must give the user honest confirmation that the action was
taken, and when a moderator does act, the effect must be **visible** — abusive content must actually stop
appearing. chirp has none of this today.

These are *capability* requirements. The **authority** to act on a report — to remove a chirp or suspend
an account — is part of the authorization model (`REQ-ID-040..059`); this section specifies the capability
the user holds and the effect the user sees.

##### REQ-ID-060 — Report a chirp or an account
- **Statement:** A signed-in user can report a specific chirp or a specific account to whoever operates the product, and is told the report was received.
- **Acceptance criteria:** From a chirp and from a profile, a signed-in user can start a report, optionally indicate a reason, submit it, and receive a clear confirmation that the report was received. The reported item is identified unambiguously in what is submitted. Submitting a report does not require the reporter to contact the operator out of band.
- **Release gating:** true
- **Traceability:** REQ-PROD-027 (users can act on unwanted content/accounts). Report screen/affordance → experience design (`REQ-UX-*`). Routing/handling of a received report and the authority to act → authorization (`REQ-ID-040..059`) and operational readiness (`REQ-OPS-*`).

##### REQ-ID-061 — Block an account
- **Statement:** A signed-in user can block another account so that the blocked account can no longer interact with them, and is told the block is in place.
- **Acceptance criteria:** A signed-in user can block an account from that account's profile; after blocking, the blocked account cannot follow, reply to, or like the blocking user's chirps, and the blocking user is shown clear confirmation that the block is active. The user can later see which accounts they have blocked and can unblock, with the interaction restrictions lifted on unblock.
- **Release gating:** true — this is the exposure-reducing control REQ-PROD-027 requires a releasable product to provide.
- **Traceability:** REQ-PROD-027. Block UI/affordance → experience design (`REQ-UX-*`). Enforcement of interaction restrictions → `software` domain.

##### REQ-ID-062 — Mute an account
- **Statement:** A signed-in user can mute another account so that account's chirps no longer appear in the muting user's own timeline and views, without affecting the muted account.
- **Acceptance criteria:** A signed-in user can mute an account; after muting, that account's chirps are absent from the muting user's timeline and are suppressed in the muting user's other content views, while the muted account is unaffected and is not told it was muted. The user can see their muted accounts and unmute, restoring that account's chirps to their views.
- **Release gating:** false — block (REQ-ID-061) satisfies REQ-PROD-027's exposure-reduction requirement for the release bar; mute is a softer, complementary control a first releasable version may ship without. Recorded as a near-term refinement, not an omission of safety.
- **Traceability:** REQ-PROD-027. Mute UI → experience design (`REQ-UX-*`).

##### REQ-ID-063 — Moderation action has a visible effect
- **Statement:** When content or an account is acted on by moderation, the effect is visible to users: removed content stops appearing and a suspended or removed account's content stops appearing.
- **Acceptance criteria:** After a chirp is removed by an authorized operator, that chirp no longer appears in any timeline, profile, or thread served to users; after an account is suspended or removed, its chirps no longer appear to users and its profile reflects the action rather than continuing to serve content as normal. The change is observable by an ordinary user exercising the product, not only in an operator view.
- **Release gating:** true
- **Traceability:** REQ-PROD-028 (abuse can be acted on by whoever runs the product; the effect is visible to users). The **authority** and operator tooling to perform the removal/suspension → authorization (`REQ-ID-040..059`) and operational readiness (`REQ-OPS-*`); this requirement owns the **user-visible effect** of that action.

##### REQ-ID-064 — Abuse controls are discoverable and confirm the action
- **Statement:** The report, block, and mute controls are reachable from where the content or account is, and every such action gives the user clear confirmation that it was taken.
- **Acceptance criteria:** A user who encounters an unwanted chirp or account can find the report and block/mute controls in context (on the chirp and on the profile) without leaving the product or consulting documentation; each control, when used, produces a visible confirmation of success or an honest, recoverable error on failure — never a silent no-op.
- **Release gating:** true — REQ-PROD-027 requires the user to *receive confirmation that the action was taken*, and the "no silent failure" principle applies to safety actions as it does to the core loops.
- **Traceability:** REQ-PROD-027. Placement/affordance and confirmation presentation → experience design (`REQ-UX-*`).

---

#### 2. Baseline abuse-resistance

Because anyone can create an account with no credential cost beyond choosing a free handle, and anyone
signed in can post, the product is **trivially floodable** today: nothing stops a script from creating
accounts in bulk or posting without limit. A releasable public product needs a **baseline** resistance to
this — enough that casual or scripted flooding does not overwhelm the product or its users on day one.
This is a *preventive* complement to the *reactive* response in part 1.

These are **requirements, not mechanisms.** This section states that limits must exist and how they must
behave from the user's side; the enforcement mechanism (where limits live, how they are counted, the
thresholds) is owned by operational readiness (`REQ-OPS-*`, `limits_and_quotas`) and the `software`
domain. No rate-limiter or spam-filter is designed here.

##### REQ-ID-065 — Baseline limit on account creation
- **Statement:** The rate and volume at which accounts can be created is limited, so that a public, anonymous sign-up cannot be used to create accounts in unlimited bulk.
- **Acceptance criteria:** Repeated automated account creation beyond a reasonable human rate is refused or throttled rather than always succeeding; a normal person signing up once is unaffected. The limit is enforced on the server side (not only in the browser), so bypassing the UI does not bypass the limit.
- **Release gating:** true — a public anonymous-signup product with no creation limit is not responsibly releasable. The specific thresholds are an operational choice; the *existence* of a baseline limit is the requirement.
- **Traceability:** REQ-PROD-027, REQ-PROD-028 (a public product must be releasable responsibly against abuse). Enforcement mechanism and thresholds → operational readiness (`REQ-OPS-*`, `limits_and_quotas`) and `software` domain.

##### REQ-ID-066 — Baseline limit on posting and high-volume actions
- **Statement:** The rate and volume at which a single account can post chirps and perform high-volume social actions (such as following and liking) is limited, so that one account cannot flood the product.
- **Acceptance criteria:** A single account posting or acting far faster or in far greater volume than a human plausibly would is throttled or refused rather than always succeeding; ordinary posting and interaction by a real user is unaffected. The limit is enforced server-side.
- **Release gating:** true — baseline anti-flood protection of the shared timeline and social graph is part of a responsibly releasable public product.
- **Traceability:** REQ-PROD-027, REQ-PROD-028. Enforcement mechanism and thresholds → operational readiness (`REQ-OPS-*`, `limits_and_quotas`) and `software` domain.

##### REQ-ID-067 — Limits are enforced safely and communicated, never silent
- **Statement:** When a baseline limit is reached, the product refuses the action safely and tells the user, rather than silently dropping it or appearing to succeed.
- **Acceptance criteria:** A user who hits a limit sees a clear, honest message that the action was rate-limited and can understand when they may retry; the limited action is not silently discarded and is not falsely shown as succeeded. Limits default to protecting the product (fail safe) rather than failing open under load.
- **Release gating:** true — consistent with the product-wide "no silent failure" expectation for user actions.
- **Traceability:** REQ-PROD-027, REQ-PROD-028. Message presentation → experience design (`REQ-UX-*`); fail-safe behaviour of the enforcement path → operational readiness (`REQ-OPS-*`) and `software` domain.

---

#### 3. Privacy & data-rights surface

A consumer product that collects personal data owes its users a plain statement of **what it holds** and
the **terms** under which it is used, a clear understanding of **what is public**, and real control over
their own data — the rights of **access/export** and **erasure**. It also owes them a limit on how much of
their personal data the people operating the product can see. chirp has none of this surface today.

The data-rights below are stated here as **rights the user holds**; the **mechanism** that satisfies them
(the export and deletion/erasure flows) is owned by `accounts_and_lifecycle` (`REQ-ID-001..019`). This
section fixes the *right*; that section fixes the *how*.

##### REQ-ID-068 — A privacy statement of what personal data is held
- **Statement:** The product provides a plain-language statement of what personal data chirp collects and holds about a user, reachable both logged-out and signed-in.
- **Acceptance criteria:** From the product, a logged-out visitor and a signed-in user can each reach a readable statement that names the categories of personal data chirp holds about a user (for example: the handle, display name, bio, the chirps they post, their follows and likes, and any data recorded when they sign in). The statement is present and readable in the product, not merely implied.
- **Release gating:** true
- **Traceability:** REQ-PROD-029 (a privacy and terms surface). Content scope owned here; page presentation and placement → experience design (`REQ-UX-*`).

##### REQ-ID-069 — A terms-of-use statement
- **Statement:** The product provides a statement of the terms under which the product may be used, reachable both logged-out and signed-in.
- **Acceptance criteria:** From the product, a logged-out visitor and a signed-in user can each reach a readable terms-of-use statement covering, at minimum, acceptable use (what content/behaviour is not allowed) and the basis on which the service is provided. The statement is present and readable in the product.
- **Release gating:** true
- **Traceability:** REQ-PROD-029. Page presentation → experience design (`REQ-UX-*`); acceptable-use terms align with the abuse-response capability in part 1.

##### REQ-ID-070 — Right of access and export to one's own data
- **Statement:** A user has the right to obtain their own personal data held by chirp, in a usable, portable form.
- **Acceptance criteria:** A signed-in user can exercise a right to obtain their own data — at least their profile, their chirps, and their follow list — and receive it in a documented, machine-readable form they can open outside chirp. The right is stated to the user as a right, not only available as an incidental feature.
- **Release gating:** true
- **Traceability:** REQ-PROD-011 (export your own data). The **export mechanism and format** that satisfies this right → `accounts_and_lifecycle` (`REQ-ID-001..019`) and domain & application design (`REQ-DOM-*`); this requirement owns the *data-right*.

##### REQ-ID-071 — Right to erasure of one's personal data
- **Statement:** A user has the right to have their personal data erased when they delete their account, and to be told what erasure covers.
- **Acceptance criteria:** A signed-in user can exercise a right to erasure such that, after they delete their account, their personal data is no longer held or served by the product, and they are shown a clear statement of what erasure removes. The right is stated to the user as a right.
- **Release gating:** true
- **Traceability:** REQ-PROD-012 (deletion is honoured). The **deletion/erasure mechanism and its guarantee** that satisfies this right → `accounts_and_lifecycle` (`REQ-ID-001..019`); durable removal → operational readiness (`REQ-OPS-*`); this requirement owns the *data-right*.

##### REQ-ID-072 — Privacy limit on the admin/support view (data minimisation)
- **Statement:** When an operator uses the admin/support capability to look up an account, they see only the data needed to answer the request — not a user's full personal data by default, and never anything a credential should protect.
- **Acceptance criteria:** The admin/support account-lookup surface exposes the minimum necessary to support a user (for example: that an account exists, its public profile, and operational status), and does **not** expose a user's authentication secrets or more personal data than the support task requires. What the operator may see is a stated limit, not left to whatever the storage happens to contain.
- **Release gating:** true — a releasable admin/support capability must respect a privacy limit; an unbounded operator view of personal data is not acceptable in a consumer product.
- **Traceability:** REQ-PROD-029 (privacy surface), REQ-PROD-033 (minimal admin/support capability). The **authority** that grants admin/support lookup → authorization (`REQ-ID-040..059`); the operator surface → operational readiness (`REQ-OPS-*`); this requirement owns the *privacy limit* on that authority.

##### REQ-ID-073 — What is public is clear to the user
- **Statement:** The product makes clear to the user what is public — that their handle, display name, bio, chirps, follows, and likes are visible to anyone, including logged-out visitors.
- **Acceptance criteria:** A user can learn, from the product (in the privacy statement and/or at the point of posting/sign-up), that chirps and profile information are public and readable without an account; a user is not led to believe content is private when it is public. No personal field is presented as private when the product in fact serves it publicly.
- **Release gating:** true — honesty about public-by-default visibility is central to a public product's privacy surface and cheap to provide.
- **Traceability:** REQ-PROD-029; REQ-PROD-013 (reading is public and requires no account — the visibility this requirement discloses). Where and how the disclosure appears → experience design (`REQ-UX-*`).

##### REQ-ID-074 — The privacy statement is accurate to what is actually held
- **Statement:** The privacy statement accurately reflects the personal data the product actually collects and holds; the product does not collect or hold categories of personal data the statement does not disclose.
- **Acceptance criteria:** The categories named in the privacy statement (REQ-ID-068) match what the product actually holds about a user; a reviewer can check the statement against the product's real data and find no undisclosed category of personal data being collected or retained.
- **Release gating:** true — an inaccurate privacy statement is a trust failure; accuracy is what makes REQ-ID-068 meaningful.
- **Traceability:** REQ-PROD-029. The actual data held is defined by domain & application design (`REQ-DOM-*`); this requirement binds the statement to that reality.

##### REQ-ID-075 — Block and mute relationships are private to the user who set them
- **Statement:** A user's block and mute relationships are visible only to that user and are not disclosed to the accounts they block or mute, nor to other users.
- **Acceptance criteria:** An account that has been blocked or muted is not told, by the product, that it was blocked or muted; no user can see another user's block or mute list. Only the user who set a block/mute can see their own.
- **Release gating:** false — satisfied by not building any surface that discloses block/mute relationships; recorded so that if a block/mute list is ever exposed, this privacy property is preserved by construction.
- **Traceability:** REQ-PROD-027 (the exposure-reduction controls), REQ-PROD-029 (privacy). Couples to REQ-ID-061/062.

---

#### Cross-spoke couplings (for the synthesis to resolve into REQ-ID cross-references)

These requirements depend on, or are depended on by, requirements owned by sibling spokes. Each is stated
here in prose so the synthesis gate can resolve it into an explicit `REQ-ID ↔ REQ-ID` cross-reference.

1. **Moderation: user-facing capability/effect ↔ authority to act.** The report capability (REQ-ID-060)
   and the visible moderation effect (REQ-ID-063) require an authorised operator/moderator to act; that
   **authority** is owned by the `authorization` spoke (`REQ-ID-040..059`). Synthesis: cross-reference
   REQ-ID-060/063 ↔ the authz requirement(s) that grant moderation authority.
2. **Data export: data-right ↔ export mechanism.** The right of access/export (REQ-ID-070) is satisfied
   by the **export mechanism and format** owned by `accounts_and_lifecycle` (`REQ-ID-001..019`).
   Synthesis: cross-reference REQ-ID-070 ↔ the accounts-lifecycle export requirement.
3. **Data erasure: data-right ↔ deletion/erasure mechanism.** The right to erasure (REQ-ID-071) is
   satisfied by the **deletion/erasure semantics and guarantee** owned by `accounts_and_lifecycle`
   (`REQ-ID-001..019`). Synthesis: cross-reference REQ-ID-071 ↔ the accounts-lifecycle deletion/erasure
   requirement.
4. **Admin/support: privacy limit ↔ admin lookup authority.** The privacy limit on the admin/support view
   (REQ-ID-072) constrains the **admin/support lookup authority** owned by `authorization`
   (`REQ-ID-040..059`). Synthesis: cross-reference REQ-ID-072 ↔ the authz admin/support authority
   requirement.
5. **Baseline limits: abuse-resistance requirement ↔ operational enforcement.** The baseline limits
   (REQ-ID-065/066/067) state the requirement; their **enforcement mechanism and thresholds** belong to
   operational readiness (`REQ-OPS-*`, `limits_and_quotas`) and the `software` domain. Noted for the
   synthesis/traceability as a cross-area dependency rather than an intra-section REQ-ID coupling.

---

#### Out of scope / recorded cuts for this release

Judgments recorded honestly, each with a reason — not silent omissions:

- **Full trust-and-safety tooling** (moderation queues, automated abuse detection/classification, strike
  systems, appeals workflow). *Reason:* beyond the minimum for a responsible release; the release bar needs
  report → act → visible-effect (part 1) and baseline limits (part 2), not a mature safety platform.
- **Appeals / re-review of moderation decisions.** *Reason:* a real improvement, but a first releasable
  version can ship with report-and-act without a formal appeals process. Near-term addition.
- **Consent management / cookie and tracking preferences.** *Reason:* chirp's privacy surface here is a
  truthful statement of what is held plus the data-rights; a consent/preferences framework is only needed
  if the product later adds tracking or optional data uses — out of scope at this commit.
- **Granular per-item privacy (private accounts, protected/followers-only chirps, per-post visibility).**
  *Reason:* chirp is public-by-default by product definition (REQ-PROD-013); private-account modes are a
  product-scope change, not part of this release's privacy surface. REQ-ID-073 instead requires the product
  be *honest* that content is public.
- **Regulatory certification of specific legal regimes.** *Reason:* the product owes a truthful
  privacy/terms surface and real access/export/erasure rights (the substance); formal certification against
  a named legal regime is an engagement/legal decision, not a design requirement settled here.

---

#### Traceability summary (`REQ-ID → REQ-PROD`, release gating)

| REQ-ID | Concern | Release gating | Traces to REQ-PROD |
|--------|---------|----------------|--------------------|
| REQ-ID-060 | Report a chirp or account | true | REQ-PROD-027 |
| REQ-ID-061 | Block an account | true | REQ-PROD-027 |
| REQ-ID-062 | Mute an account | false | REQ-PROD-027 |
| REQ-ID-063 | Visible moderation effect | true | REQ-PROD-028 |
| REQ-ID-064 | Abuse controls discoverable + confirm | true | REQ-PROD-027 |
| REQ-ID-065 | Baseline account-creation limit | true | REQ-PROD-027, REQ-PROD-028 |
| REQ-ID-066 | Baseline posting/action limit | true | REQ-PROD-027, REQ-PROD-028 |
| REQ-ID-067 | Limits safe + communicated, not silent | true | REQ-PROD-027, REQ-PROD-028 |
| REQ-ID-068 | Privacy statement (data held) | true | REQ-PROD-029 |
| REQ-ID-069 | Terms-of-use statement | true | REQ-PROD-029 |
| REQ-ID-070 | Right of access/export | true | REQ-PROD-011 |
| REQ-ID-071 | Right to erasure | true | REQ-PROD-012 |
| REQ-ID-072 | Privacy limit on admin/support view | true | REQ-PROD-029, REQ-PROD-033 |
| REQ-ID-073 | Public-by-default is clear to the user | true | REQ-PROD-029, REQ-PROD-013 |
| REQ-ID-074 | Privacy statement is accurate | true | REQ-PROD-029 |
| REQ-ID-075 | Block/mute relationships are private | false | REQ-PROD-027, REQ-PROD-029 |

---

## threat_model

This section is a **synthesis of the four concerns above**. It names the concrete threats chirp's public
surface implies — grounded in the product as it stands at `c805525` (handle-only sign-in, in-memory bearer
sessions with no expiry/sign-out, no credential, no recovery, no deletion/export, no moderation, no rate
limits, no privacy surface) — and, for each, names the requirement(s) that mitigate it, or records it as
accepted or out-of-scope with a reason. It is not a catalogue of generic web risks: every threat below is
one this product is actually exposed to today, and every mitigation is a `REQ-ID-*` elsewhere in this
document (or a deliberate cut). Where a threat was not already closed by a spoke requirement, the threat
model mints a net-new requirement in its own range (`REQ-ID-080..081`); those are stated in full in the
requirements section that follows.

The model is organised by the trust boundary each threat crosses. The single most important fact about
chirp's current surface is that **it has no authentication at all** — the handle is the whole "identity" —
so the first boundary (proving who you are) is where the gravest threats sit, and almost every other
boundary assumes it has been closed.

### Assets worth protecting

- A user's **account and its ownership** — the ability to act as, and only as, oneself.
- A user's **content and social graph** — chirps, follows, likes, profile.
- A user's **personal data** — what the product holds, and the limits on who may see it.
- **Session integrity** — that a live session acts only for its owner and ends when it should.
- The **shared surface** — the public timeline and the service itself, as a resource that must stay
  usable for everyone.
- **Operator trust** — that elevated power over user data is bounded and attributable.

### Principal threat actors

- An **unauthenticated outsider** who knows only public information (handles are public).
- An **authenticated ordinary user** acting beyond their own resources.
- A **network observer** on the path between a user and the service.
- A **malicious author** who plants content that runs in other users' browsers or forges their actions.
- An **abuser** flooding the product or harassing other users.
- A **rogue or over-broad operator** with elevated authority.

---

### T1 — Account takeover by handle (spoofing the owner) — *the defining threat today*

**Threat.** Because sign-in requires only a username, anyone who knows a handle — and every handle is
public — can obtain a session as that account and post, follow, like, edit the profile, or (once those
exist) delete the account or export its data. This is not a hypothetical: it is the product's current
behaviour. It is a complete break of the "this account is mine" promise.

**Mitigated by.** `REQ-ID-020` (sign-in requires proof of a held secret; a handle alone never
authenticates), `REQ-ID-021` (the credential is verified on every sign-in), `REQ-ID-001`/`REQ-ID-005` (an
account is created *with* a credential that is its durable ownership anchor), `REQ-ID-041` (every action is
authorized against the actor's own identity; knowing a handle does not let you act as it), and `REQ-ID-034`
(acting as an account requires a valid session).

### T2 — Credential guessing, brute force, and credential stuffing

**Threat.** Once a secret credential exists, an attacker will try to guess it — online guessing against one
account, or reuse of credentials leaked elsewhere. Without resistance this re-opens T1 by another door.

**Mitigated by.** `REQ-ID-023` (credentials resist casual guessing and online attempts are throttled),
`REQ-ID-022` (uniform, non-informative authentication failure, including no timing oracle), and
`REQ-ID-027` (the credential is exchanged only over a confidential channel). The *enforcement thresholds*
of the throttle are an operational concern (`REQ-OPS-*`, coupled via `REQ-ID-065`/`REQ-ID-066`); the
*existence* of the resistance is required here.

### T3 — Session theft / hijacking (bearer-token capture)

**Threat.** A session is a bearer credential: whoever holds the token can act as the account. Today the
token has no expiry and cannot be invalidated, so a captured token is a permanent key. Capture vectors
include a network observer (T13), a cross-site leak, or a shared/stolen device.

**Mitigated by.** `REQ-ID-027` (the session secret is unguessable, confidential, and protected in
transit), `REQ-ID-028` (bounded idle and absolute lifetime), `REQ-ID-031` (deliberate sign-out invalidates
the session server-side), `REQ-ID-032` (credential change/recovery invalidates existing sessions),
`REQ-ID-036` (a user can see and end their own other sessions — non-gating), and `REQ-ID-033` (fixation
resistance, see T4).

### T4 — Session fixation

**Threat.** An attacker pre-plants or fixes a session value and induces the victim to authenticate into
it, then rides the now-authenticated session.

**Mitigated by.** `REQ-ID-033` (authentication yields a fresh session a prior observer does not hold; a
pre-supplied session value does not become authenticated on the victim's sign-in).

### T5 — Recovery as an authentication bypass

**Threat.** An account-recovery path is, by construction, an alternate way to gain access. If it can be
completed with public information (such as a handle), it becomes a second, weaker way into every account,
defeating T1's mitigations. A recovery path that is not safe is worse than none.

**Mitigated by.** `REQ-ID-007` (recovery requires demonstrating control of a pre-established factor; public
information is never sufficient; it does not disclose whether an account exists), `REQ-ID-037` (recovery
re-proves control, results in a newly set credential, invalidates pre-existing sessions, and is never
easier than normal authentication), `REQ-ID-008` (successful recovery supersedes the lost means of access;
a failed attempt changes nothing), `REQ-ID-002` (a recovery factor is established at creation so there is
something to prove), `REQ-ID-024` (recovery never discloses the old credential), and `REQ-ID-032`
(pre-existing sessions are invalidated so an attacker's session does not survive the owner regaining
control).

### T6 — Privilege escalation / acting on another user's resources (IDOR, direct-API bypass)

**Threat.** An authenticated user tries to edit or delete another account's chirps, profile, follows, or
account — or to make an action appear to come from another account — often by calling the REST API
directly, bypassing whatever the web client allows. The API is the real security boundary.

**Mitigated by.** `REQ-ID-040` (a user may act only on resources they own), `REQ-ID-042` (a user manages
only their own social graph and likes), `REQ-ID-041` (every action is attributed to and authorized against
the actor's own identity; no acting-as-another), `REQ-ID-053` (every mutation is authorized on the server
and cannot be bypassed by calling the API directly), `REQ-ID-055` (default-deny for mutations),
`REQ-ID-056` (a valid session is necessary but not sufficient — per-resource authority must also hold),
and `REQ-ID-057` (authorization is identical across the SPA and the API; the API is not a privileged back
door).

### T7 — Abuse of elevated authority (rogue or over-broad operator/moderator)

**Threat.** Moderator and operator/administrator roles hold power over other people's accounts and data.
The threat is a holder who impersonates a user, reads more personal data than a task needs, reads or
recovers credentials, or acts without attribution ("god mode").

**Mitigated by.** `REQ-ID-045` (holding an account grants ownership authority only; elevation is never
implicit), `REQ-ID-049` (elevated authority is least-privilege: no impersonation, no over-reach, no
credential access), `REQ-ID-050` (every elevated action is attributable to an identified principal),
`REQ-ID-051` (elevated authority is explicitly assigned, revocable, and kept minimal), `REQ-ID-072` (a
stated privacy limit — data minimisation — on what the admin/support view may show), and `REQ-ID-024` (no
surface, including an operator view, reveals a credential).

### T8 — Account enumeration / identity disclosure

**Threat.** An attacker probes sign-in or recovery to learn which accounts exist, as reconnaissance for T1,
T2, or T5. (Handle *existence* is already public by design — profiles are public — so the sensitive oracle
is not "does this handle exist" but "does this authentication/recovery attempt behave differently for a
real vs. a non-existent account.")

**Mitigated by.** `REQ-ID-022` (authentication failure is uniform and non-informative across a wrong
credential for a real account and a non-existent account, with no timing oracle) and `REQ-ID-007`
(initiating recovery for an existing vs. a non-existing handle is indistinguishable to someone who cannot
complete it).

### T9 — Credential disclosure (display, log, export, or operator view)

**Threat.** A credential leaks because the product shows it, returns it, logs it, puts it in an export, or
exposes it to an operator — turning one careless surface into account takeover.

**Mitigated by.** `REQ-ID-024` (a credential is never displayed, returned, recorded in recoverable form,
or shown in any screen, API response, export, notification, operator view, or log; loss is resolved by
re-establishing a new one), reinforced by `REQ-ID-049` and `REQ-ID-072` on the operator side and
`REQ-ID-017` (an export contains only the requester's own data, never another account's secrets).

### T10 — Flooding and resource exhaustion (bulk sign-up, posting floods)

**Threat.** Anonymous, credential-free sign-up and unlimited posting make the product trivially floodable:
a script can create accounts in bulk or flood the shared timeline and social graph, degrading the product
for everyone. This is a denial-of-service against the shared surface and a force-multiplier for abuse
(T11).

**Mitigated by.** `REQ-ID-065` (a baseline limit on account creation, server-side), `REQ-ID-066` (a
baseline limit on posting and high-volume actions, server-side), and `REQ-ID-067` (limits fail safe and are
communicated, never silent). The concrete thresholds and the enforcement mechanism are operational
(`REQ-OPS-*`, `limits_and_quotas`); the *existence and behaviour* of the limits are required here.

### T11 — Harassment and abusive content (no user recourse, no moderation)

**Threat.** A public posting product with no reporting, no block/mute, and no moderation leaves users no
way to respond to harassment or unwanted content, and leaves the operator no way to act on abuse. For a
public product this is a trust-and-safety failure, not a missing feature.

**Mitigated by.** `REQ-ID-060` (report a chirp or account), `REQ-ID-061` (block an account — the gating
exposure-reduction control), `REQ-ID-062` (mute — non-gating), `REQ-ID-064` (controls are discoverable and
confirm the action), `REQ-ID-063` (moderation has a visible effect), `REQ-ID-047` (moderator authority to
act on abusive content and accounts), and `REQ-ID-052` (an account acted on by moderation loses authority
consistent with the action, enforced server-side).

### T12 — Silent session loss and dishonest state (false success / unexplained sign-out)

**Threat.** Today in-memory sessions vanish on every restart, silently signing everyone out; a user may
also believe an action succeeded when their session had actually ended. Silent failure erodes trust and can
mask security-relevant state (a user acting on a stale session).

**Mitigated by.** `REQ-ID-029` (no silent sign-out — expiry/invalidation always surfaces a clear
re-authenticate prompt, never a silent no-op or false success), `REQ-ID-030` (signed-in vs. logged-out
state is always shown truthfully), and `REQ-ID-035` (routine operations do not silently sign users out —
the session continues or the user is clearly told). Durable session continuity depends on durable data
(`REQ-OPS-*` / `REQ-DOM-*`).

### T13 — Interception / tampering on the network path

**Threat.** A network observer between a user and the service reads or alters the credential or the session
token in transit, enabling T1/T3.

**Mitigated by.** `REQ-ID-027` (all exchange of the credential and session secret occurs over a
confidential, integrity-protected channel; the product refuses to transmit them over an unprotected one).
The specific transport mechanism is a `software`/`REQ-OPS-*` "how"; the property is required here.

### T14 — Cross-site request forgery (forged authenticated mutations) — *net-new: `REQ-ID-080`*

**Threat.** chirp is a browser product whose mutating actions are authenticated. A third-party site a
signed-in user visits could attempt to make the user's browser perform an authenticated mutation (post,
follow, delete, export) without the user's intent. Whether this is exploitable depends on how the session
is carried — a "how" owned by `software` — but the *property* that forged cross-origin mutations must not
succeed is a release-relevant security requirement no spoke currently owns.

**Mitigated by.** `REQ-ID-080` (a mutating action must originate from the product's own authenticated
client; a third-party site cannot cause a user's browser to perform an authenticated mutation without the
user's intent). Composes with `REQ-ID-053`/`REQ-ID-056` (server-side per-action authorization).

### T15 — Stored injection: user content executing in another user's browser (stored XSS) — *net-new: `REQ-ID-081`*

**Threat.** chirp renders attacker-controlled text (chirps, display name, bio) to every reader of the
shared timeline and profiles. If that content can execute as code in another user's browser, a single
malicious chirp becomes a vector to steal sessions (T3) or credentials (T9) from everyone who views it —
the highest-leverage attack a public posting product faces. No spoke owns content-rendering safety.

**Mitigated by.** `REQ-ID-081` (user-submitted content is treated as data, not code: it cannot execute in
another user's browser or break out of its display context). The escaping/sanitisation/CSP mechanism is a
`software`/`REQ-UX-*` "how"; the property is required here.

### T16 — Destructive action from a hijacked but unlocked session

**Threat.** An attacker who obtains a live, unlocked session (a shared or stolen device, or a captured
token before T3's mitigations fully apply) uses it to perform irreversible, identity-changing actions —
changing the credential or recovery factor, or deleting the account — locking out the real owner.

**Mitigated by.** `REQ-ID-025` (destructive or identity-changing actions require re-proving the credential
in the current session, not the session alone), composing with `REQ-ID-032` (a credential change
invalidates other sessions) and `REQ-ID-014` (deletion is disclosed as permanent and its completion
confirmed).

### T17 — Loss of user data / deletion not honoured (a trust threat)

**Threat.** A user leaves and the product keeps serving their data, or a user's content silently
disappears — either breaks the product's promise. Today a restart wipes everyone's data (a silent,
total loss), and there is no deletion to honour.

**Mitigated by.** `REQ-ID-012` (deletion removes the account and its personal data), `REQ-ID-013`
(deletion is honoured — the separately testable after-deletion guarantee), `REQ-ID-071` (the user's right
to erasure), and, for export before leaving, `REQ-ID-015`/`REQ-ID-016`. Durability of removal and of the
data itself across restart/backup is operational (`REQ-OPS-*`, coupled via `REQ-ID-013`/`REQ-ID-035`).

### T18 — Misleading privacy posture (users unaware what is public; inaccurate statement)

**Threat.** Users may not realise their chirps, profile, follows, and likes are public to anyone including
logged-out visitors, or may be told the product holds less than it does. Either misleads users about the
exposure of their personal data.

**Mitigated by.** `REQ-ID-073` (what is public is made clear to the user), `REQ-ID-068` (a privacy
statement of what personal data is held), `REQ-ID-069` (a terms-of-use statement), `REQ-ID-074` (the
privacy statement is accurate to what is actually held), and `REQ-ID-075` (block/mute relationships are
private to the user who set them — non-gating).

---

### Threats accepted or deferred for this release (recorded, not silently dropped)

- **Multi-factor authentication** is deferred, not excluded (`REQ-ID-038`): single-factor authentication
  meeting `REQ-ID-020..024` is the first bar for this consumer product; MFA is the first strengthening to
  add, with a recorded revisit condition (higher-sensitivity data, observed credential-stuffing, or the
  arrival of tenant accounts).
- **Federated sign-in (SSO)** is out of scope (`REQ-ID-039`): chirp has no organisations whose identity
  provider would govern sign-in; the revisit condition is the same one that would reintroduce tenancy.
- **A browsable, reviewable audit trail** of moderation/admin actions beyond the attribution baseline
  (`REQ-ID-050`) is a non-gating enhancement.
- **Mature trust-and-safety tooling** (moderation queues, automated classification, strikes, appeals) is
  out of scope for the first release; the bar is report → act → visible-effect plus baseline limits.
- **Clickjacking / UI-redress** of the product's own controls is noted as a lower-severity browser threat
  whose mitigation (frame-protection) is a `software`/`REQ-OPS-*` hardening item; it is accepted for the
  first bar as lower-leverage than T14/T15 given the actions it could redress are already step-up-gated for
  the destructive ones (`REQ-ID-025`).
- **Regulatory certification** against a named legal regime is an engagement/legal decision, not a design
  requirement; the product owes the *substance* (a truthful privacy/terms surface and real
  access/export/erasure rights), which is required (`REQ-ID-068..071`).

### Threat → mitigation coverage

| Threat | Boundary | Mitigating REQ-ID (or disposition) |
|--------|----------|-------------------------------------|
| T1 Account takeover by handle | Authentication | 020, 021, 001, 005, 041, 034 |
| T2 Credential guessing / stuffing | Authentication | 023, 022, 027 (thresholds → OPS via 065/066) |
| T3 Session theft / hijacking | Session | 027, 028, 031, 032, 036, 033 |
| T4 Session fixation | Session | 033 |
| T5 Recovery as bypass | Authentication/Recovery | 007, 037, 008, 002, 024, 032 |
| T6 Act on others' resources (IDOR/API bypass) | Authorization | 040, 042, 041, 053, 055, 056, 057 |
| T7 Abuse of elevated authority | Authorization | 045, 049, 050, 051, 072, 024 |
| T8 Account enumeration | Information disclosure | 022, 007 |
| T9 Credential disclosure | Information disclosure | 024, 049, 072, 017 |
| T10 Flooding / resource exhaustion | Availability | 065, 066, 067 (thresholds → OPS) |
| T11 Harassment / abusive content | Trust & safety | 060, 061, 062, 064, 063, 047, 052 |
| T12 Silent session loss / false success | Integrity of state | 029, 030, 035 |
| T13 Interception / tampering in transit | Network | 027 (transport "how" → OPS/software) |
| T14 Cross-site request forgery | Session / browser | **080** (+ 053, 056) |
| T15 Stored injection (stored XSS) | Content rendering | **081** |
| T16 Destructive action from live session | Session / integrity | 025, 032, 014 |
| T17 Data loss / deletion not honoured | Data integrity | 012, 013, 071, 015, 016 (durability → OPS) |
| T18 Misleading privacy posture | Privacy | 073, 068, 069, 074, 075 |
| MFA | Authentication | Deferred — 038 (recorded) |
| SSO | Authentication | Out of scope — 039 (recorded) |
| Rich audit trail | Authorization | Non-gating — beyond 050 (recorded) |
| Clickjacking / UI-redress | Browser | Accepted for first bar (recorded above) |

Every threat grounded in chirp's real surface names at least one mitigating `REQ-ID-*`, or is recorded as
accepted/deferred/out-of-scope with a reason.

---

---

## requirements_and_acceptance_criteria

This section consolidates the whole `REQ-ID-*` register for the identity, security and trust surface. Each
requirement's **full statement and acceptance criteria** live in its owning section above; this section
(a) states in full the two **net-new, threat-derived** requirements the threat model surfaced that no
spoke owns (`REQ-ID-080`, `REQ-ID-081`), (b) resolves every **cross-section coupling** into an explicit
`REQ-ID ↔ REQ-ID` reference, and (c) gives the **consolidated register** with each requirement's release
gating and its `REQ-PROD-*` trace. Every requirement in this register has a statement, testable acceptance
criteria (checkable by someone who never met the author), a `release_gating` flag, and a trace to at least
one `REQ-PROD-*`.

### Net-new, threat-derived requirements (owned by this section)

#### REQ-ID-080 — Mutating actions cannot be forged from another site (CSRF resistance)
- **Statement:** A state-changing action must originate from the product's own authenticated client acting
  on the user's behalf. A third-party website that a signed-in user merely visits must not be able to cause
  the user's browser to perform an authenticated mutation (post, follow, like, profile edit, settings
  change, account deletion, export) without the user's intent.
- **Acceptance criteria:** A page hosted on an origin other than chirp's, loaded in the browser of a user
  with a valid chirp session, cannot cause any authenticated mutation to take effect on that user's
  account; an equivalent action performed through chirp's own client succeeds. The protection is enforced
  on the server, so it cannot be bypassed by crafting the request directly. A reviewer can demonstrate that
  a cross-origin forged mutation attempt is refused while the legitimate in-product action succeeds.
- **Release gating:** true — chirp is a browser product with authenticated mutations exposed on the public
  internet; forged cross-origin mutations are a standard, high-impact threat (T14) that a releasable
  version must resist.
- **Traceability:** REQ-PROD-004, REQ-PROD-006; threat T14. Composes with REQ-ID-053 and REQ-ID-056
  (server-side per-action authorization). The carrier/mechanism (token placement, same-site policy,
  anti-forgery token) is a `software`-domain "how"; this fixes the property.

#### REQ-ID-081 — User-submitted content cannot execute as code in another user's browser (stored-injection resistance)
- **Statement:** Content a user submits — chirp text, display name, bio — is treated as data, not code.
  When it is shown to any other user, it cannot execute script in that user's browser or break out of its
  intended display context.
- **Acceptance criteria:** A chirp or profile field containing markup or script-like content (for example,
  an embedded `<script>` or an event-handler attribute) is rendered to other users as inert text and does
  not execute, on the timeline, on the author's profile, and in any thread or listing that shows it. A
  reviewer can post such content from one account and confirm, from a second account viewing it, that it
  renders as literal text and runs nothing.
- **Release gating:** true — the shared timeline renders attacker-controlled content to every reader;
  stored script execution (T15) is the highest-leverage attack on a public posting product and a
  releasable version must prevent it.
- **Traceability:** REQ-PROD-016 (compose/post), REQ-PROD-013 (public reading), REQ-PROD-029 (safety of
  the public surface); threat T15. The escaping/sanitisation/CSP mechanism is a `software`/`REQ-UX-*`
  "how"; this fixes the property.

### Cross-section coupling resolution

The four concern sections are parts of one design and share a real seam. Each coupling declared for this
subtree is resolved below into an explicit `REQ-ID ↔ REQ-ID` cross-reference, so no surface silently drifts
from its counterpart.

| # | Coupling | Owning facets | Resolved cross-reference |
|---|----------|---------------|--------------------------|
| C1 | Account recovery must not become an authentication bypass | recovery *lifecycle* (accounts) ↔ what recovery must *re-prove/invalidate* (auth) | **REQ-ID-006, REQ-ID-007, REQ-ID-008 ↔ REQ-ID-037**, with REQ-ID-002 (factor established at creation), REQ-ID-024 (never disclose old credential), REQ-ID-032 (invalidate pre-existing sessions) |
| C2 | Account deletion: lifecycle semantics ↔ who-may-delete authority | deletion *semantics/guarantee* (accounts) ↔ delete *authority* (authz) | **REQ-ID-010, REQ-ID-012, REQ-ID-013 ↔ REQ-ID-043** |
| C3 | Data export/erasure: mechanism ↔ data-right (↔ authority) | export/erasure *mechanism* (accounts) ↔ *data-right* (abuse/privacy) ↔ *authority* (authz) | **REQ-ID-015, REQ-ID-017 ↔ REQ-ID-070**; **REQ-ID-012 ↔ REQ-ID-071**; authorised by **REQ-ID-044** |
| C4 | Moderation: authority to act ↔ user-facing capability + visible effect | moderator *authority* (authz) ↔ report/block *capability* and visible *effect* (abuse) | **REQ-ID-047, REQ-ID-052 ↔ REQ-ID-060, REQ-ID-063** |
| C5 | Admin/support account lookup: authority ↔ privacy limit | admin/support *authority* (authz) ↔ *privacy limit* on what support sees (abuse/privacy) | **REQ-ID-048, REQ-ID-049 ↔ REQ-ID-072** |
| C6 | Public-internet session safety depends on per-action authorization | *session safety* (auth) ↔ *per-action authorization* (authz) | **REQ-ID-034 ↔ REQ-ID-053, REQ-ID-056** |

A lighter cross-area dependency is also recorded for traceability: credential-guessing resistance
(`REQ-ID-023`) and the baseline-limit requirements (`REQ-ID-065`/`REQ-ID-066`/`REQ-ID-067`) state the
*requirement*, while their enforcement thresholds and mechanism belong to operational readiness
(`REQ-OPS-*`, `limits_and_quotas`); and durable session continuity / durable deletion
(`REQ-ID-035`/`REQ-ID-013`) depend on durable data owned by operational readiness and domain design.

### Consolidated register

`G` = release_gating. Full statement and acceptance criteria are in the owning section named by the id's
range: 001–019 `accounts_and_lifecycle`, 020–039 `authentication_and_sessions`, 040–059 `authorization`,
060–079 `abuse_and_privacy_basics`, 080–099 `threat_model`/this section.

| REQ-ID | Requirement (short) | G | Traces to REQ-PROD-* |
|--------|---------------------|---|----------------------|
| REQ-ID-001 | Account created with an ownership credential | true | 001, 004 |
| REQ-ID-002 | A recovery factor is established at creation | true | 004, 001 |
| REQ-ID-003 | Handle validity enforced at creation | true | 002 |
| REQ-ID-004 | Handle uniqueness enforced; taken handle surfaced | true | 003 |
| REQ-ID-005 | Creation yields one complete owned account; credential is the ownership anchor | true | 001, 004 |
| REQ-ID-006 | A locked-out owner has a path back to the same account | true | 007, 004 |
| REQ-ID-007 | Recovery is not an authentication bypass | true | 007, 004 |
| REQ-ID-008 | Recovery restores access and supersedes what was lost; failed attempt changes nothing | true | 007, 004 |
| REQ-ID-009 | Recovery is honestly observable from the user's side | true | 007, 006 |
| REQ-ID-010 | Self-service account deletion from within the product | true | 010, 009 |
| REQ-ID-011 | Confirmation of consequences before deletion | true | 010 |
| REQ-ID-012 | Deletion removes the account and its personal data (removal scope) | true | 010, 012 |
| REQ-ID-013 | Deletion is honoured (after-deletion guarantee) | true | 012, 010 |
| REQ-ID-014 | Deletion disclosed as permanent; completion confirmed | true | 010 |
| REQ-ID-015 | A user can export their own data, and it is complete | true | 011, 009 |
| REQ-ID-016 | The export is portable, documented, machine-readable | true | 011 |
| REQ-ID-017 | An export returns only the requesting user's own data | true | 011 |
| REQ-ID-020 | Signing in requires proof of a secret only the owner holds | true | 004 |
| REQ-ID-021 | A credential is verified against the account on every sign-in | true | 004 |
| REQ-ID-022 | Authentication failure is uniform and non-informative | true | 004 |
| REQ-ID-023 | Credentials resist guessing; online guessing is throttled | true | 004 |
| REQ-ID-024 | A credential is never revealed, displayed, or logged | true | 004 |
| REQ-ID-025 | Destructive/identity-changing actions require fresh proof | true | 004 |
| REQ-ID-026 | Authentication establishes a session; credential not re-sent per action | true | 004, 006 |
| REQ-ID-027 | The session secret is confidential, unguessable, protected in transit | true | 006 |
| REQ-ID-028 | Sessions have a bounded lifetime | true | 006 |
| REQ-ID-029 | No silent sign-out: expiry/invalidation always surfaced | true | 006 |
| REQ-ID-030 | Signed-in vs logged-out state is always shown truthfully | true | 006 |
| REQ-ID-031 | Deliberate sign-out ends the session at the server | true | 005 |
| REQ-ID-032 | Credential change and recovery invalidate existing sessions | true | 005 |
| REQ-ID-033 | The session issued at authentication resists fixation | true | 004, 006 |
| REQ-ID-034 | A session acts only as its account; every account action requires one | true | 004, 006 |
| REQ-ID-035 | Sessions survive normal operation, or end visibly — never silently | true | 006 |
| REQ-ID-036 | A user can see and end their own active sessions | false | 005, 006 |
| REQ-ID-037 | Recovery re-proves control and is not an authentication bypass | true | 007, 004 |
| REQ-ID-038 | Multi-factor authentication: deferred, not precluded | false | 004 |
| REQ-ID-039 | Single sign-on / federated identity: out of scope | false | 004 |
| REQ-ID-040 | A user may act only on resources they own | true | 004, 010 |
| REQ-ID-041 | Every action attributed to, and authorized against, the actor's own identity | true | 004 |
| REQ-ID-042 | A user manages only their own social graph and likes | true | 004 |
| REQ-ID-043 | Only the account owner (or an authorized admin) may delete an account | true | 010, 012 |
| REQ-ID-044 | Only the owner (or an authorized admin on their behalf) may export an account's data | true | 011 |
| REQ-ID-045 | Holding an account grants ownership-scoped authority only | true | 004, 033 |
| REQ-ID-046 | Named elevated authorities are defined | true | 028, 033 |
| REQ-ID-047 | Moderator authority over abusive content and accounts | true | 028 |
| REQ-ID-048 | Operator/administrator support-lookup and account-administration authority | true | 033 |
| REQ-ID-049 | Elevated authority is least-privilege: no impersonation/over-reach/credential access | true | 004, 028, 033 |
| REQ-ID-050 | Elevated actions are attributable to an identified elevated principal | true | 028, 033 |
| REQ-ID-051 | Elevated authority is explicitly assigned, revocable, and minimal | true | 033 |
| REQ-ID-052 | An account acted on by moderation loses authority consistent with the action | true | 028, 012 |
| REQ-ID-053 | Every mutating action is authorized on the server and cannot be bypassed | true | 004, 013 |
| REQ-ID-054 | Public reads are explicitly allowed without a session | true | 013 |
| REQ-ID-055 | Default-deny for mutations and unrecognised actions | true | 004, 013 |
| REQ-ID-056 | A verified identity is necessary but not sufficient for a mutation | true | 004 |
| REQ-ID-057 | Authorization is consistent across every entry point | true | 004, 013 |
| REQ-ID-060 | Report a chirp or an account | true | 027 |
| REQ-ID-061 | Block an account | true | 027 |
| REQ-ID-062 | Mute an account | false | 027 |
| REQ-ID-063 | Moderation action has a visible effect | true | 028 |
| REQ-ID-064 | Abuse controls are discoverable and confirm the action | true | 027 |
| REQ-ID-065 | Baseline limit on account creation | true | 027, 028 |
| REQ-ID-066 | Baseline limit on posting and high-volume actions | true | 027, 028 |
| REQ-ID-067 | Limits are enforced safely and communicated, never silent | true | 027, 028 |
| REQ-ID-068 | A privacy statement of what personal data is held | true | 029 |
| REQ-ID-069 | A terms-of-use statement | true | 029 |
| REQ-ID-070 | Right of access and export to one's own data | true | 011 |
| REQ-ID-071 | Right to erasure of one's personal data | true | 012 |
| REQ-ID-072 | Privacy limit on the admin/support view (data minimisation) | true | 029, 033 |
| REQ-ID-073 | What is public is clear to the user | true | 029, 013 |
| REQ-ID-074 | The privacy statement is accurate to what is actually held | true | 029 |
| REQ-ID-075 | Block and mute relationships are private to the user who set them | false | 027, 029 |
| REQ-ID-080 | Mutating actions cannot be forged from another site (CSRF resistance) | true | 004, 006 |
| REQ-ID-081 | User-submitted content cannot execute as code in another user's browser | true | 016, 013, 029 |

**Release-gating count:** 68 of 73 requirements are release-gating. The five non-gating requirements are
recorded, honest cuts a first releasable version may ship without: REQ-ID-036 (self-service session list —
credential change already cuts off other access), REQ-ID-038 (MFA deferred), REQ-ID-039 (SSO out of scope),
REQ-ID-062 (mute — block satisfies the gating exposure-reduction need), and REQ-ID-075 (block/mute privacy
— satisfied by not building a disclosing surface).

### Product-requirement coverage

Every identity/trust-related `REQ-PROD-*` is carried by at least one `REQ-ID-*` above: REQ-PROD-001
(001, 005), 002 (003), 003 (004), 004 (020–025, 033, 034, 040–041, 045, 049, 053–057, 080), 005 (031,
032, 036), 006 (026–030, 033–035, 080), 007 (006–009, 037), 009 (010, 015), 010 (010–014, 040, 043),
011 (015–017, 044, 070), 012 (012, 013, 052, 071), 013 (034, 054, 057, 073, 081), 016 (081), 027
(060–067, 075), 028 (063, 065–067, 046–052), 029 (068–075, 081), 033 (046, 048–051, 072). Requirements
whose *mechanism* lands in another area (REQ-UX-*, REQ-DOM-*, REQ-OPS-*, or `software`) are referenced by
prose at the point of coupling, not re-minted here.
