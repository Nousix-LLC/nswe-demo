# chirp — Product requirements

This document states what chirp must be, for whom, and why, and what "done" looks like from
the user's side — never how it is built. It is the foundation the rest of the design traces to.
It is grounded in the product as it exists in the repository `Nousix-LLC/nswe-demo` at commit
`c805525`, and it is held to the bar of a releasable consumer web application, not a prototype.

It has five parts, read in order:

1. **Product definition & kind** — what chirp is, and which product kind(s) apply and what each
   makes its users assume without asking.
2. **Users & personas** — the distinct people chirp serves.
3. **End-to-end journeys** — what each of them is trying to do, from first visit to the day they leave.
4. **Requirements & acceptance criteria** — the `REQ-PROD-*` register, each requirement testable by
   a stranger, each flagged for whether the release bar depends on it.
5. **In scope / out of scope** — what a releasable product of this kind includes and leaves out, with reasons.

The detailed "how," and the requirements owned by the other design areas — the domain and
persistence model (`REQ-DOM-*`), the experience design (`REQ-UX-*`), identity and security
(`REQ-ID-*`), operational readiness (`REQ-OPS-*`), and the consolidated release bar (`BAR-*`) —
live in their own design documents. Where a product-level requirement here depends on one of
those areas, it is stated from the user's side and the owning area is named; this document mints
only `REQ-PROD-*` ids.

---

## 1. Product definition & kind

### What chirp is

**chirp is a Twitter-style microblogging web application: a place where a person posts
short public messages and reads a running timeline of messages from the people they
follow.** It is a product someone uses in a browser, not a library or a toolkit. The
framework it is built on (`ferric`) is the engagement's other deliverable; chirp is the
product that proves the framework, and this document is about chirp as a thing a stranger
would use.

From the user's side, chirp is the following loop:

- **Get in.** You arrive at the site, identify yourself, and you are in — your handle is
  how everyone else knows you.
- **Post.** You write a short message (a *chirp*, up to 280 characters) and publish it.
  You can post a reply to someone else's chirp.
- **Read.** You read a timeline of chirps, newest first, a page at a time.
- **Follow.** You follow and unfollow other people; who you follow shapes what you read.
- **React.** You like and unlike chirps, and you can open any person's profile to see who
  they are and act on them (follow, view their chirps).

That is the whole product in one sentence a user would recognize: *post short messages,
read a timeline, follow people, like what you like.* Everything else in this design exists
to make that loop something a stranger could actually sign up for, trust with their
account, and rely on.

#### How it is shaped (grounded in the repository at `c805525`)

Described by repository identity, not by implementation how:

- A **single-page web application** (`crates/chirp-frontend`), delivered to the browser as
  WebAssembly, built on the in-house `ferric` UI framework and its router. The app presents
  three screens today — a timeline view (`/`), a compose-and-sign-in view (`/compose`), and
  a user-profile view (`/user/:id`) — above a persistent navigation bar.
- A **REST backend** (`crates/chirp-server`, an axum service) that the SPA calls over a
  typed contract shared by both sides (`crates/chirp-types`). The live surface at `c805525`
  is: register a user, fetch a user, open a session, post a chirp, read a timeline
  (cursor-paginated), follow/unfollow a user, like/unlike a chirp, and an unauthenticated
  liveness probe.
- It **ships as one self-contained container** that serves both the API and the SPA, and is
  operated as a hosted service — deployed to a Kubernetes cluster via GitOps (`deploy/`) and
  reachable on the web. There is no separate product the user installs.

#### The state of the product at `c805525` (where the design must take it from)

These are observations about the product as it exists, not a design for how to fix them —
the fixes belong to the requirements below, the identity/security design, and the
operational-readiness design. The gap between this state and a releasable product is what
the rest of the requirements close.

- **Identity is handle-only.** Signing in means typing a handle; the server opens a session
  for that handle and returns a token. There is no password, no credential, and no
  verification — nothing stops one person from claiming another's handle. For a product
  whose entire premise is *this person said this*, that is the central gap, not a detail.
- **Nothing is remembered across a restart.** All users, chirps, follows, and likes live in
  the server's memory; when the process restarts, the product is empty again. A microblog
  that forgets everything it was told is not yet a product a stranger can rely on.
- **The account-lifecycle and supporting surfaces are absent.** There is no sign-up distinct
  from "type a handle," no password/recovery, no account settings, no way to delete an
  account or take data out, no onboarding, no notifications, no search, no media, no
  direct messages, no blocking or muting, no moderation, and no help/admin surface.

The core loop is real and implemented; what is missing is almost everything that stands
*around* the loop and makes it releasable. Naming that is this definition's job.

### Product kind(s)

A product's kind sets the things its users assume without being told. chirp is **one kind**,
and three other candidate kinds do **not** apply — each recorded with its reason, because a
kind that does not apply still shapes what is correctly out of scope.

#### chirp is a consumer web application (the applicable kind)

chirp is a **hosted, public, account-based consumer web application**: individual people
sign up directly, use it in a browser, and it is operated as a service by the people who
build it. This is the kind the engagement aim describes — a production-quality, publicly
reachable Twitter-style product (`TASK.md` at the repository root) — and it is the kind
every requirement in this design is held to.

**What this kind makes its users assume without asking.** A person who signs up for a
hosted consumer web product takes all of the following for granted, and is surprised and
distrustful when any is missing:

- **They can get in, and get back in.** There is a real sign-up and sign-in, and if they
  forget how to get in they can recover access — their account is not lost the first time
  they clear a cookie or forget a detail. `need: real-account-signup-signin`,
  `need: account-recovery`.
- **Their identity is actually theirs.** The handle they claim cannot be silently taken or
  impersonated by anyone who types it; signing in proves it is them.
  `need: authenticated-identity`.
- **It remembers their stuff.** What they posted, who they follow, and what they liked are
  still there the next time they visit, and survive the service being restarted or updated.
  `need: durable-account-data`.
- **The account is theirs to manage and to end.** They can see and adjust their account,
  delete it when they want, and take their data with them. `need: account-settings`,
  `need: account-deletion`, `need: data-export`.
- **Only the right people can see and do the right things.** There is a clear boundary on
  who may act as them and who may change what; private details are not exposed to everyone.
  `need: authorization-boundary`, `need: privacy-basics`.
- **It behaves like a finished thing.** It tells them what is happening when a list is empty,
  something is loading, or something failed; it works on a phone; and it is reachable and
  reasonably available when they come back. `need: considered-states`,
  `need: works-on-mobile`, `need: reachable-and-available`.
- **Someone is minding it.** There is a way to get help and to report abuse, and bad actors
  can be dealt with — a public space with no way to report a problem does not feel safe.
  `need: help-and-abuse-reporting`.

These implications are the backbone of chirp's requirements. Each is surfaced above as a
prose need with an informal `need:` handle, formalized into the `REQ-PROD-*` register in
Part 4 (or, where another design area owns the detail — identity/security, experience,
operational readiness — carried as a product-level requirement that traces to that area).
Every implication named here is either reflected in a requirement or recorded out-of-scope
with a reason; Part 5 records the out-of-scope judgments.

#### Kinds that do NOT apply (recorded with reasons)

- **Multi-tenant SaaS — does not apply.** A SaaS product is sold to organizations, with
  per-tenant isolation, team/seat management, an admin who governs other users, and billing.
  chirp has no notion of a tenant or an organization: it is one shared public social space of
  individual accounts, and no money changes hands. The team-management, tenancy, and billing
  surfaces a SaaS buyer would assume are therefore **out of scope** for chirp — recorded here
  so their absence is a decision, not an oversight. `need: no-tenancy-or-billing` (as an
  explicit non-goal).
- **Commercial off-the-shelf (COTS) — does not apply.** A COTS product is delivered to
  customers to install, license, upgrade, and run in *their* environment. chirp is operated
  as a hosted service by its maker and is not distributed to customers to install or license.
  Customer-side install/licensing/upgrade tooling is therefore **out of scope**. (The
  product does ship as a container and is self-hostable in principle — see the OSS note — but
  it is not *productized for customers to install*, which is what the COTS kind would demand.)
- **Open-source project — does not apply as a product kind.** An OSS product wins users who
  will never pay or file a ticket: it assumes they can self-host it, read how it works, and
  contribute. chirp's repository README declares it a **closed demonstration that does not
  accept contributions**, with issues and pull requests limited to the repository's
  collaborators. So chirp is not an OSS *product* soliciting self-hosting users or
  contributors, and the community surfaces an OSS kind implies (contribution guide, public
  issue intake, a self-host support promise) are **out of scope**.

  One nuance, so it is not mistaken for an OSS obligation: the repository is
  public and source-available, and the app is self-hostable from the single container it
  ships as. A reader *can* run their own copy. That property is a consequence of how the
  demonstration is published, not a product kind — it does not add the assumptions an OSS
  product carries, and it does not reduce the release bar.

#### One thing the kind does NOT reduce

The repository's "closed demonstration" framing governs *contribution and distribution* — it
is why the OSS community surfaces are out of scope. It does **not** lower the product bar:
the engagement aim is a production-quality product, and absent an explicit reduction, chirp
is held to the bar of a releasable consumer web application (`need: releasable-bar-not-prototype`).
A reader should not read "demonstration" as "prototype exemption."

The in/out-of-scope view implied by this kind is consolidated in **Part 5**.

---

## 2. Users & personas

Each persona is a distinct kind of person with a distinct goal, not a demographic. A single real
human may move between them (a visitor becomes a chirper; a chirper also operates their own instance).
What separates them is what they are trying to accomplish and what they assume the product owes them.
chirp's product kind — a consumer web application first, settled in Part 1 — is why a stranger can
reach it, sign up, and use it from a browser; because it also ships as a single self-contained server
image, **running an instance** is a real activity, which gives the operator (and, depending on the
product-kind decision, a self-hoster) a place among these personas. Personas that depend on the
product-kind decision are marked.

### P1 — The unregistered visitor

**Who:** someone who has arrived at chirp with no account — from a shared link, a search, or
curiosity — and has not decided whether to join.

**Goal:** understand what chirp is and whether it is worth an account, with as little commitment as
possible, and then either join or leave.

**Context & assumptions:** a first-time visitor to a consumer web product expects to see *something*
before being asked to sign up — at minimum an explanation of what the product is, and ideally a taste
of real content (a public profile, a sample of posts). They assume signing up is quick and reversible,
and that they are not being asked for more than the product needs. They judge the whole product in the
first minute, mostly on the landing experience and the sign-up step.

**What they need from chirp:** a way in that explains itself and does not dead-end. Today the home
timeline requires authentication and there is no public discovery surface, so a visitor who is not
handed a link to a specific profile has little to look at before committing. `need: a first-visit
experience that lets an unregistered visitor understand chirp and reach sign-up without an account`.

### P2 — The everyday chirper (reader & poster)

**Who:** the core user — a registered person who reads and posts. "Reader-leaning" and "poster-leaning"
are the same persona at different moments; most people do both, and the product does not meaningfully
separate them (there is one account type, one home timeline, one compose action).

**Goal:** keep up with the people they follow and share short posts of their own, in short visits
through the day, without friction and without losing what they have posted or who they follow.

**Context & assumptions:** this persona assumes the ordinary contract of an account-based social
product: that signing in is theirs alone and cannot be done by someone who merely knows their handle;
that their posts, follows, and profile **persist** and are still there on the next visit; that they can
end a session, especially on a shared device; that they can change their mind — edit their profile,
leave, take their content with them. They expect the daily loop (open, read the timeline, post, reply,
like, follow) to be fast and to behave sensibly when there is nothing new, when they are brand new with
an empty timeline, and when something goes wrong.

**What they need from chirp:** the full participatory loop plus the account-lifecycle surfaces that
make the account feel like theirs. Several of these are the "unglamorous" surfaces the product does not
yet have; they are surfaced in the journeys below (sign-in that proves identity, session end, durable
data, account management, leaving with their data).

### P3 — The returning user after a break

**Who:** a registered chirper who has been away long enough that their session has ended and they are
no longer sure what they missed. This is P2 at a specific, high-friction moment, called out separately
because the return journey exposes needs the daily loop does not.

**Goal:** get back in and re-orient — sign back in (or recover access if they have forgotten how they
get in), and catch up on what happened while they were gone.

**Context & assumptions:** they assume getting back in is possible even if they have lost whatever they
signed in with, and that a device they trusted might remember them. They assume the catch-up is bounded
— they can see what is new without scrolling forever — and that a long absence has not quietly lost
their account or their follows.

**What they need from chirp:** a sign-in that recognizes a returning person, a recovery path when they
cannot sign in, and a timeline that handles "a lot has happened since you were here" gracefully.

### P4 — The operator / administrator

**Who:** the person responsible for running the chirp service for its users — deploying it, keeping it
up, keeping it safe, and helping users when something is wrong. In a small product the operator also
wears the support and trust-&-safety hats.

**Goal:** keep chirp available, protect the data users have entrusted to it, and be able to answer "is
it healthy?", "will the data survive a restart?", and "a user needs help / a user is abusing the
service — what can I do?".

**Context & assumptions:** an operator assumes the product can tell them whether it is healthy
(there is a `/healthz` probe today), that user data is durable across the restarts and redeploys that
operating a service entails, and that there is *some* supported way to act on an abusive account or an
account-holder's request (help a user recover, remove content, respond to a deletion request) without
editing data by hand. They do not assume a rich admin console for a product this size, but they do
assume the absence of any supported operator action is a gap, not a design choice.

**What they need from chirp:** operational visibility and durable data (the detailed requirements
belong to the operational-readiness and identity design areas; the operator journey below states the
need from the operator's side), and at least a minimal, supported way to administer accounts and
content.

### P5 — The self-hoster *(product-kind-dependent)*

**Who:** a technically capable person who runs their own copy of chirp from the published image, for
themselves or a small group, rather than using a service someone else operates.

**Goal:** stand up their own instance, understand how to configure and run it, and trust it with their
small community's data.

**Context & assumptions:** whether this persona is in scope depends on the product-kind decision made
in Part 1. chirp already ships as a single self-contained, documented container image, which makes
self-hosting technically real today. A self-hoster assumes they can run it from documentation alone,
configure the handful of things an instance needs, and that running their own copy does not cost them
the durability and account-management surfaces a hosted user would get. Because chirp's product kind is
settled as a closed hosted demonstration, this persona is recorded **out of scope** (Part 5) with that
reason rather than designed for — while the durability and account-lifecycle requirements are written
to hold for *any* deployment, so an instance someone does run inherits them.

**What they need from chirp:** a runnable, documented image (present today) and the same durability and
account-lifecycle guarantees as any other deployment — so self-hosting is not a second-class experience.

---

## 3. End-to-end journeys

Each journey is walked from the user's side, step by step, including the empty, error, and
second-visit states where they shape the experience. The journeys are walked against the product as it
exists today (the `chirp-server` REST API and the `chirp-frontend` single-page app). A step marked
**gap** is one the product does not support today but a releasable product of this kind is expected to;
each gap carries a `need:` handle so Part 4 can adopt it and make it testable. Detailed screen design
belongs to the experience design, authentication and session internals to the identity design, and
durability and operations to the domain and operational-readiness designs; the journeys here describe
what the user is trying to do and what they need, and leave each area its formal requirement.

### J1 — First visit (unregistered visitor → decision) — P1

1. The visitor arrives at chirp's address with no account and no session.
2. They look for an answer to "what is this and should I join?" A consumer web product is expected to
   present a landing experience — what chirp is, and ideally a sample of real content — before asking
   for a commitment.
   - **gap / empty state:** the home timeline requires authentication and there is no public discovery
     or landing content, so a visitor who did not arrive on a specific public profile link has little
     to evaluate. `need: a public landing/first-visit experience that explains chirp and leads to
     sign-up without requiring an account`.
3. If the visitor arrived on a link to a specific profile, they can view that public profile (the
   product serves a user's public profile without authentication). From there they understand the shape
   of the product — a person, their posts' existence — but cannot browse further without an account.
4. The visitor decides to join and proceeds to sign-up (J2), or leaves. If they leave, nothing is
   created; this is a clean exit and the expected outcome for many first visits.

**Needs surfaced:** `need: a public first-visit experience`; `need: a clear, low-commitment path from
first visit to sign-up`.

### J2 — Sign-up (create an account) — P1 → P2

1. The visitor chooses to create an account and provides a **handle** (the unique username, up to 15
   letters, digits, or underscores), a **display name**, and optionally a short **bio**.
2. The product checks the handle is well-formed and not already taken; a taken handle is reported as a
   conflict the person can resolve by choosing another. An ill-formed handle (a space, a disallowed
   character, too long) is reported against the offending field so the person can fix it. These
   validation and conflict behaviors exist today and are the expected, correct shape.
3. The account is created and the person is now a chirper (P2).
   - **gap:** sign-up collects a handle and profile only — it establishes **no secret or credential**
     the person will later use to prove they are them. Anyone who later knows the handle can sign in as
     this account (see J3). A releasable account-based product is expected to establish a real
     credential at sign-up. `need: sign-up establishes a real authentication credential, not a handle
     alone` (the credential mechanism itself is the identity area's requirement).
   - **gap:** there is no confirmation that the person controls any contact address, so there is no
     basis for later account recovery (J3a) or important notifications. `need: a verified point of
     contact captured at sign-up to support recovery and notifications` (kept minimal and
     privacy-respecting; what contact is collected is an identity/privacy decision).
4. Immediately after sign-up the person lands in the product for the first time with **nothing yet** —
   no posts, no follows, an empty home timeline (J5 covers the empty state and onboarding).

**Needs surfaced:** `need: handle uniqueness and validation feedback at sign-up` (present today);
`need: a real credential at sign-up`; `need: a verified contact for recovery/notifications`;
`need: a first-run / onboarding experience that gets a brand-new account to a non-empty timeline`.

### J3 — Sign-in (returning to an existing account) — P2 / P3

1. The person provides the handle they signed up with.
2. The product authenticates them and issues a **session token** the app then carries on their behalf;
   subsequent actions (posting, following, reading the home timeline) are performed as that
   authenticated person. The session-token mechanism exists today.
   - **gap (critical, from the user's side):** authentication today is **handle-only** — presenting a
     known handle is sufficient to receive a session for that account. From the user's perspective this
     means *anyone who knows your handle can sign in as you*, which breaks the most basic assumption a
     person has about their account. `need: sign-in proves the person is the account owner, not merely
     that they know the handle` (the authentication design is the identity area's requirement; the
     journey states the user-side need: an account only its owner can enter).
3. An unknown handle is reported as "no such account" so the person can tell a typo from a
   not-yet-created account and proceed to sign-up.
4. The authenticated person continues into their daily use (J5).

**Needs surfaced:** `need: authenticated sign-in that only the account owner can pass`;
`need: a clear not-found vs. wrong-credential distinction on sign-in`.

#### J3a — Account recovery (forgotten access) — P3

1. A returning person cannot sign in — they have forgotten or lost whatever they sign in with.
2. They look for a "can't get in?" path that re-establishes access through the verified contact
   captured at sign-up, without needing anyone to act on their behalf.
   - **gap:** there is no recovery path today. With handle-only sign-in the question is moot; once a
     real credential exists (J2/J3), a person who loses it has no supported way back in. `need: a
     self-service account-recovery path for a person who has lost their credential`.
3. On success the person is back in and continues as P2.

**Needs surfaced:** `need: self-service recovery through a verified contact`.

#### J3b — Ending a session (sign-out) — P2

1. The person wants to stop being signed in — especially on a shared or public device.
2. They look for a sign-out action that ends the current session so the next person at that device is
   not them.
   - **gap:** there is no way to end a session today — the app can drop its copy of the token, but
     there is no supported sign-out that invalidates the session server-side. On a shared device this
     leaves the account exposed. `need: a sign-out that reliably ends the session, including
     server-side invalidation`.

**Needs surfaced:** `need: sign-out that ends the session`.

### J4 — The core loop: post, read, reply, like, follow — P2

This is the activity the product exists for; it is walked as the connected loop a chirper actually
performs, not as isolated features.

1. **Read the home timeline.** The chirper opens chirp and sees their home timeline — their own posts
   and those of everyone they follow, newest first, in bounded pages (the product paginates with an
   opaque cursor and returns pages rather than everything at once). This is the default thing to do on
   opening the app.
   - **empty state:** a chirper who follows no one and has posted nothing sees an empty timeline; the
     product should explain what to do about it rather than show a blank (J5 onboarding).
   - **second-visit / catch-up state:** a chirper returning after activity sees new posts since last
     time; paging lets them catch up without loading everything. `need: a timeline that handles a large
     backlog of unseen posts gracefully (bounded catch-up, not infinite load)`.
2. **Compose and post a chirp.** The chirper writes a message up to 280 characters and posts it; it is
   attributed to them and appears in their timeline and in the timelines of their followers. The
   length limit and validation exist today.
   - **error state:** an over-long or empty message is rejected against the text field with a message
     the composer can act on (present today). The compose experience should show the limit *before*
     submission (a detail owned by the experience design). `need: compose shows remaining
     characters and prevents an over-limit post before submission`.
3. **Reply.** The chirper replies to a chirp; the reply records what it is replying to, enabling a
   threaded conversation. Replying to a chirp that no longer exists is reported as not-found rather than
   silently dropped. Reading a conversation as a thread (following a reply to its parent and back) is a
   need the journey implies. `need: read a reply in the context of the conversation it belongs to`.
4. **Like / unlike.** The chirper likes a chirp (and can unlike it); liking is idempotent — liking
   twice does not double-count — and the like count they see reflects the result. This behavior exists
   today.
5. **Follow / unfollow.** The chirper follows another user to bring that user's posts into their home
   timeline (and unfollows to remove them); following is idempotent and a person cannot follow
   themselves. This exists today. Finding *who* to follow is the open question:
   - **gap:** there is no search or discovery surface — a chirper can only follow someone whose profile
     they already reached (e.g. via an external link or a reply). A social product is expected to offer
     some way to find people. `need: a way to discover and find other users and content to follow`.
6. **View a profile.** The chirper views a user's profile — their display name, bio, and follower and
   following counts — to decide whether to follow, or to see their own public face. Public profiles are
   served today.

**Needs surfaced:** `need: home timeline (own + followed, newest-first, paged)` (present today);
`need: post a chirp within a published length limit` (present today);
`need: reply and read conversations in context`;
`need: like/unlike, idempotent, with an accurate count` (present today);
`need: follow/unfollow to shape the timeline` (present today);
`need: discover/search for people and content to follow` (gap);
`need: view a user's public profile` (present today).

### J5 — Daily & returning use (empty / error / second-visit states) — P2 / P3

1. **Brand-new account (first run).** Straight after sign-up the chirper has an empty timeline. The
   product should onboard them — explain the empty state and help them reach a non-empty timeline
   (suggest or help them find people to follow, prompt a first post) rather than present a blank page.
   `need: a first-run onboarding that moves a new account off the empty state`.
2. **Ordinary daily visit.** The chirper opens chirp, reads what is new, posts or replies, and leaves —
   often in under a minute. The loop must be fast and must not make them re-authenticate every visit on
   a device they trust (bounded by the session/sign-out needs in J3/J3b). `need: a trusted device keeps
   the chirper signed in across ordinary visits, subject to session expiry and sign-out`.
3. **Error states the chirper meets.** An expired or invalid session should return the chirper to
   sign-in with their place explained, not fail silently (the product returns an unauthorized signal
   today; presenting it well is the experience area's job). A transient server error should be
   recoverable by retrying without losing a drafted chirp. `need: expired-session and transient-error
   states that recover without losing the person's in-progress work`.
4. **Returning after a break.** A lapsed chirper (P3) signs back in (J3) or recovers access (J3a), then
   meets the catch-up state of J4.1.

**Needs surfaced:** `need: onboarding off the empty state`; `need: fast, low-friction daily re-entry`;
`need: graceful expired-session and error recovery`.

### J6 — Managing the account (profile, settings) — P2

1. The chirper wants to change how they appear — update their display name or bio — or manage the
   handful of settings an account-based product carries (their contact for recovery/notifications,
   notification preferences, session/device management).
   - **gap:** there is no way to edit a profile or change any account setting today — a profile is set
     at sign-up and fixed thereafter. A person's display name and bio are exactly the things they
     expect to be able to change. `need: edit profile (display name, bio) after sign-up`.
   - **gap:** there is no settings surface for the account-level choices the lifecycle implies (contact,
     notifications, sessions). `need: an account settings surface for contact, notifications, and
     session/device management` (the specific settings are shared with the identity and experience
     areas; the journey establishes that the surface must exist).
2. Whether a handle can be changed after sign-up is an open product question (handles are unique
   identifiers others follow by). The journey raises it; the decision is the product-definition/identity
   areas'. `need: a decision on whether and how a handle can change after sign-up`.

**Needs surfaced:** `need: edit profile after sign-up`; `need: an account-settings surface`;
`need: a handle-change decision`.

### J7 — Safety: being bothered by others — P2

1. A chirper is replied to, followed, or targeted by an account they do not want to interact with.
2. They look for the controls a social product is expected to have — block or mute an account, and
   report content or an account that violates the rules.
   - **gap:** there are no block, mute, or report controls today, and no stated rules. For a product
     that lets strangers post at each other, the absence of any safety control is a releasability
     concern, not a nice-to-have. `need: block and mute controls so a chirper can stop unwanted
     interaction`; `need: a way to report abusive content or accounts`; `need: stated rules/terms a
     report can reference`.
3. Notifications are the other side of this loop: a chirper expects to know when they are replied to,
   followed, or liked.
   - **gap:** there is no notification surface today. `need: notify a chirper of relevant activity
     (replies, follows, likes), with preferences`.

**Needs surfaced:** `need: block/mute`; `need: report abuse`; `need: stated rules`; `need: activity
notifications with preferences`.

### J8 — Leaving: deletion and taking your data out — P2

The end of the relationship is part of the product. A person who cannot leave cleanly, or cannot take
their content with them, does not fully trust the product with their data in the first place.

1. **Export my data.** The departing (or merely cautious) chirper wants to take their content with them
   — their posts, their profile, who they follow.
   - **gap:** there is no data export today. A releasable consumer product is expected to let a person
     obtain their own data. `need: a person can export their own data (posts, profile, follows) in a
     portable form`.
2. **Delete my account.** The person decides to leave and wants their account and content removed.
   - **gap:** there is no account deletion today — once created, an account cannot be removed by its
     owner. `need: a person can delete their account and have their content removed`.
   - The journey raises, for the identity/privacy areas to decide: what deletion does to the person's
     chirps that others have replied to or liked, and whether the handle becomes available again.
     `need: a defined outcome for a deleted account's content and handle`.
3. **Confirmation and finality.** Deletion is destructive and should be confirmed, and the person should
   be told what will and will not survive (e.g. that export is their chance to keep a copy). `need:
   deletion is confirmed and explains what is and is not retained`.

**Needs surfaced:** `need: self-service data export`; `need: self-service account deletion`;
`need: a defined deletion outcome for content and handle`; `need: confirmed, explained deletion`.

### J9 — Operating the service (keep it running for users) — P4

1. **Is it healthy?** The operator checks whether the service is up; chirp answers a health probe
   today, which reports process liveness. The operator needs enough signal to know the service is
   serving users, not merely running. `need: a health/readiness signal the operator can
   trust to mean "serving users"` (the detailed observability requirement is the operational-readiness
   area's).
2. **Will the data survive?** The operator deploys, restarts, and redeploys as a normal part of running
   a service. They need the users' accounts, chirps, and follows to persist across those events.
   - **gap (critical, from the operator's and every user's side):** chirp stores its data **in memory**
     with no external database, so a restart or redeploy loses every account, chirp, follow, and
     session. For a product held to a releasable bar this is a blocker: it breaks the P2 assumption
     that "my account and posts are still there next time." `need: user data (accounts, chirps,
     follows, sessions) is durable across restart and redeploy` (the persistence design is the domain
     and operational-readiness areas' requirement; the journey states that no user journey above holds
     without it).
3. **Help a user / act on abuse.** A user asks for help (cannot get in, wants their account removed) or
   an account is reported for abuse. The operator needs a supported way to act — assist recovery,
   remove content, action an account — without hand-editing data.
   - **gap:** there is no administrative or support surface today. `need: a minimal, supported
     administrative surface for the operator to act on accounts and content (assist, remove, action
     abuse)`.
4. **Back up and recover.** The operator needs to be able to recover user data after a failure.
   - **gap:** with no durable store there is nothing to back up; once data is durable, a backup/restore
     path is expected. `need: a backup and recovery path for durable user data` (owned by the
     operational-readiness area; raised here because the operator's journey requires it).

**Needs surfaced:** `need: trustworthy health/readiness`; `need: durable user data across
restart/redeploy` (blocker); `need: a supported administrative/support surface`; `need: backup and
recovery`.

### J10 — Self-hosting an instance — P5 *(product-kind-dependent)*

1. A self-hoster runs chirp from the published container image for their own small community.
2. They configure the instance from documentation, start it, and confirm it is serving. The
   self-contained, documented image makes this real today.
3. They expect their instance to give their users the same durability and account-lifecycle guarantees
   as any other deployment — the same durable-data and account-management needs from J8 and J9 apply to
   a self-hosted instance. `need: a self-hosted instance offers the same durability and
   account-lifecycle guarantees as a hosted one`.
4. Because chirp's product kind is settled as a closed hosted demonstration (Part 1), this journey is
   recorded **out of scope** (Part 5) with that reason rather than designed for; the durability and
   account-lifecycle requirements are nonetheless written to hold for any deployment.

**Needs surfaced:** `need: a runnable, documented self-host image` (present today); `need: parity of
durability and account-lifecycle guarantees for self-hosted instances` (conditional on product kind).

### Journey → need summary

This table collects the journey-derived needs that Part 4 formalizes, so every journey traces to at
least one requirement. "Status today" records whether the product supports the need today, from the
walks above.

| Journey | Need handle | Status today |
|---|---|---|
| J1 | `need: a public first-visit experience` | gap |
| J1/J2 | `need: a low-commitment path from first visit to sign-up` | gap |
| J2 | `need: handle uniqueness and validation feedback at sign-up` | present |
| J2/J3 | `need: a real authentication credential (not handle alone)` | gap (critical) |
| J2/J3a | `need: a verified contact for recovery/notifications` | gap |
| J2/J5 | `need: first-run onboarding off the empty state` | gap |
| J3 | `need: sign-in only the account owner can pass` | gap (critical) |
| J3 | `need: clear not-found vs wrong-credential on sign-in` | partial |
| J3a | `need: self-service account recovery` | gap |
| J3b | `need: sign-out that ends the session (server-side)` | gap |
| J4 | `need: home timeline (own + followed, newest-first, paged)` | present |
| J4 | `need: post a chirp within a published length limit` | present |
| J4 | `need: reply and read conversations in context` | partial |
| J4 | `need: like/unlike, idempotent, accurate count` | present |
| J4 | `need: follow/unfollow to shape the timeline` | present |
| J4 | `need: discover/search for people and content to follow` | gap |
| J4 | `need: view a user's public profile` | present |
| J5 | `need: fast, low-friction daily re-entry` | partial |
| J5 | `need: graceful expired-session and transient-error recovery` | gap |
| J6 | `need: edit profile (display name, bio) after sign-up` | gap |
| J6 | `need: an account-settings surface (contact, notifications, sessions)` | gap |
| J6 | `need: a handle-change decision` | open question |
| J7 | `need: block and mute controls` | gap |
| J7 | `need: report abusive content or accounts` | gap |
| J7 | `need: stated rules/terms` | gap |
| J7 | `need: activity notifications with preferences` | gap |
| J8 | `need: self-service data export` | gap |
| J8 | `need: self-service account deletion` | gap |
| J8 | `need: a defined deletion outcome for content and handle` | gap |
| J8 | `need: confirmed, explained deletion` | gap |
| J9 | `need: trustworthy health/readiness signal` | partial |
| J9 | `need: durable user data across restart/redeploy` | gap (blocker) |
| J9 | `need: a supported administrative/support surface` | gap |
| J9 | `need: backup and recovery for durable data` | gap |
| J10 | `need: a runnable, documented self-host image` | present |
| J10 | `need: durability/account-lifecycle parity for self-hosted instances` | gap (conditional) |

The handles marked **critical** (handle-only authentication) and **blocker** (in-memory storage) are
the two findings that most directly break a releasable-product assumption: that an account is enterable
only by its owner, and that what a person creates is still there next time. Every other gap is a
surface a releasable product of this kind is expected to have; where the product kind makes one
out of scope, it is recorded out of scope with a reason (Part 5) rather than left silently absent.

---

## 4. Requirements & acceptance criteria

This part turns the needs surfaced by the product definition and the user journeys into a register of
testable product requirements. Each requirement is stated as *what must be true from the user's or
operator's side* — never how it is built — with an acceptance criterion a person who never met the
author can check against the live product. Where the detailed requirement belongs to another design
area, the product-level requirement is stated here and the area that carries the detail is named,
without minting that area's ids.

**How to read a requirement.** Each carries an `id`, a `statement` (the condition that must hold), an
`acceptance_criteria` a stranger can exercise on the live product, and a `release_gating` flag — `true`
when a releasable product of this kind depends on it, `false` when it is in scope and expected but a
first release could ship without it. A `false` flag is never "won't build"; it is "the release bar need
not block v1 on it," and each carries its reason. The final MVP-vs-releasable staging of the `false`
items is the release-bar design's to settle (`BAR-*`); this register provides the requirements and the
release-gating flags to stage.

chirp is held to the bar of a **releasable consumer web application**, not a prototype: absent an
explicit reduction by the engagement, "it deploys and the core loop runs" is not the bar. That standard
is what the `release_gating: true` flags below encode.

### The core loop — what chirp is for

#### REQ-PROD-001 — Post a chirp
- **statement**: A signed-in person can publish a short public message (a *chirp*) up to the published length limit of 280 characters, attributed to them; an empty or over-limit message is rejected and nothing is posted.
- **acceptance_criteria**: Signed in, compose a message of 280 characters or fewer and publish it; it appears attributed to your handle. Compose a message over 280 characters, or an empty one, and submit; the product reports the problem against the compose field and no chirp is created.
- **release_gating**: true

#### REQ-PROD-002 — Read the home timeline
- **statement**: A signed-in person can read a timeline of their own chirps and those of the people they follow, newest first, in bounded pages rather than all at once.
- **acceptance_criteria**: Signed in and following at least one person who has posted, open the timeline: recent chirps appear newest-first. Request the next page and older chirps load; the product returns pages (it does not attempt to return every chirp in one response).
- **release_gating**: true

#### REQ-PROD-003 — Reply to a chirp
- **statement**: A person can reply to an existing chirp, and the reply records the chirp it answers; replying to a chirp that no longer exists is reported, not silently dropped.
- **acceptance_criteria**: Reply to a visible chirp; the reply is published and identifies its parent. Attempt to reply to a deleted or nonexistent chirp; the product reports not-found rather than appearing to succeed.
- **release_gating**: true

#### REQ-PROD-004 — Read a reply in the conversation it belongs to
- **statement**: A person can move from a reply to the chirp it answers and see the conversation it belongs to.
- **acceptance_criteria**: Given a reply, navigate to its parent chirp and see the two in relation (the reply presented as answering that parent).
- **release_gating**: false — the parent reference is already recorded (REQ-PROD-003); reading replies as a navigable conversation is an enhancement a first release can omit. Richer multi-level threading is out of scope (see Part 5). The release bar decides whether v1 includes conversation reading.

#### REQ-PROD-005 — Like and unlike
- **statement**: A person can like and unlike a chirp; liking is idempotent and the like count reflects the result.
- **acceptance_criteria**: Like a chirp; the count increases by one. Like it again; the count does not increase again. Unlike it; the count returns.
- **release_gating**: true

#### REQ-PROD-006 — Follow and unfollow
- **statement**: A person can follow and unfollow another user to shape their timeline; following is idempotent and a person cannot follow themselves.
- **acceptance_criteria**: Follow another user; that user's new chirps appear in your timeline. Follow again; nothing is duplicated. Unfollow; their chirps leave your timeline. Attempt to follow your own account; it is rejected.
- **release_gating**: true

#### REQ-PROD-007 — View a user's public profile
- **statement**: Anyone can view a user's public profile — handle, display name, bio, and follower/following counts — without signing in.
- **acceptance_criteria**: Open a user's profile with no session; it shows the handle, display name, bio, and follower/following counts.
- **release_gating**: true

### Accounts and the identity lifecycle
*(Product-level requirements. The credential, session, and authentication mechanisms are the identity/security design's detail — `REQ-ID-*` — not minted here.)*

#### REQ-PROD-008 — Real sign-up that establishes a credential
- **statement**: A new person can create an account with a unique, well-formed handle and a display name (bio optional), and sign-up establishes a secret the person will later use to prove the account is theirs — not a handle alone.
- **acceptance_criteria**: Sign up with a fresh, well-formed handle; the account is created. Verify, via REQ-PROD-010, that knowing the handle alone does not let someone else sign in as the account — i.e. sign-up established a credential beyond the handle.
- **release_gating**: true

#### REQ-PROD-009 — Handle validation and uniqueness feedback at sign-up
- **statement**: Sign-up enforces handle format (up to 15 letters, digits, or underscores) and uniqueness, reporting a taken or ill-formed handle against the field so the person can fix it.
- **acceptance_criteria**: Submit a handle already in use; the product reports a conflict naming the handle. Submit a handle with a space, a disallowed character, or more than 15 characters; the product reports a field-level validation error. Submit a valid, unused handle; it is accepted.
- **release_gating**: true

#### REQ-PROD-010 — Sign-in only the account owner can pass
- **statement**: Signing in requires proving the account is the person's own; presenting a known handle alone must not grant a session.
- **acceptance_criteria**: Knowing only another account's handle (not its credential), attempt to sign in as it; no session is granted. The legitimate owner, presenting the credential set at sign-up or recovery, is signed in.
- **release_gating**: true

#### REQ-PROD-011 — Clear not-found vs. failed-credential signal at sign-in
- **statement**: Sign-in lets a person tell "no such account" from "credential did not match," enough to distinguish a typo from a wrong secret, without exposing which accounts exist beyond what the product already makes public.
- **acceptance_criteria**: Sign in with an unknown handle; the result lets you tell the account does not exist so you can proceed to sign-up. Sign in with a known handle and a wrong credential; the result is a failed sign-in you can retry or recover from.
- **release_gating**: true

#### REQ-PROD-012 — A verified point of contact for recovery and notifications
- **statement**: The product captures and verifies a point of contact for an account, kept minimal and privacy-respecting, so recovery and important notifications have a basis.
- **acceptance_criteria**: During sign-up or in settings, provide a contact; the product confirms you control it (a verification step) before relying on it for recovery.
- **release_gating**: true — account recovery (REQ-PROD-013) depends on it. What contact is collected and how it is protected is the identity/privacy design's detail.

#### REQ-PROD-013 — Self-service account recovery
- **statement**: A person who has lost their credential can regain access themselves, through the verified contact, without another person acting on their behalf.
- **acceptance_criteria**: Having lost your credential, use a "can't get in?" path, prove control of the verified contact, and regain access. Someone without that verified contact cannot recover the account.
- **release_gating**: true

#### REQ-PROD-014 — Sign-out that ends the session server-side
- **statement**: A person can end their session so it can no longer be used — important on a shared or public device.
- **acceptance_criteria**: Sign out; then present the session that was in use (e.g. replay the prior request) — it is rejected. The next person at that device is not signed in as you.
- **release_gating**: true

#### REQ-PROD-015 — Stay signed in across ordinary visits on a trusted device
- **statement**: On a device a person chooses to trust, they remain signed in across ordinary visits, subject to session expiry and to sign-out.
- **acceptance_criteria**: Sign in, close and reopen the app on the same device within the session's validity; you are still signed in. After expiry or sign-out, you are returned to sign-in.
- **release_gating**: false — a releasable product may require sign-in each visit; staying signed in is an expected convenience the experience/identity design tunes. Recorded so the choice is explicit rather than assumed.

### Managing the account

#### REQ-PROD-016 — Edit profile after sign-up
- **statement**: A person can change their display name and bio after sign-up.
- **acceptance_criteria**: Signed in, change your display name and bio; the change persists and shows on your profile on the next view.
- **release_gating**: true

#### REQ-PROD-017 — An account settings surface
- **statement**: A person has a settings surface for account-level choices — at minimum their verified contact and their active sessions/devices, plus notification preferences where notifications exist.
- **acceptance_criteria**: Signed in, open settings; view and update your verified contact, and view and end your active sessions. Where activity notifications exist (REQ-PROD-029), set their preferences here.
- **release_gating**: true — the surface the recovery-contact and sign-out requirements imply must exist. The exact settings are shared with the identity and experience designs.

#### REQ-PROD-018 — A stated handle-change policy
- **statement**: The product states whether and how a handle can change after sign-up, and behaves consistently with that stated policy.
- **acceptance_criteria**: Find the product's stated position on handle changes; the product's behavior matches it — either a handle change is offered and works, or the handle is stated as fixed and no path changes it.
- **release_gating**: false — this is a decision-and-consistency requirement (handles are the identifier others follow by); the product-definition/identity designs make the call. A first release may ship with handles fixed, provided that position is stated in the product.

### Leaving — export and deletion

#### REQ-PROD-019 — Self-service data export
- **statement**: A person can obtain their own data — their chirps, profile, and who they follow — themselves, in a portable form.
- **acceptance_criteria**: Signed in, request your data; you receive your posts, profile, and follow list in an openable, portable form (a file or download you can read outside chirp).
- **release_gating**: true

#### REQ-PROD-020 — Self-service account deletion
- **statement**: A person can delete their own account and have their content removed.
- **acceptance_criteria**: Signed in, delete your account; afterward the account can no longer sign in, and your content is removed per the stated deletion outcome (REQ-PROD-021).
- **release_gating**: true

#### REQ-PROD-021 — A defined deletion outcome for content and handle
- **statement**: The product defines, and applies, what deletion does to the person's chirps that others have replied to or liked, and whether the handle becomes available again.
- **acceptance_criteria**: Find the stated deletion outcome. After a deletion, the product's behavior matches it — what remains (or not) of replied-to/liked chirps, and whether the freed handle can be re-registered, is as stated.
- **release_gating**: true — deletion is not releasable without a defined outcome. The privacy and identity designs own the specifics.

#### REQ-PROD-022 — Deletion is confirmed and explained
- **statement**: Account deletion is confirmed before it happens and tells the person what will and will not survive, including that export is their chance to keep a copy.
- **acceptance_criteria**: Initiate deletion; a confirmation explains the consequences and points to export, and deletion proceeds only after explicit confirmation. Cancel at the confirmation; nothing is deleted.
- **release_gating**: true

### Authorization and privacy
*(Product-level. The detailed authorization model and threat analysis are the identity/security design's — `REQ-ID-*`.)*

#### REQ-PROD-023 — A clear authorization boundary
- **statement**: A person can act only as themselves and can change only their own account and content; there is a clear boundary on who may act as an account and who may change what.
- **acceptance_criteria**: Signed in as one account, attempt through the product's surfaces to post/reply/like/follow as another account, to edit another's profile or settings, or to delete another's content; each is refused.
- **release_gating**: true

#### REQ-PROD-024 — Privacy basics
- **statement**: Private account details — a person's contact, credential, and session data — are never exposed to other users or in public responses; only intended public fields (handle, display name, bio, public chirps, counts) are public.
- **acceptance_criteria**: View another user's profile and any public responses; only public fields appear. A person's contact, credential, and session data are never returned to another user.
- **release_gating**: true

### Safety, moderation, and support
*(Product-level. The abuse model and operator tooling detail are shared with the identity/security and operational-readiness designs — `REQ-ID-*` / `REQ-OPS-*`.)*

#### REQ-PROD-025 — Report abusive content or accounts
- **statement**: A person can report a chirp or an account that breaks the rules, and sees that the report was received.
- **acceptance_criteria**: Signed in, submit a report against a chirp and against an account; each submission is acknowledged as received.
- **release_gating**: true — a public space where strangers post at each other is not releasable with no way to report a problem.

#### REQ-PROD-026 — Stated rules a report can reference
- **statement**: The product states the rules — what is and is not allowed — that a report references.
- **acceptance_criteria**: Find the rules/terms within the product; a report can point to them.
- **release_gating**: true

#### REQ-PROD-027 — Controls to stop unwanted interaction (block/mute)
- **statement**: A person can stop unwanted interaction from another account — at minimum block or mute.
- **acceptance_criteria**: Signed in, block or mute another account; afterward that account's interaction with you, or its visibility to you, is suppressed per the control's stated effect.
- **release_gating**: false — expected, but a first release may ship with reporting (REQ-PROD-025) and operator action (REQ-PROD-028) as the safety floor and add block/mute as a fast follow. The release bar decides which unwanted-interaction controls gate v1; at least one safety control (reporting) is gated above.

#### REQ-PROD-028 — A supported administrative/support surface
- **statement**: The operator has a minimal, supported way to act on accounts and content — assist a user (e.g. recovery), remove content, and action an abusive account — without hand-editing the datastore.
- **acceptance_criteria**: As the operator, through a supported interface, assist a user's recovery, remove a named chirp, and disable or action an abusive account; none of these requires editing stored data by hand.
- **release_gating**: true

### Notifications

#### REQ-PROD-029 — Activity notifications with preferences
- **statement**: A person is notified of relevant activity — replies, follows, and likes directed at them — and can set which of these they are notified about.
- **acceptance_criteria**: Have another account reply to, follow, or like you; you see a notification of it. In settings, turn one category off; notifications of that category stop.
- **release_gating**: false — a baseline notifications story is in scope and expected, but a first release can ship without it. The release bar decides whether v1 gates on notifications.

### Experience quality
*(Product-level. The screens, visual system, and the exact accessibility standard are the experience design's — `REQ-UX-*`.)*

#### REQ-PROD-030 — A public first-visit experience
- **statement**: An unregistered visitor can understand what chirp is and reach sign-up without an account, rather than meeting an authentication wall.
- **acceptance_criteria**: Arrive with no account and no session; you see an explanation of what chirp is and a clear path to sign up, without being required to sign in first.
- **release_gating**: true — a consumer product that dead-ends unregistered visitors at a sign-in wall is not releasable.

#### REQ-PROD-031 — A low-commitment path from first visit to sign-up
- **statement**: The path from first visit to a created account is short and asks for no more than the account needs.
- **acceptance_criteria**: From the landing experience, reach a created account through a sign-up that requests only a handle, a display name, an optional bio, and the credential and contact the account requires — nothing extraneous.
- **release_gating**: false — the gated sign-up behavior is REQ-PROD-008/009; the brevity of the path is a quality expectation the experience design tunes.

#### REQ-PROD-032 — First-run onboarding off the empty state
- **statement**: A brand-new account is helped off its empty timeline — the product explains the empty state and offers at least one way to reach a non-empty timeline (find people to follow, prompt a first post) rather than showing a blank page.
- **acceptance_criteria**: Immediately after sign-up, with no follows and no posts, you see guidance explaining the empty state and at least one actionable way toward a non-empty timeline, not a blank screen.
- **release_gating**: true — a releasable consumer product does not drop every new account onto a blank page.

#### REQ-PROD-033 — Considered empty, loading, and error states
- **statement**: Empty, loading, and error states across the product are explained rather than shown as blanks or silent failures.
- **acceptance_criteria**: Reach an empty list, a loading list, and a failed action; each presents a state you can understand and act on, rather than a blank area or an unexplained failure.
- **release_gating**: true

#### REQ-PROD-034 — Graceful expired-session and transient-error recovery
- **statement**: An expired or invalid session returns the person to sign-in with their place explained; a transient error is recoverable by retry without losing in-progress work such as a drafted chirp.
- **acceptance_criteria**: With an expired session, take an action; you are routed to sign-in with an explanation, not a silent failure. On a transient server error while posting, retry without losing the drafted text.
- **release_gating**: true

#### REQ-PROD-035 — Works on a phone
- **statement**: The core loop works in a phone browser at a phone-sized viewport.
- **acceptance_criteria**: On a typical phone viewport, read the timeline, compose and post, reply, like, follow, and view a profile, with no broken layout or unreachable controls.
- **release_gating**: true

#### REQ-PROD-036 — An accessibility baseline
- **statement**: The core loop is usable with assistive technology — operable by keyboard, with accessible names and roles for its controls — to a baseline the experience design sets.
- **acceptance_criteria**: Using keyboard navigation and a screen reader, complete the core loop (read the timeline, compose/post, reply, like, follow, view a profile); controls expose accessible names and roles.
- **release_gating**: true — a releasable consumer product meets an accessibility baseline; the exact standard (e.g. a WCAG level) is the experience design's to fix.

### Durability and operational readiness
*(Product-level, stated from the user's and operator's side. The persistence model is the domain design's — `REQ-DOM-*`; backups, limits, observability, and availability targets are the operational-readiness design's — `REQ-OPS-*`.)*

#### REQ-PROD-037 — Durable user data across restart and redeploy
- **statement**: Accounts, chirps, follows, likes, and sessions survive the service being restarted, updated, and redeployed — a person's account and content are still there on the next visit. This requirement holds for any deployment of chirp.
- **acceptance_criteria**: Create an account and some chirps, follows, and likes; have the service restarted or redeployed; sign in again — the account still exists and its chirps, follows, and likes are still present.
- **release_gating**: true — the in-memory store at `c805525`, which loses everything on restart, is a release blocker, not a style choice.

#### REQ-PROD-038 — Backup and recovery of durable data
- **statement**: The durable user data can be backed up and restored, so it can be recovered after a failure.
- **acceptance_criteria**: Operating the service, produce a backup of user data, then restore from it; accounts, chirps, and follows are brought back.
- **release_gating**: true

#### REQ-PROD-039 — A trustworthy health/readiness signal
- **statement**: The product exposes a health/readiness signal an operator can trust to mean "serving users," not merely "the process is running."
- **acceptance_criteria**: Query the signal while the product cannot actually serve users (e.g. its durable store is unavailable); it reports not-ready. Query it while the product can serve users; it reports ready.
- **release_gating**: true — the `/healthz` liveness probe at `c805525` reports process liveness only; a readiness meaning is required.

#### REQ-PROD-040 — Reachable and reasonably available
- **statement**: The product is reachable on the web and reasonably available when people return.
- **acceptance_criteria**: Over a sustained period, the product responds at its address with the core loop working; unavailability is the exception, not the norm (the availability target is the operational-readiness design's).
- **release_gating**: true

#### REQ-PROD-041 — Limits and quotas as a first line against abuse
- **statement**: The product enforces basic rate and volume limits so one actor cannot trivially overwhelm the service or flood others.
- **acceptance_criteria**: Exceed a reasonable rate of posting or requests; the product throttles or rejects the excess rather than serving it without limit.
- **release_gating**: true — a public write surface with no limits is a releasability concern for an internet-facing product.

### Finding people

#### REQ-PROD-042 — A way to find people and content to follow
- **statement**: The product offers some way to discover and find other users to follow, beyond already holding a link to a profile.
- **acceptance_criteria**: Signed in and holding no external link, find at least one other user to follow through the product itself (for example a search or a discovery/suggestion surface).
- **release_gating**: false — a first release can ship with follow-by-profile and follow-from-a-reply and add discovery as a near-term follow-up. This is the main social-completeness gap; the release bar should decide explicitly what minimum discovery v1 includes rather than leaving it absent by default.

### Traceability — needs and journeys to REQ-PROD

This maps the upstream needs and journeys to the ids above, so the coverage and
journey-to-requirement relationships are checkable. The product kind's implications and every
end-to-end journey resolve to at least one `REQ-PROD-*` or to a recorded out-of-scope position.

#### Product-kind implications → requirement or out-of-scope

| Kind implication (need handle) | Resolution |
|---|---|
| `need: real-account-signup-signin` | REQ-PROD-008, 010 |
| `need: account-recovery` | REQ-PROD-013 (via 012) |
| `need: authenticated-identity` | REQ-PROD-010 |
| `need: durable-account-data` | REQ-PROD-037 |
| `need: account-settings` | REQ-PROD-016, 017 |
| `need: account-deletion` | REQ-PROD-020, 021, 022 |
| `need: data-export` | REQ-PROD-019 |
| `need: authorization-boundary` | REQ-PROD-023 |
| `need: privacy-basics` | REQ-PROD-024 |
| `need: considered-states` | REQ-PROD-033, 034 |
| `need: works-on-mobile` | REQ-PROD-035 |
| `need: reachable-and-available` | REQ-PROD-040 |
| `need: help-and-abuse-reporting` | REQ-PROD-025, 026, 028 |
| `need: no-tenancy-or-billing` | Out of scope — tenancy/billing (reason in Part 5) |
| `need: releasable-bar-not-prototype` | The `release_gating: true` flags throughout; stated in scope in Part 5 |

#### Journeys → REQ-PROD

| Journey | REQ-PROD |
|---|---|
| J1 First visit | 030, 031, 007 |
| J2 Sign-up | 008, 009, 012, 031 |
| J3 Sign-in | 010, 011 |
| J3a Recovery | 013, 012 |
| J3b Sign-out | 014 |
| J4 Core loop (post/read/reply/like/follow/profile) | 001, 002, 003, 004, 005, 006, 007, 042 |
| J5 Daily & returning use | 015, 032, 033, 034 |
| J6 Managing the account | 016, 017, 018 |
| J7 Safety & notifications | 025, 026, 027, 029 |
| J8 Leaving (export & deletion) | 019, 020, 021, 022 |
| J9 Operating the service | 037, 038, 039, 040, 028, 041 |
| J10 Self-hosting *(product-kind-dependent)* | REQ-PROD-037 holds for any deployment, so an instance someone runs inherits durability and the account-lifecycle requirements; chirp as a *supported self-hosting product* is out of scope (reason in Part 5). |

Two findings from the journeys break a basic releasable-product assumption and are gated
accordingly: handle-only authentication (REQ-PROD-010) — an account must be enterable only by
its owner — and the in-memory store (REQ-PROD-037) — what a person creates must still be there
next time.

---

## 5. In scope / out of scope

This consolidates the product-kind scope view from Part 1 with what the requirements in Part 4 settle.
chirp's one applicable product kind is a **hosted, public, account-based consumer web application**;
three other kinds (multi-tenant SaaS, commercial off-the-shelf, open-source product) do not apply, and
each shapes what is correctly left out.

### In scope — what a releasable chirp of this kind includes

- **The core loop, intact**: post a chirp including replies, read a paginated home timeline,
  view profiles, follow/unfollow, like/unlike (REQ-PROD-001–007). This exists at `c805525` and
  is the product's reason to exist.
- **Real accounts and the full account lifecycle**: sign-up that establishes a credential,
  owner-only sign-in, a verified recovery contact, self-service recovery, server-side sign-out,
  profile editing, a settings surface, data export, and account deletion with a defined,
  confirmed outcome (REQ-PROD-008–022). The authentication, session, and credential mechanisms
  are the identity/security design's detail.
- **An authorization boundary and privacy basics** (REQ-PROD-023–024): a person acts only as
  themselves, and private details stay private.
- **Safety, moderation, and operator support** (REQ-PROD-025–028): reporting and stated rules
  as the safety floor, a supported operator surface to assist users and action abuse, and
  controls to stop unwanted interaction (block/mute) as an expected near-term addition.
- **A baseline notifications story** (REQ-PROD-029): told when replied to, followed, or liked,
  with preferences.
- **A finished-feeling experience** (REQ-PROD-030–036): a public first-visit experience, a
  short path to sign-up, first-run onboarding off the empty state, considered empty/loading/
  error states, graceful session-expiry and transient-error recovery, a responsive layout that
  works on a phone, and an accessibility baseline. The screens, visual system, and exact a11y
  standard are the experience design's.
- **Durability and operational readiness to be a live service** (REQ-PROD-037–041): durable
  user data across restart and redeploy, a backup and recovery path, a trustworthy
  health/readiness signal, reachability and reasonable availability, and basic limits/quotas.
  The persistence model is the domain design's; backups, observability, limits, and
  availability targets are the operational-readiness design's.
- **A way to find people to follow** (REQ-PROD-042), at least minimally.

Requirements flagged `release_gating: false` above (REQ-PROD-004, 015, 018, 027, 029, 031, 042)
are **in scope and expected** but need not block a first release; the release-bar design stages
them across usable-early vs. releasable. Their being non-gating is a deliberate, reasoned call,
not an omission.

The detailed requirements and acceptance criteria owned by the sibling design areas —
`REQ-DOM-*` (domain & persistence), `REQ-UX-*` (experience), `REQ-ID-*` (identity/security),
`REQ-OPS-*` (operational readiness) — and the consolidated release bar (`BAR-*`) are out of
*this document's* scope but in scope for the design overall. This document states the
product-level requirement from the user's side and names the area that carries the detail; it
mints no other area's ids.

### Out of scope — with reasons

- **Tenancy, organizations, team management, and billing.** chirp is a single public product of
  individual accounts, and no money changes hands; the multi-tenant SaaS kind does not apply, so
  the tenancy, seat/team management, org-admin, and billing surfaces a SaaS buyer would assume
  are out of scope (`need: no-tenancy-or-billing`, as an explicit non-goal).
- **Customer-side install, licensing, and upgrade tooling.** chirp is operated as a hosted
  service, not distributed for customers to run in their own environment; the COTS kind does not
  apply.
- **Open-source community infrastructure** — a contribution guide, public issue/PR intake, and a
  self-host support commitment. The repository is a closed demonstration that does not accept
  contributions, so the OSS product kind does not apply. The container is self-hostable in
  principle and REQ-PROD-037 and the account-lifecycle requirements are written to hold for any
  deployment, so an instance someone runs inherits them; but chirp is not productized as a
  supported self-hosting product, so supporting self-hosters as a user base (persona P5, journey
  J10) is out of scope.
- **Native mobile applications.** chirp is a responsive web app (REQ-PROD-035); native iOS/
  Android clients are a separate product surface beyond the stated aim.
- **Rich social features beyond the core loop** — direct messages, full-text search beyond basic
  discovery, media and image uploads, trending/discovery feeds, verified badges, and multi-level
  threaded conversations beyond simple replies and reading a reply in context. These are
  candidates for later scope, not v1 commitments; a releasable chirp can ship without them.
  Basic discovery (REQ-PROD-042) and recording a reply's parent (REQ-PROD-003) are in scope; the
  richer forms are not. The release-bar design decides explicitly which, if any, a first release
  includes, rather than treating their absence as a defect.
- **The implementation "how"** — the persistence engine, the authentication protocol internals,
  framework internals, and every other build decision — is out of scope for the design and
  deferred to later code work. Each is stated here, where it belongs, as a requirement with a
  testable bar.
