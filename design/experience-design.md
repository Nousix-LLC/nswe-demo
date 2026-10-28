# chirp — Experience Design

How chirp looks and behaves to the people who use it: the information architecture and interaction model, a designed screen for every journey a releasable chirp needs, the product-wide empty, loading, and error behaviour, the visual language and component library, responsive behaviour from a phone upward, the accessibility standard, and the full `REQ-UX-*` requirements register that makes each of these testable on the live product.

This document takes chirp from a three-screen skeleton — timeline, compose, and profile — to a designed, releasable consumer web product: the full account lifecycle, onboarding, settings, safety, notifications, data export and deletion, and considered empty, error, and loading states throughout. The requirements register at the end is the single source of testable acceptance criteria; each requirement carries a release-gating flag and traces to the product requirements it serves. The document is design — what the product must look like and do — and leaves the build to the implementation that follows.

## Information architecture and interaction model

This section defines the structure of chirp — the set of screens it is made of, how they are grouped,
and how a person moves between them — and the product-wide interaction conventions every screen obeys.
It is the foundation the visual system, the per-screen flows, the responsive layout, and the
accessibility work build on. Where it names a screen, it does so by the name in the **Screen inventory**
below; the rest of the experience design refers to screens by those names.

chirp today presents three screens above a persistent navigation bar: the home timeline at `/`, a
combined compose-and-sign-in view at `/compose`, and a public profile at `/user/:id`. That is a working
skeleton of the core loop, not the structure of a product a stranger can sign up for, trust, and leave.
The architecture here keeps the core loop at the centre and adds the account lifecycle, the settings and
safety surfaces, onboarding, notifications, and the signed-out entry a first-time visitor meets — each as
a named screen with a route and an archetype, so no required surface is missing and every surface has one
clear place to live.

The routes below are the user-facing address scheme the design calls for, stated from the user's side;
how they are served is a build decision for later code work. Handles are how people are known on chirp,
so a profile is addressed by its handle (`/@:handle`) rather than an internal id.

### Screen archetypes

Every screen is classified as one of seven archetypes. The archetype fixes what a person can expect of
the screen before they read a word of it, and lets the visual system and the responsive and accessibility
work design once per archetype instead of once per screen.

- **feed/list** — a paginated, newest-first (or ranked) list of items a person scans and acts on in place
  (timeline, notifications, a conversation, a followers list, search results).
- **form** — a screen whose job is to collect and submit input (sign-up, sign-in, recovery, edit profile).
- **profile** — a person's public identity surface (handle, display name, bio, counts, their chirps).
- **settings** — a grouped surface of account-level choices, each independently addressable.
- **modal/dialog** — a focused task layered over the current screen without leaving it (compose, report).
- **confirmation** — a dialog whose only job is to confirm a consequential or destructive action.
- **empty/landing** — a screen that is mostly explanation and a next step: the signed-out landing, the
  first-run onboarding, the rules and help pages, and the not-found screen.

### Screen inventory

The canonical set of screens a releasable chirp needs. Every screen names its user-facing route, its
archetype, the auth state(s) in which it is reachable (**signed-out**, **signed-in**, or **both**), and
its purpose. Sibling design areas refer to these screens by the names in the first column.

**Signed-out and public surfaces**

| Screen | Route | Archetype | Auth | Purpose |
|---|---|---|---|---|
| Landing | `/` (no session) | empty/landing | signed-out | Explain what chirp is to a first-time visitor and lead to sign-up or sign-in without requiring an account; show public content where a sample is available. |
| Sign up | `/signup` | form | signed-out | Create an account: handle, display name, optional bio, the credential, and the verified contact the account requires. |
| Sign in | `/signin` | form | signed-out | Return to an existing account by proving ownership. |
| Recover access | `/recover` | form | signed-out | Start self-service recovery for a person who has lost their credential, via the verified contact. |
| Set new credential | `/recover/confirm` | form | signed-out | Complete recovery: prove control of the contact and set a new credential. |
| Public profile | `/@:handle` | profile | both | A user's public profile — handle, display name, bio, follower/following counts, their chirps. |
| Chirp / conversation | `/@:handle/chirp/:id` | feed/list | both | A single chirp shown with the chirp it replies to and its replies — the conversation in context. |
| Followers | `/@:handle/followers` | feed/list | both | The accounts that follow this user. |
| Following | `/@:handle/following` | feed/list | both | The accounts this user follows. |
| Rules & terms | `/rules` | empty/landing | both | The stated rules and terms a report references. |
| Help & support | `/help` | empty/landing | both | How to get help; the user-facing entry to recovery, reporting, and contacting the operator. |
| Not found | any unresolved path | empty/landing | both | Explain that a resource is missing or removed and route the person back to a known place. |

**Signed-in surfaces**

| Screen | Route | Archetype | Auth | Purpose |
|---|---|---|---|---|
| Home timeline | `/` (session) | feed/list | signed-in | The home: the person's own chirps and those of the people they follow, newest-first, in bounded pages. |
| Compose | `/compose` and an overlay dialog | modal/dialog | signed-in | Write and publish a chirp, or a reply to a named chirp. |
| Explore / find people | `/explore` | feed/list | signed-in | Search for and discover people and content to follow, beyond already holding a profile link. |
| Notifications | `/notifications` | feed/list | signed-in | Activity directed at the person — replies, follows, and likes. |
| Own profile | `/@:handle` (own) | profile | signed-in | The person's own public face, with the entry to edit it. |
| Onboarding | `/welcome` | empty/landing | signed-in | First run: explain the empty timeline and offer at least one way to a non-empty one (find people, first post). |
| Settings | `/settings` | settings | signed-in | The hub for account-level choices, linking to each sub-section below. |
| Edit profile | `/settings/profile` | form | signed-in | Change display name and bio. |
| Account & contact | `/settings/account` | settings | signed-in | View and update the verified contact; the stated handle-change policy. |
| Notification preferences | `/settings/notifications` | settings | signed-in | Choose which activity categories send a notification. |
| Sessions & devices | `/settings/sessions` | settings | signed-in | View active sessions/devices, end a session, and sign out. |
| Blocked & muted | `/settings/blocks` | settings | signed-in | Manage the accounts the person has blocked or muted. |
| Export your data | `/settings/export` | settings | signed-in | Request and obtain the person's own data — chirps, profile, follows — in a portable form. |
| Delete account | `/settings/delete` | confirmation | signed-in | Delete the account with a confirmed, explained outcome. |

**Dialogs (layered over the current screen; no address of their own)**

| Screen | Trigger | Archetype | Auth | Purpose |
|---|---|---|---|---|
| Report | report action on a chirp or profile | modal/dialog | signed-in | Report a chirp or account that breaks the rules; acknowledge receipt. |
| Block / mute | block-or-mute action on a profile | confirmation | signed-in | Confirm blocking or muting an account and state the effect. |
| Delete chirp | delete action on the person's own chirp | confirmation | signed-in | Confirm removing a chirp the person posted. |
| Sign out | sign-out action in the account menu | confirmation | signed-in | End the current session; confirm when ending sessions on other devices too. |

Two surfaces in this inventory serve product requirements the release bar does not gate, and they are
marked so the staging is explicit: **Notifications** (and the notification-related settings) ride
REQ-PROD-029, and **Explore / find people** rides REQ-PROD-042 — both `release_gating: false` in the
product requirements. They are in scope and have a designed place in the architecture; the release bar
decides whether a first release includes them. Every other screen above serves a release-gating product
requirement.

### Navigation and wayfinding

chirp has two navigation shells — one a person meets before they have an account, one they live in after
signing in. The transition between them is the sign-up / sign-in / sign-out boundary.

#### The signed-out shell

A person with no session sees a light shell built to answer one question — "what is this, and do I want
an account?" — and to never dead-end at an authentication wall. It carries:

- the chirp wordmark, linking to the Landing screen;
- a clear **Sign in** and a prominent **Sign up** entry, present on every signed-out screen;
- a footer reaching **Rules & terms** and **Help & support**.

Public surfaces — a profile, a chirp's conversation, the rules, and help — render inside this shell when
reached without a session, so a visitor who arrives on a shared link lands on real content with a visible
path to joining rather than a blank gate.

#### The signed-in shell

After signing in, a person lives in a persistent shell that exposes every primary destination from every
screen, so they always know where they are and can reach anything in one step:

- **Home** — the home timeline;
- **Explore** — find people and content to follow;
- **Notifications** — activity directed at them, with an unseen-activity indicator;
- **Profile** — their own profile;
- an always-available **Compose** affordance;
- an **account menu** reaching **Settings**, **Help & support**, **Rules & terms**, and **Sign out**.

The shell is the same set of destinations on every signed-in screen; the responsive design decides how it
presents on a phone versus a wider viewport, and the accessibility work decides how it is operated and
announced. Settings is itself a small secondary navigation: a hub listing its sub-sections (edit profile,
account & contact, notification preferences, sessions & devices, blocked & muted, export, delete), each a
screen a person can open directly and return from to the hub.

#### Wayfinding

A person can always tell where they are and how they got there:

- the active primary-nav destination is visibly marked in the shell;
- the browser document title and the main heading name the current screen;
- a screen reached by drilling in (a conversation from the timeline, a sub-section from Settings, a
  followers list from a profile) offers a labelled way back to where the person came from, and the browser
  back control does the same thing.

#### Paths between account states

The architecture has three account states — **no session** (signed-out shell), **a brand-new session**
(onboarding), and **an established session** (signed-in shell) — and the moves between them are designed,
not incidental:

- signed-out → **Sign up** → a created session that lands on **Onboarding**, then into the signed-in shell;
- signed-out → **Sign in** → the signed-in shell (or the screen the person originally asked for);
- signed-out → **Recover access** → **Set new credential** → **Sign in** → the signed-in shell;
- signed-in → **Sign out** → the signed-out Landing, with protected screens no longer reachable.

### Interaction conventions

These are the product-wide patterns every screen obeys, so a person learns them once. The per-screen
flows apply them; they are not redesigned per screen.

#### Addressable screens and deep-linking

Public profiles and individual chirps have stable, shareable URLs that open the right screen directly: a
link to `/@:handle` opens that profile, and a link to `/@:handle/chirp/:id` opens that chirp in its
conversation. A settings sub-section and a followers/following list are addressable too, so a person can
bookmark or link to the exact place. Opening an addressable screen directly, with or without a session,
lands the person on that screen (subject to auth-gating below), not on a generic home.

#### Auth-gated routing and return-to

Signed-in-only screens are gated. A person who opens a signed-in-only URL without a session is routed to
**Sign in**, and on success is returned to the screen they originally asked for rather than dropped on the
home timeline. A public screen opened without a session renders in the signed-out shell.

#### Session expiry mid-use

When an action is attempted with an expired or invalid session, the product routes the person to
**Sign in** with the reason stated, preserves any in-progress input (such as a drafted chirp), and returns
them to where they were after they sign back in. Expiry is never a silent failure or a lost draft.

#### Browser back and forward

Back and forward behave the way a person expects in a web product: back returns to the previous screen
without re-submitting the previous action and without losing the person's place in a timeline they had
scrolled; opening a dialog (compose, report, a confirmation) is a step the back control dismisses, closing
the dialog rather than leaving the underlying screen.

#### Pagination and catch-up

Every list — the timeline, a conversation's replies, notifications, search results, followers and
following — loads in bounded pages, never as a single unbounded load. A list offers an explicit way to
load older items, and the timeline offers a way to pull in chirps posted since the person last looked, so
a long absence is a bounded catch-up rather than an endless scroll.

#### Confirming destructive actions

Every irreversible or destructive action is confirmed before it proceeds, through a **confirmation**
dialog that names what will happen and offers a plain way to cancel. This convention governs deleting an
account, deleting a chirp, blocking or muting an account, ending a session on another device, and signing
out of all sessions. The confirmation for deleting an account additionally states what is and is not
retained and points to data export first.

#### Transient feedback

The result of an action is always shown. A completed action (a chirp published, a follow, a like, a saved
setting, a received report) is confirmed with brief, non-blocking feedback that does not interrupt what
the person is doing. A failure is shown where the action was taken — against the field for a form, against
the control for an in-place action — with enough to understand and retry, never a silent no-op.

#### The sign-up, sign-in, and sign-out transitions

The boundary between the two shells is designed so no one is stranded:

- **Sign up → signed in.** A completed sign-up establishes the session and lands the person on
  **Onboarding** — they are not asked to sign in again immediately after creating the account, and they do
  not land on a blank timeline.
- **Sign in / sign up / recovery are interlinked.** Each of these screens links to the others, so a person
  who arrived at the wrong one (a returning user on Sign up, a new user on Sign in, someone who cannot get
  in) can correct course without leaving the flow.
- **Sign out → signed out.** Signing out ends the session and returns to the signed-out Landing; after
  sign-out the back control does not re-expose a protected screen's content.

## Screens and flows

A designed screen for each journey — layout, content, a single primary action, and the empty, error, and loading states it meets in ordinary use — grouped into the four surface clusters a chirper moves through: getting in, the core loop, the account lifecycle, and the safety and notifications surfaces.

### Getting in — entry and authentication

This is the `get_in` part of the screens-and-flows section: the screens a stranger meets before
they have a session, and the moment they cross from signed-out to signed-in and back. It covers the
first-visit **Landing**, **Sign up**, **Sign in**, the two-screen recovery (**Recover access** and
**Set new credential**), and the **Sign out** confirmation. Each is designed as an arrangement of
named components realising a named screen-inventory screen, with its layout, its content, its one
primary action, and its empty, error, and loading states — not a bare route.

These screens carry more weight than their size suggests. They are where a person decides whether
chirp is a product they will trust with an account, and they are the signed-out↔signed-in boundary,
so they are designed to the identity requirements the product sets: an account only its owner can
enter, a recovery path that runs through a verified contact and no one else, and a sign-out that ends
the session on the server rather than only forgetting it in the browser. Where a transition or a
convention is already fixed by the information architecture (auth-gated return-to, session-expiry
interception, the sign-up/sign-in/sign-out boundary), these screens compose against it and refer to
it; they do not restate it.

All six render inside the **NavShell** signed-out shell (the Sign out confirmation is the one
exception — it is a **Dialog** layered over the signed-in shell). The shell carries the chirp
wordmark to the Landing, a Sign in and a prominent Sign up entry, and footer access to Rules & terms
and Help & support on every signed-out screen, so no one ever dead-ends at an authentication wall.

---

#### Landing — `/` (no session), empty/landing archetype

**Layout.** The signed-out **NavShell** frames a single centred reading column at `space.measure`
(600px). Inside it, an **EmptyState** hero: a calm flat illustration in one or two brand hues, a
`type.display` headline, and a `type.body` `color.text.secondary` paragraph that says what chirp is
in plain words. Directly below the hero, the action pair — a prominent **PrimaryButton** "Sign up"
and a quieter **SecondaryButton** "Sign in". Below that, where a public sample is available, a bounded
**ListView** of public **ChirpCard**s (read-only, each still linking to its public profile and
conversation) so a first-time visitor sees real content, not just a pitch. Generous vertical rhythm
(`space.10`/`space.12`) keeps the screen unhurried. The footer (Rules & terms, Help & support) comes
from the shell.

**Content.** A one-line explanation of chirp ("Short public posts from the people you follow"), the
two entry actions, and — when a sample is served — a handful of public chirps. Nothing on this screen
requires an account to read.

**Primary action.** **Sign up** → opens **Sign up** (`/signup`). The secondary action, **Sign in**,
opens **Sign in** (`/signin`). Both are always present; the Landing never requires signing in to be
understood.

**States.**
- **Empty** — no public sample is available (none is served, or it is withheld). The hero explanation
  and both entry actions stand on their own; the sample region carries a short **EmptyState**
  explanatory line ("Sign up to see what people are posting") rather than a blank gap. The Landing
  still explains chirp and leads to sign-up.
- **Error** — the public-sample fetch fails. The sample region shows the **EmptyState** error variant
  ("We couldn't load a sample right now — you can still join") with a retry. The explanation and the
  Sign up / Sign in actions stay fully usable; a sample failure never blocks joining.
- **Loading** — on first paint the shell and the explanatory hero render immediately (the native
  `type.family.sans` stack means no blank flash while a font loads). The sample region shows
  **LoadingState** skeleton **ChirpCard**s at their real dimensions while it resolves, so the column
  does not jump when content arrives.

---

#### Sign up — `/signup`, form archetype

**Layout.** The signed-out shell around a single 600px form column, `type.title` heading "Create your
account". A stack of **TextField**s with `space.4` between them, then a **PrimaryButton** "Create
account" at the foot. Below the button, interlinks: "Already have an account? Sign in" and "Can't get
in? Recover access". The form asks for no more than the account needs.

**Content.** Five fields, each a labelled **TextField** with a plain hint:
- **Handle** — "@handle", hint "Up to 15 letters, numbers, or underscores". Checked for format and
  uniqueness as the person types (see loading / error states).
- **Display name** — the name shown on your chirps.
- **Bio** — optional, multi-line, marked optional so it reads as skippable.
- **Credential** — the secret you will use to prove the account is yours; obscured input. (What the
  credential is and how it is stored is the identity design's; this screen states only that one is set
  here, so a handle alone can never sign in — J2/REQ-PROD-008.)
- **Contact** — a point of contact chirp can verify, used for account recovery and important
  notifications; a one-line note says why it is asked for and that it is kept minimal. The form
  triggers a verification step and shows a pending-verification affordance; whether verification
  completes inline or just after the account is created is the identity design's call.

**Primary action.** **Create account** — disabled until the required fields are valid. On success it
establishes the session and lands the person on **Onboarding** (`/welcome`); they are not asked to
sign in again, and they do not land on a blank timeline. This routing composes against REQ-UX-014 and
is not redefined here.

**States.**
- **Empty** — a fresh form: labels and placeholders, no values, **Create account** disabled. A
  required-but-empty field is not shown as an error until the person has had a chance to fill it
  (**TextField** empty state).
- **Error** — field-level and plain: a taken or ill-formed handle shows an **InlineError** against the
  handle field ("That handle's already taken — try another" / "Handles can use letters, numbers, and
  underscores, up to 15"). A failed submit shows an **InlineError** at the point of the problem with
  every entered value preserved, so the person fixes one thing and retries. No message carries a system
  or error code.
- **Loading** — the handle uniqueness/format check runs as a quiet **TextField** pending affordance
  that resolves to accepted or a field error. On submit, **Create account** shows its busy state and
  the form locks briefly so the account is not created twice.

---

#### Sign in — `/signin`, form archetype

**Layout.** The signed-out shell around a 600px form column, `type.title` heading "Sign in". Two
**TextField**s — handle and credential — then a **PrimaryButton** "Sign in". Below it, interlinks:
"New to chirp? Sign up" and a "Can't get in?" link to **Recover access**.

**Content.** The handle you signed up with and your credential. The "Can't get in?" link is the
in-flow route to recovery for someone who has lost their credential.

**Primary action.** **Sign in** — disabled until both fields hold input. On success the person lands
in the signed-in shell, or on the screen they originally asked for when they were routed here by
auth-gating (the return-to behaviour is fixed by REQ-UX-007 and composed against, not redefined). When
this screen is the destination of a mid-use session expiry, it states the reason and preserves the
person's in-progress work, composing against REQ-UX-008 and the expired-session pattern owned by the
`empty_error_loading_states` section — it does not restate that pattern.

**States.**
- **Empty** — labelled fields, no values, **Sign in** disabled.
- **Error** — two distinct signals, so a person can tell a typo from a wrong secret (REQ-PROD-011). An
  unknown handle resolves to "We couldn't find an account with that handle" with a path to **Sign up**.
  A known handle with a wrong credential resolves to "That didn't match — try again or recover access"
  with a path to **Recover access**. Both are **InlineError** at the point of action, worded so they do
  not disclose which accounts exist beyond what the product already makes public, and never a system
  code.
- **Loading** — on submit, **Sign in** shows its busy state and the fields lock briefly so a double
  submit cannot open two sessions.

---

#### Recover access — `/recover`, form archetype

**Layout.** The signed-out shell around a 600px column, `type.title` heading "Can't get in?". A
`type.body` paragraph explains what will happen — chirp sends a recovery step to the contact verified
on the account, and no one else can start this for you. One **TextField** (the verified contact, or
the handle, depending on what the identity design keys recovery to), then a **PrimaryButton** "Send
recovery link". A link back to **Sign in** closes the interlink loop.

**Content.** The contact field and the plain explanation. The screen sets the expectation that
recovery runs end to end through the verified contact (J3a / REQ-PROD-012, 013).

**Primary action.** **Send recovery link** — on success the screen confirms, in wording that does not
reveal whether a given contact has an account, that if the contact matches an account a recovery step
has been sent ("If that contact is on an account, we've sent a recovery link to it"). Following that
step opens **Set new credential**.

**States.**
- **Empty** — labelled field, no value, the action disabled.
- **Error** — a malformed contact shows an **InlineError** against the field ("That doesn't look like
  a contact we can reach"). A send failure shows an **InlineError** with a retry. A valid but unknown
  contact is answered with the same neutral, non-disclosing acknowledgement as a match — not an error —
  so the screen never reveals who has an account.
- **Loading** — on send, **Send recovery link** shows its busy state until the request resolves.

---

#### Set new credential — `/recover/confirm`, form archetype

**Layout.** The signed-out shell around a 600px column, `type.title` heading "Set a new way to get
in". A `type.body` line explains this finishes recovery. The recovery token is carried by the link the
person followed and is not asked of them; where a recovery code must be read or typed exactly it is
shown in `type.family.mono`. A **TextField** for the new credential and a second to confirm it, then a
**PrimaryButton** "Set new credential".

**Content.** The new-credential field and its confirm, with the same plain validation voice as the
rest of the forms. On success the person is sent to **Sign in** to enter with the new credential,
completing the IA path signed-out → Recover access → Set new credential → Sign in.

**Primary action.** **Set new credential** — disabled until the new credential is valid and confirmed.
On success it routes to **Sign in** (composing against REQ-UX-015).

**States.**
- **Empty** — labelled field(s), no value, the action disabled.
- **Error** — an expired or already-used recovery link shows an **EmptyState** error variant that
  explains the link is no longer valid and offers a path to start again at **Recover access**, rather
  than a dead end. A new credential that does not meet the stated rule, or a mismatch between the field
  and its confirm, shows an **InlineError** against the field. A submit failure shows an **InlineError**
  with a retry.
- **Loading** — validating the recovery link when the screen opens shows a **LoadingState** that
  resolves either to the form or to the expired-link error. On submit, **Set new credential** shows its
  busy state.

---

#### Sign out — the account-menu confirmation, confirmation archetype (**Dialog**)

**Layout.** A **Dialog** (confirmation variant) rises over the current signed-in screen on the
`color.bg.scrim`, at `elevation.3`. A `type.heading` title "Sign out?", a `type.body` line, a
**Toggle** labelled "Also sign out on all other devices", and the action pair — a **PrimaryButton**
"Sign out" with a **SecondaryButton** "Cancel". The Dialog traps focus while open and returns focus to
the account-menu control that opened it on close.

**Content.** A plain confirmation that this ends the current session, with the option to end sessions
on every other device too. This is the only entry/auth surface reached from the signed-in shell (the
account menu), because signing out is the move back across the boundary.

**Primary action.** **Sign out** — ends the session on the server (and, when the toggle is on, the
other sessions too), then lands the person on the signed-out **Landing**. After sign-out the browser
back control does not re-expose a protected screen's content; that no-leak behaviour is fixed by
REQ-UX-016 and composed against here, not redefined.

**States.**
- **Empty** — does not apply: a **Dialog** always opens with its content. Recorded, not omitted.
- **Error** — the sign-out request fails. An **InlineError** appears inside the panel ("We couldn't
  sign you out — try again") and the Dialog stays open so the person can retry. The browser session is
  not treated as ended while the server still holds it, so the control never claims a sign-out that did
  not happen.
- **Loading** — on confirm, the **Sign out** button shows its busy state and the Dialog does not close
  until the server confirms the session has ended.

---

#### The transitions between these screens

These screens are stitched together so no one is stranded at the boundary, composing against the IA
interaction model (REQ-UX-007, 008, 014, 015, 016) rather than restating it:

- **Sign up → Onboarding.** A completed sign-up establishes the session and lands on **Onboarding**,
  not on a sign-in prompt and not on a blank timeline.
- **Interlinks.** Sign up, Sign in, and Recover access each link to the others, so a returning person
  who opened Sign up, a new person who opened Sign in, or someone who cannot get in, can correct course
  without leaving the flow. Recover access also stays reachable from Help & support.
- **Auth-gated return-to.** When a person is routed to **Sign in** from a signed-in-only screen they
  asked for, a successful sign-in returns them there rather than to a default screen.
- **Session expiry mid-use.** An action on an expired session routes to **Sign in** with the reason
  stated and the person's in-progress work preserved; the keep-the-draft pattern is owned by the
  `empty_error_loading_states` section and composed against here.
- **Sign out → signed-out Landing.** Signing out ends the session and returns to the Landing, with no
  back-leak into protected content.

---

### Core loop — reading, posting, and the social actions

The core loop is the handful of things a chirper does every day: read the home timeline, write and
post a chirp, read and reply to a conversation, like what they like, follow the people they want to
hear from, find new people to follow, and look at a profile. This section designs each of those
surfaces as a real screen — its layout, its content, its one clear primary action, and the empty,
error, and loading states it meets in ordinary use — not a route with a name under it.

Every screen here is a screen named in the Screen inventory, referred to by its name and route, and is
assembled from the named components of the Component library (`NavShell`, `ListView`, `ChirpCard`,
`ComposeBox`, `ProfileHeader`, `Avatar`, `IconButton`, `PrimaryButton`, `SecondaryButton`, `TextField`,
`Dialog`, `Toast`, `InlineFeedback`, `InlineError`, `EmptyState`, `LoadingState`). A screen's
empty/error/loading states are those components' own named states together with the product-wide
empty/loading/error patterns and the expired-session / keep-the-draft recovery owned by the
considered-states surface. Nothing in this section redefines the inventory, the component library, or
those global patterns; where a behaviour is owned by another part (follow/unfollow, edit profile,
onboarding, the global state patterns) it is composed and referred to by name, not redrawn.

The surfaces are presented in the order a person meets them in the loop — reading the timeline, writing
and reading in context, reacting in place, then identity and finding people — followed by the
consolidated `REQ-UX-051..064` requirements register. Each requirement is testable by someone who never
met its author, exercisable on the live product, carries a `release_gating` flag, and references at
least one product requirement (`REQ-PROD-*`) it serves.

---

#### Home timeline — the core reading surface

The **Home timeline** (`/` with a session, the **feed/list** archetype) is the screen chirp opens
to once a person is signed in, and it is the default thing a chirper does: read what is new from
themselves and the people they follow, newest first, and catch up on what arrived while they were
away. It is the most-visited screen in the product, so it has to paint fast, behave sensibly when
there is nothing yet to show, and recover in place when a page fails rather than dropping the reader
on a blank screen.

This design composes the frozen screen inventory and component library by name; it introduces no new
screen or component. The timeline is the signed-in **NavShell** wrapped around a **ListView** of
**ChirpCard**s held to the reading column, with the inventory's pagination and catch-up conventions
and the product-wide empty / loading / error patterns. The per-card actions (open conversation, like,
reply) are the **ChirpCard**'s own; they are designed by the posting-conversation and social-action
surfaces and only composed here.

##### Layout

The screen is the signed-in **NavShell** with the timeline content in the main region:

- **The shell** (`NavShell`, signed-in variant) is the persistent frame around the content: Home (the
  active destination, marked), Explore, Notifications, Profile, the always-available Compose
  affordance, and the account menu. The shell is persistent — moving to a conversation and back never
  blanks it. Marking Home as the current destination is the information architecture's wayfinding
  convention, composed here, not redefined.
- **The content region** is a single reading column held to the `space.measure` (600px) reading
  width, centred on wider viewports with the shell and any secondary rail outside it, and filling the
  viewport with a `space.gutter.mobile` (16px) gutter on a phone. How the shell and the column reflow
  across viewports is the responsive area's decision; this screen fixes the column, not the breakpoints.
- **The catch-up control** sits at the top of the content region, above the first card: a `ListView`
  catch-up affordance in `color.primary` reading "N new chirps", shown only when chirps have arrived
  since the person last looked. Selecting it brings those chirps in at the top and returns the reader
  to the newest.
- **The list** (`ListView`) fills the column below: a bounded page of `ChirpCard`s separated by the
  `color.border.default` hairline, newest first, with a **load-older** control (a `SecondaryButton` or
  text control) at the bottom boundary of the loaded page.

##### Content

- **Order and membership.** The list holds the person's own chirps and those of the accounts they
  follow, newest first. It is not a ranked or algorithmic feed; recency is the order, which is what the
  product requirement fixes and what a reader catching up expects.
- **Each row** is a `ChirpCard` showing, from the component library: the author's `Avatar`
  (`avatar.md`), display name (`type.heading`), handle and a short relative timestamp (`2m`, `4h`,
  then an absolute date) in `type.meta` / `color.text.secondary`, the chirp text in `type.body`, and
  the card's action row (reply, like, more-actions). The whole card is the target that opens the chirp
  in its conversation. Those per-card actions are the `ChirpCard`'s own behaviour; this screen places
  the card and does not redesign them.
- **Bounded pages, not an unbounded load.** The list loads one bounded page and offers an explicit way
  to load older chirps at the boundary; it never attempts to load every chirp at once. The pagination
  convention (bounded pages, an opaque cursor, an explicit load-older) is set by the information
  architecture (REQ-UX-010) and composed here.
- **Catch-up after an absence.** When chirps have been posted since the reader last opened the
  timeline, the top-of-list catch-up control offers to pull them in as a bounded step, so a long
  absence is a bounded catch-up rather than an endless scroll. This, too, is the IA's catch-up
  convention, composed, not redefined.
- **Scroll place is kept.** Returning from a chirp's conversation lands the reader back at their place
  in the list rather than at the top — the `ListView`'s scroll-restoration behaviour, which the IA's
  back-and-forward convention (REQ-UX-009) also requires.

##### Primary action

The screen's one prominent, screen-level action is **bring the timeline current and read the newest
chirps**: when there is new activity, the catch-up control ("N new chirps") is the primary action a
reader reaches for; when there is none, the primary act is to read and scan the page. Loading older
chirps is the secondary action at the list boundary. Composing is always available from the shell's
Compose affordance (the IA's persistent compose convention) rather than being a control this screen
owns, so the reading surface stays a reading surface.

##### States

The timeline's empty, error, and loading states are the relevant components' own named states
(`ListView`, `ChirpCard`, `EmptyState`, `LoadingState`, `InlineError`) composed with the product-wide
empty / loading / error patterns. They are not new treatments invented for this screen.

**Empty — a brand-new timeline that explains and points onward.** A reader who follows no one and has
posted nothing has an empty timeline. The `ListView` *empty* state shows the `EmptyState` block — a
calm illustration, one line explaining that the timeline fills as they follow people and post, and a
next step: a link to **Explore** to find people and a way to compose a first chirp. It is never a
blank region and never a spinner that has finished on nothing. First-run **Onboarding** (`/welcome`)
is the account-lifecycle surface that greets a newly-created account and moves it off the empty state;
this screen designs only the timeline's own empty treatment and leaves onboarding to that surface, so
a reader who lands on an empty timeline later (for example after unfollowing everyone) meets the same
explained empty state rather than nothing.

**Error — a failed page or catch-up fetch, recovered in place.** A failure is shown where it happened,
in plain words, with a retry, and already-shown chirps are never discarded:

- A **first page** that fails to load shows the `ListView` *error*: an `InlineError` with a retry in
  place of the missing list, distinct from the empty treatment so "we couldn't load this" and "there
  is nothing here" are never confused.
- A failure while **loading older** chirps shows the `InlineError` at the load-more boundary and keeps
  the chirps already on screen; the reader retries from the boundary without losing their place.
- A failed **catch-up** fetch shows the failure on the catch-up control with a retry and leaves the
  current list intact, so a broken catch-up never clears what the reader was already reading.
- An **expired session** met on the timeline is the product-wide expiry pattern, not a timeline-local
  error: the reader is routed to **Sign in** with the reason stated and returned to the timeline at
  their place afterwards (the IA's session-expiry and auth-gated-return conventions, REQ-UX-008 /
  REQ-UX-007). The timeline does not dress an expiry up as a retryable load error.

**Loading — the shape of the page, not a spinner on a blank screen.** A **first load** shows
`LoadingState` skeleton rows at the real dimensions of a `ChirpCard` — a skeleton avatar circle, two
text lines, and a short meta line — so the column does not jump when real chirps arrive and the reader
sees the page forming. Loading **older** chirps shows a bounded spinner at the list's load-more
boundary with the already-loaded chirps left in place. The persistent `NavShell` never participates in
loading — only the content region shows a loading state; the frame around it stays put. Nothing spins
forever: a load that does not resolve hands off to the list's *error* state with a retry. Under a
reduced-motion preference the skeletons are static and the spinner stays bounded, so the loading state
never relies on movement to be understood.

*Composition note: this surface composes the frozen screen inventory (Home timeline, `/`, feed/list)
and component library (`NavShell`, `ListView`, `ChirpCard`, `EmptyState`, `LoadingState`,
`InlineError`, `Avatar`) by name, the information architecture's pagination / catch-up / back and
session-expiry conventions (REQ-UX-010, REQ-UX-009, REQ-UX-008, REQ-UX-007), and the product-wide
empty / loading / error patterns (REQ-UX-086..089) — none redefined.*

---

#### Posting and conversation

This is the part of the core loop where a chirper writes and reads what was written in context: composing
and publishing a chirp inside the 280-character limit, and the Chirp / conversation screen where a chirp is
read alongside the chirp it answers and its replies, and where a reply is written. Two surfaces are designed
here — a writing surface and a reading-in-context screen — each with its layout, content, primary action,
and its empty, error, and loading states. Both are built from screens named in the screen inventory and
components named in the component library; nothing here redefines either. The single writing control,
**ComposeBox**, is designed once in the compose surface and composed by name in the conversation screen's
reply slot.

##### Compose and post a chirp

Writing a chirp is the first thing chirp exists for, and it is the surface a new person uses before
they have followed anyone or read a timeline worth reading. This section designs that surface: where a
person writes a chirp, how the product keeps them inside the published 280-character limit while they
type, how the "Post" action behaves, and what the surface looks like when it is empty, when something
is wrong, and while a chirp is being published.

The surface is one component — **ComposeBox** — shown in three places. It is designed once here, with
its counter, its limit behaviour, its primary action, and its empty, error, and loading states, and
the same component is placed inline at the top of the **Home timeline**, as the body of the **Compose**
dialog (`/compose` and the overlay dialog), and in the reply position on the **Chirp / conversation**
screen. The conversation screen composes the reply-position variant designed here rather than defining
its own writing field. The screens named here are the **Compose** and **Home timeline** and **Chirp /
conversation** entries of the screen inventory; every piece named in bold is a component-library
component used by its name, not a new invention.

This surface serves journey J4 step 2 (compose and post) and the product requirement REQ-PROD-001 (post
a chirp up to 280 characters; an empty or over-limit message is rejected and nothing is posted). The
reply-position variant also carries the writing half of replying (REQ-PROD-003); the conversation
screen owns the rest of that flow.

###### Layout and content

The **ComposeBox** reads top to bottom as one block, the same in every placement:

- the composing person's **Avatar** (`avatar.md`) at the leading edge, so a person sees whose voice the
  chirp will carry — this matters most in the reply position, where the draft sits among other people's
  chirps;
- the **draft field** — a multi-line text area on the `color.bg.subtle` draft surface, holding the
  chirp text in `type.body`, growing with the content rather than scrolling a fixed box, with a
  placeholder prompt when empty;
- the **remaining-character affordance** — a live count of characters still available, in
  `type.caption` with `type.numeral.tabular` figures so the number does not shift the layout as it
  changes;
- the **primary action** — a **PrimaryButton** labelled for the placement ("Post" when writing a new
  chirp, "Reply" in the conversation's reply position).

The draft field is the focus of the surface and takes the most width; the avatar, counter, and action
sit around it without competing for attention. In the dialog placement the box is the dialog's content
and the action pair sits at the dialog's foot; inline and in the reply position the action sits with
the box. Nothing else lives in this surface — no formatting toolbar, no media controls, no audience
picker — because a chirp in this product is plain text up to 280 characters, and the surface shows only
what that needs.

###### The character budget, enforced before submission

The product does not let a person write past the limit and only discover it when they try to post. The
counter is live: as the person types, the remaining-character count updates, and it changes appearance
before and at the boundary so the limit is felt, not just stated.

- While there is comfortable room, the counter is quiet (`color.text.secondary`), showing the number of
  characters still available.
- As the draft **nears** the limit, the counter moves to the warning role (`color.warning`) so the
  person sees the boundary coming while they still have room to adjust.
- When the draft is **over** the limit, the counter moves to the danger role (`color.danger`), the
  over-limit characters are distinguished in the field so the person can see exactly what to cut, and
  the primary action is disabled.

The primary action is disabled whenever the draft is empty or over the limit, so the ordinary way to
reach an invalid post — an empty field or too many characters — is closed before the person can act on
it. If an empty or over-limit submit is reached by any other route (a keyboard shortcut, say), it is
rejected and no chirp is created, matching REQ-PROD-001's rule that nothing is posted in that case. The
limit value shown and enforced is the product's published 280, which the ComposeBox reads from the
product requirement rather than inventing.

Colour never carries the limit state on its own: the warning and danger states of the counter are a
colour **and** the changing number (and, over the limit, the marked-over characters and the disabled
action), so a person who does not distinguish the colours still sees the boundary.

###### The primary action

The primary action is a single **PrimaryButton**, one per surface, labelled with the verb for what it
does — "Post" for a new chirp, "Reply" in the conversation's reply position — never a bare "Submit" or
"OK". It has three conditions a person can reach:

- **disabled** — the draft is empty or over the limit; the button reads as inactive and cannot be
  triggered. This is the resting state of an empty compose surface.
- **ready** — the draft holds between one character and the limit; the button is active and posting is
  the obvious next step.
- **busy** — a publish is in flight; the button shows its loading state in place, holding its label and
  width so the surface does not jump, and it cannot be triggered again.

In the **Compose** dialog the primary action is paired with a **SecondaryButton** ("Cancel") and a
close control, following the Dialog's action-pair convention; inline and in the reply position the
primary action stands with the box and the person leaves the draft by navigating away (which preserves
it, below).

###### States

The surface is designed for the states a person actually meets, not only the happy path. Each state
below is the relevant component's own named state, composed here — the surface does not improvise its
own empty, error, or loading treatment. Where a failure needs to keep a person's draft safe, this
surface composes the product-wide keep-the-draft recovery rather than redefining it; that global
pattern is set elsewhere in the experience design and this surface follows it.

**Empty.** No text yet. The draft field shows a placeholder prompt ("What's happening?" for a new
chirp; a reply placeholder that names who is being answered in the reply position), the counter sits at
the full budget, and the primary action is disabled. The empty surface is calm and ready, not an error
— a person who has not typed anything has done nothing wrong, so nothing is marked red. This is the
ComposeBox *empty* state with the PrimaryButton in its *disabled* state.

**Error.** Two different things can be wrong, and they are shown differently because the person does
something different about each:

- *Over the limit* — the draft is longer than 280 characters. This is shown while typing: the counter
  is in the danger role, the over-limit characters are distinguished in the field, and the primary
  action is disabled. There is no separate error message to dismiss; the fix is visible in the field
  and the counter, and the person edits the draft down. This is the ComposeBox *error* state for the
  over-limit case.
- *The publish failed* — the person posted a valid draft and the publish did not succeed (the network
  dropped, the server refused, the session expired mid-submit). The draft is kept exactly as written,
  an **InlineError** appears below the field in plain words with a way to try again, and the primary
  action returns to its ready state so the person can retry without retyping. A failed publish never
  clears or loses the draft. When the cause is an expired session, the surface follows the product's
  session-expiry convention — the person is routed to sign in with the reason stated, their draft is
  preserved, and they return to it afterwards — which this surface composes rather than redefines. In
  the **Compose** dialog this error shows inside the panel and the dialog stays open with the draft
  intact, per the Dialog's *error* state.

**Loading.** A publish is in flight. The primary action shows its busy state and the draft field locks
briefly so the chirp cannot be edited or submitted again mid-publish — this is what stops a double
tap from posting the same chirp twice. The lock is brief and tied to the in-flight publish, not a
general disabling of the surface. This is the ComposeBox *loading* state with the PrimaryButton in its
*loading* state. On success the chirp is published, the field clears to its empty state, and the result
is confirmed with a brief, non-blocking **Toast** ("Chirp posted") that does not interrupt what the
person does next; the new chirp appears in the places the timeline and conversation screens own. On
failure the surface moves to the publish-failed error above.

###### The three placements

The same ComposeBox, with everything above, appears in three places. The behaviour — the counter, the
280-character limit, the disabled/empty/error/loading treatment, and the keep-the-draft recovery — is
identical in all three; what differs is where the box sits and what its action is called.

- **Inline, at the top of the Home timeline.** The box sits above the timeline feed so a signed-in
  person can write without leaving home. Its action is "Post"; on success the posted chirp appears at
  the top of the timeline and the box returns to empty.
- **The Compose dialog (`/compose` and the overlay dialog).** The box is the body of the **Dialog**
  (modal variant), reachable from the always-available compose affordance in the signed-in **NavShell**
  and directly at the `/compose` URL. The dialog traps focus while open, the browser back control
  dismisses it (closing the dialog, not leaving the underlying screen), and a dismissed dialog with a
  non-empty draft follows the keep-the-draft convention rather than discarding the writing. Its action
  is "Post".
- **The reply position on the Chirp / conversation screen.** The box sits in the conversation's reply
  slot, its placeholder names the chirp being answered, and its action is "Reply". The **Chirp /
  conversation** screen composes this variant by name; the counter, the limit, the disabled/error/
  loading behaviour, and the keep-the-draft recovery are all as designed here. Posting a reply from
  this position is the writing half of REQ-PROD-003; the conversation screen owns reading the
  conversation and reporting a reply to a chirp that is gone.

##### The Chirp / conversation screen — reading in context and replying

This is the screen where a single chirp is read together with the chirp it answers and the replies it
received, and where a reply is written. It is the second half of the core loop's writing-and-reading
surfaces: the compose/post surface above is where a chirp starts; this screen is where a chirp is read
in the conversation it belongs to and answered. It is reached by opening a chirp from the timeline, a
profile, a notification, or a shared link, and it has a stable address so any of those routes — and an
outside link — land on the same place.

The bar here is a designed screen, not a route with a list under it: the focused chirp, the chirp it
answers, the replies, and the place to reply are all laid out and specified, and the screen is designed for
the cases that are not the happy path — a chirp with no replies yet, a chirp that has been removed, a page
that fails to load, and the moment before anything has loaded at all.

The reply control is the shared **ComposeBox** in its reply-position placement — it is composed here by
name, and its character counter, its 280-character limit, its disabled/empty/error/loading behaviour, and
its keep-the-draft recovery are the component library's and the compose/post surface's to specify, not this
screen's.

###### Screen

- **Inventory name**: Chirp / conversation
- **Route**: `/@:handle/chirp/:id` — a stable, shareable address (per the IA's addressable-chirp
  convention); opening it directly lands the person on this chirp in its conversation, not on a generic
  home.
- **Archetype**: feed/list
- **Auth**: both — it renders inside the **signed-out shell** for a visitor on a shared link and inside the
  **signed-in shell** for a member. The conversation reads the same in both; the one difference is the
  reply slot (below).
- **Reached from**: a **ChirpCard** opened in the home timeline, a profile, the notifications list, search
  results, or an outside link. Per the IA wayfinding convention the screen offers a labelled way back to
  where the person came from, and the browser back control returns them to that place without re-submitting
  anything and without losing their scroll position in the list they left.

###### Layout (default)

Top to bottom, within the appropriate **NavShell**:

1. **Wayfinding** — the shell marks where the person is; the document title and the main heading name the
   screen (the focused chirp's author and a short form of its text), and a labelled back control sits at the
   top of the content column (per the IA back/wayfinding convention — composed, not redesigned here).
2. **The chirp being answered (parent context)** — when the focused chirp is itself a reply, the chirp it
   answers is shown directly above it as a **ChirpCard**, introduced by a quiet relation line ("Replying to
   @handle"). The parent card is a target that opens *its* conversation, so a person can walk up the thread
   one step. When the focused chirp is a top-level chirp (answers nothing), this region is simply absent —
   not an empty slot.
3. **The focused chirp** — the subject of the screen, rendered as a **ChirpCard** given the visual weight of
   the thing the screen is about (the root of this view): author **Avatar**, display name, handle, relative
   timestamp, the chirp text, and the action row (**IconButton**s for reply, like, and more-actions). The
   reply action here scrolls to and focuses the reply slot below.
4. **The reply slot** — the primary writing affordance on this screen:
   - *Signed in*: the shared **ComposeBox** in its reply-position placement, pre-framed with "Replying to
     @handle" so it is clear what the reply answers, and carrying its own "Reply" **PrimaryButton**. The box,
     its counter, its limit, and its keep-the-draft behaviour are composed by name from the component
     library and the compose/post surface — not restated here.
   - *Signed out*: the box is replaced by a short "Sign in to reply" prompt leading to **Sign in** / **Sign
     up** in the signed-out shell, so a visitor who wants to answer is given the path to an account rather
     than a dead control or a silent one. (This composes the **ComposeBox**'s "composing not permitted in
     the current state" case together with the signed-out shell's own entries.)
5. **The replies** — a **ListView** of **ChirpCard**s, in oldest-first (chronological) order so the
   exchange reads the way it happened, with a reply count heading it ("12 replies"). It loads in bounded
   pages with the IA's load-older affordance for a long thread; it is never one unbounded load. Each reply
   card is a target that opens that reply in its own conversation, so a reply with replies of its own can be
   drilled into.

###### Content

- The focused chirp and every reply show author name, handle, relative timestamp (`2m`, `4h`, then an
  absolute date), and text, from the **ChirpCard** vocabulary.
- The parent-context relation ("Replying to @handle") and, on a reply card, the same relation where a reply
  answers another reply, so a reply is always presented as answering its parent rather than stranded in a
  flat list.
- The reply count above the list.
- In the reply slot, the "Replying to @handle" framing and the live remaining-character affordance that the
  **ComposeBox** owns.

###### Primary action

**Reply** — writing and publishing a reply from the reply slot, via the **ComposeBox**'s "Reply"
**PrimaryButton**. A published reply records the chirp it answers and appears in the replies list in its
chronological place; brief non-blocking feedback (the **Toast** / **InlineFeedback** pattern) confirms it,
per the product-wide "the result of every action is shown" convention.

###### States

- **empty** — a chirp that has no replies yet. The focused chirp and any parent context render normally; the
  replies region shows the **ListView** *empty* state as an **EmptyState** block that invites the first
  reply ("No replies yet — be the first to reply") and points at the reply slot, rather than a blank area.
  The reply slot (the **ComposeBox**, signed in) is present as the next step, so an empty conversation still
  offers the one thing to do on it.
- **error** — three distinct failures, each shown where it happens rather than as a silent drop:
  - *The focused chirp has been removed* (deleted, blocked, or otherwise unavailable): it renders as the
    **ChirpCard** *error* tombstone ("This chirp isn't available") in place of the card, rather than an
    empty gap, and the reply slot is withdrawn — there is nothing to answer. A completely unresolved address
    (no such chirp id) falls through to the inventory's **Not found** screen, which explains the resource is
    missing and routes the person back to a known place.
  - *Replying to a chirp that has gone away between opening the screen and sending* (the parent was deleted
    while the reply was being written): the reply is reported as not-found through an **InlineError** at the
    **ComposeBox** (and the focused chirp flips to the tombstone), and the draft is kept so the words are
    not lost — never a reply that appears to send into nothing.
  - *The replies fail to load* while the focused chirp loaded fine: the **ListView** *error* state shows an
    **InlineError** with a retry in place of the missing replies; the focused chirp, the parent context, and
    the reply slot stay usable, and a failure while loading *older* replies shows at the load-more boundary
    without discarding the replies already on screen.
- **loading** — the first load shows the **LoadingState** skeleton rather than a spinner on a blank screen:
  a skeleton **ChirpCard** at the focused chirp's real dimensions (and one in the parent-context slot when
  the chirp is known to be a reply), and **ListView** skeleton reply rows, so the layout does not jump when
  the content arrives. The reply slot renders immediately — the **ComposeBox** needs no data to appear — so
  the person can begin writing before the replies finish loading.

###### Consistency notes (composed, not redefined)

- The per-screen empty / error / loading states above are the named states of the components that build the
  screen (**ChirpCard**, **ListView**, **ComposeBox**, **EmptyState**, **LoadingState**, **InlineError**)
  together with the product-wide empty / loading / error patterns and the keep-the-draft recovery owned by
  the considered-states surface. In particular, a failed reply publish keeps the draft, and an expired
  session mid-reply routes to sign-in with the reason stated and the draft preserved, returning the person
  here afterwards — both are composed from that global treatment and the **ComposeBox**'s own behaviour, not
  reinvented on this screen.
- Addressability, deep-linking, pagination/catch-up, and back/forward behaviour are the IA's conventions,
  applied here by name.

---

#### Social actions — the in-place like and follow controls

Liking a chirp and following a person are the two reactions in the core loop. Neither is a screen:
each is a control that lives on another screen's component — the like control on every **ChirpCard**,
the follow control on a **ProfileHeader** and on the person rows the discovery and list screens
present. This part specifies how those two controls read and behave wherever they appear, built from
the frozen component library and composed against the per-screen states its components already carry.
It does not redefine **ChirpCard**, **ProfileHeader**, **IconButton**, **PrimaryButton**,
**SecondaryButton**, or **InlineFeedback**, and it does not design the cards, headers, or lists the
controls sit on — those are the timeline, conversation, profile, and discovery surfaces.

##### The behavior both controls share

Both controls act in place — the person stays on the screen they are on, and nothing navigates.
Both are **idempotent**: the control holds exactly two mutually exclusive states (liked / not-liked;
Following / not-following), so repeating the same intent cannot stack. The value the control shows is
the resolved result, not a client-side tally — a like count is the server's count for that chirp, and
a follow is either on or off, never counted twice.

The response is immediate and optimistic, then reconciled:

- On activation the control flips to the target state at once (the heart fills and the count moves;
  the button becomes "Following") and shows its own in-flight **loading** treatment while the change
  settles.
- On success the optimistic state stands; the state change is itself the confirmation, so no extra
  toast is raised for a result that already shows on the control (per the product's transient-feedback
  convention, a result that belongs against a specific control is shown at that control, not in a
  **Toast**).
- On failure the optimistic change is reverted — the control returns to the state it held before, and
  any count returns to its prior value — and an **InlineFeedback** in the danger role appears at the
  control, in plain words, with the action still available to retry. The control never shows a like or
  a follow that did not take.

Meaning is never carried by colour alone: every active state is a colour together with an icon or a
label, matching the library's rule for its components.

##### Like / unlike

**Where it sits.** The like control is the like **IconButton** in the action row of every **ChirpCard**
— the same control wherever a ChirpCard is drawn: the Home timeline, the Chirp / conversation screen,
a profile's chirps, and search and notification results. It reads and behaves identically in all of
them.

**Content.** The heart **IconButton** and, beside it, the chirp's like count in tabular numerals so a
changing count does not shift the row. The control exposes a text label to assistive technology in both
states ("Like" / "Liked").

**Primary action.** Toggle this chirp's like for the signed-in person: like it when not liked, unlike
it when liked.

**States.**

- **default / active.** Two states. *Not liked* — the outline heart at rest, the current like count.
  *Liked* — the active treatment the **ChirpCard** defines: `color.like`, a filled heart, and a "Liked"
  label together (never colour alone), with the count raised by this person's like. The two states are
  mutually exclusive, which is what makes a second "like" impossible — activating an already-liked
  control unlikes it rather than liking again, so the count cannot double-count.
- **error.** A like or unlike that fails: the optimistic flip is reverted (the heart returns to its
  prior fill, the count returns to its prior value) and the like **IconButton** shows **InlineFeedback**
  in the danger role at the control with a plain reason and a retry. The count shown always matches the
  result that actually happened.
- **loading.** The request is in flight: the like **IconButton** shows its own busy treatment (a
  bounded spinner in place of the heart) and is not re-triggerable until it resolves, so a rapid double
  activation cannot post two conflicting changes.

##### Follow / unfollow

**Where it sits.** The follow control is the context action on a **ProfileHeader** and the follow
action on each person row the Explore / find-people, Followers, and Following screens present (those
rows are the discovery-and-lists surface; this part specifies only the control that sits on them). It
is the same control in every place.

**Content.** A single context action: a **PrimaryButton** labelled "Follow" when the person is not
followed, a **SecondaryButton** labelled "Following" when they are (the calmer weight signals the
already-done state and carries the unfollow affordance). On a **ProfileHeader**, the follower and
following counts sit beside it in tabular numerals so the layout holds as a count changes.

**Primary action.** Toggle whether the signed-in person follows this account: follow to bring that
account's new chirps into the home timeline, unfollow to remove them.

**Cannot follow yourself.** Your own identity never presents a Follow control. On your own
**ProfileHeader** the context action is "Edit profile" (a **SecondaryButton**), not "Follow", and a
person row that represents you offers no follow affordance. Because no surface draws a Follow control
for your own account, a self-follow cannot be started from the product. If one is nonetheless submitted
(for example from a stale row), it is refused: the control does not enter the Following state, it
returns to "Follow", and **InlineFeedback** in the danger role states plainly that you cannot follow
your own account.

**States.**

- **default / active.** Two states. *Not following* — the "Follow" **PrimaryButton**. *Following* — the
  "Following" **SecondaryButton**. The two are mutually exclusive, so following an account you already
  follow does not stack a second follow; the control simply stays "Following".
- **error.** A follow or unfollow that fails: the optimistic label flip is reverted (back to its prior
  "Follow" or "Following"), any affected follower count returns to its prior value, and the control
  shows **InlineFeedback** in the danger role with a plain reason and a retry. The self-follow refusal
  above uses this same treatment.
- **loading.** The request is in flight: the button shows its busy treatment (a bounded spinner,
  label and width held so the row does not jump) and is not re-triggerable until it resolves.

---

#### Profiles, followers/following, and discovery

This part of the core loop covers how a chirper meets other people: the profile where someone's
identity and chirps live, the followers and following lists that hang off it, and the Explore screen
where a person looks for someone to follow in the first place. These are the screens a stranger uses to
decide who is worth following and to find their way to those accounts, so each is designed as a real
screen — layout, content, one clear primary action, and the empty, error, and loading states it meets in
ordinary use — not a route with a name.

Every screen here is built from the frozen component library and named in the screen inventory; nothing
below invents a screen or a component outside those vocabularies. Two behaviours that appear on these
screens are owned elsewhere and are composed, not redesigned here: the **follow / unfollow** control (the
social-actions part) and **edit profile** (the account-lifecycle part). Where this part shows one of
them, it places the existing control and links to the existing screen.

##### Public profile — `/@:handle` (profile archetype; reachable signed-out and signed-in)

A profile is the public face of one account and the thing a person reads before deciding to follow. It is
reachable without a session, because people arrive here from shared links and from replies, and a visitor
on such a link should land on real content with a visible way to join rather than an auth wall.

**Layout.** The reading column at `space.measure`, inside the shell for the current auth state
(`NavShell` signed-out for a visitor, signed-in for a member). Top to bottom:

- **`ProfileHeader`** — `Avatar` at `avatar.xl`, the display name in `type.title`, the `@handle` and bio in
  `type.body` / `color.text.secondary`, and the follower and following counts in
  `type.meta` + `type.numeral.tabular` so a changing count does not shift the layout. Each count is a
  labelled link that opens the Followers or Following list.
- Below it, a **`ListView`** of the account's own chirps as `ChirpCard`s, newest first, in bounded pages
  with the load-older affordance — the same `ListView` and `ChirpCard` the timeline uses, so a chirp reads
  the same here as anywhere.

**Content.** Handle, display name, bio, the two counts, and the person's chirps. Nothing here needs a
session to be shown (REQ-PROD-007).

**Primary action.** The header's context action: for another account, the **follow control** owned by the
social-actions part (`PrimaryButton` "Follow" / `SecondaryButton` "Following"), placed in the header and
behaving as that part specifies — this part does not redesign it. A signed-out visitor who presses it is
routed to sign-in and returned to the profile (the IA's auth-gated routing), so follow is never a silent
dead-end for someone without a session.

**States.**
- **Empty** — a sparse profile (no bio, zero followers and following, no chirps yet): the `ProfileHeader`
  *empty* state renders the bio space and the counts as calm, labelled zeros ("0 followers", "0
  following") with a one-line note rather than blank gaps, and the chirp `ListView` shows its `EmptyState`
  ("@handle hasn't chirped yet") instead of an empty column. The header is never shown as broken just
  because it is sparse.
- **Error** — three cases, each distinct from empty so the two are never confused: (1) the profile fails
  to load → the `ProfileHeader` *error* state via an `EmptyState` error variant ("We couldn't load this
  profile — try again") with a retry; (2) the account is blocked, suspended, or does not exist → a plain
  statement of that in place of the header ("This account doesn't exist" / "This account is unavailable"),
  not a half-drawn header and not the generic Not-found screen; (3) a page of chirps fails while the
  header loaded fine → the chirp `ListView` *error* state (`InlineError` with retry) in place of the
  missing page, keeping any chirps already shown.
- **Loading** — the `ProfileHeader` *loading* skeleton (skeleton avatar, name line, bio line, count line)
  above the `ListView` *loading* skeleton rows, both at their real dimensions so the screen does not jump
  when content arrives. A first load shows skeletons, not a spinner on a blank column.

##### Own profile — `/@:handle` for the signed-in owner (profile archetype; signed-in)

The same screen a member's own account presents to the world, shown to the owner with the one thing a
visitor does not get: the way in to change it.

**Layout and content.** Identical to the public profile — `ProfileHeader` over a `ListView` of the
owner's own chirps — so a person sees exactly their own public face. The difference is confined to the
header's action and the per-chirp actions.

**Primary action.** In place of Follow, the header shows **"Edit profile"** as a `SecondaryButton` that
opens the Edit profile screen (`/settings/profile`). Edit profile is the account-lifecycle part's surface;
this part places the entry and links to it, and does not design the editing form. On the owner's own
`ChirpCard`s, the more-actions menu exposes **delete** (the confirmation `Dialog` is owned by the
posting/safety parts) — referred to, not redesigned here.

**States.**
- **Empty** — a brand-new owner who has not chirped: the chirp `ListView` *empty* state speaks to the
  owner rather than about a third party ("You haven't chirped yet — write your first one") and offers the
  Compose affordance as the next step; the header shows the owner's real (possibly zero) counts as labelled
  zeros. This complements onboarding (the account-lifecycle part); it does not replace it.
- **Error** — as the public profile (profile fails to load → `EmptyState` error variant with retry; a
  chirp page fails → `ListView` error with retry). The blocked/suspended case does not arise for one's own
  account; a session that has expired routes to sign-in per the IA and returns the owner here.
- **Loading** — as the public profile: header skeleton over `ListView` skeleton rows.

##### Followers and Following — `/@:handle/followers` and `/@:handle/following` (feed/list; signed-out and signed-in)

The two lists that hang off a profile: who follows this account, and who this account follows. They are
one designed pattern used at two routes, each addressable so a person can link to or bookmark the exact
list. Both are reachable from the profile's count links and carry a labelled way back to the profile.

**Layout.** The reading column, a `type.title` heading naming the list and whose account it belongs to
("People who follow @handle" / "People @handle follows"), then a **`ListView`** of **person rows**. Each
row is `Avatar` at `avatar.md`, display name in `type.heading`, `@handle` in `type.meta`, a one-line bio
in `type.body` / `color.text.secondary`, and — for a signed-in viewer — the **follow control** (the
social-actions part's control) on the trailing edge. The whole row, excluding the follow control, is a
target that opens that person's profile. Loaded in bounded pages with load-older, per the IA's pagination
convention.

**Content.** One row per account, with enough to recognise the person and decide whether to follow them,
and the follow control to act on that decision in place.

**Primary action.** Open a person's profile (the row), or follow/unfollow them from the row's follow
control without leaving the list. A signed-out viewer sees the rows and the counts but is routed to
sign-in if they try to follow.

**States.**
- **Empty** — the `ListView` *empty* state, worded for which list it is: a followers list with no one →
  "@handle doesn't have any followers yet"; a following list with no one → on another account, "@handle
  isn't following anyone yet", and on the owner's own following list, "You're not following anyone yet"
  with Explore offered as the next step, so an empty following list points at the way to fix it.
- **Error** — a page fails to load → the `ListView` *error* state (`InlineError` with retry) in place of
  the missing rows; a failure while loading an older page shows at the load-more boundary and keeps the
  rows already shown.
- **Loading** — the `ListView` *loading* skeleton rows at person-row dimensions on first load; a bounded
  spinner at the boundary when loading older rows.

##### Explore / find people — `/explore` (feed/list; signed-in)

Where a chirper finds someone to follow when they do not already hold a link to a profile. Today a person
can only reach an account through an external link or a reply; Explore is the surface that closes that
gap (REQ-PROD-042). It is in scope and fully designed here; the release bar decides whether a first
release ships it (see the gating note on REQ-UX-064).

**Layout.** The signed-in shell with Explore marked as the active destination, then the reading column:

- A **`TextField`** search input at the top (label "Find people", a plain placeholder), the one input on
  the screen.
- Below it, a **`ListView`** of results as person rows (the same row as the followers/following lists —
  `Avatar`, name, handle, bio snippet, and the follow control), in bounded pages.

**Content.** A query field and a list of people who match it, each with enough to recognise them and the
follow control to act. Before any query, the screen shows whatever suggestions the product can offer
(for example, a short list of accounts to follow); if it can offer none, it shows the prompt alone.

**Primary action.** Search for a person, then follow a result from its row (the social-actions control)
or open their profile — without leaving Explore.

**States.**
- **Empty** — two distinct cases: (1) no query entered yet → a prompt to search, plus any available
  suggestions; if there are no suggestions, the `EmptyState` prompt stands alone ("Search for people to
  follow") rather than a blank screen; (2) a query that matches no one → the `ListView` *empty* state as a
  "no results" block ("No people found for that search") with a way to refine the query. The
  no-query and no-results states read differently so a person can tell "nothing searched yet" from
  "nothing found".
- **Error** — the search request fails → the `ListView` *error* state (`InlineError` with retry) in place
  of results, the query preserved so the person can retry without retyping.
- **Loading** — `ListView` *loading* skeleton rows while a search resolves; a bounded spinner at the
  boundary when loading further pages of results.

##### How these screens compose with the rest of the product

- **Shared vocabulary.** Every screen above is an inventory screen referred to by its name and route, and
  is assembled from library components (`ProfileHeader`, `ListView`, `ChirpCard`, `Avatar`, `TextField`,
  `EmptyState`, `LoadingState`, `InlineError`, and the action buttons). No screen or component outside
  those vocabularies is introduced.
- **Borrowed controls.** The follow/unfollow control (the social-actions part) and the Edit profile
  screen (the account-lifecycle part) are placed and linked, not redesigned.
- **States against the global patterns.** Each per-screen empty, error, and loading state above is the
  relevant component's own named state, and each is consistent with the product-wide empty / loading /
  error patterns and the expired-session recovery owned by the considered-states part. These screens
  compose that global treatment; they do not define a competing per-screen pattern.

*Scope note.* This part designs the view-a-profile surfaces (public profile, own profile, followers,
following) and the discovery/find-people surface (Explore). The follow/unfollow control these screens
carry is designed by the social-actions part; edit profile and account onboarding are the
account-lifecycle part's; the product-wide empty/error/loading patterns and expired-session recovery are
the considered-states part's. All are composed and referenced here, not redesigned.

---

### Account lifecycle — onboarding, management, and leaving

This is the **account-lifecycle** part of the screens-&-flows deliverable: the surfaces that let a signed-in
person make the account their own and, when they want to, leave it cleanly. At the current commit the live
product shows only a timeline, a compose-and-sign-in view, and a profile; there is no way to finish a first
run, edit a profile, change a setting, see where you are signed in, export your data, or delete your
account. These eight surfaces close that gap. Their absence — not their styling — is the defect they fix.

Each surface below is designed to the bar *designed screens, not a skeleton*: its place in the screen
inventory, its layout, its content, its primary action, and its empty, error, and loading states. Every
screen is an arrangement of components named in the component library, realising a screen named in the
screen inventory; each screen's states reuse the components' own named states and the product-wide
empty / loading / error patterns rather than inventing a per-screen treatment. Where a behaviour is owned
elsewhere — the sign-out transition, the session-expiry recovery, the product-wide state patterns, the
notifications activity list, the credential and session mechanism — it is referred to by name, not
redesigned here.

The eight surfaces are assembled in the screen inventory's own order:

1. **Onboarding** — `/welcome`
2. **Edit profile** — `/settings/profile`
3. **Settings hub** — `/settings`
4. **Account & contact** — `/settings/account`
5. **Notification preferences** — `/settings/notifications`
6. **Sessions & devices** — `/settings/sessions`
7. **Export your data** — `/settings/export`
8. **Delete account** — `/settings/delete`

The consolidated requirements register (`REQ-UX-065..077`) follows the eight screens.

---

#### 1. Onboarding — `/welcome`

**Inventory screen**: Onboarding · **Route**: `/welcome` · **Archetype**: empty/landing · **Auth**: signed-in ·
**Realised by**: **EmptyState** inside the signed-in **NavShell**. **Serves**: J5 (first run) ·
**REQ-PROD-032**. **Composes with**: the sign-up → onboarding landing (an IA convention — a completed
sign-up establishes the session and lands here, so this screen neither repeats nor redesigns sign-up).

Onboarding answers the one question a new account has in its first second — "I'm in, now what?" — when the
home timeline is empty because the person follows no one and has posted nothing. It replaces the blank
timeline with an explanation and a way forward.

**Layout.** The signed-in **NavShell** frames the screen as it does every signed-in surface, so Home,
Explore, Notifications, Profile, Compose, and the account menu are already reachable. Within the 600px
reading measure (`space.measure`), a single centred **EmptyState** block: a calm flat illustration in the
brand hues at the top, a headline in `type.title`, a short explanation in `type.body` /
`color.text.secondary` beneath it, and a stacked pair of actions below, separated by the generous
`space.10`/`space.12` rhythm the empty/landing archetype uses. The screen is explanation and next step —
nothing to scroll, nothing to dismiss.

**Content.**
- **Headline** (`type.title`): names where the person is and why it is empty — for example *"You're in. Your
  timeline's empty for now."*
- **Explanation** (`type.body`): one or two plain sentences saying the timeline fills with chirps from the
  people they follow and their own posts, so the two ways to fill it are to follow someone or to post.
- **Primary action — "Write your first chirp"** (**PrimaryButton**): opens **Compose**. This is the route
  that is always available: a person can post a first chirp without anyone else existing, so onboarding can
  always offer at least one way to a non-empty timeline even where discovery is not present.
- **Secondary action — "Find people to follow"** (**SecondaryButton**): opens **Explore / find people**
  (`/explore`). Explore rides a non-gating product requirement, so this route is offered when discovery is
  present and is the richer path to a non-empty timeline; the compose route above is the guaranteed one that
  does not depend on it.
- A quiet **"Skip for now"** text control (a **SecondaryButton** in its lowest-weight form) leaves onboarding
  for the home timeline, so the screen never traps a person who would rather look around — the empty timeline
  they then see is the **ListView** *empty* state, which itself points back to Explore and compose.

**Primary action**: Write your first chirp → **Compose**.

**States.**
- **Default / empty.** Onboarding is an empty/landing screen, so its resting state *is* the explanatory
  **EmptyState**: illustration, headline, explanation, and the two routes. There is no separate "empty"
  variant to design — the screen is the designed answer to an empty timeline, never a blank page.
- **Loading.** The static explanation and the two action controls render immediately with the **NavShell**,
  so the screen never blanks while anything resolves. If onboarding shows an optional preview of suggested
  people to follow (where discovery is present), that region alone shows **LoadingState** skeleton rows at
  the size of the coming rows while the rest of the screen is already usable; the person can act on either
  route before the preview settles.
- **Error.** The screen's core — explanation and the two routes — has nothing to fail, so it stays up. If an
  optional suggestions preview fails to load, that region shows the **EmptyState** *error* variant ("We
  couldn't load suggestions — you can still find people or write your first chirp") with a retry, and both
  routes remain present, so a failed preview never costs the person their way forward. An expired or invalid
  session taking them here is handled by the product-wide expired-session treatment (routed to sign-in with
  the reason stated and returned afterward) — that pattern is referred to, not redesigned here.

---

#### 2. Edit profile — `/settings/profile`

**Inventory screen**: Edit profile · **Route**: `/settings/profile` · **Archetype**: form · **Auth**: signed-in ·
**Realised by**: **TextField** + **PrimaryButton** + **SecondaryButton** + **InlineError** in a form layout,
inside the signed-in **NavShell**. **Serves**: J6 (managing the account) · **REQ-PROD-016**. **Reached from**:
the **Settings** hub as a listed sub-section, and from the "Edit profile" action on the person's **Own
profile** (the **ProfileHeader** **SecondaryButton**) — both entries are designed elsewhere and referred to
here, not redefined.

Edit profile is where a person changes how they appear after sign-up: their display name and their bio. A
change saved here shows on their profile the next time it is viewed.

**Layout.** Inside the signed-in **NavShell**, within the 600px measure, a short form:
- a heading in `type.title` naming the screen ("Edit profile"), which also sets the document title, so a
  person always knows where they are;
- a labelled way back to the **Settings** hub, consistent with how every settings sub-section returns to the
  hub;
- a **TextField** for the **display name** — single-line, labelled, required;
- a multi-line **TextField** for the **bio** — labelled, optional, showing its length allowance where one
  applies (the exact limit is a domain decision, surfaced on the field, not invented here);
- the handle shown as a read-only value with a one-line note that it is managed under **Account & contact**
  (where the product states its position on whether a handle can change) — this screen does not edit the
  handle;
- an action row pinned below the fields: **"Save changes"** (**PrimaryButton**) and **"Cancel"**
  (**SecondaryButton**).

**Content.** The current display name and bio are pre-filled so the person edits rather than retypes. Labels
and the Save verb follow the voice rules — sentence case, a verb for the action ("Save changes", not
"Submit"). Field hints say what a field is for in plain words.

**Primary action**: Save changes → persists the display name and bio and confirms the save; the new values
show on the person's profile on the next view.

**States.**
- **Default.** The form pre-filled with the current display name and bio. **Save changes** is disabled (its
  **PrimaryButton** *disabled* state) until the person makes a change, so the control never offers to save
  nothing; **Cancel** returns to the hub (or to the profile, when the person arrived from the "Edit profile"
  action) without writing anything.
- **Empty.** The optional **bio** left empty is valid — the field shows its label and hint, not an error (the
  **TextField** *empty* state). The required **display name** emptied is not flagged while the person is still
  working; only on a save attempt with an empty display name does it enter the error state below.
- **Error.** Two cases, each shown where the problem is, in plain words, with no system code and no blame, and
  without discarding what the person typed:
  - *Field-level* — an empty required display name, or a value the product rejects (for example too long),
    shows the **TextField** *error* state: a `color.danger` border and an **InlineError** beneath naming the
    problem and the fix ("Add a display name so people can recognise you").
  - *Save-level* — a save that fails on a transient or server condition shows an **InlineError** at the form
    with a plain reason and keeps every edited value so the person can retry; this is consistent with the
    product-wide keep-the-draft recovery for a transient error, which is referred to, not redesigned.
- **Loading.** On open, if the current values are still being fetched, the field region shows **LoadingState**
  skeletons at the fields' size while the form chrome and the **NavShell** render immediately. On save, **Save
  changes** shows its **PrimaryButton** *loading* (busy) state and the fields lock briefly so a change is not
  double-submitted; on success the save is confirmed with a **Toast** ("Saved") — or **InlineFeedback** at the
  action row — and the updated values are what the profile then shows.

---

#### 3. Settings hub — `/settings`

**Inventory screen**: Settings · **Route**: `/settings` · **Archetype**: settings · **Auth**: signed-in.
**Serves**: J6 (managing the account) · **REQ-PROD-017** (an account settings surface).

The Settings hub is where a signed-in person goes to manage their account. It is a navigation surface: it
lists every settings sub-section and leads into each, but designs none of their contents. It is the small
secondary navigation the IA describes (REQ-UX-017): each sub-section it lists is a screen a person can open
directly by its own URL and return from to the hub.

**Layout.** The hub renders inside the signed-in **NavShell**, in the 600px reading column (`space.measure`),
centred on wider viewports; on a phone it fills the viewport with a `space.gutter.mobile` (16px) edge. Top
to bottom:
- **Screen heading** — "Settings" in `type.title`, which the document title names too. The active primary-nav
  destination is marked in the shell.
- **Account summary** — a quiet, non-interactive header that names whose account this is: **Avatar** at
  `avatar.lg`, the display name in `type.heading`, the handle in `type.meta` / `color.text.secondary`. It
  identifies the signed-in account at a glance, which matters on a shared or borrowed device. It is context,
  not a control, and edits nothing — editing the profile is the first row below.
- **Grouped sub-section list** — the sub-sections as **SettingRow**s, gathered under short section labels
  (`type.heading`, `space.6` above each group) so the list is scannable rather than a flat run of rows.
  `color.border.default` hairlines separate the rows; each row has `space.4` padding.

There is no secondary content, no form, and no footer action; the hub is a list of destinations.

**Content — the sub-sections the hub lists.** Every settings sub-section named in the inventory appears here,
each as a **SettingRow** with a label (`type.body`), a one-line description (`type.meta` /
`color.text.secondary`), and a chevron **IconButton** (`icon.md`) as the drill-in affordance. Each row opens
the inventory screen named in its route and is independently addressable.

| Group | Row (label) | Description | Opens (inventory screen / route) |
|---|---|---|---|
| Account | Edit profile | Change your display name and bio. | Edit profile — `/settings/profile` |
| Account | Account & contact | Your verified contact and your handle. | Account & contact — `/settings/account` |
| Notifications | Notification preferences | Choose which activity sends you a notification. | Notification preferences — `/settings/notifications` |
| Privacy and safety | Blocked & muted | Accounts you've blocked or muted. | Blocked & muted — `/settings/blocks` |
| Security | Sessions & devices | Where you're signed in, and signing out. | Sessions & devices — `/settings/sessions` |
| Your data | Export your data | Download your chirps, profile, and follows. | Export your data — `/settings/export` |
| Your data | Delete account | Permanently delete your account. | Delete account — `/settings/delete` |

Notes on specific rows:
- **Delete account** is the destructive row. Its label uses `color.danger` and it carries no inline toggle or
  value — selecting it routes to the Delete account **confirmation** (designed in §8), which names what is and
  is not kept and points to export first. The hub styles and routes this row; it does not delete on a row tap
  (the destructive-confirmation convention, REQ-UX-011).
- **Notification preferences** rides REQ-PROD-029, which the release bar does not gate. The row is present
  when notifications are in the release and absent when they are not; every other row is present in the first
  release. Its absence is the only variation in the list.
- **Blocked & muted** and the Account group's **Edit profile** row link to screens designed elsewhere (the
  safety area and the edit-profile screen in §2); the hub lists them by inventory name and route and designs
  none of their contents.

**Primary action.** The hub's primary action is **opening a settings sub-section** — the drill-in on a
**SettingRow**, the repeated, equally-weighted affordance the whole screen is built around. The Account group
sits first because editing the profile and managing the contact are the most common reasons a person opens
settings. There is deliberately **no single PrimaryButton**: the settings archetype is a list of peer
destinations, not a form with one submit, so a lone call-to-action would misrepresent the screen. Sign out is
not placed here either — it is reached from the account menu in the **NavShell** and designed into Sessions &
devices (§6), so the hub lists the way to it rather than owning the action.

**States.**
- **Loading.** The sub-section list is a fixed menu and renders immediately, the way the **NavShell** does —
  moving into settings never blanks the list behind a spinner. The one part that resolves from the account is
  the **Account summary** header: while it resolves it shows **LoadingState** — the **Avatar** *loading*
  skeleton circle at `avatar.lg` plus two short skeleton lines standing in for the name and handle at their
  real size, so the list below does not jump when the summary arrives. The list is usable throughout.
- **Empty.** Does not apply. The hub lists a fixed set of sub-sections that always exists, so there is no
  empty variant of the screen — a decision that matches the composed components' own: **SettingRow** records
  *empty* as not applying at the row level, and **NavShell** records *empty* as not applying because its
  destinations are fixed. (An individual sub-section with nothing in it shows an **EmptyState** inside that
  sub-section, which its owning screen designs; that is not a state of the hub.)
- **Error.** The sub-section list does not fetch, so it has no load-failure of its own — it always renders.
  Two error paths are composed rather than invented:
  - *Account summary fails to resolve* → the **Avatar** falls back to its *error* treatment (the initials
    monogram, never a broken-image glyph); if the name and handle cannot be read, the summary collapses to a
    quiet header rather than a broken block. The sub-section list below stays fully usable, so a person can
    always reach Sessions & devices or Delete account even when their summary did not load — a cosmetic header
    never blocks the account controls.
  - *Reached without a valid session, or the session expires mid-use* → the product routes to **Sign in** with
    the reason stated and returns the person to `/settings` afterward, preserving any in-progress input
    (REQ-UX-007, REQ-UX-008). The hub composes this; it does not redesign it.

---

#### 4. Account & contact — `/settings/account`

**Inventory screen**: Account & contact · **Route**: `/settings/account` · **Archetype**: settings · **Auth**:
signed-in. **Serves**: J6 (managing the account). **Traces to**: REQ-PROD-017 (an account settings surface),
REQ-PROD-012 (a verified point of contact), REQ-PROD-018 (a stated handle-change position).

This sub-section lets a signed-in person **see and update the point of contact the account is recovered and
notified through**, complete the step that confirms they control that contact before the product relies on
it, and read where the product stands on changing their handle. It designs what the person sees and does; the
credential, session, and verification *mechanism* is the identity design's concern (`REQ-ID-*`), not this
screen's.

**Composition.** Built from the frozen component library in the settings archetype — a list of **SettingRow**s
inside the signed-in **NavShell**. Components used: **NavShell** (signed-in), **SettingRow**, **TextField**,
**PrimaryButton**, **SecondaryButton**, **InlineFeedback**, **InlineError**, **EmptyState**, **LoadingState**,
**Toast**, and the `color.warning` token for the unverified-contact caution. No new screen or component is
introduced.

**Layout.** Inside the persistent signed-in **NavShell**, the main region carries:
- A main heading **"Account & contact"** (also the browser document title) and a labelled **Back to settings**
  control returning to the Settings hub (wayfinding, REQ-UX-003 / REQ-UX-017).
- A short lead line naming what this screen is for ("The contact we use to help you recover your account and
  to reach you about it.").
- A list of **SettingRow**s, in this order:
  1. **Contact** — a SettingRow whose value slot shows the current point of contact (or its not-set
     treatment, below) together with a **verification-status** indicator:
     - *Verified* — a neutral/`color.success` mark reading "Verified", paired with the word, never colour
       alone.
     - *Unverified / not yet confirmed* — a `color.warning` caution reading "Not verified yet", paired with
       the word. This is a **caution, not an error**: the contact is present and usable to change, it has
       simply not been confirmed, so it is not shown with the `color.danger` error treatment.
     The row's control slot holds a **SecondaryButton "Change"**. Activating it reveals an inline **TextField**
     (labelled "Contact", holding the current value) with a **PrimaryButton "Save changes"** and a
     **SecondaryButton "Cancel"**. When the contact is unverified, the row also offers a **"Verify now"**
     action that starts the control-you-own verification step and then shows the contact as pending
     confirmation. What that step does under the hood — the code, link, token, or timeout — is the identity
     design's `REQ-ID-*`; this screen shows only the person-facing states: an action to start verification, a
     pending/"check your contact to confirm" indication, and the resolution to Verified or back to a caution
     with a plain reason.
  2. **Handle** — a SettingRow whose value slot shows the person's `@handle`, with a description that **states
     the product's position on changing it**. For a first release the stated position is that **the handle is
     fixed**: the description reads "Your handle is the name others follow you by. It can't be changed after
     sign-up." — and the row offers **no change control**. If a later release decides a handle may change, this
     same row is where the working change control lives and the stated position updates to match. The screen's
     behaviour always matches the position it states (REQ-PROD-018).

**Primary action.** **Save the updated contact** — the **PrimaryButton "Save changes"** in the Contact row's
edit mode. Saving a new or changed contact records it and moves it to the unverified/pending state, from which
the person completes verification. Viewing and updating the verified contact is this screen's reason to exist,
so this is its one primary action; the handle row is a stated-position read, not a competing action.

**Feedback conventions composed here.** A completed save is confirmed with brief, non-blocking feedback — a
**Toast** ("Contact saved") or **InlineFeedback** on the Contact row (REQ-UX-012); the row then reflects the
new value and its unverified status. A started verification shows a pending indication in place ("We've sent a
confirmation to your contact — confirm it to finish") with a way to resend. There is no destructive or
irreversible action on this screen.

**States.** The screen composes the components' own named states and the product-wide empty / loading / error
patterns and the expired-session keep-the-draft recovery (REQ-UX-008); it invents no conflicting per-screen
pattern.
- **Loading.** On first load, while the current contact and its verification status are fetched: a
  **LoadingState** skeleton stands in for the SettingRow list at its true dimensions (a skeleton value block
  for the Contact row, one for the Handle row), inside the **NavShell**, which renders immediately. A small
  in-place save or verify uses the control's own *loading* state — the **PrimaryButton** busy state and the
  **SettingRow** pending treatment — not the whole-screen skeleton.
- **Empty.** The meaningful empty case is **no contact on file yet**. The Contact row does not render blank: it
  shows a `color.warning` caution reading "No contact set yet", a plain line on why it matters ("Add a contact
  so you can recover your account and we can reach you about it" — REQ-PROD-012), and a **PrimaryButton "Add
  contact"** opening the same inline **TextField** used for a change. The whole-screen **EmptyState** block
  does not apply — this sub-section is not a list that can be empty; it always renders its Contact and Handle
  rows. The empty case lives inside the Contact row.
- **Error.** Three person-facing failures, each shown where its action was taken, never a silent no-op
  (REQ-UX-012):
  - *The screen fails to load* its current contact/status → an **EmptyState** error variant ("We couldn't load
    your account settings — try again") with a **retry**, inside the NavShell. This keeps a failed load
    distinct from a contact that is simply not set yet.
  - *A save fails* → a `color.danger` **TextField** border and an **InlineError** beneath the Contact field in
    plain words ("We couldn't save that — check the contact and try again"), with the typed value kept to
    retry. No system or error code is surfaced.
  - *A verification action fails* → **InlineFeedback** in the danger role at the control that started it, with
    a plain reason and a way to try again; the contact reverts to its prior status rather than showing a result
    that did not happen.
  - *The session expires mid-edit* → the person is routed to **Sign in** with the reason stated, their
    in-progress contact edit preserved, and returned here afterward (REQ-UX-008). Composed, not redesigned.

---

#### 5. Notification preferences — `/settings/notifications`

**Inventory screen**: Notification preferences · **Route**: `/settings/notifications` · **Archetype**: settings ·
**Auth**: signed-in. **Serves**: J6 (managing the account) · REQ-PROD-017 (an account settings surface). This
is the place **REQ-PROD-029** (activity notifications with preferences) puts its "set which of these you are
notified about" control. Built only from named components in the component library; it invents no screen or
component.

**What the screen is for.** A person who gets replies, follows, and likes decides which of those are worth a
notification. Each activity category can be turned on or off. A category left on sends a notification when that
activity happens; a category turned off stops notifying — the activity still occurs and still appears in the
person's own view of it, but it no longer produces a new notification. The preferences here decide **which
activity notifies**; the notifications themselves — the activity list a person reads — are the **Notifications**
surface (`/notifications`), designed in the safety & notifications area. This screen links to and refers to
that surface; it does not redesign it.

**Layout.** Realised as the settings archetype: a list of **SettingRow**s inside the signed-in **NavShell**.
- **NavShell (signed-in), persistent.** The shell frames the screen and renders immediately, so moving into
  this sub-section never blanks the frame. Its **Settings** entry is marked active, and its **Notifications**
  destination (with its unseen-activity indicator) remains reachable.
- **Settings as secondary navigation.** This screen is one sub-section of the hub, opened directly by its own
  URL and offering a clear way back to the **Settings hub**; it does not restate the hub listing.
- **Heading.** The main heading and the document title both name the screen ("Notification preferences").
- **Lead line.** One line under the heading states what the choices do — that these control which activity
  sends a notification — and links to the **Notifications** list where those notifications appear.
- **The choices.** A short list of three **SettingRow**s, one per activity category, each carrying a **Toggle**
  in its control slot.

**Content — the rows.** Three rows, one per activity category chirp notifies on (replies, follows, likes). Each
row is a **SettingRow**: a `type.body` label, a `type.meta` / `color.text.secondary` description, and a
control slot holding a **Toggle** with its visible on/off label.

| Category | Row label | Description | Control | Default |
|----------|-----------|-------------|---------|---------|
| Replies | Replies | When someone replies to your chirps. | Toggle (on/off) | On |
| Follows | Follows | When someone follows you. | Toggle (on/off) | On |
| Likes | Likes | When someone likes your chirps. | Toggle (on/off) | On |

The three categories match the activity REQ-PROD-029 names. Each defaults to **on**: the baseline story is to
be notified of relevant activity, and the person turns off what they do not want rather than opting in to
each. The set of categories is fixed and known — it does not grow or shrink at runtime.

**Primary action.** The primary action is **choosing whether an activity category sends a notification** —
flipping a category's **Toggle**. The choice takes effect immediately and is confirmed on its own row; there
is **no separate Save button**, because the **Toggle** persists its change on change (its documented
immediate-effect behaviour). Each row is independently operable.

**Immediate-save behaviour.** The screen composes the **Toggle**'s and **SettingRow**'s own named behaviours:
- **On change** — the **Toggle** flips immediately and persists the new value.
- **Success** — the change is confirmed in place with **InlineFeedback** in the success role on that row. A
  category switched off stops notifications of that category from that point; switched on, they resume.
- **In-flight (loading)** — while the change is written, the row's **Toggle** shows its brief pending treatment
  (the Toggle's *loading* state) and is not re-triggerable until the save resolves.
- **Failure** — the save fails → the **Toggle** reverts to its prior value and the row shows **InlineFeedback**
  in the danger role, in plain words, saying the setting is unchanged and inviting a retry. The control never
  shows a result that did not happen.

**States.**
- **Loading — first load.** The saved preference values are read. The **NavShell** renders immediately; in the
  content area the three rows show **LoadingState** skeleton rows at the real dimensions of a label-plus-Toggle
  row (skeletons, not a spinner on a blank panel), so the layout does not jump when the real rows arrive.
- **Error — the preferences fail to load.** The content area shows an **EmptyState** error variant — "We
  couldn't load your notification settings — try again" — with a retry, in place of the rows. This keeps a
  failed load distinct from a genuinely empty surface. (A failed individual *save* is not this screen-level
  error: it is handled on the row by the Toggle revert plus danger **InlineFeedback**.) An action taken on an
  expired or invalid session follows the product-wide session-expiry recovery — routed to **Sign in** with the
  reason stated and returned here afterward. Composed, not redefined.
- **Empty — does not apply.** The three activity categories are a fixed, known set that always render, so there
  is no "no items" condition. This matches the composed components' own decisions: the **Toggle** records
  *empty* as not applying ("a toggle is always on or off"), and the **SettingRow** records *empty* as not
  applying at the row level.

---

#### 6. Sessions & devices — `/settings/sessions`

**Inventory screen**: Sessions & devices · **Route**: `/settings/sessions` · **Archetype**: settings · **Auth**:
signed-in. **Serves**: J6 (managing the account) · REQ-PROD-017 (view and end active sessions), REQ-PROD-014
(sign-out that ends the session server-side).

This is where a signed-in person sees every place their account is currently signed in, ends a session on a
device that is not in front of them, and signs out. It is a sub-section of **Settings** — reached from the hub,
opened directly at its own URL, offering a labelled way back — built as a list of **SettingRow**s inside the
signed-in **NavShell** against the 600px measure (`space.measure`). It designs what the person sees and does;
how a session or token is represented, validated, or invalidated is the identity design's concern
(`REQ-ID-*`).

**What the person can see and do.**
- **See their active sessions.** Each session the account currently holds is listed, with enough to recognise
  it: the device and browser (for example "Chrome on macOS"), an approximate location, and when it was last
  active. The session in use right now is marked as the current one.
- **End a session on another device.** Any session other than the current one can be ended through the
  **Dialog** confirmation variant that names the effect before it proceeds (REQ-UX-011). Once ended, that
  session can no longer be used on its device.
- **Sign out.** The person can sign out of the current session. Sign-out ends the session server-side and
  leads into the product's sign-out transition — back to the signed-out Landing, with no protected screen
  reachable afterward (REQ-UX-016).

**Layout and content.** Top to bottom, inside the signed-in NavShell with the heading "Sessions & devices"
(`type.title`) and a labelled back control to the Settings hub:
1. **Intro line** — one `type.body` line in `color.text.secondary`: "These are the places you're signed in to
   chirp. End any you don't recognise, or sign out below."
2. **This device** — a single **SettingRow** for the current session, marked "This device · current session"
   with a mark and words (never colour alone). Its content: the device and browser, the approximate location,
   and "signed in <relative time>" / "active now". This row has **no End session control** — the way to end
   the current session is Sign out — so its control slot carries only the current-session marker, keeping the
   one destructive per-row action ("End session") reserved for other devices.
3. **Other sessions** — a **ListView** of **SettingRow**s, one per other active session. Each row's label is
   the device and browser; its description (`type.meta`, `color.text.secondary`) carries the approximate
   location, the relative "last active" time, and a short session identifier in `type.family.mono` so a person
   can tell two similar sessions apart by reading it exactly. Each row's control slot holds an **End session**
   **SecondaryButton** (or an **IconButton** carrying the "End session" label where the row is tight). The
   ListView owns the list-level empty, loading, and error treatments (below) and loads in bounded pages if the
   list is long (REQ-UX-010).
4. **Sign out** — the screen's primary action, presented as a prominent entry beneath the list.

**Primary action.** **Sign out** — the decisive action a person comes here to take on their own session, and
the action the release bar gates (REQ-PROD-014). Ending another device's session is the row-level destructive
action within the Other sessions list; it is confirmed the same way but is not the screen's single primary
action.

**Ending a session on another device (the confirmed, destructive flow).**
1. The person activates **End session** on another session's row.
2. A **Dialog** confirmation variant opens over the `color.bg.scrim`. Title: "End this session?" Body: "The
   session on <device>, <approximate location>, will be signed out. Whoever is using chirp there will have to
   sign in again." The action pair is a destructive confirm — a `color.danger` **PrimaryButton** "End session"
   — and a **SecondaryButton** "Cancel". Cancel closes the dialog and changes nothing.
3. On confirm, the dialog shows its loading state (the confirm button is busy and the dialog stays open until
   the action resolves). The end is server-side, so the ended session can no longer be used on its device.
4. On success, the dialog closes, the row leaves the list, and the result is shown — **InlineFeedback** in the
   success role at the list, or a **Toast** "Session ended" (REQ-UX-012). On failure, an **InlineError** appears
   inside the dialog in plain words and the dialog stays open to retry; nothing is ended until it succeeds.

**Signing out (into the sign-out transition).**
1. The person activates **Sign out**.
2. A **Dialog** confirmation variant opens. Title: "Sign out?" Body: "You'll be signed out on this device and
   returned to the chirp landing." When the account is signed in elsewhere, the dialog also offers "Also sign
   out of my other N sessions" (a **Toggle** within the dialog) — the inventory's Sign out dialog "confirm when
   ending sessions on other devices too", composed here, not a new pattern. The confirm is a **PrimaryButton**
   "Sign out" (destructive styling when "also end others" is on); Cancel is a **SecondaryButton**.
3. On confirm, the current session ends server-side and the screen leads into the product's existing sign-out
   transition: the person lands on the signed-out Landing, and the browser back control does not re-expose this
   settings screen (REQ-UX-016). This screen designs the sign-out entry *into* that transition; it does not
   redesign the transition.
4. The server-side end means the session that was in use is rejected afterward, and the next person at that
   device is not signed in as this account (REQ-PROD-014). A failed sign-out shows an **InlineError** in the
   dialog with a retry; the person is not left in a half-signed-out state.

**States.**

| State | Treatment |
|---|---|
| **Loading** | First load of the sessions list shows the **ListView** / **LoadingState** skeleton rows at the rows' real dimensions, not a spinner on a blank screen, so the layout does not jump when the sessions arrive; the skeleton honours `motion.reduced`. The **NavShell** and the heading render immediately, and the **This device** row renders as soon as the current session is known, while the other sessions resolve. |
| **Empty** (the "only this session" case) | When the account is signed in only on the current device, the Other sessions list has no items, so the **ListView** shows its **EmptyState**: "You're only signed in here. When you sign in on another device, it'll show up in this list." — never a blank area and never an apology. The **This device** row and **Sign out** remain present and usable. |
| **Error** | If the sessions list fails to load, the **ListView** shows its error treatment — an **EmptyState** error variant (or **InlineError**) with a retry: "We couldn't load your sessions — try again." The **This device** row still renders if the current session is known, so the person can still sign out. A failed End session or Sign out is shown as an **InlineError** inside the open **Dialog**, retryable, with nothing changed until it succeeds. An action attempted on an expired or invalid session follows the product-wide session-expiry convention (REQ-UX-008): routed to **Sign in** with the reason stated and returned afterward — composed here, not redefined. |

---

#### 7. Export your data — `/settings/export`

**Inventory screen**: Export your data · **Route**: `/settings/export` · **Archetype**: settings · **Auth**:
signed-in · **Journey**: J8 · **Serves**: REQ-PROD-019.

A person obtains their own data — their chirps, their profile, and who they follow — in a form they can open
outside chirp. The screen takes them through three moments: ask for the export, wait while it is prepared, and
obtain the result. What the product does with the data — how an export file is assembled — is a build and
privacy-design concern; this screen specifies what the person sees, is told, and must do.

**Layout.** A settings screen rendered in the signed-in **NavShell**, with a labelled way back to the Settings
hub (the IA wayfinding convention). Top to bottom, within the 600px measure:
- The screen heading in `type.title` — "Export your data" — and a `type.body` `color.text.secondary`
  explanation that says what the export contains (your chirps, your profile, and the accounts you follow) and
  that it arrives as a file you can open and keep outside chirp.
- A short **content summary** built from **SettingRow**s (label + `type.meta` description, no control), one row
  per data kind — *Your chirps*, *Your profile*, *Who you follow* — so the person can see what they are about
  to receive before they ask for it.
- A one-line note, in `type.meta` `color.text.secondary`, that preparing the file can take a little while and
  that they can leave the page and come back.

**Primary action.** A single **PrimaryButton** — "Prepare my data" — is the one primary action on the view.
Requesting the export is the whole job of the screen; obtaining the result is the **SecondaryButton**
("Download your data") that appears once the file is ready.

**States.**
- **Default / empty (no export yet).** On a first visit, no export has been prepared. The content area is the
  **EmptyState** treatment composed into the section: the explanation above, with "Prepare my data" as the
  offered next step — it explains what an export is and how to get one rather than showing a blank panel. This
  resting "nothing yet" view and the screen's default are the same state, which is correct for a surface whose
  job is to start a request.
- **Loading.** Two distinct loads. (1) The screen's own first paint uses **LoadingState** skeleton rows at the
  real dimensions of the summary rows, so the frame does not jump when content arrives. (2) After the person
  asks, the **preparing** state: the **PrimaryButton** shows its busy state and the section shows a bounded
  **LoadingState** spinner with a `type.body` status line ("Preparing your data…"), repeating that they can
  leave and come back. When the file is ready, the preparing state resolves to the default view with the
  **SecondaryButton** "Download your data" present and a **Toast** ("Your data is ready to download")
  confirming without stealing focus. Nothing spins indefinitely — a preparation that fails hands off to the
  error state.
- **Error.** A failed preparation shows an **InlineError** in the section, in plain words with a retry ("We
  couldn't prepare your export — try again"), and the **PrimaryButton** returns to its resting label so the
  person can ask again; no partial or broken file is offered. If the screen itself fails to load, the content
  area shows the **EmptyState** error variant ("We couldn't load this — try again") with a retry, which keeps
  a load failure distinct from the genuinely-nothing-yet empty state.

---

#### 8. Delete account — `/settings/delete`

**Inventory screen**: Delete account · **Route**: `/settings/delete` · **Archetype**: confirmation · **Auth**:
signed-in · **Journey**: J8 · **Serves**: REQ-PROD-020, REQ-PROD-021, REQ-PROD-022.

A person deletes their account and has their content removed, after a confirmation that names the consequence,
states what is and is not retained, points them to export first, and proceeds only once they have explicitly
confirmed. Cancelling leaves everything exactly as it was.

**Layout.** This surface realises the confirmation archetype as the **Dialog** confirmation variant, opened
over the Settings context and dimmed behind the `color.bg.scrim` (the IA destructive-confirmation convention).
It is reached from the destructive **SettingRow** on the Settings hub (a `color.danger` "Delete account" row)
or directly at `/settings/delete`, which opens the confirmation over the settings surface. Within the panel,
top to bottom:
- A `type.heading` title that names the action as a question — "Delete your account?"
- A `type.body` **consequence** statement in plain words: your account and your chirps will be removed, you
  will not be able to sign in again, and this cannot be undone.
- A **what is and is not retained** block that renders the product's defined deletion outcome (REQ-PROD-021 —
  the specific policy is set by the privacy and identity designs, not invented here). The block has a fixed
  shape so the person always sees the same three points, filled from that stated outcome: *What is removed*
  (your account, profile, and chirps), *What may remain* (what becomes of chirps that others have replied to
  or liked, stated as the product's outcome defines it), and *Your handle* (whether the handle becomes
  available for someone else to register again). Where the product has not yet stated one of these, the screen
  states that plainly rather than implying a guarantee it cannot make.
- A **pointer to export first**, as a `type.body` line and a **SecondaryButton** / link — "Export your data
  first" → `/settings/export` — presented as the person's chance to keep a copy before the account goes.
- An **explicit-confirmation** step: a **TextField** asking the person to type their handle to confirm (or an
  "I understand this can't be undone" acknowledgement). The destructive action stays disabled until this is
  given, so deletion is never one stray click.

**Primary action.** The destructive **PrimaryButton** (danger variant, `color.danger`) — "Delete my account".
Its calm counterpart is the **SecondaryButton** "Cancel", which closes the confirmation and returns the person
to Settings with nothing changed. On a confirmed deletion the session ends and the person is returned to the
signed-out **Landing** (the sign-out-to-signed-out-shell convention), with a brief confirmation that the
account was deleted; protected screens are no longer reachable.

**States.**
- **Default.** The panel open over the dimmed Settings surface, showing the consequence, the retained /
  not-retained block, the export pointer, the confirmation field, and the Delete / Cancel pair.
- **Disabled.** The destructive **PrimaryButton** is disabled until the explicit-confirmation step is satisfied
  (the handle typed, or the acknowledgement given) — the Dialog's disabled-confirm state. Cancel is always
  available.
- **Loading.** On confirm, the destructive **PrimaryButton** shows its busy state and the dialog stays open
  until the deletion resolves, so the action is not re-triggerable and the person is not left guessing.
  Reaching `/settings/delete` opens the confirmation immediately over the settings context, so there is no
  separate screen skeleton.
- **Error.** A failed deletion shows an **InlineError** inside the panel, in plain words with a way to retry;
  the dialog stays open, the account is not deleted, and Cancel still leaves everything intact. The person is
  never told the account was removed when it was not.
- **Empty.** Does not apply — a confirmation always opens with its content; there is no empty variant of this
  surface.

---

### Safety, notifications, and public-explanatory surfaces

chirp is a space where strangers post at each other. That is the product's reason to exist and
also its risk: a person who is replied to by someone they want nothing to do with, or who sees a
chirp that breaks the rules, needs somewhere to turn that is built into the product, not improvised
later. The surfaces in this part are the ones that make chirp feel minded — the place a person sees
what has been directed at them, the place they report a problem, the controls that stop an unwanted
account, the list of who they have shut out, the way to get help, the rules a report points to, and
the screen a person lands on when a link no longer resolves. None of these is the product's headline;
each is part of whether a stranger trusts it.

Every screen below is a screen named in the **Screen inventory** and is assembled from
**Component library** components by name; nothing here redefines either. Each screen is designed with
its layout, its content, its primary action, and its empty, error, and loading states. A state marked
"does not apply" is a design decision with its reason, in the component library's sense, not an
omission. The per-screen states compose the components' own named states and the product-wide
patterns owned by the `empty_error_loading_states` section; they do not invent a conflicting
treatment. Two of these surfaces (Notifications and the block/mute controls) ride product
requirements the release bar does not gate; that staging is carried into their `REQ-UX` entries so it
stays explicit.

The notification **preferences** settings sub-section (which activity categories send a
notification) is part of the account-lifecycle surfaces, not this part; this part designs the
Notifications activity surface and cross-references those preferences where a person would reach for
them. The operator's moderation and support tooling is the operational-readiness area's; this part
designs only the **user-facing entry** to help and support, which is where a person's path into that
tooling begins.


---

#### Notifications

The activity surface — the person's own copy of what others have directed at them. It realises the
inventory's **Notifications** screen (`/notifications`, feed/list, signed-in) inside the signed-in
**NavShell**, whose Notifications entry carries the unseen-activity indicator this screen clears.

- **Layout.** The signed-in **NavShell** frames the screen; the Notifications destination is marked
  active. The main column is a **ListView** at the 600px reading measure, titled "Notifications". At
  the head of the list sits a single, quiet filter affordance (**SecondaryButton** group: All ·
  Replies · Follows · Likes) so a person can narrow a busy list without leaving the screen. Below it,
  the list of activity, newest first, in bounded pages with the ListView's load-older affordance.
- **Content.** Each item is the row kind this list needs (per **ListView**, which hosts the row the
  screen needs): an **Avatar** (`avatar.md`) of the acting person, a line in `type.body` naming the
  activity in chirp's voice — "Dana replied to your chirp", "Sam followed you", "Three people liked
  your chirp" — a short relative timestamp in `type.meta`, and, for a reply, an inline **ChirpCard**
  preview of the reply so the person can read and act on it in place. A follow row carries a
  **SecondaryButton** to follow back; a reply row carries the ChirpCard's own reply and like
  controls. The whole row is a target that opens the chirp in its conversation or the person's
  profile.
- **Primary action.** Read the activity and act on it in place — open a reply's conversation, follow
  back, or like in return — without leaving the list. Opening the screen is itself the act that marks
  the activity seen and clears the shell's unseen indicator.
- **Unseen and cleared states.** Arriving activity raises the NavShell's unseen indicator (that
  indicator is the shell's, cross-referenced here, not redesigned). Opening Notifications marks the
  shown items seen and clears the indicator; a newly-unseen item is marked in the list (a dot in
  `color.primary` with a "New" label, never colour alone) until it is seen.
- **Empty state.** No activity yet → the ListView's **EmptyState**: a calm illustration, "No activity
  yet", and a one-line explanation that replies, follows, and likes will show up here, with a link to
  Explore so the person has a next step rather than a blank column.
- **Error state.** A page fails to load → the ListView's **InlineError** with a retry in place of the
  missing items; a failure while loading older items shows at the load-more boundary and keeps the
  items already shown. An act-in-place failure (a follow-back that did not take) shows
  **InlineFeedback** at the control in the danger role and reverts.
- **Loading state.** First load shows **LoadingState** skeleton rows at the rows' real height, not a
  spinner on a blank screen; loading older items shows a bounded spinner at the boundary.

#### Report

The way a person reports a chirp or an account that breaks the rules, and is told the report was
received. It realises the inventory's **Report** dialog (modal/dialog, signed-in), opened from the
report action in a **ChirpCard**'s more-actions menu or from a profile's more-actions menu.

- **Layout.** A **Dialog** (modal variant) over the `color.bg.scrim`: `type.heading` title "Report
  this chirp" or "Report this account"; a short line naming what is being reported (the chirp's
  author and a one-line quote, or the handle); a single-choice list of reasons drawn from the
  **Rules & terms** so a report always points at a stated rule; an optional **TextField** (multi-line)
  for "Anything else we should know?"; and the action pair — a `color.primary` **PrimaryButton**
  "Submit report" and a **SecondaryButton** "Cancel".
- **Content.** The reasons are the rule categories named on Rules & terms, phrased plainly
  ("Harassment or targeting", "Spam", "Something else"). The dialog says, in `type.meta`, that the
  operator reviews reports and that blocking the account is available too (a link that opens the
  **Block / mute** confirmation), so reporting and stopping contact are offered together.
- **Primary action.** Submit the report. On success the dialog resolves to a receipt: a brief
  in-panel confirmation and a **Toast** — "Thanks — we've received your report" — so the person knows
  it landed. This acknowledgement is the gated behaviour (a report that is silently swallowed is not
  a report).
- **Empty state.** No reason chosen yet → "Submit report" is disabled (the **Dialog**'s disabled
  confirm) with a `type.meta` hint to choose a reason; the optional detail field shows its placeholder.
- **Error state.** The submit fails → an **InlineError** inside the panel in plain words, the dialog
  stays open, and the chosen reason and any typed detail are preserved so the person can retry. If the
  session has expired at submit, the product routes to Sign in with the reason stated and the report
  draft preserved, per the IA session-expiry convention (cross-referenced, not redesigned).
- **Loading state.** Submit in flight → the confirm button shows its busy state and the dialog does
  not close until the submission resolves; the reason and detail stay locked so the report is not
  sent twice.

#### Block / mute

The controls that stop unwanted interaction, each confirmed with its effect stated plainly before it
takes. It realises the inventory's **Block / mute** screen (confirmation, signed-in), opened from a
profile's or a ChirpCard's more-actions menu.

- **Layout.** A **Dialog** (confirmation variant) over the scrim: `type.heading` title ("Block
  @handle?" / "Mute @handle?"); `type.body` content stating exactly what the action does; and the
  action pair — a `color.danger` **PrimaryButton** ("Block" / "Mute") and a **SecondaryButton**
  "Cancel".
- **Content.** The effect is stated without euphemism. Block: "They won't be able to follow you, see
  your chirps while signed in, or reply to you, and you won't see them. They aren't told you blocked
  them." Mute: "You'll stop seeing their chirps and replies in your timeline and notifications. They
  can still see and reply to you, and they aren't told." A single line notes where to undo it — the
  **Blocked & muted** settings screen.
- **Primary action.** Confirm the block or the mute. On success the dialog closes and a **Toast**
  confirms it ("@handle blocked") with an inline undo; the person is returned to where they were.
- **Empty state.** Does not apply — a confirmation always opens with its stated consequence and its
  two choices; there is nothing for it to be empty of.
- **Error state.** The action fails → an **InlineError** inside the panel; the dialog stays open so
  the person can retry, and nothing has changed until it succeeds (the control never reports an effect
  that did not take).
- **Loading state.** Confirm in flight → the confirm button shows its busy state and the dialog holds
  open until the action resolves.

#### Blocked & muted

The settings screen where a person sees and manages the accounts they have blocked or muted. It
realises the inventory's **Blocked & muted** screen (`/settings/blocks`, settings, signed-in),
reached from the **Settings** hub (the hub itself is an account-lifecycle surface, cross-referenced
here) and framed by the signed-in **NavShell**.

- **Layout.** A settings screen at the reading measure with two labelled sections, "Blocked" and
  "Muted", each a list of **SettingRow**s. Each row carries the account's **Avatar** (`avatar.md`),
  display name and handle, and a **SecondaryButton** — "Unblock" or "Unmute" — in the row's control
  slot. A back affordance returns to the Settings hub.
- **Content.** One row per blocked or muted account, the two kinds kept visually separate so a person
  can tell at a glance what each account's status is. A row's meta line notes when the account was
  blocked or muted, in `type.meta`.
- **Primary action.** Unblock or unmute an account from its row. The change takes effect immediately
  and confirms on the row with **InlineFeedback**; an unblocked or unmuted account leaves its list.
- **Empty state.** No blocked or muted accounts → an **EmptyState** within the section ("You haven't
  blocked or muted anyone") that explains these lists fill as a person uses block and mute, rather than
  an empty row. Each section shows its own empty block so one being empty does not hide the other.
- **Error state.** The list fails to load → the section's **InlineError** with a retry; an unblock or
  unmute that fails → **InlineFeedback** in the danger role on that row with the prior status restored.
- **Loading state.** **LoadingState** skeleton rows while the lists resolve; a row saving an
  unblock/unmute shows a brief pending treatment on its control.

#### Help & support

The user-facing entry to getting help — recovering access, reporting a problem, and reaching the
operator. It realises the inventory's **Help & support** screen (`/help`, empty/landing, reachable in
both shells), rendered in whichever **NavShell** the person is in. This is the entry only; the
operator's support and moderation tooling is the operational-readiness area's.

- **Layout.** An **EmptyState**-style explanatory screen at the reading measure: a `type.title`
  heading "Help & support", a short `type.body` intro, and a small set of labelled entries, each a
  heading and a sentence with a link or action.
- **Content.** The entries are the things a person actually comes here for: "Can't get in?" linking to
  **Recover access** (an entry-and-authentication surface, cross-referenced) so recovery is reachable
  from help even when the sign-in cross-links are not shipped; "Report a problem" explaining how to
  report a chirp or account and linking to the **Rules & terms**; "Managing your account" linking to
  Settings; and "Contact support" — the user-facing way to reach the operator for something the
  self-service paths do not cover, which is where a person's path into the supported operator surface
  begins. A signed-out visitor sees the same page in the signed-out shell, with the recovery and
  contact entries foremost.
- **Primary action.** Reach the right help path — most often **Recover access** or **Contact
  support**; the screen leads with those so the common need is one step away.
- **Empty state.** Does not apply — this is an explanatory page whose content is always present; it is
  itself an empty/landing screen, so there is no data-empty variant to show.
- **Error state.** If the page's content is served and fails to load, the **EmptyState** error variant
  ("We couldn't load this — try again") with a retry; the shell and its recovery and contact entries
  remain reachable so a person is never stranded on a failed help page. A "Contact support" submission
  that fails shows an **InlineError** at the point of the action with the message preserved.
- **Loading state.** The persistent shell renders immediately; while the page body resolves it shows a
  brief **LoadingState**, not a blank panel.

#### Rules & terms

The stated rules — what is and is not allowed — that a report points at, together with the product's
terms. It realises the inventory's **Rules & terms** screen (`/rules`, empty/landing, reachable in
both shells), rendered in whichever **NavShell** the person is in.

- **Layout.** A reading-measure document screen: a `type.title` heading "Rules & terms", a short
  `type.body` preamble, then the rules as named, numbered sections in `type.heading` + `type.body` —
  the categories the **Report** dialog's reasons map to — followed by the terms.
- **Content.** Each rule is a plain statement of a behaviour that is not allowed (harassment and
  targeting, spam, impersonation, and the like) in chirp's voice, written so a person can tell whether
  a chirp broke it and so a report can point at it. The terms state the ordinary expectations of using
  the service. A line at the foot links to **Report** for acting on a rule and to **Help & support**.
- **Primary action.** Read and, where the person arrived to act, report a problem — a "Report a
  problem" link opens the **Report** dialog against the chirp or account they came from. For a
  signed-out visitor the shell's Sign up remains the prominent action.
- **Empty state.** Does not apply — the rules are always present content; an empty/landing screen has
  no data-empty variant.
- **Error state.** If the content fails to load, the **EmptyState** error variant with a retry, inside
  the persistent shell so the footer and navigation stay reachable.
- **Loading state.** The shell renders immediately; a brief **LoadingState** stands in for the body
  while it resolves.

#### Not found

The screen a person lands on when a resource is missing or removed — a mistyped URL, a deleted chirp,
a blocked or suspended account, a profile that no longer exists — which explains the absence and
routes back to a known place rather than dead-ending. It realises the inventory's **Not found** screen
(any unresolved path, empty/landing, both shells).

- **Layout.** An **EmptyState** at generous spacing inside whichever **NavShell** applies: a calm
  illustration, a `type.title` heading "This isn't here", a `type.body` line explaining that the page,
  chirp, or account could not be found or has been removed, and a **PrimaryButton** back to a known
  place.
- **Content.** The explanation states both cases plainly without exposing anything private: a
  resource that never existed or was mistyped reads "We couldn't find this page"; a resource that was
  removed or is unavailable (a deleted chirp, a blocked or suspended account) reads "This chirp isn't
  available" or "This account isn't available", matching the quiet tombstone a **ChirpCard** or
  **ProfileHeader** shows in place so the two never contradict each other.
- **Primary action.** Go to a known place — "Go to your timeline" for a signed-in person, "Go to
  chirp" (the Landing) for a signed-out one — so the screen is always a way back, never a wall. A
  secondary link reaches **Help & support**.
- **Empty state.** Does not apply — this screen is itself the treatment for an absent resource; it has
  no further empty variant.
- **Error state.** Does not apply as a distinct state — Not found is already the product's answer to an
  unresolved or removed resource; a load failure of a resource that does exist is that resource's own
  error state (a ListView InlineError, a ProfileHeader error), not this screen.
- **Loading state.** Does not apply — Not found is a resolved outcome, not a surface awaiting content;
  the shell renders it immediately. While a route is still resolving, the destination screen shows its
  own **LoadingState**, and Not found appears only once resolution has concluded the resource is
  absent, so a slow load is never mistaken for a missing resource.

---

## Empty, loading, and error states

A screen is not always showing content. Besides its working state, every surface in chirp spends
time in one of three other conditions: it has nothing to show (**empty**), it is fetching what it
will show (**loading**), or something it tried to do failed (**error**). One further condition cuts
across all three — the person's session ends, or a request fails for a moment, while they have work
in progress. Left to each screen to handle on its own, these conditions get improvised differently
from screen to screen, or skipped until a demo hits them. This section sets them once, as the
product-wide patterns every screen obeys, so a person meets the same treatment on the timeline, on a
profile, in settings, and inside a dialog.

The patterns here are not new components. They are expressed over the component library's named
states — `EmptyState`, `LoadingState`, `InlineError`, `InlineFeedback`, `Toast`, and the
`ListView` / `ChirpCard` / `ProfileHeader` / `Avatar` states — and over the information
architecture's conventions for session expiry and auth-gated return. The four surface-cluster
screen designs set each screen's own empty, error, and loading states by composing those component
states with the patterns below; they do not invent a per-screen treatment that conflicts with these.

One rule governs all four conditions: **a person can always tell which condition a surface is in,
why, and what to do next.** That signal is never carried by colour alone — it is carried by an
illustration and words, a skeleton shape, an icon and a message, or a plain sentence.

### Empty — explain and offer a next step, never a blank

A surface with nothing in it shows the `EmptyState` block, or the owning component's own *empty*
state, which is the same treatment: a calm illustration, one line explaining why the surface is
empty, and a next step the person can take. It is never a bare blank region, and never a spinner
that has already finished spinning on nothing.

Where this applies, by screen and component:

- A **brand-new home timeline** — `ListView` *empty* — explains that the timeline fills as the
  person follows people and posts, and points to **Explore** and to composing a first chirp.
- An **empty notifications** surface — `ListView` *empty* — says that activity about the person
  will appear here, rather than showing a blank panel.
- An **empty followers / following** list and an **Explore / find people** search with no results —
  `ListView` *empty* — each explain the absence and, for a no-results search, suggest broadening the
  query rather than implying failure.
- A **blocked & muted** sub-section with nothing in it shows an `EmptyState` inside the section, not
  an empty `SettingRow` — the same holds for any settings sub-section (for example sessions &
  devices) that can legitimately have no entries.
- A **sparse profile** — `ProfileHeader` *empty* — renders a missing bio and zero counts as calm,
  labelled zeros and a short prompt, not as blank space.
- A person with **no picture** — `Avatar` *empty* — shows their initials monogram, never a
  broken-image glyph.

An empty surface and a failed-to-load surface are never shown as the same thing. A genuinely empty
surface uses the *empty* treatment above; a surface that tried to load and failed uses the
`EmptyState` *error* variant described below, with a retry. "There is nothing here" and "we could
not load this" are different facts and lead to different next steps, so they read differently.

### Loading — show the shape of what is coming, and never spin forever

A **first load** of a surface shows `LoadingState` skeletons at the real dimensions of the content
on its way: skeleton `ChirpCard`s in a `ListView`, a skeleton `ProfileHeader`, a skeleton `Avatar`
circle at its target size. Holding the true size means the layout does not jump when real content
arrives, and the person sees the shape of the page forming rather than a spinner on a blank screen.

An **in-place or incremental load** shows a bounded indicator at the point of the action, not a
full-screen takeover:

- loading older items at a list boundary — a bounded spinner at the `ListView` load-more boundary,
  with the already-loaded items left in place;
- a running action — the `PrimaryButton` / `SecondaryButton` busy state, holding its label and its
  width so nothing shifts, and locking briefly so the action is not double-fired;
- a setting saving — the `Toggle` / `SettingRow` pending treatment on that row;
- an async field check (handle availability at sign-up) — the `TextField` pending indicator, the
  field not yet marked valid or invalid.

**Nothing loads forever.** A bounded spinner is the only element permitted to animate continuously,
per the motion rules; a load that does not resolve hands off to the owning component's *error* state
rather than spinning on. When the reader has asked their system to reduce motion, a skeleton is
static and the spinner remains bounded, so the loading state never relies on movement to be
understood.

The persistent `NavShell` does not participate in any of this: moving between screens never blanks
the frame. Only the destination's content region shows a loading state; the navigation around it
stays put.

### Error — shown where the action was taken, in plain words, with a way forward

A failure is reported at the point of the action, in plain language, with something the person can
do about it. It is never a silent no-op (an action that quietly does nothing), never a raw system
message or error code, and never phrased to blame the person.

The level of the error decides which component carries it:

- **Field level** — a `TextField` validation failure or a failed field check shows `InlineError`
  beneath the field, naming the problem and the fix ("That handle's already taken — try another"),
  tied to the field for assistive technology.
- **Control level, in place** — a like, follow, or setting change that fails shows `InlineFeedback`
  in the danger role at the control and reverts the optimistic change, so the control never shows a
  result that did not happen; any drafted reply next to it is kept.
- **List level** — a page that fails to load shows the `ListView` *error*: an `InlineError` with a
  retry in place of the missing items. A failure while loading *older* items shows at the load-more
  boundary and keeps the items already on screen.
- **Surface level** — a whole surface that fails to load (a profile, an entire list) shows the
  `EmptyState` *error* variant with a retry, which is how a failed load is told apart from a
  genuinely empty surface.
- **Task / dialog level** — a `Dialog` action that fails shows an `InlineError` inside the panel and
  leaves the dialog open with any draft preserved, so the person can retry without starting over.
- **Background, no on-screen control** — a failure with no single control to attach to shows a
  danger-role `Toast` carrying a word as well as the colour, never colour alone.

Every one of these leaves the person a way forward — retry, correct, or a clear way out — and
preserves whatever they had entered.

### Expired session and transient error — recover without losing work

This pattern composes against the information architecture's session-expiry convention (REQ-UX-008)
and its auth-gated return (REQ-UX-007). It does not redefine them; it states how the product's state
treatment carries a person through an expiry or a passing failure without costing them their work.

- **An expired or invalid session** routes the person to **Sign in** with the reason stated plainly
  — "Your session's expired — sign in to pick up where you left off" — rather than failing silently
  or showing a generic error. After signing back in, they return to where they were: the screen and
  the position they asked for, not a default landing screen.
- **The draft survives the round-trip.** A chirp or reply drafted in the `ComposeBox` is preserved
  across the interruption — the person is routed to sign-in and the draft is handed back to the
  `ComposeBox` when they return — so an expiry never costs them their words. This is the
  `ComposeBox`'s draft-preservation behaviour, held as a product-wide guarantee wherever a person can
  have in-progress input.
- **A transient error** — a server hiccup while posting, a dropped request — is retryable in place
  without losing in-progress work. The `ComposeBox` keeps the draft and shows an `InlineError` with
  a retry; a `Dialog` stays open with its content; a list keeps what it already displayed. The retry
  succeeds without the person re-entering anything.
- **The two are never shown as each other.** An expired session is an authentication condition —
  route to sign-in, explain, return. A transient error is a retry condition — stay on the screen,
  retry in place. A passing hiccup does not dump the person to sign-in, and an expiry is not dressed
  up as a retryable error that quietly keeps failing.

### How per-screen states compose against these patterns

The entry-and-auth, core-loop, account-lifecycle, and safety-and-notifications screen designs each
specify their own screens' empty, error, and loading states. They do so by composing the component
library's named states with the four patterns above, not by inventing a conflicting treatment. The
result is one product: a person reaching an empty list, a slow load, a failed action, or a lapsed
session meets the same designed behaviour on the home timeline, a profile, a settings sub-section,
and a dialog.

---

## Visual language and design system

chirp is a place where strangers read each other in a running feed. People decide in the first
minute whether it looks like a product they can trust with an account, and they then live in it
for minutes at a time, every day, scanning text. So the visual language has two jobs: look
settled and credible on first sight, and stay calm and legible enough to read for a long time
without fatigue. Everything below serves those two jobs.

Today the product is three screens in the browser's default styling — a system font, a single
column, one underlined blue for links. The release design keeps that restraint and gives it a
system: a type scale, a colour language with a light and a dark theme, a spacing grid, a small
set of radii and elevations, consistent icons, a considered way to show people (avatars), motion
that only ever has a reason, and a voice for the words on screen. The system is expressed as a
named **token set** (further below) so that every screen draws from the same small palette of
decisions, and a stranger moving from the timeline to a profile to settings feels one product
rather than three.

Three principles run through all of it:

- **Text first, chrome last.** The content is the chirps. Colour, borders, and shadow stay quiet
  so the writing carries. Accent colour is spent only where it does work — an action, a link, the
  current place in the nav.
- **One decision, applied everywhere.** There is one type scale, one spacing step, one way to
  show a destructive action. A screen does not invent its own sizes or colours; it references the
  tokens. That is what makes the many screens read as one product.
- **Legible for everyone, in any light.** The language meets an accessibility baseline (WCAG 2.1
  AA) as a floor, not a finish: contrast, visible focus, information never carried by colour
  alone, and motion that can be turned down. This holds in both the light and the dark theme.

### Typography

One typeface family does all the work. chirp uses the platform's native UI sans — the same
`system-ui` lineage the product already loads — because it renders instantly with no web-font
fetch (so the first paint is not a flash of invisible text), it is deeply legible at small sizes,
and it looks native on every device. There is no display or brand face; the restraint is the
brand.

**Family roles**

- **`type.family.sans`** — all UI and reading text. Realised as the native UI sans stack
  (`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`).
- **`type.family.mono`** — a monospaced stack, used only where characters must align or be read
  exactly (a recovery code, a session identifier in settings). Realised as the native monospace
  stack. Not used for chirp text, handles, or counts.

Numerals in counts (followers, likes, remaining characters) use tabular (fixed-width) figures so
a changing number does not shift the layout under the reader's eye.

**Type scale** — a six-step scale on a 16px base. Sizes are in px with the line-height that goes
with each step; line-height is looser for reading sizes and tighter for headings.

| Step | Size / line-height | Weight | Used for |
|---|---|---|---|
| `type.display` | 28 / 34 | 700 | The landing and onboarding headline — one per screen, rarely. |
| `type.title` | 20 / 28 | 700 | A screen's main heading; a profile's display name. |
| `type.heading` | 17 / 24 | 600 | Section headers, a card author's name, dialog titles. |
| `type.body` | 15 / 22 | 400 | Chirp text and primary reading — the default. |
| `type.meta` | 13 / 18 | 400 | Timestamps, secondary handles, labels under a count. |
| `type.caption` | 12 / 16 | 500 | Overlines, badges, the remaining-character counter. |

Body text never goes below `type.body` (15px); 15px at a 1.45 line-height is a comfortable
long-read size at a phone's viewing distance. Weights are drawn from a set of four — 400 regular,
500 medium, 600 semibold, 700 bold — and nothing else. Hierarchy is made with size and weight,
not with colour; colour is reserved for meaning.

### Colour and theming

Colour is used sparingly and by role. A screen does not ask for "blue" or "red"; it asks for
`color.primary` or `color.danger`, and the theme supplies the value. Because every colour is a
role, the whole product themes by swapping one set of values for another: there is a **light
theme** and a **dark theme**, and a screen is authored once against the roles and renders in both.

The brand colour is the chirp blue the product already uses, `#1D9BF0`. It is kept for brand
marks, selection, and the base of the focus ring, but it is deliberately *not* used as a text
colour or a button fill on a light surface, because at that lightness it does not reach the
contrast a stranger with ordinary eyesight needs. The action and link roles use a deeper blue
that does.

Every role below is given a light value, a dark value, and the contrast it must satisfy. The
contrast intents are the rule the product is built and checked against (see REQ-UX-022); the hex
values are the design intent a build realises.

**Surfaces**

| Role | Light | Dark | Intent |
|---|---|---|---|
| `color.bg.canvas` | `#FFFFFF` | `#16202A` | The app background. Dark is a soft navy, not pure black, so long reading is easier on the eye. |
| `color.bg.surface` | `#FFFFFF` | `#1C2732` | A card, menu, or dialog surface; separated from the canvas by a border and/or elevation. |
| `color.bg.subtle` | `#F2F5F7` | `#22303C` | Inset areas, hover fills, skeleton blocks, the compose draft area. |
| `color.bg.scrim` | `rgba(15,20,25,0.40)` | `rgba(0,0,0,0.60)` | The dim behind a modal/dialog, so the layer beneath recedes. |

**Text and on-surface**

| Role | Light | Dark | Intent |
|---|---|---|---|
| `color.text.primary` | `#0F1419` | `#E7E9EA` | Primary reading text; ≥ 7:1 on the canvas (comfortably past AA). |
| `color.text.secondary` | `#536471` | `#8B98A5` | Timestamps, meta, secondary handles; ≥ 4.5:1 on the canvas. |
| `color.text.disabled` | `#8899A6` | `#5B7083` | Disabled labels; not required to meet text contrast, but must read as inactive. |
| `color.text.link` | `#1472C4` | `#4DA3EB` | Links and inline actions; ≥ 4.5:1 on the canvas in its theme. |
| `color.text.on-primary` | `#FFFFFF` | `#FFFFFF` | Text/icon on a `color.primary` fill; ≥ 4.5:1 on that fill. |

**Border**

| Role | Light | Dark | Intent |
|---|---|---|---|
| `color.border.default` | `#CFD9DE` | `#38444D` | The hairline that separates chirps, cards, and inputs. |
| `color.border.strong` | `#AEB9C0` | `#536471` | A field's resting border, a divider that needs more presence. |
| `color.border.focus` | `#1472C4` | `#7CBAF0` | The focus ring; ≥ 3:1 against the adjacent surface in its theme. |

**Semantic roles** — each is a fill with a matching on-colour. Each fill is chosen so its
on-colour meets AA text contrast, and so the fill itself meets ≥ 3:1 against the canvas (so a
coloured control is perceivable as a control).

| Role | Light fill / on | Dark fill / on | Used for |
|---|---|---|---|
| `color.primary` | `#1565C0` / `#FFFFFF` | `#1A6DC2` / `#FFFFFF` | The primary action — Post, Follow, Save, Sign up. One per view where possible. |
| `color.success` | `#1A7F37` / `#FFFFFF` | `#2EA043` / `#FFFFFF` | A completed action's confirmation; a healthy state. |
| `color.warning` | `#B35C00` / `#FFFFFF` | `#E3922B` / `#15202B` | A caution that is not an error — approaching the character limit, an unverified contact. |
| `color.danger` | `#C8372D` / `#FFFFFF` | `#F4564B` / `#15202B` | Destructive actions and errors — delete, block, a failed submit, a field error. |
| `color.info` | `#1472C4` / `#FFFFFF` | `#4DA3EB` / `#15202B` | Neutral, informational banners and tips. |
| `color.like` | `#E8356D` / `#FFFFFF` | `#F91880` / `#FFFFFF` | The active "liked" state only; always paired with a filled heart and a "Liked" label, never colour alone. |
| `color.muted` | `#F2F5F7` / `#536471` | `#22303C` / `#8B98A5` | A quiet, non-primary chip or secondary button fill. |

Semantic colour never carries meaning by itself: an error is a red border *and* an inline
message; the active like is a colour *and* a filled icon *and* a label. The light and dark themes
are a parity pair — every role exists in both, and no screen is authored for only one.

### Spacing and the layout grid

Spacing is built on a **4px base unit**. Every margin, padding, and gap is a step on this scale;
there are no in-between values. Using one rhythm everywhere is most of what makes unrelated
screens feel related.

| Token | Value | Typical use |
|---|---|---|
| `space.0` | 0 | Reset. |
| `space.1` | 4px | Tight gaps inside a control (icon to label). |
| `space.2` | 8px | Gap between related lines; chip padding. |
| `space.3` | 12px | Compact padding; list-item inset. |
| `space.4` | 16px | The default: card/screen padding, the gap between chirps, mobile gutters. |
| `space.5` | 20px | Comfortable control padding. |
| `space.6` | 24px | Section spacing; gutters on wider viewports. |
| `space.8` | 32px | Separation between major blocks. |
| `space.10` | 40px | Generous vertical breaks on landing/onboarding. |
| `space.12` | 48px | Large empty-state and hero spacing. |

**Layout grid.** The reading column — the timeline, a conversation, a profile's chirps, a form —
is held to a **content measure of 600px** (`space.measure`). At the body size this keeps a line
of text near 66–75 characters, the band where lines are easiest to track. On a phone the content
fills the viewport with a `space.4` (16px) gutter on each edge; on wider viewports the content
column stays at the 600px measure, centred, with the navigation shell and any secondary rail
placed outside it. How the shell and rails reflow across viewports is the responsive area's to
decide; this language fixes the measure, the gutters, and the spacing steps they build from.

### Corner radii

A small set of radii, applied by the kind of thing being drawn, so curvature reads as a
deliberate signal and not noise.

| Token | Value | Applied to |
|---|---|---|
| `radius.none` | 0 | Full-bleed dividers, edge-to-edge media. |
| `radius.sm` | 4px | Text fields, small inputs, menu items. |
| `radius.md` | 8px | Cards, menus, dialogs, toasts. |
| `radius.lg` | 16px | Large containers and the onboarding/landing panels. |
| `radius.pill` | 9999px | Buttons and chips — chirp's buttons are fully rounded. |
| `radius.full` | 50% | Avatars — always a circle. |

### Elevation

Elevation separates layers that stack over one another. It is ordered: a thing that sits above
another has a higher elevation token, and the order is never skipped or inverted. In the light
theme, elevation is a soft shadow. In the dark theme shadows read poorly, so a raised surface is
separated mainly by a lighter surface colour and a border, with only a faint shadow — the token
name is the same, the realisation differs by theme.

| Token | Light (shadow) | Dark (treatment) | Used for |
|---|---|---|---|
| `elevation.0` | none | none | Canvas and inline cards, separated by borders only. |
| `elevation.1` | `0 1px 2px rgba(15,20,25,0.08)` | surface `#1C2732` + border | A hovered or pulled-forward card. |
| `elevation.2` | `0 4px 12px rgba(15,20,25,0.12)` | surface + border + `0 2px 6px rgba(0,0,0,0.4)` | Menus, popovers, dropdowns. |
| `elevation.3` | `0 8px 28px rgba(15,20,25,0.18)` | surface + border + `0 8px 28px rgba(0,0,0,0.5)` | Dialogs and modals (over a scrim). |
| `elevation.4` | `0 12px 32px rgba(15,20,25,0.22)` | surface + border + `0 12px 32px rgba(0,0,0,0.55)` | Toasts, which float above everything. |

### Iconography

One icon set, drawn as **outlines** on a 24px grid with a consistent ~2px stroke and rounded
joins, so the icons sit quietly beside the type. A **filled** variant of the same icon marks a
selected or active state — a filled home for the current nav destination, a filled heart for a
liked chirp. Icons take the current text colour by default (so they theme for free) and a
semantic role colour only to carry a state.

| Token | Value | Use |
|---|---|---|
| `icon.sm` | 16px | Inline with meta text. |
| `icon.md` | 20px | The default: buttons, list affordances. |
| `icon.lg` | 24px | Primary navigation, the compose affordance. |

Two rules hold everywhere: an icon-only control always has a text label available to assistive
technology and on hover; and an icon is never the only thing distinguishing a state — the active
nav item is filled *and* marked, not merely a different colour.

### Imagery and avatars

chirp is text-first and ships no media uploads in the release scope, so imagery is deliberately
limited to two things: people (avatars) and the product's own illustration on the
explanatory screens.

**Avatars** are always circular (`radius.full`), carry a 1px `color.border.default` to separate
them from the surface beneath, and come in a fixed set of sizes so a face is the same size
wherever it appears:

| Token | Value | Where |
|---|---|---|
| `avatar.sm` | 24px | Inline mentions, dense lists. |
| `avatar.md` | 40px | The feed and most cards — the default. |
| `avatar.lg` | 48px | A conversation's root author, settings. |
| `avatar.xl` | 96px | The profile header. |

When a person has no picture, the avatar shows their initials (a monogram) on a muted fill
derived deterministically from their handle — so the same person is always the same colour, and a
missing picture is never a broken-image glyph. **Illustration** appears only on the
empty/landing, onboarding, not-found, and empty-list states: a simple flat style in one or two
brand hues, calm and explanatory, never decorative inside the feed.

### Motion

Motion in chirp is functional — it orients (a menu grows from the control that opened it),
confirms (a toast slides in to mark a done action), and shows relationship (a dialog rises over a
dimming scrim). It is never decorative, never looping, and never blocks the reader. Nothing
animates indefinitely except a bounded loading indicator.

| Token | Value | Use |
|---|---|---|
| `motion.fast` | 120ms | Hover, press, toggles, small state flips. |
| `motion.base` | 200ms | Menus, toasts, and dialogs entering and leaving. |
| `motion.slow` | 320ms | Larger section or view transitions. |
| `motion.easing.standard` | `cubic-bezier(0.2, 0, 0, 1)` | The default for most moves. |
| `motion.easing.decelerate` | `cubic-bezier(0, 0, 0, 1)` | Things entering the screen. |
| `motion.easing.accelerate` | `cubic-bezier(0.3, 0, 1, 1)` | Things leaving the screen. |

`motion.reduced` is the contract for a reader who has asked their system to reduce motion: all
non-essential motion is dropped, a transition becomes either instant or a simple ≤120ms opacity
fade, counts do not animate, and nothing conveys required information through movement that has no
static equivalent. Honouring that preference is part of the accessibility baseline, not an extra.

### Tone and voice

The words on screen are part of the visual language — they are read in the same moment as the
type and colour. chirp's voice is plain, warm, and brief. It speaks to the reader as "you", in
the active voice, in **sentence case** throughout (headings, buttons, labels) because sentence
case is calmer and faster to read than Title Case. It names things consistently: a post is a
**chirp**, the actions are **post**, **reply**, **follow**/**unfollow**, and **like**.

The microcopy rules that keep the voice consistent:

- **Buttons are verbs for the action** — "Post", "Follow", "Save changes", "Delete chirp" — never
  "OK" or "Submit".
- **Errors say what happened and what to do**, in plain words, and never blame the reader or
  surface a system or error code: "That handle's already taken — try another", not "Error:
  invalid input". A failure appears where the action was taken.
- **Empty states explain and offer a next step** rather than apologising for being empty.
- **Destructive confirmations name the consequence plainly** and offer a clear way out; the
  account-deletion confirmation says what is and is not kept.
- **Timestamps are short and relative** ("2m", "4h", then an absolute date), and counts are
  spelled the same way everywhere.

### Design tokens

This is the named token set the rest of the product builds from. Each token has a name, a value
(for colour, a light-theme and a dark-theme value), and the intent a reviewer can check it
against. Every component and screen refers to these tokens **by name**; a screen that needs a
colour, a size, a space, a radius, a duration, or an elevation uses the matching token and never
an ad-hoc value. The families are `color.*`, `type.*`, `space.*`, `radius.*`, `elevation.*`, and
`motion.*`.

**`color.*`** — role, light value, dark value, contrast intent. (Values and intents are the
tables under *Colour and theming* above; collected here as the token list.)

| Token | Light | Dark | Contrast intent |
|---|---|---|---|
| `color.brand` | `#1D9BF0` | `#1D9BF0` | Brand mark, selection, focus-ring base; not a text/fill colour on light. |
| `color.bg.canvas` | `#FFFFFF` | `#16202A` | The app background. |
| `color.bg.surface` | `#FFFFFF` | `#1C2732` | Card/menu/dialog surface. |
| `color.bg.subtle` | `#F2F5F7` | `#22303C` | Insets, hovers, skeletons. |
| `color.bg.scrim` | `rgba(15,20,25,0.40)` | `rgba(0,0,0,0.60)` | Dim behind a modal. |
| `color.text.primary` | `#0F1419` | `#E7E9EA` | ≥ 7:1 on canvas. |
| `color.text.secondary` | `#536471` | `#8B98A5` | ≥ 4.5:1 on canvas. |
| `color.text.disabled` | `#8899A6` | `#5B7083` | Reads as inactive. |
| `color.text.link` | `#1472C4` | `#4DA3EB` | ≥ 4.5:1 on canvas. |
| `color.text.on-primary` | `#FFFFFF` | `#FFFFFF` | ≥ 4.5:1 on `color.primary`. |
| `color.border.default` | `#CFD9DE` | `#38444D` | Hairline separators. |
| `color.border.strong` | `#AEB9C0` | `#536471` | Field/divider border. |
| `color.border.focus` | `#1472C4` | `#7CBAF0` | ≥ 3:1 against the adjacent surface. |
| `color.primary` | `#1565C0` | `#1A6DC2` | White on-colour ≥ 4.5:1; fill ≥ 3:1 on canvas. |
| `color.success` | `#1A7F37` | `#2EA043` | On-colour meets AA; fill ≥ 3:1. |
| `color.warning` | `#B35C00` | `#E3922B` | On-colour meets AA; fill ≥ 3:1. |
| `color.danger` | `#C8372D` | `#F4564B` | On-colour meets AA; fill ≥ 3:1. |
| `color.info` | `#1472C4` | `#4DA3EB` | On-colour meets AA; fill ≥ 3:1. |
| `color.like` | `#E8356D` | `#F91880` | Active-like only; paired with icon + label. |
| `color.muted` | `#F2F5F7` | `#22303C` | Quiet chip/secondary fill; its on-colour ≥ 4.5:1. |

**`type.*`**

| Token | Value |
|---|---|
| `type.family.sans` | Native UI sans stack (`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif`). |
| `type.family.mono` | Native monospace stack; codes and identifiers only. |
| `type.display` | 28px / 34px, weight 700. |
| `type.title` | 20px / 28px, weight 700. |
| `type.heading` | 17px / 24px, weight 600. |
| `type.body` | 15px / 22px, weight 400. |
| `type.meta` | 13px / 18px, weight 400. |
| `type.caption` | 12px / 16px, weight 500. |
| `type.weight.regular` / `.medium` / `.semibold` / `.bold` | 400 / 500 / 600 / 700. |
| `type.numeral.tabular` | Fixed-width figures for counts. |

**`space.*`** — base unit 4px.

| Token | Value |
|---|---|
| `space.0` … `space.6` | 0, 4, 8, 12, 16, 20, 24px. |
| `space.8` / `space.10` / `space.12` | 32 / 40 / 48px. |
| `space.measure` | 600px — the reading content column's max width. |
| `space.gutter.mobile` / `space.gutter.wide` | 16px / 24px. |

**`radius.*`**

| Token | Value |
|---|---|
| `radius.none` / `radius.sm` / `radius.md` / `radius.lg` | 0 / 4 / 8 / 16px. |
| `radius.pill` | 9999px (buttons, chips). |
| `radius.full` | 50% (avatars). |

**`elevation.*`** — see the *Elevation* table for the per-theme realisation.

| Token | Role |
|---|---|
| `elevation.0` | Flat; borders only. |
| `elevation.1` | Raised/hovered card. |
| `elevation.2` | Menu / popover. |
| `elevation.3` | Dialog / modal. |
| `elevation.4` | Toast. |

**`motion.*`**

| Token | Value |
|---|---|
| `motion.fast` / `motion.base` / `motion.slow` | 120 / 200 / 320ms. |
| `motion.easing.standard` | `cubic-bezier(0.2, 0, 0, 1)`. |
| `motion.easing.decelerate` | `cubic-bezier(0, 0, 0, 1)`. |
| `motion.easing.accelerate` | `cubic-bezier(0.3, 0, 1, 1)`. |
| `motion.reduced` | Drop non-essential motion; ≤120ms opacity fade or instant; no motion-only information. |

**Avatar and icon sizes** are part of the system too: `icon.sm/md/lg` = 16/20/24px;
`avatar.sm/md/lg/xl` = 24/40/48/96px. They are referenced by name like every other token.

### Consistency rules

These are the rules that make two chirp screens feel like one product. They govern how the tokens
above are applied; the component library and the per-screen designs are built to obey them.

1. **No ad-hoc values.** Every colour, type size, space, radius, duration, and elevation on any
   screen is a reference to a token above. A screen never introduces a raw hex, a one-off pixel
   size, or a bespoke duration.
2. **Colour is requested by role, not by hue.** A destructive action asks for `color.danger`; a
   primary action asks for `color.primary`. No screen picks "a red" or "a blue" of its own.
3. **Theming is a token swap, nothing more.** A screen is authored once against the semantic
   roles and renders correctly in both the light and the dark theme. No screen hard-codes a theme
   or a theme-specific colour.
4. **One type scale and one spacing rhythm, everywhere.** Headings, body, and meta come from the
   scale; vertical spacing comes from the `space.*` steps. The 600px reading measure is the same
   across the feed, a conversation, a profile, a form, and settings.
5. **Elevation is ordered and never inverted.** A card is at or below a menu, a menu below a
   dialog, a dialog below a toast. A modal always sits over the `color.bg.scrim`.
6. **Focus is always visible.** Every interactive element shows the `color.border.focus` ring on
   keyboard focus; the ring is never removed, only restyled within the token.
7. **Meaning is never carried by colour alone.** A state that uses a semantic colour also uses an
   icon, a label, or text — the error border has a message, the active like has a filled icon and
   a label.
8. **Motion uses the motion tokens and honours reduced-motion.** No screen defines its own timing
   or easing; every screen degrades to `motion.reduced` when the reader has asked for it.
9. **The voice rules apply to every string.** Sentence case, verbs on buttons, plain and
   actionable errors with no system codes, explanatory empty states.

### Component library

The component library is the set of named, reusable pieces every chirp screen is built from. It is
why the many screens read as one product and not a dozen separately-styled pages: a chirp looks the
same in the timeline, a conversation, and a profile because all three draw the same **ChirpCard**;
a form error reads the same on sign-up and in settings because both use the same **TextField** and
**InlineError**. It is also where the unglamorous states live. Empty, loading, and error are not
left to each screen to improvise — they are built into the components, so every list that uses
**ListView** inherits a designed empty block, a loading skeleton, and a load-failure retry without
re-inventing them.

Every component below names its purpose, the design tokens it is built from (by name — no component
introduces a raw colour, size, space, radius, duration, or elevation), its behavior, and its states.
The siblings that come after this one — the per-screen flows, the responsive layout, and the
accessibility work — refer to these components **by the names in bold**.

**How states are read.** Each component records which of the five shared states apply to it —
**default**, **empty**, **error**, **loading**, **disabled** — and says plainly where a state does
not apply. "Does not apply" is a design decision, not an omission: a **PrimaryButton** has no *empty*
state because a button is never empty of content, and the error of the action it triggers is shown
by a feedback component next to it, not on the button itself. Two cross-cutting rules from the visual
language hold for every component here: meaning is never carried by colour alone (an error is a
`color.danger` border *and* a message; an active state is a colour *and* an icon or label), and every
interactive component shows the `color.border.focus` ring on keyboard focus.

---

#### Identity and content

##### Avatar
- **Purpose**: Shows a person wherever they appear — in a **ChirpCard**, a **ProfileHeader**, the
  nav, and dense lists — so the same face is recognisable at a glance across the product.
- **Built from**: `radius.full` (always a circle); `color.border.default` (the 1px ring that lifts
  it off the surface); `avatar.sm` / `avatar.md` / `avatar.lg` / `avatar.xl` for size by context;
  `color.bg.subtle` + `color.text.secondary` + `type.heading` for the initials monogram; the
  monogram fill is derived deterministically from the handle so a person is always the same colour.
- **Behavior**: Renders at the size the context asks for. When it stands in for a link to a profile,
  the whole avatar is the target and it carries the focusable control's focus ring; on its own it is
  decorative and exposes the person's name to assistive technology.
- **States**:
  - **default** — the person's picture, circular, with the hairline border.
  - **empty** — no picture: a readable initials monogram on the deterministic fill, never a blank
    circle.
  - **error** — the picture fails to load: falls back to the same monogram as *empty*, never a
    broken-image glyph.
  - **loading** — a `color.bg.subtle` skeleton circle at the target size while the picture resolves.
  - **disabled** — does not apply (an avatar is not a control; when used as a link it takes the
    link's disabled treatment).

##### ChirpCard
- **Purpose**: One chirp, rendered the same way everywhere it appears — the timeline, a conversation,
  search and notification results, and a profile's chirps. The single most-repeated object in the
  product.
- **Built from**: `type.body` (the chirp text), `type.heading` (author display name), `type.meta`
  + `color.text.secondary` (handle and relative timestamp); `Avatar` at `avatar.md`; `space.4`
  padding and `space.2` internal gaps; `color.border.default` hairline between cards; `color.bg.surface`;
  `IconButton`s for reply, like, and a more-actions menu, laid out with `space.3` gaps.
- **Behavior**: The whole card is a target that opens the chirp in its conversation; the action
  controls inside it act in place without leaving the current screen. The like control uses
  `color.like` with a filled heart and a "Liked" label together (never colour alone). Timestamps are
  short and relative (`2m`, `4h`, then an absolute date). The author's own chirp exposes delete in the
  more-actions menu, which opens the **Dialog** confirmation variant.
- **States**:
  - **default** — avatar, author, handle, timestamp, chirp text, and the action row.
  - **empty** — does not apply (a card always carries a chirp; an empty *list* is the **ListView**'s
    *empty* state, not the card's).
  - **error** — two cases: an in-place action fails (the like or follow) → the control shows
    **InlineFeedback** in the danger role and reverts, draft of any reply preserved; or the chirp is
    gone (deleted, blocked, or unavailable) → a quiet tombstone row ("This chirp isn't available")
    rather than an empty gap.
  - **loading** — a skeleton card (`LoadingState`): a `color.bg.subtle` avatar circle, two
    text lines, and a short meta line, at the card's real dimensions so the list does not jump when
    content arrives.
  - **disabled** — does not apply (the card is not itself a single control; its individual actions
    carry their own disabled state).

##### ProfileHeader
- **Purpose**: The identity block at the top of a **profile** screen — the person's public face.
  Realises the profile archetype together with a **ListView** of their chirps below.
- **Built from**: `Avatar` at `avatar.xl`; `type.title` (display name), `type.body`
  (`color.text.secondary` handle and bio); `type.meta` + `type.numeral.tabular` for the follower and
  following counts so a changing count does not shift the layout; a `PrimaryButton` ("Follow") or
  `SecondaryButton` ("Following", "Edit profile" on one's own profile); `space.6` block spacing;
  `color.bg.canvas`.
- **Behavior**: Shows handle, display name, bio, and follower/following counts (the public profile
  needs no session). The primary action is context-dependent: Follow/Following for another account,
  Edit profile for one's own. The follow action is idempotent and acts in place.
- **States**:
  - **default** — full identity with picture, bio, counts, and the context action.
  - **empty** — a sparse profile: no bio and zero counts render as calm, labelled zeros and a
    one-line prompt rather than blank space; no picture uses the `Avatar` *empty* monogram.
  - **error** — the profile fails to load → an **EmptyState** error variant with retry; a
    blocked/suspended/not-found account states that plainly rather than showing a broken header.
  - **loading** — a skeleton header: skeleton avatar, name line, bio line, and count line.
  - **disabled** — does not apply (the header is a surface; its one action button carries its own
    disabled state).

---

#### Input and composition

##### ComposeBox
- **Purpose**: Write and publish a chirp, or a reply to a named chirp. Used inline at the top of the
  timeline, as the body of the **Compose** dialog, and in a conversation's reply position.
- **Built from**: `type.body` for the draft text; `color.bg.subtle` draft area with `radius.md`;
  `space.4` padding; a remaining-character affordance in `type.caption` + `type.numeral.tabular`
  that moves to `color.warning` as the limit nears and `color.danger` when exceeded; a `PrimaryButton`
  ("Post" / "Reply"); `Avatar` at `avatar.md` for the composing person.
- **Behavior**: Enforces the published 280-character limit with a visible remaining-character count.
  The Post button is disabled while the draft is empty or over the limit; an empty or over-limit
  submit is rejected and nothing is posted. A drafted chirp is preserved across an interruption (an
  expired session routes to sign-in and returns the draft). The counter and the limit are the
  component's contract; the exact limit value comes from the product requirement, not from this
  component.
- **States**:
  - **default** — resting with a drafted-or-empty field and the live counter.
  - **empty** — no text: a placeholder prompt ("What's happening?"), counter at full, Post disabled.
  - **error** — over the limit (counter in `color.danger`, Post disabled) or a failed publish (an
    **InlineError** below the field in plain words, the draft kept so the person can retry).
  - **loading** — publishing in flight: Post shows its busy state and the field locks briefly so the
    chirp is not double-posted.
  - **disabled** — the whole box is unavailable (for example, composing is not permitted in the
    current state); the field and Post read as inactive.

##### TextField
- **Purpose**: One-line and multi-line text input with a label, optional hint, and inline,
  field-level validation. The building block of every **form** screen — sign-up, sign-in, recovery,
  edit profile.
- **Built from**: `type.meta` label; `type.body` input text; `radius.sm`; `space.3` inner padding;
  `color.border.strong` resting border, `color.border.focus` ring on focus, `color.danger` border in
  error; `color.text.secondary` hint; `type.family.mono` only where a value must be read exactly (a
  recovery code).
- **Behavior**: Validates at the field level and reports against the field, not in a summary far from
  the input. Messages say what to do in plain words and never surface a system or error code. A field
  whose validity needs a server check (handle availability and format at sign-up) shows a pending
  affordance while the check runs and resolves to a clear accepted or field-level error.
- **States**:
  - **default** — labelled, with the resting border, holding valid or in-progress input.
  - **empty** — label and placeholder/hint visible, no value yet; a required-but-empty field is not
    shown as an error until the person has had a chance to fill it.
  - **error** — `color.danger` border and an **InlineError** beneath, naming the problem and the fix
    ("That handle's already taken — try another").
  - **loading** — an async check is pending (for example handle uniqueness): a quiet pending
    indicator, the field not yet marked valid or invalid.
  - **disabled** — reads as inactive (`color.text.disabled`), not focusable, not submitted.

##### Toggle
- **Purpose**: An immediate on/off choice — notification categories, and other settings that take
  effect on change rather than on a separate Save. Used in **SettingRow**s.
- **Built from**: `color.primary` on-fill and `color.muted` off-fill; `color.text.on-primary`;
  `radius.pill`; `motion.fast` + `motion.easing.standard` for the thumb move (dropped under
  `motion.reduced`); a visible on/off label so state is never colour-or-position alone.
- **Behavior**: Flips immediately and persists the change, confirming with **InlineFeedback** on its
  row. If the save fails it reverts to the prior value and shows the failure, so the control never
  lies about the stored state.
- **States**:
  - **default** — on or off, with its label.
  - **empty** — does not apply (a toggle is always on or off).
  - **error** — the save failed → reverts and shows **InlineFeedback** in the danger role.
  - **loading** — saving: a brief pending treatment while the change is written.
  - **disabled** — a choice not currently available (for example a dependent preference), shown
    inactive with its reason where one is useful.

---

#### Actions

##### PrimaryButton
- **Purpose**: The one most-important action on a view — Post, Follow, Save changes, Sign up, Delete
  (in a confirmation). One per view where possible, so the eye knows where the main action is.
- **Built from**: `color.primary` fill with `color.text.on-primary`; `radius.pill` (chirp's buttons
  are fully rounded); `type.body` `type.weight.semibold`; `space.3`/`space.5` padding;
  `color.border.focus` ring; `motion.fast` press. A destructive primary (Delete) uses `color.danger`
  in place of `color.primary`.
- **Behavior**: The label is a verb for the action in sentence case ("Save changes", "Delete chirp"),
  never "OK" or "Submit". While its action runs it shows a busy state in place, keeping its label and
  holding its width so the layout does not jump.
- **States**:
  - **default** — the resting filled button.
  - **empty** — does not apply (a button is never empty of content).
  - **error** — does not apply on the button itself; the action's failure is shown by **InlineError**
    or **InlineFeedback** at the point of action.
  - **loading** — busy/in-flight: a bounded spinner replaces or accompanies the label, the button is
    not re-triggerable, width is stable.
  - **disabled** — the action is not currently available (incomplete form, empty draft); the fill
    reads as inactive and the button is not focusable-to-activate.

##### SecondaryButton
- **Purpose**: A secondary or non-destructive alternative action — Cancel, Following (the unfollow
  affordance), secondary links styled as buttons. Lower weight than **PrimaryButton** so it never
  competes with the main action.
- **Built from**: `color.muted` fill (or a bordered-transparent variant on `color.border.strong`)
  with `color.text.primary`; `radius.pill`; `type.body` `type.weight.medium`; `space.3`/`space.5`
  padding; `color.border.focus` ring.
- **Behavior**: Same label and busy conventions as **PrimaryButton**. Paired with a PrimaryButton in
  a dialog or form, it is the calm way out (Cancel) and is never the destructive one.
- **States**:
  - **default** — the resting secondary button.
  - **empty** — does not apply.
  - **error** — does not apply on the button itself (see **PrimaryButton**).
  - **loading** — busy/in-flight, as **PrimaryButton**.
  - **disabled** — inactive treatment when unavailable.

##### IconButton
- **Purpose**: An icon-only control where space is tight and the action is well-known — reply, like,
  and more-actions on a **ChirpCard**, the compose affordance, the nav destinations, a dialog's
  close.
- **Built from**: `icon.md` (or `icon.lg` for the primary nav and compose affordance) from the one
  outline icon set, filled variant for an active/selected state; current text colour by default and a
  semantic role colour only to carry a state (`color.like` for an active like); `space.2` hit-area
  padding to a comfortable target; `radius.pill` hover/press fill in `color.bg.subtle`;
  `color.border.focus` ring.
- **Behavior**: Always exposes a text label to assistive technology and on hover — an icon is never
  the only thing a person or a screen reader has to go on. An active/selected state is shown by the
  filled icon *and* a mark or label, not by colour alone.
- **States**:
  - **default** — the outline icon at rest; plus an **active/selected** treatment (filled icon + mark)
    for the current nav destination or a liked chirp.
  - **empty** — does not apply.
  - **error** — does not apply on the control; the action's failure shows via **InlineFeedback** at
    the control.
  - **loading** — busy/in-flight: a bounded spinner in place of the icon for the running action.
  - **disabled** — inactive icon treatment, not activatable.

---

#### Structure and navigation

##### NavShell
- **Purpose**: The persistent frame a person lives in. Two shells: the **signed-out shell** a visitor
  meets, and the **signed-in shell** a member uses. The transition between them is the
  sign-up / sign-in / sign-out boundary.
- **Built from**: the chirp wordmark in `type.title`; `IconButton`s + `type.meta` labels for
  destinations; `Avatar` at `avatar.sm` in the account entry; `color.bg.canvas` / `color.bg.surface`;
  `color.border.default` edge; `elevation.0` (it is part of the frame, separated by a border); the
  active destination marked with a filled `IconButton` + `color.primary` accent *and* a mark.
- **Behavior**:
  - *Signed-out shell*: carries the wordmark (to the Landing), a clear **Sign in** and a prominent
    **Sign up**, and footer access to **Rules & terms** and **Help & support** — on every signed-out
    screen, so a visitor on a shared link never dead-ends at an auth wall.
  - *Signed-in shell*: exposes **Home**, **Explore**, **Notifications** (with an unseen-activity
    indicator), **Profile**, an always-available **Compose** affordance, and an account menu reaching
    **Settings**, **Help & support**, **Rules & terms**, and **Sign out** — reachable from every
    signed-in screen.
  - The active destination is always marked; how the shell reflows between a phone and a wider
    viewport is the responsive area's decision and how it is operated and announced is the
    accessibility area's — this component fixes the destinations and the markings, not the layout.
- **States**:
  - **default** — the shell with its destinations and the active one marked; the signed-in shell may
    show the Notifications unseen-activity indicator.
  - **empty** — does not apply (the shell always has its fixed destinations).
  - **error** — does not apply to the frame; a destination's screen holds its own error.
  - **loading** — does not apply; the shell is persistent and renders immediately, so moving between
    screens never blanks the frame.
  - **disabled** — does not apply (individual entries unavailable in a state read as inactive, but
    the shell is always present).

##### ListView
- **Purpose**: The paginated container behind every **feed/list** screen — the home timeline, a
  conversation's replies, notifications, search results, and followers/following. Hosts **ChirpCard**s
  (or the row kind the screen needs) and owns the list-level empty, loading, and error treatments.
- **Built from**: `space.measure` (the 600px reading column); `space.4` gutters on a phone;
  `color.border.default` separators; `space.4` inter-item gap; `SecondaryButton` or a text control
  for "load older"; a catch-up control ("N new chirps") in `color.primary`; **LoadingState**,
  **EmptyState**, and **InlineError** for its own states.
- **Behavior**: Loads in bounded pages, never one unbounded load. Offers an explicit way to load
  older items, and the timeline offers a way to pull in chirps posted since the person last looked.
  Keeps the person's scroll place when they return from drilling into an item.
- **States**:
  - **default** — a bounded page of items with the load-older (and, for the timeline, catch-up)
    affordances.
  - **empty** — the list has no items → an **EmptyState** block explaining why and offering a next
    step (a brand-new timeline points to Explore and compose), never a blank area.
  - **error** — a page fails to load → an **InlineError** with a retry in place of the missing items;
    a failure while loading *older* items shows at the load-more boundary without discarding the items
    already shown.
  - **loading** — first load shows **LoadingState** skeleton rows (not a spinner on a blank screen);
    loading older items shows a bounded spinner at the boundary.
  - **disabled** — does not apply.

##### SettingRow
- **Purpose**: One labelled choice within a **settings** screen — a value to view or change, a
  **Toggle**, or a link into a sub-section. The repeated unit that realises the settings archetype
  across the settings hub and its sub-sections (edit profile, account & contact, notification
  preferences, sessions & devices, blocked & muted, export, delete).
- **Built from**: `type.body` label and `type.meta` + `color.text.secondary` description; a control
  slot holding a `Toggle`, a `SecondaryButton`, a value, or a chevron **IconButton** for a drill-in;
  `space.4` row padding; `color.border.default` separators; `color.danger` label for a destructive
  row (Delete account), which opens the **Dialog** confirmation variant.
- **Behavior**: Each row is independently operable and, where it drills in, is an addressable
  sub-section a person can open directly and return from to the hub. A change that takes effect
  immediately confirms on the row with **InlineFeedback**; a destructive row routes through a
  confirmation.
- **States**:
  - **default** — label, description, and its control or value.
  - **empty** — does not apply at the row level; a sub-section with nothing in it (no blocked
    accounts, no other sessions) shows an **EmptyState** within the section, not an empty row.
  - **error** — a change made on the row failed → **InlineFeedback** in the danger role on that row,
    prior value restored.
  - **loading** — the change is saving → a brief pending treatment on the row's control.
  - **disabled** — a choice not available in the current state, shown inactive with a reason where
    one helps.

---

#### Overlays

##### Dialog
- **Purpose**: A focused task or decision layered over the current screen without leaving it. Two
  variants cover two archetypes: a **modal/dialog** (Compose, Report) and a **confirmation** (delete
  account, delete chirp, block/mute, end a session, sign out of all sessions).
- **Built from**: `color.bg.surface` panel at `radius.md` and `elevation.3`, over the
  `color.bg.scrim`; `type.heading` title; `type.body` content; a `PrimaryButton` + `SecondaryButton`
  action pair (the destructive confirm uses the `color.danger` primary, Cancel is the secondary);
  a close `IconButton`; `motion.base` rise-over-scrim (dropped to a fade under `motion.reduced`);
  `space.6` padding.
- **Behavior**: Traps focus while open and returns focus to the control that opened it on close. The
  browser back control dismisses it (closing the dialog, not leaving the underlying screen). The
  *confirmation* variant names the consequence plainly and always offers a clear way out; the
  account-deletion confirmation additionally states what is and is not kept and points to data export
  first. The *modal* variant hosts a task component such as **ComposeBox**.
- **States**:
  - **default** — the panel open over the dimmed screen, title and action pair visible.
  - **empty** — does not apply (a dialog always opens with its content).
  - **error** — the dialog's action failed → an **InlineError** inside the panel; the dialog stays
    open with any draft preserved so the person can retry.
  - **loading** — the confirm action is running → the confirm button shows its busy state; the dialog
    does not close until the action resolves.
  - **disabled** — the confirm is disabled until a required input or acknowledgement is given.

---

#### Feedback and status

##### Toast
- **Purpose**: Brief, non-blocking confirmation that an action completed — a chirp published, a
  follow, a saved setting, a received report — shown without interrupting what the person is doing.
- **Built from**: `color.bg.surface` at `radius.md` and `elevation.4` (it floats above everything);
  `type.body` message; an optional inline undo/dismiss as a text control; `motion.base` slide-in and
  auto-dismiss (a fade under `motion.reduced`); `color.success` / `color.info` / `color.danger`
  accent by kind, always paired with a word, never colour alone.
- **Behavior**: Appears, is readable without stealing focus, and dismisses itself after a bounded time
  (or on an explicit dismiss). Used for results that do not belong against a specific control; a
  failure tied to a specific control uses **InlineFeedback** at the control instead.
- **States**:
  - **default** — a success or neutral confirmation.
  - **empty** — does not apply.
  - **error** — a danger-accented variant for a background failure that has no single on-screen
    control to attach to.
  - **loading** — does not apply (a toast marks a resolved result, not an in-flight one).
  - **disabled** — does not apply.

##### InlineFeedback
- **Purpose**: The result of an in-place action shown against the control that triggered it — a like
  that took, a follow that failed — so the feedback is where the person's attention already is.
- **Built from**: `type.meta`; `color.success` or `color.danger` with an icon and a word (never
  colour alone); `space.2` gap from its control; `motion.fast`.
- **Behavior**: Appears next to the control, confirms or reports the result, and clears on its own.
  On failure it accompanies a revert of the optimistic change, so the control never shows a result
  that did not happen.
- **States**:
  - **default** — a success/neutral in-place confirmation.
  - **empty** — does not apply.
  - **error** — a failure against the control in the danger role, with a plain reason and the state
    reverted.
  - **loading** — does not apply (it reports a resolved action; the in-flight state belongs to the
    control's own *loading*).
  - **disabled** — does not apply.

##### InlineError
- **Purpose**: The field- or control-level error message — the plain-words explanation of what went
  wrong and what to do, shown at the point of the problem. The error carrier for **TextField**,
  **ComposeBox**, **Dialog**, and **ListView**.
- **Built from**: `type.meta` in `color.danger`; a danger icon beside the text (so the error is not
  colour alone); `space.1`/`space.2` gap below the field or control it belongs to.
- **Behavior**: Names what happened and what to do in plain language, never a system or error code,
  and never blames the reader ("That handle's already taken — try another"). It appears where the
  action was taken and is associated with its field for assistive technology.
- **States**:
  - **default** — the error message shown; this component *is* the error treatment, so its resting
    state is the error being presented.
  - **empty** — does not apply (absent when there is no error).
  - **error** — this component is itself the error presentation; there is no second error-of-the-error.
  - **loading** — does not apply.
  - **disabled** — does not apply.

---

#### State blocks

##### EmptyState
- **Purpose**: The designed treatment for a surface with nothing in it — a brand-new timeline, an
  empty notifications list, no blocked accounts, a not-found page, and the explanatory landing and
  onboarding screens. Realises the **empty/landing** archetype and supplies the *empty* state other
  components (notably **ListView**) compose in.
- **Built from**: a simple flat illustration in one or two brand hues (calm and explanatory, never
  decorative in the feed); `type.title` headline; `type.body` `color.text.secondary` explanation; a
  `PrimaryButton` or link for the next step; `space.10`/`space.12` generous vertical spacing;
  `color.bg.canvas`.
- **Behavior**: Explains why the surface is empty and offers a next step rather than apologising for
  being empty — a new timeline points to Explore and composing a first chirp; onboarding explains the
  empty timeline and offers at least one route to a non-empty one. It is never a blank area.
- **States**:
  - **default** — the explanatory empty block with its illustration and next step.
  - **empty** — this component *is* the empty treatment; the empty surface is its purpose.
  - **error** — an error-flavoured variant: "We couldn't load this — try again" with a retry, which
    is how a failed load differs from a genuinely-empty surface so the two are never confused.
  - **loading** — does not apply (a surface that is loading shows **LoadingState**, not EmptyState).
  - **disabled** — does not apply.

##### LoadingState
- **Purpose**: The designed treatment for content that is on its way — a skeleton that holds the
  shape of what is coming, and a bounded spinner for smaller in-place loads. Supplies the *loading*
  state other components compose in (**ListView**, **ChirpCard**, **ProfileHeader**, **Avatar**).
- **Built from**: `color.bg.subtle` skeleton blocks at the real dimensions of the content, with a
  subtle `motion.base` shimmer that honours `motion.reduced` (a static block when motion is reduced);
  `radius.sm`/`radius.md`/`radius.full` matching the shapes being stood in for; a bounded spinner
  (the only element permitted to animate continuously, per the motion rules).
- **Behavior**: Stands in for content at its true size so the layout does not jump when real content
  arrives. A first load uses skeletons rather than a spinner on a blank screen; a small or in-place
  load uses a bounded spinner. Nothing loads forever — a load that fails hands off to the owning
  component's *error* state.
- **States**:
  - **default** — the skeleton (or bounded spinner) shown while content resolves; this component *is*
    the loading treatment.
  - **empty** — does not apply.
  - **error** — does not apply here; a failed load becomes the owning component's *error* state
    (**ListView** error, **InlineError**, or an **EmptyState** error variant).
  - **loading** — this component is itself the loading presentation.
  - **disabled** — does not apply.

---

#### Archetype coverage

Every screen archetype in the IA screen inventory is furnished by at least one named component or a
documented composition of them, so no archetype is left without a designed treatment:

| Archetype | Furnished by | Example screens |
|---|---|---|
| feed/list | **ListView** hosting **ChirpCard**s, with **LoadingState**, **EmptyState**, and **InlineError** for its states | Home timeline, Chirp / conversation, Notifications, Explore, Followers, Following |
| form | **TextField** + **Toggle** + **PrimaryButton** + **SecondaryButton** + **InlineError**, in a form layout | Sign up, Sign in, Recover access, Set new credential, Edit profile |
| profile | **ProfileHeader** + a **ListView** of **ChirpCard**s | Public profile, Own profile |
| settings | a list of **SettingRow**s inside the signed-in **NavShell** | Settings hub, Account & contact, Notification preferences, Sessions & devices, Blocked & muted, Export, (Delete routes to a confirmation) |
| modal/dialog | **Dialog** (modal variant), hosting **ComposeBox** for Compose | Compose, Report |
| confirmation | **Dialog** (confirmation variant) | Delete account, Delete chirp, Block / mute, Sign out |
| empty/landing | **EmptyState** | Landing, Onboarding, Rules & terms, Help & support, Not found |

The two shells themselves — signed-out and signed-in — are the **NavShell** component, inside which
every signed-in archetype renders.

## Responsive layout

chirp is read on a phone more than anywhere else, so the design is phone-first: every screen is drawn
for a narrow, one-handed, portrait screen first, and wider viewports are treated as that same design
given more room — never as the "real" layout that a phone is a cut-down copy of. The product is one
reading column of text with a navigation frame around it; responsive design decides how that frame
reflows and how far the column is allowed to grow, so the same product feels native on a phone, a
tablet, and a laptop without being rebuilt three times.

This section owns how the product adapts across viewports. It composes against the screen inventory and
its seven archetypes from the information-architecture section, and against the component library and
design tokens from the visual-language section — in particular the 600px reading measure
(`space.measure`), the phone and wide gutters (`space.gutter.mobile` 16px, `space.gutter.wide` 24px),
and the `NavShell`, `ListView`, `ChirpCard`, `ComposeBox`, `Dialog`, `ProfileHeader`, and `SettingRow`
components. It does not redefine any of them; it says where they move, what they collapse to, what
stacks, and what must stay reachable, so that the behaviours the other sections specify hold at every
width. The non-negotiable floor is REQ-PROD-035: the core loop — read the timeline, compose and post,
reply, like, follow, view a profile — works in a phone browser with no broken layout and no unreachable
control.

### Breakpoints and the layout strategy

Adaptation happens at three ranges, anchored to the reading measure rather than to any particular
device. These breakpoints are design intent in CSS pixels; a build realises them with whatever
mechanism it prefers.

| Token | Range | What it is | What the layout does |
|---|---|---|---|
| `breakpoint.compact` | width < 640px | Phone portrait, and most phones in landscape | One column filling the viewport inside a 16px (`space.gutter.mobile`) side gutter; navigation as a bottom tab bar plus a slim top bar. |
| `breakpoint.medium` | 640px ≤ width < 1024px | Large phones, tablets, split or small desktop windows | The 600px reading measure is reached and the column centres with a 24px (`space.gutter.wide`) gutter; navigation moves to a side column, collapsed to icons when width is tight. |
| `breakpoint.wide` | width ≥ 1024px | Laptop and desktop | The reading column stays at the 600px measure, centred; a persistent left navigation shows icons with labels; a secondary rail (for example discovery entries) MAY sit outside the measure where there is room, and the core loop never depends on it. |

640px is where the 600px measure plus two comfortable gutters first fits, so below it the column fills
the screen and at or above it the column settles to the measure. The strategy is the same at every
range: **one reading column, reflowed to the width available, never widened past the measure, and never
made to scroll sideways or demand a pinch-zoom.** Growing the window gives the navigation frame and an
optional secondary rail more room; it does not stretch a line of chirp text to the full width of a
desktop, because long lines are harder to track and the product would stop feeling like itself.

### How the navigation shell adapts

The `NavShell` keeps the same set of destinations the information-architecture section fixed; only its
presentation reflows. The active destination is marked in every presentation, and the accessibility
section owns how each presentation is operated and announced.

**The signed-in shell.**

- *Compact.* A **bottom tab bar** carries the primary destinations — Home, Explore, Notifications (with
  its unseen-activity indicator), and Profile — within thumb reach, and a prominent **Compose**
  affordance is present without opening a menu. A slim top bar carries the wordmark or the current
  screen's title and the entry to the account menu (Settings, Help & support, Rules & terms, Sign out).
  No primary destination is buried behind a hamburger: the loop's everyday moves are one tap from the
  bottom of the screen.
- *Medium.* The destinations move to a **left navigation column**, shown as icons when width is tight
  and as icons with labels as room allows; Compose becomes a prominent button in that column; the bottom
  bar retires once the side navigation is present, so there is never both at once.
- *Wide.* The left navigation is **persistent with labels**, Compose sits in it as a primary button, and
  a secondary rail MAY appear opposite the reading column for discovery or supplementary content where
  the window is wide enough — the reading column stays centred at its measure between them.

**The signed-out shell.** At every width the wordmark, a **Sign in** entry, and a prominent **Sign up**
entry stay visible without opening a menu, and **Rules & terms** and **Help & support** stay reachable
(at the footer at minimum). A visitor who arrives on a shared public link lands on the content inside
this shell with the join path in sight — a narrow screen never collapses the shell into an
authentication wall or hides Sign up behind a menu.

### How each screen archetype adapts

The seven archetypes are designed once here for the whole product rather than screen by screen.

- **feed/list** (home timeline, a chirp's conversation, notifications, search results, followers and
  following). One column at every width — it *is* the reading column. On a compact viewport it fills the
  width inside the gutter; the `ListView`'s load-older control and the timeline's "N new chirps" catch-up
  stay reachable without horizontal scrolling; each `ChirpCard`'s reply, like, and more-actions controls
  stay full-size and tappable; and the person's scroll position is kept when they return from drilling
  into a chirp. Nothing in a list reflows into side-by-side columns on a phone.
- **form** (sign up, sign in, recover access, set new credential, edit profile). `TextField`s stack
  full-width within the gutter, one field per row — never a two-column form on a phone. Each field keeps
  its label and its `InlineError` with it, so a validation message is read at the field, not in a summary
  pushed off-screen. The primary submit stays reachable when the on-screen keyboard is open: the person
  can scroll the form to the submit without first dismissing the keyboard, and the keyboard never
  permanently hides the action.
- **profile** (public and own). The `ProfileHeader` stacks on a compact viewport — avatar, display name,
  handle, bio, and the follower/following counts — with its context action (Follow, or Edit profile on
  one's own profile) visible without horizontal scrolling. The person's chirps render in the single
  column below as an ordinary feed/list.
- **settings** (the hub and its sub-sections). The Settings hub is a single column of `SettingRow`s. On
  a compact viewport the hub and each sub-section are **separate full screens**, and a sub-section
  carries a labelled way back to the hub; the browser back control does the same. A two-pane
  hub-and-detail presentation is permitted on a wide viewport but is not required, and the phone
  behaviour is the baseline.
- **modal/dialog** (Compose, Report). On a compact viewport the `Dialog` modal variant presents as a
  **full-screen or bottom-sheet surface** rather than a small centred panel, so its hosted task — the
  `ComposeBox`, for instance — and its Post or submit button are comfortably usable with the keyboard
  open. The close affordance stays reachable and the browser back control dismisses the overlay without
  leaving the underlying screen. On medium and wide viewports the centred panel over the scrim (per the
  component) is used.
- **confirmation** (delete account, delete chirp, block or mute, end a session, sign out). On a compact
  viewport a confirmation presents as a bottom sheet or a full-width dialog with **both** actions fully
  visible — the destructive confirm and the plain cancel — so neither is clipped at the screen edge and
  the safe way out is always tappable. The consequence text stays legible; a long explanation scrolls
  within the surface rather than pushing the actions off-screen.
- **empty/landing** (Landing, Onboarding, Rules & terms, Help & support, Not found). The `EmptyState`
  block — illustration, headline, explanation, and next step — scales down to the narrow column with its
  next-step action kept prominent. On the signed-out Landing, Sign up and Sign in stay prominent and
  reachable on a phone.

### Touch targets, reachability, and input ergonomics

On a touch viewport the product is operated with a thumb, so size and placement are part of whether it
works at all.

- **Target size.** Every interactive control presents a touch target of at least 44×44 CSS px, with at
  least `space.2` (8px) between adjacent targets so neighbours are not mis-tapped. The `IconButton`'s
  built-in `space.2` hit padding already serves this; dense rows (a `ChirpCard`'s action row, a
  `SettingRow`'s control) keep their controls to the minimum. This is the responsive ergonomic floor; the
  accessibility section owns the formal conformance target-size success criterion and does not conflict
  with it.
- **Reachability.** The core-loop primary actions — the Compose affordance, a `ChirpCard`'s like and
  reply, and primary navigation — sit within comfortable thumb reach on a tall phone, which is why the
  signed-in shell puts navigation and Compose at the bottom on a compact viewport rather than only at the
  top. A destructive confirmation places its actions where a thumb rests, with the safe action no harder
  to reach than the destructive one.
- **Input ergonomics.** Focusing a text field does not trigger the browser's automatic zoom, and the core
  loop works in both portrait and landscape on a phone. Content reflows to the narrower dimension; the
  page never scrolls horizontally and never requires a pinch-zoom to read a chirp or operate a control.
  A drafted chirp survives an orientation change (it is already preserved across an interruption per the
  `ComposeBox`), so rotating the phone does not lose work.

### What must hold on a phone for the core loop

This is the release floor the rest of the section serves. On a phone-sized viewport in portrait, a
signed-in person can, end to end: read the home timeline and page through it; open Compose and post a
chirp within the character limit; reply to a chirp; like and unlike; follow and unfollow; and open and
read a public profile. At no step is the page scrolled horizontally, a control placed off-screen, an
action target too small to tap, or a layout broken by the narrow width. If any one of those moves cannot
be completed on a phone, the product is not releasable, regardless of how well it behaves on a desktop.

## Accessibility

Accessibility is part of whether a person can use chirp and trust it with their account, not a
finish applied at the end. A reader who navigates by keyboard, a person using a screen reader, someone
who has zoomed the page to read it, and a reader who has asked their system to stop animating things are
all people chirp is for, and the product is held to a stated standard for them in the same way it is held
to a stated standard for durability or availability.

This section does three things. It names the exact standard chirp conforms to, so "accessible" is a
checkable claim and not a sentiment. It designs to that standard across the seven screen archetypes in
the **Screen inventory** and the named pieces in the **Component library** — rather than re-deciding
accessibility screen by screen, it is decided once per archetype and once per component, which is the
same economy the visual system and the responsive design use. And it says how conformance is verified, so
a tester who never met the author can reach the same verdict. Where the visual system already fixes a
token-level rule that carries accessibility weight — the contrast intents on the palette (REQ-UX-022), the
always-visible focus ring (REQ-UX-025), icons never carrying meaning by colour alone (REQ-UX-028), and the
`motion.reduced` contract (REQ-UX-029) — this section composes against those rules and states the
conformance obligation they serve, rather than restating or redefining them.

### The standard chirp commits to

chirp conforms to the **Web Content Accessibility Guidelines (WCAG) 2.1, Level AA** — every Level A and
every Level AA success criterion in WCAG 2.1, across the product. This is the named, versioned target the
rest of this section is held to, and it is the concrete form of the product-level accessibility baseline
(REQ-PROD-036), which leaves the exact standard to this design to fix. WCAG 2.1 AA is the standard
commonly referenced by consumer and public-sector accessibility obligations, which is why it is the floor
a releasable consumer web product is expected to meet rather than an aspiration above it.

What committing to WCAG 2.1 AA obliges chirp to, in the terms a reader experiences:

- everything a person can do with a pointer, they can do with a keyboard alone, in a sensible order, with
  the focused element always visible and never trapped (WCAG 2.1.1, 2.4.3, 2.4.7, 2.1.2);
- every control has a name, a role, and a state that assistive technology can report (WCAG 4.1.2), and the
  structure of each screen — its regions, its headings, its lists — is conveyed and not merely drawn
  (WCAG 1.3.1, 2.4.6);
- text and meaningful controls meet the contrast ratios in WCAG 1.4.3 and 1.4.11, the page can be enlarged
  and reflowed without loss (WCAG 1.4.4, 1.4.10, 1.4.12), and nothing relies on colour, shape, or motion
  as the only way to carry a fact (WCAG 1.4.1, 1.4.11, 2.3.1);
- the product tells assistive technology when something changed that was not the result of a direct action
  — a chirp posted, a page loaded, an action failed, a session expired (WCAG 4.1.3, 3.3.1).

**The gated conformance set versus the whole product.** The release bar (REQ-PROD-036) gates the
accessibility baseline on the screens a person must pass through to use chirp for what it is for: the core
loop (read the timeline, compose and post, reply, like, follow, view a profile) and the screens that get
them in and oriented (sign-up, sign-in, recovery, onboarding), together with the empty, loading, and error
states those screens present. Those screens MUST meet WCAG 2.1 AA for a release. The same standard applies
to every other screen in the inventory — settings and its sub-sections, export, deletion, report and
block — and they are designed to it here; where such a screen belongs to a product surface the release bar
itself does not gate (notifications and its preferences on REQ-PROD-029, Explore on REQ-PROD-042), its
accessibility requirement inherits that surface's non-gating flag and ships when the surface ships, held to
the same standard when it does.

**The near-term direction** is WCAG 2.2 AA, which adds criteria chirp should adopt as a fast follow — a
minimum target size (2.2's 2.5.8), focus never hidden behind sticky chrome (2.4.11), and no need to
remember information across steps of a flow (3.3.7). One of these, the minimum target size, is designed in
below ahead of a full 2.2 move because it matters most on a phone; it is marked as the beyond-2.1 commitment
it is, so the staging is explicit.

### Operability — the product works without a mouse

Operability is the half of accessibility a keyboard user and most switch and voice users depend on
entirely: if a thing can only be done by pointing at it, it cannot be done by them at all.

**Everything is keyboard-operable.** Every action the product offers through a pointer is reachable and
operable with the keyboard alone, on every archetype. Concretely: in a **feed/list** a person can move
through the chirps and operate each **ChirpCard**'s reply, like, and more-actions controls and open a chirp
into its conversation; in a **form** they can reach and complete every **TextField**, **Toggle**, and
button and submit; in **settings** they can operate every **SettingRow**; the **modal/dialog** and
**confirmation** **Dialog**s and the **ComposeBox** are fully operable; and the signed-out and signed-in
**NavShell**s — every destination, the compose affordance, and the account menu — are operated from the
keyboard. No chirp feature is pointer-only (WCAG 2.1.1).

**Focus order follows the reading order, and focus is always visible.** As a person tabs through a screen,
focus moves in the order the screen reads — the nav shell, then the screen's heading and content, then its
actions — so the sequence makes sense without sight of the pointer (WCAG 2.4.3). The focused element always
shows the `color.border.focus` ring the visual system guarantees (REQ-UX-025); this section's obligation is
that focus order is logical and that focus is never lost to an invisible or off-screen element (WCAG
2.4.7). When a view changes — a **Dialog** opens, a route changes, the catch-up control loads new chirps —
focus is moved deliberately to a sensible place (the dialog's first control or its heading; the new
screen's main heading) rather than left stranded on a control that no longer exists.

**No keyboard trap, and overlays return focus.** A keyboard user can move into and out of every component
with the keyboard; nothing captures the Tab key with no way out (WCAG 2.1.2). The **Dialog** is the one
place focus is deliberately held — a modal traps focus within the open panel so Tab cycles inside it, which
is correct modal behaviour — but it is always escapable: Esc closes it, the close control and Cancel are in
the cycle, and on close focus returns to the control that opened it (composing with the component's stated
focus-return behaviour).

**Bypass to content.** Because the **NavShell** repeats its destinations at the top of every signed-in
screen, a keyboard or screen-reader user is given a way to skip past it straight to the screen's main
content, so they are not tabbing through the whole shell on every screen (WCAG 2.4.1). This is satisfied by
a visible-on-focus "Skip to content" control at the top of the tab order together with the landmark and
heading structure below, either of which lets assistive technology jump to `main`.

**Targets are big enough to hit.** Pointer and touch targets for the product's controls are at least 24×24
CSS pixels, and the primary touch targets on a phone — the **NavShell** destinations, the compose
affordance, and a **ChirpCard**'s action row — are sized to the comfortable minimum the responsive design
sets, so a control is not a pixel-perfect tap. This adopts WCAG 2.2's Target Size (Minimum) (2.5.8) ahead
of a full move to 2.2; it is a commitment beyond the 2.1 AA floor, and it serves the product's phone
requirement (REQ-PROD-035).

### Semantics — the product makes sense to assistive technology

A screen reader does not see chirp's layout; it reads the names, roles, states, and structure the product
exposes. Semantics is making sure what a sighted person perceives from position and styling is also present
as information a screen reader can report (WCAG 1.3.1, 4.1.2).

**Every component carries a name, a role, and its state.** The component library is the right place to fix
this once, because the same component carries the same semantics everywhere it appears. Each piece in the
library exposes the following to assistive technology:

| Component | Role exposed | Accessible name | States exposed |
|---|---|---|---|
| **NavShell** | `navigation` landmark (signed-out and signed-in); account entry is a menu button | "Primary" / "Account" | the current destination is marked programmatically (not colour alone); unseen-activity on Notifications is announced, not only dotted |
| **ChirpCard** | an `article` with the author and time in its label; the card's open-conversation target is a link; reply/like/more are buttons | author + relative time; each action button names its action and its subject | like exposes pressed/not-pressed; a removed chirp is announced as a labelled "not available" row, not a silent gap |
| **ProfileHeader** | a labelled header region | the person's display name and handle | the follow control exposes following / not-following; counts are read with their labels |
| **ComposeBox** | a multi-line text box with a visible, associated label | "Post a chirp" / "Reply to @handle" | the remaining-character count is associated with the field and announced as it nears and passes the limit; Post exposes disabled and busy |
| **TextField** | a labelled single- or multi-line input | its visible label | hint and error associated via description; invalid and required are exposed; a pending async check is announced |
| **Toggle** | a `switch` | its row label | on / off exposed as the switch state, with a visible on/off label as well |
| **PrimaryButton** / **SecondaryButton** | `button` | its visible verb label ("Post", "Save changes", "Delete chirp") | disabled and busy exposed; a destructive primary is named by its action, not by colour |
| **IconButton** | `button` (or a link where it navigates) | a text label always present, never the icon alone | active/selected (e.g. the current nav item, a liked chirp) exposed programmatically |
| **ListView** | a list / feed structure; "load older" and the catch-up control are buttons | the list's purpose; "Load older chirps"; "N new chirps" | its empty, loading, and load-failure states are conveyed to assistive technology (see Feedback and errors) |
| **SettingRow** | a labelled row grouping its control with its description | the setting's label | the inner control's own state; a destructive row names its consequence |
| **Dialog** | `dialog`, modal, labelled by its title | the dialog title | open/closed via presence; the confirm control exposes disabled and busy |
| **Toast** / **InlineFeedback** | a status region (see Feedback and errors) | the message text | success vs failure carried by text and icon, not colour alone |
| **InlineError** | an alert associated with its field or control | the error text | — (its presence is the error state; it is tied to the field as that field's description and invalidity) |
| **EmptyState** | a region with a heading and an action; its error variant is an alert | the headline | genuinely-empty vs failed-to-load distinguished in text, not only by illustration |
| **LoadingState** | a busy indication with a polite "loading" message; decorative skeletons are hidden from assistive technology | "Loading" | resolves to the owning component's content or its error state |
| **Avatar** | an image with the person's name as its text alternative, or marked decorative when an adjacent text name already carries it | the person's display name | the initials-monogram and load-failure fallbacks carry the same name, never a "broken image" |

**Regions and headings give each screen a shape.** Every screen exposes landmark regions — a banner and
the primary `navigation` for the **NavShell**, a single `main` for the screen's content, and a
`contentinfo` footer where the signed-out shell carries one — so a screen-reader user can jump between the
parts of the page (WCAG 1.3.1). Every screen has exactly one top-level heading that names it and matches the
document title the wayfinding model sets, and sub-headings nest in order without skipping a level (WCAG
1.3.1, 2.4.6). The settings hub and its sub-sections, which are a small secondary navigation, expose their
grouping so a person knows they are inside a sub-section and how to get back.

**Lists are exposed as lists.** The **feed/list** archetype — the home timeline, a conversation, the
notifications list, search results, and followers/following — is conveyed with its item structure, so a
screen-reader user can move item by item and knows where they are in the set rather than meeting an
undifferentiated run of text (WCAG 1.3.1). The pagination and catch-up affordances are named controls, not
mystery widgets.

**Icon-only controls always have words.** Every **IconButton** exposes a text label to assistive technology
and on hover, composing with the visual system's rule that an icon is never the only carrier of meaning
(REQ-UX-028); an active or selected state (the current nav destination, a liked chirp) is exposed as a
state and not inferred from a colour or a fill alone (WCAG 1.1.1, 4.1.2).

### Perception — the product is perceivable for different sight and motion needs

**Contrast.** chirp meets WCAG 1.4.3 for text (at least 4.5:1 for body text, 3:1 for large text) and WCAG
1.4.11 for the boundaries of meaningful controls and icons (at least 3:1), in each theme the product ships.
This is fixed at the token level by the palette's contrast intents and verified by the visual system's
contrast requirement (REQ-UX-022); the accessibility obligation stated here is that meeting those ratios is
a condition of WCAG 2.1 AA conformance and is part of the gated baseline, checked on the live product in
whatever theme(s) ship.

**Enlarging and reflowing the page.** A person can enlarge text to 200% without content being clipped or
controls becoming unreachable (WCAG 1.4.4), and the product reflows to a 320-CSS-pixel-wide viewport
without a second scrollbar in the reading direction and without loss of content or function — the single
600px reading column and phone-first layout the visual and responsive designs set is what makes this hold
(WCAG 1.4.10). Overriding text spacing (line height, paragraph, letter, and word spacing) does not clip or
overlap content (WCAG 1.4.12).

**No fact is carried by colour alone.** Composing with the visual system's rule (REQ-UX-022 / consistency
rule 7), every state that uses a semantic colour also uses text, an icon, or a label: a field error is a
`color.danger` border and an **InlineError** message; an active like is the `color.like` colour and a
filled heart and a "Liked" label; the current nav destination is a fill and a mark. A person who cannot
distinguish those colours still gets the fact (WCAG 1.4.1). The product does not restrict itself to a single
screen orientation.

**Motion is safe and can be turned down.** Nothing in chirp flashes more than three times in any one second,
so no content poses a seizure risk (WCAG 2.3.1). When a reader has asked their system to reduce motion, the
product honours it through the `motion.reduced` contract the visual system defines (REQ-UX-029): transitions
become an instant change or a short opacity fade, counts do not animate, and nothing conveys required
information only through movement. The bounded loading indicator is the one thing permitted to animate, and
it has a static, announced equivalent.

### Feedback and errors — the product speaks its state

A sighted person sees a toast slide in, a skeleton fill with chirps, or an error appear under a field. A
screen-reader user perceives none of that unless the product announces it. This is the accessible
counterpart of the considered-states work: the same empty, loading, error, and status moments the product
is required to show (REQ-PROD-033, REQ-PROD-034) must also be spoken.

**Status messages are announced without stealing focus.** When something changes that was not a direct
keystroke — a chirp is published, a follow is saved, a report is received, new chirps arrive behind the
catch-up control — the product announces it through a polite live region so a screen-reader user hears it
without being yanked away from what they are doing (WCAG 4.1.3). This is how **Toast** and **InlineFeedback**
reach assistive technology: the confirmation is spoken, and the success-versus-failure distinction is in the
words, not the colour.

**Errors are announced and tied to their cause.** A failed action is announced, and a field-level error is
both associated with its field (so moving to the field re-reads the problem) and names the field as invalid,
so a screen-reader user reaches the problem and understands it without hunting (WCAG 3.3.1). The
**InlineError** text says what happened and what to do in plain words with no system or error code
(composing with the voice rules, REQ-UX-032), which satisfies both error identification and a suggestion for
fixing it (WCAG 3.3.1, 3.3.3); every **TextField** keeps its visible, associated label and any instruction
(WCAG 3.3.2). A failure that has no single control to attach to is announced through the status region
rather than left silent.

**Empty, loading, and expired states reach assistive technology.** A **ListView**'s loading state announces
that content is loading while its decorative skeletons are hidden from the screen reader, and resolves to
either the arrived content or an announced error with a reachable retry — never silence, and never a
skeleton read out as gibberish. An **EmptyState** is a real heading and an action a screen-reader user lands
on and can act on, and its error variant is announced as an alert so a failed load is told apart from a
genuinely empty surface. When a session expires mid-action, the interception the interaction model performs
(REQ-UX-008) is announced with its reason and focus is moved to the sign-in screen, so an expired session is
a spoken, recoverable event and not a silent dead end (WCAG 3.3.1; REQ-PROD-034).

### Verification — how conformance is checked

A conformance claim is only as good as the check behind it. chirp's accessibility is verified, before a
release, by exercising the gated conformance set — the core loop and the get-in screens with their empty,
loading, and error states — three complementary ways, because no one way catches everything:

1. **An automated check** across the gated screens in each shipped theme, to catch the machine-detectable
   failures (missing names, invalid roles, contrast below ratio, missing labels). Automated tools find a
   minority of issues but find them cheaply and repeatably; a clean automated pass is necessary, not
   sufficient.
2. **A keyboard-only pass**: with the pointer unused, complete the core loop and the get-in flows — reach
   and operate every control, confirm focus order reads sensibly, focus is always visible, no component
   traps focus, the skip-to-content path works, and every **Dialog** is escapable and returns focus.
3. **A screen-reader pass** of the core loop on at least the matrix below: confirm every control is reported
   with a sensible name, role, and state; headings and regions let the reader navigate; lists are navigable
   as lists; and the posted / saved / failed / loading / expired-session moments are all announced.

**The support matrix** the screen-reader and keyboard passes are run against is at least one screen reader
per major platform with its common browser pairing — a desktop screen reader with a current browser, and a
mobile screen reader with the mobile browser — since chirp is a responsive web product people reach from
both. The matrix is stated with the release so the claim names what it was checked against.

**The gate.** A release is blocked until the gated conformance set passes all three checks with no
outstanding Level A or Level AA failure; a failure found is a fix, not a waiver. An accessibility issue on a
non-gating surface (a notifications or Explore screen) is tracked against that surface and travels with it.
This is the concrete acceptance behind the product baseline (REQ-PROD-036): the standard is WCAG 2.1 AA, the
evidence is the three passes over the gated set, and the matrix records what the evidence was gathered on.

## Requirements and acceptance criteria

The complete `REQ-UX-*` register for the experience design, consolidated across information architecture (`REQ-UX-001..019`), the visual language and component library (`REQ-UX-020..039`), screens and flows (`REQ-UX-040..089`), responsive behaviour (`REQ-UX-090..099`), and accessibility (`REQ-UX-100..119`). Every entry has a statement, an `acceptance_criteria` a person who never met the author can check against the live product, a `release_gating` flag (a `false` flag carries its reason), and the product requirement(s) it serves. Ids are unique and contiguous from `REQ-UX-001` to `REQ-UX-119`.

#### REQ-UX-001 — A signed-out shell that explains chirp and never dead-ends
- **statement**: A person with no session sees a signed-out shell carrying the chirp wordmark, a Sign in
  entry, a prominent Sign up entry, and footer access to the rules and to help, on every signed-out screen.
- **acceptance_criteria**: With no session, open the landing and a public profile link; on each, Sign up,
  Sign in, Rules & terms, and Help & support are all reachable without first signing in.
- **release_gating**: true
- **references**: REQ-PROD-030, REQ-PROD-031

#### REQ-UX-002 — A signed-in shell exposing every primary destination
- **statement**: After signing in, a persistent shell exposes Home, Explore, Notifications, Profile,
  Settings, and a Compose affordance, present and reachable from every signed-in screen.
- **acceptance_criteria**: Signed in, from the timeline, a profile, a conversation, and a settings
  sub-section, confirm each of Home, Explore, Notifications, Profile, Settings, and Compose is reachable
  from the shell without using the browser back control.
- **release_gating**: true
- **references**: REQ-PROD-002, REQ-PROD-017

#### REQ-UX-003 — The current location is always indicated
- **statement**: On every screen the product shows where the person is: the active primary-nav destination
  is marked, and the document title and main heading name the current screen.
- **acceptance_criteria**: Navigate to Home, Explore, Notifications, Profile, and a settings sub-section in
  turn; on each, the active destination is visibly marked and the browser tab title reflects that screen.
- **release_gating**: true
- **references**: REQ-PROD-002, REQ-PROD-033

#### REQ-UX-004 — The product is built from the published screen inventory
- **statement**: Every screen chirp presents appears in the screen inventory with a route, exactly one
  archetype, and a declared auth state; no screen exists outside the inventory and no inventory screen is
  missing from the product.
- **acceptance_criteria**: Walk the product signed-out and signed-in; every screen reached matches an
  inventory entry (route, archetype, auth state), and every signed-in-reachable inventory screen can be
  reached.
- **release_gating**: true
- **references**: REQ-PROD-033, REQ-PROD-032

#### REQ-UX-005 — Every required surface is reachable without an external link
- **statement**: From the signed-in shell a person can reach the timeline, compose, their profile,
  notifications, settings, discovery, help, and the rules, each in at most two steps, without holding an
  external link.
- **acceptance_criteria**: Signed in and starting from the home timeline, reach each of compose, own
  profile, notifications, settings, explore, help, and rules in no more than two steps through the product.
- **release_gating**: true
- **references**: REQ-PROD-028, REQ-PROD-042

#### REQ-UX-006 — Profiles and chirps are addressable and shareable
- **statement**: A public profile and an individual chirp each have a stable, shareable URL that opens that
  exact screen directly.
- **acceptance_criteria**: Copy the URL of a profile and of a chirp's conversation, open each in a fresh
  browser context; each lands on that profile / that chirp, not on a generic home.
- **release_gating**: true
- **references**: REQ-PROD-007, REQ-PROD-003

#### REQ-UX-007 — Auth-gated routing returns the person to where they asked to go
- **statement**: Opening a signed-in-only URL without a session routes the person to sign-in, and after a
  successful sign-in returns them to the originally requested screen rather than a default screen.
- **acceptance_criteria**: With no session, open a signed-in-only URL (for example settings); you are
  routed to sign-in; after signing in you land on that settings screen, not on the home timeline.
- **release_gating**: true
- **references**: REQ-PROD-030, REQ-PROD-034

#### REQ-UX-008 — An expired session is intercepted, explained, and recovered from
- **statement**: An action taken with an expired or invalid session routes the person to sign-in with the
  reason stated, preserves in-progress input, and returns them to where they were after they sign back in.
- **acceptance_criteria**: With a session expired, take an action (for example open the timeline or submit
  a drafted chirp); you are routed to sign-in with an explanation, your drafted text is not lost, and after
  signing in you resume where you were.
- **release_gating**: true
- **references**: REQ-PROD-034

#### REQ-UX-009 — Browser back and forward behave predictably
- **statement**: The browser back control returns to the previous screen without re-submitting the prior
  action and without losing the person's place in a scrolled timeline; opening a dialog is a step that back
  dismisses by closing the dialog.
- **acceptance_criteria**: Scroll the timeline, open a chirp, then press back — you return to the timeline
  at your place. Open the compose dialog, then press back — the dialog closes and the underlying screen is
  unchanged. Pressing back after posting does not re-post the chirp.
- **release_gating**: true
- **references**: REQ-PROD-033, REQ-PROD-034

#### REQ-UX-010 — Lists load in bounded pages with a catch-up affordance
- **statement**: Every list loads in bounded pages rather than one unbounded load, offers an explicit way
  to load older items, and the timeline offers a way to pull in chirps posted since the person last looked.
- **acceptance_criteria**: Open a timeline with more items than one page; it loads a bounded page, offers a
  way to load older chirps, and offers a way to load chirps posted since you opened it — it does not attempt
  to load every chirp at once.
- **release_gating**: true
- **references**: REQ-PROD-002

#### REQ-UX-011 — Destructive actions are confirmed before they proceed
- **statement**: Every irreversible or destructive action — delete account, delete a chirp, block or mute,
  end a session, sign out of all sessions — requires an explicit confirmation step that names the
  consequence and offers a cancel before it proceeds.
- **acceptance_criteria**: Initiate account deletion, chirp deletion, a block, and ending a session; each
  presents a confirmation that states what will happen and a cancel. Cancel each; nothing changes. The
  account-deletion confirmation also states what is and is not retained and points to export.
- **release_gating**: true
- **references**: REQ-PROD-022, REQ-PROD-020, REQ-PROD-014, REQ-PROD-027

#### REQ-UX-012 — The result of every action is shown
- **statement**: A completed action is confirmed with brief, non-blocking feedback, and a failed action is
  shown where it was taken — against the field for a form, against the control for an in-place action —
  with enough to understand and retry; no action completes or fails silently.
- **acceptance_criteria**: Publish a chirp, follow an account, and save a setting; each shows brief
  confirmation. Force a failure (for example submit an invalid form, or act while offline); the failure is
  shown at the point of action with a way to retry, not as a silent no-op.
- **release_gating**: true
- **references**: REQ-PROD-033, REQ-PROD-001, REQ-PROD-025

#### REQ-UX-013 — Compose is reachable from anywhere, and addressable
- **statement**: A signed-in person can start composing a chirp from any screen through a persistent
  compose affordance, and a dedicated compose URL supports opening compose directly.
- **acceptance_criteria**: Signed in, from the timeline, a profile, and notifications, open compose from
  the shell without navigating away first. Open the compose URL directly; the compose surface opens.
- **release_gating**: true
- **references**: REQ-PROD-001

#### REQ-UX-014 — Sign-up lands a new account in the product, onboarding first
- **statement**: A completed sign-up establishes the session and lands the person on onboarding — they are
  not prompted to sign in again immediately after creating the account, and they do not land on a blank
  timeline.
- **acceptance_criteria**: Complete a fresh sign-up; you are signed in and land on the onboarding screen
  (which explains the empty state and offers a way toward a non-empty timeline), not on a sign-in prompt
  and not on a blank page.
- **release_gating**: true
- **references**: REQ-PROD-032, REQ-PROD-030, REQ-PROD-031

#### REQ-UX-015 — Sign-in, sign-up, and recovery are interlinked
- **statement**: The sign-in, sign-up, and recovery screens each link to the others, so a person who
  reached the wrong one can correct course without leaving the flow.
- **acceptance_criteria**: On sign-in, find a link to sign-up and to recovery; on sign-up, find a link to
  sign-in; on recovery, find a link back to sign-in. Each link opens the named screen.
- **release_gating**: false — the gated behaviors are the flows themselves (REQ-PROD-010/011/013); the
  cross-links are a convenience a first release can omit, provided recovery remains reachable from help.
- **references**: REQ-PROD-011, REQ-PROD-013

#### REQ-UX-016 — Sign-out returns to the signed-out shell and does not leak back
- **statement**: Signing out ends the session and returns the person to the signed-out landing; after
  sign-out, the browser back control does not re-expose a protected screen's content.
- **acceptance_criteria**: Signed in on a screen, sign out; you land on the signed-out landing. Press back;
  you do not see the protected screen's content — you remain in the signed-out shell or are routed to
  sign-in.
- **release_gating**: true
- **references**: REQ-PROD-014

#### REQ-UX-017 — Settings groups account-level choices, each addressable
- **statement**: Account-level choices live under one settings area with labelled sub-sections — edit
  profile, account & contact, notification preferences, sessions & devices, blocked & muted, export, delete
  — each independently addressable and each returning to the settings hub.
- **acceptance_criteria**: Open settings; the sub-sections above are listed. Open each directly by its URL;
  it opens on that sub-section. From a sub-section, return to the settings hub.
- **release_gating**: true
- **references**: REQ-PROD-017, REQ-PROD-012, REQ-PROD-019, REQ-PROD-020

#### REQ-UX-018 — The rules and a help entry are reachable from both shells
- **statement**: The rules/terms and a user-facing help & support entry are reachable from both the
  signed-out shell and the signed-in shell.
- **acceptance_criteria**: With no session, reach Rules & terms and Help & support from the signed-out
  shell. Signed in, reach both from the account menu. Each opens the named screen.
- **release_gating**: true
- **references**: REQ-PROD-026, REQ-PROD-028

#### REQ-UX-019 — A notifications entry with an unseen-activity indicator
- **statement**: The signed-in shell carries a notifications entry that indicates unseen activity and
  leads to the notifications list.
- **acceptance_criteria**: Have another account reply to, follow, or like you; the shell's notifications
  entry shows an unseen indicator. Open it; the notifications list shows the activity and the indicator
  clears.
- **release_gating**: false — notifications ride REQ-PROD-029, which the release bar does not gate; the
  entry and its indicator ship with notifications when notifications ship.
- **references**: REQ-PROD-029

#### REQ-UX-020 — The product reads as one product across screens
- **statement**: Every screen draws its type, colour, spacing, radii, and controls from the one
  published token set, so unrelated screens share the same visual treatment rather than each
  styling itself.
- **acceptance_criteria**: On any three screens of different archetypes (for example the home
  timeline, a profile, and a settings sub-section), body text is the same size and leading,
  the gutters and the spacing rhythm match, buttons share one shape and one primary colour, and
  no screen introduces a colour or control style absent from the others.
- **release_gating**: true
- **references**: REQ-PROD-033, REQ-PROD-030

#### REQ-UX-021 — Text is legible and hierarchy is clear
- **statement**: Reading text is set at a legible size with comfortable line length and leading,
  and a visible hierarchy distinguishes headings, body, and meta.
- **acceptance_criteria**: Chirp and reading text renders at no less than 15px with generous line
  spacing; a reading column's line length stays within a readable band rather than spanning the
  full width of a wide window; a screen's main heading is visibly larger and heavier than its body
  text, and meta text (timestamps, handles) is visibly quieter than body text.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-035

#### REQ-UX-022 — Colour contrast meets WCAG 2.1 AA
- **statement**: Text and interactive elements meet WCAG 2.1 AA contrast — at least 4.5:1 for
  normal text (3:1 for large text) and at least 3:1 for control boundaries and icons that carry
  meaning — in whichever theme(s) the product ships.
- **acceptance_criteria**: Sample body text, secondary text, link text, primary-button text, and
  a field's border on the live product and measure each with a contrast tool; normal text reaches
  ≥ 4.5:1, large text ≥ 3:1, and control/icon boundaries ≥ 3:1. The check passes in each theme the
  product offers.
- **release_gating**: true
- **references**: REQ-PROD-036

#### REQ-UX-023 — A light theme and a dark theme, at parity
- **statement**: The product offers a light theme and a dark theme; every screen renders legibly
  and completely in both, with no element left unthemed, invisible, or failing contrast in either.
- **acceptance_criteria**: Switch between the light and dark themes and walk the core loop (home,
  a profile, compose, a settings sub-section); in each theme every screen is fully styled — no
  element disappears into its background, keeps a wrong-theme colour, or drops below the contrast
  bar.
- **release_gating**: false — the gated bar is legibility and contrast (REQ-UX-021, REQ-UX-022) in
  whatever theme(s) ship; a first release may ship a single theme and add the second as a fast
  follow, provided the shipped theme meets the bar.
- **references**: REQ-PROD-036, REQ-PROD-033

#### REQ-UX-024 — The theme follows the reader's preference and persists
- **statement**: The product defaults to the reader's system light/dark preference, lets them
  override it, and remembers the choice on the next visit.
- **acceptance_criteria**: With the operating system set to dark, open chirp fresh — it opens in
  the dark theme; set a manual override to light and reload — it stays light; clear the override
  and reload — it follows the system again.
- **release_gating**: false — depends on there being more than one theme (REQ-UX-023); ships when
  the second theme ships.
- **references**: REQ-PROD-036

#### REQ-UX-025 — Every interactive element has a visible focus indicator
- **statement**: Every control reachable by keyboard shows a clearly visible focus indicator when
  focused, and the indicator is never suppressed.
- **acceptance_criteria**: Using only the keyboard, tab through a screen's controls (links,
  buttons, fields, menu items); each shows a distinct focus ring that stands out from its
  surroundings, and no focusable control receives focus without a visible indicator.
- **release_gating**: true
- **references**: REQ-PROD-036

#### REQ-UX-026 — Consistent spacing, grid, and reading measure
- **statement**: Spacing and layout follow one spacing scale and one reading-column measure across
  archetypes, so content is neither cramped nor stretched to an unreadable line length.
- **acceptance_criteria**: On a wide window, the reading column (timeline, conversation, profile
  chirps, a form) is held to a bounded measure rather than spanning the whole width; on a phone
  viewport the same content fills the width with a consistent edge gutter; the spacing between
  items is visibly uniform within and across these screens.
- **release_gating**: true
- **references**: REQ-PROD-035, REQ-PROD-033

#### REQ-UX-027 — Layers stack in a consistent, legible order
- **statement**: Overlapping surfaces follow one elevation order — content below menus, menus
  below dialogs, dialogs below toasts — and a modal dialog dims the content behind it.
- **acceptance_criteria**: Open a menu (it sits above the content), then a dialog (it sits above
  the content on a dimmed scrim, and the menu does not show through), then trigger a toast (it
  appears above the dialog); at no point does a lower layer render on top of a higher one.
- **release_gating**: true
- **references**: REQ-PROD-033, REQ-PROD-022

#### REQ-UX-028 — Icons are consistent and never the sole carrier of meaning
- **statement**: Icons come from one consistent set, an active/selected state is shown by more
  than colour, and every icon-only control exposes a text label.
- **acceptance_criteria**: The current primary-nav destination is shown filled and marked, not by
  colour alone (confirm by viewing in greyscale or with a colour-blind simulation — the active
  item is still identifiable); every icon-only control (compose, like, menu) exposes an accessible
  name to a screen reader and a label on hover.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-002

#### REQ-UX-029 — Motion is purposeful, bounded, and respects reduced-motion
- **statement**: Motion is used only to orient, confirm, or show relationship; nothing animates
  indefinitely except a bounded loading indicator; and the product honours a reader's
  reduced-motion preference.
- **acceptance_criteria**: Open and close a menu and a dialog and trigger a toast — each
  transition is brief (well under half a second) and settles; no element animates or blinks
  continuously apart from a loading indicator. With the system's reduce-motion setting on, repeat
  the actions — animation is removed or reduced to a short fade, and no information is lost.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-033

#### REQ-UX-030 — People are shown consistently, and a missing picture is never broken
- **statement**: Avatars are rendered consistently (circular, from a fixed set of sizes), and a
  person with no picture is shown a readable initials placeholder rather than a broken-image glyph.
- **acceptance_criteria**: Across the feed, a profile header, and the navigation, the same
  person's avatar is circular and uses the size for that context; open a profile for an account
  with no picture — it shows that account's initials on a coloured fill, never a broken-image icon
  or an empty box.
- **release_gating**: false — the designed avatar system (fixed sizes, deterministic placeholder)
  is polish; the hard minimum, that no state renders a broken image, is carried as part of the
  considered-states requirement (REQ-UX-031 / REQ-PROD-033).
- **references**: REQ-PROD-033, REQ-PROD-007

#### REQ-UX-031 — Empty, loading, and error states have a designed visual treatment
- **statement**: Empty lists, loading content, and failed actions each have a defined visual
  treatment drawn from the system — an explanatory empty block, a loading skeleton, and an inline
  error in the danger role — rather than a blank area, a frozen screen, or a raw failure.
- **acceptance_criteria**: Reach an empty list (for example a brand-new account's timeline), a
  list while it is loading, and a failed action (for example submit a form with a server error);
  the empty list shows an explanatory block with a next step, the loading list shows a skeleton
  placeholder rather than a blank or a spinner-on-blank, and the failure shows an inline message in
  the product's error styling at the point of action — none of the three is a blank area or an
  unstyled error.
- **release_gating**: true
- **references**: REQ-PROD-033, REQ-PROD-032

#### REQ-UX-032 — A consistent voice, with plain and actionable errors
- **statement**: UI copy follows one voice — sentence case, verbs on action buttons, consistent
  product terms — and error messages are in plain language that says what happened and what to do,
  never exposing a system or error code.
- **acceptance_criteria**: Inspect the buttons on several screens — action buttons read as verbs
  ("Post", "Follow", "Save changes") in sentence case, not "OK"/"Submit" or Title Case. Force an
  error (an invalid form entry, then a failed submit); the message is in plain language with a way
  forward, and no raw error code, stack text, or internal identifier is shown to the reader.
- **release_gating**: true
- **references**: REQ-PROD-034, REQ-PROD-030, REQ-PROD-033

#### REQ-UX-033 — Screens are built from the named component vocabulary, not one-off controls
- **statement**: Every screen is assembled from the named components in this library — one chirp card,
  one compose box, one button set, one text field, one dialog, and so on — rather than each screen
  introducing its own version of the same thing.
- **acceptance_criteria**: On three screens of different archetypes (for example the home timeline, a
  profile, and a settings sub-section), a chirp is rendered the same way, buttons share one shape and
  one primary colour, and text fields and error messages share one treatment; no screen shows a
  bespoke control that does the job of a library component but looks or behaves differently.
- **release_gating**: true
- **references**: REQ-PROD-033

#### REQ-UX-034 — Every component is built from the published design tokens
- **statement**: Each component draws its colour, type, spacing, radii, elevation, and motion from the
  published token set by name; no component introduces a raw hex, a one-off size, or a bespoke
  duration.
- **acceptance_criteria**: Inspect a chirp card, a primary button, a text field, and a dialog on the
  live product in both offered themes; their colours match the token roles (a primary action is the
  one `color.primary`, a destructive one `color.danger`), their text sizes match the type scale, and
  nothing renders a colour or size that is absent from the token set.
- **release_gating**: true
- **references**: REQ-PROD-033

#### REQ-UX-035 — The chirp card and compose box behave to the product's core-loop contract
- **statement**: The compose box enforces the published character limit with a visible
  remaining-character affordance and rejects an empty or over-limit chirp; the chirp card shows the
  author, handle, relative timestamp, chirp text, and the like, reply, and more-actions controls, and
  shows the active-like state with a filled icon and label, not colour alone.
- **acceptance_criteria**: Open compose, type past the limit — the counter turns to the danger
  treatment and Post is disabled; clear to an empty draft — Post is disabled and nothing posts; post a
  valid chirp — it appears as a chirp card with avatar, author, handle, relative time, text, and the
  action row. Like a chirp — it shows a filled heart and a "Liked" label together, and is still
  identifiable as liked in greyscale.
- **release_gating**: true
- **references**: REQ-PROD-001, REQ-PROD-005

#### REQ-UX-036 — Every list has designed empty, loading, and load-failure states
- **statement**: Lists are rendered by the list component, which shows a loading skeleton on first
  load, an explanatory empty block when there is nothing to show, and an inline retry when a page
  fails — never a blank area, a spinner on a blank screen, or a raw failure.
- **acceptance_criteria**: Reach a brand-new account's empty timeline — it shows an explanatory block
  with a next step, not a blank page. Open a list while it loads — it shows skeleton rows at the real
  item size, not a spinner on blank. Force a page-load failure — the list shows an inline message with
  a retry in place of the missing items, keeping any items already shown.
- **release_gating**: true
- **references**: REQ-PROD-002, REQ-PROD-033

#### REQ-UX-037 — Forms validate inline at the field level, in plain language
- **statement**: Form inputs use the text-field component with field-level validation and an inline
  error in the danger role; error messages say what happened and what to do, never a system or error
  code, and an input that needs a server check shows a pending state before resolving.
- **acceptance_criteria**: On sign-up, submit a handle already in use — the error appears against the
  handle field in plain words naming the handle, not in a distant summary and not as a code; submit a
  mis-formatted handle — a field-level validation error explains the rule; while a handle is being
  checked for availability, the field shows a pending state rather than flicking prematurely to valid
  or invalid.
- **release_gating**: true
- **references**: REQ-PROD-009, REQ-PROD-001

#### REQ-UX-038 — Consequential actions are confirmed through the dialog confirmation variant
- **statement**: Every destructive or irreversible action is confirmed through the dialog confirmation
  variant that names the consequence and offers a cancel; the account-deletion confirmation also
  states what is and is not kept and points to data export first.
- **acceptance_criteria**: Initiate deleting a chirp, blocking an account, and deleting the account;
  each presents a confirmation that names what will happen and offers a plain cancel, and nothing
  changes on cancel. The account-deletion confirmation additionally states what is and is not retained
  and points to export before proceeding.
- **release_gating**: true
- **references**: REQ-PROD-022

#### REQ-UX-039 — The result of every action is shown through the feedback components
- **statement**: A completed action is confirmed with a brief, non-blocking toast or inline feedback;
  an in-flight action shows its control's busy state; a failed in-place action shows inline feedback at
  the control and preserves any in-progress work rather than failing silently or discarding a draft.
- **acceptance_criteria**: Publish a chirp, follow an account, and save a setting — each shows brief,
  non-blocking confirmation. Trigger an action and observe its control while it runs — the control
  shows a busy state and is not re-triggerable. Force an in-place failure (act while offline, or a
  transient server error while posting) — the failure is shown at the control with a way to retry and
  the drafted text is not lost.
- **release_gating**: true
- **references**: REQ-PROD-001, REQ-PROD-034

#### REQ-UX-040 — A first-visit Landing that explains chirp without an account
- **statement**: A person with no session reaches a Landing that explains what chirp is and offers a
  clear path to sign up and to sign in, without being required to sign in first, and that shows a
  sample of public content where one is available.
- **acceptance_criteria**: With no session, open the site root. You see an explanation of what chirp
  is and both a Sign up and a Sign in action, reachable without signing in. Where a public sample is
  served, real public chirps appear; where none is served, the Landing still explains chirp and offers
  both actions rather than a blank area.
- **release_gating**: true
- **references**: REQ-PROD-030, REQ-PROD-031

#### REQ-UX-041 — Sign-up creates an account from only what it needs
- **statement**: The Sign up screen collects a handle, a display name, an optional bio, the credential,
  and the contact the account requires — and nothing extraneous — and creates the account on submit.
- **acceptance_criteria**: Open Sign up. The form requests a handle, a display name, an optional bio
  (marked optional), a credential, and a contact, and no fields beyond these. Complete it with valid
  values and submit; an account is created and you are signed in.
- **release_gating**: true
- **references**: REQ-PROD-008, REQ-PROD-031

#### REQ-UX-042 — Handle format and uniqueness are reported against the field
- **statement**: Sign-up checks the handle for format (up to 15 letters, numbers, or underscores) and
  uniqueness, and reports a taken or ill-formed handle against the handle field so the person can fix
  it before submitting.
- **acceptance_criteria**: Enter a handle already in use; an inline message against the field names the
  conflict. Enter a handle with a space, a disallowed character, or more than 15 characters; an inline
  field-level message explains the rule. Enter a valid, unused handle; the field resolves to accepted.
- **release_gating**: true
- **references**: REQ-PROD-009

#### REQ-UX-043 — Sign-up captures a contact to be verified
- **statement**: Sign-up captures a point of contact and makes clear it will be verified before the
  product relies on it for recovery or notifications, showing the person a pending-verification state.
- **acceptance_criteria**: During sign-up, provide a contact; the screen states the contact must be
  confirmed and shows that verification is pending (or prompts the verification step), rather than
  treating an unverified contact as confirmed.
- **release_gating**: true
- **references**: REQ-PROD-012

#### REQ-UX-044 — Sign-in proves the account is the person's own
- **statement**: Signing in requires the credential set at sign-up or recovery; presenting a known
  handle alone does not grant a session.
- **acceptance_criteria**: Enter another account's handle with no credential (or a wrong one); no
  session is granted. Enter your own handle with the correct credential; you are signed in.
- **release_gating**: true
- **references**: REQ-PROD-010

#### REQ-UX-045 — Sign-in distinguishes no-such-account from wrong-credential
- **statement**: Sign-in lets a person tell "no account with that handle" from "the credential did not
  match", enough to distinguish a typo from a wrong secret, without disclosing which accounts exist
  beyond what the product already makes public.
- **acceptance_criteria**: Sign in with a handle that does not exist; the result tells you the account
  was not found and offers a path to Sign up. Sign in with a real handle and a wrong credential; the
  result is a failed sign-in you can retry or recover from, offering a path to Recover access. Neither
  message reveals account existence beyond what a public profile already shows, and neither shows a
  system code.
- **release_gating**: true
- **references**: REQ-PROD-011

#### REQ-UX-046 — Recover access starts self-service recovery through the verified contact
- **statement**: A person who cannot sign in can start recovery themselves from a Recover access
  screen that runs through the account's verified contact, and the screen's response does not reveal
  whether a given contact is on an account.
- **acceptance_criteria**: Open Recover access and submit the verified contact; the screen confirms, in
  wording that does not disclose account existence, that a recovery step has been sent if the contact
  matches an account. Submit a contact not on any account; the response is the same neutral
  acknowledgement, not a "not found" error.
- **release_gating**: true
- **references**: REQ-PROD-013, REQ-PROD-012

#### REQ-UX-047 — Set new credential completes recovery and returns to sign-in
- **statement**: Following the recovery step opens a Set new credential screen that lets the person set
  a new credential and then sign in with it; an expired or used recovery link is explained with a way
  to start recovery again rather than a dead end.
- **acceptance_criteria**: Follow a valid recovery link; set a new credential; you are routed to Sign
  in and can sign in with it. Follow an expired or already-used recovery link; the screen explains the
  link is no longer valid and offers a path back to Recover access.
- **release_gating**: true
- **references**: REQ-PROD-013

#### REQ-UX-048 — Sign-out confirmation ends the session, optionally on all devices
- **statement**: Signing out is confirmed before it proceeds, ends the current session on the server,
  and offers to end sessions on all other devices; a failed sign-out is reported and leaves the session
  intact rather than appearing to succeed.
- **acceptance_criteria**: From the account menu, choose sign out; a confirmation names the consequence
  and offers a cancel and an option to end other devices' sessions. Confirm; you land on the signed-out
  Landing and the prior session can no longer be used. Cancel; nothing changes. Force the sign-out to
  fail; the confirmation reports the failure and you remain signed in.
- **release_gating**: true
- **references**: REQ-PROD-014

#### REQ-UX-049 — Each entry/auth screen has designed empty, error, and loading states
- **statement**: The Landing, Sign up, Sign in, Recover access, Set new credential, and Sign out
  surfaces each present a designed state for empty, loading, and error — explained rather than shown as
  a blank, a hang, or a silent failure — using the component library's own named states.
- **acceptance_criteria**: On each of the six surfaces, reach its empty or initial state, a loading
  state (submit or fetch in flight), and a failure (an invalid submission, a failed request, or an
  expired recovery link). Each presents something the person can read and act on — a field-level
  message, a busy control, a skeleton, or an explained error with a next step — not a blank area or an
  unexplained failure. Where a state does not apply (the Sign out Dialog has no empty state), that is
  stated.
- **release_gating**: true
- **references**: REQ-PROD-033, REQ-PROD-034

#### REQ-UX-050 — The entry/auth screens land the person on the right place and do not leak back
- **statement**: A completed sign-up lands the person on onboarding; sign-in returns a person routed
  from a gated screen to that screen; signing out returns to the signed-out Landing with no back-leak
  into protected content; and Sign up, Sign in, and Recover access each link to the others.
- **acceptance_criteria**: Complete a fresh sign-up; you land on the onboarding screen, not a sign-in
  prompt or a blank page. With no session, open a signed-in-only screen, get routed to Sign in, and
  sign in; you land on the screen you asked for. Sign out and press back; you do not see the protected
  screen's content. On Sign in, Sign up, and Recover access, find and follow a link to each of the
  others.
- **release_gating**: true — the sign-up→onboarding landing and the sign-out no-leak are release-gating
  (REQ-PROD-032, REQ-PROD-014); the three-way interlinks ride the non-gating REQ-UX-015 and a first
  release may omit them provided Recover access stays reachable from Help & support.
- **references**: REQ-PROD-032, REQ-PROD-014, REQ-PROD-034

#### REQ-UX-051 — The home timeline reads own and followed chirps, newest first, in bounded pages
- **statement**: The signed-in home timeline shows the person's own chirps and those of the accounts
  they follow, ordered newest first, loaded in bounded pages with an explicit way to load older
  chirps — never as a single unbounded load — inside the persistent signed-in shell with Home marked
  as the current destination.
- **acceptance_criteria**: Signed in and following at least one account that has posted, open `/`: the
  timeline shows recent chirps from you and the people you follow, newest first, as a bounded page,
  with the Home destination marked in the shell. Use the load-older control; older chirps load as a
  further bounded page and the product does not attempt to fetch every chirp at once.
- **release_gating**: true
- **references**: REQ-PROD-002

#### REQ-UX-052 — A catch-up affordance brings in chirps posted since the last look
- **statement**: When chirps have been posted since the person last viewed the timeline, the screen
  offers a catch-up control ("N new chirps") at the top of the list that pulls those chirps in as a
  bounded step and returns the reader to the newest, so a long absence is a bounded catch-up rather
  than an endless scroll; returning from a chirp's conversation keeps the reader's place in the list.
- **acceptance_criteria**: Open the timeline, then have another account you follow post while it is
  open: a catch-up control stating how many new chirps are available appears at the top. Select it;
  the new chirps load in at the top without an unbounded scroll. Open a chirp from the list and press
  back; you return to the timeline at the position you left, not at the top.
- **release_gating**: true
- **references**: REQ-PROD-002

#### REQ-UX-053 — The timeline has designed empty, error, and loading states
- **statement**: The timeline presents a designed empty state (a brand-new or emptied timeline
  explains why it is empty and points to Explore and to composing a first chirp, never a blank), a
  designed error state (a failed first page, a failed load-older, or a failed catch-up is shown in
  place with a retry and already-loaded chirps are not discarded), and a designed loading state (a
  first load shows skeleton rows at real card dimensions, not a spinner on a blank screen; loading
  older shows a bounded spinner at the boundary; the navigation frame never blanks and nothing spins
  forever).
- **acceptance_criteria**: With an account that follows no one and has posted nothing, open the
  timeline: it explains the empty state and offers a route to Explore and to composing, rather than a
  blank area. Force the first page to fail: a plain-words message with a retry is shown in place of
  the list, visibly different from the empty state. Force a load-older to fail: the failure shows at
  the boundary and the chirps already on screen remain. Open the timeline on a slow connection: the
  content region shows skeleton rows at the size of real chirps while the shell stays visible.
- **release_gating**: true
- **references**: REQ-PROD-002

#### REQ-UX-054 — The 280-character limit is enforced while composing, before submission
- **statement**: The compose surface shows a live remaining-character count that warns as the published
  280-character limit nears and marks the draft as over the limit once it is exceeded, and it keeps a
  person from posting an empty or over-limit chirp — the primary action is disabled while the draft is
  empty or over the limit, and an empty or over-limit submit reached by any means is rejected with no
  chirp created.
- **acceptance_criteria**: Signed in, open the compose surface and type. The remaining-character count
  decreases as you type and changes to a warning appearance as you approach 280. Type past 280: the
  count shows an over-limit state, the excess is distinguished in the field, and the "Post" action is
  not activatable. Empty the field: "Post" is again not activatable. Reduce a valid draft and post it:
  the chirp is published. At no point can an empty or over-limit chirp be posted.
- **release_gating**: true
- **references**: REQ-PROD-001

#### REQ-UX-055 — Publishing locks against double-posting, confirms on success, and keeps the draft on failure
- **statement**: While a publish is in flight the primary action shows a busy state and the draft field
  is briefly locked so the same chirp cannot be posted twice; on success the chirp is published, the
  field clears, and a brief non-blocking confirmation is shown; on failure the draft is kept exactly as
  written, an inline error states what happened with a way to retry, and the person can post again
  without retyping.
- **acceptance_criteria**: Compose a valid chirp and post it: the action shows a busy state, the field
  does not accept a second submit of the same draft during the publish, and on success the chirp
  appears, the field clears, and a brief confirmation is shown. Force a publish failure (for example
  post while offline): the draft text remains in the field, an inline error explains the failure with a
  retry, and retrying with connectivity restored posts the chirp. No successful post is duplicated and
  no failed post loses the draft.
- **release_gating**: true
- **references**: REQ-PROD-001

#### REQ-UX-056 — One compose surface in three placements, with a context-labelled action and an empty state
- **statement**: The compose surface is the one ComposeBox component, presented inline at the top of
  the home timeline, as the body of the Compose dialog (at `/compose` and as an overlay dialog opened
  from the persistent compose affordance), and in the reply position of the Chirp / conversation
  screen; its primary action is labelled for the placement ("Post" when writing a chirp, "Reply" when
  answering one); and its empty state shows a placeholder prompt with the counter at the full budget
  and the action disabled, in every placement.
- **acceptance_criteria**: Signed in, reach the compose surface three ways — inline at the top of the
  home timeline, by opening the Compose dialog from the shell and by opening `/compose` directly, and
  in the reply position on a chirp's conversation screen. In each, an untouched field shows a
  placeholder prompt, the remaining-character count at 280, and a disabled primary action; the action
  reads "Post" in the timeline and dialog and "Reply" in the conversation. Dismiss the Compose dialog
  with text in the field and reopen it: the draft is not lost.
- **release_gating**: true
- **references**: REQ-PROD-001, REQ-PROD-003

#### REQ-UX-057 — A chirp is read in the conversation it belongs to
- **statement**: The Chirp / conversation screen presents a chirp together with the chirp it answers (when
  it is a reply) and its replies, each reply shown as answering its parent, so the exchange is read in
  context rather than as a flat, unrelated list. The replies load in bounded pages, oldest-first, with a way
  to load older replies.
- **acceptance_criteria**: Open a chirp that is a reply and that has replies of its own. The chirp it
  answers is shown above it with a "Replying to" relation and is itself openable; the focused chirp is
  shown as the subject of the screen; its replies appear below in chronological order with a reply count,
  and a long thread offers a way to load older replies rather than loading everything at once.
- **release_gating**: false — reading replies as a navigable conversation rides REQ-PROD-004, which the
  release bar does not gate; the parent reference itself is already recorded by the gating reply requirement
  (REQ-UX-058 / REQ-PROD-003), so a first release may ship the reply action without the navigable
  conversation read.
- **references**: REQ-PROD-004

#### REQ-UX-058 — A reply is written from the conversation, and a reply to a gone chirp is reported
- **statement**: A signed-in person can write and publish a reply from the conversation screen's reply slot
  using the shared compose control; the reply records the chirp it answers and appears in the conversation.
  Replying to a chirp that no longer exists is reported as not-found, with the draft kept, rather than
  appearing to send. A signed-out visitor sees a path to sign in or sign up in place of the reply control
  rather than a dead or silent one.
- **acceptance_criteria**: Signed in on a chirp's conversation, write a reply within the character limit and
  publish it; it appears in the replies as answering that chirp and its publication is confirmed. Attempt to
  reply to a chirp that has been deleted; the product reports not-found at the compose control and keeps the
  drafted text rather than appearing to succeed. Open the same conversation with no session; in place of the
  reply control there is a visible way to sign in or sign up to reply.
- **release_gating**: true
- **references**: REQ-PROD-003

#### REQ-UX-059 — Like is an in-place, idempotent toggle with an accurate count
- **statement**: The like control on every **ChirpCard** is an in-place toggle with two states —
  not-liked and liked — where the liked state is shown by colour together with a filled heart and a
  "Liked" label (never colour alone); liking is idempotent and the like count shown reflects the
  resolved result; a failed like or unlike reverts the optimistic change and reports at the control,
  and an in-flight change shows the control's busy state.
- **acceptance_criteria**: On any chirp, activate like: the control shows the liked state (filled
  heart, like colour, and a "Liked" label — not colour alone) and the count rises by one. Activate like
  again on the now-liked chirp, or reload the screen: the count does not rise a second time. Unlike it:
  the control returns to not-liked and the count returns to its earlier value. Force a like to fail
  (for example act while offline): the control returns to its prior state, the count is unchanged, and
  a danger message appears at the control with a way to retry — the count never shows a like that did
  not take. Open the same chirp in the timeline, in its conversation, and on the author's profile: the
  like control looks and behaves the same in each.
- **release_gating**: true
- **references**: REQ-PROD-005

#### REQ-UX-060 — Follow is an in-place, idempotent control that cannot follow yourself
- **statement**: The follow control on a **ProfileHeader** and on the person rows of the discovery and
  list screens is an in-place control with two states — "Follow" (a **PrimaryButton**) and "Following"
  (a **SecondaryButton**); following is idempotent; a person's own identity never presents a Follow
  control (the own-profile action is "Edit profile") so a self-follow cannot be started, and any
  attempt is refused and shown at the control; a failed follow or unfollow reverts and reports at the
  control, and an in-flight change shows the control's busy state.
- **acceptance_criteria**: On another person's profile, activate "Follow": the control becomes
  "Following" and that account's new chirps appear in your home timeline. Activate it again, or reload:
  nothing is duplicated — one follow holds and the control stays "Following". Unfollow: the control
  returns to "Follow" and their chirps leave your timeline. Open your own profile: the context action
  is "Edit profile", never "Follow", and no screen offers a control to follow your own account. Force a
  follow to fail: the control returns to its prior label, any follower count is unchanged, and a danger
  message appears at the control with a way to retry. Find the follow control on a profile header and on
  a person row in Explore or a followers/following list: it looks and behaves the same in each.
- **release_gating**: true
- **references**: REQ-PROD-006

#### REQ-UX-061 — A public profile shows identity and chirps, with or without a session
- **statement**: A user's profile at `/@:handle` shows their avatar, display name, handle, bio, and
  follower and following counts, above their own chirps in newest-first bounded pages, and is reachable
  both signed-out and signed-in.
- **acceptance_criteria**: With no session, open a profile link; the header shows the handle, display
  name, bio, and the follower and following counts, and the person's chirps load below in a bounded page
  with a way to load older ones. Signed in, open the same profile and see the same content plus the follow
  control in the header.
- **release_gating**: true
- **references**: REQ-PROD-007

#### REQ-UX-062 — The owner's profile is their public face with a way to edit it
- **statement**: When the signed-in owner opens their own profile, they see the same public profile
  anyone else sees, with an "Edit profile" entry in place of the follow control that opens the edit-profile
  screen.
- **acceptance_criteria**: Signed in, open your own profile; it shows your avatar, display name, handle,
  bio, counts, and your chirps as a visitor would see them, and an "Edit profile" control that opens the
  edit-profile screen. The follow control is not shown on your own profile.
- **release_gating**: true
- **references**: REQ-PROD-007

#### REQ-UX-063 — Followers and following are addressable, paginated lists
- **statement**: A profile's follower and following counts open the `/@:handle/followers` and
  `/@:handle/following` lists, each an addressable, paginated list of people showing enough to recognise
  each account, with the follow control on each row for a signed-in viewer and a labelled way back to the
  profile.
- **acceptance_criteria**: From a profile, open the followers list by its count; it shows one row per
  follower (avatar, name, handle, bio) in a bounded page with a way to load more, a way back to the
  profile, and — signed in — a follow control per row. Open the following list the same way with the same
  behaviour. Open each list's URL directly; it lands on that list.
- **release_gating**: true
- **references**: REQ-PROD-007, REQ-PROD-006

#### REQ-UX-064 — A way to find people to follow
- **statement**: A signed-in person can find accounts to follow from within the product at `/explore`, by
  searching for people and following a result in place, without already holding a link to a profile.
- **acceptance_criteria**: Signed in and holding no external link, open Explore, search for a person, and
  follow at least one result from the list without leaving the screen; a search that matches no one shows a
  "no results" state with a way to refine, distinct from the before-you-search prompt.
- **release_gating**: false — Explore implements REQ-PROD-042, which the product requirements do not gate;
  a first release may ship with follow-by-profile and follow-from-a-reply and add discovery as a near-term
  follow-up. The release bar should decide explicitly what minimum discovery v1 includes.
- **references**: REQ-PROD-042

#### REQ-UX-065 — Onboarding explains the empty timeline and offers a way off it
- **statement**: A brand-new account landing on Onboarding (`/welcome`) sees an explanation of why the timeline
  is empty and at least one actionable route to a non-empty timeline — composing a first chirp, and, where
  discovery is present, finding people to follow — rather than a blank page. The compose route is always
  offered, so the screen meets this bar even where discovery is not yet present.
- **acceptance_criteria**: Complete a fresh sign-up; you land on `/welcome` and see text explaining the empty
  timeline and a control that opens Compose to write a first chirp. Where Explore is present, a second control
  opens it to find people to follow. At no point is the screen a blank page.
- **release_gating**: true
- **references**: REQ-PROD-032

#### REQ-UX-066 — Onboarding never blanks or dead-ends across its states
- **statement**: Onboarding's explanation and its routes render immediately and stay available across its
  loading and error states: a loading suggestions preview shows skeletons rather than blanking the screen, and
  a failed or empty suggestions preview still leaves at least the compose route in place, so the person always
  has a way forward.
- **acceptance_criteria**: Open `/welcome` and, while any suggestions preview is still loading, confirm the
  explanation and the "Write your first chirp" route are already usable and no region is blank. Force the
  suggestions preview to fail; it shows an error with retry and the compose route is still present and works.
- **release_gating**: true
- **references**: REQ-PROD-032, REQ-PROD-033

#### REQ-UX-067 — Edit profile changes display name and bio and persists them
- **statement**: A signed-in person can open Edit profile (`/settings/profile`) from the Settings hub or from
  the "Edit profile" action on their own profile, change their display name and bio, and save; the saved
  values show on their profile on the next view.
- **acceptance_criteria**: Signed in, open Edit profile from the Settings hub; change the display name and the
  bio and save. The save is confirmed. Open your own profile; the new display name and bio are shown. Repeat
  reaching Edit profile from the "Edit profile" action on your own profile.
- **release_gating**: true
- **references**: REQ-PROD-016

#### REQ-UX-068 — Edit profile validates, reports failures in place, and keeps the person's input
- **statement**: Edit profile reports a field-level problem (an empty required display name, or a rejected
  value) against the field in plain words, reports a failed save at the form without discarding edited values
  so the person can retry, and confirms a successful save — never a silent success or a silent failure, and
  never a system or error code.
- **acceptance_criteria**: Clear the display name and attempt to save; the field shows an error naming the
  problem and no save occurs. Cause a save to fail (for example act during a transient server error); the
  failure is shown at the form, your edited display name and bio are still there, and retrying saves. A
  successful save shows a confirmation.
- **release_gating**: true
- **references**: REQ-PROD-016, REQ-PROD-033, REQ-PROD-034

#### REQ-UX-069 — The settings hub lists every account sub-section, each addressable and returning to the hub
- **statement**: Signed in, a Settings hub at `/settings` presents a labelled list of every account settings
  sub-section — edit profile, account & contact, notification preferences, sessions & devices, blocked & muted,
  export your data, and delete account — each a row the person can open from the hub and by the sub-section's
  own URL, with a labelled way back to the hub from each. The delete-account row is shown as destructive and
  opens a confirmation rather than deleting on selection. Where notifications are not in the release, the
  notification-preferences row is absent; every other row is present.
- **acceptance_criteria**: Signed in, open `/settings`. The sub-sections above are listed as openable rows.
  Open each by selecting its row, and again by entering its URL directly in a fresh tab; each lands on that
  sub-section, not on a generic home. From each sub-section, use the on-screen way back and the browser back
  control; both return to the hub. Confirm the delete-account row is visibly destructive and, when selected,
  presents a confirmation that can be cancelled — selecting the row does not delete the account. If the account
  summary fails to load, confirm every sub-section row is still reachable.
- **release_gating**: true
- **references**: REQ-PROD-017

#### REQ-UX-070 — View, update, and confirm control of the verified contact
- **statement**: From the Account & contact screen (`/settings/account`), a signed-in person can see their
  current point of contact with its verification status, change it, and — because the product relies only on a
  contact it has confirmed the person controls — complete a control-you-own verification step for a new or
  unverified contact. An unverified contact is shown as a caution, not an error.
- **acceptance_criteria**: Signed in, open `/settings/account`; the current contact and a verification status
  of either "Verified" or "Not verified yet" are shown. Change the contact and save; the product confirms the
  save and shows the new contact as not-yet-verified, and offers a way to complete a step that confirms you
  control it before it is used for recovery. Confirm that an unverified contact is presented as a caution (a
  non-error treatment), and that a save failure is shown against the contact field with your typed value
  preserved to retry. A person who has not completed verification cannot have that contact treated as verified.
- **release_gating**: true
- **references**: REQ-PROD-012, REQ-PROD-017

#### REQ-UX-071 — The handle-change position is stated on the account screen and the screen matches it
- **statement**: The Account & contact screen states the product's position on whether and how the account
  handle can change after sign-up, shown with the handle, and the screen behaves consistently with that stated
  position.
- **acceptance_criteria**: On `/settings/account`, find the product's stated position on handle changes
  presented with your `@handle`. Confirm the screen's behaviour matches the stated position: either a change
  control is offered and works, or the handle is stated as fixed and the screen offers no path that changes it.
  If stated as fixed, confirm no control on the screen changes the handle.
- **release_gating**: false — rides REQ-PROD-018, a decision-and-consistency requirement; a first release may
  state the handle as fixed provided that position is stated on the screen.
- **references**: REQ-PROD-018, REQ-PROD-017

#### REQ-UX-072 — Per-category notification preferences, saved immediately
- **statement**: On the Notification preferences screen (`/settings/notifications`) a signed-in person can
  choose, for each activity category — replies, follows, and likes — whether that category sends a
  notification. A change takes effect immediately and is confirmed on its row with no separate save; turning a
  category off stops notifications of that category and turning it on resumes them. If a change cannot be saved,
  the control returns to its previous state and says so.
- **acceptance_criteria**: Signed in, open `/settings/notifications` directly by its URL. The screen lists
  Replies, Follows, and Likes, each with an on/off control that shows its current state, and offers a way back
  to the settings hub. Turn Likes off; the row confirms the change in place, with no separate Save step. Have
  another account both like and reply to one of your chirps; a reply notification appears in the Notifications
  list and no like notification does. Turn Likes back on; a later like produces a notification again. With
  saving forced to fail, flip a toggle: it returns to its previous position and shows a plain message that the
  setting is unchanged.
- **release_gating**: false — notification preferences ride REQ-PROD-029, which the release bar does not gate; a
  first release may ship without notifications, and these preferences ship when notifications ship.
- **references**: REQ-PROD-029, REQ-PROD-017

#### REQ-UX-073 — See active sessions and end another device's session
- **statement**: A signed-in person can open Sessions & devices and see each place the account is currently
  signed in — each with enough to recognise it (device and browser, approximate location, last-active time) and
  the current session clearly marked — and can end any other session through a confirmation that names the
  effect, after which that session can no longer be used on its device.
- **acceptance_criteria**: Signed in on two devices, open `/settings/sessions`; both sessions are listed, the
  one you are using is marked as the current device, and each other session shows its device, an approximate
  location, and a last-active time. Choose End session on the other device; a confirmation states what will
  happen and offers a cancel. Confirm it: the session leaves the list, the result is shown, and on the other
  device the next action on that session is rejected and requires a fresh sign-in.
- **release_gating**: true
- **references**: REQ-PROD-017, REQ-PROD-014

#### REQ-UX-074 — Sign out ends the session server-side and does not leak back
- **statement**: A signed-in person can sign out from Sessions & devices; sign-out ends the current session
  server-side so it can no longer be used, returns the person to the signed-out landing, and leaves no
  protected screen reachable with the browser back control. Where the account is signed in elsewhere, the
  sign-out confirmation offers to end those other sessions too.
- **acceptance_criteria**: Signed in, open `/settings/sessions` and choose Sign out; a confirmation states you
  will be signed out and, when other sessions exist, offers to end them too. Confirm it: you land on the
  signed-out landing; pressing browser back does not re-expose the settings screen; and replaying the request
  from the session that was in use is rejected, so the next person at that device is not signed in as you.
- **release_gating**: true
- **references**: REQ-PROD-014

#### REQ-UX-075 — A person can request and obtain their own data in a portable form
- **statement**: From a dedicated export screen, a signed-in person can request their data and obtain their
  chirps, profile, and follow list in a form they can open outside chirp, with the request, the preparing
  state, and the obtainable result each shown.
- **acceptance_criteria**: Signed in, open the export screen; it names what the export contains and offers a
  single request action. Request the export; the screen shows a preparing state (not a frozen or blank screen)
  and, when ready, offers a way to download the file. Open the downloaded file outside chirp; it contains your
  chirps, your profile, and who you follow. Force a preparation failure; the screen says so in plain words and
  lets you retry, and offers no broken file.
- **release_gating**: true
- **references**: REQ-PROD-019

#### REQ-UX-076 — Deletion is confirmed with its consequence, its retained/not-retained outcome, and a pointer to export
- **statement**: The delete-account confirmation names the consequence plainly, states what is and is not
  retained per the product's defined deletion outcome (including what becomes of replied-to or liked chirps and
  whether the handle frees up), and points the person to export their data first — all before any deletion
  occurs.
- **acceptance_criteria**: Signed in, open the delete-account confirmation. It states that the account and
  chirps are removed and that this cannot be undone; it shows what is removed, what may remain, and what
  happens to the handle, matching the product's stated deletion outcome; and it offers a way to export your
  data first. None of this requires having deleted anything to see it.
- **release_gating**: true
- **references**: REQ-PROD-021, REQ-PROD-022

#### REQ-UX-077 — Deletion proceeds only on explicit confirmation, and cancelling leaves everything intact
- **statement**: Account deletion proceeds only after the person explicitly confirms (the destructive action
  stays disabled until confirmation is given); on deletion the person is signed out to the signed-out shell, and
  cancelling at any point leaves the account and its content unchanged.
- **acceptance_criteria**: Open the delete-account confirmation; the delete action is unavailable until you give
  the explicit confirmation it asks for. Cancel; you return to settings and the account still signs in with its
  content intact. Give the confirmation and delete; afterward the account can no longer sign in and you are
  returned to the signed-out landing, and the browser back control does not re-expose the deleted account's
  screens.
- **release_gating**: true
- **references**: REQ-PROD-020, REQ-PROD-022

#### REQ-UX-078 — A notifications surface that shows directed activity and clears when seen
- **statement**: The signed-in Notifications screen shows activity directed at the person — replies,
  follows, and likes — newest first in bounded pages, lets them act on an item in place, and opening
  it marks the shown activity seen and clears the shell's unseen-activity indicator.
- **acceptance_criteria**: Have another account reply to, follow, and like you; the shell's
  Notifications entry shows an unseen indicator. Open Notifications; the three activities appear
  newest-first, a reply can be opened in its conversation and a follow followed back from the row, and
  the unseen indicator clears. Confirm the list loads a bounded page with a way to load older items,
  not every item at once.
- **release_gating**: false — notifications ride REQ-PROD-029, which the release bar does not gate; the
  surface and its indicator ship when notifications ship.
- **references**: REQ-PROD-029

#### REQ-UX-079 — Reporting a chirp or account, acknowledged as received
- **statement**: A signed-in person can report a chirp or an account against a stated rule, optionally
  add detail, and on submission sees that the report was received; a report is never silently
  swallowed.
- **acceptance_criteria**: From a chirp's and from an account's more-actions menu, open the report
  dialog, choose a reason drawn from the rules, and submit; each submission is acknowledged as received
  (an in-product confirmation). Open the dialog and attempt to submit with no reason chosen; submission
  is prevented until a reason is chosen.
- **release_gating**: true
- **references**: REQ-PROD-025, REQ-PROD-026

#### REQ-UX-080 — Block and mute, each confirmed with its effect stated
- **statement**: A signed-in person can block or mute another account through a confirmation that
  states plainly what the action does before it takes, with a way to cancel, and a confirmation that it
  took.
- **acceptance_criteria**: From an account's more-actions menu, choose block; a confirmation states the
  effect (they can't see or interact with you; you won't see them; they aren't told) and offers cancel.
  Cancel; nothing changes. Choose mute; a confirmation states its distinct effect. Confirm either; it
  takes and is confirmed, and the account appears on the blocked & muted screen.
- **release_gating**: false — block/mute ride REQ-PROD-027, which the release bar does not gate;
  reporting (REQ-UX-079) is the gated safety floor and these controls ship as the expected near-term
  addition.
- **references**: REQ-PROD-027

#### REQ-UX-081 — Managing the accounts a person has blocked or muted
- **statement**: A signed-in person can see the accounts they have blocked and the accounts they have
  muted, told apart, and can unblock or unmute any of them, with the change taking effect immediately.
- **acceptance_criteria**: Block one account and mute another, then open the blocked & muted settings
  screen; each appears under the right heading. Unblock the first and unmute the second; each leaves its
  list and the change is confirmed. With none blocked or muted, the screen explains the lists are empty
  rather than showing blank rows.
- **release_gating**: false — the management screen serves the block/mute controls of REQ-PROD-027,
  which the release bar does not gate; it ships with those controls.
- **references**: REQ-PROD-027

#### REQ-UX-082 — A user-facing help & support entry reachable from both shells
- **statement**: A help & support screen, reachable from both the signed-out and the signed-in shell,
  gives a person the way to recover access, to report a problem, and to contact the operator for what
  self-service does not cover.
- **acceptance_criteria**: With no session, open Help & support from the signed-out shell; a path to
  recover access and a way to contact support are present. Signed in, open it from the account menu;
  the same entries plus account management are present, and "Can't get in?" reaches recovery and
  "Contact support" reaches the operator contact path.
- **release_gating**: true
- **references**: REQ-PROD-028, REQ-PROD-013

#### REQ-UX-083 — Stated rules a report can point to
- **statement**: The product states its rules — what is and is not allowed — as named categories a
  person can read and a report can reference, reachable from both shells.
- **acceptance_criteria**: Open Rules & terms from the shell; the rules are present as named
  categories. Open the report dialog; its reasons correspond to those stated rule categories, so a
  report points at a stated rule rather than an unexplained choice.
- **release_gating**: true
- **references**: REQ-PROD-026

#### REQ-UX-084 — A not-found screen that explains and routes back
- **statement**: An unresolved or removed resource — a mistyped path, a deleted chirp, an unavailable
  account — lands the person on a not-found screen that explains the absence without exposing private
  detail and offers a way back to a known place.
- **acceptance_criteria**: Open a URL that does not resolve; a not-found screen explains the resource
  is missing and offers a way to a known place (the timeline when signed in, the landing when signed
  out). Open a removed chirp's or an unavailable account's address; the screen states it is unavailable
  rather than showing a broken page, consistent with the in-place tombstone shown elsewhere.
- **release_gating**: true
- **references**: REQ-PROD-033

#### REQ-UX-085 — Every safety and notification surface presents considered empty, error, and loading states
- **statement**: Each surface in this part presents a considered empty, error, and loading state
  (composed from the component library's named states and the product-wide patterns), or states plainly
  where a state does not apply, rather than a blank area or a silent failure.
- **acceptance_criteria**: On the Notifications screen force each of an empty list, a failed page load,
  and a first load; each shows an understandable state (an explained empty block, an inline error with
  retry, skeleton rows) rather than a blank or a hang. On the report and block/mute dialogs force a
  failed submission; each keeps the dialog open with its input preserved and an inline error. On the
  blocked & muted screen with nothing blocked, an explained empty state appears.
- **release_gating**: true
- **references**: REQ-PROD-033, REQ-PROD-034

---

*Composition note.* Every screen above is a Screen-inventory screen referred to by name, assembled
from Component-library components by name, using those components' own named states; no screen or
component outside the frozen vocabularies is introduced. The Notifications row is the "row kind the
screen needs" that **ListView** provides for, composed from existing components, not a new component.
The notification **preferences** sub-section, the **Settings** hub, and the **Recover access** screen
are owned by sibling parts and are cross-referenced here, not redesigned. The per-screen empty, error,
and loading states compose the product-wide patterns owned by the `empty_error_loading_states` section
and are left for that section and the subtree synthesis to confirm consistent across the assembled
whole.

#### REQ-UX-086 — Empty surfaces are explained, never blank
- **statement**: Every surface that has nothing to show presents a designed empty treatment — a
  short explanation of why it is empty and at least one next step — rather than a blank area, and a
  genuinely-empty surface is visibly distinct from one that failed to load.
- **acceptance_criteria**: Reach a brand-new home timeline, an empty notifications surface, and an
  empty followers list; each shows an explanation and at least one next step (for example, the new
  timeline points to Explore and to composing a first chirp), not a blank region. Then cause a
  surface to fail loading; it shows a distinct "couldn't load — try again" treatment with a retry,
  clearly different from the empty treatment.
- **release_gating**: true
- **references**: REQ-PROD-033

#### REQ-UX-087 — Loading shows the shape of what is coming and never runs forever
- **statement**: A first load shows skeletons at the real dimensions of the content, an in-place or
  incremental load shows a bounded indicator at the point of the action, the persistent navigation
  frame never blanks, and no load spins indefinitely — a load that does not resolve becomes an error
  with a way forward.
- **acceptance_criteria**: Open the home timeline on a slow connection; the content area shows
  skeleton rows at the size of real chirps rather than a spinner on a blank page, and the navigation
  frame stays visible. Load older items; a bounded spinner appears at the list boundary while the
  already-loaded items remain on screen. Force a load to fail; it stops and offers a retry rather
  than spinning without end.
- **release_gating**: true
- **references**: REQ-PROD-033

#### REQ-UX-088 — Failures appear where the action was taken, in plain words, with a way forward
- **statement**: Every failure is reported at the point of the action — field, control, list,
  surface, dialog, or a toast where there is no on-screen control — in plain language with no system
  or error code, never as a silent no-op, and always leaves the person a way to retry or correct
  with their input preserved.
- **acceptance_criteria**: Trigger a field error (for example a taken handle at sign-up), a failed
  in-place action (for example a like or follow that fails), and a failed list page; each shows a
  plain-words message at the point of the problem with a retry or correction, no error code and no
  blank, and the optimistic in-place action reverts rather than showing a result that did not
  happen.
- **release_gating**: true
- **references**: REQ-PROD-033

#### REQ-UX-089 — An expired session and a transient error both recover without losing work
- **statement**: An action taken with an expired or invalid session routes the person to sign-in
  with the reason stated and returns them to where they were afterwards, and a transient error is
  retryable in place; in both cases in-progress work such as a drafted chirp is preserved. This
  composes against the information architecture's session-expiry convention (REQ-UX-008) and
  auth-gated return (REQ-UX-007) and does not redefine them.
- **acceptance_criteria**: With a session expired, draft a chirp and try to post; you are routed to
  sign-in with a stated reason rather than a silent failure, and after signing in your drafted text
  is intact and you are back where you were. Separately, cause a transient server error while
  posting; you can retry in place and the post succeeds without re-typing the draft.
- **release_gating**: true
- **references**: REQ-PROD-034

#### REQ-UX-090 — The core loop is fully usable on a phone
- **statement**: On a phone-sized viewport in portrait, a signed-in person can complete the whole core
  loop — read the home timeline, compose and post a chirp, reply, like and unlike, follow and unfollow,
  and view a public profile — with no broken layout, no horizontally-clipped content, and no control that
  is off-screen or too small to operate.
- **acceptance_criteria**: On a 360×640 CSS-px portrait viewport, signed in: open and scroll the home
  timeline; open Compose and post a chirp; reply to a chirp; like then unlike a chirp; follow then
  unfollow an account; open a public profile. Every step completes, and at no step is there horizontal
  page scrolling, a control hidden off-screen, or an action target that cannot be tapped.
- **release_gating**: true
- **references**: REQ-PROD-035, REQ-PROD-001, REQ-PROD-002

#### REQ-UX-091 — One reading column, reflowed not rescaled, across viewports
- **statement**: The reading column is a single column at every viewport — filling the width inside the
  phone gutter on a compact screen and held to the 600px measure, centred, on wider screens — so content
  reflows to the available width and never requires horizontal scrolling or pinch-zoom to read.
- **acceptance_criteria**: Open the timeline at a 360px-wide viewport and at a 1280px-wide viewport. At
  360px the column fills the width inside a side gutter with no horizontal scrollbar; at 1280px the
  reading column is about 600px wide and centred, not stretched across the whole window. Neither width
  requires zooming or sideways scrolling to read a chirp.
- **release_gating**: true
- **references**: REQ-PROD-035, REQ-PROD-033

#### REQ-UX-092 — The signed-in navigation shell adapts and keeps every destination reachable
- **statement**: The signed-in navigation shell presents as a bottom tab bar with a slim top bar on a
  compact viewport and as a side navigation on medium and wide viewports; in every presentation Home,
  Explore, Notifications, Profile, Settings, and a Compose affordance are present and reachable, and the
  active destination is marked.
- **acceptance_criteria**: Signed in, at a phone width the primary destinations appear in a bottom bar
  reachable with the thumb and Compose is reachable without opening a menu; at a desktop width the same
  destinations appear in a persistent side navigation. At both widths, starting from the timeline, reach
  Home, Explore, Notifications, Profile, Settings, and Compose, and see the current destination marked.
- **release_gating**: true
- **references**: REQ-PROD-002, REQ-PROD-035

#### REQ-UX-093 — The signed-out shell never dead-ends at any viewport
- **statement**: On a compact viewport the signed-out navigation shell keeps the chirp wordmark, a Sign
  in entry, and a prominent Sign up entry visible without opening a menu, and keeps Rules & terms and
  Help & support reachable; a visitor on a shared public link reaches the content and a path to join at
  any width.
- **acceptance_criteria**: With no session, open the Landing and a public profile link at a phone width
  and at a desktop width. On each, Sign up and Sign in are visible without opening a menu, and Rules &
  terms and Help & support are reachable (by scrolling to the footer at minimum). No width presents an
  authentication wall or hides the join path.
- **release_gating**: true
- **references**: REQ-PROD-030, REQ-PROD-035

#### REQ-UX-094 — feed/list screens adapt without losing their controls
- **statement**: Every feed/list screen (home timeline, a chirp's conversation, notifications, search
  results, followers and following) is one column at every viewport, keeps its load-older and timeline
  catch-up affordances reachable, keeps each chirp card's reply/like/more controls tappable, and
  preserves the person's scroll place when they return from drilling into an item.
- **acceptance_criteria**: On a phone viewport, open a timeline longer than one page: the load-older
  control and the "N new chirps" catch-up are reachable without horizontal scrolling, and a chirp card's
  reply, like, and more controls are each tappable. Open a chirp, then press back; the timeline is
  restored at the prior scroll position.
- **release_gating**: true
- **references**: REQ-PROD-002, REQ-PROD-035

#### REQ-UX-095 — form screens are usable with the on-screen keyboard
- **statement**: Every form screen (sign up, sign in, recover access, set new credential, edit profile)
  stacks its text fields full-width within the gutter, keeps each field's label and its inline error with
  the field, and keeps the primary submit reachable when the on-screen keyboard is open; no form requires
  horizontal scrolling or a side-by-side layout on a phone.
- **acceptance_criteria**: On a phone viewport, open sign up and focus a field so the on-screen keyboard
  appears: the field, its label, and any error message are visible, and the submit can be reached (by
  scrolling the form) without dismissing the keyboard first. Submit an invalid field; its error shows
  with the field, not in a summary elsewhere. No field is cut off horizontally.
- **release_gating**: true
- **references**: REQ-PROD-008, REQ-PROD-035

#### REQ-UX-096 — profile and settings screens stack and keep their way back
- **statement**: On a compact viewport the profile header stacks avatar, display name, handle, bio, and
  counts with its context action (Follow, or Edit profile on one's own profile) visible, and the Settings
  hub and each sub-section are full screens with a labelled way back to the hub; a two-pane presentation
  is permitted on wide viewports but not required.
- **acceptance_criteria**: On a phone viewport open a public profile: the header's avatar, display name,
  bio, counts, and the Follow (or Edit profile) action are all visible without horizontal scrolling, with
  the chirps list in one column below. Open Settings, open a sub-section, and return: the sub-section is a
  full screen and a labelled back returns to the Settings hub.
- **release_gating**: true
- **references**: REQ-PROD-007, REQ-PROD-017, REQ-PROD-035

#### REQ-UX-097 — overlays present as full-height surfaces on a phone with their actions reachable
- **statement**: On a compact viewport the dialog's modal variant (Compose, Report) and confirmation
  variant (delete account, delete chirp, block or mute, end a session, sign out) present as a full-screen
  or bottom-sheet surface whose content and action pair are fully visible and operable — the confirm and
  the cancel/close are both reachable without being clipped — and the browser back control dismisses the
  overlay; on wider viewports the centred panel over the scrim is used.
- **acceptance_criteria**: On a phone viewport, open Compose: the compose box and its Post button are
  visible and usable with the keyboard open, and back closes it without leaving the underlying screen.
  Open a destructive confirmation: both the confirm and the cancel are fully visible and tappable, and
  neither is cut off at the screen edge.
- **release_gating**: true
- **references**: REQ-PROD-001, REQ-PROD-022, REQ-PROD-035

#### REQ-UX-098 — touch targets are large enough and reachable
- **statement**: On a touch viewport every interactive control presents a touch target of at least 44×44
  CSS px with at least 8px between adjacent targets, and the core-loop primary actions (the Compose
  affordance, a chirp card's like and reply, and primary navigation) sit within comfortable thumb reach
  rather than only at the top of a tall screen.
- **acceptance_criteria**: On a phone viewport, measure the tap targets of the Compose affordance, a chirp
  card's like and reply controls, and the primary navigation: each occupies at least 44×44 px and adjacent
  targets do not abut. The Compose affordance and primary navigation are reachable in the lower portion of
  the screen, not only at the top.
- **release_gating**: true
- **references**: REQ-PROD-035, REQ-PROD-033

#### REQ-UX-099 — no auto-zoom, no horizontal scroll, portrait and landscape
- **statement**: The product fits the viewport width at every supported size so the core loop needs
  neither pinch-zoom nor horizontal page scrolling, focusing a text field does not trigger the browser's
  automatic zoom, and the core loop works in both portrait and landscape on a phone.
- **acceptance_criteria**: On a phone viewport, walk the core loop in portrait and then in landscape: at
  no point does the page scroll horizontally or require a pinch-zoom, and focusing a sign-in or compose
  field does not zoom the page. Content reflows to the narrower dimension rather than overflowing it.
- **release_gating**: true
- **references**: REQ-PROD-035, REQ-PROD-033

#### REQ-UX-100 — A named, versioned accessibility standard: WCAG 2.1 AA
- **statement**: chirp commits to conform to WCAG 2.1 Level AA (every Level A and AA success criterion)
  across the product; the core loop and the get-in screens (sign-up, sign-in, recovery, onboarding) with
  their empty/loading/error states are the gated conformance set that must meet it for a release, and the
  same standard applies to every other screen, inheriting the non-gating flag of any surface the release bar
  itself does not gate.
- **acceptance_criteria**: Find chirp's stated accessibility standard; it names WCAG 2.1 and Level AA, not
  "accessible" in the abstract. Walk the core loop and confirm its screens are held to that standard (the
  checks in REQ-UX-118 pass over them).
- **release_gating**: true
- **references**: REQ-PROD-036

#### REQ-UX-101 — Every action is operable by keyboard alone
- **statement**: Every action available with a pointer is reachable and operable with the keyboard alone, on
  every screen archetype and both navigation shells; no feature is pointer-only.
- **acceptance_criteria**: With the pointer unused, sign in, read the timeline, open and operate a
  **ChirpCard**'s reply/like/more controls, compose and post a chirp, follow an account, open a profile, and
  operate a settings sub-section — each action completes using only the keyboard.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-001, REQ-PROD-002

#### REQ-UX-102 — Focus order is logical and focus is always visible
- **statement**: Keyboard focus moves in an order that matches each screen's reading order, the focused
  element always shows the visible focus indicator (REQ-UX-025), and when a view changes (a dialog opens, a
  route changes, new content loads) focus is moved deliberately to a sensible target rather than lost.
- **acceptance_criteria**: Tab through the timeline, a form, and a settings screen; focus moves in a sensible
  order and the focused control is always visibly indicated. Open a dialog — focus moves into it; close it —
  focus returns to the control that opened it. Change route — focus lands on the new screen, not on a
  vanished control.
- **release_gating**: true
- **references**: REQ-PROD-036

#### REQ-UX-103 — No keyboard trap; modal focus is held but escapable
- **statement**: A keyboard user can move into and out of every component; only a modal **Dialog**
  deliberately holds focus while open, and it is always escapable (Esc, a close control, and Cancel in the
  cycle) and returns focus on close.
- **acceptance_criteria**: Tab through each screen; focus never becomes stuck with no way out. Open the
  compose dialog and a confirmation dialog; Tab cycles within the open dialog, Esc closes it, and focus
  returns to the opener.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-022

#### REQ-UX-104 — A way to bypass the navigation shell to the content
- **statement**: A keyboard or screen-reader user is given a way to skip the repeated **NavShell** and reach
  a screen's main content directly, via a visible-on-focus skip control at the top of the tab order and the
  landmark/heading structure.
- **acceptance_criteria**: On a signed-in screen, Tab once from the top; a "Skip to content" control appears
  and, activated, moves focus past the nav to the main content. A screen reader can also jump to the `main`
  region directly.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-002

#### REQ-UX-105 — Controls meet a minimum target size
- **statement**: Pointer and touch targets for the product's controls are at least 24×24 CSS pixels, and the
  primary phone touch targets (nav destinations, the compose affordance, a chirp's action row) meet the
  comfortable minimum the responsive design sets.
- **acceptance_criteria**: On a phone viewport, measure the nav destinations, the compose affordance, and a
  chirp card's like/reply controls; each meets at least the 24×24 CSS-pixel minimum and the primary targets
  meet the responsive design's larger touch minimum.
- **release_gating**: false — target size is a WCAG 2.2 criterion (2.5.8) adopted ahead of a full move to
  2.2 and beyond the committed 2.1 AA floor; it is an expected mobile-ergonomics improvement a first release
  can stage.
- **references**: REQ-PROD-035, REQ-PROD-036

#### REQ-UX-106 — Every control exposes a name, a role, and its state
- **statement**: Every interactive component in the library exposes an accessible name, a correct role, and
  its current state to assistive technology, per the component-semantics mapping in this section.
- **acceptance_criteria**: With a screen reader, move through a chirp card, the compose box, a text field, a
  toggle, a button, an icon button, and a dialog; each is announced with a sensible name, the right role
  (button, switch, text box, dialog, …), and its state (a like as pressed/not, a toggle as on/off, a busy or
  disabled button as such).
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-001

#### REQ-UX-107 — Each screen exposes landmark regions
- **statement**: Every screen exposes landmark regions — a banner and the primary navigation for the nav
  shell, a single `main` for the content, and a footer region where the signed-out shell carries one — so an
  assistive-technology user can move between the parts of the page.
- **acceptance_criteria**: With a screen reader, list the landmarks on the landing, the timeline, and a
  settings sub-section; each exposes a navigation region, exactly one main region, and (on the signed-out
  shell) a footer, and the user can jump between them.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-030

#### REQ-UX-108 — One top-level heading per screen, in a nested order
- **statement**: Every screen has exactly one top-level heading naming it (matching the document title from
  the wayfinding model), with sub-headings nested in order and no skipped levels.
- **acceptance_criteria**: With a screen reader, bring up the heading list for the timeline, a profile, and a
  settings sub-section; each has a single top-level heading that names the screen and matches the browser tab
  title, and the sub-headings descend in order.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-002

#### REQ-UX-109 — Lists and feeds are exposed as navigable lists
- **statement**: The feed/list archetype (timeline, conversation, notifications, search results,
  followers/following) exposes its item structure, so a screen-reader user can move item by item and tell
  where they are in the set; the pagination and catch-up affordances are named controls.
- **acceptance_criteria**: With a screen reader on the timeline, navigate by item; each chirp is a
  distinguishable list item and the reader can tell one from the next, and the "load older" and "N new
  chirps" controls are reported as named buttons.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-002

#### REQ-UX-110 — Icon-only controls expose text labels and their active state
- **statement**: Every icon-only control exposes a text label to assistive technology, and an active or
  selected state (current nav destination, liked chirp) is exposed as a programmatic state, not inferred from
  colour or fill — composing with REQ-UX-028.
- **acceptance_criteria**: With a screen reader, focus the compose affordance, a like control, and the
  current nav destination; each reports a text name, the like reports pressed/not-pressed, and the current
  nav destination reports its selected state.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-005

#### REQ-UX-111 — Contrast meets WCAG 2.1 AA in every shipped theme
- **statement**: Text meets WCAG 1.4.3 (≥ 4.5:1, or ≥ 3:1 for large text) and meaningful control and icon
  boundaries meet WCAG 1.4.11 (≥ 3:1), in each theme the product ships — the token-level contrast fixed and
  verified by REQ-UX-022, carried here as a condition of the committed standard.
- **acceptance_criteria**: On the live product, sample body text, secondary text, link text, primary-button
  text, and a field border with a contrast tool in each shipped theme; normal text reaches ≥ 4.5:1, large
  text ≥ 3:1, and control/icon boundaries ≥ 3:1 (as REQ-UX-022 requires), confirming the AA contrast
  criteria.
- **release_gating**: true
- **references**: REQ-PROD-036

#### REQ-UX-112 — Text resizes and the page reflows without loss
- **statement**: Text can be enlarged to 200% without clipped content or unreachable controls (WCAG 1.4.4),
  the product reflows to a 320-CSS-pixel viewport without a second scroll direction or loss of content/function
  (WCAG 1.4.10), and overridden text spacing does not clip or overlap content (WCAG 1.4.12).
- **acceptance_criteria**: Zoom the browser to 200% and read the timeline and a form — no content is cut off
  and every control is still reachable. Narrow the viewport to 320 CSS pixels — content reflows into one
  column with no horizontal scrolling of the reading content. Apply a text-spacing override — nothing
  overlaps or is clipped.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-035

#### REQ-UX-113 — No fact is carried by colour alone, and no content flashes
- **statement**: Every state that uses a semantic colour also carries text, an icon, or a label so the fact
  survives without colour perception (WCAG 1.4.1, composing with REQ-UX-022), and no content flashes more
  than three times in any one second (WCAG 2.3.1).
- **acceptance_criteria**: View the core loop in greyscale or a colour-blindness simulation; a field error,
  an active like, and the current nav destination are each still identifiable without their colour. Observe
  the product through normal use; nothing flashes more than three times per second.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-033

#### REQ-UX-114 — Reduced motion is honoured with no loss of information
- **statement**: When a reader has asked their system to reduce motion, the product honours it through the
  `motion.reduced` contract (REQ-UX-029) — transitions become instant or a short fade, counts do not animate,
  and no required information is conveyed only by movement; the bounded loading indicator has an announced,
  static equivalent.
- **acceptance_criteria**: With the system reduce-motion setting on, open and close a menu and a dialog,
  trigger a toast, and load a list; motion is removed or reduced to a short fade, nothing animates
  continuously except a loading indicator that is also announced, and no information is lost.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-033

#### REQ-UX-115 — Status changes are announced without stealing focus
- **statement**: A change not caused by a direct keystroke — a chirp posted, a follow or setting saved, a
  report received, new chirps arriving behind the catch-up control — is announced through a polite live
  region, so a screen-reader user is told without being moved off what they are doing; this is how **Toast**
  and **InlineFeedback** reach assistive technology (WCAG 4.1.3).
- **acceptance_criteria**: With a screen reader, post a chirp, follow an account, and save a setting; each
  result is spoken as a status message and focus is not moved away from the person's current control. Leave
  the timeline open while new chirps arrive; the catch-up availability is announced politely.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-001, REQ-PROD-033

#### REQ-UX-116 — Errors are announced and tied to their field
- **statement**: A failed action is announced; a field-level error is associated with its field (so focusing
  the field re-reads the problem) and marks the field invalid, and the message says what happened and what to
  do with no system or error code (WCAG 3.3.1, 3.3.2, 3.3.3; composing with REQ-UX-032's voice rules).
- **acceptance_criteria**: With a screen reader on sign-up, submit a taken handle; the error is announced, is
  read when the handle field is focused, the field is reported invalid, and the message names the problem and
  a fix in plain words with no error code. Force a submit failure; the failure is announced, not silent.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-034

#### REQ-UX-117 — Empty, loading, and expired-session states reach assistive technology
- **statement**: A list's loading state announces that content is loading while its decorative skeletons are
  hidden from the screen reader and resolves to content or an announced error with a reachable retry; an
  **EmptyState** is a heading and action a screen-reader user lands on, with a failed load announced as an
  alert distinct from a genuinely empty surface; and an expired session mid-action is announced with its
  reason and moves focus to sign-in (REQ-UX-008).
- **acceptance_criteria**: With a screen reader, open a list while it loads — "loading" is announced and the
  skeletons are not read out; force a load failure — an error with a retry is announced. Reach a brand-new
  empty timeline — a heading and a next-step action are presented, not silence. Act with an expired session —
  the reason is announced and focus moves to the sign-in screen.
- **release_gating**: true
- **references**: REQ-PROD-036, REQ-PROD-033, REQ-PROD-034

#### REQ-UX-118 — Conformance is verified three ways over the gated set
- **statement**: Before a release, the gated conformance set (the core loop and the get-in screens with their
  empty/loading/error states) is verified by an automated check in each shipped theme, a keyboard-only pass,
  and a screen-reader pass of the core loop, against a stated support matrix of at least one screen reader per
  major platform with its common browser.
- **acceptance_criteria**: For a release, find the three recorded checks over the gated set — an automated
  scan per theme, a keyboard-only walk, and a screen-reader walk — and the support matrix they were run on;
  each gated screen appears in each check.
- **release_gating**: true
- **references**: REQ-PROD-036

#### REQ-UX-119 — Accessibility gates the release, and failures are fixes not waivers
- **statement**: A release is blocked until the gated conformance set passes the three checks (REQ-UX-118)
  with no outstanding Level A or AA failure; an accessibility failure is remedied, not waived, and a failure
  on a non-gating surface is tracked against that surface and ships with it.
- **acceptance_criteria**: Confirm that for a release the gated set has no open Level A or AA accessibility
  failure, and that any issue recorded against a non-gating surface (notifications, Explore) is tracked with
  that surface rather than blocking unrelated release work.
- **release_gating**: true
- **references**: REQ-PROD-036
