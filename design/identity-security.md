# Identity, security & trust

This document designs the identity, security, and trust surface a releasable, publicly exposed
chirp needs, stated as requirements (`REQ-ID-*`) and the threat model those requirements answer. It
is held to one bar: **real authentication, not a stub; sessions safe to expose on the public
internet; a clear authority boundary; and the privacy, abuse-resistance, and data-rights floor a
product that holds people's accounts is expected to have.** It is grounded in the product as it
exists in `Nousix-LLC/nswe-demo` at commit `c805525`.

It states *what must be true* from the account holder's, the operator's, and the attacker's side. It
is requirements, not implementation: where a topic has an engineering "how" — password hashing,
token formats, session storage, rate-limiter internals, transport configuration — the design fixes
the externally observable property that must hold and names the code or operational area that owns
the build. Every requirement traces to at least one product requirement (`REQ-PROD-*`) and carries a
testable acceptance criterion a stranger could check against the live product.

The document has six sections:

1. **Accounts and lifecycle** — the account object and its full life, from sign-up to deletion.
2. **Authentication and sessions** — real auth and a session safe to expose publicly.
3. **Authorization** — who may see and change what.
4. **Abuse and privacy basics** — reporting, controls, and the data-rights floor.
5. **Threat model** — the threats chirp's public surface implies and what the design answers.
6. **Requirements and acceptance criteria** — the consolidated `REQ-ID-*` register, every entry with
   its full fields.

The two findings that most directly break a releasable-product assumption, and which this design
closes, are **handle-only authentication** (signing in is typing a handle; anyone who knows a handle
can act as that account) and **no session hygiene** (no server-side sign-out, no expiry, no
recovery, no deletion, no export).

---

## Accounts and lifecycle

This section designs chirp's **account object** and the full life of an account — how a person
creates one, manages it, and ends it, and how they and their data leave. It states what must be true
from the account holder's side, to the bar of a releasable, publicly exposed, account-based consumer
product.

Two boundaries matter throughout. First, the **account depends on authentication but does not own
it.** Sign-up establishes a credential, recovery re-establishes one, and both end sessions — but the
credential, session, and sign-out *mechanisms* are the authentication-and-sessions area's
requirements (`REQ-ID-030..059`). This section states the lifecycle events from the account's side
and names that area for the mechanism. Second, the account's **public/private field boundary** is
defined here as part of the account object, but who is *allowed* to read or change each field is the
authorization area's to enforce (`REQ-ID-060..079`); the retention and erasure baseline deletion
relies on is the abuse-and-privacy area's (`REQ-ID-080..099`).

The account lifecycle today is almost entirely absent. At `c805525`, signing in is typing a handle,
there is no credential, no verified contact, no recovery, no settings, no profile edit, no export,
and no deletion. Each requirement in this design closes one part of that gap and says how a stranger
would check it on the live product.

### The account object

An **account** is the thing a person owns and acts through. It comprises:

- a **handle** — the unique public identifier, up to 15 letters, digits, or underscores, that others
  follow, reply to, and see on every chirp;
- a **display name** and an optional **bio** — public, person-chosen presentation;
- a **verified point of contact** — private, used only for recovery and notifications;
- a **credential** — private, owned by the authentication-and-sessions area, that proves the account
  is the person's own;
- the person's **content and graph** — their chirps, who they follow, and what they liked — which
  the domain design (`REQ-DOM-*`) persists and this lifecycle governs on export and deletion.

The split between public fields (handle, display name, bio, public chirps, follower/following
counts) and private fields (contact, credential, session data) is the account's structural privacy
boundary. Defining the boundary is this section's job; enforcing who may read or write across it is
the authorization area's.

### Sign-up: establishing a real account

A releasable product cannot let one person claim another by typing a name. Sign-up must create an
account whose owner holds a secret, collect only what the account needs, and capture a contact the
product has confirmed the person controls — the basis for ever letting them back in.

### Managing the account: profile, settings, handle

Once in, a person expects to change how they appear and to reach the handful of account-level choices
the lifecycle implies — their contact, their active sessions, their notification preferences — from
one place. The handle is the exception: because others follow and address an account by its handle,
whether it can change is a stated product decision, not an open surface.

### Recovery: getting back in

A person who loses their credential must be able to return without another person acting for them,
and only through the contact the product verified at sign-up. Recovery is the account-side event; the
credential it re-establishes and the sessions it ends belong to the authentication-and-sessions area.

### Leaving: export and deletion

The end of the relationship is part of the product. A person must be able to take their own data out
in a form they can open elsewhere, and to delete their account with a defined, confirmed outcome for
their content and their handle. A product a person cannot leave cleanly is one they do not fully
trust in the first place.

---

## Authentication and sessions

Chirp today lets a person sign in by typing a handle: the server opens a session for whoever presents
it. There is no secret, no proof of ownership, and no verification — anyone who knows a handle can
sign in as that account. That breaks the premise the whole product rests on, that a chirp attributed
to a person was posted by that person. The session the server issues has no expiry, no server-side
invalidation, and no sign-out; the app can forget its copy of the token, but the session stays
usable. On a shared device that leaves the account open to the next person.

This section specifies what real authentication and a session safe to expose on the public internet
must guarantee for chirp, from the user's and the attacker's side. It states requirements, not
mechanisms: it does not choose a password-hashing algorithm, a token format, or a session store.
Where a requirement implies an engineering "how" — hashing, transport configuration, the durable
store that holds session state, the enforcement of guess-rate limits — the requirement names the
property that must hold and leaves the build to the code and operational-readiness areas, which it
points to by name.

Two decisions set the boundary of this area. Chirp is a single public product of individual accounts,
not a tenant platform, so it carries no enterprise single-sign-on obligation. And the account object,
its sign-up, and its recovery flow are the accounts-and-lifecycle area's to own; this area owns the
**mechanism requirements** — what proving identity and holding a safe session must guarantee — and
references that sibling where sign-up or recovery establishes or resets the secret.

The credential a person proves with is referred to here as their **secret**. The requirements do not
mandate a password specifically; they mandate a secret only the owner holds, and the properties any
such secret and its session must satisfy. Multi-factor authentication (REQ-ID-050) and third-party /
social sign-in (REQ-ID-051) are recorded out of scope for the release bar, each with its reason; the
design must not foreclose adding either later.

---

## Authorization

Authorization is the question of **what each actor may see and change**, once authentication has
established *who the actor is*. Proving identity — secrets, sessions, recovery — is the sibling
authentication-and-sessions concern; this section takes a proven identity as given and specifies what
that identity is permitted to do, and what every viewer (including one with no identity at all) is
permitted to read.

chirp's authority model rests on one premise the whole product depends on: **a person acts only as
themselves**. A chirp carries the weight of *this person said this* only if no one else could have
posted, edited, or deleted it as them, and only if the private details behind an account stay with
its owner. The model has three parts, and they are one model seen from three sides:

- **The read boundary** — which fields any viewer may read without an account, which require a
  session, and which belong to the account owner alone.
- **Actor-scoped action authority** — a signed-in person may create, change, and remove only their
  own account and their own content, always attributed to themselves, never acting as another
  account.
- **Operator authority** — the separate, least-privilege, auditable authority the supported
  administrative surface needs to assist a user, remove content, and action an abusive account, held
  by provisioned operators rather than by any ordinary account.

Two framing rules run through all three parts. First, the model is **default-deny**: an action or a
read that is not explicitly permitted is refused, so a surface added later does not silently inherit
authority no one granted it. Second, this section owns the **authority boundary** of the
public-versus-private field split — *who* may read or write each field — while the data-protection
rules for those same private fields (what is collected, how long it is kept, how it is erased on
deletion) belong to the abuse-and-privacy concern. The two are coordinated by naming the same field
set, not by restating each other's requirements.

### The field model this section authorizes against

The authorization requirements refer to a single field model, stated once here:

- **Public fields** (readable by any viewer, with or without a session): a user's handle, display
  name, bio, follower count and following count, and that user's public chirps with their reply and
  like counts. This is the set `REQ-PROD-007` and `REQ-PROD-024` name as public.
- **Owner-only fields** (readable and changeable only by the authenticated account owner): the
  verified contact, the authentication credential, the list of active sessions and devices,
  notification preferences, account settings, and any data-export payload.
- **Owner-scoped views** (composed for, and served to, one authenticated owner): the home timeline of
  the owner's own and followed accounts' chirps, and the account-management views.

Any field not listed as public is non-public by default. The *why* and the *lifecycle* of the
owner-only fields — minimization, retention, erasure on account deletion — are the abuse-and-privacy
concern's to state; this section fixes only who may read and write them.

### State today (grounded in `Nousix-LLC/nswe-demo` @ `c805525`)

A session identifies an actor, and actions run as that actor; a user's public profile is served
without a session. There is no model for what an actor may *not* do beyond its own account, no stated
public-versus-owner-only field boundary, and no operator authority at all — no separation between an
ordinary account and whoever administers the service, and no supported administrative action. The
requirements specify the model the product is missing; the enforcement mechanism (how a session is
checked, how a field is withheld, how an operator is provisioned) is deferred to the `chirp-server`
code work in the `software` domain.

---

## Abuse and privacy basics

chirp is a public space where strangers post at each other. That is its appeal and its risk: the same
open write surface that lets anyone reply to anyone also lets anyone harass, flood, or dox anyone. A
product that invites that interaction without giving people a way to report it, rules to report it
against, a way to stop an account that is bothering them, and a limit on how fast one actor can act,
is not finished — it has shipped the interaction and withheld the safety that makes the interaction
survivable. This section specifies that floor as requirements, together with the privacy and
data-rights surface a product that holds people's contact details, credentials, and posts is expected
to have.

The scope here is deliberately the minimum a releasable public microblog needs, not a full
trust-and-safety platform. Two concerns interlock. **Abuse resistance** is what lets a person who is
being targeted act: report the content, point at the rule it breaks, block or mute the account, and
rely on the service not being trivially floodable. **Privacy and data rights** is what the product
owes every account whether or not it is ever abused: collect only what the account needs, never leak
a person's private fields to other users, erase their data when they leave, and let them see, take,
and remove their own data. The two meet at a single seam — what a report or a public view is allowed
to expose about an account is bounded by the private-field rule.

These are requirements, not mechanisms. How a limiter is implemented, how reports are queued, how
erasure is executed against a datastore, and what retention windows are configured are the concern of
later code work and of the operational-readiness area. Three handoffs to sibling areas are
coordinated rather than duplicated: the public/private field boundary (*who* may read which field) is
the authorization area's, while this section owns the data-protection guarantee that a person's
contact, credential, and session data never reach another user; acting on a report (the operator
surface) is the authorization area's, while this section requires only that a report is captured and
routed to it; and the deletion outcome for content and handle is the accounts-and-lifecycle area's,
while this section requires that deletion erases the private data and leaves no usable session.

Rate and volume limits are stated here as an identity-and-trust requirement because they are a first
line against abuse, but their enforcement, quota values, and operational tuning belong to the
operational-readiness area's limits-and-quotas surface (`REQ-OPS-*`); the requirement traces to the
product limit requirement and names that area, and does not define the enforcement.

---

## Threat model

This threat model walks chirp's public, internet-facing surface against the six STRIDE categories —
spoofing, tampering, repudiation, information disclosure, denial of service, elevation of privilege —
plus the content-abuse category a public social product cannot omit. It is derived from the four
areas above: each threat names what the design answers (the `REQ-ID-*` that mitigates it) or records
a residual risk and why it is acceptable for the release bar.

### The surface and the assets

chirp is a single-page web app delivered to the browser and a REST backend, operated as a hosted
service reachable on the open internet. Anyone, with or without an account, can read public profiles
and public chirps. A signed-in person holds a **secret** that proves ownership and carries a
**session credential** for subsequent actions. The product holds, per account, private fields a
viewer must never see: the verified contact, the credential, and session data.

The assets an attacker would go after:

- **Account control** — the ability to act as an account (its secret, and the sessions derived from
  it). Losing this breaks *this person said this*.
- **Attribution integrity** — that the author recorded on a chirp, reply, like, or follow is the
  actor who performed it.
- **Private fields** — the verified contact, the credential, and session data, held server-side.
- **Service availability** — the product being reachable and responsive for ordinary users.

### Spoofing an identity

**Threat.** Someone signs in as, or otherwise acts as, an account that is not theirs — the exact gap
in the handle-only product today, where knowing a handle is enough to receive a session.

**What the design answers.** Sign-up establishes a real secret and an account is not usable until one
is set (REQ-ID-002, REQ-ID-030); a session is granted only on the correct secret, with no
handle-only or debug bypass (REQ-ID-031, REQ-ID-032); the secret meets a stated strength expectation
and high-volume guessing is slowed and leaks no progress (REQ-ID-034, REQ-ID-035); the secret is
never stored or returned in a recoverable form (REQ-ID-036); a fresh session identity is established
at sign-in, defeating fixation (REQ-ID-043); session validity is server-authoritative and cannot be
forged or extended by the client (REQ-ID-040); and the recovery path resists non-owners and does not
enumerate accounts (REQ-ID-015), re-establishing the secret only after proven contact control
(REQ-ID-014) and ending sessions that predate the reset (REQ-ID-016, REQ-ID-046).

**Residual risk (accepted — R1).** With MFA out of scope for the bar (REQ-ID-050), a phished or
guessed single secret remains a path to an account. The anti-guessing (REQ-ID-035), bounded and
revocable sessions (REQ-ID-038, REQ-ID-039, REQ-ID-045), and recovery-ends-old-sessions
(REQ-ID-046) requirements bound the damage and give a path to eject an attacker. Accepted for a
public microblog of individual accounts with no regulatory obligation; revisit if the account's
sensitivity grows. The design does not foreclose adding MFA later.

### Tampering with content or attribution

**Threat.** An actor alters content, or makes an action appear to come from another account —
undermining attribution, which is chirp's core value.

**What the design answers.** The author on a chirp, reply, like, or follow is set by the product from
the authenticated session, not from actor-supplied input, and cannot be forged (REQ-ID-068); social
write actions are performed as the signed-in actor and cannot be directed under another identity
(REQ-ID-065); a person may change and remove only their own account and content (REQ-ID-066,
REQ-ID-067); session validity is server-authoritative and a tampered session credential yields
nothing (REQ-ID-040); and operator content removal and account actioning apply the product's defined
outcome and are recorded (REQ-ID-074).

### Repudiation

**Threat.** An actor denies having taken an action, or an operator's administrative action cannot be
attributed after the fact.

**What the design answers.** Each session is individually identifiable (REQ-ID-045); each operator
action records which operator performed it, what was done, the target, and when, in a retained record
not editable through ordinary surfaces (REQ-ID-073); and a filed report is recorded with who reported
what, against whom, and which rule (REQ-ID-082), giving the operator a record to act on and a pattern
across reports.

**Residual risk (accepted).** Ordinary-user actions are not cryptographically signed, so strong
non-repudiation of an individual post is not provided. For a public microblog this is proportionate;
server-authoritative attribution (REQ-ID-068) and per-session identity (REQ-ID-045) are the bar.

### Information disclosure of private fields

**Threat.** An account's private fields — verified contact, credential, session data — leak to
another user, to a public or unauthenticated response, to the report surface, or to an observer on
the network between the person and the product.

**What the design answers.** Unauthenticated reads are limited to the public fields (REQ-ID-061);
owner-only fields and owner-scoped views are returned only to the authenticated owner (REQ-ID-062,
REQ-ID-063); a refused action or read discloses no owner-only field and nothing beyond the public
surface (REQ-ID-069); the secret is never returned in a recoverable form to anyone, including the
owner and operators (REQ-ID-036), and operators cannot read the credential (REQ-ID-072); the session
credential is not placed in URLs, logs, or script-readable storage where it can be stolen
(REQ-ID-042); the product collects only what each account needs (REQ-ID-086); private fields never
reach another user, a public response, or the report/block/mute flows (REQ-ID-087); an export
excludes secrets and other accounts' private data (REQ-ID-019) and is scoped to the requester's own
account (REQ-ID-018); deletion erases private data (REQ-ID-023, REQ-ID-088); and a stated retention
posture bounds how long personal data is kept (REQ-ID-089).

**New requirement this threat drives — REQ-ID-100 (transport confidentiality, product-wide).** The
authentication area protects the session credential in transit (REQ-ID-041), but private fields reach
the wire in more places than that one credential: the secret during sign-up and recovery, the
verified contact, and the export payload. No sibling area owns a product-wide transport guarantee,
and REQ-ID-087 assumes private fields are protected end to end without owning that property. REQ-ID-100
closes it: every private field is exchanged only over an encrypted transport, generalizing REQ-ID-041
to the whole private surface.

### Denial of service

**Threat.** An attacker makes the product unavailable or degrades it for ordinary users — by
credential-guessing load, by flooding writes, or by creating accounts automatically at scale.

**What the design answers.** Repeated sign-in guessing is slowed, limited, or made to cost the
attacker (REQ-ID-035); basic rate and volume limits on writes and requests stop one actor trivially
flooding others or the service (REQ-ID-085).

**New requirement this threat drives — REQ-ID-101 (anti-automation / bot resistance).** Rate limits
(REQ-ID-085) bound how fast one actor acts, and anti-guessing (REQ-ID-035) bounds sign-in attempts,
but neither resists a bot creating many accounts to spread load across them or to flood from fresh
identities — a standard attack on a public sign-up surface. REQ-ID-101 requires a basic measure that
raises the cost of automated account creation and automated abuse, coordinated with the
operational-readiness limits surface for the enforcement depth.

**Residual risk (accepted — R2).** The depth of denial-of-service protection — concrete thresholds,
lockout windows, per-source limits, and infrastructure-level protection against volumetric attacks —
is the operational-readiness area's (`REQ-OPS-*` limits-and-quotas) and the availability target
(`REQ-PROD-040`). This design states the identity-and-trust properties (REQ-ID-035, REQ-ID-085,
REQ-ID-101) as the first line and names the operational area for the enforcement; full volumetric DoS
resistance is not owned here and is a known, bounded residual carried by that area.

### Elevation of privilege

**Threat.** One session acts beyond the authority of its account, or an ordinary account obtains
operator authority.

**What the design answers.** Every state-changing action requires a valid authenticated session and
is refused by default otherwise (REQ-ID-060); a person acts only as themselves and cannot act as or
on behalf of another account (REQ-ID-064); session validity and scope are server-authoritative and a
client cannot alter whose session it is (REQ-ID-040); operator authority is a separate, provisioned
role that no account gains through ordinary sign-up or sign-in (REQ-ID-070), is limited to the
supported least-privilege action set (REQ-ID-071), and cannot impersonate users or read credentials
(REQ-ID-072).

### Content abuse and harassment (public-surface specific)

**Threat.** The open write surface is used to harass, target, or dox a person — a threat specific to a
public social product rather than a classic STRIDE letter, but one a releasable public microblog must
answer.

**What the design answers.** A person can report a chirp or an account against stated rules
(REQ-ID-080, REQ-ID-081); the report is captured and routed to the authorized operator/support
surface with enough context to act (REQ-ID-082); operators can remove content and action an abusive
account under a defined outcome (REQ-ID-071, REQ-ID-074); and a person can block or mute an account to
stop unwanted interaction (REQ-ID-083, REQ-ID-084).

**Residual risk (accepted — R3).** Block and mute (REQ-ID-083, REQ-ID-084) are non-gating by the
product's own decision (`REQ-PROD-027`), so at the gated floor harassment mitigation rests on
reporting (REQ-ID-080), stated rules (REQ-ID-081), report routing (REQ-ID-082), and operator action
(REQ-ID-070–074), together with the rate/volume limits (REQ-ID-085). The release bar (`BAR-*`)
decides whether v1 gates one of block/mute; the threat is bounded by the reporting-and-operator floor
until it does.

### Accepted residual risks (summary)

| Id | Residual | Why acceptable for the bar | Bounded by |
|----|----------|----------------------------|------------|
| R1 | A phished or guessed single secret can reach an account (MFA out of scope). | A public microblog of individual accounts with no regulatory obligation; a well-formed single secret plus session and anti-guessing controls meets "real auth, not a stub." Not foreclosed later. | REQ-ID-035, 038, 039, 045, 046 |
| R2 | Full volumetric / infrastructure DoS resistance is not owned here. | Identity-and-trust states the first line; depth is the operational-readiness area's to set and tune. | REQ-ID-035, 085, 101; `REQ-OPS-*`, `REQ-PROD-040` |
| R3 | Block/mute non-gating, so the gated harassment floor is reporting + operator action. | The product requirement makes block/mute a reasoned fast-follow; reporting is the gated floor. The release bar may elect to gate one. | REQ-ID-080, 081, 082, 070–074, 085 |

---

## Requirements and acceptance criteria

The consolidated `REQ-ID-*` register: every requirement from the four areas above plus the two
threat-driven requirements this document mints, each with its full fields. Every requirement traces
to at least one `REQ-PROD-*` in the product-requirements register. `release_gating: false` entries
are in scope and expected but need not block a first release; each states its reason inline, and the
release-bar design (`BAR-*`) stages them.

### Accounts and lifecycle (REQ-ID-001 .. REQ-ID-029)

#### REQ-ID-001 — The account object and its public/private field boundary
- **statement**: An account comprises a public identity (handle, display name, optional bio) and private account data (verified contact, credential, session data), plus the person's content and follow graph. Public fields are readable by any viewer; private fields are never part of any other user's view of the account.
- **acceptance_criteria**: View any account's public profile with no session: handle, display name, bio, and follower/following counts appear. Inspect every product surface that returns another account to a viewer: none returns that account's contact, credential, or session data.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-007, REQ-PROD-024

#### REQ-ID-002 — Sign-up establishes a real credential
- **statement**: Creating an account establishes a secret the owner will later use to prove the account is theirs; a handle alone is never sufficient to act as the account. The credential mechanism is the authentication-and-sessions area's; from the account's side, an account does not exist in a usable state until such a credential is set.
- **acceptance_criteria**: Complete sign-up for a fresh handle; the account is created only once a credential is set. Then, knowing only the handle (not the credential), attempt to sign in as that account through any product surface: no session is granted.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-008

#### REQ-ID-003 — Handle format and uniqueness with field-level feedback
- **statement**: Sign-up accepts a handle only if it is well-formed (up to 15 letters, digits, or underscores) and not already taken, reporting a taken or ill-formed handle against the handle field so the person can correct it.
- **acceptance_criteria**: Submit a handle already in use; sign-up reports a conflict naming the handle. Submit a handle with a space, a disallowed character, or more than 15 characters; sign-up reports a field-level error on the handle. Submit a valid, unused handle; it is accepted.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-009

#### REQ-ID-004 — A verified point of contact captured at sign-up
- **statement**: Sign-up captures one point of contact for the account and records it as a private field, so recovery and important notifications have a basis. What kind of contact is collected and how it is protected in storage is a privacy/identity decision carried with abuse-and-privacy.
- **acceptance_criteria**: During sign-up, provide a contact; after sign-up the account holds that contact as a private field (never shown to other users), and recovery (REQ-ID-013) and notifications can reference it.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-012

#### REQ-ID-005 — The contact is confirmed before it is relied on
- **statement**: The product confirms the person controls the captured contact before using it for recovery or notifications; an unconfirmed contact is not a basis for regaining access.
- **acceptance_criteria**: Provide a contact at sign-up and do not complete its confirmation step; attempt recovery (REQ-ID-013) through that contact — it is not accepted as a recovery basis. Complete the confirmation step; the contact is then usable for recovery.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-012

#### REQ-ID-006 — Sign-up collects only what the account needs
- **statement**: Sign-up asks for a handle, a display name, an optional bio, the credential, and one point of contact — and nothing beyond what the account requires.
- **acceptance_criteria**: Inspect the sign-up surface: the only fields requested are handle, display name, optional bio, credential, and one contact. No additional personal data is required to create an account.
- **release_gating**: false — the specific gated collection requirements are REQ-ID-002/003/004; "nothing extraneous" is a data-minimization expectation stated here for the sign-up surface and coordinated with abuse-and-privacy, which owns the product-wide minimization baseline.
- **verification_method**: inspection
- **traces_to**: REQ-PROD-012, REQ-PROD-031

#### REQ-ID-007 — Edit display name and bio after sign-up
- **statement**: A signed-in person can change their display name and bio after sign-up, under the same format rules as at sign-up, and the change persists and shows on their public profile.
- **acceptance_criteria**: Signed in, change your display name and bio and save; reload your public profile — the new values show. Submit an invalid display name or bio; the product reports a field-level error and does not save it.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-016

#### REQ-ID-008 — An account settings surface
- **statement**: A signed-in person has one settings surface for their account-level choices — at minimum their verified contact, their active sessions/devices, and their notification preferences where notifications exist.
- **acceptance_criteria**: Signed in, open settings from the product's navigation; it presents, in one place, the verified contact, the active sessions/devices, and (where notifications exist) notification preferences.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-017

#### REQ-ID-009 — View and update the verified contact in settings
- **statement**: From settings, a person can see their current verified contact and change it; a changed contact is confirmed (REQ-ID-005) before it replaces the old one as the recovery/notification basis.
- **acceptance_criteria**: In settings, view the current contact. Enter a new contact; until its confirmation step is completed, recovery and notifications still use the prior confirmed contact. Complete confirmation; the new contact takes over.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-017, REQ-PROD-012

#### REQ-ID-010 — View and end active sessions/devices in settings
- **statement**: From settings, a person can see the sessions/devices currently signed in to their account and end any of them. Ending a session invalidates it; the invalidation mechanism is the authentication-and-sessions area's.
- **acceptance_criteria**: Signed in on two devices, open settings on one and view the active sessions; end the other session; the ended session can no longer act as the account (verify by attempting an action from it).
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-017, REQ-PROD-014

#### REQ-ID-011 — Notification preferences in settings
- **statement**: Where activity notifications exist, a person can set which categories they are notified about from the settings surface. The notification capability itself is carried by the notifications/abuse-privacy areas; this requirement places its preferences in account settings.
- **acceptance_criteria**: In settings, find notification preferences; turn one category off and confirm; notifications of that category stop. If notifications are not in the release, settings states that no preferences apply yet rather than showing an empty control.
- **release_gating**: false — activity notifications are a non-gating product requirement (REQ-PROD-029); the preference controls gate only when notifications ship. Recorded so the settings surface accounts for them rather than omitting the seam.
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-017, REQ-PROD-029

#### REQ-ID-012 — A stated handle-change policy
- **statement**: The product states whether and how a handle can change after sign-up and behaves consistently with that statement. For the first release, a handle is **fixed for the life of the account**: it cannot be changed from any product surface, and the product states this where a person would look for the option. The reason: the handle is the public identifier others follow and address, so a change path carries redirect, follower-continuity, and impersonation questions a first release states as deferred rather than answers.
- **acceptance_criteria**: Find the product's stated position on handle changes; it says the handle is fixed. Search every account surface for a handle-change action; none exists, matching the stated position.
- **release_gating**: false — this is a decision-and-consistency requirement (per REQ-PROD-018, non-gating); a first release may ship with handles fixed provided that position is stated in the product.
- **verification_method**: inspection
- **traces_to**: REQ-PROD-018

#### REQ-ID-013 — Self-service account recovery through the verified contact
- **statement**: A person who has lost their credential can regain access themselves, through the contact verified for the account, without another person acting on their behalf.
- **acceptance_criteria**: Having lost your credential, use a "can't get in?" path, prove control of the verified contact, and regain access to the account. A person who does not control that contact cannot complete recovery.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-013

#### REQ-ID-014 — Recovery re-establishes a credential, gated on proven contact control
- **statement**: Recovery lets the owner set a new credential for the account only after they have proven control of the verified contact. Setting the credential is the authentication-and-sessions area's mechanism; this requirement binds recovery to re-establishing owner-held access rather than granting a session without a new secret.
- **acceptance_criteria**: Complete the recovery path by proving control of the verified contact; you are prompted to set a new credential, and afterward you sign in with that new credential. Reaching the credential-reset step without proving contact control is not possible.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-013, REQ-PROD-012

#### REQ-ID-015 — Recovery resists non-owners and does not leak account existence
- **statement**: Recovery cannot be completed by anyone who does not control the verified contact, and the recovery surface does not reveal whether a given handle or contact has an account beyond what the product already makes public.
- **acceptance_criteria**: Attempt recovery for an account whose contact you do not control; it cannot be completed. Submit a handle or contact with no account to the recovery surface; the response does not disclose account existence in a way that enumerates accounts.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-013, REQ-PROD-011

#### REQ-ID-016 — Recovery and credential reset end existing sessions
- **statement**: Completing recovery (and any credential reset) ends the account's existing sessions, so a lost or compromised credential does not leave old sessions active. The session-invalidation mechanism is the authentication-and-sessions area's.
- **acceptance_criteria**: Sign in, then from another context recover the account and set a new credential; the original session can no longer act as the account.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-013, REQ-PROD-014

#### REQ-ID-017 — Self-service data export in a portable form
- **statement**: A signed-in person can obtain their own data — their chirps, their profile, and who they follow — themselves, in a form they can open outside chirp. The packaging format is deferred to code; the requirement is that the export is self-service and portable.
- **acceptance_criteria**: Signed in, request your data; you receive your posts, your profile, and your follow list as a file or download you can open outside chirp without special tooling.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-019

#### REQ-ID-018 — Export is scoped to the requester's own account
- **statement**: An export contains only the requesting account's own data; it never includes another account's data. The authorization enforcement of "own account only" is the authorization area's.
- **acceptance_criteria**: Export your data; inspect the result — it contains your chirps, profile, and follows and no other account's records.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-019, REQ-PROD-024

#### REQ-ID-019 — Export excludes secrets
- **statement**: An export never includes the account's credential or session data, and never includes another person's private contact (for example, contacts of accounts the person follows).
- **acceptance_criteria**: Inspect an export: it contains no credential and no session tokens, and no private contact data belonging to other accounts.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-019, REQ-PROD-024

#### REQ-ID-020 — Self-service account deletion
- **statement**: A signed-in person can delete their own account; afterward the account can no longer sign in, and the person's content is handled per the defined deletion outcome (REQ-ID-021).
- **acceptance_criteria**: Signed in, delete your account; afterward, attempting to sign in to that account fails, and the person's content has been handled as REQ-ID-021 states.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-020

#### REQ-ID-021 — A defined outcome for a deleted account's content and handle
- **statement**: The product defines, states, and applies what deletion does to the person's content and handle. For the first release the defined outcome is: the person's chirps are removed from public view; a reply by another person that pointed at a now-deleted chirp shows a "chirp unavailable" placeholder rather than breaking the conversation, and like/reply counts adjust accordingly; and the freed handle is **retired, not immediately re-registerable**, so a departed account cannot be impersonated by someone claiming its handle. The retention/erasure timing behind "removed" is coordinated with abuse-and-privacy.
- **acceptance_criteria**: Find the stated deletion outcome in the product. Delete an account whose chirps others replied to and liked; its chirps leave public view, replies to them show the unavailable placeholder, counts adjust, and the handle cannot be registered by a new sign-up (matching the stated outcome).
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-021

#### REQ-ID-022 — Deletion is confirmed and explains what survives
- **statement**: Account deletion is confirmed before it happens and tells the person what will and will not survive, including that export (REQ-ID-017) is their chance to keep a copy; a person can cancel at the confirmation and nothing is deleted.
- **acceptance_criteria**: Initiate deletion; a confirmation explains the consequences and points to export, and deletion proceeds only after explicit confirmation. Cancel at the confirmation; the account and all content remain intact.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-022

#### REQ-ID-023 — Deletion erases the account's private data
- **statement**: Deletion removes the account's private data — its verified contact, credential, and session records — so no private account data remains after deletion. The retention baseline and any legally required erasure window are coordinated with abuse-and-privacy.
- **acceptance_criteria**: Delete an account; afterward the account's contact, credential, and session records are no longer present on any product surface or in an export, consistent with the stated retention baseline.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-020, REQ-PROD-024

#### REQ-ID-024 — Deletion ends all active sessions
- **statement**: Deleting an account immediately ends all of that account's active sessions, so no session can continue to act as a deleted account. The session-invalidation mechanism is the authentication-and-sessions area's.
- **acceptance_criteria**: Signed in on two devices, delete the account from one; the other device's session can no longer act as the account.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-020, REQ-PROD-014

### Authentication and sessions (REQ-ID-030 .. REQ-ID-059)

#### REQ-ID-030 — Sign-up establishes a secret, not a handle alone
- **statement**: Creating an account establishes a secret that only the account owner holds and that is required to sign in afterward; a handle by itself never stands in for that secret. (The sign-up flow that collects and confirms the secret is owned by the accounts-and-lifecycle area; this requirement fixes what that flow must result in.)
- **acceptance_criteria**: After an account is created, signing in requires presenting the secret set at sign-up. Knowing only the handle, with no secret, does not produce a session.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-008, REQ-PROD-010

#### REQ-ID-031 — Sign-in only the account owner can pass
- **statement**: A session is granted only on presentation of the account's correct secret; presenting a known handle without the matching secret grants nothing.
- **acceptance_criteria**: Attempt to sign in to another account using only its handle (public information) and no secret, or a wrong secret; no session is granted and no authenticated action can be taken as that account. The legitimate owner, presenting the correct secret, is signed in.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-010

#### REQ-ID-032 — The secret is the sole gate to a session
- **statement**: There is no path to an authenticated session that bypasses verification of the account's secret — no debug route, no handle-only fallback, no default or shared secret.
- **acceptance_criteria**: Inspection of the sign-in surface and any alternate entry points finds no route that issues a session without verifying the presented secret against the account.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-010

#### REQ-ID-033 — Not-found and failed-secret are distinguishable without leaking who exists
- **statement**: A person can tell "no such account" from "that secret did not match," enough to distinguish a mistyped handle from a wrong secret, without the product revealing which accounts exist beyond what it already makes public. (Chirp profiles and handles are public, so confirming that a handle resolves to an account discloses nothing not already public; the signal must not go further — for example by revealing a private contact or whether a contact is registered.)
- **acceptance_criteria**: Sign in with an unknown handle; the result lets you conclude the account does not exist, so you can proceed to sign-up. Sign in with a known handle and a wrong secret; the result is a failed sign-in you can retry or recover from. Neither path exposes a private attribute (such as a recovery contact) of any account.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-011

#### REQ-ID-034 — The secret meets a stated strength expectation
- **statement**: The product requires a secret strong enough that it cannot be guessed by trying a handful of common values, and states that expectation to the person setting it. (The specific policy — minimum length, disallowed common values — is tuned by the experience and accounts areas; this requirement fixes that a stated, enforced minimum exists.)
- **acceptance_criteria**: Attempt to set a trivially weak secret (for example a short or well-known common value); it is rejected with guidance. A secret meeting the stated policy is accepted.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-008, REQ-PROD-010

#### REQ-ID-035 — Repeated sign-in guessing is slowed and does not leak progress
- **statement**: Repeated failed sign-in attempts against an account or from a source are slowed, limited, or otherwise made to cost the attacker, so a secret cannot be found by high-volume guessing; the responses to guesses do not tell an attacker they are getting closer. The enforcement mechanics (counters, lockout windows, source limits) are owned by the operational-readiness limits requirements — this area states the anti-guessing property and names `REQ-OPS-*` for the enforcement detail, and does not mint `REQ-OPS-*`.
- **acceptance_criteria**: Drive many failed sign-in attempts against one account; attempts are throttled, delayed, or capped rather than answered at full speed indefinitely, and the throttled response does not distinguish "almost right" from "wrong."
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-010, REQ-PROD-011

#### REQ-ID-036 — The secret is never stored or returned in a recoverable form
- **statement**: The product cannot reveal any account's secret to anyone, including the account owner and operators: the secret is never returned by any surface and is never held in a form from which the original value can be read back. Recovery restores access by setting a new secret, not by retrieving the old one. (How the secret is protected at rest — hashing and its parameters — is the code area's "how"; this requirement fixes the externally observable property.)
- **acceptance_criteria**: No product surface (API response, export, settings screen, operator tool) returns an account's secret. The only supported way back in after losing a secret is to set a new one through recovery.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-008, REQ-PROD-010

#### REQ-ID-037 — A successful sign-in issues a session distinct from the secret
- **statement**: Authentication issues a session credential that the app then carries on the person's behalf for subsequent actions; this session credential is separate from the account secret, so routine requests never carry the secret itself.
- **acceptance_criteria**: Sign in; subsequent authenticated actions succeed while carrying only the issued session credential, not the account secret. The account secret is not present on requests after sign-in.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-010

#### REQ-ID-038 — Sessions have a bounded lifetime and expire
- **statement**: Every session has a bounded maximum lifetime after which it is no longer accepted; a session does not remain valid forever.
- **acceptance_criteria**: A session that has passed its maximum lifetime is rejected, and the person is returned to sign-in. The bound is a defined value, not open-ended.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-014, REQ-PROD-015

#### REQ-ID-039 — Sign-out ends the session server-side
- **statement**: A person can sign out, and sign-out invalidates the session on the server so it can no longer be used — not merely dropping the app's local copy.
- **acceptance_criteria**: Sign out; then present the session that was in use (for example, replay the prior request). It is rejected. The next person at that device is not signed in as you.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-014

#### REQ-ID-040 — Session validity is decided server-side and cannot be forged or extended by the client
- **statement**: Whether a session is valid, and until when, is determined by the server; a client cannot mint a session, extend its own session's lifetime, or alter whose session it is by changing what it presents. (The durable store that holds authoritative session state so it survives a server restart is a domain/operational concern — this requirement fixes that validity is server-authoritative and names `REQ-DOM-*`/`REQ-OPS-*` for the durable-store "how"; it does not mint them.)
- **acceptance_criteria**: Tampering with the presented session credential (altering its contents, substituting another account's identifier, or extending an expiry it encodes) does not yield a valid session for a different account or a longer life. A session accepted before a server restart behaves consistently with the invalidation and expiry rules after one.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-010, REQ-PROD-014

#### REQ-ID-041 — The session credential is protected in transit
- **statement**: The session credential is exchanged only over an encrypted transport, so a network observer between the person and the product cannot read or capture it. (The transport configuration is the operational area's "how"; this requirement fixes that the session credential is never sent in the clear.) The product-wide generalization of this property to all private fields is REQ-ID-100.
- **acceptance_criteria**: The session credential is transmitted only over an encrypted channel; a request carrying it over an unencrypted channel is refused or redirected rather than served. Inspection confirms no path exchanges the session credential in cleartext.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-010, REQ-PROD-014

#### REQ-ID-042 — The session credential is not exposed where it can be stolen
- **statement**: The session credential is not placed where it leaks — not in URLs, not in logs, and not readable by page scripts that a cross-site attack could run — so that a stolen copy cannot be replayed as the owner.
- **acceptance_criteria**: Inspection of URLs, server and client logs, and the surfaces available to page scripts finds the session credential in none of them. A captured request intended to exfiltrate the credential through those channels yields nothing usable.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-010, REQ-PROD-014

#### REQ-ID-043 — A fresh session identity is established at sign-in (fixation resistance)
- **statement**: Signing in establishes a new session identity; a session identifier a person carried before authenticating cannot become an authenticated session. This prevents an attacker from fixing a known session identifier on a victim and inheriting it once the victim signs in.
- **acceptance_criteria**: Capture a pre-sign-in session identifier, then sign in; the identifier in force after sign-in differs from the pre-authentication one, and the old identifier does not carry authenticated privileges.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-010

#### REQ-ID-044 — An expired, signed-out, or unknown session is rejected consistently
- **statement**: A session credential that is expired, has been signed out, or is not recognized is rejected the same way — treated as unauthenticated — and the person is returned to sign-in rather than shown a silent failure or a partial, half-authenticated state.
- **acceptance_criteria**: Present an expired session, a signed-out session, and a made-up session credential; each is rejected and routed to sign-in. None grants any authenticated action.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-014, REQ-PROD-015

#### REQ-ID-045 — Each session is individually identifiable and independently revocable
- **statement**: A person's sessions are distinguishable from one another, so that one session (one device) can be ended without ending the others, and so the account-settings surface can list active sessions and end any of them. (The settings screen that lists and ends sessions is owned by accounts-and-lifecycle and the experience area under REQ-PROD-017; this requirement fixes the mechanism that makes per-session revocation possible.)
- **acceptance_criteria**: With two active sessions on two devices, end one; that one is rejected on its next use while the other keeps working. The set of a person's active sessions can be listed for the account-settings surface to present.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-014

#### REQ-ID-046 — Recovery re-establishes the secret and does not reopen old sessions
- **statement**: When a person recovers access by setting a new secret (recovery itself is owned by accounts-and-lifecycle, REQ-PROD-013), the new secret takes effect for future sign-in and sessions that predate the reset are not silently left valid — a recovery triggered because the account may be compromised must be able to end the attacker's sessions.
- **acceptance_criteria**: Complete a recovery that sets a new secret; the old secret no longer signs in, and sessions established before the reset are ended (or the person is given the option to end them) rather than remaining usable.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-010, REQ-PROD-014

#### REQ-ID-047 — Stay-signed-in on a trusted device is an explicit, opt-in choice
- **statement**: Remaining signed in across ordinary visits on a device is offered only as a choice the person makes for a device they treat as trusted, not a default applied to every device. On a device not marked trusted, the ordinary bounded session applies.
- **acceptance_criteria**: Sign in without choosing to be remembered; the ordinary session lifetime applies. Choose to stay signed in on a trusted device; you remain signed in across close-and-reopen within the allowed period.
- **release_gating**: false — staying signed in is a convenience, not a condition of a releasable product; a releasable chirp may require sign-in each visit (this mirrors REQ-PROD-015). Recorded so the choice is explicit rather than assumed.
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-015

#### REQ-ID-048 — A remembered session is itself bounded and revocable
- **statement**: Where the stay-signed-in convenience of REQ-ID-047 is offered, the remembered session still has a bounded maximum lifetime and is subject to sign-out and server-side invalidation like any other session; "stay signed in" does not mean "forever" and does not create a session that cannot be ended.
- **acceptance_criteria**: With stay-signed-in chosen, confirm the remembered session expires at its stated maximum and that sign-out and a remote end-this-session action both invalidate it.
- **release_gating**: false — this requirement binds only if the optional convenience in REQ-ID-047 is offered; when it is offered, these bounds are mandatory and gate that feature.
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-015, REQ-PROD-014

#### REQ-ID-050 — Multi-factor authentication is out of scope for the release bar
- **statement**: A second authentication factor (MFA) is not required for chirp's release bar. The reason: chirp is a single public product of individual accounts with no enterprise or regulatory obligation that compels a second factor, and a well-formed single secret plus the session and anti-guessing requirements above meets the bar of "real auth, not a stub." The credential and session design must not foreclose adding MFA later as an opt-in protection.
- **acceptance_criteria**: The release does not depend on an MFA feature. Review of the credential and session design confirms no decision taken here would have to be undone to add an optional second factor later.
- **release_gating**: false — recorded as a deliberate scope decision with its reason, not an omission.
- **verification_method**: inspection
- **traces_to**: REQ-PROD-010

#### REQ-ID-051 — Third-party / social sign-in is out of scope for the release bar
- **statement**: Signing in through a third-party or social identity provider is not required for chirp's release bar. The reason: chirp issues and holds its own account secret for individual accounts and has no enterprise single-sign-on obligation; adding a social provider brings its own account-linking, privacy, and provider-dependency surface that is not needed to reach a releasable product. The design must not foreclose adding a third-party sign-in option later.
- **acceptance_criteria**: The release does not depend on any third-party sign-in. Review of the sign-in and account-secret design confirms an external provider could be added later without reworking the owner-only sign-in guarantees above.
- **release_gating**: false — recorded as a deliberate scope decision with its reason, not an omission.
- **verification_method**: inspection
- **traces_to**: REQ-PROD-008, REQ-PROD-010

### Authorization (REQ-ID-060 .. REQ-ID-079)

#### REQ-ID-060 — Authenticated session is the sole basis of action authority (default-deny)
- **statement**: Every action that creates, changes, or removes data is authorized only for an actor with a valid authenticated session, and is refused otherwise; any action not explicitly permitted by this authority model is refused rather than allowed by default.
- **acceptance_criteria**: With no session, attempt each state-changing action the product exposes (post, reply, like, follow, edit profile, change a setting, delete content, delete account); each is refused. A read restricted to an owner (for example the account-settings view) is likewise refused without a session.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-023, REQ-PROD-010

#### REQ-ID-061 — Unauthenticated read scope is limited to the public fields
- **statement**: A viewer with no session may read only the public fields — a user's handle, display name, bio, follower and following counts, and that user's public chirps with their reply and like counts — and nothing else.
- **acceptance_criteria**: With no session, open a user's profile and their chirps: the public fields are shown. Through the same unauthenticated surfaces, no owner-only field (contact, credential, session list, settings, notification preferences) is returned for that user or any other.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-007, REQ-PROD-024

#### REQ-ID-062 — Owner-only fields are readable only by the account owner
- **statement**: The owner-only fields of an account — verified contact, authentication credential, active session and device list, notification preferences, account settings, and any data-export payload — are readable only by the authenticated owner of that account, and are never returned to another user or included in any public response.
- **acceptance_criteria**: Signed in as one account, attempt through every product surface to read another account's contact, session list, settings, or notification preferences; none is returned. Signed in as the owner, the same fields are readable for your own account only.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-024

#### REQ-ID-063 — Owner-scoped views are served only to their owner
- **statement**: A view composed for one account — the home timeline of the owner's own and followed accounts' chirps, and the account-management views — is served only to the authenticated owner of that account, not to other accounts or to unauthenticated viewers.
- **acceptance_criteria**: Signed in as account A, request account A's home timeline and account views; they are served. Attempt to obtain account B's home timeline or account views while signed in as A, or with no session; neither is served.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-002, REQ-PROD-024

#### REQ-ID-064 — A person acts only as themselves (no impersonation)
- **statement**: Every create, change, or remove action is authorized for, and carried out as, the authenticated actor of the session; the product provides no way for one account to act as, or on behalf of, another account.
- **acceptance_criteria**: Signed in as account A, attempt through the product's surfaces to post, reply, like, or follow as account B, or to edit account B's profile or settings; each is refused and nothing is recorded as B's action.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-023, REQ-PROD-010

#### REQ-ID-065 — Social write actions are performed as the signed-in actor
- **statement**: Posting a chirp, replying, liking and unliking, and following and unfollowing are authorized only for the signed-in actor and are performed as that actor; they cannot be directed to take effect under another account's identity.
- **acceptance_criteria**: Signed in, post, reply, like, and follow; each is attributed to your handle. Attempt the same actions carrying another account's identity through any available parameter or surface; the action is refused or applied only to your own account, never to the other.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-001, REQ-PROD-003, REQ-PROD-005, REQ-PROD-006, REQ-PROD-023

#### REQ-ID-066 — Change authority is limited to the actor's own account
- **statement**: A person may change only their own account — their profile (display name, bio), their contact, their notification preferences, and their account settings — and may not change any field of another account.
- **acceptance_criteria**: Signed in as account A, change A's display name, bio, contact, and settings; the changes take effect for A. Attempt to change any of the same for account B through any surface; each attempt is refused and B is unchanged.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-016, REQ-PROD-017, REQ-PROD-023

#### REQ-ID-067 — Removal authority is limited to the actor's own content and account
- **statement**: A person may delete only their own chirps and may delete only their own account; they may not delete another account or another account's content.
- **acceptance_criteria**: Signed in as account A, delete a chirp A posted, and delete account A; both succeed. Attempt to delete a chirp authored by account B, or to delete account B, through any surface; each is refused and B's content and account remain.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-020, REQ-PROD-023

#### REQ-ID-068 — Attribution is bound to the acting account and cannot be forged
- **statement**: The author recorded on a chirp, reply, like, or follow is the authenticated actor that performed it, set by the product from the session rather than from actor-supplied input, so that no actor can create or alter content attributed to another account.
- **acceptance_criteria**: Signed in as account A, create a chirp while supplying account B as the author through any available field; the stored chirp is attributed to A, or the request is refused — never attributed to B. No surface lets an actor change the author of existing content.
- **release_gating**: true
- **verification_method**: automated_test
- **traces_to**: REQ-PROD-023, REQ-PROD-001

#### REQ-ID-069 — A refused action does not disclose owner-only data or private existence
- **statement**: When an action or read is refused by this authority model, the response carries no owner-only field and reveals nothing about another account beyond what is already public, so that probing the authority boundary cannot be used to read private data or to learn more than the public surface already shows.
- **acceptance_criteria**: Signed in as account A, attempt a refused cross-account read or change against account B; the refusal response contains no owner-only field of B and no information about B not already readable on B's public profile. The refusal is consistent whether or not B exists, beyond what public lookup already discloses.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-024, REQ-PROD-011

#### REQ-ID-070 — Operator authority is a separate, provisioned role, not an ordinary account's
- **statement**: The authority to perform the supported administrative actions is held by provisioned operators and is distinct from ordinary-user authority; no account gains operator authority through ordinary sign-up or sign-in, and an ordinary session carries none of it.
- **acceptance_criteria**: Inspect how operator authority is granted; it is provisioned outside the ordinary sign-up/sign-in path. Signed in as an ordinary account, attempt each operator action (assist recovery, remove another's content, action an account); each is refused.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-028

#### REQ-ID-071 — Operators are authorized only for the supported, least-privilege action set
- **statement**: An operator is authorized to do only the supported administrative actions — assist a user's account recovery, remove content, and action (for example disable or suspend) an abusive account — and nothing beyond that set.
- **acceptance_criteria**: As a provisioned operator, perform each supported action and confirm it succeeds. Confirm, by inspecting the operator surface, that it exposes no authority beyond the supported set (for example it offers no way to read a credential or to post as a user).
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-028, REQ-PROD-025

#### REQ-ID-072 — Operators cannot impersonate users or read authentication credentials
- **statement**: Operator authority does not allow posting or acting as a user, and does not allow reading any account's authentication credential; an operator's access to an owner-only field is limited to what a supported action requires (for example initiating a recovery the owner completes through their verified contact).
- **acceptance_criteria**: As an operator, confirm there is no action that posts, replies, likes, or follows as a user. Confirm no operator action returns an account's credential. Confirm that assisting recovery routes the user through their own verified contact rather than exposing or resetting the credential to the operator.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-028, REQ-PROD-024

#### REQ-ID-073 — Operator actions are attributable and auditable
- **statement**: Each operator action on an account or on content records which operator performed it, what was done, and when, and that record is retained so administrative actions can be reviewed and attributed after the fact.
- **acceptance_criteria**: As an operator, remove a chirp and action an account; afterward a retained record shows the operator identity, the action, the target, and the time for each. The record is available to review and is not editable through the ordinary product surfaces.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-028

#### REQ-ID-074 — Operator removal and account actioning have a defined, recorded outcome
- **statement**: When an operator removes content or actions an account, the effect on that content and account is the product's defined outcome — consistent with the account-deletion outcome the accounts-and-lifecycle concern defines — and the applied outcome is recorded.
- **acceptance_criteria**: As an operator, remove a chirp and action an account; the resulting state of that content and account matches the product's stated outcome for each, and the applied outcome is recorded against the action.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-028, REQ-PROD-021

### Abuse and privacy basics (REQ-ID-080 .. REQ-ID-099)

#### REQ-ID-080 — Report a chirp or an account
- **statement**: A signed-in person can report a chirp or an account that breaks the rules, and is shown that the report was received.
- **acceptance_criteria**: Signed in, submit a report against a chirp and, separately, against an account; each submission returns an acknowledgement that the report was received. The report lets the reporter indicate which rule it concerns (see REQ-ID-081).
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-025

#### REQ-ID-081 — Stated rules a report can reference
- **statement**: The product states, in a place reachable from within the product, the rules that say what is and is not allowed, and a report can reference them.
- **acceptance_criteria**: From within the product, reach the rules/terms without an external link; the report flow (REQ-ID-080) lets the reporter point at a rule the content or account breaks.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-026

#### REQ-ID-082 — A report is captured and routed for action
- **statement**: A filed report is recorded and made available to the authorized operator/support surface, so it can be acted on; it is not merely acknowledged to the reporter and discarded.
- **acceptance_criteria**: Submit a report; afterward, through the supported operator/support surface (owned by the authorization area), the report is present with enough context to act on it — what was reported, by whom, against whom, and which rule. The acting itself (remove content, action the account) is the authorization area's requirement, not this one.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-025

#### REQ-ID-083 — Block an account
- **statement**: A person can block another account, after which that account can no longer interact with them (reply to, follow, or otherwise reach them) per the block's stated effect.
- **acceptance_criteria**: Signed in, block another account; afterward that account's attempts to interact with you are refused or hidden per the stated effect of a block, and the product states what a block does.
- **release_gating**: false — block and mute (REQ-ID-083, REQ-ID-084) are expected unwanted-interaction controls, but the product requirement that defines them lets a first release ship with reporting (REQ-ID-080) and operator action (authorization area) as the gated safety floor and add block/mute as a fast follow. The release bar decides which of block/mute v1 must include; at least the reporting floor is gated.
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-027

#### REQ-ID-084 — Mute an account
- **statement**: A person can mute another account, after which that account's content no longer appears to them, without the muted account being told.
- **acceptance_criteria**: Signed in, mute another account; afterward that account's chirps and replies are suppressed from your views per the stated effect of a mute, and the product states what a mute does and that it is not disclosed to the muted account.
- **release_gating**: false — see the reason on REQ-ID-083; the bar decides whether v1 includes mute.
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-027

#### REQ-ID-085 — Rate and volume limits as a first line against abuse
- **statement**: The product applies basic rate and volume limits to writes and requests so one account or client cannot trivially flood other users or overwhelm the service. Stated here as an identity-and-trust requirement; the enforcement, quota values, and tuning are the operational-readiness area's limits-and-quotas surface (`REQ-OPS-*`), not defined here.
- **acceptance_criteria**: Exceed a reasonable posting or request rate from one account; the product throttles or rejects the excess rather than serving it without limit, and a legitimate user at a normal rate is not blocked. The specific thresholds are the operational-readiness area's to set.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-041

#### REQ-ID-086 — Data minimization
- **statement**: The product collects only the data an account needs to function — a handle, a display name, an optional bio, the authentication credential, and a point of contact for recovery and notifications — and does not require data beyond that to use the core product.
- **acceptance_criteria**: Inspect what sign-up and the account surfaces collect and store; every collected field maps to a stated account function (identity, authentication, recovery/contact, profile). No field is collected that no function needs.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-024

#### REQ-ID-087 — Private-field protection
- **statement**: A person's private data — their point of contact, their authentication credential, and their session data — is never exposed to another user, never returned in a public or unauthenticated response, and never surfaced through the report or block/mute flows. Only the intended public fields (handle, display name, bio, public chirps, public counts) are visible to others. Protection of these fields in transit is REQ-ID-100.
- **acceptance_criteria**: As another user and as an unauthenticated visitor, view a person's profile, their public chirps, and any public responses; only the public fields appear. File a report against an account (REQ-ID-080); the report flow does not reveal the reported or reporting account's contact, credential, or session data. A person's credential is never returned to anyone, including the owner, in readable form.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-024

#### REQ-ID-088 — Erasure on account deletion
- **statement**: When a person deletes their account, their private data — contact, credential, and session data — is erased, and every session for the account is invalidated, consistent with the product's stated deletion outcome for content and handle.
- **acceptance_criteria**: Delete an account; afterward the account cannot sign in, no session for it remains usable, and its contact and credential data are no longer held. What happens to the account's public chirps and to its handle follows the stated deletion outcome (the accounts-and-lifecycle area's requirement); this requirement covers the erasure of the private data and the invalidation of sessions.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-021, REQ-PROD-024

#### REQ-ID-089 — A stated retention posture
- **statement**: The product states how long it keeps account data, reports, and operational records that contain personal data, and keeps them no longer than that stated posture, so data is not retained indefinitely by default.
- **acceptance_criteria**: Find the product's stated retention posture for account data, reports, and personal-data-bearing records; the product's behavior is consistent with it (records past the stated window are not retained). Reports are kept long enough to act on and to establish a pattern, then handled per the stated posture.
- **release_gating**: false — a stated retention posture is expected, but a first release may ship with a simple, stated posture and refine the windows later; the release bar decides the minimum. The operational-readiness area owns how records are aged out in practice.
- **verification_method**: inspection
- **traces_to**: REQ-PROD-024

#### REQ-ID-090 — A data-rights baseline
- **statement**: A person has a baseline set of rights over their own data, proportionate to a public microblog: to access and obtain a portable copy of it, and to have it erased. This baseline is satisfied by the account's own surfaces, not by a manual request process.
- **acceptance_criteria**: Signed in, obtain a portable copy of your own data (posts, profile, follows) through the product (the export mechanics are the accounts-and-lifecycle area's requirement), and erase your own data by deleting your account (REQ-ID-088 and the deletion outcome). No step requires contacting an operator by hand. The product states this baseline so a person can find it.
- **release_gating**: true
- **verification_method**: manual_user_check
- **traces_to**: REQ-PROD-019, REQ-PROD-024

### Threat model (REQ-ID-100 .. REQ-ID-119)

#### REQ-ID-100 — Private fields are protected in transit (product-wide)
- **statement**: Every private field chirp exchanges with a person — the account secret during sign-up, sign-in, and recovery; the session credential; the verified contact; and the data-export payload — is transmitted only over an encrypted transport, so a network observer between the person and the product cannot read or capture any of them. This generalizes REQ-ID-041, which fixes the property for the session credential alone, to the whole private surface; the transport configuration is the operational-readiness area's "how."
- **acceptance_criteria**: Exercise sign-up, sign-in, recovery, an authenticated action, and a data export; inspect each exchange — no private field (secret, session credential, verified contact, or export payload) is transmitted over an unencrypted channel. A request that would carry any such field over an unencrypted channel is refused or redirected rather than served.
- **release_gating**: true
- **verification_method**: inspection
- **traces_to**: REQ-PROD-024, REQ-PROD-010, REQ-PROD-014

#### REQ-ID-101 — Anti-automation / bot resistance on sign-up and abusable write surfaces
- **statement**: The product raises the cost of automated abuse that per-account rate limits and sign-in anti-guessing do not address — in particular automated mass account creation at the public sign-up surface, and automated high-volume use of the public write surfaces from many fresh identities. It applies at least a basic anti-automation measure so a bot cannot cheaply create accounts or flood the service at scale. The enforcement mechanics, thresholds, and tuning are the operational-readiness area's limits-and-quotas surface (`REQ-OPS-*`); this requirement states the property and names that area, and does not mint `REQ-OPS-*`.
- **acceptance_criteria**: Drive automated account creation against the sign-up surface at a volume a human would not reach; the product slows, challenges, or rejects the automated attempts rather than creating accounts without limit, while a legitimate person signing up once is not blocked. Drive automated writes from many newly created accounts; the product resists the aggregate automated load rather than serving it unbounded.
- **release_gating**: true
- **verification_method**: operational_check
- **traces_to**: REQ-PROD-041, REQ-PROD-010

---

## Coverage note

Every `REQ-PROD-*` identity and security need is answered here: real sign-up and owner-only sign-in
(008, 010 → REQ-ID-002, 030–037, 060, 064), handle validation (009 → REQ-ID-003), not-found vs
failed-credential signalling (011 → REQ-ID-033, 069, 015), verified contact and recovery (012, 013 →
REQ-ID-004, 005, 009, 013–016), server-side sign-out and session hygiene (014, 015 → REQ-ID-038–048),
profile and settings (016, 017 → REQ-ID-007–011, 066), handle-change policy (018 → REQ-ID-012),
export (019 → REQ-ID-017–019, 090), deletion and its outcome (020, 021, 022 → REQ-ID-020–024, 074,
088), the authorization boundary (023 → REQ-ID-060, 064–068) and privacy basics (024 → REQ-ID-001,
018, 019, 023, 061–063, 069, 072, 086, 087, 100). The identity aspects of safety and support are
answered too: reporting and rules (025, 026 → REQ-ID-080–082), block/mute (027 → REQ-ID-083, 084),
and the operator authority behind the supported admin surface (028 → REQ-ID-070–074). Two product
needs are carried by sibling areas by design and named rather than re-minted here: the enforcement
depth of limits and quotas (`REQ-PROD-041`) is the operational-readiness area's (`REQ-OPS-*`), to
which REQ-ID-035, 085, and 101 defer; and durable storage of accounts, sessions, and content
(`REQ-PROD-037`) is the domain and operational-readiness areas', on which every lifecycle and session
requirement here depends.
