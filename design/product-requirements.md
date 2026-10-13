# chirp — Product Requirements

This document is the foundation of the chirp design: what the product must be, for whom, and why, and
what "done" looks like from the user's side. Every other design area (domain & application design,
experience design, identity & trust, operational readiness, the release bar) traces back to the
requirements registered here.

**What this document settles, and what it does not.** It states *product-level* requirements — what
chirp must do and be — not *how* it is built. The persistence engine, the authentication protocol, the
framework internals, and the screen-level visual design are deliberately left to the sibling design
areas and to later implementation work. Where a requirement's detailed mechanism belongs to another
area, this document states the product-level requirement from the user's or operator's side and names
that area; the area carries its own `REQ-DOM-*` (domain & application design), `REQ-UX-*` (experience
design), `REQ-ID-*` (identity & trust), or `REQ-OPS-*` (operational readiness). This document is the
sole owner of the `REQ-PROD-*` register.

**Grounding.** Observations of "the product as it exists today" are grounded in the repository
`Nousix-LLC/nswe-demo` at commit `c805525`. They are stated so the gaps are real, not hypothetical.

The five sections below are the required sections of this artifact:
`product_definition_and_kind`, `users_personas`, `user_journeys`,
`requirements_and_acceptance_criteria`, and `in_scope_out_of_scope`.

---

## 1. Product definition & kind

*(section: `product_definition_and_kind`)*

### What chirp is

chirp is a **public, account-based microblogging web application** — a Twitter-style product. A
person visits it in a browser, creates an account under a chosen handle, writes short text posts
("chirps"), follows other people, reads a reverse-chronological **home timeline** of the people they
follow, and likes and replies to posts. It is a social reading-and-posting product: the thing it is
*for* is letting someone say something short in public, build a small following, and keep up with the
people they follow.

chirp is delivered as a **single web application**: one self-contained service that serves both the
browser app (a single-page app the user actually interacts with) and the REST API behind it. A user
needs nothing but a URL and a browser; there is no separate client to install.

### What exists today (grounded at `c805525`)

The definition above is grounded in the product as it stands in `Nousix-LLC/nswe-demo` at commit
`c805525`. Described from the user's side, the live product today lets a person:

- **Create an account** with a handle, a display name, and an optional bio.
- **"Sign in"** as a handle — and here is the most important gap in the current product: sign-in is
  **handle-only**. A visitor becomes a given user simply by presenting that user's handle; there is
  **no password or any other credential** (the server even notes real credential exchange is
  deliberately left out). In its current form the product cannot tell the real owner of a handle
  from anyone who types it.
- **Post a chirp** (short validated text, optionally as a reply to another chirp).
- **Read a home timeline** of the accounts they follow, newest first, a page at a time.
- **Follow and unfollow** other users; **like and unlike** chirps; **view a user's profile**.

The browser app presents this as three places: a timeline view, a compose/sign-in view, and a user
profile view. The service is packaged as one container image and runs as a reachable deployment.

Two facts about the current product matter to the definition because users will feel them directly:

1. **Nothing a user creates survives a restart.** The product keeps all accounts, chirps, follows,
   and likes **in memory only** — there is no database. When the service restarts or redeploys,
   every account and every post is gone. For a product whose whole purpose is to let people build a
   presence and a following over time, this is a defining limitation, not an implementation detail.
   *(The concrete persistence requirement and its "how" are owned by the domain and operational
   design areas; stated here only as the product-kind expectation that a user's data endures.)*

2. **An account cannot be fully lived in or left.** There is no way to recover access, change
   account settings, delete the account, or take one's data out. These are defined below as
   expectations the product's kind carries.

### Which product kind(s) apply, and what each makes users assume

Of consumer web app / multi-tenant SaaS / commercial off-the-shelf (COTS) / open-source project, named
concretely for *this* product:

**Primary kind — a consumer web application.** chirp is a public-facing product used by individual
people with their own personal accounts. This is the dominant kind and it sets the baseline of what
users assume without being told:

- *I can sign up myself,* in the browser, without being invited or provisioned by anyone.
- *This account is mine.* Signing in proves I am me; someone who merely knows my handle cannot post
  as me. (The live product does not yet meet this assumption — see the handle-only sign-in gap.)
- *What I create will still be here tomorrow.* My posts, my follows, my account persist; they are
  not quietly discarded.
- *I can get back in if I lose access,* and *I can manage and eventually close my account,* and
  *I can take my data with me when I leave.*
- *The product is reasonably safe and private* with the data I give it, and tells me what it holds
  about me.

**A self-hostable, single-image deployable product (a COTS/OSS-adjacent surface).** chirp is also
distributed as **public source plus one self-contained container image with a documented
build-run-deploy path** (a multi-stage image build, and deployment manifests under `deploy/`). That
distribution shape carries a second, narrower set of expectations for *whoever runs it*:

- *I can build it reproducibly and run it from the documented steps,* without hidden prerequisites.
- *I can configure it for my environment* through documented settings, and *operate it* (observe its
  health, recover its data) from what the repository provides.

This surface is real and worth designing to, because the product is handed over as something
installable and operable — not only as a hosted site.

**Kinds that do NOT apply to chirp (recorded with reasons):**

- **Multi-tenant SaaS — not applicable.** chirp has no notion of an organization, team, or tenant,
  no per-tenant isolation, and no subscription. It is a single shared consumer social product, not a
  product sold to organizations as isolated tenants. *Watch-point for later:* if chirp ever grew
  team or organization accounts, that would reintroduce this kind and its expectations (tenancy,
  roles, admin, audit) — out of scope for this release.
- **Commercial off-the-shelf (COTS) product — not applicable as a commercial offering.** chirp is a
  closed demonstration; it is not sold, licensed, or supported as a purchasable product. The
  *self-hostable deployable* expectations above (reproducible build, documented operation) still
  apply because of how it is distributed — but the commercial COTS expectations (licensing,
  entitlement, versioned upgrades for paying customers, a support channel) do not.
- **Open-source community project — not applicable as a community project.** The repository is a
  **closed demonstration that explicitly does not accept contributions** and carries no open-source
  license. chirp therefore has the *outward shape* of public source (readable code, a reproducible
  build) without being run as an OSS project: there is no contributor workflow, governance, or
  community-support expectation to design for, and none should be assumed.

---

## 2. Users & personas

*(section: `users_personas`)*

chirp is a public, consumer-facing social posting product: a Twitter-style service anyone can read
and, once they have an account, post to. Its users divide cleanly into the people who come to read and
post, the people who have not signed up yet, and the person who keeps the service running. Each persona
below is given by what they are trying to do and the context they do it in — not by demographics for
their own sake.

### P1 — The unregistered visitor ("just looking")

**Who:** Someone who arrives at chirp for the first time — from a shared link, a search result, or
curiosity — with no account and no commitment.

**Goal:** Understand what chirp is, see whether there is anything worth reading, and decide in under a
minute whether to sign up. They want to *look before they join*.

**Context:** They judge the whole product in the first screen. If the first thing they meet is an empty
page, an error, or a demand to log in before they can see anything, they leave and do not come back.
This persona is the top of the funnel for every other persona; a product that neglects them has no new
users.

**What the product owes them:** a readable public timeline on arrival, a clear idea of what the product
is, and an obvious, low-friction way to get in.

> Current state (`c805525`): the timeline view does load for a logged-out visitor, but the only
> account-related control anywhere in the SPA is a *Log in* handle field on the compose view. There is
> **no sign-up screen** — a visitor who does not already have a handle has no in-product way to create
> one.

### P2 — The new registered user ("I just joined")

**Who:** A visitor (P1) who has just created an account and signed in for the first time.

**Goal:** Get to their first successful post and their first followed account quickly, and feel that
the account is really *theirs* — that what they do will still be there tomorrow.

**Context:** The first session after sign-up decides retention. A new user with an empty timeline, no
one to follow, and no guidance about what to do next has been handed a blank room. They also, often
unconsciously, expect that signing up *means something* — that their identity is protected and their
posts persist.

**What the product owes them:** a first-run that does not dead-end on emptiness, a way to find at least
a few accounts or posts worth following, and the baseline trust that their account and content survive.

### P3 — The returning everyday reader/poster ("the core user")

**Who:** The product's bread-and-butter user, back for their second, tenth, hundredth visit. The same
person acts as **reader** (scrolling the timeline, opening profiles) and as **poster** (composing
chirps, following, liking, replying).

**Goal:** On each visit: catch up on what's new, post what they have to say, and interact — follow new
accounts, like what they enjoy, reply into conversations.

**Context:** This persona lives in the core loops and notices friction instantly: a timeline that does
not remember where they were, a compose box that loses their draft, a like that silently fails, a
session that logs them out without warning. They also, eventually, want to manage their own presence —
edit their profile, change their mind about who they follow, and (one day) leave cleanly.

**What the product owes them:** fast, reliable core loops with honest feedback; continuity of session
and state across visits; and control over their own account and data over time.

*Two lenses on P3 worth calling out because the product treats them differently:* the **reader-only**
sub-case (a signed-in or signed-out person who only consumes) must be first-class — reading must never
require posting; and the **conversational** sub-case (replies) is latent in the data model
(`reply_to` exists on a chirp) but has no journey in the UI yet.

### P4 — The operator / administrator ("I keep it running")

**Who:** The person responsible for deploying chirp and keeping it available for P1–P3 — whoever
operates the single container image and its deployment. Because chirp ships as one self-contained
image, the operator is also the **deployer/self-hoster**: the same person stands the service up and
keeps it up.

**Goal:** Stand the service up, know whether it is healthy, and keep users' data and access intact —
with a way to recover when something goes wrong.

**Context:** The operator is a user of the product too, just through a different surface. Today that
surface is thin: a liveness probe that reports process liveness only. There is no administrative view
of the running product, no moderation capability, no visibility into users or content, and — because
the store is in-memory — **every restart discards all accounts, posts, and sessions**. The operator
currently cannot answer a user who writes in to say "my account is gone," because from the operator's
side there are no users to look at and nothing was ever persisted.

**What the product owes them:** a truthful health signal, durable data so that keeping the service
"up" actually keeps users' content, a recovery path, and enough of an operational/administrative view
to support users.

### A deliberately-absent persona worth naming: the moderator

A public posting product of this kind normally has a **moderator / trust-and-safety** role — someone who
can act on abusive content and accounts. chirp today has no such role, no reporting, and no moderation
surface. This is noted here as a real gap in the persona set rather than silently omitted; the formal
abuse/authorization requirements belong to identity & trust.

---

## 3. User journeys

*(section: `user_journeys`)*

Each journey is walked end to end — including the first visit, the account lifecycle, the core loops,
daily/returning use with its empty/error/second-visit states, and **leaving**. Observations of the
product as it stands are grounded at commit `c805525`. Where the detailed requirement belongs to a
sibling design area (detailed screens to experience design, authentication and account internals to
identity & trust, durable storage to domain & operational design), the user-side expectation is
recorded here and the mechanism is left to that area.

### J1 — First visit (unregistered visitor, P1)

1. **Arrival.** The visitor opens chirp and lands on the timeline. The product should, in this first
   screen, make clear what chirp is and show real content so there is a reason to stay.
   - *Empty state:* if there are no chirps yet (a fresh deployment, or one just restarted and wiped),
     the first screen is empty. A first-time visitor who sees nothing has no reason to return.
   - *Error state:* if the timeline fails to load, the visitor must see an honest, recoverable message,
     not a blank or broken page. (The timeline view does render a distinct error state today; the
     content of that message is an experience-design concern.)
2. **Exploring without an account.** The visitor can open a profile from a chirp. There is no search,
   directory, or "who to follow," so exploration is limited to following links that already exist on
   screen.
3. **Deciding to join.** When the visitor decides to sign up, they need an obvious control to do so.
   - *Current gap:* the SPA exposes only a *Log in* handle field and **no sign-up screen**; account
     creation exists only at the API. A real visitor cannot become a user through the product.

### J2 — Sign-up (visitor → new registered user, P1 → P2)

1. **Starting sign-up.** From the first screen or the sign-in screen, the visitor chooses to create an
   account.
2. **Providing identity.** Today an account is a username (handle, ≤15 characters), a display name, and
   an optional bio. There is **no email, no password, and no other credential** — the code states
   plainly that real credential exchange (passwords, OAuth) is out of scope at this commit. This means
   sign-up collects nothing that could later prove the account is yours.
   - *Validation state:* the username rules (length, allowed characters) must be enforced with clear,
     inline feedback at sign-up, not only deep in the API.
   - *Collision state:* two people may want the same handle. The journey must handle "that handle is
     taken" gracefully.
3. **First signed-in moment.** On success the user is signed in and lands somewhere purposeful.
   - *Empty-state risk:* a brand-new account follows no one and the timeline may be sparse. Dropping the
     new user onto an empty timeline is the classic new-user dead end.

### J3 — Sign-in and sign-out (returning user, P3)

1. **Signing in.** A returning user opens the compose view (the only place sign-in lives today), enters
   their handle, and receives a session token; the UI re-renders into its signed-in state.
   - *Current gap:* because sign-in is handle-only, **anyone who knows a handle can sign in as that
     user**. From the user's side this means their account is not actually protected.
   - *Error state:* a failed or unauthorized sign-in surfaces inline (the compose view does render a
     server error inline today); the message must be understandable.
2. **Staying signed in.** Across a session the user should know they are signed in and stay that way
   predictably. Sessions are held in memory today, so a restart silently invalidates every session and
   signs everyone out without notice.
3. **Signing out.** A user must be able to deliberately end their session — especially on a shared
   device.
   - *Current gap:* the app models a logged-out session state, but there is no visible **sign-out
     control** in the SPA's navigation.

### J4 — Account recovery (returning user who has lost access, P3)

1. **Losing access.** A user cannot get back into their account.
2. **Recovering.** They need a way to re-establish access.
   - *Current gap:* with handle-only sign-in there is nothing to recover — and equally nothing
     protecting the account — so there is no recovery journey at all today. Once real authentication
     exists, recovery becomes essential: a user who forgets or loses their credential must have a safe
     way back in. Recovery is one of the surfaces products routinely ship without.

### J5 — The core loops (everyday reader/poster, P3)

These are the loops the core user lives in. Each is walked with its empty, error, and loading states.

- **Read the timeline.** The user scrolls the latest chirps; cursor-based pagination offers "load
  more." States that must each be honest: *loading* (first fetch), *content* (the list), *empty* (no
  chirps yet — see J1), *error* (fetch failed), and *end of list* (no further pages). The timeline view
  implements loading/error/empty/paginated states today.
- **Compose and post.** A signed-in user writes a chirp (≤280 characters), which is validated locally
  before send, with busy/success/error feedback. *Draft safety:* inputs are uncontrolled so a re-render
  does not clobber typing; a user must not lose a draft to a transient error.
- **View a profile.** The user opens an account to see who it is. States: *loading*, *loaded*,
  *not-found* (no such user), *error*.
- **Follow / unfollow.** From a profile the user follows or unfollows; counts update reactively.
- **Like / unlike.** The user likes or unlikes a chirp; the like count updates.
- **Reply (latent).** The data model carries `reply_to`, but no screen lets a user read a thread or
  write a reply. Conversation is a defining part of a product of this kind.

**Cross-cutting discovery gap:** across all core loops there is **no search and no way to find people or
content** beyond links already on screen.

### J6 — Daily / returning use and second-visit states (P3)

1. **Coming back.** On a later visit the user expects to pick up roughly where they left off: still
   signed in (or an honest prompt to sign in again), their follows intact, and new content since last
   time visible.
   - *Second-visit continuity:* because data and sessions are in-memory today, a service restart between
     visits wipes the user's account, posts, follows, and session — the user returns to find themselves
     logged out and their account effectively gone. This is the single most damaging second-visit
     failure.
2. **Catching up.** The timeline should let the user see what is new since last time and move through it
   without losing their place.
3. **Managing presence.** Over repeated visits the user wants to adjust their profile (display name,
   bio) and reconsider who they follow.
4. **Empty/quiet states on return.** A user who follows few accounts may return to a quiet timeline;
   the product should still offer something to do (discover more to follow).

### J7 — Leaving: account deletion and taking data out (P3)

This is the journey products most often omit, and the one this persona's trust depends on most.

1. **Deciding to leave.** A user decides to stop using chirp and wants to remove themselves.
2. **Deleting the account.** They need a clear, confirmable way to delete their account and understand
   what deletion does to their posts, follows, and likes.
   - *Current gap:* there is **no account-deletion capability** anywhere in the product — no UI and no
     API route. A user who wants to leave cannot.
3. **Taking their data out.** Before (or instead of) leaving, a user may want to export what they have
   created — their chirps, their profile, their follow list.
   - *Current gap:* there is **no data-export capability**. A user cannot retrieve their own content in
     any portable form.
4. **After leaving.** The user should be able to trust that deletion is honoured and that they are
   actually signed out and removed.

### J8 — The operator / administrator journey (P4)

1. **Standing it up.** The operator deploys the single self-contained image and expects it to come up
   serving both the API and the SPA. (The container and deployment mechanics are owned by operational
   design; the operator-as-user journey is walked here.)
2. **Knowing it is healthy.** The operator checks health. Today the health probe reports process
   liveness only — it cannot report whether the product is actually working for users (no dependency or
   data check, because there is no durable dependency).
3. **Keeping data intact.** The operator's core duty is that keeping the service up keeps users' content.
   With an in-memory store, it does not: a restart or redeploy silently discards every account, chirp,
   follow, and session.
4. **Supporting and protecting users.** When a user asks for help, or when content must be acted on, the
   operator needs some view of and control over the running product. Today there is none: no admin view,
   no user lookup, no moderation.
5. **Recovering from failure.** When something breaks, the operator needs a path back to a working
   service with data intact.

### Journey coverage summary

| Journey | What it surfaces (summarised) |
|---|---|
| J1 First visit | logged-out first screen useful even when empty; discover beyond the timeline; obvious route to join |
| J2 Sign-up | create account in-product; real credential established; handle validation & collisions; onboarding past empty state |
| J3 Sign-in / sign-out | authenticate with proof not just a public handle; predictable sessions; deliberate sign-out |
| J4 Recovery | safe account-recovery path once real auth exists |
| J5 Core loops | reliable timeline/compose/profile/follow/like with honest states; replies if in product; search/discovery |
| J6 Daily / returning use | account & content intact across visits/restarts; see what's new; edit profile & manage follows |
| J7 Leaving | self-service account deletion; data export; deletion actually honoured |
| J8 Operator / admin | product-level health; data survives operations; admin/support & moderation; recovery path |

Each journey traces to at least one requirement in §4 (see the Journey → requirement coverage table).

---

## 4. Requirements & acceptance criteria

*(section: `requirements_and_acceptance_criteria`)*

The register below turns every need surfaced by the product definition (§1) and the user journeys (§3)
into a testable requirement. Each requirement states what must be true from the user's or operator's
side; its acceptance criteria are written so that a person who never met the author can exercise the
live product and reach the same yes/no. Each requirement carries a `release_gating` flag: **true** where
a releasable product of chirp's kind genuinely depends on it, **false** where it is a real improvement a
first releasable version can ship without. A `false` flag does not mean unwanted — it means an honest,
recorded cut.

**Boundary.** These are *product-level* requirements. Where the detailed requirement or its mechanism is
owned by a sibling design area, the requirement names that area; the area will carry its own
`REQ-DOM-*`, `REQ-UX-*`, `REQ-ID-*`, or `REQ-OPS-*`. No "how" is settled here.

### A. Getting in — identity & access

#### REQ-PROD-001 — Self-service sign-up inside the product
- **Statement:** A visitor can create their own chirp account entirely within the product's web UI, without being invited, provisioned, or using the raw API.
- **Acceptance criteria:** Starting as a logged-out visitor, a person can reach a "create account" control from the first screen and from the sign-in screen, supply the required account details, submit, and end up signed in as a newly created account — using only the product UI in a browser. (Today account creation exists only at the API with no sign-up screen; this requirement closes that gap.)
- **Release gating:** true
- **Traceability:** J1, J2; P1→P2. Detailed sign-up screen → experience design (REQ-UX-*).

#### REQ-PROD-002 — Handle validation with understandable feedback
- **Statement:** The product validates a chosen handle at sign-up and explains any rejection in terms the user can act on.
- **Acceptance criteria:** Entering a handle that breaks the rules (longer than the 15-character limit, or containing disallowed characters) produces a clear, specific inline message explaining why, at or before submission; entering a valid handle is accepted. The message does not require reading an API error to understand.
- **Release gating:** true
- **Traceability:** J2. Inline-feedback presentation → experience design (REQ-UX-*).

#### REQ-PROD-003 — Handle-uniqueness conflicts are surfaced clearly
- **Statement:** When a requested handle is already taken, the product says so and creates no duplicate.
- **Acceptance criteria:** Attempting to sign up with a handle that already exists yields a clear "that handle is taken" message and results in no second account and no overwrite of the existing one; choosing a different, free handle then succeeds.
- **Release gating:** true
- **Traceability:** J2.

#### REQ-PROD-004 — Signing in proves the account is yours
- **Statement:** Signing in requires proof that the account belongs to the person signing in; knowing a public handle is not sufficient to sign in as, or act as, that account.
- **Acceptance criteria:** A person who knows another user's handle but does not hold that user's credential cannot sign in as that account and cannot post, follow, like, or change settings as it; the legitimate owner, presenting their credential, can. (Today sign-in is handle-only — anyone who types a handle becomes that user; this requirement closes that gap.)
- **Release gating:** true
- **Traceability:** J3; P3. Authentication mechanism → identity & trust (REQ-ID-*).

#### REQ-PROD-005 — Deliberate sign-out
- **Statement:** A signed-in user can deliberately end their session from the product UI.
- **Acceptance criteria:** A signed-in user can find and use a visible sign-out control; after using it the product treats them as logged out (they must sign in again before they can post), which is observable on a shared device by a second person being unable to act as the first user's account afterward.
- **Release gating:** true
- **Traceability:** J3. Session invalidation → identity & trust (REQ-ID-*).

#### REQ-PROD-006 — Predictable session state; no silent sign-out
- **Statement:** A user can always tell whether they are signed in, and session expiry or invalidation is communicated rather than silent.
- **Acceptance criteria:** The UI consistently shows signed-in versus logged-out state; when a session has expired or been invalidated, the next action the user attempts surfaces a clear prompt to sign in again instead of failing silently or appearing to succeed. (Today sessions are lost without notice on a restart; this requirement removes the silent failure.)
- **Release gating:** true
- **Traceability:** J3, J6. Session lifetime/mechanism → identity & trust (REQ-ID-*).

#### REQ-PROD-007 — Account recovery path
- **Statement:** A user who loses access to their credential has a defined, safe way to regain access to their account.
- **Acceptance criteria:** A user who cannot sign in can start a recovery flow from the sign-in screen and, on completing it, regain access to the same account (same handle, posts, and follows); a person who does not control the account's recovery factor cannot gain access this way.
- **Release gating:** true
- **Traceability:** J4. Recovery mechanism → identity & trust (REQ-ID-*).

### B. Living in and leaving the account

#### REQ-PROD-008 — View and edit your own profile
- **Statement:** A signed-in user can see and change the basics of their own account.
- **Acceptance criteria:** A signed-in user can view their account's handle, display name, and bio, and can change the display name and bio; after saving, the changes are shown on their profile and persist on a later visit.
- **Release gating:** true
- **Traceability:** J6; P3. Profile-editing screens → experience design (REQ-UX-*).

#### REQ-PROD-009 — A home for managing the account
- **Statement:** The product gives a signed-in user a reachable place to manage their account.
- **Acceptance criteria:** From the signed-in navigation a user can reach an account-management area that gathers, at minimum, profile editing and the controls for account deletion and data export; the area is reachable without using the raw API.
- **Release gating:** true
- **Traceability:** J6, J7. Settings screen layout → experience design (REQ-UX-*).

#### REQ-PROD-010 — Self-service account deletion
- **Statement:** A user can delete their own account from the product and understand what deletion removes.
- **Acceptance criteria:** A signed-in user can initiate deletion of their own account, is shown a clear confirmation stating what will be removed (the account, its chirps, its follows, and its likes), and on confirming, the account can no longer be signed into. (Today there is no deletion UI or API route; this requirement adds the capability.)
- **Release gating:** true
- **Traceability:** J7. Deletion semantics & authorization → identity & trust (REQ-ID-*).

#### REQ-PROD-011 — Export your own data
- **Statement:** A user can take their own data out of the product in a usable, portable form.
- **Acceptance criteria:** A signed-in user can request an export and obtain their own data — at least their profile, their chirps, and their follow list — in a documented, machine-readable format they can download and open outside chirp.
- **Release gating:** true
- **Traceability:** J7. Export format/shape → identity & trust (REQ-ID-*) and domain & application design (REQ-DOM-*).

#### REQ-PROD-012 — Deletion is honoured
- **Statement:** After a user deletes their account, the account and its personal data are actually gone and the user is signed out.
- **Acceptance criteria:** Once deletion is confirmed, attempts to sign in as that account fail, its profile returns a not-found result, the deleting user is signed out, and the account's personal data is not served again through the product.
- **Release gating:** true
- **Traceability:** J7. Deletion guarantees → identity & trust (REQ-ID-*); durable removal → operational readiness (REQ-OPS-*).

### C. The core product loop

#### REQ-PROD-013 — Reading never requires an account
- **Statement:** Reading the public timeline and user profiles never requires signing in.
- **Acceptance criteria:** A logged-out visitor can load the home timeline and open a user profile and read their chirps without being forced to create an account or sign in first.
- **Release gating:** true
- **Traceability:** J1, J5; P1 and the reader-only sub-case of P3.

#### REQ-PROD-014 — A useful logged-out first screen, even when empty
- **Statement:** The first screen communicates what chirp is and stays useful even when there is little or no content.
- **Acceptance criteria:** On arrival, a logged-out visitor sees either real timeline content or, when there is none, a clear statement of what chirp is together with an obvious way to sign up or sign in — never a blank page or an unexplained error. If the timeline fails to load, an honest, recoverable message is shown instead of a broken screen.
- **Release gating:** true
- **Traceability:** J1. First-screen copy and layout → experience design (REQ-UX-*).

#### REQ-PROD-015 — An obvious route from reading to joining
- **Statement:** A visitor who decides to join always has a visible way to do so.
- **Acceptance criteria:** A "create an account" control is visible on the first screen and on the sign-in screen, and following it reaches sign-up (REQ-PROD-001).
- **Release gating:** true
- **Traceability:** J1, J2. Placement/affordance → experience design (REQ-UX-*).

#### REQ-PROD-016 — Compose and post a chirp
- **Statement:** A signed-in user can post a short text chirp.
- **Acceptance criteria:** A signed-in user can enter text up to the product's limit of 280 characters, submit it, and see the new chirp appear in the product; text beyond the limit is prevented with clear feedback, and the user is shown busy/success/error feedback on submission.
- **Release gating:** true
- **Traceability:** J5.

#### REQ-PROD-017 — Composing never silently loses the user's text
- **Statement:** A failed post is recoverable, with the user's draft preserved.
- **Acceptance criteria:** If submitting a chirp fails (for example, a transient error), the typed text remains in the compose field and the failure is reported, so the user can retry without retyping; an in-progress draft is not clobbered by the screen re-rendering.
- **Release gating:** true
- **Traceability:** J5.

#### REQ-PROD-018 — Home timeline with honest states and paging
- **Statement:** The home timeline clearly distinguishes loading, content, empty, error, and end-of-list, lets the user page through older chirps, and lets the user recover from an error without reloading the whole page.
- **Acceptance criteria:** A user can observe each state: a loading indicator on the first fetch; the list when chirps exist; a clear empty message when there are none; a recoverable error message on a failed fetch that offers a retry without a full page reload; a "load more" that fetches older chirps; and a clear end-of-list when no further pages remain.
- **Release gating:** true
- **Traceability:** J5, J1. State presentation → experience design (REQ-UX-*).

#### REQ-PROD-019 — Profile view handles its real states
- **Statement:** Opening a user profile handles loading, loaded, not-found, and error — not only the happy path.
- **Acceptance criteria:** Opening a profile shows a loading indicator while fetching; the profile when it loads; a clear not-found result for a handle/id that does not exist; and an honest, recoverable error message on failure.
- **Release gating:** true
- **Traceability:** J5. State presentation → experience design (REQ-UX-*).

#### REQ-PROD-020 — Follow and unfollow reflect the true state
- **Statement:** A user can follow and unfollow accounts, and the product reflects the real resulting state and reports failures honestly.
- **Acceptance criteria:** From a profile, a signed-in user can follow an account and later unfollow it; the follow state and the follower/following counts shown match the actual state after the action, and a failed follow/unfollow is reported rather than silently dropped.
- **Release gating:** true
- **Traceability:** J5, J6.

#### REQ-PROD-021 — Like and unlike are reliable
- **Statement:** A user can like and unlike a chirp, and the on-screen like count matches reality.
- **Acceptance criteria:** A signed-in user can like a chirp and later unlike it; the displayed like count matches the actual count after the action, and a failed like/unlike is reported rather than silently dropped.
- **Release gating:** true
- **Traceability:** J5.

#### REQ-PROD-022 — A new user's first session is not an empty dead-end
- **Statement:** After sign-up, the product guides the new user toward a first follow and a first post rather than dropping them onto a blank timeline.
- **Acceptance criteria:** Immediately after sign-up, a new user with an empty timeline is presented with a next action (for example, accounts to follow or a prompt to post) rather than only an empty screen.
- **Release gating:** false — a first releasable version can ship with a functional-but-empty first session; this onboarding is a real improvement, not a release blocker.
- **Traceability:** J2, J6; P2. Onboarding flow → experience design (REQ-UX-*).

#### REQ-PROD-023 — Replies: coherent if present
- **Statement:** If replies are part of the product, a user can read a conversation and reply to a chirp; if they are not, the product does not expose a half-built reply capability.
- **Acceptance criteria:** Either (a) a user can open a chirp, read its thread, and post a reply that appears in that thread; or (b) the product exposes no reply affordance and the latent reply capability in the data model is not surfaced to users. The product does not present a reply control that does nothing.
- **Release gating:** false — the first releasable version may defer a full reply experience, provided it does not ship a dead reply affordance; conversation is the most likely near-term addition.
- **Traceability:** J5; the conversational sub-case of P3. If built, reply screens → experience design (REQ-UX-*).

#### REQ-PROD-024 — Finding people and content
- **Statement:** A user can find accounts and content beyond the links already visible on screen.
- **Acceptance criteria:** A user can locate an account or content they are not already looking at — for example, by searching for a handle or browsing a directory or suggestions — rather than being limited to following links that happen to be on the current screen.
- **Release gating:** false — the first releasable version may launch with the core post/read/follow/like loop and links-on-screen discovery; search/discovery is recorded as a near-term addition rather than a release blocker.
- **Traceability:** J1, J5, J6.

#### REQ-PROD-025 — Returning users can tell what is new without losing their place
- **Statement:** A returning user can see what is new since last time and navigate it without losing their position.
- **Acceptance criteria:** On a return visit a user can distinguish new content from already-seen content and can move through the timeline (including paging) without being thrown back to the top or losing their scroll position.
- **Release gating:** false — a quality-of-life improvement; the core returning-use loop is usable without it.
- **Traceability:** J6.

#### REQ-PROD-026 — A user's account, posts, and social graph persist
- **Statement:** A user's account, chirps, follows, and likes survive service restarts and redeploys.
- **Acceptance criteria:** A user who creates an account, posts chirps, and follows accounts finds all of it intact after the service has been restarted or redeployed — the account can still be signed into and its posts and follows are still present. (Today all data is held in memory and is lost on restart; this requirement removes that loss.)
- **Release gating:** true
- **Traceability:** J2, J6, J8; P2, P3, P4. Persistence mechanism → domain & application design (REQ-DOM-*); durability in operation → operational readiness (REQ-OPS-*).

### D. Trust, safety, privacy and the legal surface

#### REQ-PROD-027 — Users can act on unwanted content and accounts
- **Statement:** A user can deal with unwanted content or accounts — at minimum, report them and reduce their own exposure.
- **Acceptance criteria:** A signed-in user can report a chirp or an account, and can take an action that reduces their own exposure to an unwanted account (for example, block/mute); the user receives confirmation that the action was taken.
- **Release gating:** true — a public product that lets anyone post cannot be released responsibly with no way for users to respond to abuse.
- **Traceability:** J5, J8; the deliberately-absent moderator persona. Enforcement/authorization → identity & trust (REQ-ID-*).

#### REQ-PROD-028 — Abuse can be acted on by whoever runs the product
- **Statement:** Someone operating chirp can act on abusive content and accounts.
- **Acceptance criteria:** An authorized operator/moderator can remove an abusive chirp and act on an abusive account (for example, suspend or remove it), and the effect is visible to users (the content no longer appears).
- **Release gating:** true
- **Traceability:** J8; the absent moderator persona. Moderation authorization/tooling → identity & trust (REQ-ID-*); operator surface → operational readiness (REQ-OPS-*).

#### REQ-PROD-029 — A privacy and terms surface
- **Statement:** The product states what data it holds about a user and the terms under which the product is used.
- **Acceptance criteria:** A user (logged-out or signed-in) can reach, from the product, a plain statement of what personal data chirp collects and holds about them and the terms of use; the statement is present and readable rather than absent.
- **Release gating:** true — a public product collecting personal data is expected to carry this surface.
- **Traceability:** Kind implication (consumer web app: "reasonably safe and private; tells me what it holds"); J7. Content/scope of the statement → identity & trust (REQ-ID-*).

### E. Operability and the self-hostable surface

#### REQ-PROD-030 — A health signal that reflects whether the product works for users
- **Statement:** Whoever operates chirp can tell whether the product — not merely the process — is healthy.
- **Acceptance criteria:** An operator can consult a health signal that reflects whether the product is actually able to serve users (including its dependence on durable data), and that signal reports unhealthy when the product cannot serve users even if the process is still running. (Today the health probe reports process liveness only.)
- **Release gating:** true
- **Traceability:** J8; P4. Health-check definition and probes → operational readiness (REQ-OPS-*).

#### REQ-PROD-031 — Normal operations do not destroy user data
- **Statement:** Restarting, redeploying, or scaling the service does not destroy users' accounts or content.
- **Acceptance criteria:** After an operator restarts, redeploys, or scales the service, user accounts, chirps, follows, and likes are still present and usable (this is the operator-side face of REQ-PROD-026).
- **Release gating:** true
- **Traceability:** J8; P4. Durable storage → domain & application design (REQ-DOM-*) and operational readiness (REQ-OPS-*).

#### REQ-PROD-032 — A defined recovery path for the service and its data
- **Statement:** When something fails, there is a defined path back to a working service with user data intact.
- **Acceptance criteria:** An operator can follow documented steps to recover the running service and restore user data after a failure, and verify afterward that accounts and content are present.
- **Release gating:** true
- **Traceability:** J8; P4. Backups/restore procedure → operational readiness (REQ-OPS-*).

#### REQ-PROD-033 — A minimal admin and support capability
- **Statement:** Someone running chirp can find an account, respond to a support request, and remove abusive content.
- **Acceptance criteria:** An authorized operator can look up a user account, see enough about it to answer a support request (for example "my account is gone" or "I can't sign in"), and remove abusive content — through a provided operator/admin surface rather than by directly manipulating storage. (Today there is no admin view, user lookup, or moderation surface.)
- **Release gating:** true — a releasable consumer product cannot be operated without this.
- **Traceability:** J8; P4 and the absent moderator persona. Admin authorization & surface → identity & trust (REQ-ID-*) and operational readiness (REQ-OPS-*).

#### REQ-PROD-034 — Reproducible build/run and documented operation (self-hostable surface)
- **Statement:** Whoever runs chirp can build, deploy, configure, and observe it from the repository's documentation, without hidden prerequisites.
- **Acceptance criteria:** Following the repository's documented steps, a person can build the product image reproducibly, run it, configure it for their environment through documented settings, and observe its health — using only what the repository provides.
- **Release gating:** true — chirp is distributed as source plus a self-contained image with a build-run-deploy path, so the releasable bar keeps this surface working.
- **Traceability:** Kind implication (self-hostable/deployable product); J8; P4. Build/deploy/config/observability detail → operational readiness (REQ-OPS-*).

### Journey → requirement coverage

Every end-to-end journey from §3 traces to at least one `REQ-PROD-*`. (Requirements whose detailed
mechanism sits in a sibling area are still minted here at product level so the journey is covered.)

| Journey | Covering REQ-PROD-* |
|---|---|
| J1 First visit | 013, 014, 015, 024 |
| J2 Sign-up | 001, 002, 003, 004, 022 |
| J3 Sign-in / sign-out | 004, 005, 006 |
| J4 Account recovery | 007 |
| J5 Core loops | 013, 016, 017, 018, 019, 020, 021, 023, 024, 027 |
| J6 Daily / returning use | 006, 008, 009, 025, 026 |
| J7 Leaving (deletion & export) | 009, 010, 011, 012, 029 |
| J8 Operator / administrator | 026, 028, 030, 031, 032, 033, 034 |

### Product-kind implication coverage

Each implication §1 named for an applicable kind is either reflected in a requirement or recorded
out-of-scope with a reason in §5.

| Kind | Implication | Where it lands |
|---|---|---|
| Consumer web app | I can sign up myself | REQ-PROD-001 |
| Consumer web app | This account is mine (credentialed) | REQ-PROD-004 |
| Consumer web app | What I create persists | REQ-PROD-026 |
| Consumer web app | I can get back in if I lose access | REQ-PROD-007 |
| Consumer web app | I can manage and close my account | REQ-PROD-008, 009, 010 |
| Consumer web app | I can take my data with me | REQ-PROD-011 |
| Consumer web app | Reasonably safe & private; told what is held | REQ-PROD-027, 029 |
| Self-hostable / deployable | Build reproducibly and run from documented steps | REQ-PROD-034 |
| Self-hostable / deployable | Configure and operate from what the repo provides | REQ-PROD-030, 032, 034 |
| Multi-tenant SaaS | — | Out of scope (see §5), with reason |
| Commercial COTS | — | Out of scope (see §5), with reason |
| Open-source community project | — | Out of scope (see §5), with reason |

---

## 5. In scope / out of scope

*(section: `in_scope_out_of_scope`)*

This section states what a **releasable** product of chirp's kind includes and what it reasonably leaves
out. It consolidates the product-kind scope view from §1 with what the requirement register in §4
settles. Everything judged out of scope carries a reason.

### In scope — what a releasable chirp includes

A releasable chirp is a **consumer web application** with a **self-hostable/deployable surface**. In scope:

- **Getting in — identity & access.** Self-service sign-up in the product UI, with handle validation and
  uniqueness feedback; credentialed sign-in that proves account ownership; deliberate sign-out;
  predictable session state with no silent sign-out; and an account-recovery path.
  *(REQ-PROD-001–007.)*
- **Living in and leaving the account.** Viewing and editing one's own profile; a reachable
  account-management home; self-service account deletion with a clear statement of consequences; export
  of one's own data in a portable format; and deletion that is actually honoured.
  *(REQ-PROD-008–012.)*
- **The core product loop.** Reading the public timeline and profiles without an account; a useful
  logged-out first screen (including its empty and error states) with an obvious route to join; composing
  and posting chirps within the limit without losing drafts; a home timeline with honest
  loading/content/empty/error/end-of-list states and paging; profile views that handle not-found and
  error; reliable follow/unfollow and like/unlike with truthful counts; and durability of accounts,
  chirps, follows, and likes across restarts and deploys.
  *(REQ-PROD-013–021, 026.)*
  - Tracked but **not release-gating** (real improvements a first version may ship without): new-user
    onboarding past an empty first session; a coherent replies experience (or the deliberate absence of a
    dead reply affordance); search/discovery beyond on-screen links; and "what's new" continuity on
    return. *(REQ-PROD-022–025.)*
- **Trust, safety, privacy and the legal surface.** Users can report and reduce exposure to unwanted
  content and accounts; an operator/moderator can act on abuse; and a privacy-and-terms surface states
  what data is held and the terms of use.
  *(REQ-PROD-027–029.)*
- **Operability and the self-hostable surface.** A health signal that reflects whether the product works
  for users; normal operations (restart/redeploy/scale) that do not destroy user data; a defined recovery
  path for the service and its data; a minimal admin/support capability; and a reproducible
  build/run with documented operation.
  *(REQ-PROD-030–034.)*

The detailed requirements and mechanisms for many of the above are owned by sibling design areas — domain
& application design (`REQ-DOM-*`), experience design (`REQ-UX-*`), identity & trust (`REQ-ID-*`), and
operational readiness (`REQ-OPS-*`). This document states the product-level requirement from the user's
or operator's side; each area carries its own `REQ-*` for the "how".

### Out of scope — with reasons

- **Implementation "how".** The persistence engine, authentication protocol internals, framework
  internals, and screen-level visual design are **not** settled in this product-requirements document.
  *Reason:* they are owned by the sibling design areas' `REQ-*` and by later implementation work; stating
  them here would confuse what the product must do with how it is built.
- **Multi-tenancy, organizations, team accounts.** *Reason:* chirp is a single shared consumer social
  product, not B2B SaaS — it has no notion of a tenant, team, or per-tenant isolation. *Watch-point:* if
  chirp ever grew organization or team accounts, this kind and its expectations (tenancy, roles, admin,
  audit) would return and would need to be scoped in.
- **Billing, payments, paid tiers.** *Reason:* chirp has no commercial model; there is nothing to charge
  for. Out of scope unless the engagement later introduces monetization.
- **Richer social features beyond the core loop** — direct messages, media/image upload,
  hashtags/trending, and notifications of replies and new followers. *Reason:* these are natural
  Twitter-style extensions but are not required for a first releasable version of the post/read/follow/
  like loop; they belong in later work. (Of these, lightweight notifications and search/discovery are the
  most likely near-term additions; note that a basic replies experience and search/discovery are tracked
  as non-gating requirements REQ-PROD-023 and REQ-PROD-024 rather than being excluded outright.)
- **Federation / interoperability with other microblogging networks.** *Reason:* not part of chirp's
  product aim.
- **Open-source community governance** — contributor guide, external issue triage, a published license
  for reuse. *Reason:* chirp is a closed demonstration that explicitly does not accept contributions and
  carries no open-source license; there is no contributor workflow or community-support expectation to
  design for. (The *outward shape* of public source and a reproducible build is retained via the
  self-hostable surface, REQ-PROD-034 — but it is not run as an OSS project.)
- **Commercial off-the-shelf (COTS) expectations** — licensing, entitlement, versioned upgrades for
  paying customers, and a commercial support channel. *Reason:* chirp is not sold, licensed, or supported
  as a purchasable product. The *self-hostable* build-and-operate expectations (REQ-PROD-034) still apply
  because of how chirp is distributed, but the commercial COTS expectations do not.
