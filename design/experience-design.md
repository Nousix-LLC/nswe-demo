# chirp — Experience Design

*How chirp looks and behaves to the people who use it, and the testable bar it is held to.*
This document is the consolidated **experience design** for chirp (`Nousix-LLC/nswe-demo`,
grounded in the product at commit `c805525`). It specifies **what the product must look like and
how it must behave, and the bar a stranger can test it against — not front-end code**. Every
implementation "how" (framework, CSS, component code) is deferred to later code work.

It covers, as one coherent picture: the **information architecture & interaction** model; **every
screen and flow** in chirp's inventory with each of its applicable **empty / error / loading /
not-found / end-of-list / session-expired / validation / busy / success** states; the **visual
language & design system**; the **responsive layout**; the **accessibility** standard; and a single
unified **requirements register (`REQ-UX-001 … REQ-UX-119`)** that is the release bar for the
experience.

The bar is deliberate: **designed screens, not a skeleton.** A product is judged in its first
minute, and that judgement falls hardest on the unglamorous surfaces nobody demos — sign-up/in,
recovery, settings, account deletion, data export, the empty timeline, the failed submit. Those
surfaces are first-class here.

## The screen inventory (18 surfaces)

The design covers **eighteen** screens/surfaces, each designed with its applicable states or
recorded out-of-scope with a reason:

- **Entry / logged-out:** logged-out landing timeline; sign-up; sign-in; account recovery.
- **Signed-in core loop:** home timeline; compose; profile (own & other); chirp detail/thread +
  reply *(non-gating)*; new-user onboarding *(non-gating)*; search / discovery *(non-gating)*.
- **Account lifecycle / settings:** settings home; edit profile; account deletion; data export;
  sign-out.
- **Trust, safety & legal:** report / block & mute; privacy & terms.
- **Operator / admin (UX only):** admin / support surface.

## How to read this document

| # | Section | What it fixes |
|---|---------|---------------|
| 1 | Information architecture & interaction | Navigation model (logged-out/signed-in), the screen map, and the product-wide interaction & feedback patterns every screen reuses. |
| 2 | Screens & flows, and empty/error/loading states | Each inventory screen at wireframe level with its happy path and **every applicable state** designed to concrete copy + recovery, plus the cross-cutting state model. |
| 3 | Visual language & design system | Design tokens (type, colour, spacing, radius, elevation, grid, iconography, motion) and the component library, with the consistency rules screens are held to. |
| 4 | Responsive layout | Breakpoints and how navigation and every screen reflow across phone and larger screens; touch & on-screen-keyboard ergonomics. |
| 5 | Accessibility | The WCAG 2.1 AA standard made concrete per component and per screen, with the contrast audit and its required fixes. |
| 6 | Requirements & acceptance criteria | The single unified `REQ-UX-001 … REQ-UX-119` register: every requirement user-side, testable, release-gating-flagged, and traced. |

**Scope boundary (what is deliberately *not* here).** Implementation mechanism is deferred to the
code work. The *mechanisms* behind the surfaces designed here — credentials, sessions, auth
lifetime, authorization, deletion/export internals, moderation enforcement — are the identity &
security design's; persistence and export *format* are the domain & operational-readiness designs';
the release bar criteria (`BAR-*`) are the release-bar document's. This experience design specifies
the *screens, interactions, and the product-level UX requirement* for those surfaces; it does not
redefine the sibling areas' requirements.

**Grounding.** Grounded in chirp's product requirements (journeys J1–J8 and the
`REQ-PROD-*` register) and the current product at `Nousix-LLC/nswe-demo` @ `c805525` (a small SPA —
timeline, compose+sign-in, profile; handle-only login; minimal styling and no design system — taken
as the starting point to improve, not a target to preserve).

## Design reconciliations & known remediations (verification outcomes)

Assembling and verifying the five concern designs surfaced the following cross-section outcomes.
They are recorded here so a builder cannot miss them.

1. **Contrast remediations (release-gating — enforced by REQ-UX-113).** A WCAG 2.1 contrast audit of
   the visual system's colour tokens found three values below their intended ratio. The **token
   names stay stable; only the values darken**:
   - `color.border` (measured 1.44:1) must reach ≥ 3:1 where it is the sole boundary of an essential
     control (inputs, meaningful outlines) — target ≈ `#808D97`; purely decorative dividers
     (`color.border.subtle`) are exempt.
   - `color.warning.base` (measured 2.75:1) must reach ≥ 4.5:1 for white text on the fill and ≥ 3:1
     as an icon/large fill — target ≈ `#B35E08` — **or** warnings use `color.warning.ink` on
     `color.warning.bg` (which already passes) and never white text on `warning.base`.
   - The **focus ring** must hold ≥ 3:1 on white, subtle, and accent-tinted row backgrounds (the
     `color.accent.base` ring passes on white but not on tinted rows): draw it in
     `color.accent.strong` (`#1565A0`) or as a two-tone indicator.

   Until these land, the contrast requirements (REQ-UX-072, REQ-UX-113) are only partially met on
   gating screens. Every other audited text/UI pairing passes AA.

2. **Open component requests (design-system backlog).** Four screens asked the design system for a
   component it does not yet name; each ships now on a composable interim shape and the request is
   recorded rather than hand-rolled:
   - **C1** — a reply glyph + reply-count presentation on the chirp card (for the thread/reply
     screen; interim: the like-count treatment reused).
   - **C2** — a single-select reason control (option-list / radio group) for the report flow
     (interim: a segmented control for a short reason set).
   - **C3** — a mute glyph in the icon inventory (interim: a labelled mute menu item).
   - **C4** — a long-document contents / scroll-spy for privacy & terms (interim: anchored headings
     + a static "On this page" list).

3. **Cross-area seams to reconcile at build.** Two trust/safety seams reach beyond the experience
   design and are recorded for the owning areas: a possible **blocked/muted-accounts management
   list** as an account-settings surface (reversal is already reachable from the overflow today), and
   the **block/mute consequence-copy ↔ enforcement reconciliation** — the user-facing statement of
   what block/mute does must match the authoritative identity-&-security enforcement model at build.

---

## 1. Information architecture & interaction

*(section: `information_architecture_and_interaction`)*

### Information architecture & interaction

This section defines **the structure of chirp and how a user moves through it**: the navigation
model in both the logged-out and signed-in states, the screen map over the frozen screen inventory,
and the product-wide interaction and feedback patterns every screen reuses. It designs *what the
product's structure and behaviour must be and the bar it is tested against* — not the detailed
screen content, the visual system, the responsive rules, or the accessibility specifics, which are
owned by the sibling concerns and build on the structure defined here.

**Scope note.** Screen-level content and the per-screen empty/error/loading state designs are owned
by `screens_and_flows` / `empty_error_loading_states`; components and tokens by
`visual_language_and_design_system`; reflow by `responsive_layout`; operability by `accessibility`.
This section fixes the destinations those concerns populate and the interaction rules they obey.

#### Starting point (what exists today, grounded at `c805525`)

The current SPA (`crates/chirp-frontend`) is three routes behind a single persistent two-link nav:

```
nav:  [ Timeline · Compose ]
/            -> timeline view            (public read; loading/content/empty/error/paged)
/compose     -> compose view             (logged-out: handle field + "Log in";
                                          logged-in:  compose-a-chirp form)
/user/:id    -> profile view             (a user's chirps + follow/like)
(fallback)   -> not-found view
```

Sign-in is folded into `/compose` as a handle-only field; there is no sign-up, sign-in-as-its-own-
destination, recovery, settings, profile editing, account deletion, data export, sign-out control,
report/block, privacy/terms, onboarding, search, chirp-thread, or admin surface. The information
architecture below keeps what works (public-read timeline at `/`, a persistent nav, uncontrolled
inputs that preserve drafts across re-render) and grows the structure to make **every surface in the
frozen inventory reachable through the product**, never only through the API.

---

#### 1. Navigation model

chirp has **two navigation states** — logged-out and signed-in — plus one **operator surface** that
is deliberately kept out of the consumer navigation. The navigation model is the same information
architecture seen from two authorization vantage points; it is not two separate apps.

##### 1.1 Logged-out navigation

A logged-out visitor (persona P1) can **read freely** and always has an obvious route to join.

```
logged-out nav / chrome (persistent on every logged-out surface)

  [ chirp ]                         Search(*)        [ Sign in ]  [ Sign up ]
     |                                 |                 |            |
   home -> SCR-landing_timeline    SCR-search_*     SCR-signin   SCR-signup
                                    (*non-gating)

  footer / low-prominence chrome:  About chirp  ·  Privacy & terms (SCR-privacy_terms)
```

- **Primary**: the product wordmark routes to the public landing/home timeline (`SCR-landing_timeline`).
- **Join affordance**: `Sign up` and `Sign in` are persistently visible on every logged-out surface
  (nav and the first screen), satisfying "an obvious route from reading to joining".
- **Readable destinations without an account**: landing timeline and any user profile
  (`SCR-profile`) open without authentication; opening a profile is reached contextually from a chirp
  (a link on the author), not from the primary nav.
- **Secondary / low-prominence**: `Privacy & terms` is reachable from the logged-out chrome (footer
  or overflow); `Search` appears only if/where the non-gating discovery surface is present.
- **Recovery** (`SCR-recovery`) is **not** a top-level nav item; it is reached *from* the sign-in
  screen ("Can't sign in?"), so the recovery entry lives where a user who cannot get in will look.

##### 1.2 Signed-in navigation

A signed-in user (personas P2/P3) gets a persistent primary navigation covering the core loop, plus
an **account menu** that gathers identity and account-lifecycle surfaces.

```
signed-in primary nav (persistent)

  [ chirp ]   Home    Compose    Search(*)              ( @handle ▾ )  <- account menu
     |          |        |          |                        |
  home      SCR-home_  (compose   SCR-search_*          account menu (overflow):
            timeline   affordance) (*non-gating)          - Profile        -> SCR-profile (own)
                                                           - Settings       -> SCR-settings_home
                                                           - Privacy & terms-> SCR-privacy_terms
                                                           - Sign out       -> SCR-signout
```

- **Primary destinations** (always visible): `Home` (`SCR-home_timeline`), `Compose`
  (a compose affordance, see §2 and REQ-UX-012), and `Search` where present.
- **Identity indicator + account menu**: the signed-in nav always shows *who you are signed in as*
  (`@handle` / avatar) — this doubles as the consistent signed-in-state indicator (REQ-UX-015) and
  as the entry to the account overflow.
- **The account menu** routes to the user's own `Profile`, to `Settings`, to `Privacy & terms`, and
  holds the **`Sign out`** control. Sign-out has a single, discoverable, consistent home here (and is
  mirrored inside `SCR-settings_home`) — never buried and never duplicated into an unpredictable
  location.
- **Settings is the account-management hub** (`SCR-settings_home`): a reachable gather-point for
  `Edit profile` (`SCR-edit_profile`), `Data export` (`SCR-data_export`), `Account deletion`
  (`SCR-account_deletion`), `Sign out`, and the `Privacy & terms` link. Nothing in the account
  lifecycle requires the raw API.
- **Own vs other profile**: on the user's **own** profile the structure exposes an `Edit profile`
  affordance and a route into `Settings`; on **another** user's profile those are absent and
  follow/unfollow is present instead.
- **Contextual (not top-level) signed-in surfaces**: `Compose` (may be modal/inline, §2), a chirp's
  detail/thread (`SCR-chirp_thread`, conditional, §2.3), `Report/block` (`SCR-report_block`, reached
  from the overflow on a chirp or a profile), and `Edit profile` (reached from own profile or
  settings) are reached *in context*, not from the primary nav.

##### 1.3 Operator / admin surface (kept separate by design)

`SCR-admin_support` is an **operator-facing** surface (persona P4). Its information architecture is
deliberately **segregated from the consumer navigation**: it is a distinct operator route, is not
exposed in or linked from the signed-in consumer nav, and is reached only through its own entry. This
section designs only its *place in the IA and its reachability*; the authorization gate, the operator
capabilities, and whether a given operator may see it are owned by identity & trust (`REQ-ID-*`) and
operational readiness (`REQ-OPS-*`). Kept intentionally light here.

##### 1.4 Reachability guarantee

Every surface in the screen inventory is reachable through the product UI from an
appropriate entry point — no surface is API-only. The map:

| Surface | Reached from |
|---|---|
| `SCR-landing_timeline` | wordmark / home (logged-out) |
| `SCR-signup` | logged-out nav; link on `SCR-signin` and on `SCR-landing_timeline` |
| `SCR-signin` | logged-out nav; link on `SCR-signup` |
| `SCR-recovery` | "Can't sign in?" on `SCR-signin` |
| `SCR-home_timeline` | signed-in nav `Home`; landing destination after sign-in |
| `SCR-compose` | `Compose` in signed-in nav; compose affordance on `SCR-home_timeline` |
| `SCR-profile` | author link on any chirp; `Profile` in account menu (own) |
| `SCR-chirp_thread` | a chirp on a timeline/profile (conditional — §2.3) |
| `SCR-onboarding` | automatic first destination after `SCR-signup` (first session) |
| `SCR-search_discovery` | `Search` in nav (where present) |
| `SCR-settings_home` | account menu `Settings` |
| `SCR-edit_profile` | own `SCR-profile`; `SCR-settings_home` |
| `SCR-account_deletion` | `SCR-settings_home` |
| `SCR-data_export` | `SCR-settings_home` |
| `SCR-signout` | account menu; `SCR-settings_home` |
| `SCR-report_block` | overflow on a chirp (`SCR-home_timeline`/`SCR-chirp_thread`) or on `SCR-profile` |
| `SCR-privacy_terms` | logged-out footer; account menu / `SCR-settings_home` (signed-in) |
| `SCR-admin_support` | its own operator route (not in consumer nav — §1.3) |

---

#### 2. Screen map / information architecture

The frozen inventory organizes into **top-level destinations** (addressable, bookmarkable, in the
nav), **contextual surfaces** (reached in-flow, often modal/inline, usually not independently
bookmarkable), and the **operator route** (segregated). Destinations are stable routes so the browser
back button, deep links, and refresh behave predictably.

```
LOGGED-OUT (public)
  / ................................ SCR-landing_timeline        [top-level]
      -> (author link) ............. SCR-profile (read-only)     [contextual]
  /signup .......................... SCR-signup                  [top-level]
  /signin .......................... SCR-signin                  [top-level]
      -> /recovery ................. SCR-recovery                [nested under sign-in]

          === sign-up / sign-in succeeds -> transition to SIGNED-IN (see §2.4) ===

SIGNED-IN (core loop)
  /home ............................ SCR-home_timeline           [top-level]
      -> compose ................... SCR-compose                 [contextual: modal/inline or /compose]
      -> (a chirp) ................. SCR-chirp_thread            [contextual/top-level, conditional §2.3]
      -> (author link) ............. SCR-profile                 [top-level]
  /search .......................... SCR-search_discovery        [top-level, non-gating]
  /user/:id ........................ SCR-profile                 [top-level]
      -> (own) edit ................ SCR-edit_profile            [contextual]
      -> (a chirp/account) report .. SCR-report_block            [contextual: overflow + confirm]
  first session after sign-up ...... SCR-onboarding              [transient first destination]

ACCOUNT / SETTINGS (account menu -> Settings hub)
  /settings ........................ SCR-settings_home           [top-level hub]
      -> /settings/profile ......... SCR-edit_profile            [nested]
      -> /settings/export .......... SCR-data_export             [nested]
      -> /settings/delete .......... SCR-account_deletion        [nested]
      -> sign out .................. SCR-signout                 [action -> logged-out]
      -> privacy & terms ........... SCR-privacy_terms           [nested/link]

TRUST / LEGAL (reachable both states)
  /privacy ......................... SCR-privacy_terms           [top-level, low-prominence]

OPERATOR (segregated — §1.3)
  (operator route) ................. SCR-admin_support           [separate; not in consumer nav]
```

Route paths above are **illustrative destinations**, not an implementation contract — what is fixed
here is the *hierarchy* (what is top-level vs nested vs contextual) and the *reachability*. The exact
URL strings are a build-time concern for the code work; the current SPA's `/`, `/compose`, `/user/:id`
are preserved or generalized rather than broken.

##### 2.1 Top-level vs nested vs contextual

- **Top-level destinations** are bookmarkable and survive a refresh and a deep link: landing/home
  timeline, sign-up, sign-in, home timeline, profile, search, settings, privacy/terms.
- **Nested destinations** live under a parent hub (the settings sub-pages under `SCR-settings_home`;
  recovery under sign-in) and are reached through it.
- **Contextual surfaces** are invoked in-flow without losing the underlying context: compose
  (modal/inline), report/block (overflow + confirm), edit-profile. A contextual surface that a user
  may want to deep-link to (a chirp thread) is also given a stable destination (§2.3).

##### 2.2 Compose placement

Compose is reachable from anywhere in the signed-in core loop without abandoning the current view: it
is a `Compose` nav action and a compose affordance on the home timeline, and may be realized as an
inline composer or a modal over the timeline (as the current product already folds compose into a
view) rather than forcing a separate full-page route. The draft-preservation guarantee (uncontrolled
inputs that survive a re-render, already true in the current SPA) is part of the compose interaction
and is carried as an interaction requirement (see REQ-UX-012 and the compose owner's screen design).

##### 2.3 Chirp thread / detail (conditional)

Per the frozen inventory's conditional note on `SCR-chirp_thread` and `REQ-PROD-023`, the IA takes an
explicit either/or stance: **either** a chirp has a coherent detail/thread destination (a chirp is
openable to read its thread and reply, reached by selecting the chirp) **or** no reply/thread
affordance appears anywhere in the navigation or screen structure. The IA MUST NOT place a thread
entry point or reply control that leads nowhere. The "deliberately absent" stance is a valid,
designed outcome of this section, not a gap.

##### 2.4 The logged-out -> signed-in transition

The transition between the two navigation states is a designed moment, not an incidental re-render:

- On **successful sign-in** (`SCR-signin`) or **sign-up** (`SCR-signup`), the user lands on a
  **purposeful signed-in destination** — the signed-in home timeline (`SCR-home_timeline`), or, for a
  brand-new account's first session, the onboarding first-run (`SCR-onboarding`, §2.5) — never back on
  the logged-out landing screen.
- **Return-to-intent**: when the user arrived at sign-in because they attempted a gated action while
  logged out or session-expired (see the auth-gating pattern, §3.3), completing authentication returns
  them to that action/destination, not to a generic home.
- The navigation chrome **swaps atomically** from the logged-out set (§1.1) to the signed-in set
  (§1.2) on the session becoming non-`None`, and back to logged-out on sign-out
  (`SCR-signout`). The identity indicator (REQ-UX-015) reflects the new state immediately.

##### 2.5 First-run placement (onboarding)

After sign-up, the **first signed-in destination is the onboarding first-run** (`SCR-onboarding`) —
offering a first follow and/or a first post — rather than a blank home timeline. This is a structural
placement decision (where the new user lands) owned here; the onboarding screen's content is designed
by the screens concern. Non-gating, per `REQ-PROD-022`.

---

#### 3. Global interaction & feedback patterns

These are the **product-wide rules the screens reuse**. Defining them once here is what makes the
product feel coherent instead of per-screen improvised. Each screen owner applies these patterns; the
visual rendering of each pattern (exact components, colour, motion) is the visual-system concern.

##### 3.1 Every action gives honest feedback

No user action completes silently and no action falsely appears to succeed. Every action resolves
into exactly one observable outcome drawn from the frozen `state_vocabulary`: a `busy_submitting`
indication while in flight, then a `success` confirmation or an honest, recoverable `error`.
"Honest" means the product never shows success for a failed action and never leaves an action in an
indeterminate state.

##### 3.2 Optimistic update + reconcile-on-failure

For **quick, reversible** actions whose expected outcome is near-certain — **like/unlike** and
**follow/unfollow**, and their counts — the product MAY update the UI optimistically on tap, before
the server confirms, to keep the core loop fast. On failure it MUST **reconcile**: visibly revert the
optimistic change to the true state and surface an honest error, so the on-screen state always matches
reality after the action settles (`REQ-PROD-020`, `REQ-PROD-021`). Optimism is confined to
reversible, low-stakes actions; it is never applied to destructive or irreversible ones (§3.4).

##### 3.3 Auth-gating: read freely, prompt only on a gated action, never a silent fail

Reading — the public timeline and user profiles — **never** requires an account (`REQ-PROD-013`). An
action that genuinely requires authentication (compose, like, follow, reply, edit, delete, export,
report) is gated **at the point of action**, not by walling off the content behind it. When a
logged-out user — or a user whose session has **expired/been invalidated** — triggers a gated action,
the product surfaces a **clear sign-in prompt** and, on success, returns the user to the action they
attempted (§2.4). It MUST NOT fail silently, MUST NOT appear to succeed, and MUST NOT discard the
user's in-progress input in the process (`REQ-PROD-006`). A `session_expired` outcome always routes
through this same prompt rather than a dead end.

##### 3.4 Confirmation & destructive-action pattern

**Destructive or irreversible** actions — account deletion (`SCR-account_deletion`), and account-
level trust actions such as block (`SCR-report_block`) — MUST require an explicit confirmation that
**states the consequences in plain terms** before proceeding (for deletion: that the account, its
chirps, its follows, and its likes are removed — `REQ-PROD-010`/`REQ-PROD-012`). **Reversible**
actions (like, follow, unfollow) MUST NOT be gated behind a confirmation — over-confirming trains
users to dismiss dialogs. The consequence copy is authored by the screen owner; the *rule for when a
confirmation is required and what it must state* is fixed here.

##### 3.5 Consistent surfacing of the state vocabulary

The frozen `state_vocabulary` (`loading`, `content`, `empty`, `error`, `not_found`, `end_of_list`,
`session_expired`, `validation`, `busy_submitting`, `success`) is the **shared contract every screen
presents consistently**. This section fixes the *global rules*; the per-screen state designs belong to
`empty_error_loading_states`:

- A **state means the same thing and is presented the same way** wherever it appears — an `error` on
  the timeline, on a profile, and on a submit are recognizably the same pattern, and every `error`
  offers recovery (retry) **without a full page reload** (`REQ-PROD-018`).
- **`validation`** is inline and actionable at or before submit (handle rules, compose length), never
  surfaced only as a raw server/API error (`REQ-PROD-002`).
- **`empty`** states are first-class, not blank screens: every empty state states what it is and
  offers the next action (join, compose, follow, search as applicable).
- **`not_found`** (unknown handle/id, deleted account) is an honest designed state, not a crash or a
  redirect that hides the cause.
- **`session_expired`** always routes through the auth-gating prompt (§3.3).

##### 3.6 Consistent signed-in/out indicator

The product **always shows which authorization state the user is in** — a persistent identity/account
indicator in the signed-in nav and the Sign in/Sign up affordances in the logged-out nav — so a user
can tell at a glance whether they are signed in and as whom (`REQ-PROD-006`). The indicator updates
immediately on the logged-out <-> signed-in transition (§2.4).

---

#### 4. Requirements & acceptance criteria (REQ-UX-001 .. REQ-UX-019)

Range owned by `the information-architecture & interaction section` per the REQ-UX id ranges. Each requirement is
stated from the user's side, is testable by a stranger exercising the live product, carries a
`release_gating` flag, and traces to >=1 `REQ-PROD-*` and/or journey. These requirements govern the
navigation model, the screen map, and the global interaction patterns — the structure and behaviour
other concerns build on.

##### REQ-UX-001 — Persistent signed-in primary navigation over the core loop
- **Statement:** A signed-in user has a persistent primary navigation that reaches the core-loop and account destinations from anywhere in the product.
- **Acceptance criteria:** From any signed-in screen, a visible, persistent navigation offers, at minimum, Home (timeline), a Compose affordance, the user's own Profile, and an account entry that reaches Settings — each reachable in one interaction, without using the raw API or the browser URL bar.
- **Release gating:** true
- **Traces to:** J5, J6; REQ-PROD-009, REQ-PROD-016.

##### REQ-UX-002 — Logged-out navigation makes reading and joining obvious
- **Statement:** A logged-out visitor can read public content and always sees an obvious route to join.
- **Acceptance criteria:** On every logged-out surface, a persistent Sign up and Sign in are visible; the landing/home timeline and user profiles are readable without signing in; and a "create an account" control is present on the first screen and on the sign-in screen.
- **Release gating:** true
- **Traces to:** J1; REQ-PROD-013, REQ-PROD-014, REQ-PROD-015.

##### REQ-UX-003 — Every inventory surface is reachable through the product
- **Statement:** Every screen/surface in the product's screen inventory is reachable by navigating the product UI, with no consumer surface reachable only via the raw API.
- **Acceptance criteria:** Starting from the appropriate entry (logged-out landing or signed-in home), a person can navigate to each consumer surface in the inventory through visible controls/links (per the reachability map in §1.4); no consumer surface requires typing an API request or an unlinked URL. (The operator surface is reachable through its own operator entry, §1.3.)
- **Release gating:** true
- **Traces to:** J1–J8; REQ-PROD-009, REQ-PROD-015.

##### REQ-UX-004 — A reachable account-management hub gathers the lifecycle surfaces
- **Statement:** A signed-in user has a reachable settings/account hub that gathers the account-lifecycle surfaces in one place.
- **Acceptance criteria:** From the signed-in navigation a user reaches a settings home that links, at minimum, to edit profile, data export, account deletion, sign-out, and privacy/terms — without using the raw API.
- **Release gating:** true
- **Traces to:** J6, J7; REQ-PROD-009.

##### REQ-UX-005 — Sign-out has a single, discoverable, consistent placement
- **Statement:** A signed-in user can always find a sign-out control in a predictable place.
- **Acceptance criteria:** A visible sign-out control is present in the signed-in account navigation and within the settings hub, in the same place across the product; using it transitions the product to the logged-out state (§REQ-UX-011). It is not hidden, not duplicated into unpredictable locations.
- **Release gating:** true
- **Traces to:** J3; REQ-PROD-005.

##### REQ-UX-006 — Own-profile exposes edit and settings; own vs other is distinguished
- **Statement:** On their own profile a user can reach profile editing and account management; another user's profile does not show those controls.
- **Acceptance criteria:** Viewing one's own profile shows an edit-profile affordance and a route to settings; viewing another user's profile shows follow/unfollow instead and shows no edit/settings controls.
- **Release gating:** true
- **Traces to:** J5, J6; REQ-PROD-008.

##### REQ-UX-007 — Privacy & terms reachable in both navigation states
- **Statement:** A user can reach the privacy & terms surface whether logged out or signed in.
- **Acceptance criteria:** A link to privacy & terms is present in the logged-out chrome and in the signed-in account navigation/settings, and following it reaches the privacy & terms surface.
- **Release gating:** true
- **Traces to:** J7; REQ-PROD-029.

##### REQ-UX-008 — A discoverable entry point to search/discovery
- **Statement:** Where search/discovery is present, it has a discoverable entry point in the navigation rather than being hidden.
- **Acceptance criteria:** A user can locate and open the search/discovery surface from the navigation (signed-in, and logged-out where offered); if discovery is not shipped in this version, no dead search entry point appears in the navigation.
- **Release gating:** false — search/discovery is a non-gating product requirement (REQ-PROD-024); its navigation entry inherits that flag.
- **Traces to:** J1, J5, J6; REQ-PROD-024.

##### REQ-UX-009 — The operator/admin surface is segregated from the consumer navigation
- **Statement:** The operator/admin surface is reached through its own entry and is not woven into the consumer navigation.
- **Acceptance criteria:** No link to the admin/support surface appears in the signed-in consumer navigation; the admin/support surface is reached only through its own operator entry. (Whether a given person may open it is an authorization concern owned by identity & trust / operational readiness, not tested here.)
- **Release gating:** true
- **Traces to:** J8; REQ-PROD-028, REQ-PROD-033.

##### REQ-UX-010 — A coherent screen map with stable destinations and hierarchy
- **Statement:** The product organizes its surfaces into a coherent map of top-level destinations, nested sub-pages, and contextual surfaces, with destinations that behave predictably under deep-link, refresh, and back.
- **Acceptance criteria:** Each top-level destination (per §2) is reachable by a stable address that survives a page refresh and the browser back button and can be returned to directly; nested surfaces are reached through their parent hub; contextual surfaces (compose, report) open without discarding the underlying screen's context.
- **Release gating:** true
- **Traces to:** J1, J5, J6, J7; REQ-PROD-009, REQ-PROD-018, REQ-PROD-019.

##### REQ-UX-011 — A designed logged-out → signed-in transition
- **Statement:** Signing up or signing in lands the user on a purposeful signed-in destination and the navigation swaps to the signed-in state; signing out returns to the logged-out state.
- **Acceptance criteria:** On successful sign-in/sign-up the user lands on the signed-in home timeline (or, for a new account's first session, the onboarding first-run) rather than the logged-out landing; the navigation chrome changes to the signed-in set and the identity indicator shows the user; on sign-out the product returns to the logged-out state and navigation. When sign-in was triggered by a gated action, completing it returns the user to that action.
- **Release gating:** true
- **Traces to:** J2, J3; REQ-PROD-001, REQ-PROD-004, REQ-PROD-006, REQ-PROD-022.

##### REQ-UX-012 — Compose reachable in-context without losing state
- **Statement:** A signed-in user can start composing from the core loop without leaving their current context or losing an in-progress draft.
- **Acceptance criteria:** A compose affordance is reachable from the home timeline and the primary navigation; invoking it (inline, modal, or route) does not discard the surrounding timeline context, and an in-progress compose draft survives a re-render and a transient failure (the user can retry without retyping).
- **Release gating:** true
- **Traces to:** J5; REQ-PROD-016, REQ-PROD-017.

##### REQ-UX-013 — Chirp thread/detail is coherent-if-present, never a dead affordance
- **Statement:** The product either gives a chirp a coherent detail/thread destination or exposes no thread/reply affordance at all.
- **Acceptance criteria:** Either (a) selecting a chirp opens a detail/thread destination where its thread can be read and a reply posted into that thread; or (b) no thread-open or reply control appears anywhere in the navigation or screen structure. The product never presents a thread/reply entry point that leads nowhere.
- **Release gating:** false — replies are a non-gating product requirement (REQ-PROD-023), provided no dead affordance ships.
- **Traces to:** J5; REQ-PROD-023.

##### REQ-UX-014 — The first signed-in destination after sign-up is a guided first-run
- **Statement:** A newly signed-up user lands on a guided first-run rather than a blank timeline.
- **Acceptance criteria:** Immediately after sign-up, the first signed-in destination offers a next action (accounts to follow and/or a prompt to post) instead of only an empty home timeline.
- **Release gating:** false — onboarding is a non-gating product requirement (REQ-PROD-022).
- **Traces to:** J2, J6; REQ-PROD-022.

##### REQ-UX-015 — A consistent signed-in / logged-out state indicator
- **Statement:** The product always shows, at a glance, whether the user is signed in and as whom.
- **Acceptance criteria:** On every signed-in screen a persistent indicator shows the signed-in identity (handle/avatar); on logged-out screens the navigation shows the sign-in/sign-up affordances instead; the indicator updates immediately on the logged-out↔signed-in transition.
- **Release gating:** true
- **Traces to:** J3, J6; REQ-PROD-006.

##### REQ-UX-016 — Auth-gating at the point of action, never a silent fail
- **Statement:** Reading never requires an account; an action that requires authentication prompts for sign-in at the moment of the action and returns the user to it, and never fails silently or falsely succeeds.
- **Acceptance criteria:** A logged-out visitor can read the timeline and profiles without a prompt; attempting a gated action (compose, like, follow, reply, edit, delete, export, report) — including when a session has expired — surfaces a clear sign-in prompt, and on success resumes the attempted action; no gated action silently does nothing or appears to succeed while logged out/expired.
- **Release gating:** true
- **Traces to:** J3, J5; REQ-PROD-006, REQ-PROD-013.

##### REQ-UX-017 — Optimistic updates reconcile to the true state on failure
- **Statement:** Quick reversible actions may update the UI optimistically but always reconcile to reality when the action fails.
- **Acceptance criteria:** For like/unlike and follow/unfollow, a successful action leaves the on-screen state and counts matching the true state; a failed action visibly reverts the optimistic change to the true state and reports the failure honestly rather than leaving a wrong count or a silent drop.
- **Release gating:** true
- **Traces to:** J5; REQ-PROD-020, REQ-PROD-021.

##### REQ-UX-018 — Destructive actions confirm consequences; reversible ones do not over-confirm
- **Statement:** Irreversible or destructive actions require an explicit, consequence-stating confirmation; reversible actions are not gated behind confirmations.
- **Acceptance criteria:** Initiating account deletion (and account-level block) presents an explicit confirmation stating what the action does (for deletion: the account, its chirps, follows, and likes are removed) before it proceeds; like/follow/unfollow proceed without a confirmation dialog.
- **Release gating:** true
- **Traces to:** J7, J5; REQ-PROD-010, REQ-PROD-012, REQ-PROD-027.

##### REQ-UX-019 — The state vocabulary is surfaced consistently product-wide
- **Statement:** Loading, content, empty, error, not-found, end-of-list, session-expired, validation, busy, and success are presented consistently everywhere they occur, and every error is recoverable without a full page reload.
- **Acceptance criteria:** The same state is recognizably the same pattern across screens (an error on the timeline, a profile, and a submit are the same pattern); every error state offers a retry that does not reload the whole page; validation is inline and actionable at/before submit; empty states state what they are and offer a next action; not-found and session-expired are honest designed states (session-expired routing through the sign-in prompt of REQ-UX-016).
- **Release gating:** true
- **Traces to:** J1, J5, J6; REQ-PROD-014, REQ-PROD-018, REQ-PROD-019.

---

## 2. Screens, flows & states

*(sections: `screens_and_flows` and `empty_error_loading_states`)*

### Screens, flows & states — chirp experience design

*Design sections: `screens_and_flows` and `empty_error_loading_states`, with the unified
`REQ-UX-020..069` register. Part of the chirp experience design; grounded in the product at
`Nousix-LLC/nswe-demo` @ `c805525`. Design intent and the testable bar only — what, not how;
no front-end code.*

This deliverable assembles the cross-cutting **state model** and the four **surface-group**
designs into one coherent picture of every screen in chirp's frozen inventory — each with its
content/layout intent at wireframe level, its happy-path flow, and **each applicable state**
(empty / error / loading / not-found / end-of-list / session-expired / validation /
busy-submitting / success) with concrete copy intent and a recovery affordance — plus the one
`REQ-UX-020..069` requirement register. It is built to a single bar: *designed screens, not a
skeleton* — the product is judged in its first minute, and that judgement falls on the states
nobody demos.

**How this document is organised.**

1. **Unified requirement register — `REQ-UX-020..069`** (below): the fifty screens-and-states
   requirements, each user-side, testable by a stranger against the live product, flagged
   release-gating or not, and traced to the product requirements and journeys it serves. This is
   the release bar for screens, flows & states.
2. **The state model** (section `empty_error_loading_states`): the one product-wide model every
   surface instantiates per screen.
3. **Screens & flows** (section `screens_and_flows`): the four surface groups — entry & auth, the
   signed-in core loop, account lifecycle & settings, and trust/safety/admin — assembled from
   their owning concern sections.

**Scope boundary.** Mechanism is deferred: credential/session/auth lifetime, deletion/export
internals and authorization, and persistence belong to the identity-&-trust, domain, and
operational-readiness requirement areas; implementation belongs to the code work. The visual
language & design system, the information-architecture & interaction model, responsive layout,
and accessibility are sibling experience-design sections — referenced here, never redefined.

---

#### Unified requirement register — `REQ-UX-020 .. REQ-UX-069`

Fifty requirements across the eighteen inventory screens plus the cross-cutting state model,
partitioned into disjoint sub-ranges: the state model `020..029`, entry & auth `030..039`, the
core loop `040..051`, account & settings `052..061`, and trust/safety/admin `062..069`. Every id
is unique, lies inside its owning concern's sub-range and within `REQ-UX-020..069`, collides with
no other, and leaves no gap. Each requirement's full statement and acceptance criteria live in the
owning screen's subsection below.

| REQ-UX | Screen / concern | Gating | Traces to |
|---|---|---|---|
| REQ-UX-020 | state model — one consistent model | true | REQ-PROD-014, 018, 019; J1, J5, J6 |
| REQ-UX-021 | state model — loading shown in place | true | REQ-PROD-018, 019, 014; J1, J5 |
| REQ-UX-022 | state model — empty explains & offers next | true | REQ-PROD-014, 018, 022, 024; J1, J5, J6 |
| REQ-UX-023 | state model — errors honest, in-place, input-safe | true | REQ-PROD-018, 017, 019; J1, J5 |
| REQ-UX-024 | state model — not-found honest, with a way back | true | REQ-PROD-019, 012; J5, J7 |
| REQ-UX-025 | state model — end-of-list explicit ≠ empty | true | REQ-PROD-018; J5 |
| REQ-UX-026 | state model — session expiry prompts re-sign-in | true | REQ-PROD-006; J3, J6 |
| REQ-UX-027 | state model — validation inline & at/before submit | true | REQ-PROD-002, 003, 016; J2, J5 |
| REQ-UX-028 | state model — busy, no duplicate, reconcile on fail | true | REQ-PROD-016, 017, 020, 021; J5 |
| REQ-UX-029 | state model — success honest, observable, never false | true | REQ-PROD-016, 020, 021; J5 |
| REQ-UX-030 | SCR-landing_timeline — says what chirp is + join route | true | REQ-PROD-013, 014, 015; J1 |
| REQ-UX-031 | SCR-landing_timeline — reading needs no account | true | REQ-PROD-013; J1 |
| REQ-UX-032 | SCR-landing_timeline — honest loading/empty/error/end | true | REQ-PROD-014, 018; J1 |
| REQ-UX-033 | SCR-signup — in-product account creation | true | REQ-PROD-001, 015; J1, J2 |
| REQ-UX-034 | SCR-signup — handle validation & uniqueness inline | true | REQ-PROD-002, 003; J2 |
| REQ-UX-035 | SCR-signup — honest busy/error/success, no lost input | true | REQ-PROD-001, 017; J2 |
| REQ-UX-036 | SCR-signin — credentialed, interlinked w/ signup+recovery | true | REQ-PROD-004, 015; J3 |
| REQ-UX-037 | SCR-signin — honest, non-enumerating, never false success | true | REQ-PROD-004, 006; J3 |
| REQ-UX-038 | SCR-recovery — reachable; regain the same account | true | REQ-PROD-007; J4 |
| REQ-UX-039 | SCR-recovery — honest, non-enumerating states | true | REQ-PROD-007; J4 |
| REQ-UX-040 | SCR-home_timeline — reverse-chron feed, honest states, paging | true | REQ-PROD-018; (025 non-gating); J5, J6 |
| REQ-UX-041 | SCR-home_timeline — like/follow/compose honest; reconcile | true | REQ-PROD-016, 020, 021; J5 |
| REQ-UX-042 | SCR-home_timeline — feed action on expired session prompts | true | REQ-PROD-006; J6 |
| REQ-UX-043 | SCR-compose — compose & post, visible ≤280, non-optimistic | true | REQ-PROD-016; J5 |
| REQ-UX-044 | SCR-compose — draft survives; failure/expiry honest | true | REQ-PROD-017, 006; J5, J6 |
| REQ-UX-045 | SCR-profile — honest, own-vs-other, real states | true | REQ-PROD-019, 008, 013, 012; J5, J6 |
| REQ-UX-046 | SCR-profile — follow/unfollow optimistic, reconciled | true | REQ-PROD-020; J5, J6 |
| REQ-UX-047 | SCR-chirp_thread — open thread, honest states, no dead affordance | false | REQ-PROD-023, 018, 019; J5 |
| REQ-UX-048 | SCR-chirp_thread — reply coherent, validated, non-optimistic | false | REQ-PROD-023, 016, 017; J5 |
| REQ-UX-049 | SCR-onboarding — first-run next action + observable payoff | false | REQ-PROD-022; J2, J6 |
| REQ-UX-050 | SCR-search_discovery — find accounts & content | false | REQ-PROD-024; J1, J5, J6 |
| REQ-UX-051 | SCR-search_discovery — honest search states (running ≠ none) | false | REQ-PROD-024; J1, J5, J6 |
| REQ-UX-052 | SCR-settings_home — reachable account-management hub | true | REQ-PROD-009; J6, J7 |
| REQ-UX-053 | SCR-edit_profile — view & edit display name/bio, persists | true | REQ-PROD-008; J6 |
| REQ-UX-054 | SCR-edit_profile — validates inline, never loses the edit | true | REQ-PROD-008, 002, 017; J6 |
| REQ-UX-055 | SCR-account_deletion — states removal + explicit confirm | true | REQ-PROD-010; J7 |
| REQ-UX-056 | SCR-account_deletion — signed out + not-found after | true | REQ-PROD-012; J7 |
| REQ-UX-057 | SCR-data_export — export own data, portable, downloadable | true | REQ-PROD-011; J7 |
| REQ-UX-058 | SCR-data_export — honest progress → real download or error | true | REQ-PROD-011, 018; J7 |
| REQ-UX-059 | SCR-signout — consistent control; logged out after | true | REQ-PROD-005; J3 |
| REQ-UX-060 | account surfaces — session expiry prompts, never silent | true | REQ-PROD-006; J6, J3 |
| REQ-UX-061 | SCR-settings_home — privacy & terms reachable from hub | true | REQ-PROD-029, 009; J7 |
| REQ-UX-062 | SCR-report_block — report a chirp/account, told it filed | true | REQ-PROD-027, 017; J5 |
| REQ-UX-063 | SCR-report_block — block (consequences stated) & mute | true | REQ-PROD-027; J5 |
| REQ-UX-064 | SCR-report_block — honest states, never silent/false | true | REQ-PROD-027, 017, 006, 018; J5 |
| REQ-UX-065 | SCR-privacy_terms — reachable both nav states, no account | true | REQ-PROD-029, 013; J7 |
| REQ-UX-066 | SCR-privacy_terms — presented plainly & readably | true | REQ-PROD-029; J7 |
| REQ-UX-067 | SCR-admin_support — operator looks up account by handle/id | true | REQ-PROD-033; J8 |
| REQ-UX-068 | SCR-admin_support — detail enough to answer support | true | REQ-PROD-033; J8 |
| REQ-UX-069 | SCR-admin_support — act on abuse, effect visible to users | true | REQ-PROD-028, 033; J8 |

**Gating shape.** The release-gating screens-and-states core is the one state model
(`020..029`), entry & auth (`030..039`), the gating core loop (`040..046`), account lifecycle &
settings (`052..061`), and trust/safety/admin (`062..069`). The chirp thread / reply
(`047..048`), new-user onboarding (`049`), and search / discovery (`050..051`) are **non-gating,
recorded cuts** — a first releasable version may ship without them — but each is designed in full
regardless. The one non-negotiable inside the non-gating thread: the product must never ship a
reply or thread-open affordance that leads nowhere (the "no dead affordance" proviso is carried as
a release-gating requirement by the information-architecture & interaction section).

---

#### Empty, error & loading states — the cross-cutting state model

*(section: `empty_error_loading_states`)*

This section designs **chirp's one product-wide model for the non-happy states** — the moments the product
is loading, has nothing to show, has failed, cannot find the thing, has reached the end, has lost the
session, is rejecting input, is working on a submission, or has just succeeded. These are the states a
product is judged on in its first minute, and the ones most often left as a blank page, a raw error code, a
spinner with no exit, or a silent failure. The model here is deliberately **defined once** so that every
surface section instantiates the same treatment per screen rather than reinventing state handling screen by
screen — that consistency is what makes chirp read as one intentional product instead of a set of screens
that each fail differently.

**What this settles, and what it leaves to its siblings.**

- It **owns** the shared behaviour, copy *intent and voice*, and recovery affordance of each state in the
  frozen `state_vocabulary`, plus the cross-cutting rules that bind them, plus the `REQ-UX-020..029`
  register for the state model.
- It **elaborates** the per-screen state matrix into a complete model; it does **not** redefine
  the state vocabulary, the screen inventory, or the REQ-UX id scheme.
- It **composes with**, and does not re-draw, the visual-system **state blocks** (visual-system §3.14), the
  busy control state (§3.0), the input validation states and character counter (§3.3/§3.4), the toast
  (§3.12) and inline alert (§3.13), and the IA **global interaction patterns** (ia-interaction §3). Where a
  needed presentation would require a component the visual system lacks, this section **flags it for resolution** rather than inventing one. (None was needed — see §3.)
- The **exact per-screen wording** and the choice of which applicable states a given screen shows are the
  surface sections' (`entry_auth`, `core_loop`, `account_settings`, `trust_safety_admin`), driven by
  the per-screen state matrix. This section gives each surface section the model and representative
  exemplar copy to instantiate; it does not author any single screen.
- The **mechanisms** behind these states — the auth/session protocol and session lifetime, deletion and
  export internals, the persistence engine — are out of UX scope and belong to identity & trust
  (`REQ-ID-*`), domain (`REQ-DOM-*`), and operational readiness (`REQ-OPS-*`). This section designs only
  what the user sees and can do, and the testable bar.

---

##### 0. The model

###### 0.1 The frozen state vocabulary (designed here)

The states are drawn verbatim from the screen inventory & state vocabulary; none is added or renamed.

| State | One-line role |
|---|---|
| `content` | The loaded, happy path — the baseline every other state is a departure from. Rendered by the surface's own component; no state block. |
| `loading` | Work is in flight on a first/again fetch; content is not yet ready. |
| `empty` | The fetch succeeded but there is nothing to show — and *why* matters. |
| `error` | A fetch or submit failed; the failure is honest and recoverable. |
| `not_found` | The specific thing (profile / chirp / account) does not exist or is gone. |
| `end_of_list` | A paginated list has no further pages — an explicit end, not an ambiguous stop. |
| `session_expired` | An authenticated action was attempted after the session expired/was invalidated. |
| `validation` | Input does not satisfy the rules; the field says what to fix, before/at submit. |
| `busy_submitting` | A submission is in flight; the control prevents a duplicate and holds its place. |
| `success` | An action completed; the confirmation is honest and the result is observable. |

###### 0.2 State principles (every state obeys these)

1. **Honest.** The product never shows success for a failed action, never a false "loaded" for an error,
   and never leaves an action in an indeterminate state (ia-interaction §3.1). A state tells the truth
   about what happened.
2. **Consistent by construction.** A given state **means the same thing and looks the same wherever it
   appears** — an `error` on the timeline, on a profile, and on a submit are recognisably one pattern,
   built from the same visual-system block (ia-interaction §3.5; visual-system §5). Surface sections
   instantiate; they do not invent per-screen variants of a state's shape.
3. **Recoverable.** Every state that can trap a user offers a way forward — a retry, a route back, a
   re-sign-in, a correction — and `error`/`loading` recover **without a full page reload** (REQ-PROD-018).
   No dead end, no indefinite spinner.
4. **Input-preserving.** No state discards work the user has in progress: a draft survives a failed send
   and a re-render, a form survives a validation bounce, and a session-expiry prompt does not silently
   drop what the user typed (REQ-PROD-017; ia-interaction §3.3).
5. **Not carried by colour alone.** Every state signal is also carried by icon, text, or weight, so it is
   legible to someone who cannot distinguish the colours (visual-system §3.0/§3.13, REQ-UX-073).

###### 0.3 The per-state anatomy every surface section instantiates

So the four surface sections produce *one shared model per screen* rather than divergent handling, each state
below is specified with the same five-part anatomy, and each per-screen instance fills the same slots:

- **When it occurs** — the trigger that puts the surface into this state.
- **Copy intent & voice** — the tone and what the copy must convey, with **representative exemplars** a
  surface section adapts to the specific screen (the exemplars are illustrative, not frozen strings).
- **Recovery affordance** — the one clear way forward the state must offer.
- **Composition** — the visual-system block/component that renders it and the IA pattern it obeys (named,
  never re-drawn here).
- **Per-screen instantiation** — how a surface section tailors it, and the variants to distinguish.

###### 0.4 Shared copy voice (binds the exemplars below and every per-screen instance)

- **Plain, calm, and specific.** Say what happened or what to do in ordinary words; never a raw error code,
  stack detail, or API message surfaced to the user.
- **Non-blaming and recoverable in tone.** "Something went wrong loading your timeline" — not "You did
  something wrong." State that the situation is recoverable where it is.
- **Short, then a next step.** A one-line title that names the situation, at most a sentence of support,
  and a clear action label (a verb: "Try again", "Go to home", "Sign in", "Compose").
- **Consistent terms.** The product's own nouns — *chirp*, *timeline*, *handle*, *profile*, *follow* —
  used the same way everywhere. Never leak implementation nouns (endpoint, token, store) into user copy.
- **Repository-ready.** No host, cluster, system, or run detail in any user-facing string.

---

##### 1. The designed states

###### 1.0 `content` — the loaded baseline

- **When it occurs:** the fetch resolved and there is data to show. This is the happy path and the
  reference every other state departs from.
- **Copy intent & voice:** n/a as a block — the content itself is the message; the surface's own component
  carries it.
- **Recovery affordance:** n/a (positive terminal of a fetch).
- **Composition:** rendered by the surface's primary component — the chirp card/list (visual-system §3.6),
  the profile header/identity cluster (§3.5), the settings list (§3.15) — never a state block. Visual-system
  §4 maps `content` to "the relevant component".
- **Per-screen instantiation:** each surface section designs the loaded layout of its own screens; this model
  only fixes that `content` is the baseline and that the transition **into** it from `loading` must not jump
  the layout (shape-matched skeletons, §1.1).

###### 1.1 `loading`

- **When it occurs:** a surface is fetching data the user is waiting on — first load of the timeline,
  profile, thread, search, settings, data-export preparation, or admin lookup — or re-fetching after a
  retry. Applies to any screen that fetches.
- **Copy intent & voice:** usually **wordless** — a shape, not a sentence. Where a wait is unusually long,
  a brief reassuring line ("Still loading…") is permitted, never an apology or a progress fiction. For a
  long, known-slow operation (data-export generation) the copy names the operation ("Preparing your
  export…") so the wait is understood.
- **Recovery affordance:** a load must never be a trap. If loading exceeds a reasonable wait, the surface
  offers a **Try again** (which transitions to `error`-style recovery in place) rather than leaving an
  indefinite spinner with no exit.
- **Composition:** the **Loading/skeleton state block** (visual-system §3.14) — shape-matched skeletons
  (chirp skeleton, profile skeleton) on `bg.subtle`, so arriving content does not shift the layout; a
  **spinner** only for in-control/`busy_submitting` actions, not for whole-surface loads. The already-loaded
  chrome (nav, identity indicator) is **not** blocked by a content load.
- **Per-screen instantiation:** the surface section picks the matching skeleton shape for its screen
  (list skeleton for timelines/search, header+list skeleton for profile, row skeleton for settings/admin);
  the determinate progress bar (visual-system §3.15) is reserved for export generation.

###### 1.2 `empty`

- **When it occurs:** the fetch succeeded but the set is empty. The crucial design point is that **"empty"
  has several distinct causes that must read differently** — a bare blank area is never acceptable.
- **Copy intent & voice:** explain *why it is empty* and offer *what to do next*. Distinguish the variants:
  - **Logged-out landing, no content yet** — say what chirp is and route to joining.
    *Exemplar:* "chirp is where you post short updates and follow people. Nothing here yet — be the first."
    + *Create account* / *Sign in*.
  - **New-user / quiet timeline** (follows no one or few) — point at discovery, not a dead end.
    *Exemplar:* "Your timeline is quiet. Follow a few people to start seeing chirps." + *Find people* /
    *Compose*.
  - **No results** (search, admin lookup no match) — confirm the search ran and suggest broadening.
    *Exemplar:* "No results for that search. Try a different handle or term."
  - **Profile with no chirps** — state it plainly; on own profile invite a first post.
    *Exemplar:* "@handle hasn't chirped yet." (own) "You haven't chirped yet — write your first one."
- **Recovery affordance:** the **primary next action for that surface** — sign up / compose / find people /
  adjust the search — never a blank screen the user must leave.
- **Composition:** the **Empty state block** (visual-system §3.14): icon + title + supportive copy + one
  primary action. Obeys ia-interaction §3.5 ("empty states are first-class, state what they are, offer the
  next action").
- **Per-screen instantiation:** each surface section selects the applicable variant(s) from the list above
  for its screen and writes the exact copy; the logged-out-landing and new-user variants are the two that
  most affect first-impression and retention (J1, J2, REQ-PROD-014/022).

###### 1.3 `error`

- **When it occurs:** a fetch or a submit failed (transient network failure, server error, or a failed
  action). Applies to any screen that fetches or submits.
- **Copy intent & voice:** honest, specific, and non-blaming — name *what* failed and that it can be
  retried; never a raw error code, never a false success, never a blank/broken screen.
  *Exemplars:* page/section — "Something went wrong loading your timeline." + *Try again*; submit —
  "Your chirp didn't send. Your text is safe — try again."; conflict/validation-adjacent — handled as
  `validation` (§1.7) where it is about the input, as `error` where it is about the request failing.
- **Recovery affordance:** **Retry in place, without a full page reload** (REQ-PROD-018), and **preserve
  any user input** — a compose draft, a profile edit, a report reason survives the failure so the user
  retries without retyping (REQ-PROD-017).
- **Composition:** for a **page/section** failure, the **Error state block** with a **Retry** that refetches
  in place (visual-system §3.14); for a **form/action** failure, the **inline alert** at the form
  (visual-system §3.13) with the draft kept — not a toast the user can miss for a blocking failure.
  Transient, non-blocking notices may use a toast with *Retry* (§3.12). Obeys ia-interaction §3.1/§3.5.
- **Per-screen instantiation:** fetch-heavy screens (timeline, profile, thread, search, admin) use the
  section Error block; submit screens (sign-up, sign-in, compose, edit-profile, report, export request)
  use the inline alert with input preserved. The surface section writes the specific line.

###### 1.4 `not_found`

- **When it occurs:** the specific thing addressed does not exist or is gone — an unknown handle/id
  profile, an unknown or removed chirp/thread, or an attempt to reach a **deleted** account
  (REQ-PROD-012/019).
- **Copy intent & voice:** state plainly that the thing does not exist or was removed, and **distinguish
  "never existed" from "deleted"** where the product knows which.
  *Exemplars:* "That profile doesn't exist." / "This account has been deleted." / "This chirp is
  unavailable." Avoid a redirect that hides the cause and avoid a crash.
- **Recovery affordance:** a **route back to a known place** — Home timeline, Search, or Back.
- **Composition:** the **Not-found state block** (visual-system §3.14) for a whole surface; the **chirp
  `tombstone` variant** (visual-system §3.6) for a single missing chirp inside an otherwise-loaded list or
  thread ("This chirp is unavailable", no actions). Obeys ia-interaction §3.5.
- **Per-screen instantiation:** `SCR-profile` and `SCR-chirp_thread` use the block for a missing
  subject and the tombstone for a missing item in a list; `SCR-account_deletion`'s post-delete
  not-found is the "deleted" wording (ties to REQ-PROD-012); `SCR-admin_support` lookup-with-no-match is an
  `empty` (no match for a query), not a `not_found` (a named thing that is gone) — the surface sections keep
  that distinction.

###### 1.5 `end_of_list`

- **When it occurs:** a paginated list (home timeline, profile chirps, search results) has returned its
  last page — the user has reached the bottom.
- **Copy intent & voice:** make the end **explicit and positive**, clearly different from `empty` (empty =
  nothing at all; end-of-list = you've seen it all).
  *Exemplars:* "You're all caught up." / "No more results."
- **Recovery affordance:** a sensible onward action — refresh for new, back to top, or broaden the search —
  rather than an ambiguous dead stop or an endless spinner probing for a non-existent next page.
- **Composition:** the **End-of-list state block** (visual-system §3.14) — a subtle terminal marker at the
  foot of the list, distinct from the Empty block.
- **Per-screen instantiation:** the surface sections owning paginated lists (`core_loop` for home timeline,
  profile chirps, search) instantiate it at the list foot; non-paginated surfaces do not show it.

###### 1.6 `session_expired` (satisfies REQ-PROD-006)

- **When it occurs:** the user attempts an **authenticated action** (compose, like, follow, reply, edit,
  delete, export, report, or any signed-in surface fetch) **after the session has expired or been
  invalidated** (including after a service restart, which invalidates in-memory sessions today).
- **Copy intent & voice:** a **clear re-sign-in prompt** that says the session ended and the action needs a
  fresh sign-in — **never a silent fail and never a false success.**
  *Exemplar:* "Your session has expired. Sign in again to continue." + *Sign in*.
- **Recovery affordance:** route to sign-in **preserving the user's context and intent** so they resume the
  exact action afterwards (return-to-intent, ia-interaction §2.4/§3.3); **do not discard in-progress input**
  (a composed draft, an edit) without warning.
- **Composition:** the **Session-expired state block / prompt** (visual-system §3.14) — inline on the
  attempted action or as an interstitial — routed through the **auth-gating pattern** (ia-interaction §3.3)
  and reflected by the **state indicator** flipping to logged-out (ia-interaction §3.6). The session's
  lifetime and invalidation mechanism are `REQ-ID-*`; only the prompt and the resume are designed here.
- **Per-screen instantiation:** every signed-in surface whose `screen_states` lists `session_expired`
  (home timeline, compose, profile, thread, settings home, edit-profile, deletion, export, report, admin)
  routes its expired-session case through this single prompt, not a per-screen message. This is the state
  the whole "no silent sign-out" guarantee (REQ-PROD-006) hangs on.

###### 1.7 `validation`

- **When it occurs:** a form field's input violates a rule — handle rules (≤15 chars, allowed characters)
  and uniqueness at sign-up/edit, the ≤280-character compose limit, required fields, a report reason — at
  or before submit.
- **Copy intent & voice:** **inline, field-level, actionable, and specific** — point at the field and say
  the fix, in plain language, never a generic "invalid input" and never a raw server/API error surfaced to
  the user.
  *Exemplars:* "Handles can be up to 15 characters." / "Only letters, numbers, and underscores." /
  "That handle is taken — try another." / "Your chirp is over the 280-character limit."
- **Recovery affordance:** the message points to the exact field and the correction; the **submit control's
  state is honest** — disabled while the input is invalid *or* an error state on attempted submit, one
  consistent choice per form — and **typed text is never lost** when a submit is blocked.
- **Composition:** the **input/textarea error and validated states** (visual-system §3.3) and the
  **character counter** with its approaching-limit/over-limit states (§3.4); the message sits in the field's
  message slot, never only as a page-level error. Obeys ia-interaction §3.5 (validation is inline and
  actionable). Server-side uniqueness that can only be known on submit is surfaced back into the field's
  message slot, not as a bare request error.
- **Per-screen instantiation:** the form-bearing surfaces (`entry_auth` sign-up/sign-in/recovery,
  `core_loop` compose, `account_settings` edit-profile, `trust_safety_admin` report) instantiate the
  field-level messages for their own fields; the compose over-limit case also blocks the submit per §3.4.

###### 1.8 `busy_submitting`

- **When it occurs:** a submission is in flight — posting a chirp, saving a profile, following/liking,
  confirming a deletion, requesting an export, filing a report, signing in/up/out.
- **Copy intent & voice:** minimal — the control itself shows the work ("Posting…", or a spinner on the
  control); the point is to **show it is working and prevent a duplicate submission**, not to narrate.
- **Recovery affordance:** on **failure**, return to an **editable state with the input intact** and show
  the `error` treatment (§1.3); the control never strands the user in a permanent busy state. For
  **optimistic** reversible actions (like/unlike, follow/unfollow), the control may update ahead of
  confirmation but MUST **reconcile to the true state on failure** (ia-interaction §3.2).
- **Composition:** the **§3.0 busy control state** (visual-system) — spinner, non-interactive, **size
  preserved so the layout does not jump**; the **follow/like controls' busy→reverted** visuals
  (visual-system §3.7/§3.8) for the optimistic case. Obeys ia-interaction §3.1 (no silent completion).
- **Per-screen instantiation:** every submit-bearing surface uses the busy control state; the two
  optimistic controls (like, follow) are the only ones permitted to update before confirmation, and only
  with visible reconciliation on failure — irreversible/destructive actions (deletion) are **never**
  optimistic (ia-interaction §3.2/§3.4).

###### 1.9 `success`

- **When it occurs:** an action completed — chirp posted, profile saved, follow/like applied, report filed,
  export ready, account deletion confirmed, signed in/out.
- **Copy intent & voice:** an **honest confirmation that the action completed**, brief and specific; the
  result must be **observable** (the new chirp appears, the setting shows saved, the export offers a
  download). **Never a false success** — a success state is shown only when the action truly succeeded.
  *Exemplars:* "Chirp posted." / "Profile saved." / "Report received." / "Your export is ready." +
  *Download*.
- **Recovery affordance:** n/a as a positive terminal — but the success must **reconcile optimistic UI to
  truth**: if an optimistically-shown success turns out to have failed, it reverts and surfaces the error
  (ia-interaction §3.2). The *observable result* is itself the confirmation of record.
- **Composition:** a **toast** for a brief success (visual-system §3.12) or an **inline success alert**
  where a persistent confirmation helps (§3.13); the optimistic control reconciled to its true state
  (§3.7/§3.8). Visual-system §4 maps `success` to toast/inline + reconciled control. Obeys ia-interaction
  §3.1.
- **Per-screen instantiation:** quick actions (post, like, follow, save) confirm by the result appearing
  plus a toast; longer or consequential actions (export ready, deletion done, report filed) use the inline
  confirmation and, for deletion, hand off to the signed-out + not-found treatment (§1.4, REQ-PROD-012).

---

##### 2. Cross-cutting rules (bind every state and every surface section)

1. **One state, one pattern, product-wide.** A state is instantiated from the single model above on every
   screen it applies to; a surface section never hand-rolls a screen-specific shape for a state (visual-system
   §5.3). The per-screen state matrix is each surface section's **coverage checklist**: every
   state listed for a screen must be *designed* (copy intent + recovery), not merely named; a state
   the section drops must be recorded out-of-scope with a
   reason, never silently omitted.
2. **Honest resolution of every action.** Every action resolves into exactly one observable outcome from
   the vocabulary — `busy_submitting` → `success` or `error` — with no silent completion and no false
   success (ia-interaction §3.1).
3. **Recover without a full page reload.** `error` and over-long `loading` recover in place (REQ-PROD-018);
   the already-loaded chrome is never thrown away to show a content-area state.
4. **Preserve the user's work.** No state discards an in-progress draft/edit/reason — on `error`,
   `validation`, `busy_submitting` failure, or `session_expired` (REQ-PROD-017).
5. **Expired sessions always route through the one re-sign-in prompt** (§1.6, REQ-PROD-006), never a dead
   end and never a false success.
6. **Optimism only for reversible actions, always reconciled** (like/follow); never for destructive ones
   (ia-interaction §3.2/§3.4).
7. **No state carried by colour alone** — icon/text/weight accompanies every state signal (visual-system
   §3.0/§3.13, REQ-UX-073).
8. **Compose, don't invent.** Every state is rendered by a named visual-system block/component; a state the
   component set cannot serve is **flagged for resolution**, never invented here.

---

##### 3. Composition check — the model maps cleanly onto the existing components (no gap)

Every state in the vocabulary is renderable by an existing visual-system component and obeys an existing IA
pattern; no new component is required, so none is invented and nothing is flagged.

| State | Visual-system component (renders it) | IA pattern (governs it) |
|---|---|---|
| `content` | Primary component — chirp card/list §3.6, profile §3.5, settings list §3.15 | screen map §2 |
| `loading` | Loading/skeleton block §3.14; in-control spinner §3.0/§3.15 | state surfacing §3.5 |
| `empty` | Empty block §3.14 | §3.5 (first-class empties) |
| `error` | Error block + Retry §3.14 (page/section); inline alert §3.13 (form) | honest feedback §3.1/§3.5 |
| `not_found` | Not-found block §3.14; chirp tombstone §3.6 | §3.5 |
| `end_of_list` | End-of-list block §3.14 | §3.5 |
| `session_expired` | Session-expired prompt block §3.14 | auth-gating §3.3; indicator §3.6 |
| `validation` | Input error/validated §3.3 + character counter §3.4 | §3.5 (inline, actionable) |
| `busy_submitting` | §3.0 busy control; follow/like busy §3.7/§3.8 | feedback §3.1; optimistic §3.2 |
| `success` | Toast §3.12 / inline success §3.13; reconciled control §3.7/§3.8 | §3.1; optimistic reconcile §3.2 |

This matches visual-system §4's own state→component mapping; the two are consistent.

---

##### 4. Requirements & acceptance criteria — REQ-UX-020 .. REQ-UX-029

Range owned by the state model. Each requirement is stated
from the user's side, is testable by a stranger exercising the live product, carries a `release_gating`
flag, and traces to ≥1 `REQ-PROD-*` and/or journey. These govern the state model the surface sections
instantiate per screen.

###### REQ-UX-020 — One consistent state model across the product
- **Statement:** Loading, empty, error, not-found, end-of-list, session-expired, validation, busy, and
  success are presented as one consistent, recognisable model wherever they occur, rather than handled
  differently on each screen.
- **Acceptance criteria:** Moving across the timeline, a profile, compose, search, settings, and a form, a
  person finds the same state recognisably the same — an error looks and recovers the same on a fetch and a
  submit; an empty state always states what it is and offers a next action; no screen shows a bespoke,
  one-off version of a state.
- **Release gating:** true
- **Traces to:** REQ-PROD-014, REQ-PROD-018, REQ-PROD-019; J1, J5, J6.

###### REQ-UX-021 — Loading is shown in place and never traps the user
- **Statement:** While a surface is fetching, the product shows in-place loading that does not blank the
  screen or jump the layout, and a slow load always offers a way out.
- **Acceptance criteria:** On a first load of the timeline, a profile, search, or settings, the user sees
  shape-matched placeholders (not a blank page and not a full-screen spinner that replaces the chrome), and
  when content arrives the layout does not jump; if a load stalls beyond a reasonable wait, a retry is
  offered rather than an indefinite spinner with no exit.
- **Release gating:** true
- **Traces to:** REQ-PROD-018, REQ-PROD-019, REQ-PROD-014; J1, J5.

###### REQ-UX-022 — Empty states explain why and offer the next action
- **Statement:** Every empty surface explains why it is empty and offers the primary next action, and the
  different causes of emptiness read differently.
- **Acceptance criteria:** A logged-out visitor landing on an empty timeline sees what chirp is plus a way
  to sign up/sign in; a new or quiet signed-in timeline offers finding people or posting; an empty search
  says no results and suggests broadening; a profile with no chirps says so — and none of these is a bare
  blank area.
- **Release gating:** true
- **Traces to:** REQ-PROD-014, REQ-PROD-018, REQ-PROD-022, REQ-PROD-024; J1, J5, J6.

###### REQ-UX-023 — Errors are honest, recoverable in place, and never lose input
- **Statement:** A failed load or action shows an honest, plain-language message, recovers without a full
  page reload, and preserves any text the user had entered.
- **Acceptance criteria:** When a fetch fails, the surface shows a clear message (not a blank, broken, or
  raw-code screen) and a retry that refetches in place without reloading the whole page; when a submit
  fails (for example a chirp send), the typed text remains and the user can retry without retyping.
- **Release gating:** true
- **Traces to:** REQ-PROD-018, REQ-PROD-017, REQ-PROD-019; J1, J5.

###### REQ-UX-024 — Not-found is an honest designed state with a way back
- **Statement:** When the specific profile, chirp, or account addressed does not exist or has been removed,
  the product says so plainly and offers a route back, distinguishing "deleted" from "never existed" where
  it knows.
- **Acceptance criteria:** Opening an unknown handle shows a clear "doesn't exist" result with a route to
  home/search, not a crash or a silent redirect; reaching a deleted account says it was deleted; a missing
  chirp inside a list shows an unavailable placeholder rather than vanishing without explanation.
- **Release gating:** true
- **Traces to:** REQ-PROD-019, REQ-PROD-012; J5, J7.

###### REQ-UX-025 — End-of-list is explicit and distinct from empty
- **Statement:** When a paginated list has no more pages, the product makes the end explicit and clearly
  different from an empty list.
- **Acceptance criteria:** Paging to the bottom of the home timeline, a profile's chirps, or search
  results shows a clear end marker ("you're all caught up" / "no more results") with a sensible onward
  action, and this is visibly different from the empty-state message shown when there was nothing at all.
- **Release gating:** true
- **Traces to:** REQ-PROD-018; J5.

###### REQ-UX-026 — Session expiry prompts a re-sign-in and never fails silently (REQ-PROD-006)
- **Statement:** When a user's session has expired or been invalidated, the next action they attempt
  surfaces a clear prompt to sign in again — never a silent failure and never an apparent success — and
  signing in returns them to what they were doing without losing their input.
- **Acceptance criteria:** After a session ends (including after a service restart), attempting a signed-in
  action (compose, like, follow, edit, delete, export, report) shows a clear "session expired, sign in
  again" prompt; the action does not silently do nothing and does not appear to succeed; after signing in,
  the user is returned to the attempted action and any text they had entered is still there.
- **Release gating:** true
- **Traces to:** REQ-PROD-006; J3, J6.

###### REQ-UX-027 — Validation is inline, actionable, and at or before submit
- **Statement:** Form fields show validation at the field, in language the user can act on, at or before
  submission, with an honest submit-control state and no loss of typed text.
- **Acceptance criteria:** Entering an over-length or disallowed handle, a taken handle, or an over-limit
  chirp produces a clear inline message at the field explaining the fix (not a generic "invalid input" and
  not a raw API error), at or before submit; the submit control's state is honest (disabled or errored
  consistently); and the user's typed text is preserved when submission is blocked.
- **Release gating:** true
- **Traces to:** REQ-PROD-002, REQ-PROD-003, REQ-PROD-016; J2, J5.

###### REQ-UX-028 — Submissions show a busy state, prevent duplicates, and reconcile on failure
- **Statement:** Every submission shows it is in flight, prevents a duplicate submission, and on failure
  returns to an editable state with input intact; optimistic reversible actions reconcile to the true state
  when they fail.
- **Acceptance criteria:** Posting, saving a profile, following, liking, requesting an export, or
  confirming a deletion each show a busy indicator (without the layout jumping) and cannot be double-fired
  while in flight; a failed submit returns the user to an editable state with their input and an honest
  error; a failed like/follow visibly reverts to the true state and count rather than leaving a wrong one.
- **Release gating:** true
- **Traces to:** REQ-PROD-016, REQ-PROD-017, REQ-PROD-020, REQ-PROD-021; J5.

###### REQ-UX-029 — Success is honest, observable, and never false
- **Statement:** The product confirms a completed action honestly and makes its result observable, and
  never shows success for an action that did not succeed.
- **Acceptance criteria:** Posting a chirp, saving a profile, filing a report, or completing an export
  shows a clear confirmation and the result is observable (the chirp appears, the setting shows saved, the
  export is downloadable); an action that actually failed is never shown as succeeded — any optimistic
  success that fails reverts and reports the failure.
- **Release gating:** true
- **Traces to:** REQ-PROD-016, REQ-PROD-020, REQ-PROD-021; J5.

###### 4.1 Traceability summary (REQ-UX → REQ-PROD / journeys)

| REQ-UX | Gating | Traces to |
|---|---|---|
| REQ-UX-020 | true | REQ-PROD-014, 018, 019; J1, J5, J6 |
| REQ-UX-021 | true | REQ-PROD-018, 019, 014; J1, J5 |
| REQ-UX-022 | true | REQ-PROD-014, 018, 022, 024; J1, J5, J6 |
| REQ-UX-023 | true | REQ-PROD-018, 017, 019; J1, J5 |
| REQ-UX-024 | true | REQ-PROD-019, 012; J5, J7 |
| REQ-UX-025 | true | REQ-PROD-018; J5 |
| REQ-UX-026 | true | REQ-PROD-006; J3, J6 |
| REQ-UX-027 | true | REQ-PROD-002, 003, 016; J2, J5 |
| REQ-UX-028 | true | REQ-PROD-016, 017, 020, 021; J5 |
| REQ-UX-029 | true | REQ-PROD-016, 020, 021; J5 |

All ten ids are unique, fall within the owned `REQ-UX-020..029` sub-range, and lie within the parent
`REQ-UX-020..069` range.

---

##### 5. Coverage & composition notes

- **Coverage.** Every state in the screen inventory & state vocabulary (including the `content` baseline) is
  designed above with concrete copy intent and a recovery affordance  — §1.0–§1.9. The the per-screen state matrix seeds are
  elaborated, not redefined.
- **Composition.** The model maps entirely onto existing visual-system components and IA patterns (§3); no
  component is invented, and no gap was found to flag. Should a
  surface section find a screen state it cannot serve with these components, it records that for resolution rather than inventing one.
- **For the four surface sections.** Use the per-screen state matrix as the per-screen coverage
  checklist; for each listed state instantiate the matching §1 pattern (its block + IA pattern + the voice
  in §0.4) and write the exact per-screen copy from the exemplars. `session_expired` always routes through
  §1.6; optimistic UI is confined to like/follow (§1.8); destructive flows are never optimistic and are
  confirmed per ia-interaction §3.4 (the surface sections own the consequence copy).
- **Boundary kept.** This fragment designs *what* each state looks and behaves like and the testable bar —
  no front-end code; session/auth lifetime, deletion/export internals, and persistence are deferred to
  `REQ-ID-*`/`REQ-DOM-*`/`REQ-OPS-*`.

---

#### Screens & flows

*(section: `screens_and_flows` — the four surface groups, assembled)*

The four surface-group fragments below — entry & auth, the signed-in core loop, account lifecycle & settings, and trust/safety/admin — together design every screen in the frozen 18-screen inventory, each with its happy-path flow and every applicable state, instantiating the one state model above.

---

#### Entry & auth — screens & flows

*(section: `screens_and_flows` — fragment: `entry_auth`)*

This fragment designs **the first screens a stranger meets and the path from visitor to signed-in
user**: the logged-out landing / public timeline, create-account, credentialed sign-in, and account
recovery. The bar is *designed screens, not a skeleton* — each screen is given its content/layout
intent at wireframe level, its happy-path flow, and **every state that applies to it** per
the per-screen state matrix, with concrete copy intent and a recovery affordance, so that
someone who was never in the room could find chirp, understand what it is, and join — and get back in
if they lose access.

**What this fragment settles, and what it leaves to its siblings.**

- It **owns** the screen/flow/state design and the `REQ-UX-030..039` register for the four entry/auth
  screens.
- It **instantiates** the cross-cutting state model (`empty_error_loading_states`) per screen; it does
  **not** redefine any state's meaning, voice, or recovery pattern — it selects the applicable
  variant and writes the exact per-screen copy.
- It **composes from** named visual-system components and places each screen per the IA screen map and
  logged-out navigation; it invents **no** component — a gap would be flagged for resolution
  (none was needed — see §5).
- It designs **UX only — what, not how.** The **credential mechanism** (what a user signs in with and
  how it is verified, session lifetime, the recovery factor) is identity & trust's (`REQ-ID-*`); this
  fragment designs the screen, the flow, the fields, and the feedback a user sees. Where a flow depends
  on a credential, it is designed as a generic **credential/secret** field so the UX stands whatever
  `REQ-ID-*` chooses.
- Destinations **after** authentication (signed-in home timeline, onboarding first-run) are owned by
  `core_loop` and the IA transition (§2.4); this fragment designs the entry/auth screens and hands off
  to those destinations, it does not design them.

**The starting point it improves (grounded at `c805525`).** Today there is no landing screen distinct
from the raw timeline, **no sign-up screen at all** (account creation lives only at the API), sign-in
is a **handle-only** field folded into the compose view (anyone who types a handle becomes that user),
and **no recovery**. The current timeline already renders honest loading/content/empty/error/paged
states and preserves drafts via uncontrolled inputs — those are kept and built on; the gaps
(a real landing, in-product sign-up, credentialed sign-in as its own destination, recovery) are closed.

---

##### 1. Shared entry/auth conventions (bind all four screens)

So the four screens read as one coherent entry flow rather than four unrelated forms:

- **Logged-out chrome.** All four render inside the logged-out navigation (ia-interaction §1.1): the
  `chirp` wordmark → landing, and persistent **Sign in** + **Create account** CTAs (visual-system nav
  §3.9), with a low-prominence **Privacy & terms** in the footer. The state indicator shows the
  logged-out affordances (ia-interaction §3.6).
- **One primary action per screen** (visual-system §3.1 / §5.4): *Create account* on sign-up, *Sign in*
  on sign-in, *Send recovery link* / *Set new credential* on recovery, *Create account* on the landing
  empty-state. Secondary routes are text links (§3.2).
- **Forms share one anatomy** (visual-system §3.3): label · field · helper text · inline **message
  slot**. Validation is inline, field-level, and at/before submit (state model §1.7); the submit
  control's busy/disabled state is honest (state model §1.8); **typed input is never lost** on a
  validation bounce, a failed submit, or a re-render (REQ-PROD-017 draft-safety, carried over from the
  current uncontrolled-input behaviour).
- **The three form screens share one state set** — `validation`, `busy_submitting`, `error`, `success`
  (+ `content`) — each instantiated from the state model (§1.7/§1.8/§1.3/§1.9) with the shared voice
  (state-model §0.4): plain, calm, specific, non-blaming; a one-line title, at most a sentence, then a
  verb-labelled action.
- **Interlinking (no dead ends).** Sign-in ↔ Sign-up are mutually linked; recovery is reached from
  sign-in ("Can't sign in?"); the landing routes to both sign-up and sign-in (ia-interaction
  reachability §1.4). Every screen offers a way onward.
- **Credential boundary.** Sign-up and sign-in show a **credential field** (a secret the user sets and
  later presents) with entry, show/hide, and inline validation designed here; **what** the credential is
  and how it is verified is `REQ-ID-*`. The design must not hard-code handle-only sign-in — it closes
  that gap by presenting a credential field whose mechanism is deferred.
- **Honest success + transition.** A successful sign-up or sign-in routes to a **purposeful signed-in
  destination** and swaps the chrome to signed-in (ia-interaction §2.4) — never back to the logged-out
  landing; return-to-intent is honoured when auth was triggered by a gated action.

---

##### 2. SCR-landing_timeline — logged-out landing / public home timeline

**Role.** The first screen a visitor (persona P1) meets. It must, in the first screen, make clear what
chirp is, show real content when there is any, and give an obvious route to join — **reading requires
no account** (REQ-PROD-013). Serves J1; REQ-PROD-013, REQ-PROD-014, REQ-PROD-015.

**Content / layout intent (wireframe).**

```
┌───────────────────────────────────────────────────────────┐
│  chirp                               [ Sign in ] [Create account] │  logged-out nav §3.9
├───────────────────────────────────────────────────────────┤
│  chirp — short public posts from people you follow.         │  one-line "what chirp is"
│  Read anything here; create an account to post and follow.  │  (shown above/with the feed;
│                                                   [Create account]│   persistent, not a wall)
├───────────────────────────────────────────────────────────┤
│  ┌ chirp card (§3.6) ─────────────────────────────────────┐ │
│  │ (avatar) Display name  @handle · 2h           (⋯)       │ │  reading is public; the
│  │ chirp body text, line breaks preserved…                 │ │  author link opens a
│  │ ♡ 12                                                     │ │  read-only profile (contextual)
│  └─────────────────────────────────────────────────────────┘ │
│  … more chirps (newest first) …                             │
│  [ Load more ]                                              │
├───────────────────────────────────────────────────────────┤
│  About chirp · Privacy & terms                              │  low-prominence footer
└───────────────────────────────────────────────────────────┘
```

- Single readable column (`layout.content.max` 600px, visual-system §2.6). The timeline is the
  product's substance, so content is the hero; a slim, persistent "what chirp is" line + a **Create
  account** CTA sit with the feed so a visitor understands the product and sees the join route **without
  the content being walled off** behind a login (auth-gating: read freely, ia-interaction §3.3).
- Chirps render with the one chirp card (visual-system §3.6) and identity cluster (§3.5); like/follow
  controls appear but, for a logged-out visitor, route to sign-in on activation (the control is
  disabled→prompts, visual-system §3.7/§3.8; the gating is IA's §3.3 — not a dead control).

**Happy-path flow.** Visitor arrives → reads the public timeline (newest first) → opens an author's
profile from a chirp if curious → decides to join → taps **Create account** (→ SCR-signup) or **Sign
in** (→ SCR-signin).

**States** (matrix: `loading, empty, error, end_of_list`; `content` = the feed above):

- **`loading`** (state model §1.1). First fetch of the public timeline. **Shape-matched chirp
  skeletons** on `bg.subtle` (visual-system §3.14), never a blank page or a full-screen spinner; the
  nav, the "what chirp is" line, and the Create-account CTA are **not** blocked by the content load, so
  a visitor understands the product and can join even while the feed loads. *Recovery:* if the load
  stalls beyond a reasonable wait, offer **Try again** rather than an endless spinner.
- **`empty`** (state model §1.2, "logged-out landing, no content yet" variant). This is the
  make-or-break first impression on a fresh or just-restarted deployment: **never a bare blank area.**
  *Copy intent:* say what chirp is and invite joining. *Exemplar:* **"chirp is where you post short
  updates and follow people. Nothing here yet — be the first."** *Recovery / next action:* the primary
  **Create account** (and secondary **Sign in**) — the empty state is itself a working join screen, not
  a dead end (REQ-PROD-014).
- **`error`** (state model §1.3). The timeline failed to load. *Copy intent:* honest, non-blaming, no
  raw code or broken page. *Exemplar:* **"Something went wrong loading the timeline."** *Recovery:*
  **Try again** that refetches **in place, without a full page reload** (REQ-PROD-018); the chrome and
  join route remain usable so a visitor can still sign up even if the feed is down. Rendered by the
  Error state block (visual-system §3.14).
- **`end_of_list`** (state model §1.5). The visitor paged to the bottom. *Copy intent:* explicit,
  positive end — distinct from empty. *Exemplar:* **"You're all caught up."** *Recovery:* offer
  **Back to top** / **Refresh for new**; and, since this is a logged-out surface, a gentle **Create
  account to post** reinforcing the join route at the natural stopping point. End-of-list state block
  (§3.14), visibly different from the Empty block.

**Boundary notes.** Reading the timeline and profiles never prompts for an account (REQ-PROD-013);
only an *action* (post/like/follow) gates to sign-in. The signed-in home timeline (`SCR-home_timeline`)
and its additional states (`session_expired`, `busy_submitting`, `success`) are `core_loop`'s, not
this screen's.

---

##### 3. SCR-signup — create account

**Role.** In-product self-service account creation, reachable from the landing and from sign-in; closes
the "no sign-up screen" gap. Handle validation and uniqueness feedback are designed here; the credential
mechanism is `REQ-ID-*`. Serves J2; REQ-PROD-001, REQ-PROD-002, REQ-PROD-003, REQ-PROD-015.

**Content / layout intent (wireframe).**

```
┌───────────────────────────────────────────────────────────┐
│  chirp                                   [ Sign in ]        │
├───────────────────────────────────────────────────────────┤
│  Create your account                                        │  font.display / h1
│                                                             │
│  Handle                                                     │  input §3.3 + counter §3.4 (≤15)
│  [ @ yourhandle              ]  12/15                       │  message slot: rules / "taken"
│  Letters, numbers, and underscores. Up to 15 characters.    │  helper text (font.meta)
│                                                             │
│  Display name                                               │  input §3.3
│  [ Your name                 ]                              │
│                                                             │
│  Credential  (set a password / secret — mechanism: REQ-ID)  │  input §3.3, type=secret, show/hide
│  [ ••••••••••            ] (show)                           │  message slot: strength/err
│                                                             │
│  Bio (optional)                                             │  textarea §3.3
│  [                          ]                               │
│                                                             │
│  [ Create account ]   ← one primary (§3.1)                  │
│  Already have an account?  Sign in                          │  link §3.2 → SCR-signin
│  By joining you agree to the Privacy & terms.               │  link → SCR-privacy_terms
└───────────────────────────────────────────────────────────┘
```

- Fields: **handle** (with the character counter §3.4 enforcing ≤15 and the handle rules), **display
  name**, a **credential** field (secret; show/hide; the actual factor is `REQ-ID-*`), and an optional
  **bio**. One **Create account** primary; a **Sign in** text link for people who already have an
  account; a Privacy & terms link (REQ-PROD-029 surface, owned by `trust_safety_admin` — referenced,
  not designed here).

**Happy-path flow.** Visitor taps *Create account* → fills handle (sees live rule/uniqueness feedback)
→ sets a credential → optionally display name/bio → **Create account** → `busy` → account created and
signed in → **transition to a purposeful signed-in destination**: the onboarding first-run for a brand
new account where present (`SCR-onboarding`, non-gating), else the signed-in home timeline — never the
logged-out landing (ia-interaction §2.4; the destination is `core_loop`/IA's).

**States** (matrix: `validation, busy_submitting, error, success`):

- **`validation`** (state model §1.7; visual-system §3.3/§3.4). Inline, field-level, actionable, at or
  before submit — never a generic "invalid input" or a raw API error. *Handle rules:* **"Handles can be
  up to 15 characters."** / **"Only letters, numbers, and underscores."** (the counter §3.4 shows
  remaining and turns over-limit). *Uniqueness* (knowable only on submit) is surfaced **back into the
  handle field's message slot**, not as a bare request error: **"That handle is taken — try another."**
  (REQ-PROD-003). *Required fields* (handle, credential) are flagged at the field. *Recovery:* the
  message points at the exact field and the fix; the submit control is honestly disabled-while-invalid
  or errors on attempted submit (one consistent choice); **typed text is preserved**.
- **`busy_submitting`** (state model §1.8; visual-system §3.0). On submit the **Create account** button
  shows an in-flight state ("Creating account…"), is non-interactive, **holds its size** (no layout
  jump), and **prevents a duplicate submission**. *Recovery:* on failure, return to an editable form
  with every field's input intact and show the `error` treatment.
- **`error`** (state model §1.3; visual-system §3.13 inline alert). A submit that failed for a reason
  other than field validation (a transient/server failure). *Copy intent:* honest and recoverable.
  *Exemplar:* **"We couldn't create your account just now. Your details are safe — try again."**
  *Recovery:* retry **without losing the entered handle / display name / bio**; a handle-uniqueness
  conflict is routed to the field's `validation` slot (above), not shown as a bare error.
- **`success`** (state model §1.9). *Copy intent:* honest confirmation whose observable result is being
  signed in and landed on the next destination (the account exists and the user is in). Rather than a
  throwaway toast on a screen the user is leaving, success **is** the transition into the signed-in
  destination (ia-interaction §2.4); an optional **"Welcome to chirp, @handle."** inline confirmation on
  arrival. **Never a false success** — shown only once the account is actually created and the session
  established.

**Boundary note.** What a "credential" is, how it is stored/verified, and session establishment are
`REQ-ID-*`. The onboarding first-run content is `core_loop`'s (`SCR-onboarding`, non-gating,
REQ-PROD-022); this screen only hands off to it.

---

##### 4. SCR-signin — credentialed sign-in

**Role.** A credentialed sign-in **as its own destination** (replacing today's handle-only field folded
into compose), with visible routes to sign-up and recovery. Closes the "anyone who knows a handle can
sign in" gap at the UX level: a credential is presented and feedback is honest; the verification is
`REQ-ID-*`. Serves J3; REQ-PROD-004, REQ-PROD-006, REQ-PROD-015.

**Content / layout intent (wireframe).**

```
┌───────────────────────────────────────────────────────────┐
│  chirp                                   [ Create account ] │
├───────────────────────────────────────────────────────────┤
│  Sign in to chirp                                           │  h1
│                                                             │
│  Handle                                                     │  input §3.3
│  [ @ yourhandle              ]                              │
│                                                             │
│  Credential  (mechanism: REQ-ID)                            │  input §3.3, type=secret, show/hide
│  [ ••••••••••            ] (show)                           │
│                                                             │
│  [ Sign in ]   ← one primary (§3.1)                         │
│  Can't sign in?                                             │  link §3.2 → SCR-recovery
│  New to chirp?  Create account                              │  link §3.2 → SCR-signup
└───────────────────────────────────────────────────────────┘
```

- Fields: **handle** and a **credential** (secret, show/hide). One **Sign in** primary. A **"Can't sign
  in?"** link to recovery (so a locked-out user finds recovery exactly where they look — ia-interaction
  §1.1) and a **Create account** link for new visitors.
- This screen is also the destination the **auth-gating prompt** and the **`session_expired`** prompt
  route to (ia-interaction §3.3; state model §1.6): when a logged-out or expired user attempts a gated
  action, they arrive here with their **intent preserved**, and on success are returned to that action
  (return-to-intent) — this screen presents the credential capture for that flow.

**Happy-path flow.** Returning user opens Sign in → enters handle + credential → **Sign in** → `busy` →
authenticated → chrome swaps to signed-in and the user lands on the signed-in home timeline (or is
returned to the action that sent them here) — ia-interaction §2.4.

**States** (matrix: `validation, busy_submitting, error, success`):

- **`validation`** (state model §1.7; visual-system §3.3). Inline, pre-submit: required handle and
  credential flagged at the field (**"Enter your handle."** / **"Enter your credential."**). *Recovery:*
  points at the field; submit honestly disabled-until-complete; typed input preserved.
- **`busy_submitting`** (state model §1.8; visual-system §3.0). **Sign in** shows "Signing in…", is
  non-interactive, holds size, prevents a double submit. *Recovery:* on failure return to the editable
  form with input intact and show the `error` treatment.
- **`error`** (state model §1.3; visual-system §3.13 inline alert at the form — not a toast, because the
  user must act on it). *Copy intent:* honest, non-enumerating, non-blaming — **does not reveal whether
  the handle exists** (an invalid handle and a wrong credential read the same, so sign-in is not an
  account-enumeration oracle — a trust posture whose enforcement is `REQ-ID-*`, surfaced here as the
  copy choice). *Exemplar:* **"That handle and credential don't match. Check them and try again."**
  *Recovery:* retry in place with the handle preserved; the **"Can't sign in?"** recovery route is
  present for a user who has genuinely lost access. Today's view already renders a server error inline;
  this keeps that and makes the message honest and understandable (not a raw `Unauthorized`).
- **`success`** (state model §1.9). The honest, observable result is being signed in and landed on the
  signed-in destination with the identity indicator now showing `@handle` (ia-interaction §3.6).
  **Never a false success** — the signed-in state is shown only on a real authentication. (A failed
  sign-in never appears to succeed — REQ-PROD-006.)

**Session note (REQ-PROD-006, state model §1.6).** This screen is the single re-entry point for
`session_expired`: when a session expires or is invalidated (including after a service restart, which
invalidates in-memory sessions today), the next authenticated action routes here with a **"Your session
has expired. Sign in again to continue."** prompt and returns the user to what they were doing — never a
silent sign-out or a false success. The `session_expired` state itself is designed in the state model
and fires on the signed-in surfaces (`core_loop` et al.); this screen is where it lands. Session
lifetime/invalidation mechanism is `REQ-ID-*`.

---

##### 5. SCR-recovery — account recovery

**Role.** A safe path back in for a user who has lost access to their credential, reached from sign-in
("Can't sign in?"). The recovery **factor and mechanism** are `REQ-ID-*`; the screen, the flow, and the
honest feedback are designed here. Serves J4; REQ-PROD-007.

**Content / layout intent (wireframe) — a two-step flow.**

```
Step 1 — request recovery                     Step 2 — set a new credential
┌─────────────────────────────────┐           ┌─────────────────────────────────┐
│  chirp              [ Sign in ]  │           │  chirp              [ Sign in ]  │
├─────────────────────────────────┤           ├─────────────────────────────────┤
│  Can't sign in?                  │           │  Set a new credential            │
│  Tell us how to reach you and    │           │  Choose a new secret for @handle.│
│  we'll send recovery steps.      │           │                                  │
│                                  │           │  New credential                  │
│  Recovery factor (mechanism:ID)  │  input    │  [ ••••••••••      ] (show)       │
│  [ your recovery factor      ]   │  §3.3     │  Confirm new credential          │
│                                  │           │  [ ••••••••••      ] (show)       │
│  [ Send recovery steps ]         │  primary  │  [ Set new credential ]  primary │
│  Remembered it?  Sign in         │  link     │  (reached via a recovery link/   │
└─────────────────────────────────┘           │   code — carrier is REQ-ID)      │
                                               └─────────────────────────────────┘
```

- **Step 1 (request):** the user supplies the **recovery factor** (what it is — email, backup code,
  etc. — is `REQ-ID-*`; designed here as one labelled field) and requests recovery. One **Send recovery
  steps** primary; a **Sign in** link for a user who remembered.
- **Step 2 (reset):** reached by following the recovery carrier (link/code — the carrier is `REQ-ID-*`).
  The user sets a **new credential** with a **confirm** field; one **Set new credential** primary. On
  success the account is regained (same handle, chirps, follows — REQ-PROD-007) and the user is routed
  to sign in (or signed in directly, per `REQ-ID-*`).

**Happy-path flow.** Locked-out user taps "Can't sign in?" on SCR-signin → enters recovery factor →
**Send recovery steps** → `busy` → `success` ("If that factor matches an account, recovery steps are on
their way.") → user follows the carrier → Step 2 → sets + confirms new credential → **Set new
credential** → `busy` → `success` → signs in and regains the same account.

**States** (matrix: `validation, busy_submitting, error, success` — applied across both steps):

- **`validation`** (state model §1.7; visual-system §3.3). Step 1: the recovery-factor field is
  required and format-checked inline (**"Enter your recovery factor."**). Step 2: the new credential and
  its confirmation are validated inline — **"Enter a new credential."** / **"Those don't match."** —
  with the credential-rule messages (rules themselves are `REQ-ID-*`). *Recovery:* points at the field;
  submit honestly gated; typed input preserved.
- **`busy_submitting`** (state model §1.8; visual-system §3.0). The primary ("Sending…" / "Saving…")
  holds size, is non-interactive, prevents a duplicate request. *Recovery:* on failure, return to the
  editable step with input intact and show `error`.
- **`error`** (state model §1.3; visual-system §3.13 inline alert). An honest, recoverable failure of
  the request or the reset. *Exemplar (request):* **"We couldn't start recovery just now. Try again in
  a moment."** *Exemplar (reset, e.g. an expired recovery link):* **"This recovery link has expired —
  start again from sign-in."** with a route **back to Step 1 / Sign in**. *Recovery:* retry in place; no
  raw code; a dead link routes back rather than crashing.
- **`success`** (state model §1.9). Step 1 success is a **non-enumerating** confirmation — it does
  **not** reveal whether an account exists for the factor (so recovery is not an account-enumeration
  oracle): **"If that matches an account, we've sent the next steps."** Step 2 success is honest and
  observable: **"Your credential is updated — you can sign in now."** routing to SCR-signin (or signed
  in directly). **Never a false success** — shown only on a real send / real reset. (The anti-
  enumeration posture is enforced in `REQ-ID-*`; it is reflected here as the success/error copy choice.)

**Boundary note.** The recovery factor, the carrier (link/code/email), credential rules, token
lifetime, and the anti-enumeration guarantee are `REQ-ID-*`. This screen designs the two-step flow, the
fields, and the honest, non-enumerating feedback a user sees. Richer multi-factor recovery is a later
`REQ-ID-*` concern; the basic recovery path designed here is the gating one (REQ-PROD-007).

---

##### 6. Requirements & acceptance criteria — REQ-UX-030 .. REQ-UX-039

Range owned by `the entry & auth surfaces`. Each requirement is stated
from the user's side, is testable by a stranger exercising the live product, carries a `release_gating`
flag, and traces to ≥1 `REQ-PROD-*` and/or journey. These are **screen-level** entry/auth requirements;
the product-wide navigation/state rules they build on are `ia_interaction` (REQ-UX-001..019) and the
state model (REQ-UX-020..029), referenced not restated.

###### REQ-UX-030 — The logged-out landing says what chirp is and offers an obvious join route, even when empty
- **Statement:** On arrival a logged-out visitor can tell what chirp is and always sees an obvious way to create an account or sign in, whether or not there is any content yet.
- **Acceptance criteria:** Opening chirp while logged out shows a one-line statement of what chirp is together with a visible **Create account** and **Sign in**; when the public timeline has no chirps, the screen shows a clear "what chirp is + be the first" message with a working **Create account** action rather than a blank page; neither the statement nor the join controls require scrolling past a login wall.
- **Release gating:** true
- **Traces to:** REQ-PROD-013, REQ-PROD-014, REQ-PROD-015; J1.

###### REQ-UX-031 — Reading the landing timeline (and profiles reached from it) never requires an account
- **Statement:** A logged-out visitor can read the public timeline and open an author's profile from it without being forced to sign in.
- **Acceptance criteria:** From the landing, a logged-out visitor can read the feed and open a profile linked from a chirp and read its chirps, with no sign-in prompt interrupting reading; a sign-in prompt appears only when the visitor attempts an action that needs an account (post, like, follow), not for reading.
- **Release gating:** true
- **Traces to:** REQ-PROD-013; J1.

###### REQ-UX-032 — The landing timeline handles its loading, empty, error, and end-of-list states honestly
- **Statement:** The logged-out landing timeline presents loading, empty, error, and end-of-list as honest, recoverable states, not a blank page, a spinner with no exit, a raw error, or an ambiguous stop.
- **Acceptance criteria:** A visitor sees shape-matched placeholders on first load (chrome and join route still usable); an explained empty state with a next action when there are no chirps; on a failed load, a plain message with a **Try again** that refetches without reloading the whole page; and a clear "you're all caught up" marker at the end of the list, visibly different from the empty state.
- **Release gating:** true
- **Traces to:** REQ-PROD-014, REQ-PROD-018; J1. Instantiates the state model (REQ-UX-021..025).

###### REQ-UX-033 — A visitor can create an account entirely in the product, reached from landing and sign-in
- **Statement:** A logged-out visitor can reach a create-account screen from the first screen and from the sign-in screen and create an account using only the product UI.
- **Acceptance criteria:** From both the landing and the sign-in screen, a visible control reaches a sign-up screen; supplying the required details (a handle, a credential, at minimum) and submitting creates a new account and signs the person in — without using the raw API; a link back to **Sign in** is present for people who already have an account.
- **Release gating:** true
- **Traces to:** REQ-PROD-001, REQ-PROD-015; J1, J2.

###### REQ-UX-034 — Sign-up validates the handle inline and surfaces uniqueness conflicts clearly
- **Statement:** At sign-up, handle rules and handle-uniqueness conflicts are explained inline, in language the user can act on, at or before submit, without losing other entered details.
- **Acceptance criteria:** Entering an over-length (>15) or disallowed-character handle produces a clear inline message at the field (and the counter reflects the 15-character limit) before the account is created; submitting a handle that is already taken yields a clear "that handle is taken" message at the handle field and creates no duplicate account; choosing a free handle then succeeds with the display name/bio the user already typed still present.
- **Release gating:** true
- **Traces to:** REQ-PROD-002, REQ-PROD-003; J2. Instantiates the state model validation pattern (REQ-UX-027).

###### REQ-UX-035 — Sign-up gives honest busy/error/success feedback and never loses entered details
- **Statement:** Submitting the sign-up form shows it is in flight, prevents a duplicate submission, and on failure returns an editable form with the entered details intact; success is shown only when the account is really created.
- **Acceptance criteria:** Pressing Create account shows a busy control that cannot be double-fired and does not jump the layout; a non-validation failure shows an honest, recoverable message with the entered handle/display name/bio preserved for retry; on success the person ends up signed in on a purposeful signed-in destination (not the logged-out landing), and no failed attempt ever appears to have created an account.
- **Release gating:** true
- **Traces to:** REQ-PROD-001, REQ-PROD-017; J2. Builds on the logged-out→signed-in transition (REQ-UX-011) and submission pattern (REQ-UX-028).

###### REQ-UX-036 — Sign-in is a credentialed screen of its own, interlinked with sign-up and recovery
- **Statement:** A returning user has a dedicated sign-in screen that asks for a credential (not a handle alone) and links to both account creation and recovery.
- **Acceptance criteria:** The sign-in screen presents a handle field and a credential field and a Sign in action; it shows a visible **Create account** link and a visible **"Can't sign in?"** recovery link; signing in with a correct handle+credential signs the user in and swaps the UI to the signed-in state showing who they are. (Knowing only a handle, with no credential, does not sign a person in — the enforcing mechanism is identity & trust.)
- **Release gating:** true
- **Traces to:** REQ-PROD-004, REQ-PROD-015; J3. Credential verification mechanism → identity & trust (REQ-ID-*).

###### REQ-UX-037 — Sign-in feedback is honest, non-enumerating, recoverable, and never a false success
- **Statement:** A failed sign-in shows an honest message that does not reveal whether the handle exists, preserves the entered handle, and never appears to succeed; an expired session routes to this screen rather than failing silently.
- **Acceptance criteria:** Signing in with a wrong credential (or an unknown handle) shows the same clear "handle and credential don't match" message — not a raw error and not a different message that reveals whether the handle exists — with the handle preserved and a recovery route offered; a failed sign-in never shows the signed-in state; and when a session has expired, the next attempted action lands on this screen with a clear "session expired, sign in again" prompt and returns the user to what they were doing on success.
- **Release gating:** true
- **Traces to:** REQ-PROD-004, REQ-PROD-006; J3. Session/credential mechanism → identity & trust (REQ-ID-*); instantiates session_expired (REQ-UX-026).

###### REQ-UX-038 — A recovery flow is reachable from sign-in and lets a locked-out user regain the same account
- **Statement:** A user who cannot sign in can start recovery from the sign-in screen and, on completing it, regain access to the same account with a new credential.
- **Acceptance criteria:** The "Can't sign in?" link on sign-in opens a recovery screen; supplying the recovery factor starts the flow, and following the recovery steps lets the user set a new credential and then sign in to the same account (same handle, chirps, and follows); a user who does not control the recovery factor cannot gain access this way.
- **Release gating:** true
- **Traces to:** REQ-PROD-007; J4. Recovery factor/carrier/mechanism → identity & trust (REQ-ID-*).

###### REQ-UX-039 — Recovery states are honest and non-enumerating, with recoverable errors
- **Statement:** Recovery shows inline validation, a busy state, honest success that does not disclose whether an account exists, and recoverable errors (including an expired recovery link) with a way back.
- **Acceptance criteria:** Requesting recovery validates the factor inline and shows a busy state that cannot be double-fired; the request's confirmation does not reveal whether an account exists for that factor ("if that matches an account, we've sent the next steps"); setting a new credential validates it and its confirmation inline; an expired or invalid recovery link shows a clear message with a route back to start again rather than a crash; and no step shows a false success.
- **Release gating:** true
- **Traces to:** REQ-PROD-007; J4. Anti-enumeration guarantee & token lifetime → identity & trust (REQ-ID-*); instantiates the state model (REQ-UX-027..029).

###### 6.1 Traceability summary (REQ-UX → REQ-PROD / journeys)

| REQ-UX | Screen | Gating | Traces to |
|---|---|---|---|
| REQ-UX-030 | SCR-landing_timeline | true | REQ-PROD-013, 014, 015; J1 |
| REQ-UX-031 | SCR-landing_timeline | true | REQ-PROD-013; J1 |
| REQ-UX-032 | SCR-landing_timeline | true | REQ-PROD-014, 018; J1 |
| REQ-UX-033 | SCR-signup | true | REQ-PROD-001, 015; J1, J2 |
| REQ-UX-034 | SCR-signup | true | REQ-PROD-002, 003; J2 |
| REQ-UX-035 | SCR-signup | true | REQ-PROD-001, 017; J2 |
| REQ-UX-036 | SCR-signin | true | REQ-PROD-004, 015; J3 |
| REQ-UX-037 | SCR-signin | true | REQ-PROD-004, 006; J3 |
| REQ-UX-038 | SCR-recovery | true | REQ-PROD-007; J4 |
| REQ-UX-039 | SCR-recovery | true | REQ-PROD-007; J4 |

All ten ids are unique, fall within the owned `REQ-UX-030..039` sub-range, and lie within the parent
`REQ-UX-020..069` range. Every entry/auth
screen is `gating: true` in the inventory, so the register is gating true throughout; honest cuts within
this flow (richer multi-factor recovery, social/remember-me sign-in) are deferred to `REQ-ID-*` and
later work rather than minted as non-gating UX here.

---

##### 7. Coverage, composition & handoff notes

- **Screen coverage.** All four owned screens are designed:
  SCR-landing_timeline (§2), SCR-signup (§3), SCR-signin (§4), SCR-recovery (§5). Each is designed with
  every state in its the per-screen state matrix row plus the implied `content`:
  - SCR-landing_timeline: `loading, empty, error, end_of_list` — all designed (§2).
  - SCR-signup: `validation, busy_submitting, error, success` — all designed (§3).
  - SCR-signin: `validation, busy_submitting, error, success` — all designed (§4).
  - SCR-recovery: `validation, busy_submitting, error, success` — all designed (§5).
  No state was dropped; none is recorded out-of-scope.
- **States designed, not skeletal.** Each state carries
  concrete copy intent and a recovery affordance, instantiated from the state model
  (`empty_error_loading_states` §1.1–§1.9) with its shared voice (§0.4) — not merely named.
- **Components consistent.** Every screen is composed from named
  visual-system components (nav §3.9, button §3.1, link §3.2, input/textarea §3.3, character counter
  §3.4, identity cluster §3.5, chirp card §3.6, inline alert §3.13, state blocks §3.14, modal §3.11,
  tokens §2) and placed per the IA screen map and logged-out nav (ia-interaction §1.1/§1.4/§2). **No
  component was invented and no gap was found to flag** — the component set and the state model cover
  every entry/auth screen and state.
- **Req-UX register.** REQ-UX-030..039, all
  within the owned sub-range, each user-side, stranger-testable, release-gating-flagged, and traced to
  ≥1 REQ-PROD / journey (§6).
- **Seam respected.** No screen inventory entry, state-vocabulary entry, REQ-UX id scheme, or state
  meaning was redefined; the credential/session/recovery **mechanisms** are consistently deferred to
  `REQ-ID-*` and the destinations-after-auth to `core_loop`/IA, with each hand-off named (not designed
  twice).
- **What, not how (identity respected).** Design intent at wireframe level and committed-text assets
  only; no front-end code. The current SPA at `c805525` is treated as the starting point to improve
  (handle-only login → credentialed sign-in; no sign-up → in-product sign-up; raw timeline → designed
  landing; nothing → recovery), not a target to preserve.

---

### Core loop — signed-in screens, flows & states (`screens_and_flows/core_loop`)

The **daily-use heart of chirp**: the six signed-in screens a person lives in once they are in the
product — the feed they read, the composer they post from, the profiles they visit and manage, the
thread where a chirp becomes a conversation, the first run that gives a new account somewhere to start,
and the search that lets them find people and content beyond the links already on screen. This document
is the assembled core-loop section of the chirp experience design: a single coherent picture of what each
screen is, how a person moves through it, and **every state it must handle honestly** — at wireframe
level, with concrete copy intent and a recovery affordance for each state.

It is built to one bar: **designed screens, not a skeleton.** A product is judged in its first minute of
real use, and that judgement falls hardest on the states nobody demos — the empty feed, the failed post,
the expired session, the profile that does not exist, the search that found nothing. Each of those is a
first-class, designed state here, not an afterthought.

#### How this document is organised

1. **The unified `REQ-UX-040..051` register** (below) — the twelve core-loop requirements, each stated
   from the user's side, testable by a stranger against the live product, flagged release-gating or not,
   and traced to the product requirements and journeys it serves. This is the release bar for the core
   loop.
2. **The six screen fragments** (§ Home timeline … § Search / discovery) — each owned and authored by its
   screen design and assembled here verbatim. Each fragment carries its own layout intent, happy-path flow,
   designed states, requirements, and notes.

#### What is deliberately *not* here (scope boundary)

- **How it is built.** This is *what, not how*: behaviour, appearance, and the testable bar. Front-end
  code, routing, persistence, and the session/auth lifetime are deferred to the code work and to the
  identity / domain / ops requirement areas (`REQ-ID-*`, `REQ-DOM-*`, `REQ-OPS-*`).
- **Screens owned by sibling surface groups** — entry/auth (landing, sign-up, sign-in, recovery),
  account/settings (settings, edit-profile, deletion, export, sign-out), and trust/safety/admin
  (report/block, privacy/terms, admin/support) — and the cross-cutting experience layers (the visual
  system, the IA & navigation model, the shared state-design model, responsive rules, accessibility).
  Those are **referenced, never redefined** here; where a core-loop screen needs something they own, it
  composes it and, if it is missing, **flags it** (see the propagated findings at the foot of this
  document) rather than inventing it.

---

#### A. Unified requirement register — `REQ-UX-040 .. REQ-UX-051`

Twelve requirements across the six screens, partitioned into disjoint per-screen
sub-ranges. Every id is **unique**, lies **inside its owning screen's sub-range** and within the core-loop
range `REQ-UX-040..051` (itself inside the experience range `REQ-UX-020..069`), **collides with no other**,
and leaves **no gap**. Each is user-side, carries acceptance criteria a stranger could check, a
`release_gating` flag, and a trace to ≥1 `REQ-PROD-*` and/or journey. Full statements and acceptance
criteria live in each screen's fragment (§ cross-reference in the last column).

| REQ-UX | Screen | Gating | Traces to | Realises (seams) | Full text |
|---|---|---|---|---|---|
| REQ-UX-040 | SCR-home_timeline | **true** | REQ-PROD-018; (REQ-PROD-025 non-gating); J5, J6 | honest states + paging | § Home timeline 5 |
| REQ-UX-041 | SCR-home_timeline | **true** | REQ-PROD-016, 020, 021; J5 | OPTIMISTIC_RECONCILE, COMPOSE_ENTRY_SEAM, SHARED_CHIRP_CARD | § Home timeline 5 |
| REQ-UX-042 | SCR-home_timeline | **true** | REQ-PROD-006; J6 | session-expired via shared prompt | § Home timeline 5 |
| REQ-UX-043 | SCR-compose | **true** | REQ-PROD-016; J5 | COMPOSE_ENTRY_SEAM, visible ≤280 counter | § Compose 7 |
| REQ-UX-044 | SCR-compose | **true** | REQ-PROD-017, 006; J5, J6 | COMPOSE_DRAFT_SURVIVES, honest session-expiry | § Compose 7 |
| REQ-UX-045 | SCR-profile | **true** | REQ-PROD-019, 008, 013, 012; J5, J6 | own-vs-other, empty ≠ not_found | § Profile 5 |
| REQ-UX-046 | SCR-profile | **true** | REQ-PROD-020; J5, J6 | OPTIMISTIC_RECONCILE (follow) | § Profile 5 |
| REQ-UX-047 | SCR-chirp_thread | false¹ | REQ-PROD-023, 018, 019; J5 | CONDITIONAL_REPLY_COHERENT, SHARED_CHIRP_CARD | § Chirp thread 5 |
| REQ-UX-048 | SCR-chirp_thread | false¹ | REQ-PROD-023, 016, 017; J5 | CONDITIONAL_REPLY_COHERENT, COMPOSE_ENTRY_SEAM | § Chirp thread 5 |
| REQ-UX-049 | SCR-onboarding | false | REQ-PROD-022; J2, J6 | first-run next-action + observable payoff | § Onboarding 6 |
| REQ-UX-050 | SCR-search_discovery | false | REQ-PROD-024; J1, J5, J6 | dedicated find-people-and-content surface | § Search / discovery 7 |
| REQ-UX-051 | SCR-search_discovery | false | REQ-PROD-024; J1, J5, J6 | honest search states (running ≠ no-results) | § Search / discovery 7 |

**Gating shape.** The gating core loop is **read the feed, compose and post, visit and manage profiles**
(`REQ-UX-040..046`, all release-gating). The thread/reply experience (`REQ-UX-047..048`), the new-user
first run (`REQ-UX-049`), and search/discovery (`REQ-UX-050..051`) are **non-gating, recorded cuts** — a
first releasable version may ship without them — but each is **designed in full** regardless, because a
non-gating surface is still a designed screen.

¹ **The one non-negotiable inside the non-gating thread.** Replies are a conditional product requirement
(REQ-PROD-023): the product may ship *without* a thread/reply experience, but it must **never ship a
reply or thread-open affordance that leads nowhere**. That "no dead affordance" proviso is itself
**release-gating** and is carried by `REQ-UX-013` (owned by the IA & interaction section, outside this
range). The core-loop contract freezes the reply stance to the **reply-built** branch: wherever a reply
affordance appears (on a feed card, a profile chirp, a search result), it opens the coherent thread
designed in § Chirp thread — it is never a stub.

#### B. The core-loop cross-screen seams

The six screens are parts of one design and share five seams, fixed as shared decisions before the individual screens were designed:

- **SHARED_CHIRP_CARD** — every surface that shows a chirp (feed, profile list, thread root + replies,
  search results) renders the **one** shared chirp card, never a per-screen variant.
- **COMPOSE_ENTRY_SEAM** — there is **one** composer (SCR-compose); the feed's compose affordance, the
  onboarding first-post offer, the profile's own-empty "first chirp", and the thread's reply composer all
  route to or instantiate that one composer and never redefine it.
- **OPTIMISTIC_RECONCILE** — **like and follow are the only optimistic actions**; they update immediately
  and reconcile to the true state on failure. Posting a chirp and posting a reply are **not** optimistic
  (busy → success/error); deleting is confirmed, never optimistic.
- **COMPOSE_DRAFT_SURVIVES** — an unsent chirp draft survives a failed send, a re-render, and a session
  expiry; it is never silently discarded.
- **CONDITIONAL_REPLY_COHERENT** — the reply experience is designed coherently and in full (reply-built
  branch), non-gating, with no dead affordance anywhere.

---

### The six screen fragments

The fragments below are presented in core-loop reading order
(feed → compose → profile → thread → onboarding → search). Each retains its own section numbering and notes.

---

#### Home timeline (signed-in feed) — SCR-home_timeline

*(section: `screens_and_flows/core_loop/home_timeline`)*

The home timeline is **the daily-use front door of chirp** — the first thing persona P3 (the returning
reader/poster) and a freshly-onboarded P2 see every visit, and the surface the whole product is judged by
in its first minute of real use (J5, J6). It is the signed-in, reverse-chronological feed of the chirps
posted by the accounts the user follows, newest first, paged a screenful at a time. Everything a user does
in the core loop either starts here (compose, open a profile, like, follow, open a chirp) or returns here
afterwards.

This fragment designs **what the screen is, how a user moves through it, and every state it must handle
honestly** — at wireframe level, with concrete copy intent and a recovery affordance for each state. It
does **not** redesign the composer (that is the one `SCR-compose` surface this screen *routes to*,
identity `COMPOSE_ENTRY_SEAM`), the chirp card (the one shared component from the visual system, identity
`SHARED_CHIRP_CARD`), the navigation chrome (owned by the IA model and the visual system), or the state
model (owned by the state model and *instantiated* here). A component or pattern this screen needs
but cannot find is flagged for resolution, never invented. Where a signed-in action meets an expired
session, this screen routes through the one re-sign-in prompt (state model §1.6) rather than a bespoke
message. **What, not how:** no routing/framework/persistence mechanism is settled here.

##### What this improves over the product today (grounded at `c805525`)

The current timeline view already does several things worth keeping, and this design keeps them: a
public-readable reverse-chronological list, honest `loading`/`content`/`empty`/`error` states, cursor-based
**Load more**, and an explicit **"You're all caught up."** end-of-list note. The gaps this design closes:

- **Identity is a raw number.** Today a chirp shows its `author_id` as a bare integer. This design renders
  every chirp through the shared chirp card with the identity cluster (avatar + display name + `@handle`),
  so a reader can tell who posted what.
- **Timestamps are raw milliseconds.** Today a chirp shows `"{ms} ms"`. This design shows a relative
  timestamp ("2m", "4h", "Mar 3"), with the absolute time on hover/title.
- **Likes are display-only; there is no like control, no follow, and no compose affordance on the feed.**
  Today the timeline only *shows* a like count and routing to compose/login is a separate screen. This
  design adds an actionable, optimistic like control and a compose entry affordance **on** the feed, and
  (where a feed item or an inline suggestion exposes an account) an optimistic follow control — all
  reconciling to the true state on failure (`OPTIMISTIC_RECONCILE`).
- **No signed-in identity indicator and no session-expiry handling.** Today the nav is just
  `[Timeline · Compose]` and an expired in-memory session fails silently. This design assumes the IA
  signed-in nav (identity indicator + account menu, IA §1.2) and routes any timeline action attempted on an
  expired session through the single re-sign-in prompt (state model §1.6), never a silent fail.
- **Empty means one thing today ("No chirps yet.").** This design distinguishes the *first-run / quiet*
  empty (you follow no one yet) from a genuine *load error*, because they need different copy and different
  recovery.

---

##### 1. Layout & content intent (wireframe level)

The home timeline sits in the single readable content column (`layout.content.max` 600px, `layout.gutter`;
visual-system §2.6) beneath the **persistent signed-in nav** (IA §1.2: brand, Home, Compose, Search-where-
present, and the identity indicator + account menu). This screen does **not** draw the nav; it assumes it
and highlights **Home** as the active destination (nav active state, visual-system §3.9).

```
┌───────────────────────────────────────────────────────────────┐
│  [chirp]   Home*   Compose   Search      ( @handle ▾ )          │  ← persistent signed-in nav (IA §1.2;
├───────────────────────────────────────────────────────────────┤     NOT drawn by this screen; Home active)
│                                                                 │
│  ┌───────────────────────────────────────────────────────┐    │  ← COMPOSE ENTRY AFFORDANCE
│  │ (○)  What's happening?                        [ Chirp ] │    │    (identity COMPOSE_ENTRY_SEAM): an inline
│  └───────────────────────────────────────────────────────┘    │    prompt that opens the ONE composer
│                                                                 │    (SCR-compose). Not the composer itself.
│  ── New chirps since you were here (3) ──  [ Show ]             │  ← optional "what's new" affordance
│                                                                 │    (non-gating, REQ-PROD-025)
│  ┌───────────────────────────────────────────────────────┐    │
│  │ (○) Ada Lovelace  @ada · 2m                        ⋯    │    │  ← CHIRP CARD, default variant
│  │     Just shipped the analytical engine notes.          │    │    (identity SHARED_CHIRP_CARD;
│  │     ♡ 12                                               │    │     visual-system §3.6). Avatar +
│  └───────────────────────────────────────────────────────┘    │     identity cluster + relative time +
│  ┌───────────────────────────────────────────────────────┐    │     overflow (report/block, or delete on
│  │ (○) Alan T.  @alan · 4h                            ⋯    │    │     own chirp) + body (line breaks kept)
│  │     On computable numbers, again.                      │    │     + action row (like control w/ count;
│  │     ♥ 41                                               │    │     reply affordance only if SCR-chirp_
│  └───────────────────────────────────────────────────────┘    │     thread ships — see §4).
│                            ·                                    │
│                            ·                                    │
│  ┌───────────────────────────────────────────────────────┐    │
│  │                  [ Load more ]                          │    │  ← footer: Load more / Loading more… /
│  └───────────────────────────────────────────────────────┘    │     inline retry / end-of-list note (§3)
└───────────────────────────────────────────────────────────────┘
```

- **Compose entry affordance (top of feed).** A persistent, low-friction prompt ("What's happening?") with
  the user's avatar and a **primary `Chirp` button** (visual-system §3.1). Activating it (the field or the
  button) opens the **one** composer surface, `SCR-compose`, as the inline/modal composer the IA permits
  (IA §2.2), *without discarding the timeline context underneath* (REQ-UX-012). This screen owns only the
  **entry point**; the composer's layout, ≤280 validation, counter, busy/success/error states, and draft
  preservation belong to `the compose design` (`COMPOSE_ENTRY_SEAM`). The entry affordance is also the on-feed
  realization of the nav `Compose` action. (The same affordance is the first-post offer `SCR-onboarding`
  routes to — onboarding references it; it is not redefined there.)
- **Feed list.** A vertical, divider-separated stack of **chirp cards** (visual-system §3.6, `default`
  variant), newest first, each card rendered identically to how a chirp appears on a profile, in a thread,
  and in search (`SHARED_CHIRP_CARD`): avatar (`sm`/`md`) + identity cluster (display name `font.label` +
  `@handle` `font.meta`, §3.5) + **relative timestamp** (absolute on hover/title) + overflow trigger
  (report/block via `SCR-report_block`; **Delete** on the user's own chirp, confirmed per IA §3.4) + body
  (preserves the author's line breaks) + an action row carrying the **like control** (and a reply
  affordance only in the "reply built" world — §4). The feed never hand-rolls a chirp row.
- **Like control (on every card).** The shared like control (visual-system §3.8): outline/not-liked ↔
  filled/liked with a truthful count, the liked state carried by the filled icon **and** weight, not colour
  alone (REQ-UX-073). Optimistic (§2, `OPTIMISTIC_RECONCILE`).
- **Follow control (contextual).** Where the feed exposes an account the user does not follow — a
  "who to follow" suggestion row on a quiet feed (§3.2), or the account on a card the design chooses to
  expose follow on — the shared **follow control** (visual-system §3.7, `Follow`/`Following`/busy) appears,
  optimistic and reconciled (§2). The *primary* home of follow/unfollow is `SCR-profile`; the home timeline
  only mirrors the same optimistic pattern where it surfaces an account.
- **Footer.** A single slot beneath the list that resolves to exactly one of: **Load more** button,
  **"Loading more…"** indicator, an **inline error + Try again** (continuation failure), or the
  **end-of-list** marker (§3).
- **"What's new" affordance (non-gating).** On a return visit, an unobtrusive "New chirps since you were
  here" marker lets the user pull in newer chirps without being yanked to the top or losing their scroll
  position (REQ-PROD-025). This is a **recorded non-gating enhancement** (see REQ-UX-040's non-gating note);
  the gating core is the honest-states feed with paging.

**Composition note.** Every colour, type, space, and control above is a visual-system token/component; this
screen introduces no new component. If a needed presentation were missing it would be flagged for resolution — none was: the chirp card, nav, buttons, follow/like controls, state blocks, toast, and
inline alert cover this screen entirely.

---

##### 2. Happy-path flow (the loaded core loop)

1. **Arrive.** The user opens chirp signed in (direct, via the nav `Home`, or as the landing destination
   after sign-in — IA §2.4) and lands on the home timeline. The screen requests the first page of the
   followed-accounts feed and shows `loading` (§3.1) in place beneath the already-rendered nav and compose
   affordance — the chrome is never blanked.
2. **Read.** The first page resolves to `content`: a reverse-chronological list of chirp cards, newest
   first. The user scrolls and reads. Reading requires no further action and never a prompt (reading is
   free — REQ-PROD-013; here the reader is already signed in).
3. **Page.** At the foot of the list the user activates **Load more**; the footer shows "Loading more…"
   while the next page is fetched by continuation cursor, then the older chirps are appended beneath the
   current ones **without moving the user's place** and without re-fetching the whole feed. This repeats
   until the last page, which shows the **end-of-list** marker (§3.4) instead of a Load more button.
4. **Like.** The user taps the like control on a card. The icon fills and the count increments
   **immediately** (optimistic); the control shows its busy affordance; on success the state simply stands;
   on failure it **reverts** to the true icon and count with an honest error (§3.6, `OPTIMISTIC_RECONCILE`).
5. **Compose.** The user activates the compose entry affordance; the one composer (`SCR-compose`) opens over
   the feed without losing scroll/context; on a successful post the new chirp becomes observable at the top
   of the feed and a brief **"Chirp posted."** success is shown (§3.7). (The composer owns its own
   busy/validation/error/draft behaviour; this screen owns the *entry* and the *result appearing*.)
6. **Branch out.** From a card the user may open the author's **profile** (`SCR-profile`, via the identity
   cluster), open the **chirp** itself (`SCR-chirp_thread`, only if replies are built — §4), **follow** a
   surfaced account (optimistic — §3.6), or **report/block** or **delete own** via the overflow (routed to
   `SCR-report_block` / confirmed deletion). Each returns the user to the feed.
7. **Return later (J6).** On a subsequent visit the user picks up where they were; the "what's new"
   affordance (§1, non-gating) lets them pull newer chirps in without losing their position.

---

##### 3. The states — designed, not skeletal

The required-state matrix for SCR-home_timeline is
`[loading, empty, error, end_of_list, session_expired, busy_submitting, success]`
(the per-screen state matrix), plus the `content` baseline (always implied). Each
state below **instantiates the shared state model** (the state model §1.x) with this screen's concrete
copy intent and recovery affordance, rendered by the named visual-system block, obeying the named IA
pattern. Exemplar copy is illustrative, in the product's calm, plain, non-blaming voice (state model §0.4),
repository-ready; the exact strings are a build-time concern.

###### 3.0 `content` — the loaded feed (baseline)
- **When:** the first page resolved and there is at least one chirp from a followed account.
- **Design:** the divider-separated stack of shared chirp cards (§1), newest first, under the compose entry
  affordance; the footer offers paging (§3.4). This is the reference every other state departs from; the
  transition from `loading` into `content` must **not jump the layout** (shape-matched skeletons, §3.1).
- **Recovery:** n/a (positive baseline).

###### 3.1 `loading` — first fetch
- **When:** the first page of the feed is being fetched (fresh arrival or a retry of the first page).
- **Copy intent & voice:** wordless — **shape-matched chirp skeletons** on `bg.subtle`, not a blank page or
  a full-screen spinner, so the arriving content does not shift the layout. The nav and compose affordance
  stay interactive throughout (the chrome is never blocked). A rare long wait may show a quiet "Still
  loading…", never an apology or a fake progress bar.
- **Recovery affordance:** a load is never a trap — if the first fetch stalls beyond a reasonable wait, the
  surface offers **Try again** rather than an indefinite spinner (transitions to the `error` recovery in
  place).
- **Composition:** Loading/skeleton state block (visual-system §3.14); IA state-surfacing §3.5.

###### 3.2 `empty` — the fetch succeeded but there is nothing to show
The design point is that **"empty" here has two distinct causes that must read differently**, and neither
is a bare blank area (state model §1.2):
- **First-run / quiet feed (you follow no one, or too few to fill a screen).** The common new-user and
  low-follow case. Point at discovery, not a dead end.
  - *Exemplar:* "Your timeline is quiet. Follow a few people and their chirps show up here." +
    primary **Find people** (routes to `SCR-search_discovery` where present) and secondary **Compose your
    first chirp** (opens the composer). Where suggestions exist, a short **who-to-follow** row with
    optimistic follow controls (§3.6) is offered inline.
  - *Note:* immediately after sign-up the user is routed to `SCR-onboarding` first (IA §2.5), so this quiet
    state is primarily the returning low-follow case; it must still stand on its own for anyone who reaches
    the feed empty.
- **Followed accounts exist but have posted nothing yet.** Rare but real.
  - *Exemplar:* "No chirps yet from the people you follow. Check back soon, or find more people to follow."
    + **Find people**.
- **Recovery affordance:** the surface's primary next action — **Find people** / **Compose** — never a
  blank screen the user must leave. (Distinct from the logged-out landing empty, which is the entry/auth
  section's; this is the *signed-in* feed.)
- **Composition:** Empty state block (visual-system §3.14); IA first-class-empties §3.5. (Clearly different
  from `error` — the fetch *succeeded*.)

###### 3.3 `error` — the feed failed to load
- **When:** the first-page fetch failed (transient network/server failure).
- **Copy intent & voice:** honest, specific, non-blaming — name what failed and that it is recoverable;
  never a raw error code, a blank/broken screen, or a false "empty."
  - *Exemplar:* "Something went wrong loading your timeline." + **Try again**.
- **Recovery affordance:** **Retry in place, without a full page reload** (REQ-PROD-018) — the retry
  re-fetches the first page beneath the preserved chrome. (A *continuation* failure while paging is the
  inline footer error of §3.4, which keeps the already-loaded chirps on screen.)
- **Composition:** Error state block + Retry (visual-system §3.14) for the whole-feed failure; IA honest-
  feedback §3.1/§3.5. This is the same `error` pattern the user meets on a profile and a submit — one
  recognisable shape product-wide (REQ-UX-020/083).

###### 3.4 `end_of_list` — no more pages
- **When:** paging has returned the last page; there are no older chirps.
- **Copy intent & voice:** make the end **explicit and positive**, clearly different from `empty` (empty =
  nothing at all; end-of-list = you've seen it all). The current product's **"You're all caught up."** is
  exactly right and is kept.
  - *Exemplar:* "You're all caught up." (optionally "— nothing older to show.")
- **Recovery affordance:** a sensible onward action — **Back to top** / **Refresh for new** — rather than an
  ambiguous dead stop or an endless spinner probing for a non-existent next page.
- **Composition:** End-of-list state block (visual-system §3.14), a subtle terminal marker at the list foot,
  distinct from the Empty block.

###### 3.5 `session_expired` — a timeline action after the session ended
- **When:** the user attempts an **authenticated action from the feed** (like, follow, compose-open+post,
  delete own, report) **after the session has expired or been invalidated** — including after a service
  restart, which invalidates in-memory sessions today. (Reading the feed itself does not require a live
  session; this fires on the *action*.)
- **Copy intent & voice:** a **clear re-sign-in prompt** that says the session ended and the action needs a
  fresh sign-in — **never a silent fail and never a false success** (REQ-PROD-006).
  - *Exemplar:* "Your session has expired. Sign in again to continue." + **Sign in**.
- **Recovery affordance:** route to sign-in **preserving the user's context and intent** (return-to-intent,
  IA §2.4/§3.3) so that after signing in the user is returned to the feed and the attempted action can be
  completed; **do not discard in-progress input** (e.g. a drafted chirp handed to the composer) without
  warning. An optimistic like/follow attempted on an expired session **reverts** to its true state rather
  than appearing to succeed.
- **Composition:** the single Session-expired prompt block (visual-system §3.14) routed through the auth-
  gating pattern (IA §3.3) and reflected by the identity indicator flipping to logged-out (IA §3.6). This
  screen uses the one shared prompt (state model §1.6), not a per-screen message. The session lifetime and
  invalidation mechanism are `REQ-ID-*` — only the prompt and the resume are designed here.

###### 3.6 `busy_submitting` — an action from the feed is in flight (incl. optimistic reconcile)
- **When:** a submission started from the feed is in flight — an optimistic **like/unlike** or
  **follow/unfollow**, or a **delete own chirp** confirmation.
- **Copy intent & voice:** minimal — the control itself shows the work and **prevents a duplicate**; the
  point is to show it is working and hold the layout, not to narrate.
- **Optimism & reconcile (identity `OPTIMISTIC_RECONCILE`):** like and follow are the **only** optimistic
  actions on this screen — the icon/label and count update **immediately** on tap, ahead of confirmation,
  to keep the core loop fast; on **failure** they **visibly revert** to the true server state and count and
  surface an honest error, so the on-screen state always matches reality after the action settles
  (IA §3.2; visual-system §3.7/§3.8). Posting a chirp is **not** optimistic (the composer shows busy →
  success/error); **delete own chirp** is destructive and is **never** optimistic — it is confirmed first
  (IA §3.4) and shows a plain busy state, never a pre-emptive removal.
- **Recovery affordance:** on failure, return to the editable/true state with an honest error (§3.3 voice,
  inline/toast as appropriate); the control never strands the user in a permanent busy state and the layout
  does not jump (busy control preserves its size, visual-system §3.0).
- **Composition:** §3.0 busy control; follow/like busy→reverted visuals (visual-system §3.7/§3.8); IA
  no-silent-completion §3.1.

###### 3.7 `success` — an action from the feed completed
- **When:** an action completed — a chirp posted (via the composer opened from the entry affordance), a like
  or follow settled, a chirp deleted.
- **Copy intent & voice:** an **honest confirmation**, brief and specific, with the **result observable**:
  a posted chirp **appears at the top of the feed** (the confirmation of record); a settled like/follow
  shows its true filled/Following state and count; a deleted chirp is removed from the feed.
  - *Exemplar:* a brief toast "Chirp posted." / "Followed @ada." Quick actions (like) confirm by the result
    itself appearing — no toast needed for every tap.
- **Recovery affordance:** n/a as a positive terminal — but an **optimistic** success that later fails
  reverts and surfaces the error (never a false success; IA §3.1/§3.2).
- **Composition:** toast for a brief success (visual-system §3.12), or the reconciled control itself
  (§3.7/§3.8); the new chirp rendered as a shared chirp card at the feed head.

###### 3.8 `not_found` — out of scope for this screen (recorded)
`not_found` is **not** in SCR-home_timeline's state matrix and is correctly out of scope here: the feed is a
list, not an addressable single subject. A single missing/removed chirp that would otherwise appear in the
list is shown via the chirp-card **`tombstone`** variant ("This chirp is unavailable", no actions;
visual-system §3.6, state model §1.4) rather than a whole-screen not-found; the whole-screen `not_found` is
`SCR-profile`/`SCR-chirp_thread`'s concern. Recorded so the deliberate scoping is visible, not
a dropped state.

---

##### 4. The reply affordance on a feed card (deferred to the frozen conditional-reply stance)

A chirp card carries a reply affordance **only in the "reply built" world** (identity
`CONDITIONAL_REPLY_COHERENT`; REQ-PROD-023 / REQ-UX-013). `the chirp-thread design` owns the frozen stance that
the reply experience **is** built and coherent, so this screen's card **does** expose a "reply/open" entry
that opens `SCR-chirp_thread`, and never a dead control. If the thread experience were ever cut, this screen
would show **no** reply affordance at all rather than a control that leads nowhere. This screen does not
design the thread or the reply composer — it only exposes the coherent entry per the frozen stance and
defers the thread design to `the chirp-thread design`.

---

##### 5. Requirements & acceptance criteria — REQ-UX-040 .. REQ-UX-042

Range owned by `the home-timeline design`
(`REQ-UX-040 .. REQ-UX-042`). Each requirement is stated from the user's side, is testable by a stranger
exercising the live product, carries a `release_gating` flag, and traces to ≥1 `REQ-PROD-*` and/or journey.
Composes against `req_ux_schema`; the parent id scheme, screen inventory, and state vocabulary are used as
frozen and not redefined.

###### REQ-UX-040 — The home timeline is a reverse-chronological feed with honest states and paging
- **Statement:** A signed-in user sees a reverse-chronological feed of the chirps from the accounts they
  follow, with honest loading, content, empty, error, and end-of-list states, and can page through older
  chirps and recover from a load error without reloading the whole page.
- **Acceptance criteria:** Signed in, a person on the home timeline observes: a shape-matched loading state
  on first fetch (not a blank page or a full-screen spinner); the chirps newest-first when there are any; a
  clear "your timeline is quiet / follow people" message (not a blank area) when the feed is empty, visibly
  different from a load error; an honest, recoverable error with a retry that re-fetches **without a full
  page reload** when the fetch fails; a **Load more** that appends older chirps without losing the user's
  place; and a clear **"you're all caught up"** end marker when no older chirps remain. Each state is
  recognisably the same pattern used elsewhere in the product.
- **Release gating:** true
- **Traces to:** REQ-PROD-018; J5, J6. *(Non-gating enhancement, recorded not separately minted: a
  "what's new since last visit" affordance that pulls newer chirps without losing scroll position —
  REQ-PROD-025; its absence does not block release.)*

###### REQ-UX-041 — Like, follow, and compose from the feed are honest; like/follow reconcile on failure
- **Statement:** From the home timeline a user can like a chirp, follow a surfaced account, and start
  composing; like and follow update immediately and reconcile to the true state when they fail, posting a
  chirp is confirmed honestly, and the compose affordance opens the one composer without losing the feed.
- **Acceptance criteria:** Tapping like fills the control and increments the count immediately and, if the
  action fails, the control and count visibly revert to the true value with an honest error rather than
  leaving a wrong one; the same holds for follow/unfollow on an account surfaced in the feed; activating the
  compose affordance opens the composer over the feed without discarding the user's scroll/context, and a
  posted chirp becomes visible in the feed with an honest confirmation; a failed post is reported (the
  composer preserves the draft) and never shown as succeeded; every chirp is rendered by the same chirp card
  used on profiles, threads, and search (same avatar + display name + `@handle` + timestamp + like control),
  with no bespoke feed row.
- **Release gating:** true
- **Traces to:** REQ-PROD-016, REQ-PROD-020, REQ-PROD-021; J5. (Realizes the frozen seams
  `OPTIMISTIC_RECONCILE`, `COMPOSE_ENTRY_SEAM`, `SHARED_CHIRP_CARD`.)

###### REQ-UX-042 — A feed action on an expired session prompts re-sign-in and never fails silently
- **Statement:** When a user's session has expired or been invalidated, the next action they attempt from
  the home timeline surfaces a clear prompt to sign in again — never a silent failure and never an apparent
  success — and after signing in the user is returned to the feed and the attempted action.
- **Acceptance criteria:** After a session ends (including after a service restart), attempting a signed-in
  action from the feed (like, follow, open-and-post a chirp, delete own, report) shows a clear "session
  expired, sign in again" prompt and does not silently do nothing or appear to succeed; an optimistic like
  or follow attempted on an expired session reverts to its true state rather than showing a false result;
  after signing in, the user is returned to the home timeline able to complete the action, with any
  in-progress input preserved. Reading the feed itself (scrolling, paging) is never interrupted by this
  prompt.
- **Release gating:** true
- **Traces to:** REQ-PROD-006; J6. (Routes through the shared session-expired prompt, state model §1.6 /
  IA §3.3.)

###### 5.1 Traceability summary (REQ-UX → REQ-PROD / journeys)

| REQ-UX | Gating | Traces to | Realizes |
|---|---|---|---|
| REQ-UX-040 | true | REQ-PROD-018; (REQ-PROD-025 non-gating); J5, J6 | states designed in full |
| REQ-UX-041 | true | REQ-PROD-016, 020, 021; J5 | OPTIMISTIC_RECONCILE, COMPOSE_ENTRY_SEAM, SHARED_CHIRP_CARD |
| REQ-UX-042 | true | REQ-PROD-006; J6 | (session-expired via state model §1.6) |

All three ids are unique, fall within the owned `REQ-UX-040..042` sub-range, lie within the parent
`REQ-UX-020..069` range, and collide with no other child's range.

---

##### 6. Coverage & composition notes

- **Coverage.** Every state in SCR-home_timeline's matrix is designed with concrete copy intent and a
  recovery affordance: `content` §3.0, `loading` §3.1, `empty` §3.2 (two
  distinct variants), `error` §3.3, `end_of_list` §3.4, `session_expired` §3.5, `busy_submitting` §3.6
  (incl. optimistic reconcile), `success` §3.7. `not_found` is recorded out-of-scope with a reason (§3.8).
- **Frozen seams honoured.** `COMPOSE_ENTRY_SEAM` — this screen owns the compose *entry affordance* and
  routes to the one `SCR-compose`, never redefining the composer (§1, §2.5). `OPTIMISTIC_RECONCILE` — like
  and follow are the only optimistic actions and reconcile on failure; posting and delete are not optimistic
  (§3.6). `SHARED_CHIRP_CARD` — every chirp uses the one visual-system chirp card, no bespoke variant (§1,
  REQ-UX-041). `CONDITIONAL_REPLY_COHERENT` — the card's reply/open affordance exists per the reply-built
  stance and defers the thread to `the chirp-thread design` (§4), never a dead control.
- **Composed, not invented.** Builds on the IA signed-in nav model & screen map (IA §1.2/§2) and the global
  interaction patterns (optimistic+reconcile §3.2, auth-gating §3.3, honest feedback §3.1); composes from
  visual-system components and state blocks. No component or pattern was missing;
  nothing was invented and nothing is flagged. A reader who finds a needed component absent should treat its
  flag — not a local invention — as the correct outcome.
- **Boundary kept (what, not how).** Design intent, behaviour, and the testable bar only — no front-end
  code; routing/framework/persistence and the session/auth lifetime (`REQ-ID-*`) are out of UX scope.
- **Grounding.** Grounded in `Nousix-LLC/nswe-demo` @ `c805525`.

---

#### Compose a chirp — surface, flow & states

*(section: `screens_and_flows/core_loop/compose`  ·  screen: `SCR-compose`)*

This section designs **SCR-compose**, the one surface a signed-in person uses to write and post a chirp. It
is the single composer the whole core loop routes to (identity `COMPOSE_ENTRY_SEAM`): the Compose action in
the signed-in nav, the compose affordance on the home timeline, and onboarding's first-post offer all open
*this* surface — none of them re-draw a composer of its own. It is designed to the bar of *a designed
screen, not a skeleton*: the loaded happy path, and **every state in its frozen matrix** —
`[validation, busy_submitting, error, success, session_expired]` plus the `content` baseline — each with
concrete copy intent and a named recovery affordance, instantiating the shared state model
(the state model §1) from the shared visual-system components (`the visual language & design system` §3).

**What this surface owns, and what it defers.**

- It **owns** the composer's layout and placement, the happy-path post flow, the ≤280 limit with a visible
  character counter, inline validation, the **draft-survival guarantee** (`COMPOSE_DRAFT_SURVIVES`), the
  non-optimistic post feedback (`OPTIMISTIC_RECONCILE` reserves optimism for like/follow), and the
  `REQ-UX-043..044` register for this screen.
- It **composes against, and does not redefine**: the state model (the state model §1.3/§1.6/§1.7/§1.8/
  §1.9), the IA compose placement and interaction patterns (`the information architecture & interaction model` §2.2, §3.1, §3.3), and the
  visual-system components (textarea + message slot §3.3, character counter §3.4, button §3.1, inline alert
  §3.13, toast §3.12, session-expired block §3.14, chirp card §3.6). A presentation these components cannot
  serve is **flagged for resolution, never invented** (none was needed — see §6).
- It **defers the mechanism**: the session lifetime and its invalidation, the auth token, the create-chirp
  endpoint and persistence, and rate limiting are identity/trust and domain/ops concerns (`REQ-ID-*`,
  `REQ-DOM-*`, `REQ-OPS-*`). This section designs only what the user sees and does, and the testable bar.
  **What, not how.**

---

##### 1. Starting point (what exists today, grounded at `c805525`)

The current composer lives in `crates/chirp-frontend/src/compose.rs`, folded into the `/compose` route and
gated behind a **handle-only login**: logged-out, the route shows a login field; logged-in, it shows the
compose form. The compose form today is a title ("Compose"), a "Signed in as @handle" line, a label
("What's happening?"), a 3-row `<textarea>` with `placeholder="Say something chirpy…"` and a silent
`maxlength="280"`, a single "Chirp" button (label flips to "Chirping…" while busy, disabled when the body is
empty or a request is in flight), and two inline banners — an error (`role="alert"`) and a success
(`role="status"`).

What works and is **kept**:

- **Non-optimistic posting.** The button shows a busy label and the form resolves to an explicit success or
  an inline error — it does not fake the post. This section preserves that (it is the correct choice for a
  create action) and makes the feedback honest and consistent.
- **Draft survives a re-render.** Inputs are uncontrolled (the view never writes the field's `value`), so a
  re-render does not clobber what the user is typing. This section keeps that and extends the guarantee to a
  failed send and to session expiry.
- **Inline validation without a round-trip** for the obvious cases (empty / over-length rejected locally).

What is **improved** (today's gaps against the bar):

1. **No visible character counter and no approaching-limit warning** — the 280 limit is enforced only by a
   silent `maxlength`, which quietly swallows a 281st typed character and silently truncates pasted text.
   The user is never shown how much room is left, and an over-limit paste is cut without explanation. §3.1
   adds the counter and an honest over-limit treatment (`REQ-UX-043`, builds on `the visual language & design system` §3.4).
2. **A leaky success string.** Today's confirmation is `"Chirped! ({chirp.id})"`, surfacing an internal id.
   §4 (success) replaces it with an honest confirmation whose record is the posted chirp itself appearing.
3. **No routed session-expired treatment.** A post attempted after the session has gone is shown as a
   generic inline error, not the one re-sign-in prompt — a silent-ish failure the whole "no silent sign-out"
   guarantee (`REQ-PROD-006`) forbids. §4 (session_expired) routes it through the shared prompt
   (the state model §1.6) with the draft preserved.
4. **The composer is only a full route.** Per the IA (`the information architecture & interaction model` §2.2) the composer should be
   reachable in-context (inline or modal over the feed) so a person composes without abandoning the
   timeline, and the current `/compose` route is generalized, not broken. §2 designs it as that one
   in-context surface while remaining deep-linkable.

The handle-only login currently folded into `/compose` is **not** this screen's concern — credentialed
sign-in/sign-up/recovery are the entry/auth section's (`SCR-signin`/`SCR-signup`/`SCR-recovery`), and the
logged-out and signed-in navigation that reaches the composer is the IA model's (`the information architecture & interaction model` §1).
This surface is the composer a *signed-in* person uses; where a signed-in compose meets an expired session
it routes to re-sign-in (§4, session_expired).

---

##### 2. The surface — placement, layout & anatomy (wireframe level)

###### 2.1 Placement (the one composer, reached in context)

SCR-compose is a **contextual surface** in the signed-in core loop (`the information architecture & interaction model` §2.1/§2.2): it opens
**over or within the home timeline** without discarding the timeline context behind it, and it is also
reachable as a stable destination so it survives a refresh and a deep link. It is realized as either an
**inline composer at the top of the feed** or a **modal over the feed** (visual-system modal §3.11); the
choice between inline and modal is a responsive-layout decision (narrow viewports favor a full-height modal,
wide viewports favor the inline composer) and is deferred there — both render the identical anatomy and
states below. The current `/compose` route is preserved as the deep-link/refresh fallback destination.

**Entry points route here; they do not re-draw it** (identity `COMPOSE_ENTRY_SEAM`): the signed-in nav's
Compose action (`the visual language & design system` §3.9 nav primary CTA), the compose affordance on `SCR-home_timeline`
(owned by `the home-timeline design`), and onboarding's first-post offer (owned by `the onboarding design`) each
**open this surface**. Those owners reference the composer; they do not redefine its layout, validation, or
states. Opening the composer never discards an in-progress draft that is already there (§5).

###### 2.2 Layout anatomy (loaded `content` baseline)

Built entirely from visual-system components and tokens — no raw values, no one-off controls
(`the visual language & design system` §5 rules 1–2):

```
┌─ composer (inline at feed top, or modal surface · elevation.2 + scrim when modal) ─────┐
│                                                                           [ ✕ close ]  │  (close only in the modal realization)
│  ⬡  Display Name                                                                       │  identity cluster (visual-system §3.5):
│      @handle   · you                                                                   │  avatar (sm/md) + name + @handle + "you" badge
│  ────────────────────────────────────────────────────────────────────────────────    │
│  ┌──────────────────────────────────────────────────────────────────────────────┐    │
│  │  What's happening?                                                             │    │  textarea (visual-system §3.3), label associated,
│  │  (the draft — author's line breaks preserved; grows with content)             │    │  placeholder is NOT the label, autofocus on open
│  │                                                                                │    │
│  └──────────────────────────────────────────────────────────────────────────────┘    │
│  [ message slot — inline validation / error lives here, at the field ]                 │  (visual-system §3.3 message slot / §3.13 inline alert)
│                                                                                        │
│  ·························································································· │
│                                                            268 left        [  Chirp  ] │  character counter (visual-system §3.4) · primary button (§3.1)
└────────────────────────────────────────────────────────────────────────────────────┘
```

- **Identity cluster** (`the visual language & design system` §3.5): the signed-in author's avatar (monogram placeholder today —
  chirp has no image upload), display name (`font.label`, `text.primary`) and `@handle` (`font.meta`,
  `text.secondary`) with the "you" badge — so the person sees *as whom* they are about to post. This also
  carries the signed-in indicator the IA requires (`the information architecture & interaction model` §3.6, `REQ-UX-015`); it replaces
  today's plain "Signed in as @handle" line.
- **Textarea** (`the visual language & design system` §3.3): the single compose field. Label "What's happening?" is associated
  with the field (today's `<label for>` is kept); the placeholder ("Say something chirpy…") is never the only
  label. The field **autofocuses on open** so the person can type immediately, grows to fit a few lines, and
  **preserves the author's line breaks** (whitespace is content — the live `ChirpText` does not trim).
- **Character counter** (`the visual language & design system` §3.4): a `font.caption` remaining-count indicator paired with the
  field — the piece missing today (§3.1).
- **Primary button** (`the visual language & design system` §3.1 `primary`, the screen's single primary action): label "Chirp",
  pill, min target `layout.touch.min` 44px. It is the only primary on the surface.
- **Close** (modal realization only): a `secondary`/`icon-button` "✕" with an accessible name; closing with a
  non-empty draft is handled in §5 (the draft is not silently thrown away).

The composer sits in the single readable column (`layout.content.max` 600px). Chirps the person then sees
posted render in the one shared chirp card (`SHARED_CHIRP_CARD`, `the visual language & design system` §3.6) — this surface does
not draw a chirp of its own.

---

##### 3. The happy path (loaded → posted)

1. **Open.** An entry point opens the composer over/within the feed; the textarea autofocuses. If a draft
   from a previous unfinished attempt exists, it is already present (§5).
2. **Type.** As the person types, the **counter updates live** showing characters remaining; the Chirp
   button is **enabled only when the body is non-empty and no request is in flight** (today's enablement
   rule, kept) — an empty composer cannot post.
3. **Submit.** Activating Chirp validates locally (≤280, non-empty — §4 validation), then enters
   `busy_submitting`: the button shows it is working and is non-interactive so the post cannot be
   double-fired (§4 busy_submitting). Posting is **not optimistic** — nothing is shown as posted until the
   server confirms.
4. **Resolve.** Exactly one honest outcome (`the information architecture & interaction model` §3.1): on success the composer clears and
   confirms, and the **new chirp is observable** at the top of the feed (§4 success); on failure the
   composer returns to an editable state with the **draft intact** and an honest, recoverable error (§4
   error); if the session had expired, the person is routed to re-sign-in with the draft preserved and
   returned to the post afterwards (§4 session_expired).

---

##### 4. The designed states

Each state instantiates the shared model (the state model §1.x) and the shared conventions
(the per-screen state matrix); copy is **representative intent**, adapted to this screen, not a
frozen string. The required matrix for SCR-compose is
`[validation, busy_submitting, error, success, session_expired]` + the implied `content` baseline — all are
designed below.

###### 4.0 `content` — the loaded composer
- **When:** the composer is open and ready (the §2.2 layout). The baseline every other state departs from.
- **Copy & voice:** the field label ("What's happening?") and placeholder ("Say something chirpy…"); the
  counter shows the full budget remaining ("280 left") on an empty field.
- **Recovery:** n/a (baseline).
- **Composition:** rendered by the primary components (textarea §3.3, counter §3.4, button §3.1, identity
  cluster §3.5) — no state block (the state model §1.0).

###### 4.1 `validation` — inline, actionable, at or before submit (the state model §1.7)
- **When:** the body violates a rule — **empty** (the Chirp button is disabled, so this surfaces only on an
  all-whitespace or edge attempt) or **over the 280-character limit** (reached by typing past the limit or
  pasting a longer block).
- **Copy & voice:** specific and at the field, never a generic "invalid input" and never a raw server/API
  message. Over-limit: *"Your chirp is over the 280-character limit — trim it to post."* Empty-on-attempt:
  *"Write something to chirp."*
- **Recovery:** the message sits in the field's **message slot** (`the visual language & design system` §3.3), the counter
  flips to its **over-limit** style (`error.ink`, `the visual language & design system` §3.4), and the **submit is blocked with
  the typed text kept** — nothing the person wrote is lost while the submit is blocked. The counter's
  **approaching-limit** warning (`warning.ink`, within ~20) is shown *before* the limit so the person is
  warned, not surprised.
- **Composition:** textarea error state + character counter states (`the visual language & design system` §3.3/§3.4);
  validation is honest about the submit control's state (disabled vs errored — one consistent choice).
- **Improvement over today:** replaces the silent `maxlength` truncation with a visible counter, an
  approaching-limit warning, and an explicit over-limit message; a pasted over-length block is explained,
  not silently cut.

###### 4.2 `busy_submitting` — in flight, no duplicate, holds its place (the state model §1.8)
- **When:** a valid chirp has been submitted and the create request is in flight.
- **Copy & voice:** minimal — the button shows the work ("Chirping…", today's label, kept); the point is to
  show it is working and prevent a second submission, not to narrate.
- **Recovery:** the button is **non-interactive while busy** and **preserves its size** so the layout does
  not jump (`the visual language & design system` §3.0 busy control); the field is left editable-but-pending and is **never
  cleared until success**, so a failure returns to an editable composer with the draft intact (→ §4.3). The
  control never strands the person in a permanent busy state.
- **Composition:** the §3.0 busy control state on the primary button. Posting is **not optimistic**
  (`OPTIMISTIC_RECONCILE`): the chirp is not shown anywhere as posted during `busy_submitting` — only
  like/follow may update ahead of confirmation, and this surface has neither.

###### 4.3 `error` — honest, recoverable, draft preserved (the state model §1.3)
- **When:** the post failed — a transient network/server failure, or any non-validation request failure.
  (A failure that is actually an expired session is the `session_expired` case, §4.5, not a generic error.)
- **Copy & voice:** honest, plain, non-blaming, and explicit that the **text is safe**, never a raw error
  code and never a false success. *"Your chirp didn't send. Your text is safe — try again."*
- **Recovery:** an **inline alert at the composer** (`the visual language & design system` §3.13, `role="alert"` — today's
  banner, re-skinned) rather than a toast the person could miss for a blocking failure; **Retry re-posts in
  place without a full page reload** (`REQ-PROD-018`), and the **draft is kept in the field** so the person
  retries without retyping (`REQ-PROD-017`, identity `COMPOSE_DRAFT_SURVIVES`).
- **Composition:** inline alert §3.13; the field is not cleared (today's behavior — `clear_field` runs only
  on success — is correct and is kept as the explicit guarantee).

###### 4.4 `success` — honest confirmation, result observable, no id leak (the state model §1.9)
- **When:** the server confirmed the chirp was created.
- **Copy & voice:** a brief, honest confirmation — *"Chirp posted."* — with **no internal id surfaced** (the
  live `"Chirped! ({id})"` leak is removed; the record of success is the chirp itself appearing).
- **Recovery:** n/a (positive terminal) — but the **result is observable**: the composer clears, and the new
  chirp appears at the top of the home timeline (rendered by the shared chirp card, `SHARED_CHIRP_CARD`). The
  counter resets to the full budget; a modal composer closes back to the feed.
- **Composition:** a **toast** for the brief confirmation (`the visual language & design system` §3.12), or an inline success
  (`role="status"`, today's success banner) where the inline composer stays open; the observable new chirp
  is the confirmation of record. Honest only — success is shown only when the post truly succeeded
  (`the information architecture & interaction model` §3.1).

###### 4.5 `session_expired` — route to the one re-sign-in prompt, draft kept (the state model §1.6)
- **When:** the person attempts to post after the session has **expired or been invalidated** — including
  after a service restart, which wipes in-memory sessions today (`REQ-PROD-006`; J6 second-visit failure).
- **Copy & voice:** the **single shared re-sign-in prompt**, not a per-screen message and not a generic
  error: *"Your session has expired. Sign in again to post."* — never a silent fail and never a false
  "posted".
- **Recovery:** routed through the **auth-gating pattern** (`the information architecture & interaction model` §3.3) to sign-in,
  **preserving the draft and the intent** so that after signing in the person is returned to the composer
  with their text intact and can post (return-to-intent, `the information architecture & interaction model` §2.4). The draft is **not
  discarded** by the expiry (`REQ-PROD-017`).
- **Composition:** the shared **session-expired prompt block** (`the visual language & design system` §3.14), inline on the
  attempted post; the signed-in indicator (the identity cluster / nav) flips to logged-out
  (`the information architecture & interaction model` §3.6). The session's lifetime and invalidation mechanism are `REQ-ID-*`; only the
  prompt, the draft preservation, and the resume are designed here.
- **Improvement over today:** the live view surfaces an expired-session post as a generic inline error;
  this routes it through the one honest re-sign-in prompt the "no silent sign-out" guarantee depends on.

---

##### 5. The draft-survival guarantee (identity `COMPOSE_DRAFT_SURVIVES`)

An unsent draft is **never silently discarded**. It survives, concretely, all of:

- **a re-render of the surface** — the field is uncontrolled, so re-rendering never overwrites the typed text
  (today's behavior, kept and made an explicit requirement);
- **a failed send** — on `error` the composer returns to an editable field with the draft intact and a Retry,
  the field cleared only on a confirmed success (§4.3/§4.4);
- **a session expiry** — the draft is carried through the re-sign-in route and restored on return (§4.5);
- **closing a modal composer with text in it** — closing with a non-empty draft keeps the draft so the next
  open restores it (a lightweight "keep your draft?" affordance is acceptable; silently dropping it is not).

This is the compose instantiation of the product-wide input-preserving principle (the state model §0.2
rule 4, §2 rule 4; `REQ-PROD-017`). Draft **persistence across a full browser close** is a larger storage
question (local persistence) and is an **honest, recorded extension**, not part of this gating guarantee —
flagged here for the backlog, not silently promised.

---

##### 6. Composition check — maps cleanly onto existing components (no gap)

| This screen needs | Rendered by (visual-system) | IA / state pattern |
|---|---|---|
| Compose field + label + message slot | Textarea & field group §3.3 | validation §3.5 / `state-patterns` §1.7 |
| ≤280 counter, approaching/over-limit | Character counter §3.4 | `state-patterns` §1.7 |
| Post action, busy state | Button §3.1 (`primary`) + §3.0 busy | feedback §3.1 / `state-patterns` §1.8 |
| Author identity + signed-in indicator | Avatar & identity cluster §3.5 | indicator §3.6 |
| Send-failure message (draft kept) | Inline alert §3.13 | honest error §3.1 / `state-patterns` §1.3 |
| Brief success confirmation | Toast §3.12 / inline success §3.13 | §3.1 / `state-patterns` §1.9 |
| Session-expired prompt | Session-expired block §3.14 | auth-gating §3.3 / `state-patterns` §1.6 |
| In-context / modal placement | Modal §3.11 + scrim | compose placement §2.2 |
| The posted chirp (observed result) | Chirp card §3.6 (shared) | `SHARED_CHIRP_CARD` |

Every piece this surface needs already exists in the visual system and the IA model; **no component is
invented and no gap is flagged**. Had one been missing, it would be flagged for resolution, not patched here.

---

##### 7. Requirements & acceptance criteria — REQ-UX-043 .. REQ-UX-044

Range owned by `the compose design` (`REQ-UX-043 .. REQ-UX-044`). Each
requirement is stated from the user's side, is testable by a stranger exercising the live product, carries a
`release_gating` flag, and traces to ≥1 `REQ-PROD-*` and/or journey. Both ids are unique, fall inside the
owned sub-range, and lie within this section's `REQ-UX-040..051` and the experience `REQ-UX-020..069`.

###### REQ-UX-043 — Compose and post a chirp, with a visible limit and honest, non-optimistic feedback
- **Statement:** A signed-in person can write a chirp of up to 280 characters in one composer reached from
  the core loop, see how much room is left, and post it with clear busy/success/error feedback — and the
  posted chirp becomes observable in the product.
- **Acceptance criteria:** From the home timeline (via the Compose action or the feed's compose affordance) a
  signed-in person opens a composer, types text, and sees a live remaining-character count that warns as the
  280 limit nears and prevents posting over the limit with a clear at-field message (not a silent cut and not
  a raw error); submitting shows a busy state that cannot be double-fired and resolves to either a clear
  confirmation with the new chirp visible at the top of the timeline, or an honest, recoverable error — the
  chirp is never shown as posted before the server confirms it. The same single composer is reached from the
  nav Compose action and the timeline affordance (not two different compose UIs).
- **Release gating:** true
- **Traces to:** REQ-PROD-016; J5. (Builds on `the visual language & design system` REQ-UX-078 counter and the IA
  `COMPOSE_ENTRY_SEAM`.)

###### REQ-UX-044 — Composing never silently loses the draft; failure and session-expiry are honest
- **Statement:** A person's in-progress chirp is never silently lost — it survives a failed send, a
  re-render of the composer, and a session that has expired — and a post attempted on an expired session
  surfaces a clear re-sign-in prompt rather than a silent failure or a false success.
- **Acceptance criteria:** When a chirp send fails, the typed text remains in the composer and an honest
  message with a retry is shown, so the person can retry without retyping; re-opening or re-rendering the
  composer does not wipe the draft; and when a post is attempted after the session has expired (for example
  after a service restart), the person is shown a clear "session expired — sign in again to post" prompt
  (never a silent no-op and never an apparent success), and after signing in they are returned to the
  composer with their draft intact and can post it.
- **Release gating:** true
- **Traces to:** REQ-PROD-017, REQ-PROD-006; J5, J6. (Instantiates the state model REQ-UX-023/026/028
  for this screen; identity `COMPOSE_DRAFT_SURVIVES`.)

###### 7.1 Traceability summary

| REQ-UX | Gating | Traces to |
|---|---|---|
| REQ-UX-043 | true | REQ-PROD-016; J5 |
| REQ-UX-044 | true | REQ-PROD-017, REQ-PROD-006; J5, J6 |

---

##### 8. Coverage & composition notes

- **Coverage.** SCR-compose is designed with every state in its frozen matrix
  (`validation, busy_submitting, error, success, session_expired`) plus the `content` baseline — each with
  concrete copy intent and a named recovery affordance (§4), instantiating the state model §1 and
  following the per-screen state matrix. No state was dropped.
- **Seams honoured here.** `COMPOSE_DRAFT_SURVIVES` (§4.3/§4.5/§5), `OPTIMISTIC_RECONCILE` — posting is
  non-optimistic (§4.2), `COMPOSE_ENTRY_SEAM` — one composer that entry points route to (§2.1), and
  `SHARED_CHIRP_CARD` — the posted chirp uses the shared card (§4.4/§6). Every state is designed, not skeletal, and every element composes from existing components (§4, §6).
- **Cross-screen seams owned elsewhere.** The compose *affordances* on `SCR-home_timeline` and
  `SCR-onboarding` are owned by those sections and only reference this composer; if either needs a composer
  behavior this fragment does not describe, that is a seam note for resolution, not an invention there.
- **REQ-UX range.** Only `REQ-UX-043..044` minted; the full `COMPOSE_DRAFT_SURVIVES` and non-optimistic
  guarantees are carried within those two ids (the per-state consistency requirements live in the state
  model's `REQ-UX-023/026/028/029`, which this screen instantiates rather than re-mints). No id outside the
  owned sub-range is used; no shortfall to report.
- **Honest cut, recorded (not a silent omission).** Draft **persistence across a full browser close/relaunch**
  (local-storage-style persistence) is left out of the gating draft-survival guarantee and flagged for the
  backlog (§5). An image/media attachment on a chirp is out of scope for this product version (chirp is
  text-first; no upload exists today) and is noted, not designed.

---

#### Profile — SCR-profile (own & other-user)

*(section: `screens_and_flows/core_loop/profile`)*

A profile is the account seen whole: who the user is, how many people they follow and are followed by,
and the chirps they have posted. It is read by two different people with two different intents — **its
owner** (checking and managing their own presence — J6) and **another user** (deciding whether this is
someone to follow — J5) — and the one screen must serve both honestly. Reading a profile never requires an
account (REQ-PROD-013); the **logged-out** reader of a profile is the entry/auth section's and the IA model's
concern (ia-interaction §1.1/§3.3). This fragment designs the **signed-in** profile and routes an expired
session through the one shared re-sign-in prompt (state model §1.6).

**What this screen owns, and what it defers.** It owns the profile's layout, its happy-path flow, each state
in its matrix, the paged chirps list, the follower/following counts, and the **optimistic follow/unfollow**
action. It does **not** design the **edit-profile** screen (`SCR-edit_profile`) or the **settings** hub
(`SCR-settings_home`) — both are sibling-owned; this screen only exposes the affordance and **references the
route**. It composes the **one shared chirp card** (identity `SHARED_CHIRP_CARD`) for every chirp, routes any
"compose" affordance to the one composer (`COMPOSE_ENTRY_SEAM`), and never invents a component the visual
system lacks — a gap is flagged for resolution (§6), not patched here.

---

##### 1. Content & layout (wireframe-level)

Single centred reading column (`layout.content.max` 600px, `layout.gutter`), under the persistent signed-in
nav (ia-interaction §1.2; nav is **not** blocked by a profile content load). Top-to-bottom:

```
┌─ signed-in nav (persistent; identity indicator ▾) ──────────────────────────┐   (owned by IA/nav; not this screen)
├──────────────────────────────────────────────────────────────────────────────┤
│  PROFILE HEADER  (identity cluster — visual-system §3.5)                        │
│  ┌──────┐                                                                       │
│  │avatar│  Display Name                                   [ own: Edit profile ] │  ← xl 64 monogram placeholder (no
│  │  xl  │  @handle            [ "you" badge on own ]      [ other: Follow ▸▾ ]  │     uploads today — §3.5), honest
│  └──────┘                                                 [ other: ⋯ overflow ] │     about the absence
│                                                                                 │
│  Bio text (font.body; author line breaks preserved; omitted/placeholder if none)│
│                                                                                 │
│  ●  <N> Following      ●  <M> Followers        (count pills — §3.15; §2.4)       │  ← display metrics (see §2.4)
├──────────────────────────────────────────────────────────────────────────────┤
│  CHIRPS                                                                          │
│  ┌───────────────────────────── shared chirp card (§3.6) ─────────────────────┐ │  ← SHARED_CHIRP_CARD: identity
│  │ avatar · Display Name · @handle · 2h ·            ⋯                         │ │     cluster + body + like control
│  │ chirp body text …                                                          │ │     + relative timestamp; identical
│  │ ♥ 12   (reply affordance only where threads are built — SCR-chirp_thread)   │ │     anatomy to timeline/thread/search
│  └────────────────────────────────────────────────────────────────────────────┘ │
│  … (reverse-chronological list of this account's chirps) …                       │
│  [ Load more ]                                                                   │  ← cursor paging → end_of_list (§2.3)
└──────────────────────────────────────────────────────────────────────────────┘
```

- **Header** uses the identity cluster (visual-system §3.5): `xl` 64 avatar (generated monogram placeholder
  — chirp has no image upload today, so this is honest rather than a broken image), **Display Name**
  (`font.label`→profile uses `font.h1` for the name) + **`@handle`** (`font.meta`, `text.secondary`), the
  **bio** (`font.body`, line breaks preserved; a short placeholder line when empty, never a broken gap), and
  the follower/following **count pills** (§3.15, one product-wide count-format rule per §3.8).
- **Own vs other** is the header's defining difference — see §2.2.
- **Chirps** render as a vertical, divider-separated list of the **shared chirp card** (§3.6, `default`
  variant); no profile-specific chirp row. The like control inside each card is the shared optimistic like
  (§3.8, state model §1.8); the reply affordance appears on a card **only** where threads are built
  (`SCR-chirp_thread`, sibling-owned) — never a dead control. A deleted chirp inside the list renders as the
  card `tombstone` variant (§3.6), not a vanished row.

---

##### 2. Happy path & key behaviours

###### 2.1 Happy path (content)
A user opens a profile (from an author link on any chirp, or — for their own — from the account menu's
*Profile*, ia-interaction §1.4). The header resolves and the first page of the account's chirps loads
(`loading` → `content`). The reader sees who the account is, their counts, and their chirps newest-first, and
can **Load more** to page back through older chirps until the end (`end_of_list`). From here the owner can
reach **Edit profile** / **Settings**; another user can **Follow/Unfollow**. Transition into `content` from
`loading` must not jump the layout (shape-matched profile skeleton, state model §1.1).

###### 2.2 Own-vs-other distinction (REQ-UX-045; ia-interaction §1.2/§2, REQ-UX-006/REQ-PROD-008)
The same screen renders two header action sets, chosen by whether the viewed account is the signed-in user's
own:
- **Own profile:** a **"you" badge** on the identity cluster; an **Edit profile** affordance (`secondary`
  button, visual-system §3.1) that **routes to `SCR-edit_profile`**; and a **route into Settings**
  (`SCR-settings_home`). *Both target screens are sibling-owned — this screen exposes the affordance and
  references the route only; it does not design the edit form or the settings hub.* No Follow control appears
  on one's own profile. The empty-chirps state invites a first post (§3, `empty`).
- **Other user's profile:** a **Follow / Unfollow** control (§2.3, §3) and an **overflow `⋯` menu** (§3.10)
  whose **Report/block** item routes to `SCR-report_block` (sibling-owned — reference only). **No** Edit /
  Settings controls appear. (A logged-out reader sees neither follow nor edit; that case is the entry/auth
  section's — this screen is the signed-in profile.)

###### 2.3 Chirps list, paging & counts
- **Paging:** cursor-based "**Load more**" appends older chirps; reaching the last page shows the explicit
  `end_of_list` marker (§3), distinct from `empty`.
- **Shared card:** every chirp is the one shared chirp card (`SHARED_CHIRP_CARD`); the per-chirp like is the
  shared optimistic like (§3.8), reconciling on failure exactly like follow (§2.4) — governed product-wide by
  the state model, not re-specified per card here.

###### 2.4 Follow / unfollow — optimistic & reconciled (REQ-UX-046; identity `OPTIMISTIC_RECONCILE`)
Follow/unfollow is one of the **only two** optimistic actions in the core loop (the other is like; identity
`OPTIMISTIC_RECONCILE`, ia-interaction §3.2, state model §1.8). On tap:
1. The control flips immediately (`Follow` → `Following`, or the reverse) and the **follower count moves with
   it** (optimistic), with a brief in-flight `busy` visual on the control (§3.7); the layout does not jump.
2. On **success** the control and count settle on the true state (`success`, state model §1.9) — the result
   itself is the confirmation; no toast is required for this quick action.
3. On **failure** the control and the count **visibly revert to the true prior state** and an honest,
   non-blocking error is surfaced (toast with *Retry*, §3.12; state model §1.3/§1.8) — never a wrong count
   left on screen and never a silent drop.
Follow/unfollow is **reversible and therefore never gated behind a confirmation** (ia-interaction §3.4).
Unfollow uses the `Following` → hover-reveal `Unfollow` affordance (§3.7); it is not destructive and needs no
dialog.

- **Counts are honest display metrics.** The follower/following numbers are shown and kept truthful
  (the follower count reconciles with the follow action above). A dedicated **followers / following list**
  screen is **not in the frozen screen inventory** (the screen inventory & state vocabulary); the counts are
  therefore **display-only in this release** and do not open a list. This is recorded as a deliberate,
  honest cut (not an invented screen) and **flagged for resolution** (§6) in case the register wants
  to carry it — it is NOT silently added here.

---

##### 3. States (the matrix for SCR-profile — designed, not skeletal)

Required states (the per-screen state matrix:SCR-profile`):
`[loading, empty, error, not_found, end_of_list, busy_submitting, success, session_expired]` (+ `content`,
implied). Each instantiates the shared state model (the state model §1) with the matching convention
(the per-screen state matrix): copy **intent** (exemplars illustrative, not frozen strings) +
recovery affordance + the visual-system block that renders it. **The empty-vs-not_found distinction is the
load-bearing one for this screen** and is called out explicitly.

| State | When it occurs on SCR-profile | Copy intent (exemplar) | Recovery affordance | Renders with |
|---|---|---|---|---|
| `content` | Header resolved and ≥1 chirp loaded. | n/a — the profile itself is the message. | n/a (positive). | Identity cluster §3.5 + chirp card list §3.6. |
| `loading` | First fetch of the header and/or the first chirps page, or a retry. | Wordless. | A stalled load offers **Try again** (→ in-place `error` recovery); never an endless spinner. | **Profile skeleton** = header skeleton + chirp-list skeleton on `bg.subtle` (state block §3.14); chrome not blocked. |
| `empty` | **The profile EXISTS but has no chirps.** | **Own:** "You haven't chirped yet — write your first one." · **Other:** "@handle hasn't chirped yet." | **Own:** **Compose** (routes to the one composer, `COMPOSE_ENTRY_SEAM`). **Other:** no action needed — the header (incl. Follow) is still fully present. | **Empty** state block §3.14, shown **in the chirps region only** — the header, counts, and follow/edit affordances remain. |
| `not_found` | **The profile DOES NOT EXIST or was removed** — unknown handle/id, or a **deleted** account (REQ-PROD-012/019). | Distinguish where known: "That profile doesn't exist." · "This account has been deleted." | A **route back to a known place** — Home timeline / Search / Back. | **Not-found** state block §3.14, replacing the **whole** surface (no header — there is no account to show). |
| `error` | A header or chirps fetch **failed** (transient/server). | "Something went wrong loading this profile." (whole surface) · "Couldn't load more chirps." (list page). | **Retry in place, no full page reload** (REQ-PROD-018); a partial failure retries just the failed region, keeping the loaded header. | **Error** state block + **Retry** §3.14 (whole/section); inline retry at the list foot for a failed "Load more". |
| `end_of_list` | The chirps list returned its **last page**. | "No more chirps." (explicit end, clearly ≠ `empty`). | Back to top / refresh for new. | **End-of-list** state block §3.14 at the list foot. |
| `busy_submitting` | **Follow/unfollow in flight** (the only submit on this screen; the per-card like shares this). | Minimal — the control shows the in-flight busy; the optimistic flip is already visible (§2.4). | On failure → reconcile to true state (see `error`/§2.4); the control never strands in busy. | **Follow control busy** state §3.7 (size preserved, no layout jump). |
| `success` | Follow/unfollow **completed**. | n/a — the reconciled control + count IS the honest confirmation; **never** a false success. | n/a (positive terminal); result observable (control + count match reality). | Reconciled follow control §3.7; no toast needed for this quick action. |
| `session_expired` | An **authenticated action** (follow/unfollow, or the owner's Edit route) attempted **after the session expired/was invalidated** — including after a service restart (J6) — or a signed-in profile fetch after expiry. | "Your session has expired. Sign in again to continue." (the **one** shared prompt, state model §1.6). | Route to **sign-in preserving the user's intent** (ia-interaction §2.4/§3.3) so they resume the exact action (e.g. the follow) afterwards; in-progress state is not silently dropped. | **Session-expired prompt** block §3.14 via the auth-gating pattern (ia-interaction §3.3); indicator flips (§3.6). **Never a silent fail or false success** (REQ-PROD-006). |

**empty ≠ not_found (explicit).** `empty` = *a real account that has posted nothing* → the header, counts,
and Follow/Edit are all present; only the chirps region shows the empty block (own: invite a first post;
other: a plain "hasn't chirped yet"). `not_found` = *there is no such account (or it was deleted)* → there is
no header to show, so the whole surface is the Not-found block with a route back, distinguishing "never
existed" from "deleted" where the product knows (ties REQ-PROD-012). A missing single chirp *inside* an
otherwise-loaded list is neither — it is the card `tombstone` (§3.6), per the state model §1.4.

---

##### 4. Flow summary

```
open profile ─▶ loading ─┬─▶ content ─┬─▶ Load more ─▶ (more) ─▶ content
                         │            └─▶ (last page) ─▶ end_of_list
                         ├─▶ empty            (exists, no chirps)      ── own: Compose ▸ composer
                         ├─▶ not_found        (no such / deleted)      ── ▸ Home / Search / Back
                         └─▶ error            (fetch failed)           ── ▸ Retry in place

other-user header ─▶ Follow/Unfollow ─▶ busy_submitting (optimistic flip + count move)
                                           ├─▶ success   (control + count settle to truth)
                                           ├─▶ error     (revert control + count; honest Retry)   [OPTIMISTIC_RECONCILE]
                                           └─▶ session_expired ─▶ shared re-sign-in prompt ─▶ resume the follow

own header ─▶ Edit profile ─▶ (route to SCR-edit_profile — sibling-owned)
          └▶ Settings      ─▶ (route to SCR-settings_home — sibling-owned)
```

---

##### 5. Requirements & acceptance criteria — REQ-UX-045 .. REQ-UX-046

Owned sub-range per `the frozen design contract:the profile design` = **REQ-UX-045 ..
REQ-UX-046** (inside the parent `REQ-UX-040..051`, within `REQ-UX-020..069`). Each requirement is stated from
the user's side, is testable by a stranger exercising the live product, carries a `release_gating` flag, and
traces to ≥1 `REQ-PROD-*` and/or journey (`req_ux_schema`: id · statement · acceptance_criteria ·
release_gating · traces_to).

###### REQ-UX-045 — A profile presents the user honestly, distinguishes own from other, and handles its real states
- **Statement:** A profile shows who the account is (name, `@handle`, bio, follower/following counts) and
  their chirps newest-first with paging; it shows an owner their own-profile controls and another viewer the
  follow control instead; and it presents loading, empty, not-found, error, end-of-list, and session-expired
  as honest, distinct designed states — in particular an account that exists but has not chirped is not shown
  as "not found".
- **Acceptance criteria:** Opening an existing account shows its identity (display name, `@handle`, bio or a
  sensible placeholder) and follower/following counts, and its chirps most-recent-first with a "load more"
  that reveals older chirps and a clear end-of-list when there are no more; while fetching, a shape-matched
  placeholder shows (not a blank page) and a failed fetch shows an honest message with a retry that does not
  reload the whole page; **viewing one's own profile** shows an edit-profile affordance and a route to
  settings and **no follow control**, while **viewing another account** shows a follow/unfollow control and
  **no edit/settings controls**; an account that **exists but has no chirps** shows a clear "hasn't chirped
  yet" (and, on one's own, an invitation to post) — **visibly different** from the result for a **handle that
  does not exist or an account that was deleted**, which says so plainly and offers a route back; and
  attempting a signed-in action after the session has expired shows the shared "sign in again" prompt rather
  than failing silently or appearing to succeed.
- **Release gating:** true
- **Traces to:** REQ-PROD-019, REQ-PROD-008, REQ-PROD-013, REQ-PROD-012; J5, J6.

###### REQ-UX-046 — Follow/unfollow from a profile is optimistic, truthful, and reconciles on failure
- **Statement:** From another user's profile, following or unfollowing updates immediately and the follower
  count moves with it, settling on the true state on success and visibly reverting the control and the count
  on failure with an honest error — never leaving a wrong state or count and never silently dropping the
  action; and the action is never gated behind a confirmation dialog.
- **Acceptance criteria:** On another account's profile the control reads "Follow" when not following and
  "Following" when following; activating it changes the control and the follower count straight away and
  shows a brief in-progress state, and on success both match the real resulting state; if the action fails,
  the control and the count visibly return to their true prior values and an honest, recoverable error is
  reported (not a silent drop and not a wrong count); following/unfollowing proceeds without a confirmation
  dialog; and no follow control appears on the user's own profile.
- **Release gating:** true
- **Traces to:** REQ-PROD-020; J5, J6.

###### 5.1 Traceability summary (REQ-UX → REQ-PROD / journeys)

| REQ-UX | Gating | Traces to |
|---|---|---|
| REQ-UX-045 | true | REQ-PROD-019, 008, 013, 012; J5, J6 |
| REQ-UX-046 | true | REQ-PROD-020; J5, J6 |

Both ids are unique, fall within the owned `REQ-UX-045..046` sub-range, lie within the parent
`REQ-UX-040..051` and grandparent `REQ-UX-020..069` ranges, and each is user-side, testable against the live
product, release-gating-flagged, and traced.

---

##### 6. Coverage & composition notes

- **Coverage.** Every state in the SCR-profile matrix is designed with concrete copy intent and a recovery
  affordance (§3), including the explicit **empty-vs-not_found** distinction required by the brief . `content` is the loaded baseline (§2.1). No matrix state is dropped.
- **Seam compliance.** Chirps use the one shared chirp card (`SHARED_CHIRP_CARD`); follow/unfollow and the
  per-card like are the shared optimistic+reconcile pattern (`OPTIMISTIC_RECONCILE`); the own-profile
  "compose a first chirp" routes to the one composer (`COMPOSE_ENTRY_SEAM`). The edit-profile and settings
  screens are **sibling-owned** — this screen references the routes (`SCR-edit_profile`,
  `SCR-settings_home`) and does **not** design them.
- **Composition (components consistent).** Every element is a visual-system component (identity cluster §3.5,
  chirp card §3.6, follow control §3.7, like §3.8, menu §3.10, state blocks §3.14, count pill §3.15) built on
  the IA model (own-vs-other §1.2, auth-gating §3.3, optimistic reconcile §3.2); **no component is invented**.
- **Flag (not invented).** A dedicated **followers / following list** screen is **absent from the frozen
  screen inventory**; the counts are therefore designed as **display-only** this release and this is reported
  for resolution as a recorded cut — **not** silently added as a new screen or route.
- **Boundary (what, not how).** What-not-how throughout — behaviour, layout intent, and the testable bar only, no
  front-end code; session/auth lifetime and the deletion/edit mechanisms are deferred to `REQ-ID-*` /
  sibling screens.

---

#### Chirp detail / thread + reply — SCR-chirp_thread

*(section: `screens_and_flows/core_loop/chirp_thread`)*

This screen is where a single chirp becomes a **conversation**: a reader opens a chirp to read it in full,
see the replies to it, and — if signed in — add their own reply. It is the one surface in the core loop
that realises `REQ-PROD-023` (replies). The product-requirements register and the IA both frame replies as
**conditional**: *either* the product gives a chirp a coherent read-thread-and-reply destination, *or* it
exposes no reply/thread affordance at all — it must never ship a reply control that does nothing. The
core-loop contract **freezes that choice to the REPLY-BUILT branch** (identity `CONDITIONAL_REPLY_COHERENT`):
this screen designs the full read-and-reply experience, in full, coherently, with every applicable state —
and the reply remains **non-gating** (an honest, recorded cut if the first release defers it), on the one
hard condition that nothing half-built ever ships.

##### What this screen owns, and what it defers

- It **owns** the `screens_and_flows/core_loop/chirp_thread` fragment — the thread's content/layout intent
  at wireframe level, its happy-path flow, each state in its matrix, and the `REQ-UX-047..048` register.
- It **composes** the one shared **chirp card** (visual-system §3.6; identity `SHARED_CHIRP_CARD`) for the
  thread root and every reply — the same anatomy and like/reply placement used on the timeline, profile, and
  search — rather than a thread-specific chirp layout.
- It **instantiates** the shared composer pattern (owned by `the compose design`; identity `COMPOSE_ENTRY_SEAM`)
  in **reply mode** for the reply affordance; it does **not** redefine the composer's layout, validation
  rules, character counter, or busy/success/error states. Only the parts genuinely distinct to replying —
  the binding to a root chirp (`reply_to`), the "replying to" context, the inline placement, and the
  explicitly **non-optimistic** resolution — are designed here (§3).
- It **instantiates**, per state, the shared state model (the state model §1) and the
  the per-screen state matrix; it does not redefine the state vocabulary or re-draw a state block.
- It **defers**: the reply/thread *data mechanism* (how `reply_to` threads are stored and fetched, the
  thread query/pagination shape) to domain & application design (`REQ-DOM-*`); auth/session lifetime and the
  authorization of reply/report/block to identity & trust (`REQ-ID-*`); front-end code to the code work
  (**what, not how**).
- It **flags, never invents** (§6): one real gap in the visual system (a reply glyph / reply-count
  presentation on the chirp card) is flagged for resolution rather than filled here.

---

##### 1. Reachability, placement & layout (wireframe level)

**Reached by** selecting a chirp — its reply affordance, or the card itself as a target — from any surface
that renders a chirp card: the home timeline, a profile, or search results (ia-interaction §1.4, §2.3). The
thread is a **stable, deep-linkable destination** (ia-interaction §2.1/§2.3): it survives a refresh and the
browser back button and can be opened directly, because a conversation is a thing people share links to. The
persistent signed-in (or logged-out) nav chrome and identity indicator remain (ia-interaction §1.2/§3.6);
the thread content is a content-area load that never blanks the chrome (state-patterns §1.1).

**Layout** — a single readable column (`layout.content.max` 600px, `layout.gutter`; visual-system §2.6),
top to bottom:

```
 ┌───────────────────────────────────────────────────────┐
 │  [‹ Back]   Thread                                      │   screen header: back affordance (chevron-left,
 │                                                         │   §2.7 glyph) + a quiet "Thread" label; Back
 │                                                         │   returns to the originating surface
 ├───────────────────────────────────────────────────────┤
 │  ┌─ CHIRP CARD · detail variant ──────────────────────┐│   THREAD ROOT — the chirp being viewed, rendered
 │  │ (avatar md) Display Name  @handle · 10:32·12 Mar ▸  ││   with the shared card's `detail` variant
 │  │                                            [⋯]      ││   (§3.6: larger body `font.body.lg`, fuller meta,
 │  │ The chirp body, author's line breaks preserved,     ││   absolute timestamp; overflow = report/block,
 │  │ at the detail reading size.                         ││   or delete on own chirp).
 │  │ ♥ 24     ⟳reply 3   ← reply affordance (focuses the ││   Action row: like control + count (§3.8),
 │  └─────────────────────────── composer below)         ─┘│   reply affordance + reply count (see §6 gap).
 ├───────────────────────────────────────────────────────┤
 │  ┌─ REPLY COMPOSER (signed in) ───────────────────────┐│   REPLY AFFORDANCE — the shared composer in reply
 │  │ Replying to @handle                                 ││   mode (§3). Logged-out / expired → this row is a
 │  │ [ textarea · uncontrolled, draft-safe ]             ││   "Sign in to reply" affordance instead (auth-
 │  │                                   280 ·  [ Reply ] ││   gating §3.3), never a dead or silently-failing
 │  └─────────────────────────────────────────────────── ┘│   control.
 ├───────────────────────────────────────────────────────┤
 │  Replies                                                │   REPLIES — chirps whose reply_to is the root,
 │  ┌─ CHIRP CARD · default ─────────────────────────────┐│   each rendered with the shared card's `default`
 │  │ (avatar sm) Name @h · 2m   ♥ 1   ⟳reply · 0    [⋯] ││   variant, in reading order (oldest first — a
 │  │ A reply. Selecting it opens ITS thread.             ││   conversation reads top-down, distinct from the
 │  └─────────────────────────────────────────────────── ┘│   reverse-chron timeline). A reply is itself a
 │  ┌─ CHIRP CARD · default ─────────────────────────────┐│   chirp card, so selecting it opens its own
 │  │ ...                                                 ││   thread (the reply_to chain stays coherent).
 │  └─────────────────────────────────────────────────── ┘│
 └───────────────────────────────────────────────────────┘
```

- **Thread root** uses the chirp card `detail` variant; **replies** use the `default` variant; density is
  consistent within the replies list (visual-system §3.6). No screen hand-rolls a chirp row.
- **Reading order of replies is oldest-first** (chronological) — a conversation is read top-to-bottom —
  deliberately unlike the reverse-chronological home timeline; this is the one ordering decision this screen
  makes and it is stated so the implementation does not assume the timeline's order.
- **Reply count** on a card's action row reflects how many replies a chirp has and is the affordance that
  opens that chirp's thread; its glyph and count presentation are the flagged gap in §6.
- **Like on the root and on replies is optimistic and reconciles** on failure (identity `OPTIMISTIC_RECONCILE`;
  ia-interaction §3.2; §1.8 of the state model). **The reply itself is NOT optimistic** — see §3.
- **Logged-out reading is first-class** (`REQ-PROD-013`): a logged-out visitor can open a thread and read the
  root and its replies; only the *reply* action is gated, at the point of action (§3; auth-gating §3.3).
- **Replies as a set, not an infinite feed.** The primary design renders the thread's replies as a complete,
  modest set under the root; `end_of_list` is therefore **not** in this screen's state matrix and is not
  designed here. If a thread's replies ever grow large enough to page, `end_of_list` would then apply and
  would use the shared End-of-list block (visual-system §3.14) unchanged — noted as a forward-compatible
  extension, not a silently dropped state.

---

##### 2. Happy-path flow (`content`)

1. On a chirp card anywhere in the product, the user activates the **reply affordance** (or opens the card)
   → the product navigates to **SCR-chirp_thread** with that chirp as the thread root (a stable destination).
2. **`loading`** while the root and its replies are fetched (§4.1).
3. **`content`**: the root renders as the `detail` card at the top; beneath it the reply composer (signed in)
   or a "Sign in to reply" affordance (logged-out); beneath that the replies list in oldest-first order.
4. The user reads, and may: **like** the root or any reply (optimistic, reconciled); **open an author's
   profile** from any identity cluster; **open a reply as its own thread** (its reply count / card is the
   entry); **report/block** via the card overflow (routes to `SCR-report_block`); **go Back** to the
   originating surface.
5. To converse, the signed-in user types into the reply composer → the **character counter** tracks remaining
   length → **Reply** (the screen's single `primary` action). `validation` guards empty/over-limit (§4.7).
6. On submit: **`busy_submitting`** (the Reply control shows busy, prevents a duplicate; **not** optimistic)
   → **`success`**: the posted reply **appears in the thread** (observable), a brief "Reply posted." toast
   confirms, the composer clears and the root's reply count increments. On failure: **`error`** inline at the
   composer with the **draft preserved** (§4.3).

This is the full coherent read-and-reply experience the frozen stance requires: a real thread destination,
a real reply that lands in it, and no affordance anywhere that leads nowhere (`CONDITIONAL_REPLY_COHERENT`).

---

##### 3. The reply affordance (distinct design; shared composer referenced, not redefined)

The reply composer's **field, label, character counter, inline validation, busy/success/error feedback, and
draft-safety are the shared composer's** (owned by `the compose design`; identity `COMPOSE_ENTRY_SEAM`). This
screen **instantiates that one composer in reply mode** and MUST NOT redefine its layout, its `≤280`
validation (the live product validates with `ChirpText::parse`), its counter behaviour (visual-system §3.4),
or its state set. What is **genuinely distinct about replying, and is designed here**:

- **Binding to a root chirp.** The reply is a chirp whose `reply_to` is the thread root. In the live product
  `compose.rs` builds `CreateChirpRequest { text, reply_to: None }` — the field exists but is always `None`;
  reply mode is exactly the case that populates it. (Whether/how `reply_to` is persisted and threaded is
  `REQ-DOM-*`; this screen only fixes that a reply is bound to its root.)
- **Context.** The composer shows **"Replying to @handle"** so the user is never unsure what they are
  replying to, using the shared identity cluster (visual-system §3.5).
- **Inline placement** directly under the root (not a separate full route) — the thread stays on screen while
  replying, mirroring how the product already folds compose into a view.
- **Explicitly non-optimistic resolution.** Per `OPTIMISTIC_RECONCILE`, like/follow are the *only* optimistic
  actions; **posting a reply shows `busy_submitting` → `success` (the reply appears) or `error`** — it is
  never shown as posted before the server confirms. This is the one place the distinction most tempts a
  wrong choice, so it is stated outright.
- **Gating at the point of action, never dead.** For a logged-out visitor the composer is replaced by a
  **"Sign in to reply"** affordance; for a user whose session expired mid-thread, attempting to reply routes
  through the one re-sign-in prompt (§4.6). In neither case is a reply control shown that does nothing or
  fails silently — the hard proviso of the conditional stance.

---

##### 4. The designed states

Each state instantiates the matching pattern in the state model §1 (its visual-system block + IA pattern
+ the shared copy voice, §0.4), with concrete copy intent and a recovery affordance — designed, not merely
named. Matrix for SCR-chirp_thread
(the per-screen state matrix): `[loading, empty, error, not_found, session_expired, validation,
busy_submitting, success]`, plus the `content` baseline (§1–§2).

###### 4.1 `loading`
- **When:** the thread is first fetched (root + replies), or re-fetched after a retry.
- **Copy & voice:** wordless — shape-matched skeletons, not a sentence or a spinner that replaces the chrome.
- **Recovery:** a load is never a trap — if the fetch stalls beyond a reasonable wait, offer **Try again**
  (transitions to the §4.3 error-style recovery in place) rather than an indefinite spinner.
- **Composition:** the Loading/skeleton block (visual-system §3.14) — a **detail-card skeleton** for the root
  plus a few **reply-card skeletons** on `bg.subtle`, shaped so arriving content does not jump the layout
  (state-patterns §1.1). The nav/identity chrome stays live.

###### 4.2 `empty`
- **When:** the thread loaded and the **root chirp exists, but it has no replies yet**. This is an `empty`
  (nothing to show *and the reason matters*), explicitly distinct from `not_found` (the chirp itself is gone,
  §4.4).
- **Copy & voice:** state plainly that there are no replies and offer the next action.
  *Exemplar (signed in):* "No replies yet. Be the first to reply." *(logged out):* "No replies yet.
  Sign in to be the first to reply."
- **Recovery:** the primary next action for the surface — for a signed-in reader the reply composer is right
  there (the empty block sits above an active composer); for a logged-out reader, the "Sign in to reply"
  affordance.
- **Composition:** the Empty block (visual-system §3.14) in the replies region only — the root card and the
  composer remain; never a bare blank area below the root.

###### 4.3 `error`
- **When:** the thread **fetch** failed (root/replies did not load), **or** a **reply submit** failed.
- **Copy & voice:** honest, specific, non-blaming; never a raw code, a false success, or a broken screen.
  *Exemplars:* fetch — "Something went wrong loading this thread." + *Try again*; reply submit —
  "Your reply didn't send. Your text is safe — try again."
- **Recovery:** **fetch** failure → the section **Error block with Retry** that re-fetches **in place without
  a full page reload** (`REQ-PROD-018`); **submit** failure → an **inline alert at the composer** with the
  **draft preserved** (`REQ-PROD-017`) so the user retries without retyping — not a toast that a blocking
  failure could be missed behind.
- **Composition:** Error block §3.14 (fetch) / inline alert §3.13 (submit) (state-patterns §1.3;
  ia-interaction §3.1/§3.5).

###### 4.4 `not_found`
- **When:** the **root chirp** addressed does not exist or was **deleted** (`REQ-PROD-012`/`REQ-PROD-019`) —
  e.g. opening a link to a chirp that has since been removed. Also covers a **single deleted reply** inside an
  otherwise-loaded thread.
- **Copy & voice:** say plainly that the chirp is gone; distinguish "deleted" from "never existed" where
  known. *Exemplars:* whole thread — "This chirp is unavailable." / "This chirp has been deleted."; a missing
  reply in a loaded thread — the chirp **`tombstone`** placeholder "This chirp is unavailable" (no actions).
- **Recovery:** a route back to a known place — **Home timeline** or **Back**.
- **Composition:** the Not-found block (visual-system §3.14) for a missing **root**; the chirp **tombstone**
  variant (§3.6) for a missing **reply** inside a loaded thread — the reply does not silently vanish
  (state-patterns §1.4).

###### 4.5 `session_expired` (routes through the one re-sign-in prompt — `REQ-PROD-006`)
- **When:** the user attempts an **authenticated action on the thread** (post a reply, like) **after the
  session has expired or been invalidated** (including after a service restart, which invalidates the
  in-memory sessions today).
- **Copy & voice:** a clear re-sign-in prompt — never a silent fail, never a false "reply posted."
  *Exemplar:* "Your session has expired. Sign in again to continue." + *Sign in*.
- **Recovery:** route to sign-in **preserving the user's context and the typed reply** (return-to-intent,
  ia-interaction §2.4/§3.3) so, after signing in, the user is back on this thread with their draft intact and
  can post it — the in-progress reply is never discarded without warning.
- **Composition:** the shared Session-expired prompt (visual-system §3.14) via the auth-gating pattern
  (ia-interaction §3.3); the identity indicator flips to logged-out (§3.6). This is the single product-wide
  prompt (state-patterns §1.6), not a thread-specific message.

###### 4.6 `validation`
- **When:** the reply field's input breaks the rule — **empty**, or **over the 280-character limit** — at or
  before submit. (Same rule the live composer enforces via `ChirpText::parse`.)
- **Copy & voice:** inline, at the field, actionable and specific — never a generic "invalid input" or a raw
  API error. *Exemplar:* "Your reply is over the 280-character limit."
- **Recovery:** the character counter shows the approaching/over-limit state; the **Reply** control's state is
  **honest** (the over-limit state disables submit, per the shared composer's one consistent choice), and the
  **typed reply is never lost** when submit is blocked (visual-system §3.3/§3.4; state-patterns §1.7).
- **Composition:** the shared input/counter validation states — **referenced from the composer, not
  redefined** here.

###### 4.7 `busy_submitting`
- **When:** a reply post is **in flight**.
- **Copy & voice:** minimal — the **Reply** control shows the work ("Replying…"/spinner); the point is to
  show it is working and **prevent a duplicate reply**, not to narrate.
- **Recovery:** on **failure**, return to an **editable composer with the reply text intact** and show the
  §4.3 submit error; the control never strands the user in a permanent busy state.
- **Composition:** the §3.0 busy control state (visual-system) — spinner, non-interactive, **size preserved
  so the layout does not jump** (state-patterns §1.8). **Not optimistic** — the reply is not shown in the
  thread until the server confirms (contrast like/follow, §1).

###### 4.8 `success`
- **When:** a reply **posted successfully**.
- **Copy & voice:** honest, brief, and the result is **observable** — the new reply **appears in the thread**
  and the root's reply count increments; a brief confirmation toast backs it. *Exemplar:* "Reply posted."
- **Recovery:** n/a (positive terminal). The composer clears for the next reply; **never a false success** —
  success is shown only after the reply truly landed (no optimistic reply, so nothing to reconcile for the
  reply itself; an optimistic *like* that failed would revert per §1).
- **Composition:** a toast (visual-system §3.12) plus the reply appearing in the list — the observable result
  is the confirmation of record (state-patterns §1.9; ia-interaction §3.1).

---

##### 5. Requirements & acceptance criteria — REQ-UX-047 .. REQ-UX-048

Range owned by `the chirp-thread design` (`REQ-UX-047 .. REQ-UX-048`).
Each requirement is stated from the user's side, is testable by a stranger exercising the live product,
carries a `release_gating` flag, and traces to ≥1 `REQ-PROD-*` and/or journey.

**On gating.** `SCR-chirp_thread` serves the conditional reply requirement `REQ-PROD-023`
(`release_gating: false`) and the screen is flagged non-gating in the frozen inventory, so both requirements
below are **`release_gating: false`** — an honest, recorded cut: a first release may defer the full
reply/thread experience. **The one part that is NOT optional** — that the product must never ship a *dead*
reply/thread affordance — is already carried as a **release-gating** requirement by `REQ-UX-013`
(ia-interaction); these two requirements inherit that proviso and are written so that *if* the thread ships,
this is the bar it must meet.

###### REQ-UX-047 — Open a chirp to read its thread, with honest states and no dead affordance
- **Statement:** Selecting a chirp opens a coherent detail/thread destination that shows the chirp and its
  replies, handles every honest state, is reachable and deep-linkable, and never presents a thread/reply
  affordance that leads nowhere.
- **Acceptance criteria:** From a chirp on the home timeline, a profile, or search, a person can open a
  detail view that shows that chirp as the thread root and its replies, each rendered with the same chirp
  card used elsewhere in the product; the view shows a shape-matched loading placeholder while fetching; an
  honest, recoverable error with a retry that re-fetches **without a full page reload** on a failed fetch; a
  clear "no replies yet" when the root has none (distinct from a not-found); an "unavailable / has been
  deleted" not-found with a route back when the chirp itself is gone, and an unavailable **placeholder** (not
  a silent disappearance) for a deleted reply inside a loaded thread; the thread can be reached again by its
  address after a refresh; and nowhere in the product does a reply or thread-open affordance lead to a dead
  end.
- **Release gating:** false — the thread/reply experience is a non-gating addition (`REQ-PROD-023`); recorded
  cut. Proviso: the "no dead affordance" condition is release-gating via `REQ-UX-013`.
- **Traces to:** REQ-PROD-023, REQ-PROD-018, REQ-PROD-019; J5.

###### REQ-UX-048 — Reply to a chirp coherently: validated, non-optimistic, draft-safe, never dead
- **Statement:** A signed-in user can post a reply to a chirp from its thread with honest validation, busy,
  success, and error feedback and a preserved draft; the reply is confirmed before it is shown (not
  optimistic); and a logged-out or session-expired user is prompted to sign in at the moment of replying
  rather than meeting a dead or silently-failing control.
- **Acceptance criteria:** On a chirp's thread, a signed-in user sees a reply composer bound to that chirp
  ("Replying to @handle"); entering an empty reply or one over 280 characters is prevented with a clear
  inline message at or before submit, without losing the typed text; submitting shows a busy state that
  prevents a second submission and resolves either to the reply **appearing in the thread** with a clear
  confirmation, or to an honest, recoverable error with the draft still present; the reply is shown in the
  thread **only after it is confirmed** (it does not appear optimistically and then vanish); a logged-out
  visitor sees a "Sign in to reply" affordance instead of a dead field, and a user whose session expired
  while replying is routed to sign-in with their draft preserved and returned to the thread to post it; and
  no reply control anywhere does nothing when used.
- **Release gating:** false — replies are a non-gating product requirement (`REQ-PROD-023`); recorded cut.
  Proviso: the "no dead affordance" condition is release-gating via `REQ-UX-013`.
- **Traces to:** REQ-PROD-023, REQ-PROD-016, REQ-PROD-017; J5.

###### 5.1 Traceability summary (REQ-UX → REQ-PROD / journeys)

| REQ-UX | Gating | Traces to |
|---|---|---|
| REQ-UX-047 | false | REQ-PROD-023, 018, 019; J5 |
| REQ-UX-048 | false | REQ-PROD-023, 016, 017; J5 |

Both ids are unique, fall within the owned `REQ-UX-047..048` sub-range, lie within the core-loop range
`REQ-UX-040..051`, and within the parent `REQ-UX-020..069` range.

---

##### 6. Coverage & composition notes (coverage, composition, flagged gap)

- **Frozen stance satisfied (`CONDITIONAL_REPLY_COHERENT`).** The REPLY-BUILT branch is designed in full: a
  real thread destination (§1–§2), a real reply bound to its root that lands in the thread (§3), every
  applicable state (§4), and the explicit guarantee that no reply/thread affordance is ever a dead control.
  The reply is **non-gating** and **non-optimistic**. The "deliberately-absent" branch is **not** designed —
  by the contract it is frozen out.
- **State coverage.** Every state in this screen's matrix —
  `loading, empty, error, not_found, session_expired, validation, busy_submitting, success` — is designed
  with concrete copy intent and a recovery affordance (§4), plus the `content` baseline (§1–§2). `end_of_list`
  is **not** in this screen's matrix and is not designed; the replies render as a complete set, with paging +
  the shared End-of-list block noted as a forward-compatible extension if a thread ever needs it (§1) — a
  recorded decision, not a silent drop.
- **Shared chirp card (`SHARED_CHIRP_CARD`).** The root uses the card `detail` variant, replies use `default`,
  a deleted chirp/reply uses `tombstone` — the one shared card (visual-system §3.6), no thread-specific
  variant.
- **Optimistic/reconcile (`OPTIMISTIC_RECONCILE`).** Like on the root/replies is optimistic and reconciles on
  failure; **the reply is not optimistic** (busy → success/error). Stated explicitly in §1, §3, §4.7–§4.8.
- **Compose seam (`COMPOSE_ENTRY_SEAM`).** The reply affordance **references** the one composer pattern
  (owned by `the compose design`) in reply mode and designs only the reply-distinct parts; it does not redefine
  the composer's layout, validation, counter, or states (§3). If the composer owner's final design changes
  the field/counter/validation, this screen inherits it unchanged.
- **Components consistent — one flagged gap, not invented.** The chirp card's
  **reply affordance and reply count** are the one place this screen needs more than the visual system
  currently specifies: visual-system §3.6 names a "reply affordance … only if replies are built" in the card
  anatomy, but the **required glyph inventory (§2.7) lists `compose` and no distinct reply/comment glyph**,
  and a **reply-count** presentation is not specified (only the like count, §3.8, is). Per the no-invention
  rule this is **flagged for resolution** for the visual system to resolve (add a reply/comment glyph
  and a reply-count treatment consistent with the like count), **not** invented here. Everything else on the
  card and in the composer composes existing components.

---

#### New-user onboarding / first run — SCR-onboarding

*(section: `screens_and_flows/core_loop/onboarding`)*

This fragment designs **SCR-onboarding**, the new user's first signed-in moment. Its single job is to replace
the classic new-user dead end — a brand-new account that follows no one, dropped onto a blank timeline — with
a first run that gives the user **a reason to stay and a concrete thing to do next** (`REQ-PROD-022`; journeys
J2, J6; persona P2). It is **non-gating** (the screen inventory & state vocabulary marks SCR-onboarding `gating: false`;
`REQ-PROD-022` is a tracked, non-blocking improvement): a first releasable version may ship with a
functional-but-empty first session. It is designed in full regardless — a non-gating surface is still a
designed screen, not a skeleton.

##### 0. Placement, scope & ownership

- **Where it sits (owned by IA, not redefined here).** After a successful sign-up, the first signed-in
  destination is this first run rather than the home timeline (ia-interaction §2.5; the structural placement
  decision and the sign-up→signed-in transition are IA's REQ-UX-014). On arrival the navigation chrome has
  already swapped to the signed-in set and the identity indicator shows `@handle` (ia-interaction §1.2 / §3.6).
  SCR-onboarding is a **transient first destination**, not a bookmarkable top-level route; the user leaves it
  for the home timeline and does not normally return.
- **What this fragment owns.** The *content, layout, flow, states, and testable bar* of the first-run screen
  itself — the two first-session offers (a **first follow** and a **first post**) and the observable
  transition into a now-populated timeline. `REQ-UX-049` (below) is the one requirement in this fragment's
  range (`REQ-UX-049 .. REQ-UX-049`).
- **What it defers.** It does **not** redefine the composer: the first-post offer **routes to the one composer
  surface** `SCR-compose` (identity `COMPOSE_ENTRY_SEAM`), which owns compose layout, the ≤280 counter,
  validation, busy/success/error, and draft survival. It does not redefine the follow control, the chirp card,
  the state blocks, or the nav — it **composes** them from the visual system. Session/auth mechanics are
  `REQ-ID-*`; what accounts are suggested and how they are ranked is a product/data concern (`REQ-PROD-024`
  territory) — this screen designs the *presentation and behaviour* of whatever suggestions the product can
  supply, and designs the honest fallback when it can supply none.

##### 1. Layout (the `content` baseline — wireframe level)

The loaded happy path, in the standard signed-in chrome and content column (visual-system §2.6, §3.9):

```
+--------------------------------------------------------------------+
|  [chirp]   Home   Compose   Search(*)              ( @handle ▾ )    |  signed-in nav §3.9; identity indicator §3.6
+--------------------------------------------------------------------+
|                                                                    |
|   Welcome to chirp, @handle.                                       |  title — font.display/h1
|   Your timeline is empty for now. Follow a few people and post     |  supportive copy — text.secondary
|   your first chirp to get it started.                              |
|                                                                    |
|   Find people to follow                                            |  section label — font.label
|   +--------------------------------------------------------------+ |
|   | (av) Display Name  @handle                     [ Follow ]    | |  suggested-account row =
|   |       one-line bio (truncated)                               | |  avatar §3.5 + identity cluster §3.5
|   +----------------------------- divider §3.15 ------------------+ |  + follow control §3.7
|   | (av) Display Name  @handle                     [ Following ] | |  (reconciled state after a follow)
|   |       one-line bio                                           | |
|   +------------------------------------------------------------- + |
|   | (av) Display Name  @handle                     [ Follow ]    | |
|   +--------------------------------------------------------------+ |
|                                                                    |
|   Write your first chirp                                           |  section label — font.label
|   [  Write your first chirp                                   ]    |  PRIMARY button §3.1 → opens SCR-compose
|                                                                    |  (COMPOSE_ENTRY_SEAM; the one composer)
|                    Go to your timeline                             |  tertiary/text link §3.2 — always available
+--------------------------------------------------------------------+
```

Layout intent:

- **Personalised, honest welcome.** A short title using the user's own `@handle` and one line that names *why*
  the timeline is empty (brand-new account, follows no one yet) and what the two next actions are. Plain, calm
  voice (state-patterns §0.4).
- **"Find people to follow"** — a short vertical list of **suggested-account rows**. Each row is composed from
  existing components: an **avatar** (monogram placeholder, since there are no uploads — visual-system §3.5),
  the **identity cluster** (display name + `@handle`, §3.5), an optional truncated one-line bio
  (`text.secondary`), and the **follow control** (§3.7) at the trailing edge; rows separated by the
  **divider** (§3.15). This is an *arrangement* of existing primitives (as the chirp card arranges avatar +
  identity cluster), **not** a new component — see §4.
- **"Write your first chirp"** — a single **primary** button (the screen's one primary per the one-primary
  rule, visual-system §3.1) that **opens the product's one composer** (`SCR-compose`, may be modal/inline per
  ia-interaction §2.2). This screen does not render a composer of its own.
- **"Go to your timeline"** — a persistent **tertiary/text** affordance (visual-system §3.2) so the first run
  is **never a wall**: the user can proceed to the home timeline at any time, with or without following or
  posting. This keeps the non-gating screen from ever becoming a forced step.

##### 2. Happy-path flow

1. The user completes sign-up (J2); on success they are signed in and land here (ia-interaction §2.5), chrome
   already in the signed-in state (§1.2 / §3.6).
2. The screen presents the two offers: suggested accounts to follow, and the first-post entry.
3. **First follow** — the user taps **Follow** on a suggested row. The control updates **optimistically** to
   *Following* and reconciles to the true state on settle (busy → success, or revert + error — §3.3 below).
   This is one of the two optimistic actions in the product (identity `OPTIMISTIC_RECONCILE`).
4. **First post** — the user taps **Write your first chirp**, which opens the one composer (`SCR-compose`).
   They compose and post **there**; the composer owns validation, the ≤280 counter, busy/success/error, and
   draft survival (identity `COMPOSE_ENTRY_SEAM`; `COMPOSE_DRAFT_SURVIVES` is the composer's). On return the
   new chirp exists.
5. **Into a populated timeline** — the user reaches the home timeline (via **Go to your timeline**, or offered
   as the natural next step after a first follow/post). The timeline is now **observably populated** — it
   shows the followed account's chirps and/or the user's own new chirp (rendered by the one shared chirp card,
   identity `SHARED_CHIRP_CARD`) — which is the point of the whole first run (§3.4, the `success` framing).

Either offer alone is a complete success — the user may follow without posting, or post without following, or
do both; the screen does not require both before letting the user continue.

##### 3. States

The frozen matrix for this screen is `[empty, busy_submitting, success, error]`
(the per-screen state matrix:SCR-onboarding`), plus the implied `content` baseline (§1). Each state
instantiates the one state model (the state model) with its five-part anatomy: *when it occurs · copy intent
& voice · recovery affordance · composition · instantiation*.

###### 3.0 `content` — the loaded first run
Covered by §1: welcome + suggested-account rows + first-post primary + "Go to your timeline". The transition
*into* this from the brief suggestions fetch uses the shared loading/skeleton treatment in place (a
chirp/row-shaped skeleton, state-patterns §1.1) and does not blank the chrome. (`loading` is not a required
onboarding state in the matrix and is not designed here as a first-class state — see §7 boundary note; the
chrome and the first-post offer are available immediately, independent of the suggestions fetch.)

###### 3.1 `empty` — no accounts to suggest yet (the designed first-run opportunity)
- **When it occurs.** The suggestions fetch succeeded but returned **nothing to follow** — the honest reality
  of an early or small instance where there are no (or no eligible) other accounts to recommend. This is the
  state the brief singles out: the first-run empty is the *designed opportunity*, never a bare blank.
- **Copy intent & voice.** Explain *why* it is empty and still give the user something to do — this is the
  "new-user / quiet timeline" variant of the empty model (state-patterns §1.2), not the "no search results"
  variant. Name that there is simply no one to suggest yet, and pivot to posting and proceeding. Plain and
  non-blaming (§0.4). *Exemplar:* "No one to suggest just yet — you're early. Write your first chirp, and your
  timeline will fill up as more people join and you follow them."
- **Recovery affordance.** The surface's primary next actions remain: **Write your first chirp** (primary →
  the one composer) and **Go to your timeline** (tertiary). The user is never stranded by the absence of
  suggestions.
- **Composition.** The **Empty state block** (visual-system §3.14, new-user variant): icon + title +
  supportive copy + primary action — here reusing the first-post primary as that action, with the timeline
  link beneath. The "Find people to follow" list region is replaced by this block; the first-post region
  persists.
- **Instantiation.** This is distinct from the home timeline's own empty (`SCR-home_timeline`): there the
  message is "your timeline is quiet, follow people"; here it is "there is no one to suggest yet, so post and
  proceed." Both are first-class empties, differently worded (state_conventions.empty).

###### 3.2 `busy_submitting` — a follow in flight (optimistic)
- **When it occurs.** The user taps **Follow** on a suggested row and the follow request is in flight. (Posting
  is **not** handled here — it happens in the one composer, which owns its own busy state.)
- **Copy intent & voice.** Minimal and non-narrated: the follow control shows the in-flight/busy visual and the
  label moves toward *Following*; the point is to show it is working and prevent a duplicate tap, not to
  narrate (state-patterns §1.8). Because follow is **optimistic** (identity `OPTIMISTIC_RECONCILE`,
  ia-interaction §3.2), the row may read *Following* immediately, pending confirmation.
- **Recovery affordance.** On success the row settles to *Following* (§3.3 success); on failure it **reverts
  visibly** to *Follow* with an honest message (§3.4 error) — the control never strands the user in a
  permanent busy state and the on-screen state matches reality after it settles.
- **Composition.** The **follow control busy → reverted/confirmed** visual (visual-system §3.7) with the §3.0
  busy state; size preserved so the row does not jump. Obeys honest-feedback (ia-interaction §3.1).
- **Instantiation.** The follow control is the only submitting control on this screen; the first-post primary
  hands off to the composer rather than submitting here. Multiple rows may each follow independently; each
  reflects its own true state.

###### 3.3 `success` — the first run paid off, observably
- **When it occurs.** A follow completed (the row shows *Following*, count/state reconciled to truth), and/or a
  first chirp was posted in the composer, and the user reaches a **now-populated home timeline**.
- **Copy intent & voice.** Honest and brief, and above all **observable** (state-patterns §1.9): the designed
  confirmation of a first run is not a congratulatory banner but the **result appearing** — the followed
  account now reads *Following*, and the home timeline the user lands on now shows real content (that account's
  chirps and/or the user's own new chirp). A brief **toast** may confirm a follow ("Following @handle");
  never a false success.
- **Recovery affordance.** n/a (positive terminal) — but any optimistic follow shown as succeeded that in fact
  failed **reverts and surfaces the error** (ia-interaction §3.2), so success is never faked.
- **Composition.** The reconciled **follow control** (§3.7) for the follow; a **toast** (§3.12) for the brief
  confirmation; and, on the destination timeline, the **one shared chirp card** (§3.6, identity
  `SHARED_CHIRP_CARD`) rendering the now-present content. The first run's success is defined by that
  observable populated timeline, not by a self-congratulatory onboarding screen.
- **Instantiation.** This fragment designs the *observability* of the outcome (the user can see their action
  took effect) and hands rendering of the populated timeline to `SCR-home_timeline`; it does not re-draw the
  timeline here.

###### 3.4 `error`
Two honest sub-cases, each per the error model (state-patterns §1.3; recover in place, never lose work):
- **(a) Suggestions failed to load (section fetch error).** The "Find people to follow" region shows the
  **Error state block with Retry** (visual-system §3.14) that **refetches in place without a full page reload**
  (`REQ-PROD-018`). *Exemplar:* "Couldn't load suggestions. Your account is all set — try again, or head to
  your timeline." The first-post primary and the timeline link remain available, so a suggestions failure is
  never a dead end on the first run.
- **(b) A follow failed (action error).** The optimistic row **reverts to the true state** (*Follow*, true
  count) and surfaces an honest, non-blocking message — a **toast with Retry** (§3.12) or an inline note — per
  the optimistic reconcile rule (ia-interaction §3.2; state-patterns §1.8). *Exemplar:* "Couldn't follow
  @handle — try again." No partial/incorrect state is left on screen.
- **First-post errors are the composer's** (`SCR-compose`): a failed send keeps the draft and shows the
  composer's inline error (`COMPOSE_DRAFT_SURVIVES`); this screen does not duplicate that treatment.
- **Voice.** Plain, specific, non-blaming; never a raw error code or a false success (§0.4).

##### 4. Component composition check (maps cleanly onto the visual system — no gap)

| Element on this screen | Rendered by (visual-system) |
|---|---|
| Signed-in chrome + identity indicator | Navigation §3.9; identity cluster §3.5 (indicator per ia §3.6) |
| Welcome title + copy | Type scale §2.2 (`font.display`/`h1`, `text.secondary`) |
| Suggested-account row | **Composition** of avatar §3.5 + identity cluster §3.5 + follow control §3.7 + divider §3.15 — an arrangement of existing primitives (as the chirp card arranges them), **not** a new component |
| First-post primary CTA | Button §3.1 (`primary`, one per screen) → opens `SCR-compose` |
| "Go to your timeline" | Link / tertiary button §3.2 / §3.1 |
| `empty` (no suggestions) | Empty state block §3.14 (new-user variant) |
| `busy_submitting` (follow) | Follow control busy §3.7 + §3.0 busy state |
| `success` (follow / populated timeline) | Toast §3.12; reconciled follow control §3.7; shared chirp card §3.6 on the destination |
| `error` (a) suggestions fetch | Error state block + Retry §3.14 |
| `error` (b) follow action | Toast with Retry §3.12 (inline alert §3.13 if persistent) |

No element on this screen requires a component the visual system lacks, so **none is invented**. The suggested-account row is explicitly a composition of existing components; if the visual system judges it warrants a first-class "account row" component, that is a
visual-system addition to flag, **not** an invention made here. No gap is flagged.

##### 5. Cross-screen seam compliance (core-loop seams)

- **`COMPOSE_ENTRY_SEAM`** — the first-post offer **routes to the one composer** `SCR-compose`; this fragment
  references it and does not redefine composer layout, validation, or states. ✔
- **`OPTIMISTIC_RECONCILE`** — **follow** here is optimistic and reconciles to the true state on failure;
  **posting is not optimistic** (it runs through the composer's busy → success/error). ✔
- **`SHARED_CHIRP_CARD`** — any chirp shown as a result of the first run (on the destination timeline) is the
  one shared chirp card (§3.6); this screen renders no bespoke chirp row. ✔
- **Screen coverage & states designed** — SCR-onboarding appears in exactly this
  fragment and every state in its matrix (`empty`, `busy_submitting`, `success`, `error`) plus `content` is
  **designed** with copy intent and a recovery affordance (§1, §3). ✔

##### 6. Requirement — REQ-UX-049

Within the owned range `REQ-UX-049 .. REQ-UX-049` (the onboarding design),
inside the parent `REQ-UX-040..051` and the grandparent `REQ-UX-020..069`. Stated from the user's side,
testable against the live product by a stranger, release-gating-flagged, traced to ≥1 `REQ-PROD-*`/journey.
This requirement owns the first-run **screen's content and behaviour**; it complements, and does not redefine,
IA's `REQ-UX-014` (which owns the *placement* decision that a new user lands on a guided first run).

###### REQ-UX-049 — The new-user first run offers a concrete next action and makes its payoff observable
- **Statement:** Immediately after sign-up, the first-run screen offers the user at least one concrete next
  action — following a suggested account and/or writing a first chirp — and, once an action is taken, lets the
  user reach a home timeline that observably reflects it, instead of leaving them on a blank timeline; when
  there is nothing to suggest, the screen still explains why and offers a first post and a way to the timeline.
- **Acceptance criteria:** A stranger who completes sign-up and reaches the first run can observe all of:
  1. the screen presents **(a)** at least one suggested account to follow with a working **Follow** control
     whose label and the account's state reflect the true result after the action (optimistic, reconciling on
     failure), **and/or (b)** a prompt to write a first chirp that **opens the product's one composer**
     (not a second composer defined on this screen);
  2. after following a suggested account and/or posting a first chirp, the user can reach the home timeline
     and see content that reflects the action (the followed account's chirps and/or their own new chirp);
  3. when there are **no accounts to suggest**, the screen shows an honest message saying so and still offers
     writing a first chirp and continuing to the timeline — **not** a bare blank area;
  4. a failure to load suggestions shows an honest message with a **retry in place** (no full-page reload),
     and a failed follow **reverts** to the true state with an honest message — no false success and no stuck
     busy control;
  5. the user can proceed to the home timeline at any time without being forced to follow or post first.
- **Release gating:** **false** — SCR-onboarding is non-gating (the screen inventory & state vocabulary `gating: false`;
  `REQ-PROD-022` is explicitly a tracked, non-blocking improvement: a first releasable version may ship with a
  functional-but-empty first session). Stated explicitly as an honest, recorded non-gating cut.
- **Traces to:** `REQ-PROD-022` (a new user's first session is not an empty dead-end); journeys **J2**
  (sign-up → new user's first signed-in moment) and **J6** (second-visit / returning use); persona P2.
  Complements `REQ-UX-014` (IA — first-run placement) and reuses `REQ-UX-022` (empty states explain why and
  offer the next action) and the state-model requirements `REQ-UX-028`/`REQ-UX-029` (busy/reconcile; honest,
  observable success).

##### 7. Coverage, composition & boundaries

- **Coverage.** Every state in the per-screen state matrix:SCR-onboarding` — `empty`, `busy_submitting`,
  `success`, `error` — is designed with concrete copy intent and a recovery affordance, plus the `content`
  baseline (§1, §3). None is left skeletal; none is dropped.
- **States deliberately not designed as first-class here (recorded, not silently omitted):**
  - **`loading`** is **not** in this screen's matrix. The brief suggestions fetch uses the shared in-place
    skeleton (state-patterns §1.1) within the content region, but the chrome and the first-post offer are
    available immediately; loading is not a first-class onboarding state. (Adding it would be permissible per
    the per-screen state matrix, but it is not required and is left to the shared pattern.)
  - **`session_expired`** is **not** in this screen's matrix, because the first run begins on a just-established
    session. If a user lingers until the session expires and then attempts a follow/post, that authenticated
    action routes through the **one shared re-sign-in prompt** (state-patterns §1.6, `REQ-PROD-006`), owned by
    the state model and the entry/auth surface — not re-designed here.
- **Deferred (out of UX scope).** Session/auth lifetime and invalidation (`REQ-ID-*`); which accounts are
  suggested and their ranking/selection (a product/data concern, `REQ-PROD-024` territory) — this screen
  designs the presentation and behaviour of whatever suggestions the product supplies, and the honest fallback
  when it supplies none. The composer's internals (`SCR-compose`), the home timeline's rendering
  (`SCR-home_timeline`), and the sign-up flow (`SCR-signup`) are their owners'.
- **A missing component is flagged, never invented.** None was needed (§4). The suggested-account row is a
  composition of existing visual-system primitives; promoting it to a first-class component is a visual-system
  decision for resolution, not an invention here.

---

### SCR-search_discovery — Search / discovery

#### 1. What this screen is

The one surface for **finding accounts and content beyond the links already on screen** — the gap the
product has today, where a visitor or signed-in user can only follow links that happen to be present
(REQ-PROD-024; journeys J1 first visit, J5 core loops, J6 daily/returning use). A user types a query,
runs it, and gets matching **accounts** and **chirps**; from the results they can open a profile, open
a chirp/thread, or follow an account.

- **Product-requirement served:** REQ-PROD-024 (finding people and content). **Non-gating** — the first
  releasable version may ship with on-screen-links discovery only; this surface is a recorded near-term
  addition, not a release blocker. Everything below is **designed** regardless (a non-gating surface is
  still designed, not skeletal).
- **Starting point:** net-new. The current SPA has **no** search, directory, or suggestions surface
  (established in the product requirements and frozen in the inherited seams). There is no existing
  behaviour to preserve — this is designed from zero, honestly.
- **Who can use it:** reading never requires an account (REQ-PROD-013), so **both logged-out and
  signed-in** users can search and view results. The only account-gated action on this surface is
  *following* an account result (see §5.1).

#### 2. Where it sits (builds on the IA model — does not redefine it)

- **Route:** `/search` — a top-level, bookmarkable, deep-linkable destination (per the IA screen-map and
  routing conventions). A query SHOULD be reflected in the URL (e.g. `/search?q=…&type=…`) so a result
  view survives refresh and can be shared; the exact param shape is the responsive/routing concern, not
  redefined here.
- **Entry point:** the **`Search` nav item** in both the logged-out chrome and the signed-in primary nav,
  present only where this non-gating surface ships (the entry point and its non-gating flag are already
  specified by **REQ-UX-008**, owned by the IA & interaction section — this screen *is reached through*
  that entry; it does not restate it). No dead `Search` entry appears if the surface is not shipped.
- **Chrome:** the persistent nav/chrome stays in place; this screen renders in the content region. Search
  work never blanks the already-loaded chrome (loading convention).

#### 3. Layout (wireframe level)

Single column within the content region; the search field is the fixed head of the surface, results
scroll beneath it. Composed entirely from existing visual-system components — no new component invented.

```
┌───────────────────────────────────────────────────────────┐
│  (persistent nav / chrome — Search active)                  │
├───────────────────────────────────────────────────────────┤
│  Search                                                     │  ← surface title (font.h2)
│  ┌───────────────────────────────────────────────┐         │
│  │ 🔍  Search chirp…                           ✕  │         │  ← search input (§3.3 search variant:
│  └───────────────────────────────────────────────┘         │     leading search icon + clear control)
│  ‹ inline message slot: validation / helper ›               │  ← §3.3 message slot (validation state)
│                                                             │
│  [ Accounts │ Chirps ]                                      │  ← segmented control (§3.15 tabs);
│                                                             │     per-tab result count when known
│  ┌─ results region ────────────────────────────────────┐   │
│  │  (state-dependent: resting / loading / content /     │   │
│  │   no-results empty / error / end-of-list — §5)       │   │
│  └──────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────┘
```

- **Search field** — the §3.3 *search input* variant: leading `search` glyph, a **clear (`✕`)** control
  that empties the field and returns to the resting state. Placeholder `Search chirp…` is never the only
  label; an accessible label (`Search`) is always present. The field is the focus on arrival.
- **Result-type segmented control** — `[ Accounts │ Chirps ]` (§3.15). Each tab holds its **own** result
  list and its **own** state, so a query can match accounts but not chirps (or vice-versa) and each tab
  states its own outcome honestly. Default tab: **Accounts** (discovery is primarily "find people to
  follow", J6). A tab MAY show a result count badge (§3.15 count pill) once a search completes.
- **Results region** — renders the per-tab state via the shared blocks/components in §5.

#### 4. Happy path (the `content` state)

1. User opens `/search` from the `Search` nav item → **resting state** (§5.0): field focused, empty,
   with guidance.
2. User types a query (≥ the minimum length) and submits (Enter) or the search runs as they type
   (debounced); the active tab determines what is searched first.
3. Surface shows the **running-search indicator** (§5.1 `loading`) — shape-matched skeletons, never a
   blank gap.
4. Results arrive and render in the active tab:
   - **Accounts tab:** a list of **account result rows** (§5, each = avatar + identity cluster + a
     **follow control**). Activating a row opens that `SCR-profile`.
   - **Chirps tab:** a list of **chirp results**, each rendered by the **one shared chirp card**
     (`SHARED_CHIRP_CARD`, §3.6 `default`) — identical anatomy to the timeline/profile/thread; activating
     a card opens `SCR-chirp_thread` (where that conditional surface is built) or the author profile.
5. Switching tabs re-renders the other tab's results (or its own loading/empty/error state) for the same
   query without re-typing.
6. Long result lists page ("load more" / infinite scroll per the IA paging pattern); reaching the end
   shows **end-of-list** (§5.4).

**Result item anatomy (reuse only — nothing hand-rolled):**

| Result kind | Rendered by | Opens | Account-gated action |
|---|---|---|---|
| Account | Avatar (§3.5) + identity cluster (display name + `@handle`) + **Follow control (§3.7)** | `SCR-profile` | Follow/Unfollow |
| Chirp | **Shared chirp card** (§3.6 `default`) — avatar, identity, timestamp, body, like control | `SCR-chirp_thread` / author `SCR-profile` | Like / reply (inherited from the card) |

> The follow control on an account result and the like/reply controls on a chirp card are the **shared
> components**; their optimistic-update-and-reconcile and auth behaviour (logged-out → route to sign-in;
> session-expired → re-sign-in prompt) are owned by those shared controls and the home-timeline/profile
> seam, and are **referenced, not redefined** here. `session_expired` is therefore not an owned state of
> this (public) surface — it can only arise *inside* a shared action control and is handled there.

#### 5. States (each designed: copy intent + recovery)

The required state matrix for this screen is `[loading, empty, error, end_of_list, validation]`
(the per-screen state matrix). `content` (§4) is the loaded happy path. The **resting (pre-query)** state (§5.0)
is designed in addition, as the brief requires. Each state instantiates the shared state model and the
shared copy/recovery conventions; copy below is **intent with concrete committed text**, not final
microcopy.

##### 5.0 Resting — pre-query (designed in addition to the matrix)

The default on arrival at `/search`, before any query. **Not** a blank page and **not** a "no-results"
message (nothing has been searched yet).

- **Intent:** invite a query and say plainly what can be found here. Rendered as an **Empty block**
  (§3.14) variant tuned for "ready to search", distinct in copy from the no-results empty (§5.3).
- **Copy (intent):** title *"Find people and chirps"*; body *"Search for an account by name or @handle,
  or search chirps for a word or phrase."*
- **Optional discovery affordance:** where a lightweight *suggested accounts / who-to-follow* capability
  exists, the resting state MAY show a short **"Suggestions"** list of account result rows so the surface
  does something useful before the first keystroke (supports J6 "discover more to follow"). Suggestions
  require a data capability that is a product/domain concern; if it is not available, the resting state
  shows the prompt alone — **honestly**, with no empty "Suggestions" shell. *(This is the only part of
  the surface that depends on a capability beyond plain query search; it is explicitly optional and
  non-gating.)*
- **Recovery:** n/a (entry state); the field is focused so the next action is obvious.

##### 5.1 `loading` — a search is running

- **Intent:** show that the search is **in progress** without blanking the chrome or the field. Rendered
  as the **Loading/skeleton block** (§3.14): shape-matched skeleton rows that match the active tab's
  result shape (account-row skeletons on the Accounts tab, chirp-card skeletons on the Chirps tab) on
  `bg.subtle`, so there is **no layout jump** when results arrive. MAY echo the query ("Searching for
  '…'") above the skeletons.
- **This is visibly distinct from no-results (§5.3):** the running state is animated placeholders /
  in-progress indication; it carries **no** "nothing found" message. The two are never conflated — a
  user can always tell "still working" from "finished, found nothing".
- **Recovery:** if the search exceeds a reasonable wait, offer a **Retry** (and fall through to the error
  state §5.2 on failure); never leave an indefinite spinner with no exit.

##### 5.2 `error` — the search failed

- **Intent:** honest, specific, non-blaming — the search could not be completed and it is recoverable.
  Rendered as the **Error block** (§3.14) in the results region (or an inline alert §3.13 for a
  tab-local failure). Copy (intent): *"Couldn't run your search. Something went wrong on our end."* Never
  a raw error code, never a blank region, never a false "no results".
- **Recovery:** a **Retry** that re-runs the **same** query **without a full page reload**; the typed
  query is **preserved** in the field. If one tab fails while the other succeeds, the failure is scoped
  to the failing tab.

##### 5.3 `empty` — searched, nothing matched (no-results)

- **Intent:** the search **completed** and returned **zero** matches for this query in this tab. Rendered
  as the **Empty block** (§3.14) *search-no-results* variant, with copy clearly different from the
  resting state (§5.0) and from a running search (§5.1). Copy (intent): title *"No results for '…'"*
  (naming the actual query); body *"Check the spelling, try different or fewer words"* — and, for the
  Accounts tab, *"…or search by exact @handle."*
- **Recovery (a real next action, never a dead end):** offer the constructive next steps —
  **(a)** revise the query (the field stays filled and focused for editing); **(b)** **switch result
  type** when the *other* tab has matches ("No accounts — see N chirps instead →"); **(c)** where
  suggestions exist (§5.0), a **"Browse suggestions"** link back to discoverable accounts. At least one
  concrete next action is always present.

##### 5.4 `end_of_list` — end of a paged result list

- **Intent:** make the end of a **non-empty** paged result list explicit, distinct from no-results
  (§5.3): results were found and the user has now seen all of them. Rendered as the **End-of-list block**
  (§3.14) — a subtle terminal marker. Copy (intent): *"That's everything for '…'."*
- **Recovery:** offer a sensible onward action — **broaden the search** (fewer/different words) or
  **switch result type** to the other tab. Distinct, by copy and treatment, from both empty and loading.

##### 5.5 `validation` — the query itself is invalid

- **Intent:** inline, actionable, specific — shown **at the field** (§3.3 message slot), before a dead
  search is run. Two cases:
  - **Empty query:** submitting with an empty field does **not** run a search; copy (intent): *"Type
    something to search for."* The run control reflects this honestly (disabled while empty).
  - **Too-short query:** a query under the minimum length (e.g. fewer than 2 characters) shows copy
    (intent): *"Enter at least 2 characters to search."* and does not run until satisfied.
  Messages are specific and point at the fix — never a generic "invalid input", never a raw API error.
- **Recovery:** the message points at exactly what to change; the submit/run control's state stays honest
  (disabled vs error) and the typed text is never cleared out from under the user.

#### 6. Components used (all exist — no gaps to flag)

| Need | Visual-system component | Notes |
|---|---|---|
| Query entry | Text input §3.3 — **search input** variant | leading search icon + clear control; message slot for validation |
| Result-type switch | Tabs / segmented control §3.15 | `[ Accounts │ Chirps ]`; optional count pill |
| Account result | Avatar §3.5 + identity cluster §3.5 + Follow control §3.7 | shared controls; auth/reconcile owned upstream |
| Chirp result | **Chirp card §3.6 `default`** (`SHARED_CHIRP_CARD`) | identical to timeline/profile/thread; no bespoke row |
| loading / empty / error / end-of-list | State blocks §3.14 | shape-matched skeletons; no-results, error-with-retry, end-of-list variants |
| validation | §3.3 input error/helper states | inline at the field |
| transient confirmations | Toast / inline alert §3.12 / §3.13 | e.g. follow failure reported by the shared control |

No component required by this screen is missing from the visual system; nothing is invented. (Were one
missing, it would be flagged for resolution, not hand-rolled.)

#### 7. Requirements (REQ-UX-050 .. REQ-UX-051)

Both inherit REQ-PROD-024's non-gating flag; both are user-side and testable against the live product by
someone who never met the author.

##### REQ-UX-050 — A dedicated surface to find accounts and content
- **Statement:** A user (logged-out or signed-in) can open a dedicated search surface and find accounts
  and chirps beyond the links already on screen, by entering a query and seeing matching results.
- **Acceptance criteria:** From the navigation `Search` entry a user reaches a surface with a labelled
  search field. Entering a query of at least the minimum length and running it returns matching
  **accounts** and **chirps**, switchable via a result-type control. Account results show the account's
  display name, `@handle`, and avatar together with a follow control; chirp results are rendered by the
  **same chirp card** used on the timeline and profile — no screen shows a visibly different chirp layout.
  Activating an account result opens its profile; activating a chirp result opens that chirp/author.
  *(Check: open Search, type a known `@handle` → that account appears with a Follow control and opens its
  profile; type a word known to be in a chirp, switch to Chirps → that chirp appears as a standard chirp
  card.)*
- **Release gating:** false — inherits REQ-PROD-024; discovery is a recorded near-term addition, not a
  release blocker.
- **Traces to:** REQ-PROD-024; J1, J5, J6.

##### REQ-UX-051 — Honest search states; a running search is distinct from no-results
- **Statement:** The search surface distinguishes a search that is still running from a search that
  completed with no matches, validates the query before running it, makes the end of a result list
  explicit, and offers a constructive next step in every non-content state rather than a blank area, a
  bare spinner, or a dead end.
- **Acceptance criteria:** An empty or too-short query shows a specific inline message at the field and
  does **not** run a dead search. While a search runs, the surface shows a clearly in-progress indicator
  (shape-matched skeletons/placeholders), not a static blank. When a search completes with **zero**
  matches, the surface shows a distinct no-results message that **names the query** and offers at least
  one next action (revise the query, switch result type, or browse suggestions) — visibly different from
  the running indicator. A failed search shows an honest error with a **Retry** that does not reload the
  whole page and preserves the typed query. Reaching the end of a **non-empty** paged list shows an
  explicit end-of-list marker distinct from no-results. On arrival with no query, a resting state explains
  what can be searched instead of showing a blank page. *(Check: submit empty → field message, no search;
  submit `1` char → min-length message; submit a nonsense string → "No results for '…'" with a next
  action, not a spinner; force a fetch failure → error with Retry and the query still in the field.)*
- **Release gating:** false — inherits REQ-PROD-024.
- **Traces to:** REQ-PROD-024; J1, J5, J6 (instantiates the shared `loading`/`empty`/`error`/
  `end_of_list`/`validation` copy & recovery conventions).

#### 8. Scope notes & assumptions

- **State coverage:** every state in this screen's matrix `[loading, empty, error, end_of_list,
  validation]` is designed with copy intent + a recovery affordance; `content` (§4) and the pre-query
  **resting** state (§5.0) are designed in addition. The crucial **no-results `empty` vs running
  `loading`** distinction is called out explicitly (§5.1 vs §5.3). No state is dropped.
- **`session_expired` deliberately not owned here:** this surface is public; the only account-gated
  action is following an account result, which uses the **shared follow control** whose logged-out /
  session-expired behaviour is owned upstream. Recorded here so the omission is a decision, not a gap.
- **Suggestions / who-to-follow (§5.0) is optional and capability-dependent:** plain query search is the
  core, releasable-shape design; a suggestions list depends on a product/domain capability and is
  explicitly optional and non-gating. Recorded so the dependency is tracked, not assumed.
- **Live-product starting point:** no search exists in the current product; this is net-new, consistent
  with the frozen screen inventory and the product requirements. No existing search behaviour was
  preserved because none exists.
- **What vs how:** this fragment specifies the surface, flow, states, copy intent, and the requirement
  bar only (wireframe-level + committed text). URL param shape, debounce timing, paging mechanism, and
  the search backend are implementation / responsive concerns deferred to their owners.

---

#### Account lifecycle & settings — screens & flows

*(section: `screens_and_flows`, fragment: `account_settings`)*

This fragment designs the surfaces through which a chirp account **lives, changes, and ends, and through which
the user leaves with their data**: the settings hub, profile editing, account deletion, data export, and
sign-out. These are the consequential, routinely-forgotten surfaces — today `Nousix-LLC/nswe-demo` @ `c805525`
has **none** of them in the SPA: no settings, no way to edit a profile in-product, no account deletion (no UI and
no API route), no data export, and no visible sign-out control. Absence here is the exact defect this work exists
to prevent, so each screen below is designed to the bar of *designed screens, not a skeleton*: its content and
layout intent at wireframe level, its happy-path flow, and **each state that applies to it** (per
the per-screen state matrix) with concrete copy intent and a recovery affordance.

**What this fragment settles, and what it defers.**

- It **owns** the screen/flow/state design of the five screens above and the `REQ-UX-052..061` register for them.
- It **instantiates** the one shared state model in `the state model (empty, error & loading states)` (§1.0–§1.9)
  per screen — it does not re-draw a state; it names the model's pattern and writes the per-screen copy.
- It **composes from** named `the visual language & design system` components and places each screen in the
  `the information architecture & interaction model` navigation model. It invents no component; the component set covers these screens
  (composition check, §6 — no gap found/flagged).
- It **defers the mechanism**: deletion semantics & authorization and session invalidation are `REQ-ID-*`; the
  export **file format** is `REQ-DOM-*`/`REQ-ID-*`; persistence is `REQ-DOM-*`/`REQ-OPS-*`. This fragment designs
  only what the user sees and can do, and the testable bar (**what, not how**).

**Design posture for this area (the throughline).** Two of these five flows are **consequential or destructive**
(deletion, export) and one is a **state transition** (sign-out). The area's throughline, taken from
`the information architecture & interaction model` §3.4 and the state model §2, is: **be honest about consequences and reversibility,
confirm the irreversible with its consequences stated, never be optimistic about a destructive action, never
show success for something that did not happen, and never silently drop the session.**

---

##### 1. SCR-settings_home — the account-management hub

**Role & placement.** The single reachable gather-point for everything about the user's own account. Reached from
the signed-in **account menu → Settings** and from the own-profile route (`the information architecture & interaction model` §1.2, §2:
`/settings` top-level hub with nested sub-pages). It closes the product-requirements gap REQ-PROD-009 ("a home for
managing the account") and makes nothing in the account lifecycle require the raw API.

**Content & layout intent (wireframe level).** Single readable column (`layout.content.max` 600px).

- **Title** "Settings" (`font.h1`).
- **Identity cluster** at the top (`visual-system` §3.5): avatar (monogram placeholder when no image) + display
  name (`font.label`) + `@handle` (`font.meta`), so the user sees *which* account they are managing — this doubles
  as the signed-in-state indicator (`the information architecture & interaction model` §3.6).
- **Account group** — a list of **settings rows** (`visual-system` §3.15: label + short description + trailing
  chevron), each routing to its nested sub-page:
  - *Edit profile* → `SCR-edit_profile` ("Change your display name and bio")
  - *Data export* → `SCR-data_export` ("Download your chirps, profile, and follows")
  - *Account* group, visually separated by a divider, holding the **destructive** item:
  - *Delete account* → `SCR-account_deletion` — row styled with destructive intent (`error.ink` label/icon,
    `visual-system` §3.10/§3.15), but it only *navigates*; the actual confirmation lives on the deletion screen.
- **Session group** — *Sign out* → triggers `SCR-signout` (mirrors the account-menu control; consistent placement,
  REQ-UX-059 / `the information architecture & interaction model` §3.6).
- **Legal** — *Privacy & terms* → `SCR-privacy_terms` (low-prominence link/row; REQ-UX-061, REQ-PROD-029).
- **One primary action**: none is "primary" on a hub — rows are navigational; the destructive item is visually
  distinct, not primary (`visual-system` §5.4/§5.5).

**Happy-path flow.** Account menu → *Settings* → the hub lists the account surfaces → the user picks one → routes
to the nested sub-page → Back returns to the hub. No surface in the account lifecycle is reachable only via the API
(satisfies REQ-PROD-009; builds on `the information architecture & interaction model` REQ-UX-004).

**Applicable states** (matrix: `loading, error, session_expired`; `content` implied):

| State | Design (copy intent + recovery) · instantiates state-patterns |
|---|---|
| `content` | The settings list above, rendered by the settings-row/list component (`visual-system` §3.15). Baseline (§1.0). |
| `loading` | If the hub fetches the current-account summary for the identity cluster, show a **row skeleton** (state-patterns §1.1; `visual-system` §3.14) — the nav/chrome is not blanked. Wordless; a stalled load offers *Try again* rather than an indefinite spinner. |
| `error` | If that summary fetch fails, show the **Error state block + Retry** that refetches in place without a full page reload (state-patterns §1.3; `visual-system` §3.14), copy e.g. "Couldn't load your settings." The navigable rows that need no fetch remain usable where possible; the chrome is kept. |
| `session_expired` | Opening Settings after the session has expired routes through the **one re-sign-in prompt** (state-patterns §1.6; `the information architecture & interaction model` §3.3) — "Your session has expired. Sign in again to continue." + *Sign in* — never a blank hub or a false render; returns to Settings on success (REQ-UX-060, REQ-PROD-006). |

*Out of matrix, recorded:* no `empty`/`not_found`/`end_of_list`/`validation`/`busy_submitting`/`success` — the hub
is a static navigational surface; its actions' busy/success/validation belong to the sub-pages it routes to.

---

##### 2. SCR-edit_profile — edit display name & bio

**Role & placement.** Edit the mutable basics of one's own account. Reached from the own-profile *Edit profile*
affordance and from `SCR-settings_home` (`the information architecture & interaction model` §1.2, §2 `/settings/profile` nested; own-vs-other
profile distinction, REQ-UX-006). Satisfies REQ-PROD-008.

**Content & layout intent.** Single column; a short form (`visual-system` §3.3 field group):

- **Handle** shown **read-only** (`font.body`, `text.secondary`), with a plain note "Handles can't be changed."
  — the handle is identity and is immutable here; deferring any handle-change capability (not a product
  requirement) rather than implying one.
- **Display name** — single-line input, pre-filled with the current value; label `font.label`; required.
- **Bio** — `textarea`, pre-filled; optional; a **character counter** (`visual-system` §3.4) if a bio length limit
  applies (the limit value is a domain concern, `REQ-DOM-*`; the counter + over-limit behaviour is designed here).
- **Actions**: **Save** (`primary` — the one primary action, `visual-system` §3.1) and **Cancel** (`secondary`,
  returns without saving). If the form is dirty, Cancel surfaces a light "Discard your changes?" confirm
  (`visual-system` §3.11) so an accidental Cancel does not silently drop edits (state model principle 4,
  input-preserving).

**Happy-path flow.** Own profile / Settings → *Edit profile* → form pre-filled with current display name & bio →
edit → **Save** → busy → **success** (toast "Profile saved", `visual-system` §3.12) → the new values are shown on
the profile and **persist on a later visit** (REQ-PROD-008; the persistence mechanism is `REQ-DOM-*`).

**Applicable states** (matrix: `loading, validation, busy_submitting, error, success, session_expired`):

| State | Design (copy intent + recovery) · instantiates state-patterns |
|---|---|
| `content` | The pre-filled form (baseline, §1.0). |
| `loading` | While fetching the current profile to pre-fill, show a **form skeleton** (field-shaped placeholders, state-patterns §1.1) — not a blank form that then jumps when values arrive. |
| `validation` | **Inline, field-level** (state-patterns §1.7; `visual-system` §3.3/§3.4): display name required ("Add a display name") and within its limit; bio within its limit (counter warns approaching, blocks over-limit). Messages say the fix, not "invalid input"; the **Save** control's state is honest (disabled while invalid *or* errored on attempted submit — one consistent choice); **typed text is never lost** (REQ-PROD-017). |
| `busy_submitting` | **Save** shows the §3.0 busy control (spinner, size preserved, non-interactive) and prevents a duplicate save (state-patterns §1.8). Profile edit is **never optimistic** (not a like/follow) — the saved values appear only on real success. |
| `error` | A failed save shows an **inline alert at the form** (`visual-system` §3.13) — "Couldn't save your changes. Your edits are still here — try again." — with **the edits intact** and a retry, no full page reload (state-patterns §1.3; REQ-PROD-017/018). |
| `success` | Honest confirmation (toast "Profile saved") and the result is **observable**: the profile shows the new display name/bio (state-patterns §1.9). Never shown unless the save truly succeeded. |
| `session_expired` | Saving/loading after expiry routes through the **one re-sign-in prompt**, **preserving the in-progress edit** so the user resumes the save afterward (state-patterns §1.6; REQ-UX-060, REQ-PROD-006) — never a silent fail, never a false "saved". |

---

##### 3. SCR-account_deletion — delete the account, consequences stated

**Role & placement.** The self-service deletion flow. Reached from `SCR-settings_home` → *Delete account*
(`the information architecture & interaction model` §2 `/settings/delete` nested). Satisfies REQ-PROD-010, and its post-state ties to
REQ-PROD-012. **Deletion semantics and authorization are `REQ-ID-*`; what the user sees, the confirmation, and the
consequence copy are the point of this screen** (brief).

**Content & layout intent.** A deletion surface plus a destructive **confirm dialog** (`visual-system` §3.11):

- **Explanatory section** (plain, calm): "Deleting your account can't be undone." then a clear statement of
  **what is removed** — the standing consequence copy (REQ-PROD-010 / `the information architecture & interaction model` §3.4):
  > "Deleting your account permanently removes **your account, your chirps, your follows, and your likes**. This
  > can't be undone."
- **An explicit confirmation gate** so deletion cannot be a single mis-tap: an acknowledgement the user must
  perform — e.g. **type your handle to confirm** (text input, `visual-system` §3.3) or an explicit "I understand
  this is permanent" checkbox — before the destructive control enables.
- **Actions in the confirm dialog**: **Cancel** (`secondary`, and the **safe default focus**) + **Delete account**
  (`destructive` button, `error.base`/`text.inverse`, `visual-system` §3.1/§3.11) — the destructive control is
  visually distinct and is **not** the default-focused control (`visual-system` §5.5; REQ-UX-076).

**Happy-path flow.** Settings → *Delete account* → read the consequences → perform the explicit confirmation
(type handle / acknowledge) → **Delete account** → busy → **success**: the account is deleted, the user is **signed
out**, and lands on the logged-out landing with an honest confirmation ("Your account has been deleted."). The
deletion is **never optimistic** (`the information architecture & interaction model` §3.2/§3.4). Afterward, that account is **not-found**
and cannot be signed into (the user-side face of REQ-PROD-012; the durable-removal guarantee is `REQ-ID-*`/`REQ-OPS-*`).

**Applicable states** (matrix: `validation, busy_submitting, error, success, session_expired, not_found`):

| State | Design (copy intent + recovery) · instantiates state-patterns |
|---|---|
| `content` | The explanatory section + confirmation gate (baseline, §1.0). |
| `validation` | The confirmation gate (state-patterns §1.7): until the typed handle matches / the acknowledgement is checked, **Delete account** is disabled with an inline hint ("Type your handle to confirm"). Honest control state; no accidental deletion. |
| `busy_submitting` | **Delete account** shows the busy control, non-interactive, prevents a duplicate (state-patterns §1.8). Destructive ⇒ **never optimistic**; nothing is shown as removed until the deletion truly completes. |
| `error` | A failed deletion shows an **inline alert** — "We couldn't delete your account. Nothing was deleted — please try again." — honest that **the account is intact**, with retry (state-patterns §1.3). Never a partial or ambiguous outcome. |
| `success` | On real completion: the user is **signed out** and sees "Your account has been deleted." on the logged-out landing (state-patterns §1.9; hands off to the signed-out + not-found treatment). Shown only when deletion succeeded. |
| `session_expired` | Attempting to delete after the session expired routes through the **re-sign-in prompt** (state-patterns §1.6) — it MUST NOT silently delete and MUST NOT falsely succeed (REQ-UX-060, REQ-PROD-006); on re-sign-in the user returns to the deletion screen (it re-requires the explicit confirmation — a destructive action is never auto-resumed). |
| `not_found` | A later attempt to reach the now-deleted account shows the **"deleted" not-found** wording — "This account has been deleted." (state-patterns §1.4, the deleted-vs-never-existed distinction) — with a route back to the home/landing timeline (user-side of REQ-PROD-012). |

---

##### 4. SCR-data_export — request and obtain your own data

**Role & placement.** Request and download one's own data. Reached from `SCR-settings_home` → *Data export*
(`the information architecture & interaction model` §2 `/settings/export` nested). Satisfies REQ-PROD-011. **The export file *format* is
`REQ-DOM-*`/`REQ-ID-*`; the request → progress → download UX is the point of this screen** (brief). Export serves
**own data only** (the authorization that it is the user's own data is `REQ-ID-*`).

**Content & layout intent.** Single column:

- **Explanation** of what an export includes: "Your export includes **your profile, your chirps, and your follows**"
  — in a **portable, machine-readable file you can download and open outside chirp** (the concrete format is named
  by `REQ-DOM-*`; not invented here).
- **Request control**: **Request export** (`primary`).
- **Progress area** (after request): an honest "Preparing your export…" with a **determinate progress bar** where
  the progress is known (`visual-system` §3.15), or an honest indeterminate indicator otherwise — **never a fake
  progress fiction** (state model §1.1).
- **Download area** (when ready): a **Download** control (`visual-system` §3.1/§3.2) with the export file name shown
  in `font.family.mono` (`visual-system` §2.2 reserves mono for export file names) and a plain "Your export is ready."

**Happy-path flow.** Settings → *Data export* → read what's included → **Request export** → busy (request
accepted) → **preparing** (generation; honest progress) → **ready** → **Download** → the user downloads a portable
file of their profile, chirps, and follows (REQ-PROD-011). A long generation does not trap the user: they may leave
and return to find it ready (or still preparing), rather than being held on an indefinite spinner.

**Applicable states** (matrix: `busy_submitting, loading, success, error, session_expired`):

| State | Design (copy intent + recovery) · instantiates state-patterns |
|---|---|
| `content` | The explanation + *Request export* (baseline, §1.0). |
| `busy_submitting` | *Request export* shows the busy control and **prevents duplicate requests** (state-patterns §1.8) — "Requesting…"; honest acceptance, not a premature "ready". |
| `loading` | Export generation is the known-slow operation the state model names explicitly (state-patterns §1.1): "Preparing your export…" with the **determinate bar** where possible; a stalled preparation offers *Try again* / come back later rather than an indefinite spinner with no exit. |
| `success` | "Your export is ready." + **Download** (state-patterns §1.9) — the result is observable and real; no success shown unless a downloadable file genuinely exists. |
| `error` | A failed request or generation shows an **inline alert + retry** — "We couldn't prepare your export. Please try again." — and **never offers a partial or corrupt download** (state-patterns §1.3; REQ-PROD-018). |
| `session_expired` | Requesting or downloading after expiry routes through the **re-sign-in prompt** (state-patterns §1.6; REQ-UX-060, REQ-PROD-006) — export is own-data only, so a resumed session re-establishes the user before any data is served. |

*Out of matrix, recorded:* no `not_found`/`empty`/`end_of_list`/`validation` — export has no addressable subject to
be missing, no list to be empty/ended, and no free-text field to validate (it is a single-action request).

---

##### 5. SCR-signout — the sign-out control & its feedback

**Role & placement.** The deliberate end-of-session control. It has a **single, discoverable, consistent home** in
the signed-in **account menu**, and is **mirrored in `SCR-settings_home`** (`the information architecture & interaction model` §1.2, §3.6;
REQ-UX-005/-059). Satisfies REQ-PROD-005. This screen is the control + its confirmation/feedback; session
invalidation itself is `REQ-ID-*`.

**Content & layout intent.** Sign-out is **reversible** (the user can sign back in), so it does **not** get the
heavy destructive-confirm treatment of deletion (over-confirming trains dismissal — `the information architecture & interaction model`
§3.4). It gets:

- The **control** itself: *Sign out* in the account menu (`visual-system` §3.10 menu item) and as a row in
  `SCR-settings_home`.
- A **lightweight confirmation** suited to shared devices — a small confirm ("Sign out of chirp?" + *Cancel* /
  *Sign out*, `visual-system` §3.11) — to prevent an accidental tap, with *Cancel* the default focus. (Kept light,
  not consequence-heavy.)
- **Feedback** on completion.

**Happy-path flow.** Account menu / Settings → *Sign out* → light confirm → busy → **signed out**: the session ends,
the navigation chrome **swaps atomically to the logged-out set**, the identity indicator flips to the Sign in / Sign
up affordances (`the information architecture & interaction model` §2.4, §3.6), a brief toast "Signed out" confirms, and the user lands
on the logged-out landing/home timeline. Afterward the product treats them as logged out — they must sign in again
before they can post (REQ-PROD-005), observable on a shared device by a second person being unable to act as them.

**Applicable states** (matrix: `busy_submitting, success, error`):

| State | Design (copy intent + recovery) · instantiates state-patterns |
|---|---|
| `content` | The sign-out control in its consistent placements (baseline, §1.0). |
| `busy_submitting` | *Sign out* shows the busy control and prevents a duplicate (state-patterns §1.8) — "Signing out…". |
| `success` | The product is now logged out: nav swapped, identity indicator flipped, brief "Signed out" toast, landing on the logged-out timeline (state-patterns §1.9). Shown only when the session truly ended. |
| `error` | If sign-out fails, show an **honest** inline/toast error — "We couldn't sign you out. Please try again." — and keep the user **signed in** (never show a false "signed out" while the session is still live) with a retry (state-patterns §1.3). |

*Out of matrix, recorded:* no `validation`/`loading`/`not_found`/`empty`/`end_of_list`. `session_expired` is not in
the matrix and is designed as a graceful no-op here: signing out of an **already-expired** session simply resolves to
the logged-out state (the desired end-state) with no error theatre — consistent with the "no silent/ false outcome"
rule without inventing a prompt for a user who is already leaving.

---

##### 6. Composition check — the area maps onto existing components (no gap)

Every screen and state above is served by an existing `the visual language & design system` component and obeys an existing
`the information architecture & interaction model` pattern; **no component is invented, and nothing is flagged**.

| This fragment needs | Served by (visual-system) | Governed by (ia-interaction) |
|---|---|---|
| Settings hub list of navigable rows | Settings row / list §3.15; divider §3.15 | Account hub §1.2, screen map §2 |
| Identity at top of settings | Avatar + identity cluster §3.5 | State indicator §3.6 |
| Edit-profile form + validation | Text input/textarea §3.3; character counter §3.4; button §3.1 | Validation surfacing §3.5 |
| Deletion consequence + explicit confirm | Modal/dialog (confirm) §3.11; destructive button §3.1; text input §3.3 | Confirmation/destructive §3.4 |
| Export request → progress → download | Button/link §3.1/§3.2; determinate progress bar §3.15; mono file name §2.2 | Feedback §3.1; honest states §3.5 |
| Sign-out control + light confirm | Menu item §3.10; settings row §3.15; confirm dialog §3.11; toast §3.12 | Consistent placement §3.6; transition §2.4 |
| Every non-happy state | State blocks §3.14; inline alert §3.13; toast §3.12; §3.0 busy | State model (state-patterns §1); auth-gating §3.3 |

This agrees with `visual-system` §4's state→component mapping and §5's consistency bar, and with the state model's
own composition check (`state-patterns` §3).

---

##### 7. Requirements & acceptance criteria — REQ-UX-052 .. REQ-UX-061

Range owned by `the account & settings surfaces`. Each requirement is
stated from the user's side, is **testable by a stranger exercising the live product**, carries a `release_gating`
flag, and traces to ≥1 `REQ-PROD-*` and/or journey. Mechanisms (auth/session, deletion authorization, export
format) are deferred to `REQ-ID-*`/`REQ-DOM-*`/`REQ-OPS-*` and are **not** restated as UX requirements here.

###### REQ-UX-052 — A reachable settings / account-management hub
- **Statement:** A signed-in user has a reachable settings home that gathers the account-lifecycle surfaces in one place, without using the raw API.
- **Acceptance criteria:** From the signed-in account navigation, a user can open a Settings home that shows links to, at minimum, edit profile, data export, account deletion, sign-out, and privacy & terms; each is reachable from this hub in one interaction; none of these requires typing an API request or an unlinked URL.
- **Release gating:** true
- **Traces to:** REQ-PROD-009; J6, J7. (Builds on IA REQ-UX-004.)

###### REQ-UX-053 — View and edit your own display name and bio
- **Statement:** A signed-in user can view their account's handle, display name, and bio and can change the display name and bio, with the change persisting.
- **Acceptance criteria:** On the edit-profile screen the user sees their current handle (shown, not editable), display name, and bio; changing the display name and/or bio and saving shows the new values on their profile immediately and the values are still present on a later visit; the handle is not presented as editable.
- **Release gating:** true
- **Traces to:** REQ-PROD-008; J6. (Persistence mechanism → REQ-DOM-*.)

###### REQ-UX-054 — Editing a profile validates inline and never loses the edit
- **Statement:** The edit-profile form validates at the field in actionable language, and never discards the user's in-progress edit on a validation bounce, a failed save, or a session expiry.
- **Acceptance criteria:** Entering an empty display name or an over-limit bio produces a clear inline message at the field (not a generic error and not a raw API error), with the save control in an honest state (disabled or errored, consistently); a failed save keeps the typed edits and offers a retry without a full page reload; the edit is preserved when a save is blocked.
- **Release gating:** true
- **Traces to:** REQ-PROD-008, REQ-PROD-002, REQ-PROD-017; J6. (Inline-validation styling → REQ-UX-077; state model → REQ-UX-027.)

###### REQ-UX-055 — Account deletion states what is removed and requires explicit confirmation
- **Statement:** Initiating account deletion presents a clear statement of what deletion removes and requires an explicit confirmation before it proceeds, with the safe choice as the default.
- **Acceptance criteria:** Starting account deletion shows, before anything is deleted, that the account, its chirps, its follows, and its likes will be removed and that it cannot be undone; the destructive confirm is reachable only after an explicit confirmation step (e.g. typing the handle or acknowledging permanence); the destructive control is visually distinct and is not the default-focused control (Cancel is).
- **Release gating:** true
- **Traces to:** REQ-PROD-010; J7. (Destructive-confirm pattern → IA REQ-UX-018 / visual REQ-UX-076; deletion semantics/authorization → REQ-ID-*.)

###### REQ-UX-056 — Deletion is honoured on the user's side: signed out and not-found
- **Statement:** After a user confirms deletion, the product signs them out and the account is afterward presented as gone.
- **Acceptance criteria:** On confirming deletion the user is signed out and shown an honest confirmation that the account was deleted; a subsequent attempt to open that account's profile shows a clear "this account has been deleted" result (distinct from a never-existed profile) with a route back to a known place; the user cannot act as the deleted account afterward.
- **Release gating:** true
- **Traces to:** REQ-PROD-012; J7. (Durable removal guarantee → REQ-ID-*/REQ-OPS-*; not-found state → REQ-UX-024.)

###### REQ-UX-057 — Export your own data in a portable, downloadable form
- **Statement:** A signed-in user can request an export of their own data and download it in a portable form.
- **Acceptance criteria:** From the data-export screen the user can request an export and, when it is ready, download a file containing at least their profile, their chirps, and their follow list, which opens outside chirp; the export covers the user's own data only.
- **Release gating:** true
- **Traces to:** REQ-PROD-011; J7. (Export file format → REQ-DOM-*/REQ-ID-*.)

###### REQ-UX-058 — Export shows honest progress and resolves to a real download or an honest error
- **Statement:** The export flow shows honest busy/preparing feedback, never traps the user, and resolves to a genuine download or an honest, recoverable error.
- **Acceptance criteria:** Requesting an export shows an accepted/busy state that prevents duplicate requests; while the export is being prepared the user sees honest progress (not a fake or indefinite one) and is not trapped on a spinner with no exit; a ready export offers a download of a real file; a failed request or preparation shows a clear message with retry and offers no partial or broken download.
- **Release gating:** true
- **Traces to:** REQ-PROD-011, REQ-PROD-018; J7. (State model → REQ-UX-021/-028/-029.)

###### REQ-UX-059 — A sign-out control in a consistent place; after it, you are logged out
- **Statement:** A signed-in user can always find a sign-out control in a predictable place, and using it leaves the product in the logged-out state.
- **Acceptance criteria:** A visible sign-out control is present in the signed-in account navigation and within the settings hub, in the same place across the product; using it (with at most a lightweight confirmation) returns the product to the logged-out state — the navigation shows sign-in/sign-up and the user must sign in again before posting — observable on a shared device by a second person being unable to act as the first user afterward.
- **Release gating:** true
- **Traces to:** REQ-PROD-005; J3. (Placement → IA REQ-UX-005; session invalidation → REQ-ID-*.)

###### REQ-UX-060 — Session expiry in an account surface prompts re-sign-in and never acts silently
- **Statement:** When a user attempts an account-lifecycle action (edit profile, delete, export, open settings) after their session has expired, the product prompts a re-sign-in and neither fails silently nor appears to succeed, preserving their context.
- **Acceptance criteria:** After a session has expired, attempting to save a profile edit, start/confirm a deletion, or request/download an export surfaces a clear "session expired, sign in again" prompt; the action does not silently do nothing, does not appear to succeed, and does not perform a destructive action; after signing in the user returns to what they were doing with any in-progress input intact (a destructive action re-requires its explicit confirmation).
- **Release gating:** true
- **Traces to:** REQ-PROD-006; J6, J3. (Product-wide session-expiry state → REQ-UX-026; auth-gating → REQ-UX-016.)

###### REQ-UX-061 — Privacy & terms reachable from the settings hub
- **Statement:** A signed-in user can reach the privacy & terms surface from the settings hub.
- **Acceptance criteria:** The settings home shows a link to privacy & terms, and following it reaches the privacy & terms surface.
- **Release gating:** true
- **Traces to:** REQ-PROD-029, REQ-PROD-009; J7. (Both-states reachability owned by IA REQ-UX-007; this is the settings-hub entry.)

###### 7.1 Traceability summary (REQ-UX → REQ-PROD / journeys)

| REQ-UX | Screen(s) | Gating | Traces to |
|---|---|---|---|
| REQ-UX-052 | SCR-settings_home | true | REQ-PROD-009; J6, J7 |
| REQ-UX-053 | SCR-edit_profile | true | REQ-PROD-008; J6 |
| REQ-UX-054 | SCR-edit_profile | true | REQ-PROD-008, 002, 017; J6 |
| REQ-UX-055 | SCR-account_deletion | true | REQ-PROD-010; J7 |
| REQ-UX-056 | SCR-account_deletion | true | REQ-PROD-012; J7 |
| REQ-UX-057 | SCR-data_export | true | REQ-PROD-011; J7 |
| REQ-UX-058 | SCR-data_export | true | REQ-PROD-011, 018; J7 |
| REQ-UX-059 | SCR-signout | true | REQ-PROD-005; J3 |
| REQ-UX-060 | edit/deletion/export/settings | true | REQ-PROD-006; J6, J3 |
| REQ-UX-061 | SCR-settings_home | true | REQ-PROD-029, 009; J7 |

All ten ids are unique, fall within the owned `REQ-UX-052..061` sub-range, and lie within the parent
`REQ-UX-020..069` range.

---

##### 8. Coverage & handoff notes

- **Screen coverage.** All five owned screens are designed, each in exactly this
  fragment, with every state the matrix lists for it:

  | Screen | Required states (matrix) | Designed |
  |---|---|---|
  | SCR-settings_home | loading, error, session_expired (+content) | §1 — all |
  | SCR-edit_profile | loading, validation, busy_submitting, error, success, session_expired (+content) | §2 — all |
  | SCR-account_deletion | validation, busy_submitting, error, success, session_expired, not_found (+content) | §3 — all |
  | SCR-data_export | busy_submitting, loading, success, error, session_expired (+content) | §4 — all |
  | SCR-signout | busy_submitting, success, error (+content) | §5 — all |

  States recorded out-of-matrix are noted per screen with a reason (§§1–5); none required by the matrix was dropped.

- **States designed, not skeletal.** Each applicable state above has
  concrete copy intent and a named recovery affordance, instantiating the shared model in
  `the state model (empty, error & loading states)` §1 — not merely named.

- **Components consistent.** Every screen is built from named `visual-system`
  components and placed in the `ia-interaction` model (§6); no component was invented and **no gap was found to
  flag**. Should the design reveal a needed component the set lacks, it is recorded, not invented.

- **Boundaries kept (what, not how).** No front-end code; deletion semantics/authorization and session
  invalidation → `REQ-ID-*`; export format → `REQ-DOM-*`/`REQ-ID-*`; persistence/durable removal →
  `REQ-DOM-*`/`REQ-OPS-*`. This fragment does not redefine the screen inventory, the state vocabulary, or the
  REQ-UX id scheme (identities respected; composes against the design contract, the design contract, the design contract).

- **Seams to siblings.** `SCR-signout` and the session-expiry prompt share the auth/transition behaviour owned by
  `the entry & auth surfaces` (the sign-in destination/return-to-intent) and the global state in
  the state model (§1.6) — this fragment references, does not re-author, them. REQ-UX-060 is scoped to the
  account surfaces and traces REQ-PROD-006 without redefining REQ-UX-026/-016.

---

### screens_and_flows / trust_safety_admin — trust, safety, legal & operator-admin

*Section fragment: `screens_and_flows/trust_safety_admin`. Screens: **`SCR-report_block`**, **`SCR-privacy_terms`**, **`SCR-admin_support`**. Owned requirements: **REQ-UX-062 … REQ-UX-069**. Grounded in `Nousix-LLC/nswe-demo` @ `c805525`.*

This fragment designs the three surfaces chirp needs so that **users can stay safe and informed, and whoever runs
the product can help and protect them** — two audiences, one product seen from two vantage points:

- **For the end user** — a way to **respond to unwanted content or accounts** (`SCR-report_block`) and a **plain,
  readable statement of the data the product holds and the terms of use** (`SCR-privacy_terms`).
- **For the operator** — a minimal surface to **look up an account, answer a support request, and act on abuse**
  (`SCR-admin_support`), deliberately kept light and walled off from the consumer navigation.

The current product has none of these — no report, block, or mute (not even the overflow menu those actions hang
from), no privacy/terms surface, and no operator view of the running product at all. All three fill genuine gaps a
public posting product of this kind cannot responsibly ship without.

**What this fragment settles, and what it leaves to others.** It is **what, not how**: wireframe-level content and
layout intent, reading and action flows, reachability, and each applicable state with concrete copy intent and a
recovery affordance, plus the testable `REQ-UX-062..069` register. Every element composes from the shared visual
system, is placed and behaved per the IA & interaction model, and instantiates the one shared state model; this
fragment redefines no inventory, state, component, or id, and where an ideal component is missing it is **surfaced
as an open request (§5), not invented**. Three boundaries are held firm and deferred:

- **Enforcement & authorization** behind report / block / mute and operator moderation (who may act, what moderation
  does, what a block or a removal enforces) → identity & trust (`REQ-ID-*`) and operational readiness (`REQ-OPS-*`).
- **The content & scope of the privacy/terms text itself** (the actual wording, data categories, effective date)
  → identity & trust (`REQ-ID-*`).
- **Persistence meaning** of "remove"/"suspend"/"delete", session lifetime, the operator route's operational wiring,
  and any audit trail → `REQ-DOM-*` / `REQ-OPS-*` / `REQ-ID-*`.

This fragment designs the surfaces, flows, states, and the bar they are tested against — not the mechanism, not the
authorization model, and not the legal copy.

---

### 1. `SCR-report_block` — report & block *(end user)*

How a signed-in user **responds to unwanted content or accounts** without needing anyone else's help: they can
**report** a chirp or an account, and **reduce their own exposure** by **blocking** or **muting** an account — each
with honest confirmation. It is a small surface but a release-gating one (REQ-PROD-027): a product that lets anyone
post cannot be shipped responsibly with no way for a user to react to abuse. This surface owns the trust/safety items
on the chirp/profile overflow, the report flow, the block and mute flows, their wireframe layout intent, and every
applicable state; it owns `REQ-UX-062..064`.

#### 1.0 Starting point

The current product has **no trust/safety surface at all**: a chirp card exposes an author link and a like control
and nothing else — no overflow menu, no report, no block, no mute, no moderation. A user who meets an abusive chirp
or account today has no in-product response but to stop looking or leave. This screen fills that gap, building on two
additions the IA and visual-system work make on top of the current product: the **chirp/profile overflow menu**
(visual system Menu, §3.10; the chirp card §3.6 already reserves an overflow trigger slot) as the entry point, and
the **modal/dialog, toast, and inline-alert** feedback surfaces (visual system §3.11–§3.13) these flows use.

#### 1.1 Entry points & reachability

`SCR-report_block` is a **contextual** surface (IA §2.1): never a top-level nav destination — it is reached *in
context*, from the thing being acted on, through an **overflow menu** (IA §1.2/§1.4 reachability map). Two entry
contexts:

| Reached from | Overflow trigger sits on | Target of the action |
|---|---|---|
| A chirp on `SCR-home_timeline` or `SCR-chirp_thread` | the chirp card's overflow (`⋯`) trigger (visual system §3.6/§3.10) | the **chirp** (report) and/or its **author account** (report / block / mute) |
| A profile `SCR-profile` (another user's) | the profile header's overflow (`⋯`) trigger | the **account** (report / block / mute) |

- **Signed-in only, gated at the point of action.** Reading never requires an account (IA §3.3, REQ-PROD-013); these
  actions do. A logged-out viewer (or an expired session) who triggers a safety action is routed through the standard
  sign-in prompt and returned to the action on success (§1.4.6). On another user's content the overflow always
  presents these items; whether the viewer is *authorized* to act is an enforcement concern (`REQ-ID-*`), not
  surfaced as a UX gate here.
- **Own content is different.** The overflow on the user's *own* chirp shows **Delete** (owned by the
  account/content screens, not here), **not** report/block/mute; a user does not report or block themselves.
- **The overflow reflects the current relationship, honestly.** If the user has already blocked or muted the account,
  the overflow shows **Unblock @handle** / **Unmute @handle** instead of Block / Mute, so the action is always
  truthful and reversible (REQ-UX-063; the observable result is the confirmation — state model §1.9). A dedicated
  *list* of blocked/muted accounts for bulk management is a plausible account-settings surface; it is **not** invented
  here and is surfaced as a possible sibling (§5), since reversal is already reachable from the same overflow.

##### 1.1.1 Overflow safety items (wireframe intent)

```
chirp card ⋯            profile header ⋯
┌──────────────────────────┐     ┌──────────────────────────┐
│ ⚑  Report chirp          │     │ ⚑  Report @handle         │
│ ⚑  Report @handle        │     │ ⊘  Block @handle          │   (destructive-ink item)
│ ⊘  Block @handle         │     │ 🔇 Mute @handle            │
│ 🔇 Mute @handle           │     └──────────────────────────┘
└──────────────────────────┘
```

Rendered by the **Menu** component (visual system §3.10): `elevation.1`, `radius.sm`, icon + label items, a separator
above the exposure-reducing group; **Block is a destructive-intent item** (`error.ink` label, block glyph) per
§3.10; Mute is a neutral item. Icons come from the required glyph inventory (`report/flag`, `block`, plus a mute
glyph) — a **mute glyph is requested** from the icon set (§5, D2). Every item carries an accessible name (visual
system §2.7/§3.0). The menu obeys the §3.0 interaction states (hover/focus-visible/disabled) and dismisses on
outside-click/Escape without acting.

#### 1.2 The surface & layout intent (wireframe level)

Three distinct interaction shapes hang off the overflow, matched to the stakes of each action (IA §3.4 — destructive
confirms, reversible does not over-confirm):

- **Report** → a short **form modal** (collects a little input, then files).
- **Block** → a **confirm dialog** that states consequences (account-level, treated as destructive).
- **Mute** → a **direct, reversible action** with an undoable confirmation (no dialog gate).

##### 1.2.1 Report — modal form (visual system §3.11 `form` variant)

```
┌───────────────────────────────────────────────┐
│ Report this chirp                           ✕  │  title (font.h2)
│                                                 │
│ ┌─────────────────────────────────────────────┐│  what you're reporting (context):
│ │ ◎ Ada Lovelace  @ada · 2h                    ││  - chirp: identity cluster (§3.5) + a
│ │ "the quoted chirp text, truncated…"          ││    short quoted snippet from chirp card (§3.6)
│ └─────────────────────────────────────────────┘│  - account: identity cluster only
│                                                 │
│ What's wrong?  (helps us look into it)          │  reason — single-select, short set
│ [ Spam ] [ Abuse ] [ Doesn't belong ] [ Other ]│  (segmented control §3.15 as the composable
│                                                 │   today-shape; see §5 flagged ideal component)
│ Add detail (optional)                           │  textarea (§3.3) + counter (§3.4)
│ ┌─────────────────────────────────────────────┐│  required only if "Other" is chosen
│ │                                             ││
│ └─────────────────────────────────────────────┘│
│                                   280 remaining │
│                                                 │
│                     [ Cancel ]   [ Submit report ]  <- primary (§3.1), one per screen
└───────────────────────────────────────────────┘
```

- The **reason** is a light, user-facing categorization so the user can say *what* is wrong; it is **not** an
  authorization or enforcement taxonomy and drives no moderation behaviour here (that is `REQ-ID-*`). The categories
  shown are **representative, not frozen** (state model §0.3 — exemplars, not fixed strings).
- The **detail** textarea is optional in general and **required when "Other" is selected** (so an unspecified "Other"
  report is not filed blind). It carries the character counter (§3.4) and its typed text is draft-safe (REQ-PROD-017).
- One **primary** action — *Submit report* (§3.1 primary; one primary per screen); *Cancel* is the secondary and the
  safe default focus.
- Deliberately **not over-confirmed**: a report is reversible-in-consequence from the user's side (it just files), so
  filing it needs no second "are you sure?" — only an honest confirmation that it was received (§1.4.5).

##### 1.2.2 Block — confirm dialog (visual system §3.11 `confirm` variant; IA §3.4; REQ-UX-018)

Block is an **account-level** action and is treated as **destructive-by-consequence**, so it **MUST state its
consequences in plain terms before proceeding** (IA §3.4; REQ-UX-018):

```
┌───────────────────────────────────────────────┐
│ Block @handle?                                  │  title (font.h2)
│                                                 │
│ You won't see @handle's chirps, and they won't  │  consequence copy, plain terms, user-side
│ be able to see yours or interact with you.      │  (representative — see note below)
│ You can unblock them at any time.               │  states reversibility honestly
│                                                 │
│                     [ Cancel ]   [ Block ]       │  destructive button (§3.1 destructive);
└───────────────────────────────────────────────┘   Cancel is the default focus, NOT Block
```

- **Consequence copy is the design here; the enforcement behind it is not.** The dialog MUST tell the user, in plain
  product language, what blocking *means for them* before they commit — a first-class UX requirement (REQ-UX-063).
  The exact behaviours a block enforces server-side (visibility, interaction, follow relationships) are owned by
  identity & trust (`REQ-ID-*`); the copy above is **representative of the user-facing meaning of "block"** and MUST
  be reconciled with the authoritative `REQ-ID-*` enforcement model at build so the stated consequences match reality
  (an honest state never promises an effect the system does not deliver — state model §0.2). This reconciliation is
  recorded as a cross-area seam (§5, D5); it is a reconciliation, not an invention.
- The destructive **Block** button is visually distinct (`destructive`, §3.1) and is **not** the default-focused
  control; **Cancel** is the safe default (visual system §3.11; REQ-UX-076).

##### 1.2.3 Mute — direct reversible action (no confirm gate)

Mute is lighter and reversible, so per IA §3.4 it is **not** gated behind a confirmation dialog (over-confirming
trains users to dismiss dialogs). Selecting **Mute @handle** applies directly and confirms with an **undoable toast**
(visual system §3.12):

```
   ┌───────────────────────────────────────┐
   │ 🔇 Muted @handle.            [ Undo ]  ✕ │   toast (§3.12), auto-dismiss + manual dismiss
   └───────────────────────────────────────┘
```

The overflow item flips to **Unmute @handle** immediately (honest, reflects true state). Mute's user-side meaning —
*you stop seeing their chirps; they are not told* — is representative and reconciled with `REQ-ID-*` the same way as
block (§5, D5).

##### 1.2.4 Reversal

**Unblock** and **Unmute** are reached from the same overflow (which now shows them, §1.1), confirm with a brief
toast, and flip the item back. Unblock, being the *un*-doing of a destructive action and itself non-destructive,
needs no consequence dialog.

#### 1.3 Happy-path flows

1. **Report a chirp.** On a chirp's `⋯` → *Report chirp* → the report modal opens showing the quoted chirp → user
   picks a reason (and detail if "Other") → *Submit report* → busy on the button → **"Report received"** confirmation
   (§1.4.5) → modal closes. The user is returned to exactly where they were on the timeline/thread, context intact
   (IA §2.1 — contextual surfaces do not discard the underlying screen).
2. **Report an account.** From a chirp's `⋯` → *Report @handle*, or from a profile's `⋯` → *Report @handle* → same
   modal, targeted at the account (identity cluster, no chirp snippet) → submit → received.
3. **Block an account.** `⋯` → *Block @handle* → consequence-stating confirm dialog → *Block* → busy on the
   destructive button → the account's content disappears from the user's view (the **observable result** is the
   confirmation; state model §1.9) plus a brief "You blocked @handle" (toast/inline) → overflow now reads
   *Unblock @handle*.
4. **Mute an account.** `⋯` → *Mute @handle* → applied directly → undoable "Muted @handle" toast → overflow reads
   *Unmute @handle*.

Every flow resolves into exactly one honest outcome from the state vocabulary — no silent completion, no false
success (IA §3.1; state model §0.2).

#### 1.4 Designed states

The authoritative applicable set for `SCR-report_block` is **`validation`, `busy_submitting`, `error`, `success`,
`session_expired`**, with the `content` happy path implied. Each is designed below by instantiating the shared state
model, not re-invented. States **outside** this set do not apply and are recorded out-of-scope with a reason in
§1.4.7.

##### 1.4.0 `content` — the loaded surface
- **When:** the overflow is open, or the report modal / block dialog is shown. There is no data fetch to wait on; the
  surface's content is the menu items, the report form, or the confirm dialog themselves.
- **Composition:** Menu (§3.10), Modal form/confirm (§3.11), rendered from tokens and components; no state block.
  Transition in/out uses `motion.base`/`motion.slow`, instant under reduced motion.

##### 1.4.1 `validation` (state model §1.7; visual system §3.3/§3.4)
- **When:** the report form is submitted (or about to be) with a required input missing or out of bounds — "Other"
  chosen with no detail, or the detail over its character limit.
- **Copy intent & voice:** inline, field-level, actionable, plain — points at the field and says the fix, never a
  generic "invalid input" or a raw error. *Exemplars:* "Add a short detail so we know what's wrong." (required-detail);
  "You're over the limit — shorten your note." (over-limit).
- **Recovery affordance:** the message sits in the field's message slot; the character counter shows approaching/over-
  limit (§3.4); the **submit control's state is honest** (disabled while invalid, or errored on attempted submit —
  one consistent choice); **typed text is never lost** when submit is blocked (REQ-PROD-017).
- **Per-screen note:** reason selection itself is single-select so it cannot be "invalid"; validation here governs the
  detail field only.

##### 1.4.2 `busy_submitting` (state model §1.8; visual system §3.0)
- **When:** *Submit report*, *Block*, *Unblock*, or *Mute*/*Unmute* is in flight.
- **Copy intent & voice:** minimal — the control shows the work (spinner on the button; "Submitting…" acceptable),
  **size preserved so the dialog does not jump** (§3.0); the point is to show it is working and **prevent a duplicate
  submission** (the button is non-interactive while busy).
- **Recovery affordance:** on failure, return to the editable state with input intact and show `error` (§1.4.3); the
  control never strands the user in a permanent busy state. These actions are **not optimistic** — only like/follow
  are (IA §3.2); a destructive action is never optimistic (§3.4), so block shows a real busy state and only confirms
  on true success.

##### 1.4.3 `error` (state model §1.3; visual system §3.13 inline alert / §3.12 toast)
- **When:** a report or block/mute submission fails (transient network/server failure).
- **Copy intent & voice:** honest, specific, non-blaming; names what failed and that it can be retried; never a raw
  code, never a false success. *Exemplars:* "Your report didn't go through. Your note is safe — try again." /
  "Couldn't block @handle. Try again."
- **Recovery affordance:** for the **report/block form-and-dialog** failure, an **inline alert inside the modal** with
  **Retry in place** (not a toast the user can miss for a blocking action), and the chosen reason and typed detail
  **preserved** so the user retries without redoing the form (REQ-PROD-017/018). For the **mute/unblock toast** path,
  a transient failure surfaces an error toast with *Retry* and the overflow item reconciles to the true (unchanged)
  state — no false flip.

##### 1.4.5 `success` (state model §1.9; visual system §3.12 toast / §3.13 inline)
- **When:** a report is filed, or a block/mute/unblock/unmute completes.
- **Copy intent & voice:** an honest, brief confirmation that the action completed; the result is **observable**.
  **Never a false success** (shown only when the action truly succeeded). *Exemplars:* "Report received."; "You
  blocked @handle." + the account's content leaving the user's view; "Muted @handle." + *Undo*.
- **Recovery affordance:** n/a as a positive terminal; for the undoable actions (mute/unmute) the toast carries
  *Undo*. The report confirmation deliberately says only that it was **received** — it promises no outcome or
  timeline, because what moderation does is out of scope and over-promising would be dishonest.

##### 1.4.6 `session_expired` (state model §1.6; IA §3.3/§3.6)
- **When:** the user opens the overflow and triggers report/block/mute **after their session has expired or been
  invalidated** (including after a service restart, which clears in-memory sessions today).
- **Copy intent & voice:** a clear re-sign-in prompt — "Your session has expired. Sign in again to continue." +
  *Sign in* — **never a silent fail and never a false "reported/blocked"** (REQ-PROD-006).
- **Recovery affordance:** routed through the **one** auth-gating prompt (IA §3.3), **preserving the user's intent and
  any typed report detail** so they resume the exact action after signing in (return-to-intent, IA §2.4); the state
  indicator flips to logged-out (§3.6). The session's lifetime/invalidation mechanism is `REQ-ID-*`; only the prompt
  and the resume are designed here.

##### 1.4.7 States recorded out-of-scope for this screen (with reason)
- `loading` — this screen fetches no data the user waits on (its content is menu/form/dialog); the target
  chirp/account was already loaded by the host surface. Not applicable.
- `empty` / `end_of_list` — no list is presented. Not applicable.
- `not_found` — if the target chirp/account is already gone, that is surfaced by the **host** timeline/profile (chirp
  `tombstone`, state model §1.4), not by this screen; the overflow action simply isn't offered on a thing that isn't
  there. Deferred to the host surface, not designed here.

---

### 2. `SCR-privacy_terms` — privacy & terms *(end user)*

A **plain, readable** statement of **what personal data chirp holds about a user** and the **terms under which the
product is used**, reachable by everyone and readable without an account. This surface designs the **vessel, its
reachability, and its readability** — not the words: the content and scope of the legal and privacy text itself is
deferred to identity & trust (`REQ-ID-*`). It owns `REQ-UX-065..066`.

#### 2.1 Screen summary

| | |
|---|---|
| **Purpose** | Give any visitor a plain, readable statement of **what personal data chirp holds about a user** and the **terms** under which the product is used. |
| **Who** | Everyone — a logged-out visitor (persona P1) and a signed-in user, including the leaving user (persona P3). Reading **never requires an account** (REQ-PROD-013). |
| **Why it exists** | REQ-PROD-029: a public product that collects personal data is expected to carry this surface; the statement must be **present and readable rather than absent**. The current product has **no privacy/terms surface** — this screen fills that gap. |
| **Character** | A **mostly-static, long-form readable document**. Not a form, not an action-flow: the content is the message. The single interaction is reading (and following links). |
| **Journey tie-in** | J7 ("Leaving: deletion and taking data out"): the user deciding what chirp knows about them, and whether to export or delete, reads this surface to understand what data is held and under what terms. |

#### 2.2 Reachability (both navigation states) — REQ-UX-065, REQ-UX-007

The surface is **one screen reached from several stable entry points**, present in **both** navigation states.
Reachability is inherited from the IA/interaction model (§1.1, §1.2, §1.4, §2) and is **not redefined here** — this
fragment restates the entry points it is responsible for honouring.

```
LOGGED-OUT (public — persona P1)
  persistent footer / low-prominence chrome on every logged-out surface:
        About chirp   ·   Privacy & terms  ──────────────┐
                                                          │
SIGNED-IN (account menu — persona P2/P3)                  │
  ( @handle ▾ ) account menu (overflow):                  ├──▶  SCR-privacy_terms
        - Profile                                         │     (one screen, one
        - Settings ───▶ SCR-settings_home                 │      stable destination
                          · … · Privacy & terms ──────────┤      at /privacy)
        - Privacy & terms ────────────────────────────────┘
```

- **Logged-out**: a `Privacy & terms` link lives in the persistent footer / low-prominence chrome of **every**
  logged-out surface (IA §1.1). Following it reaches `SCR-privacy_terms` **without any sign-in prompt** — this surface
  is **not** auth-gated (contrast the gated actions in IA §3.3).
- **Signed-in**: reachable from the **account menu** directly (`Privacy & terms`, IA §1.2) and from within the
  **account-management hub** `SCR-settings_home` as a **Settings row / link** (visual system §3.15) — the same link,
  never buried and never duplicated into an unpredictable location (IA §1.2).
- **Stable address**: a single top-level, low-prominence destination (`/privacy`, IA §2) that survives a page refresh
  and the back button and can be returned to directly; the settings and footer links resolve to it. The screen is
  **reached from several places but exists once** — no divergent copies.
- **Link treatment**: the entry points use the **Link** component (visual system §3.2) in the chrome/footer and the
  **Settings row / list** component (§3.15) in the settings hub.

The reachability requirement is restated as **REQ-UX-065** (§4), complementing the IA-level `REQ-UX-007` (privacy &
terms reachable in both navigation states) by binding it to **this screen** and adding the no-account-required and
single-destination guarantees.

#### 2.3 Information architecture of the document & layout intent (wireframe level)

A **calm, single-column long-form reading document**, built from the visual system's reading typography (§2.2) within
the readable content column (§2.6, `layout.content.max` = 600px, `layout.gutter` 16px). "Content is the hero" (visual
system §1): generous whitespace, clear hierarchy, chrome recedes.

```
┌───────────────────────────────────────────────┐  width capped at layout.content.max (600px)
│  [chirp]  nav/chrome (logged-out OR signed-in)  │  ← chrome swaps per nav state (IA §1.1/§1.2); the
│                                                 │     document body is identical in both states
├───────────────────────────────────────────────┤
│                                                 │
│  Privacy & terms                     (font.h1)  │  ← screen title
│  Last updated: {effective date}      (font.meta,│  ← effective-date line; the DATE VALUE is content
│                                       secondary)│     (REQ-ID-*), the affordance of showing one is UX
│                                                 │
│  {one-line plain-language orientation}  (body)  │  ← "In short, here is what chirp knows about you and
│                                                 │     the terms of use." (intent; exact words REQ-ID-*)
│                                                 │
│  On this page                                   │  ← lightweight in-page section index (see §5 flag):
│   · What data we hold     · Your choices         │     anchors to the sections below, aids a long read
│   · How it is used        · Terms of use         │
│  ───────────────────────────────────── divider │
│                                                 │
│  What data chirp holds about you    (font.h2)   │  ← section heading
│  {prose: paragraphs + lists}         (body)     │  ← section body (words = REQ-ID-*)
│                                                 │
│  How your data is used              (font.h2)   │
│  {prose}                                        │
│                                                 │
│  Your choices                       (font.h2)   │
│  {prose} …  Export your data → (link, §3.2)     │  ← optional cross-links to EXISTING surfaces (§3.1)
│            Delete your account → (link, §3.2)   │
│                                                 │
│  Terms of use                       (font.h2)   │
│  {prose}                                        │
│                                                 │
│  {contact / how to ask about your data}  (body) │  ← intent only; channel value is REQ-ID-*
│                                                 │
├───────────────────────────────────────────────┤
│  footer / chrome (Privacy & terms link present) │
└───────────────────────────────────────────────┘
```

##### 2.3.1 Layout & content-structure intent
- **Single readable column**, `layout.content.max` (600px), centred, 16px side gutters (visual system §2.6).
- **Typographic hierarchy** (visual system §2.2): screen title `font.h1`; section headings `font.h2`; body `font.body`
  (16px / 1.45 line-height) for comfortable reading; the effective-date and contact meta lines `font.meta` in
  `text.secondary`; never essential text below `font.meta` in secondary colour.
- **Scannable sectioning**: the statement is broken into a small number of **named sections** with `font.h2` headings
  (illustrative set: *what data is held* · *how it is used* · *your choices* · *terms of use*), so a reader can find
  the part they care about rather than facing a wall of text. The **exact section titles and the prose are content
  (`REQ-ID-*`)**; this fragment fixes that there **is** a heading-and-section structure using the shared type scale.
- **An "On this page" in-page index** anchors to the sections — a readability aid for a long document. See the
  component note in §5 (the visual system provides sectioning primitives but not a dedicated long-document
  table-of-contents / scroll-spy; this is **flagged**, not invented).
- **Effective-date line** ("Last updated: …") directly under the title — standard, trust-building craft for a
  legal/privacy surface. The date **value** is content (`REQ-ID-*`); showing one is the UX decision made here.
- **Optional cross-links to existing surfaces**: within "Your choices", `Export your data →` (`SCR-data_export`) and
  `Delete your account →` (`SCR-account_deletion`), using the Link component (§3.2). These connect the privacy
  statement to the J7 actions a reader naturally wants next. They are **links to surfaces owned elsewhere** — this
  fragment invents no behaviour for them — and they appear **only when the reader is signed in** (both targets are
  signed-in account-management surfaces per IA §1.2); logged-out, the "Your choices" prose stands without the action
  links.

##### 2.3.2 Legibility bar (binds REQ-UX-066)
Comfortable contrast and clear hierarchy (visual system §1.4 "trustworthy and legible"); body text at `font.body` size
and line-height within the 600px measure; links distinguishable by **colour and underline-on-interaction**, not
colour alone (§3.2); headings establish the section structure. The document is **plain and readable** — not a raw text
dump, not an unstyled wall, and not a scanned PDF.

#### 2.4 Happy-path reading flow — the `content` state

`content` is the primary, designed path and the baseline for this surface.

1. A visitor (logged-out or signed-in) follows a `Privacy & terms` entry point (§2.2).
2. The surface renders the document **immediately** at its stable address — title, effective date, the section index,
   and the sectioned statement — within the reading column.
3. The visitor **reads**, optionally jumping via the "On this page" index to a section, and optionally following an
   inline/choices link (export, deletion, or a contact affordance).
4. There is **no gate, no sign-in wall, and no required action** to read. The visitor leaves by normal navigation
   (back, the chrome, or an outbound link they chose).

**Design decision — static baseline.** The statement is treated as **statically-available content**: on the happy
path the surface does **not** depend on a successful data fetch to be readable. This is deliberate and is the
strongest way to satisfy REQ-PROD-029 ("present and readable rather than absent") and REQ-PROD-013 (reading never
requires an account): the legal/privacy surface is the **one surface that must never be a blank page or an error**.
Whether the implementation bundles the text or serves it is a *how* left to the code/identity work — but the UX bar is
that **a reader always gets a readable document**. The `loading` and `error` states in §2.5 are therefore designed as
an **honest fallback for the fetched variant**, not as the expected happy path.

#### 2.5 Applicable states — `loading`, `error` (apply only if the statement is fetched)

Per the per-screen state matrix, `SCR-privacy_terms` lists `loading` and `error`, which **apply only if the statement
is fetched**. Rather than record them out-of-scope, this fragment **designs both** (concrete copy intent + recovery
affordance), instantiating the shared state model, so the screen is **not skeletal under either implementation
choice** (static or fetched). `content` is designed in §2.4.

##### 2.5.1 `loading` (state model §1.1)
- **When:** only in the fetched variant, while the statement is in flight on a first/again fetch (not reached at all
  in the static baseline, §2.4).
- **Copy intent & voice:** **wordless** — a shape, not a sentence (state model §1.1, voice §0.4). A **document-shaped
  skeleton** (title bar + a few paragraph-line placeholders) within the reading column, so arriving content does not
  jump the layout. If the wait is unusually long, a single calm line ("Still loading…") is permitted — never an
  apology, never a progress fiction.
- **Recovery affordance:** a load must never trap the reader. If loading exceeds a reasonable wait, the surface offers
  **Try again**, transitioning to in-place `error`-style recovery (§2.5.2) rather than an endless spinner.
- **Composition:** the **Loading / skeleton state block** (visual system §3.14), shape-matched to a long document
  (header + paragraph lines) on `bg.subtle`.

##### 2.5.2 `error` (state model §1.3)
- **When:** only in the fetched variant, when the fetch of the statement fails (transient network or server failure).
- **Copy intent & voice:** honest, specific, non-blaming — name that the statement could not be loaded and that it can
  be retried; never a raw error code, never a false "loaded", never a blank/broken screen (state model §1.3, voice
  §0.4). *Exemplar intent:* "We couldn't load the privacy & terms right now." + **Try again**.
- **Recovery affordance:** **Retry in place, without a full page reload** (REQ-PROD-018); the chrome/footer (and its
  `Privacy & terms` link) remain present so the reader is never stranded. Because this surface must be readable, the
  **error state carries the Retry as its primary way forward** and must never be the terminal experience of a reader
  who came to read.
- **Composition:** the **Error state block with Retry** (visual system §3.14) within the reading column.
- **Design note:** this state exists **because** a fetched implementation can fail — the recommended static baseline
  avoids it entirely. The fragment records the state as designed so that whichever implementation path is taken, the
  surface stays honest; if the surface ships fully static, `loading` and `error` **do not apply** and this is a
  correct, pre-designed outcome, not a skipped state.

*(Other state-model states — `empty`, `not_found`, `end_of_list`, `session_expired`, `validation`, `busy_submitting`,
`success` — are **not applicable** to this surface: it has no empty set, no per-item identity to be missing, no
pagination, no auth gate, no form/submit. Recorded here deliberately, per the matrix, rather than left ambiguous.)*

---

### 3. `SCR-admin_support` — operator / admin support *(operator)*

This surface is the one **someone running chirp uses to help and protect users** — look up an account, see enough
about it to answer a support request, and act on abuse. It is designed to the bar of *a designed screen, not a
skeleton*: a wireframe-level layout, a happy-path flow, and every state the matrix marks applicable, each with
concrete copy intent and a recovery affordance. It is deliberately **light** — a minimal operator surface, **not** a
full admin console — and it designs the **screen, interaction, and feedback only**. It owns `REQ-UX-067..069`.

**Who it is for.** The operator / administrator (persona **P4**), on the operator journey **J8** step 4, "supporting
and protecting users." Today the operator has no view of the running product at all: no admin surface, no user lookup,
no moderation. From the operator's side there is nothing to look at, so a user who writes in to say *"my account is
gone"* or *"I can't sign in"* cannot be helped. This screen is the UX-level fill for that gap. The product
requirements also name a **deliberately-absent moderator** persona; this surface is where that role's work would be
done, kept minimal here.

**What it satisfies.** `REQ-PROD-033` (a minimal admin & support capability — find an account, answer a support
request, remove abusive content, through a provided surface rather than by touching storage) and `REQ-PROD-028`
(abuse can be acted on by whoever runs the product, with the effect visible to users), both at **UX level**.

**What it explicitly does NOT design (deferred — see §3.6).** *Whether* a given person may open this surface,
*whether* they may act, and *how* that is enforced are **capability and authorization** concerns owned by identity &
trust (`REQ-ID-*`) and operational readiness (`REQ-OPS-*`). This screen **invents no authorization model** and designs
no login/role/permission UI; it is designed as the surface an *already-authorized* operator sees once admitted. The
route's operational wiring, the meaning of "remove"/"suspend" in storage, session lifetime, and the operator
health/recovery duties (`REQ-PROD-030/031/032`) are likewise out of this screen's UX scope.

#### 3.1 Where it lives — a segregated operator route (not consumer navigation)

`SCR-admin_support` sits on a **distinct operator route that is walled off from the consumer navigation** (IA
§1.3/§1.4, `REQ-UX-009`). It is **not** linked from the signed-in consumer nav, the account menu, or any consumer
surface, and is reached **only through its own operator entry**. The consumer and operator experiences are the same
product seen from different vantage points, not two apps — but this surface is intentionally not discoverable from
inside the consumer product.

- The screen presents a **minimal operator chrome**, visually distinct from the consumer nav so an operator is never
  in doubt which surface they are on: the product wordmark, an **"Operator · support"** label, and a persistent
  indicator of **which operator is signed in** (the identity-indicator pattern, IA §3.6; present for accountability,
  kept light). No consumer primary nav (Home / Compose / Search) appears here.
- **Reachability of the entry itself, and the gate that admits an operator, are deferred** (§3.6). This fragment
  designs the surface *behind* the gate; it does not design the gate.

#### 3.2 Screen anatomy & layout (wireframe level)

The surface is **data-dense**, so it uses the wide operator column `layout.wide.max` = **960px** (visual system §2.6)
rather than the 600px consumer reading column. All type, colour, spacing, radius and elevation come from visual-system
tokens (§2); machine values — account ids, chirp ids — are set in `font.family.mono` (§2.2), which the visual system
reserves for exactly this admin/support surface.

```
┌──────────────────────────────────────────────────────────────────────────┐  operator chrome (not consumer nav)
│  chirp · Operator · support                               signed in as @op │
├──────────────────────────────────────────────────────────────────────────┤
│  Find an account                                                           │  lookup
│  ┌────────────────────────────────────────────────┐                       │
│  │ 🔍  handle or account id                      ✕ │   [ Look up ]         │  search input §3.3 + primary §3.1
│  └────────────────────────────────────────────────┘                       │
│  Enter a handle (e.g. @ada) or an account id.                              │  helper §3.3
├──────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────────────────┐ │  account support-detail
│  │ (◍) Ada Lovelace   @ada            [ Active ]        id: a1b2c3  (mono) │ │  identity cluster §3.5 + status badge §3.15
│  │     Joined 12 Mar 2026 · 48 chirps · last active 2h ago                │ │  support detail (meta, §2.2 typography)
│  │                                                      ⟶ account actions ⌄│ │  account-level moderation menu §3.10/§3.1
│  ├──────────────────────────────────────────────────────────────────────┤ │
│  │  Recent chirps                                                         │ │  recent-activity list
│  │  ┌────────────────────────────────────────────────────────────┐  ⋯    │ │  chirp card §3.6 + overflow §3.10
│  │  │ Ada Lovelace @ada · 2h    "…"                        ♥ 3     │       │ │    overflow → Remove chirp (destructive)
│  │  └────────────────────────────────────────────────────────────┘       │ │
│  │  ┌────────────────────────────────────────────────────────────┐       │ │
│  │  │ This chirp is unavailable (removed)                          │       │ │  tombstone variant §3.6 (after removal)
│  │  └────────────────────────────────────────────────────────────┘       │ │
│  └──────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────┘
```

##### 3.2.1 Operator chrome
Minimal, as §3.1: wordmark + "Operator · support" + the acting-operator indicator. No consumer nav. A sign-out
affordance is present (the operator is a signed-in user too); its placement reuses the consumer sign-out pattern
conceptually but lives in this chrome, not the consumer account menu.

##### 3.2.2 Lookup — the operator's entry into the running product
- **Component:** the **search input** variant (visual system §3.3) — leading search icon, a clear (`✕`) control —
  labelled **"Find an account"**, with helper text **"Enter a handle (e.g. @ada) or an account id."** A single
  `primary` **"Look up"** action (§3.1); at most one primary on the screen.
- **Behaviour:** the operator looks an account up **by handle or by id**. This is the operator's entry point — the
  surface opens on the lookup with no result yet (a neutral resting prompt, not an error). Submitting resolves into
  exactly one honest outcome (`loading` → `content` / `empty` / `not_found` / `error`), per IA §3.1.
- The lookup field **persists** above any result so the operator can immediately search again; its typed value is
  preserved across a bounced submit (`validation`) and across an `error` retry.

##### 3.2.3 Account support-detail — *enough to answer, not a data dump*
When a lookup matches, the surface shows a compact **support-detail** block — deliberately scoped to what answers a
support request, not a full account export:
- **Identity cluster** (visual system §3.5): display name + `@handle`, with the account id shown in `mono` (§2.2) for
  unambiguous reference.
- **Account status badge** (§3.15 badge; status never carried by colour alone — icon + text, `REQ-UX-073`): one of
  **Active**, **Signed-out** (valid account, no live session — answers *"I can't sign in"*), **Suspended**, or
  **Deleted** (answers *"my account is gone"*).
- **Support meta:** join state ("Joined …"), basic activity (chirp count, last-active relative timestamp). This is the
  minimum that lets an operator answer the two canonical questions in the brief; it is **not** a full personal-data
  readout (what the account's data comprises in detail is a privacy/identity concern, `REQ-ID-*`).
- **Recent chirps:** the account's recent chirps rendered with the **one** chirp card component (§3.6) — the same card
  used everywhere, so the operator sees chirps as users do — each carrying an **overflow** (§3.10) whose destructive
  item is **"Remove chirp."**

##### 3.2.4 Moderation actions — remove a chirp, act on an account
Two moderation capabilities, both **destructive and therefore always confirmed** (IA §3.4; visual system §3.11) and
**never optimistic** (IA §3.2 confines optimism to reversible like/follow):

- **Remove an abusive chirp.** From a chirp card's overflow → **Remove chirp** (`destructive`, `error.ink` item).
  Opens a **confirm dialog** (§3.11) that **states the consequence in plain terms** — *"Remove this chirp? It will no
  longer appear to anyone on chirp."* — with **"Cancel"** as the safe default focus and a distinct `destructive`
  **"Remove chirp"** action (visual system §3.1/§3.11, `REQ-UX-076`). On confirm: `busy_submitting` on the dialog
  action → on success the chirp is replaced **in place** by the **tombstone** variant (§3.6, *"This chirp is
  unavailable (removed)"*) and a success toast confirms. The removal is understood to **become visible to users** —
  the content no longer appears (`REQ-PROD-028`).
- **Act on an abusive account.** An **account-actions** menu (§3.10) on the detail block offers **Suspend account** and
  **Remove account** (`destructive`). Each opens a **confirm dialog** stating the consequence — *"Suspend @ada? Their
  chirps stop appearing to other users until the suspension is lifted."* / *"Remove @ada? Their account and chirps
  stop appearing to users."* — safe default focus, distinct destructive action. On confirm: `busy_submitting` → on
  success the **status badge updates** (→ Suspended / Removed) and a toast confirms.

Keeping it light: there is **no free-text moderation-reason requirement, no case queue, no audit-log UI, no bulk
tooling** here — those are admin-console scope beyond this minimal surface. The one place input is validated is the
lookup field (§3.2.2). (An audit trail of operator actions is an operational-readiness concern, `REQ-OPS-*`, not
designed here.)

#### 3.3 Happy-path flow (`content`) — J8 step 4

1. An already-authorized operator opens the segregated operator route (the admitting gate is deferred, §3.6) and lands
   on the lookup surface.
2. The operator types the user's **handle or account id** and submits **Look up** → `loading`.
3. The **support-detail** block loads (`content`): identity, **status badge**, join/activity, recent chirps.
4. The operator **answers the support request** by reading the status — *"my account is gone"* reads as **Deleted**;
   *"I can't sign in"* reads as **Signed-out** (or Active) — a concrete, honest answer rather than a shrug.
5. If content or the account is abusive, the operator removes the chirp (overflow → confirm → remove) or acts on the
   account (account actions → confirm → suspend/remove); each resolves to an observable `success` (tombstone in place,
   or an updated status badge).
6. The effect is understood to **become visible to users**: removed content no longer appears.

#### 3.4 Designed states

Per the per-screen state matrix, `SCR-admin_support` designs: `loading`, `empty`, `error`, `not_found`,
`busy_submitting`, `success`, `session_expired`, `validation` — plus the implied `content` happy path (§3.3). Each
state instantiates the shared state model (no per-screen reinvention) and is rendered by a named visual-system block
(nothing invented — see §3.5). The **`empty` vs `not_found` distinction** the model draws (state model §1.4) is
honoured explicitly.

##### 3.4.1 `loading` (state model §1.1)
- **When:** a lookup is in flight, or the detail/recent-chirps are fetching after a match.
- **Copy & voice:** wordless — a shape, not a sentence; the operator chrome and the persisted lookup field are **not**
  blanked while the result area loads.
- **Recovery:** a load never traps — if it stalls past a reasonable wait, a **Try again** is offered rather than an
  indefinite spinner.
- **Composition:** the **Loading/skeleton** state block (§3.14) — a **row skeleton** matched to the detail/list shape
  so arriving content does not jump the layout; an in-control spinner (§3.0) only for the moderation-action `busy`
  case (§3.4.5), not for the whole-surface load.

##### 3.4.2 `empty` (state model §1.2) — *the query ran, nothing matched*
- **When:** the lookup **succeeded** but **no account matches** the handle or id entered. This is the **`empty`** case,
  and it is **distinct from `not_found`** (§3.4.4): the search ran and returned nothing, as opposed to a specific named
  account that is gone.
- **Copy & voice:** confirm the search ran and help the operator refine it — *"No account matches that handle or id.
  Check the spelling, or try the other identifier (a handle like @ada, or an account id)."* Never a bare blank area.
- **Recovery:** the lookup field stays, is re-focused, and holds the typed query so the operator can correct it — the
  primary next action for this surface.
- **Composition:** the **Empty** state block (§3.14), "admin no-match" variant (named in visual system §3.14 and state
  model §1.2).

##### 3.4.3 `error` (state model §1.3)
- **When:** a lookup/fetch fails, or a moderation action fails (transient network/server failure).
- **Copy & voice:** honest, specific, non-blaming — never a raw error code or a false success. Fetch — *"Something went
  wrong looking that up."* Action — *"That didn't go through — the chirp was not removed. Try again."*
- **Recovery:** **fetch** failure → **Error** state block with **Retry** that refetches **in place without a full page
  reload** (`REQ-PROD-018`); the persisted lookup query is kept. **Action** failure → an **inline alert** at the
  action (§3.13), the action returns to its editable/pre-confirm state, **no false success** and **no partial effect
  shown** (the chirp is not tombstoned, the badge does not change).
- **Composition:** Error state block + Retry (§3.14) for fetch; inline alert (§3.13) for an action.

##### 3.4.4 `not_found` (state model §1.4) — *a named account that is gone*
- **When:** the operator addresses a **specific** account that **does not exist or has been removed** — e.g. opening a
  known account id (a prior reference, a deep link) that has since been **deleted/removed**, or an id that never
  existed. Contrast §3.4.2: a **lookup with no match is `empty`**; a **named account that is gone is `not_found`** —
  this fragment keeps that distinction, as the model requires.
- **Copy & voice:** state it plainly and **distinguish "deleted" from "never existed"** where the product knows —
  *"This account has been deleted."* / *"No account with that id exists."* (ties to `REQ-PROD-012`, the
  deletion-honoured wording). This is also how an operator confirms a *"my account is gone"* report when the account
  was genuinely deleted.
- **Recovery:** a **route back** to the lookup (a clear "Back to lookup" / the lookup field itself).
- **Composition:** the **Not-found** state block (§3.14) for the addressed account.

##### 3.4.5 `busy_submitting` (state model §1.8)
- **When:** a confirmed moderation action is in flight — removing a chirp, suspending or removing an account.
- **Copy & voice:** minimal — the confirm dialog's action button shows the work ("Removing…" / a spinner); the point
  is to **show it is working and prevent a duplicate**, not to narrate.
- **Recovery:** on **failure** → return to the editable/pre-confirm state with the `error` treatment (§3.4.3), input
  and dialog intact; the control never strands the operator in a permanent busy state. Destructive actions are **never
  optimistic** — the effect is shown **only after** the action truly succeeds (IA §3.2/§3.4).
- **Composition:** the §3.0 **busy control** state on the dialog's action button — non-interactive, **size preserved**
  so the dialog does not jump.

##### 3.4.6 `success` (state model §1.9)
- **When:** a moderation action completed.
- **Copy & voice:** an honest confirmation whose **result is observable**, never a false success — *"Chirp removed."* /
  *"@ada suspended."* / *"@ada removed."*
- **Recovery / result of record:** the **observable result is the confirmation** — a removed chirp becomes a
  **tombstone in place** (§3.6); an account action **updates the status badge** (→ Suspended / Removed). A brief
  **toast** (§3.12) accompanies it.
- **Composition:** toast (§3.12) + the reconciled surface (tombstone §3.6 / updated badge §3.15).

##### 3.4.7 `session_expired` (state model §1.6, satisfies `REQ-PROD-006`)
- **When:** the operator attempts an action (a lookup, or a moderation action) **after the operator session has
  expired or been invalidated** — including after a service restart, which invalidates in-memory sessions today.
- **Copy & voice:** a clear re-sign-in prompt — *"Your session has expired. Sign in again to continue."* — **never a
  silent fail and never a false success** (a moderation action must not appear to have happened).
- **Recovery:** route to sign-in **preserving the operator's context** (the account being viewed, the action
  attempted) so it can be resumed; do not silently discard the in-progress action.
- **Composition:** the **Session-expired** prompt block (§3.14), routed through the single auth-gating pattern (IA
  §3.3) and reflected by the operator indicator flipping to signed-out (IA §3.6). *The session's lifetime and
  invalidation mechanism are `REQ-ID-*`; only the prompt and the resume are designed here.*

##### 3.4.8 `validation` (state model §1.7)
- **When:** the **lookup field** input violates a rule — an empty submit, or an entry that is neither a plausible
  handle nor a plausible account id.
- **Copy & voice:** inline, field-level, actionable — *"Enter a handle or an account id to look up."* / *"That doesn't
  look like a handle (e.g. @ada) or an account id."* — never a generic "invalid input" and never a raw server/API
  error.
- **Recovery:** the message points at the field, the submit control's state is honest (one consistent choice:
  disabled-until-valid *or* error-on-submit), and the **typed text is never lost** when a submit is blocked.
- **Composition:** the input error/validated states (§3.3). *(Scope note: validation is on the lookup field; the
  surface deliberately requires no free-text on moderation actions — see §3.2.4 "keeping it light.")*

#### 3.5 Composition check — every state and element maps onto an existing component (no gap)

The surface is built **only** from existing visual-system components and IA patterns; **no component is invented**,
and the one data-dense need the brief flags (operator tables) is served by the reserved wide column plus existing
rows/cards. Nothing is flagged for resolution from this screen.

| Need on this screen | Existing component / token (named, not re-drawn) |
|---|---|
| Data-dense operator width | `layout.wide.max` 960px (§2.6) |
| Account id / chirp id typography | `font.family.mono`, reserved for admin/support (§2.2) |
| Account lookup | Search input variant (§3.3) + one `primary` button (§3.1) |
| Account identity | Identity cluster — display name + `@handle` (§3.5) |
| Account status | Badge / count pill (§3.15), icon + text (never colour-alone, `REQ-UX-073`) |
| Support meta (join/activity) | `font.meta` / `font.label` typography (§2.2); settings-row layout (§3.15) |
| Recent chirps | Chirp card `default` (§3.6) + overflow menu (§3.10) |
| Account-level actions | Menu (§3.10) + `destructive` buttons (§3.1) |
| Destructive confirmation | Modal `confirm` dialog, safe default focus (§3.11, `REQ-UX-076`) |
| Removed chirp result | Chirp card `tombstone` variant (§3.6) |
| `loading` / `empty` / `error` / `not_found` / `session_expired` | State blocks (§3.14) |
| `busy_submitting` | §3.0 busy control state |
| `success` | Toast (§3.12) + reconciled surface |
| `error` on an action | Inline alert (§3.13) |

This satisfies the component-consistency requirement for this screen.

#### 3.6 Deferrals (explicitly out of this screen's UX scope)

Recorded so the deferral is honest, not silent:

- **Capability & authorization — deferred to `REQ-ID-*` / `REQ-OPS-*`.** *Whether* a person may open
  `SCR-admin_support`, *whether* they may look up or act, and *how* that is enforced are not designed here. The screen
  **invents no authorization model** — no role picker, no permission matrix, no login UI — and is designed as the
  surface an already-authorized operator sees.
- **The operator route's existence & operational wiring — `REQ-OPS-*`.** How the segregated route is exposed and
  admitted is operational; this fragment designs only the surface behind it and its *not-in-consumer-nav* placement
  (`REQ-UX-009`).
- **What "remove" / "suspend" mean in storage — `REQ-DOM-*` / `REQ-OPS-*`.** This screen designs the operator intent
  and the user-visible effect (content no longer appears); the persistence/enforcement is not UX.
- **Session lifetime & invalidation — `REQ-ID-*`.** Only the `session_expired` prompt and resume are designed (§3.4.7).
- **Audit trail of operator actions — `REQ-OPS-*`.** Not designed here (kept light).
- **Operator health / durability / recovery duties (`REQ-PROD-030/031/032`) — not a screen,** owned by the
  operational-readiness area; out of this UX fragment.

---

### 4. Requirements register — REQ-UX-062 … REQ-UX-069 (unified)

The eight requirements this fragment owns, drawn from the three screens' disjoint sub-ranges — report/block
`REQ-UX-062..064`, privacy/terms `REQ-UX-065..066`, operator/admin `REQ-UX-067..069` — and here presented as one
register. Each is stated from the user's or operator's side, is testable by a stranger exercising the live product,
carries a `release_gating` flag, and traces to ≥1 `REQ-PROD-*` and/or journey. All eight ids are unique and fall
within `REQ-UX-062..069` (hence within the root `REQ-UX-020..069`); there are no collisions.

##### REQ-UX-062 — A signed-in user can report a chirp or an account and is told it was filed
- **Statement:** From a chirp or a profile, a signed-in user can report the chirp or the account, give a brief reason,
  and receive honest confirmation that the report was filed — without losing what they typed if it fails.
- **Acceptance criteria:** Opening the overflow (`⋯`) on another user's chirp offers *Report chirp* and *Report
  @handle*, and on another user's profile offers *Report @handle*; choosing one opens a form that shows what is being
  reported and lets the user pick a reason (and add detail); submitting it shows a busy state, then a clear "report
  received" confirmation and returns the user to where they were; a submission that fails shows an honest in-place
  error and preserves the chosen reason and typed detail so the user can retry without redoing the form. The
  confirmation claims only that the report was received — it does not promise any moderation outcome.
- **Release gating:** true
- **Traces to:** REQ-PROD-027; REQ-PROD-017; J5; persona P3 and the deliberately-absent moderator.

##### REQ-UX-063 — A signed-in user can reduce their own exposure: block (with stated consequences) and mute
- **Statement:** A signed-in user can block an account behind an explicit confirmation that states, in plain terms,
  what blocking does before it proceeds, and can mute an account directly without a confirmation gate; both reflect the
  resulting state honestly and are reversible.
- **Acceptance criteria:** The overflow on another user's chirp or profile offers *Block @handle* and *Mute @handle*;
  choosing **Block** presents a confirmation that states the consequences for the user before anything happens, with a
  visually distinct destructive confirm and the safe (Cancel) choice focused by default, and only on confirming does
  the block take effect (the account's content leaves the user's view and the overflow then offers *Unblock @handle*);
  choosing **Mute** takes effect immediately with an undoable confirmation and no blocking dialog, and the overflow
  then offers *Unmute @handle*; blocking and muting can each be reversed from the same overflow. The screen presents no
  claim about *who* may perform these actions or how they are enforced.
- **Release gating:** true
- **Traces to:** REQ-PROD-027; REQ-UX-018 (destructive-confirm interaction pattern); J5; persona P3.

##### REQ-UX-064 — Report and block/mute have honest states and never fail silently or falsely succeed
- **Statement:** Every report and block/mute action shows that it is working, confirms success only when it truly
  succeeded, recovers from failure in place without discarding the user's input, and prompts a re-sign-in when the
  session has expired — never a silent failure and never a false success.
- **Acceptance criteria:** Submitting a report or confirming a block shows a busy state that cannot be double-fired; a
  success is shown only when the action completed and its result is observable; a failed submit shows an honest,
  plain-language message with a retry in place and keeps the chosen reason and typed detail; attempting any of these
  actions after the session has expired (including after a service restart) surfaces the standard "session expired —
  sign in again" prompt, preserves the attempt and typed detail, returns the user to the action after signing in, and
  never silently does nothing or appears to succeed.
- **Release gating:** true
- **Traces to:** REQ-PROD-027; REQ-PROD-017; REQ-PROD-006; REQ-PROD-018; J5; persona P3.

##### REQ-UX-065 — Privacy & terms is reachable in both navigation states, without an account
- **Statement:** A visitor can reach the privacy & terms surface whether logged out or signed in, and can read it
  without creating an account or signing in.
- **Acceptance criteria:** A `Privacy & terms` link is present in the **logged-out** footer/chrome and in the
  **signed-in** account navigation **and** in the settings/account-management home; following any of them reaches
  **`SCR-privacy_terms`** at its stable address; the surface renders a readable statement **with no sign-in prompt and
  no gate**; a logged-out visitor who follows the link is **not** required to authenticate.
- **Release gating:** true — REQ-PROD-029 is release-gating; a public product that collects personal data cannot be
  released with this surface unreachable or account-walled.
- **Traces to:** REQ-PROD-029; REQ-PROD-013; REQ-UX-007 (IA-level reachability); J7.

##### REQ-UX-066 — The privacy & terms statement is presented plainly and readably
- **Statement:** The surface presents the statement of data held and terms of use as a plain, legible, structured
  document — not an absent page, a raw dump, or an unreadable wall of text.
- **Acceptance criteria:** `SCR-privacy_terms` shows a titled document with a visible **last-updated/effective date**,
  broken into **named sections with headings**, set in the product's reading typography within the single readable
  content column; body text is legible (comfortable size, line-height, and contrast) and links within it are
  distinguishable by colour **and** underline-on-interaction; the statement is **present and readable** (satisfying
  REQ-PROD-029's "readable rather than absent"), and if the statement is fetched and the fetch fails, the surface shows
  an honest error with a working retry rather than a blank or broken page.
- **Release gating:** true — the readable, present statement is the substance of release-gating REQ-PROD-029; an
  unreadable or absent surface fails it.
- **Traces to:** REQ-PROD-029; J7; builds on the reading typography and content-column identity (visual system
  REQ-UX-071, REQ-UX-074).

##### REQ-UX-067 — An operator can look up a user account by handle or id
- **Statement:** From the operator/support surface, an authorized operator can find a specific user account by entering
  either its handle or its account id.
- **Acceptance criteria:** On `SCR-admin_support`, a visible lookup field accepts a handle (e.g. `@ada`) or an account
  id and, on submit, returns the matching account's support detail; a lookup that matches nothing shows an **empty "no
  account matches"** result (not an error and not a blank area) with the query preserved for correction; an empty or
  malformed query shows an inline validation message at the field without losing the typed text. *(Whether a given
  person may open the surface or run a lookup is an authorization concern owned by identity & trust / operational
  readiness, not tested here.)*
- **Release gating:** true
- **Traces to:** REQ-PROD-033; J8.

##### REQ-UX-068 — The account detail is enough to answer a support request
- **Statement:** For a found account, the operator sees enough about it to answer a common support request, without a
  full personal-data dump.
- **Acceptance criteria:** The support-detail view shows, at minimum, the account's **status** (Active / Signed-out /
  Suspended / Deleted), its handle and display name, its join state, and basic activity (e.g. chirp count, last
  active); a *"my account is gone"* request is answerable by the status reading **Deleted**, and an *"I can't sign in"*
  request by the status reading **Signed-out** (or Active); the status is conveyed by text/icon, not colour alone. The
  detail is scoped to answering support, not a complete data export (the statement of all data held is deferred to
  `REQ-ID-*`).
- **Release gating:** true
- **Traces to:** REQ-PROD-033; J8.

##### REQ-UX-069 — An operator can act on abuse, and the effect becomes visible to users
- **Statement:** The operator can remove an abusive chirp and act on an abusive account, each via an explicit
  consequence-stating confirmation, and the effect is understood to become visible to users (the content no longer
  appears).
- **Acceptance criteria:** From a chirp in the detail view, an operator can choose **Remove chirp** and is shown a
  confirmation that states the consequence (the chirp will no longer appear to anyone) with a non-destructive default
  focus; on confirming, the chirp is shown removed in place (a tombstone) with an honest success confirmation, and a
  failure is reported without any false "removed" state. From the account detail, an operator can **suspend or remove**
  the account via the same confirm-then-act pattern, after which the account's status is shown updated. Destructive
  actions are never applied optimistically and never report success unless they succeeded. *(Whether a given operator
  is permitted to act, and how the removal is enforced in storage, are deferred to identity & trust / operational
  readiness.)*
- **Release gating:** true
- **Traces to:** REQ-PROD-028, REQ-PROD-033; J8.

#### 4.1 Traceability summary (REQ-UX → screen / REQ-PROD / journeys)

| REQ-UX | Screen | Gating | Traces to |
|---|---|---|---|
| REQ-UX-062 | SCR-report_block | true | REQ-PROD-027, 017; J5 |
| REQ-UX-063 | SCR-report_block | true | REQ-PROD-027 (+ REQ-UX-018 pattern); J5 |
| REQ-UX-064 | SCR-report_block | true | REQ-PROD-027, 017, 006, 018; J5 |
| REQ-UX-065 | SCR-privacy_terms | true | REQ-PROD-029, 013; REQ-UX-007; J7 |
| REQ-UX-066 | SCR-privacy_terms | true | REQ-PROD-029; J7 (+ visual system REQ-UX-071, 074) |
| REQ-UX-067 | SCR-admin_support | true | REQ-PROD-033; J8 |
| REQ-UX-068 | SCR-admin_support | true | REQ-PROD-033; J8 |
| REQ-UX-069 | SCR-admin_support | true | REQ-PROD-028, 033; J8 |

All eight ids are unique, in-range (REQ-UX-062..069), user/operator-side and testable against the live product,
release-gating-flagged, and traced to ≥1 `REQ-PROD-*` and a journey.

---

### 5. Open component requests & cross-area seams

Everything in this fragment composes from existing visual-system components and IA patterns, with the following
exceptions **surfaced for resolution, not invented or plugged** ("compose, don't invent"). None of these blocks release: each screen ships on a composable interim today-shape. These are carried up to
the screens-level and visual-system owners for resolution.

| # | Item | Raised by | Status & recommendation |
|---|---|---|---|
| D1 | **Single-select reason control** (report form) | SCR-report_block | The ideal control is a single-select option list / radio group; the visual system defines inputs, a textarea, a boolean toggle, and a segmented control, but **no radio-group/option-list component**. Interim composable shape: the **segmented control (§3.15)** for a short (≤4) reason set, used here so the screen is releasable now. **Recommend** adding a single-select option-list component to the visual system, or confirming the segmented control as the sanctioned shape for short single-selects. Non-blocking. |
| D2 | **Mute glyph** | SCR-report_block | The icon inventory lists `report/flag` and `block` but **no mute glyph**. **Requested** as an addition so the mute item is not icon-less. Non-blocking. |
| D3 | **Blocked/muted accounts management list** | SCR-report_block | A dedicated list for bulk management is a plausible **account-settings sibling surface**. Reversal is already reachable from the overflow, so this screen is complete without it; **noted** as a possible sibling, **not** invented here. Out of this design's scope. |
| D4 | **Long-document table-of-contents / scroll-spy** | SCR-privacy_terms | The "On this page" index needs a way to list and anchor to sections. Anchored `font.h2` headings + the static "On this page" list meet the readable bar now; a persistent scroll-tracking contents nav is **absent** from the visual system. **Flagged** to add via amendment if a longer statement warrants it. Enhancement, non-blocking. |
| D5 | **Consequence-copy ↔ enforcement reconciliation** (cross-area seam) | SCR-report_block | The user-facing consequence copy for **Block** and **Mute** states the meaning of these actions; its wording MUST be reconciled at build with the authoritative enforcement semantics owned by identity & trust (`REQ-ID-*`) so a stated consequence matches what the system actually does (honesty: a state never promises an effect the system does not deliver). This is a **reconciliation**, not a component gap or an invention — recorded as a cross-area seam. |

`SCR-admin_support` flagged **no** component gaps: every element and state maps onto an existing component (§3.5).

---

*Scope boundary held throughout: design intent and the testable bar only — **what, not how**. No front-end code; no
authorization, enforcement, or moderation model (deferred to `REQ-ID-*`/`REQ-OPS-*`); no server-side meaning of
block/mute/remove/suspend; and no legal/privacy copy, data-category list, or effective-date value (deferred to
`REQ-ID-*`). All three screens compose from the shared visual system and IA/interaction model and instantiate the one
shared state model; nothing was silently invented.*

---

## 3. Visual language & design system

*(section: `visual_language_and_design_system`)*

### Visual language & design system

*(section: `visual_language_and_design_system`)*

#### What this section settles, and what it leaves to its siblings

This section defines **the raw material every chirp screen is built from**: a set of **design tokens**
(type, color, spacing, radius, elevation, layout grid, iconography, motion) and a **component library**
(the reusable building blocks), together with the **usage and consistency rules** that keep the product
looking like one intentional product rather than a set of unrelated screens. Today the product has none
of this — all styling is a handful of inline rules (`system-ui` font, a single accent `#1D9BF0`, a 40rem
column, underlined links) with class hooks in the markup but no visual language behind them. A product
with no design system is judged as unfinished in the first minute; this section is the floor that
prevents that.

It is **design intent and values, not implementation**. Tokens are given as named values a later implementation maps to CSS custom properties; components are given as anatomy + states + rules, not as markup or CSS.
"How" (the framework, the stylesheet, the class names) is deferred.

**Boundaries to sibling sections** (so nothing is designed twice or left in a seam):

- **Screen flows, screen-level copy, and which state each screen shows** belong to the **screens & flows**
  section. This section provides the *components and state blocks*; the screens *use* them.
- **Breakpoints, reflow, touch-target exact sizes, and per-screen responsive behaviour** belong to the
  **responsive layout** section. This section provides *static* layout tokens (content column width,
  gutter, grid) and a target minimum touch size; it does **not** define how layouts reflow.
- **Accessibility conformance** (WCAG level, keyboard semantics, ARIA, the actual contrast-ratio audit)
  belongs to the **accessibility** section. This section **declares the contrast and focus *intent*** for
  every token and component and flags each pairing the accessibility section must verify (see §6).
- **Interaction patterns** (auth-gating, optimistic update + reconciliation, confirmation flow) belong to
  the **IA & interaction** section. This section provides the *visual states* those patterns switch
  between (e.g. follow → following → busy; the confirm dialog).

**Grounding.** Grounded in `Nousix-LLC/nswe-demo` at `c805525` (the current SPA:
timeline, compose+sign-in, profile; handle-only login; minimal CSS) and in the product requirements
(journeys J1–J8, register `REQ-PROD-001..034`).

---

#### 1. Design principles (the voice tokens and components serve)

chirp is a **public, text-first consumer microblogging product**. The visual language follows five
principles, and every token and component below is justified by one of them:

1. **Content is the hero.** The product's substance is short text. Typography and generous whitespace
   carry the design; chrome recedes. The reading column is calm and uncluttered.
2. **One confident accent.** A single brand blue signals action and identity. Accent is spent on *the*
   action, not sprinkled; a screen has at most one primary action.
3. **Honest about every state.** Loading, empty, error, not-found, end-of-list and session-expired are
   first-class designed blocks, not afterthoughts — the product is judged in its first minute, often on
   an empty or failing screen.
4. **Trustworthy and legible.** Comfortable contrast, clear hierarchy, predictable controls. Status is
   never carried by colour alone. Destructive actions look and feel distinct.
5. **Consistent by construction.** Screens assemble from this library; they do not invent one-off
   components or states. Consistency is the bar every screen is held to (§5).

---

#### 2. Design tokens

Tokens are named semantically so a theme is a value remap, not a redesign (see §2.9). Raw values are
given for the default (light) theme. The root type size is **16px = 1rem**.

##### 2.1 Colour

**Neutral ramp** (surfaces, text, borders):

| Token | Value | Primary use | Contrast intent (verify: accessibility section) |
|---|---|---|---|
| `color.bg` | `#FFFFFF` | App background / card surface | base surface |
| `color.bg.subtle` | `#F7F9FA` | Row hover, inset areas, skeleton base | — |
| `color.border.subtle` | `#EFF3F4` | Dividers between chirps/rows | ≥ 3:1 vs `bg` not required (decorative divider) |
| `color.border` | `#CFD9DE` | Input borders, card outlines, control outlines | ≥ 3:1 vs `bg` (UI boundary) — **verify** |
| `color.text.tertiary` | `#8899A6` | Placeholders, disabled text, de-emphasised icons | **Exempt from 4.5:1 by role** (placeholder/disabled); MUST NOT carry essential text — **verify role usage** |
| `color.text.secondary` | `#536471` | Meta: handles-as-secondary, timestamps, counts, helper text | intended ≥ 4.5:1 vs `bg` (normal text) — **verify** |
| `color.text.primary` | `#0F1419` | Body text, display names, titles | intended ≥ 12:1 vs `bg` |
| `color.text.inverse` | `#FFFFFF` | Text on accent/semantic fills | pair-verified per fill below |

**Accent ramp** (brand + action). The brand base `#1D9BF0` keeps the product's heritage but **fails
normal-text contrast on white**, so the system splits accent into a *base* (for ≥3:1 UI uses) and a
*strong* (wherever accent carries or is text):

| Token | Value | Use | Contrast intent (verify: accessibility section) |
|---|---|---|---|
| `color.accent.subtle` | `#E8F5FE` | Selected/active row tint, info-banner background | background only |
| `color.accent.base` | `#1D9BF0` | Active nav indicator/underline, focus ring, decorative/large icon, brand mark | intended ≥ 3:1 vs `bg` (UI/large only) — **verify** |
| `color.accent.hover` | `#1A7FC9` | Hover on strong accent fills/links | — |
| `color.accent.strong` | `#1565A0` | **Primary-button fill (with `text.inverse`), text links, accent text** | intended ≥ 4.5:1 both vs `bg` and for inverse-on-fill — **verify** |
| `color.accent.pressed` | `#114E7D` | Active/pressed primary fill | ≥ 4.5:1 for inverse-on-fill — **verify** |

**Semantic ramp** (status). Each has a *background* (banners), a *base* (fills/large icons), and an
*ink* (text/small icon) chosen so status text passes on white:

| Token group | background | base (fill/icon) | ink (text/small icon) | Use |
|---|---|---|---|---|
| success | `color.success.bg` `#E6F4EC` | `color.success.base` `#1A8754` | `color.success.ink` `#0E6B41` | "Chirp posted", "Profile saved", validated field |
| error | `color.error.bg` `#FCEAEC` | `color.error.base` `#D7263D` | `color.error.ink` `#A51C2B` | Failed submit, validation error, **destructive fill** |
| warning | `color.warning.bg` `#FBF1E3` | `color.warning.base` `#E8820E` | `color.warning.ink` `#8A4B00` | Approaching limit, cautionary confirmation |
| info | `color.accent.subtle` | `color.accent.base` | `color.accent.strong` | Neutral informational notices |

`*.ink` tokens are the status **text** colour (intended ≥ 4.5:1 on `bg` — **verify**); `*.base` is for
fills and large/decorative icons (intended ≥ 3:1 — **verify**); `*.bg` is background only. **Rule:
status is always paired with an icon and/or text label, never carried by colour alone** (REQ-UX-073).

`color.scrim` = `#0F1419` at **40% opacity**, for modal/overlay backdrops.

##### 2.2 Typography

**Families** (no web-font dependency — honest and fast for this product):

- `font.family.sans` = system UI sans stack (`system-ui` and the platform equivalents) — all product text.
- `font.family.mono` = system monospace stack — reserved for machine values in the admin/support and
  data-export surfaces (ids, export file names). Not used in normal content.

**Weights:** `font.weight.regular` 400 · `medium` 500 · `semibold` 600 · `bold` 700.

**Type scale** (step scale; `rem` with px reference at 16px root):

| Token | Size | Line-height | Weight | Use |
|---|---|---|---|---|
| `font.display` | 1.75rem / 28px | 1.2 | bold | Page/landing title, empty-state title |
| `font.h1` | 1.375rem / 22px | 1.25 | bold | Screen title (profile name header, settings title) |
| `font.h2` | 1.125rem / 18px | 1.3 | semibold | Section heading, dialog title |
| `font.body.lg` | 1.0625rem / 17px | 1.45 | regular | Chirp detail (thread root) body |
| `font.body` | 1rem / 16px | 1.45 | regular | **Chirp body text**, form field text, general copy |
| `font.label` | 0.9375rem / 15px | 1.3 | semibold | Display name, button label, field label, nav item |
| `font.meta` | 0.8125rem / 13px | 1.3 | regular | `@handle`, timestamp, counts, helper text |
| `font.caption` | 0.75rem / 12px | 1.3 | medium | Badges, character counter, fine print |

**Rules:** chirp body uses `font.body` with comfortable line-height and preserves the author's line
breaks; display name (`font.label`, `text.primary`) and `@handle` (`font.meta`, `text.secondary`) are a
fixed pairing everywhere (see the identity cluster, §3.5); never set essential text below `font.meta`
size in `text.secondary`.

##### 2.3 Spacing

4px base unit; a single scale used for all padding, gaps and margins (no arbitrary values):

`space.0` 0 · `space.1` 4px · `space.2` 8px · `space.3` 12px · `space.4` 16px · `space.5` 20px ·
`space.6` 24px · `space.8` 32px · `space.10` 40px · `space.12` 48px · `space.16` 64px.

Defaults: card/dialog padding `space.4`; stack gap between chirps `space.0` (divider-separated) with
`space.4` internal padding; form field vertical gap `space.4`; inline control gap `space.2`.

##### 2.4 Radius

`radius.sm` 6px (inputs, menus, inline alerts) · `radius.md` 12px (cards, dialogs, state blocks) ·
`radius.lg` 16px (large surfaces) · `radius.pill` 999px (buttons, follow/like controls, chips) ·
`radius.full` 50% (avatars). Buttons are pill-shaped; cards and dialogs are `radius.md`.

##### 2.5 Elevation & overlay

Expressed as intent (soft, neutral, low-opacity shadows); exact shadow values are the implementation's to map.

| Token | Intent | Used by |
|---|---|---|
| `elevation.0` | Flat; separated by `border`/divider only | Cards, rows, inline content |
| `elevation.1` | Small soft shadow | Dropdown / overflow / account menus, popovers |
| `elevation.2` | Medium shadow + `color.scrim` backdrop | Modals / dialogs |
| `elevation.3` | Soft shadow, no backdrop | Toasts |

##### 2.6 Layout grid & content column (static tokens; reflow deferred to responsive)

- `layout.content.max` = **600px** — the single readable column for timeline, compose, profile, chirp
  thread, and settings forms (a deliberate widening from today's 40rem for the card layout).
- `layout.wide.max` = **960px** — for data-dense operator/admin tables only.
- `layout.gutter` = `space.4` (16px) page side padding.
- `layout.touch.min` = **44px** — target minimum interactive size (the *value*; exact enforcement and
  per-screen application are the responsive layout / accessibility sections' concern).

How the nav and these columns rearrange across phone/tablet/desktop is **not** defined here — that is
the responsive layout section. This section only fixes the column/gutter/target values it consumes.

##### 2.7 Iconography

- One **single open stroke icon set**, consistent weight (~1.75px stroke at 24px), rounded joins.
- Sizes: `icon.sm` 16px (inline with text), `icon.md` 20px (dense controls), `icon.lg` 24px (default
  nav/action). Icons inherit text colour (`currentColor`) so they obey the colour tokens automatically.
- **Required glyph inventory** (the screens depend on these existing in the set): home, profile/person,
  settings/gear, compose (new chirp), like-outline, like-filled, follow/person-add, search, overflow
  (`⋯`), close, back/chevron-left, chevron-right, error/alert-triangle, success/check-circle, info,
  warning, external-link, sign-out, delete/trash, download/export, report/flag, block, link/copy.
- Icons are either decorative (hidden from assistive tech) or labelled; **every icon-only control carries
  an accessible name** — this section states the requirement; the accessibility section verifies it.

##### 2.8 Motion

`motion.fast` 120ms (hover, focus, small state changes) · `motion.base` 200ms (toasts in/out, menu open,
optimistic count change) · `motion.slow` 320ms (modal in/out). Standard ease-in-out. Motion is
supportive, never blocking. **All motion must honour a reduced-motion preference** (states change
instantly when reduced motion is requested) — stated here; the accessibility section verifies.

##### 2.9 Theming note (forward-compatible, non-gating)

Because tokens are **semantic** (`color.text.primary`, not "near-black"), a dark theme is a value remap
of the same token names with no component changes. A dark theme is a real improvement but an honest cut
for the first releasable version — captured as the non-gating REQ-UX-089.

---

#### 3. Component library

Every component is given as **purpose · variants · anatomy · states · tokens · usage**. Components are
built only from §2 tokens — no raw values. Where a component already has a class hook in today's markup
(e.g. `chirp`, `chirp-author`, `chirp-follow-button`, `banner`), the system *re-skins the existing hook*
rather than inventing parallel markup — a migration convenience for the implementation, not a design
dependency.

##### 3.0 The shared interaction-state model

Every interactive component supports this one state set, so behaviour is predictable across the product:

- **default** — resting.
- **hover** — pointer over (subtle `bg.subtle` or darkened fill).
- **active / pressed** — during press (`accent.pressed` for primary; darker fill otherwise).
- **focus-visible** — a **2px `accent.base` focus ring** with a 2px offset, on *every* interactive
  element reachable by keyboard. (Appearance is defined here; conformance is the accessibility section's.)
- **disabled** — reduced emphasis (`text.tertiary`, no hover), not focusable as an action.
- **busy** (submit-capable controls only) — shows a spinner, is non-interactive, and **preserves its
  size** so layout does not jump.

##### 3.1 Button

- **Purpose:** trigger an action.
- **Variants:** `primary` (fill `accent.strong`, `text.inverse`) · `secondary` (outline `border`,
  `text.primary`, transparent fill) · `tertiary/text` (no border, `accent.strong` label) ·
  `destructive` (fill `error.base` + `text.inverse`, or outline `error.ink` for lower-emphasis) ·
  `icon-button` (square-ish, pill/round, icon-only, requires accessible name).
- **Sizes:** `sm` (compact, inline), `md` (default), `lg` (primary page actions). Pill radius.
- **States:** the full §3.0 set, including `busy` (label replaced/augmented by spinner, width held).
- **Usage:** **at most one `primary` per screen** — the screen's main action (post, sign in, create
  account, save). `destructive` is reserved for genuinely destructive actions and is **always confirmed
  by a dialog** (§3.9); it is never the default-focused control in that dialog. Min size target
  `layout.touch.min`.

##### 3.2 Link

- Text link: `accent.strong`, underline on hover/focus (resting underline optional for inline links in
  body copy for scannability). Standalone navigational links use `font.label`. Distinguishable from
  body text by colour **and** underline-on-interaction (not colour alone). Replaces today's always-
  underlined `#1D9BF0` link with the contrast-correct `accent.strong`.

##### 3.3 Text input, textarea & field group

- **Purpose:** collect text (handle, display name, bio, compose, search, recovery, report reason).
- **Anatomy:** label (`font.label`) · field (`radius.sm`, `border`, `font.body`) · optional helper text
  (`font.meta`, `text.secondary`) · message slot (error/success).
- **Variants:** single-line input · `textarea` (compose, bio) · search input (with leading search icon
  and clear control).
- **States:** default · focus (ring + `accent.base` border) · filled · disabled · **error** (border
  `error.ink`, message `error.ink` + alert icon) · **success/validated** (border/check `success.ink`
  where confirmation helps, e.g. handle available). Placeholder uses `text.tertiary` and is never the
  only label.
- **Usage:** validation is **inline and actionable, at or before submit** (maps the seam `validation`
  state): handle rules (≤15 chars, allowed characters) and uniqueness ("that handle is taken") shown in
  the message slot in plain language, not a raw API error; compose length enforced via the counter
  (§3.4). Error messages state what to do, not just that something is wrong.

##### 3.4 Character counter

- **Purpose:** show remaining length for bounded fields (compose ≤280, handle ≤15, bio limit).
- **States:** normal (`text.secondary`) · **approaching limit** (`warning.ink`, within the last ~20) ·
  **over limit** (`error.ink`, submit blocked). Reads as remaining count.
- **Usage:** always paired with the field it governs; the over-limit state disables the associated
  submit button and surfaces the error without losing typed text (draft safety, REQ-PROD-017).

##### 3.5 Avatar & identity cluster

- **Avatar:** circular (`radius.full`), sizes `sm` 32 · `md` 40 · `lg` 48 · `xl` 64 (profile header).
  chirp has no image upload today, so the **default is a generated monogram placeholder** (initial on a
  neutral/accent-tinted disc, deterministic per handle) — honest about the absence of uploads rather
  than showing a broken image.
- **Identity cluster:** a fixed pairing used everywhere a user is shown — **display name**
  (`font.label`, `text.primary`) + **`@handle`** (`font.meta`, `text.secondary`), optional **"you"
  badge** on the current user's own content, and (future) a verified marker slot. Reused in the chirp
  card, profile header, nav account menu, menus, and mentions so a user looks the same across the
  product.

##### 3.6 Chirp card

- **Purpose:** the core content unit — one chirp. Used by timeline, profile, thread, search results.
- **Anatomy:** avatar (`sm`/`md`) · identity cluster · relative timestamp (`font.meta`; absolute on
  hover/title) · overflow menu trigger (report/block, or delete on own chirp) · body text (`font.body`,
  preserves line breaks; `@mentions`/links styled as links — whether they navigate is IA/screens) ·
  action row (like control with count; reply affordance **only if** replies are built per REQ-PROD-023 —
  never a dead control).
- **Variants:** `default` (list) · `detail` (thread root — larger body `font.body.lg`, fuller meta) ·
  `tombstone` (deleted/not-found chirp: muted placeholder "This chirp is unavailable", no actions).
- **States:** default · hover (row `bg.subtle` when the whole card is a target) · focus-visible (when
  the card is itself navigable).
- **Usage:** one card component only — no screen hand-rolls a chirp row. Density (padding, avatar size)
  is consistent within a list.

##### 3.7 Follow control

- **Purpose:** follow/unfollow an account; reflect the true resulting state (REQ-PROD-020).
- **States:** `Follow` (primary or `accent.strong` outline) · `Following` (secondary/neutral; on hover
  reveals the destructive-intent "Unfollow") · **busy** (optimistic; spinner) · **disabled** (own
  profile / logged-out → routes to sign-in). On failure the control **reconciles visibly** back to the
  true state (the reconcile *pattern* is IA's; the control provides the busy→reverted visual).
- **Usage:** label and count always match the real state after the action; a failed follow is reported
  (toast/inline), not silently dropped.

##### 3.8 Like control

- **Purpose:** like/unlike a chirp with a truthful count (REQ-PROD-021).
- **Anatomy:** like icon-button (`like-outline` ↔ `like-filled`) + count (`font.meta`).
- **States:** not-liked (outline, `text.secondary`) · liked (filled, `accent.base` or a dedicated like
  colour — paired with the filled icon so colour is not the sole signal) · busy (optimistic) · disabled
  (logged-out → prompts sign-in). Count formats consistently (e.g. `1,024` → `1K` above a threshold,
  one rule product-wide).
- **Usage:** optimistic toggle that reconciles the icon **and** count on failure; failures reported.

##### 3.9 Navigation (component only; placement/reflow is responsive)

- **Purpose:** the product's primary navigation chrome. Content differs logged-out vs signed-in (the
  *what* belongs to the IA & interaction section); this section provides the **nav component and its
  states**.
- **Anatomy:** brand wordmark/mark · nav items (icon + `font.label`) · the compose CTA (a `primary`
  button when signed in) · the account cluster (avatar → account/overflow menu) when signed in, or the
  **Sign in** + **Create account** CTAs when logged out.
- **Nav item states:** default · hover (`bg.subtle`) · **active** (current section — `accent.base`
  indicator + `text.primary`/medium weight; active is shown by indicator **and** weight, not colour
  alone) · focus-visible · disabled.
- **Usage:** the **sign-out control is always reachable** from the signed-in account menu (closes the
  gap that today has no visible sign-out, REQ-PROD-005); the compose CTA is the nav's single primary.

##### 3.10 Menu (dropdown / overflow / account)

- **Purpose:** a compact list of actions from a trigger (account menu, chirp overflow).
- **Anatomy:** surface (`elevation.1`, `radius.sm`) · menu items (icon + label) · separators ·
  **destructive items** (`error.ink`, e.g. "Delete chirp", "Sign out" styled as neutral-strong).
- **States (per item):** default · hover (`bg.subtle`) · focus-visible · disabled.
- **Usage:** destructive menu items that cause data loss still route through a confirm dialog (§3.11).

##### 3.11 Modal / dialog

- **Purpose:** focused interruption for confirmation or a short form.
- **Variants:** `confirm` (title, body, cancel + primary/destructive action) · `form` (short form, e.g.
  start data export, report reason).
- **Anatomy:** scrim (`color.scrim`) · surface (`elevation.2`, `radius.md`, `layout.content.max`-capped
  width, size `sm`/`md`) · title (`font.h2`) · body · action row (secondary "Cancel" + the primary or
  **destructive** action).
- **States:** open/closed; action buttons carry the §3.0 states incl. busy.
- **Usage:** **every destructive or irreversible action is confirmed here** — account deletion (stating
  what is removed: account, chirps, follows, likes — REQ-PROD-010), delete own chirp, moderation
  removal, and sign-out where appropriate. The destructive action is visually distinct (`destructive`
  button) and is **not** the default-focused control; "Cancel" is the safe default.

##### 3.12 Toast / snackbar

- **Purpose:** transient, non-blocking confirmation or error (`elevation.3`).
- **Variants:** success · error · info (icon + message + optional single action e.g. "Undo"/"Retry" +
  dismiss).
- **Usage:** brief successes ("Chirp posted", "Profile saved", "Export ready", "Link copied"); auto-
  dismiss (`motion.base`) with manual dismiss; errors that are purely transient and non-blocking. For
  errors the user must act on or that block a flow, use the inline alert (§3.13) instead. Toast position
  is the responsive layout section's.

##### 3.13 Inline alert / banner

- **Purpose:** a persistent, in-context message (re-skins today's `banner`).
- **Variants:** error · success · warning · info — each `*.bg` background, `*.ink` text, matching icon,
  optional action (e.g. **Retry** on a fetch error).
- **Usage:** form-level and page-level messages that must stay visible — sign-in failure, compose send
  failure (with draft preserved), a fetch error with retry, a "handle taken" conflict. Always icon +
  text (colour is never the sole carrier).

##### 3.14 State blocks (the honest-states backbone)

One reusable family renders the seam's data states, so **no screen hand-rolls a state**. Each block is a
slotted layout: optional icon/illustration · title (`font.h2`/`font.display` for empty) · supportive
copy (`text.secondary`) · optional action.

| Block | Renders seam state(s) | Anatomy & intent |
|---|---|---|
| **Loading / skeleton** | `loading` | Shape-matched skeletons (chirp skeleton, profile skeleton) on `bg.subtle`; spinner for in-control/`busy_submitting` actions. No layout jump when content arrives. |
| **Empty** | `empty` | Icon + title + supportive copy + a primary action. Variants: logged-out landing (says what chirp is + Create account / Sign in), new-user/quiet timeline (find people / post), profile with no chirps, search no-results, admin no-match. (Exact copy is the screens & flows section's.) |
| **Error** | `error` | Alert icon + honest, plain message + **Retry** that refetches **without a full page reload**. Never a blank or broken screen. |
| **Not-found** | `not_found` | Title + supportive copy + a way back (Home/Back). For unknown handle/id, unknown chirp, deleted account. |
| **End-of-list** | `end_of_list` | Subtle terminal marker on a paged list ("You're all caught up"); distinct from empty. |
| **Session-expired** | `session_expired` | A clear re-sign-in prompt (inline on the attempted action or an interstitial) — never a silent fail or false success (REQ-PROD-006). Block provided here; where/when it fires is IA's. |

`content`, `validation`, `busy_submitting`, and `success` states are rendered by the relevant primary
component (card/list, inputs+counter, the §3.0 busy state, toast/inline success) rather than a standalone
block — mapped explicitly in §4.

##### 3.15 Supporting components

- **Settings row / list:** a navigable row (label + optional description + trailing chevron or toggle)
  for the settings/account-management home gathering profile edit, data export, deletion, sign-out,
  privacy/terms. States per §3.0. **Toggle/switch** sub-component for boolean settings (on/off/disabled).
- **Tabs / segmented control:** for within-screen sectioning (e.g. profile sections, settings groups).
  States: default/hover/active(`accent.base` indicator + weight)/focus/disabled.
- **Badge / count pill:** small count or label ("you", unread/new marker, like/follower counts) —
  `font.caption`, `radius.pill`. One count-formatting rule product-wide.
- **Progress / spinner:** indeterminate spinner (default) for loads and `busy` controls; an optional
  determinate bar for data-export generation.
- **Divider:** `border.subtle`, separating chirps/rows and dialog sections.
- **Scrim / overlay:** `color.scrim` backdrop behind modals and any future drawer.

---

#### 4. State vocabulary → component mapping

Every state in the state vocabulary has a defined rendering here, so screen coverage and completeness can be checked mechanically:

| Seam state | Rendered by |
|---|---|
| `loading` | State block: Loading/skeleton (§3.14); in-control spinner for actions |
| `content` | The relevant component (chirp card/list §3.6, profile header §3.5, etc.) |
| `empty` | State block: Empty (§3.14) |
| `error` | State block: Error with Retry (§3.14) for page/section; Inline alert (§3.13) for form/action |
| `not_found` | State block: Not-found (§3.14); chirp tombstone variant (§3.6) |
| `end_of_list` | State block: End-of-list (§3.14) |
| `session_expired` | State block: Session-expired prompt (§3.14) |
| `validation` | Input/textarea error/validated states + Character counter (§3.3, §3.4) |
| `busy_submitting` | §3.0 busy state on the submitting control (button/follow/like) |
| `success` | Toast (§3.12) or inline success alert (§3.13); optimistic control reconciled |

---

#### 5. Usage & consistency rules — the bar every screen is held to

A screen is consistent with this system when all of the following hold. This is the checklist the
screens & flows section can hold any screen to (the component-consistency requirement):

1. **Tokens, not raw values.** Every colour, size, space, radius, and shadow on the screen comes from a
   §2 token; no ad-hoc hex/px.
2. **Components, not one-offs.** Every element is an instance of a §3 component; a screen that needs
   something new extends the system here rather than inventing a local component.
3. **States are system blocks.** Every data-fetching surface renders the Loading, Empty, and Error
   blocks from §3.14 (plus Not-found / End-of-list / Session-expired where the seam's state matrix
   marks them applicable) — no bespoke per-screen states.
4. **One primary action per screen**, styled `primary`; secondary actions styled `secondary`/`tertiary`.
5. **Destructive = destructive style + dialog confirm**, with a non-destructive default focus.
6. **Every interactive element has the five states** (default/hover/active/focus-visible/disabled) and
   submit-capable controls also have **busy**; focus-visible is never removed.
7. **Submission pattern** is uniform: busy → success (toast/inline) or error (inline, recoverable,
   **draft/text preserved**); optimistic controls reconcile visibly on failure.
8. **Status is never colour-only** — always paired with icon and/or text.
9. **Identity is one cluster** (display name + `@handle`, §3.5) wherever a user appears.
10. **Icon-only controls carry an accessible name**; timestamps are relative with absolute on hover.
11. **Repository-ready hygiene** on all design content (no host/system detail; named by
    repository identity).

---

#### 6. Contrast & colour intent handoff to the accessibility section

This section **declares intent and flags what to verify**; the accessibility section owns the audit
against its stated WCAG level. Items to verify:

- **Normal-text ≥ 4.5:1 on `color.bg`:** `text.primary`, `text.secondary`, every `*.ink`
  (`accent.strong`, `success.ink`, `error.ink`, `warning.ink`). Confirm each ramp value holds the ratio
  as specified; darken the token if it does not (the token name stays).
- **Inverse-on-fill ≥ 4.5:1:** `text.inverse` on `accent.strong` (primary buttons), on `error.base`
  (destructive), on `success.base`, `warning.base` where used with text.
- **UI/boundary & large ≥ 3:1:** `color.border`, `accent.base` as active indicator / focus ring, large
  or decorative icons, status `*.base` fills.
- **Role-exempt but check usage:** `text.tertiary` (placeholder/disabled only) MUST NOT be the sole
  carrier of essential text anywhere.
- **Not colour-alone:** verify every status/active signal is also carried by icon/text/weight (§3.8,
  §3.9, §3.13).
- **Focus & motion:** verify the §3.0 focus-visible ring meets non-text-contrast and visibility
  requirements, and that `motion.*` respects reduced-motion.

---

#### 7. Requirements & acceptance criteria — REQ-UX-070 .. REQ-UX-089

Composes against `req_ux_schema` (id · statement · acceptance_criteria · release_gating · traces_to) and
the parent id scheme. Acceptance criteria are written so a stranger can exercise the live product and
reach the same yes/no. `traces_to` references `REQ-PROD-*` and/or journeys `Jn`; some also note the
accessibility section, which verifies the stated intent.

###### REQ-UX-070 — One consistent visual language across the product
- **Statement:** Every screen uses the same typography, colour, spacing, and component shapes, so chirp
  reads as a single intentional product.
- **Acceptance criteria:** Moving across the timeline, a profile, compose, settings, sign-in, and at
  least one dialog, a person sees the same type styles, the same accent, the same pill buttons, the same
  card, and the same spacing rhythm; no screen looks like it belongs to a different product or uses an
  off-system colour/control.
- **Release gating:** true
- **Traceability:** REQ-PROD-014; J1, J5, J6.

###### REQ-UX-071 — Readable typographic hierarchy
- **Statement:** Text has a clear hierarchy and chirp body copy is comfortable to read.
- **Acceptance criteria:** On any screen, titles, labels, body, and meta are visually distinguishable
  (size/weight); chirp body text is at least 16px with comfortable line spacing and preserves the
  author's line breaks.
- **Release gating:** true
- **Traceability:** REQ-PROD-016, REQ-PROD-018; J5.

###### REQ-UX-072 — Colour system with sufficient contrast
- **Statement:** Text and interface colours meet the product's stated contrast intent.
- **Acceptance criteria:** Body and meta text, link/accent text, and status text are legible against
  their backgrounds and meet the intended ratios (AA normal text 4.5:1; UI/large 3:1) as confirmed by
  the accessibility audit; no primary content relies on a colour pairing below its intended ratio.
- **Release gating:** true
- **Traceability:** REQ-PROD-014; J1. Verified by the accessibility section.

###### REQ-UX-073 — Status is never carried by colour alone
- **Statement:** Every status or state signal is conveyed by more than colour.
- **Acceptance criteria:** Error, success, warning, liked, following, and active-nav signals are each
  accompanied by an icon, text, or weight change, so a person who cannot distinguish the colours can
  still tell the state.
- **Release gating:** true
- **Traceability:** REQ-PROD-018, REQ-PROD-020, REQ-PROD-021. Verified by the accessibility section.

###### REQ-UX-074 — Consistent spacing and a readable content column
- **Statement:** Spacing follows one scale and primary content sits in a single readable column.
- **Acceptance criteria:** Padding and gaps across screens are visibly consistent (one rhythm), and the
  timeline/compose/profile/settings content sits in a single centred column rather than stretching full
  width on a wide display.
- **Release gating:** true
- **Traceability:** REQ-PROD-014; J1, J5.

###### REQ-UX-075 — A button system with full interaction states
- **Statement:** Buttons share one system (primary/secondary/tertiary/destructive) and respond to
  interaction.
- **Acceptance criteria:** A person can see distinct primary, secondary, and destructive buttons;
  hovering, pressing, and keyboard-focusing a button each produce a visible change; a disabled button is
  visibly inactive; and a submitting button shows a busy indicator without the layout jumping.
- **Release gating:** true
- **Traceability:** REQ-PROD-016, REQ-PROD-010; J5, J7.

###### REQ-UX-076 — Destructive actions are distinct and confirmed
- **Statement:** Actions that destroy data look destructive and require confirmation.
- **Acceptance criteria:** Initiating account deletion (or deleting a chirp / removing content) presents
  a visually distinct destructive control and a confirmation dialog stating what will be removed; the
  dialog's default focus is the safe (cancel) choice, not the destructive one.
- **Release gating:** true
- **Traceability:** REQ-PROD-010, REQ-PROD-027, REQ-PROD-028; J7.

###### REQ-UX-077 — Inputs with inline validation styling
- **Statement:** Form fields show validation inline, in language the user can act on.
- **Acceptance criteria:** Entering an invalid handle (too long / disallowed characters) or a taken
  handle shows a clear inline error on the field in plain language at or before submit; a valid/available
  entry is accepted (and may show a validated state); the message does not require reading a raw API
  error.
- **Release gating:** true
- **Traceability:** REQ-PROD-002, REQ-PROD-003, REQ-PROD-008, REQ-PROD-016; J2, J6.

###### REQ-UX-078 — Character-limit feedback that never loses the draft
- **Statement:** Length-limited fields show remaining characters and prevent over-limit submission
  without discarding typed text.
- **Acceptance criteria:** Typing in compose shows remaining characters, warns as the 280 limit nears,
  and blocks submission over the limit; the handle field reflects its 15-character limit; in all cases
  the user's typed text is preserved when blocked or on a failed send.
- **Release gating:** true
- **Traceability:** REQ-PROD-016, REQ-PROD-017, REQ-PROD-002; J2, J5.

###### REQ-UX-079 — One consistent chirp card everywhere
- **Statement:** A chirp is presented by the same card component wherever it appears.
- **Acceptance criteria:** A chirp shown in the timeline, on a profile, in a thread, and in search
  results has the same structure — avatar, display name, `@handle`, timestamp, body, and like control —
  with no screen rendering a visibly different chirp layout.
- **Release gating:** true
- **Traceability:** REQ-PROD-016, REQ-PROD-018, REQ-PROD-019; J5.

###### REQ-UX-080 — Consistent identity presentation with a placeholder avatar
- **Statement:** A user is always shown as display name + `@handle`, with a sensible avatar placeholder
  when there is no image.
- **Acceptance criteria:** Everywhere a user appears (card, profile header, nav, menus), the display
  name and `@handle` are shown in the same pairing; an account with no uploaded image shows a consistent
  generated monogram placeholder rather than a broken or empty image.
- **Release gating:** true
- **Traceability:** REQ-PROD-008, REQ-PROD-019; J5, J6.

###### REQ-UX-081 — Follow control reflects the true state
- **Statement:** The follow control clearly shows follow / following / in-progress and matches reality.
- **Acceptance criteria:** From a profile, the control reads "Follow" when not following and "Following"
  when following; activating it shows an in-progress state and settles on the real resulting state and
  count; a failed follow/unfollow is reported and the control returns to the true state rather than
  showing a false one.
- **Release gating:** true
- **Traceability:** REQ-PROD-020; J5, J6.

###### REQ-UX-082 — Like control reflects the true state and count
- **Statement:** The like control clearly shows liked / not-liked and a truthful count.
- **Acceptance criteria:** Liking a chirp changes the control to a filled/liked appearance and updates
  the count; unliking reverts both; a failed like/unlike is reported and the control and count return to
  the real value; the liked state is distinguishable without relying on colour alone.
- **Release gating:** true
- **Traceability:** REQ-PROD-021; J5.

###### REQ-UX-083 — Standard state blocks used consistently
- **Statement:** Loading, empty, error, not-found, and end-of-list are rendered by the shared state
  blocks on every screen where they apply.
- **Acceptance criteria:** On every data-fetching surface (timeline, profile, thread, search, export,
  admin lookup), the loading, empty, and error presentations — plus not-found and end-of-list where
  applicable — look and behave consistently and are recognisably the same blocks across screens, not
  one-off per-screen treatments.
- **Release gating:** true
- **Traceability:** REQ-PROD-014, REQ-PROD-018, REQ-PROD-019; J1, J5.

###### REQ-UX-084 — Honest, recoverable error state with retry
- **Statement:** A failed load or action shows an honest message and a way to recover without reloading
  the whole page.
- **Acceptance criteria:** When a fetch fails, the screen shows a clear message (not a blank or broken
  page) and a Retry that refetches in place; after the cause clears, Retry restores the content without a
  full page reload.
- **Release gating:** true
- **Traceability:** REQ-PROD-018, REQ-PROD-019; J1, J5.

###### REQ-UX-085 — Uniform submission feedback
- **Statement:** Every submission gives consistent busy, success, and failure feedback.
- **Acceptance criteria:** Posting a chirp, saving a profile, following, liking, requesting an export,
  and confirming a deletion each show a busy state during the action and resolve to a clear success
  (toast or inline) or an honest, recoverable failure — with the same feedback pattern across the
  product and no silent success or silent failure.
- **Release gating:** true
- **Traceability:** REQ-PROD-016, REQ-PROD-017, REQ-PROD-020, REQ-PROD-021; J5.

###### REQ-UX-086 — Standard feedback surfaces (dialog, toast, inline alert)
- **Statement:** The product uses one dialog, one toast, and one inline-alert pattern for interruptions,
  transient feedback, and persistent in-context messages.
- **Acceptance criteria:** Confirmations appear as a consistent centred dialog with a scrim; brief
  successes appear as a consistent transient toast; persistent in-context messages (sign-in error,
  compose send failure, handle conflict) appear as a consistent inline alert; the same situation type
  uses the same surface everywhere.
- **Release gating:** true
- **Traceability:** REQ-PROD-010, REQ-PROD-016, REQ-PROD-027; J5, J7.

###### REQ-UX-087 — Consistent iconography with labelled icon-only controls
- **Statement:** Icons come from one consistent set and icon-only controls are always identifiable.
- **Acceptance criteria:** Icons across the product share a consistent style and size system and take
  their colour from their context; every control that shows only an icon (overflow, like, close, sign-out,
  search) exposes an accessible name so its purpose is unambiguous.
- **Release gating:** true
- **Traceability:** REQ-PROD-005, REQ-PROD-027; J3, J5. Icon-name exposure verified by the
  accessibility section.

###### REQ-UX-088 — Visible focus on every interactive element
- **Statement:** Every interactive element shows a clear focus indicator when reached by keyboard.
- **Acceptance criteria:** Tabbing through any screen moves a visible focus ring across every actionable
  element (links, buttons, inputs, menu items, cards-as-targets); no interactive element is focusable
  without a visible indicator.
- **Release gating:** true
- **Traceability:** REQ-PROD-005, REQ-PROD-009; J3, J6. Keyboard/focus conformance owned by the
  accessibility section.

###### REQ-UX-089 — Optional dark theme via semantic tokens
- **Statement:** The design supports an optional dark theme as a token remap, without restructuring
  components.
- **Acceptance criteria:** Where a dark theme is offered, switching it changes background, surface, text,
  border, and accent to their dark values across the whole product consistently, with the same components
  and contrast intent; where it is not offered in the first release, the semantic token structure still
  makes it a value-only change later.
- **Release gating:** false — a dark theme is a real improvement but an honest cut for the first
  releasable version; the semantic tokens keep it a cheap later addition.
- **Traceability:** Consumer-web-app expectation (theme choice); REQ-PROD-014 (polish of the first
  impression). Recorded cut, not an omission.

---

#### 8. Traceability summary (REQ-UX → REQ-PROD / journeys)

| REQ-UX | Gating | Traces to |
|---|---|---|
| REQ-UX-070 | true | REQ-PROD-014; J1, J5, J6 |
| REQ-UX-071 | true | REQ-PROD-016, 018; J5 |
| REQ-UX-072 | true | REQ-PROD-014; J1 |
| REQ-UX-073 | true | REQ-PROD-018, 020, 021 |
| REQ-UX-074 | true | REQ-PROD-014; J1, J5 |
| REQ-UX-075 | true | REQ-PROD-016, 010; J5, J7 |
| REQ-UX-076 | true | REQ-PROD-010, 027, 028; J7 |
| REQ-UX-077 | true | REQ-PROD-002, 003, 008, 016; J2, J6 |
| REQ-UX-078 | true | REQ-PROD-016, 017, 002; J2, J5 |
| REQ-UX-079 | true | REQ-PROD-016, 018, 019; J5 |
| REQ-UX-080 | true | REQ-PROD-008, 019; J5, J6 |
| REQ-UX-081 | true | REQ-PROD-020; J5, J6 |
| REQ-UX-082 | true | REQ-PROD-021; J5 |
| REQ-UX-083 | true | REQ-PROD-014, 018, 019; J1, J5 |
| REQ-UX-084 | true | REQ-PROD-018, 019; J1, J5 |
| REQ-UX-085 | true | REQ-PROD-016, 017, 020, 021; J5 |
| REQ-UX-086 | true | REQ-PROD-010, 016, 027; J5, J7 |
| REQ-UX-087 | true | REQ-PROD-005, 027; J3, J5 |
| REQ-UX-088 | true | REQ-PROD-005, 009; J3, J6 |
| REQ-UX-089 | false | Consumer-web-app theme expectation; REQ-PROD-014 |

---

## 4. Responsive layout

*(section: `responsive_layout`)*

### Responsive layout

*(section: `responsive_layout`)*

#### What this section settles, and what it leaves to its siblings

This section defines **how chirp behaves across screen sizes and input types** — the breakpoint set,
how the navigation and the content column reflow at each breakpoint, how every screen type in the
frozen inventory adapts to a phone, and the touch/ergonomics rules that make the product usable with a
thumb and an on-screen keyboard. It is the bridge between the *static* design (the navigation model and
the design tokens) and a product someone can actually use on the device in their hand.

It is **design intent and rules, not implementation**. Breakpoints are given as named tiers with
viewport ranges an implementation maps to media/container queries; reflow is given as behaviour and layout
intent, not as CSS. "How" (the framework, the stylesheet, the query syntax) is deferred.

**Boundaries to sibling sections** (so nothing is designed twice or left in a seam):

- **The navigation *content*** — which items exist logged-out vs signed-in, what the account menu holds,
  the reachability guarantee, the logged-out↔signed-in transition — belongs to the **IA & interaction**
  section. This section defines only how that navigation **reflows** (bottom bar ↔ top bar ↔ left rail)
  while preserving its reachability.
- **Design tokens and component anatomy** — the content-column width (`layout.content.max` 600px), the
  wide column (`layout.wide.max` 960px), the gutter (`layout.gutter` 16px), the touch target *value*
  (`layout.touch.min` 44px), the type scale, the component states — belong to the **visual language &
  design system** section. This section *consumes* those tokens and defines how layouts built from them
  rearrange; it invents no new token and no new component.
- **Screen-level content, copy, and which state each screen shows** belong to the **screens & flows**
  section. This section defines how each screen *type* reflows and how its state blocks adapt, not the
  per-screen copy.
- **Accessibility conformance** — keyboard operability, focus order, ARIA, contrast, the formal
  target-size audit against the stated WCAG level — belongs to the **accessibility** section. This
  section states the responsive/touch *intent* (44px targets, keyboard-aware forms, touch equivalents
  for hover-only affordances) and flags each item that section verifies; it does not own the audit.

**Grounding.** Grounded in `Nousix-LLC/nswe-demo` at `c805525`: the current SPA renders
a **single fixed ~40rem column** with a two-link text nav (`Timeline · Compose`) and **no breakpoints**
— it does not adapt to the viewport, so on a phone the fixed column and text-link nav are cramped and on
a wide display the content neither centres as a calm column nor uses the extra space. The current layout
is the **starting point to improve, not a target to preserve**.

---

#### 1. Design approach

chirp is a **consumer web application used on phones and on larger screens** (product definition: a
browser app a person reaches from a URL; the consumer-web-app kind assumes it works on the phone in
their hand). Four principles govern the responsive design:

1. **Design the phone first; the desktop is the phone with room to breathe.** The smallest screen is
   the hardest and the most common first visit; every gating surface is designed to work at the compact
   tier first, then given more room — never the reverse.
2. **One readable column, everywhere.** The product's substance is short text. The content column is the
   same single calm column at every size (its width capped by the visual system's `layout.content.max`);
   larger screens add *navigation and optional supplementary regions around it*, not width to the reading
   line.
3. **Reflow, don't hide.** Reflowing to a smaller screen rearranges the navigation and stacks content;
   it never drops a surface or an action. Every surface in the frozen inventory stays reachable at every
   breakpoint — especially sign-out, settings, and compose.
4. **Reachable by thumb and testable on a real device.** Interactive targets meet the touch minimum,
   the core loop sits in the thumb zone on a phone, forms stay usable with the on-screen keyboard up,
   and no primary content scrolls horizontally. The bar is written so a tester can check it on an actual
   phone and an actual desktop, not only in a design tool.

---

#### 2. Breakpoint set

A four-tier set, named by capability rather than by device, with viewport-width ranges. The tiers are
the shared reference every reflow rule below and every screen type uses. Values are chosen so the
600px content column (plus its 16px gutters, ≈632px) locks cleanly, and so the tiers land on real
device classes.

| Tier | Viewport width | Typical devices | Primary-nav form | Content column |
|---|---|---|---|---|
| **Compact** | `< 640px` | Phones (portrait); small phones down to ~320px | **Bottom tab bar** + top app bar + compose FAB | Fluid single column, full width minus `layout.gutter` each side |
| **Medium** | `640px – 1023px` | Large phones (landscape), small tablets (portrait) | **Top app bar**, nav items inline | Locks to `layout.content.max` (600px), centred; gutters absorb free space |
| **Expanded** | `1024px – 1279px` | Tablets (landscape), small laptops | **Left navigation rail** (icon + label) | 600px column centred in the space beside the rail |
| **Wide** | `≥ 1280px` | Laptops, desktops | **Left navigation rail** (fuller) | 600px column centred; optional supplementary right region (see §3.4, §7 REQ-UX-104) |

Notes:

- The ranges are **inclusive-exclusive and contiguous** (no gap, no overlap): Compact `[0,640)`, Medium
  `[640,1024)`, Expanded `[1024,1280)`, Wide `[1280,∞)`. An implementation maps these to breakpoints; the exact
  query mechanism is deferred.
- **Compact is the baseline**, not a special case: the design is authored at Compact and the wider tiers
  are progressive enhancements over it.
- The only surface that uses `layout.wide.max` (960px) rather than the 600px column is the
  **operator/admin** data-dense surface, and only at Expanded/Wide (see §4.10).
- Reflow is **viewport-driven and robust to resize and orientation change**: rotating a phone to
  landscape or resizing a window moves the layout between tiers without losing screen state or scroll
  position (REQ-UX-103), and layouts respect device **safe-area insets** (notches, rounded corners, the
  home indicator) so no control is occluded or untappable.

---

#### 3. Navigation reflow

The navigation *content* is fixed by the IA section (logged-out: wordmark, Search where present, Sign
in, Sign up, footer Privacy & terms; signed-in: wordmark, Home, Compose, Search where present, and the
account menu holding Profile, Settings, Privacy & terms, Sign out; the operator surface segregated).
This section defines how that same content **changes shape** across tiers while keeping the IA's
reachability guarantee intact at every size.

##### 3.1 Signed-in navigation

- **Compact (`< 640px`): bottom tab bar + top app bar + compose FAB.**
  - A fixed **bottom tab bar** carries the core-loop destinations in the thumb zone: **Home**,
    **Search** (only where discovery is shipped — no dead tab, per IA REQ-UX-008), and **Profile**
    (own). Tabs are icon + short label; the active tab is shown by indicator **and** weight, not colour
    alone (consuming the nav component's active state).
  - A **compose FAB** (floating primary action, the nav's single `primary`) sits above the bottom bar in
    thumb reach, so composing is always one tap (§4.3).
  - A slim **top app bar** holds the **wordmark** and the **account entry** (avatar / `@handle`, the
    persistent signed-in-state indicator). Tapping it opens the **account menu as a bottom sheet**
    carrying Profile, Settings, Privacy & terms, and **Sign out** — so settings and sign-out remain
    reachable at the smallest size without a tab of their own.
- **Medium (`640–1023px`): top app bar.** The primary nav collapses back into a single horizontal top
  bar: wordmark, Home, Compose, Search (where present), and the account menu (`@handle ▾`). The bottom
  tab bar and FAB retire; Compose becomes a bar action/button (§4.3). This matches and generalises the
  current product's single top nav rather than breaking it.
- **Expanded / Wide (`≥ 1024px`): left navigation rail.** The nav becomes a persistent **vertical rail**
  on the left: wordmark at the top; Home, Search (where present); **Compose as a full-width `primary`
  button** in the rail; and the **account cluster** (avatar + `@handle`) pinned at the bottom, opening
  the account menu (Profile / Settings / Privacy & terms / Sign out) upward. At Wide the rail may show
  fuller labels; at Expanded it may compress toward icon-forward with labels retained where space allows
  (never icon-only without an accessible name — flagged to accessibility).

At **every** tier the complete signed-in surface set is reachable: Home, Compose, Search (where
present), Profile, Settings, Privacy & terms, and Sign out. **Sign-out and Settings always live in the
account menu** in whatever form the nav takes (bottom sheet, top-bar dropdown, or rail flyout) — a
single, predictable home, never dropped on small screens (REQ-UX-092; IA REQ-UX-004/REQ-UX-005).

##### 3.2 Logged-out navigation

- **Compact:** a top app bar with the **wordmark** and the **join affordance** — `Sign up` as a visible
  primary and `Sign in` alongside (or `Sign in` + a `Sign up` primary where space is tight), both
  persistent on every logged-out surface. `Privacy & terms` and `About` live in the low-prominence
  footer reached by scrolling or an overflow. No bottom bar is needed (few destinations). Reading is the
  priority; the join route is always on screen (IA REQ-UX-002).
- **Medium and up:** the logged-out chrome stays a top bar with wordmark, Search (where present), and
  the persistent `Sign in` / `Sign up`; the footer carries Privacy & terms / About. The left rail is a
  signed-in pattern; logged-out keeps the lighter top bar across Medium/Expanded/Wide since its
  destination set is small.

##### 3.3 Operator / admin navigation

The operator surface is segregated from the consumer navigation by the IA section and is **not** woven
into any consumer nav at any breakpoint. Its own responsive behaviour (a data-dense surface) is in §4.10.

##### 3.4 Supplementary region (Wide only, optional)

At Wide (`≥ 1280px`) the space to the right of the centred column MAY host a **supplementary region**
for the non-gating Search / discovery / "who to follow" surface where that is shipped. When discovery is
not shipped, the region is simply **omitted** and the column stays centred — never a dead or empty rail.
Because this enhancement is tied to a non-gating product requirement, it is itself non-gating
(REQ-UX-104); the gating floor at Wide is the rail + centred column.

---

#### 4. Per-screen-type responsive behaviour

Every screen/surface in the screen inventory is covered below, grouped by screen
*type* (screens that reflow identically share a rule). Each uses the breakpoint set (§2), the reflowed
navigation (§3), and the visual system's components and state blocks (the state blocks reflow with their
host screen — a skeleton, empty, error, not-found, end-of-list, or session-expired block fills the same
single column and never forces horizontal scroll).

##### 4.1 Timelines — `SCR-landing_timeline`, `SCR-home_timeline`

- Single vertical column of chirp cards at every tier; the card spans the column width (fluid at
  Compact, 600px at Medium+). Cards never render side-by-side.
- `loading` skeletons, `empty`, `error` (with in-place Retry), and `end_of_list` fill the column and
  reflow with it; paging ("load more") control is full-width-reachable at Compact and centred in the
  column at larger tiers.
- The compose affordance is the FAB at Compact and an inline composer / `Compose` action at Medium+
  (§4.3). Avatars use their token sizes and never overflow (§4.11).
- Scroll position is preserved across a tier change / rotation (REQ-UX-103) so a returning reader is not
  thrown to the top (supports REQ-PROD-025's position-keeping at the layout level).

##### 4.2 Profile — `SCR-profile` (own and other)

- **Header** reflows: avatar (`xl` 64 at Medium+, may step to `lg` 48 at Compact to save vertical
  space), display name + `@handle` cluster, follower/following counts, and the primary action
  (**Follow/Following** on another's profile, **Edit profile** + route to Settings on own). At Compact
  the counts wrap below the identity cluster and the primary action becomes full-column-width and
  thumb-reachable; at Medium+ they sit on one row.
- The profile's chirps list reflows exactly as the timeline (§4.1), including its `empty` (no chirps)
  and `not_found` (unknown/deleted account) blocks.

##### 4.3 Compose — `SCR-compose`

- **Compact:** composing opens a **full-screen composer** (reached from the FAB) — textarea, character
  counter, and the `primary` post action, all above the on-screen keyboard (§5). It presents as a
  full-height sheet rather than a tiny inline box so the draft and counter are never cramped.
- **Medium and up:** compose is an **inline composer at the top of the home timeline** and/or a
  **modal** over the current view (capped at `layout.content.max`), per the IA's "reachable in-context
  without losing state." Either way the surrounding timeline context is preserved.
- At every tier the draft survives a re-render and a transient failure and the counter/post action stay
  visible (REQ-UX-093; composes REQ-PROD-016/017, IA REQ-UX-012).

##### 4.4 Forms — `SCR-signup`, `SCR-signin`, `SCR-recovery`, `SCR-edit_profile`, report reason (`SCR-report_block` form), export request (`SCR-data_export` form)

- Single-column form at every tier: label above field, fields stacked, inputs full-width at Compact and
  capped at `layout.content.max` at Medium+ (a form never stretches to a wide display).
- The `primary` submit (Create account, Sign in, Save, Request export, Submit report) and any inline
  `validation`/`error` message stay visible with the keyboard up (§5); validation is inline at/before
  submit (consuming the input component's states — REQ-UX-096).
- The join/recovery cross-links ("Create an account", "Can't sign in?") remain visible and tappable
  below the form at Compact.

##### 4.5 Settings & account-management — `SCR-settings_home`, and sub-pages `SCR-edit_profile`, `SCR-data_export`, `SCR-account_deletion`

- `SCR-settings_home` is a single-column **settings-row list** (label + optional description + trailing
  chevron) at every tier, each row meeting the touch minimum (§5). It gathers Edit profile, Data export,
  Account deletion, Sign out, and Privacy & terms (IA REQ-UX-004).
- Sub-pages open as their own column (Compact/Medium) or beside the rail (Expanded/Wide); a two-pane
  list-plus-detail at Wide is an acceptable enhancement but is **not** required — the single-column form
  is the gating floor (REQ-UX-096).
- `SCR-account_deletion` and `SCR-data_export` forms/confirmations follow §4.4 and §4.6.

##### 4.6 Modals & dialogs — confirm (deletion, block, delete-own-chirp, sign-out), form dialogs (report, start export)

- **Compact:** a dialog presents as a **full-screen or near-full sheet** so its title, body, and action
  row are legible and the actions are thumb-reachable; the destructive action stays visually distinct and
  is **not** the default focus, and "Cancel" remains obvious (consuming the modal component's rules).
- **Medium and up:** a **centred dialog** over a scrim, width capped at `layout.content.max`, per the
  visual system. Action row never requires horizontal scroll.
- Consequence copy (what deletion removes) stays fully visible without truncation at every tier
  (REQ-UX-097; composes REQ-PROD-010/012/027, IA REQ-UX-018).

##### 4.7 Chirp thread / detail — `SCR-chirp_thread` (conditional)

- **Only if replies are built** (per the IA's either/or stance and REQ-PROD-023): the thread is a single
  column — a `detail` chirp card as the root, replies as standard cards below, and a reply composer that
  reflows exactly like compose (§4.3). If replies are deliberately absent, there is no thread surface to
  reflow and no dead entry point at any breakpoint. This section designs the "if built" reflow and
  records the "deliberately absent" case as a valid no-op — it adds no thread affordance.

##### 4.8 Onboarding — `SCR-onboarding` (non-gating)

- A single-column card/list of suggested first follows and a first-post prompt, reflowing as a list at
  Compact and a centred column at Medium+. Inherits the non-gating flag of the onboarding requirement
  but is designed, not skipped.

##### 4.9 Search / discovery — `SCR-search_discovery` (non-gating)

- The search input sits in the top bar / rail (or at the top of the results at Compact); results are a
  single-column list reflowing as the timeline, with `empty`/no-results and `loading` blocks. At Wide it
  MAY occupy the supplementary region (§3.4). Non-gating, inheriting REQ-PROD-024 / IA REQ-UX-008.

##### 4.10 Operator / admin — `SCR-admin_support`

- The one **data-dense** surface. At **Expanded/Wide** it uses `layout.wide.max` (960px) to show a user
  lookup and account/moderation detail as a table/list without cramping.
- At **Compact/Medium** a wide table MUST NOT force the **primary app content** to scroll horizontally:
  the table **reflows to stacked rows/cards** (each record a labelled block) as the gating behaviour; a
  horizontal scroll is permitted **only contained within the table region itself** (clearly a scrollable
  pane), never applied to the whole screen. Key columns (handle, account id, status, primary action) are
  prioritised when stacked.
- Kept intentionally light — only the screen/interaction reflow is in scope here; the operator
  capabilities and authorization are REQ-ID/REQ-OPS (REQ-UX-101).

##### 4.11 Trust / legal — `SCR-privacy_terms`; identity/media across screens

- `SCR-privacy_terms` is long-form reading in the single readable column (`layout.content.max`) at every
  tier, with a comfortable line length — never full-bleed text on a wide display.
- `SCR-signout` is an action (control in the account menu / settings), not a layout; its reachability at
  every tier is covered by §3.1.
- **Media & avatars** everywhere scale by the visual system's token sizes; the monogram-placeholder
  avatar scales with them; any chirp body media (future) is constrained to the column width and never
  overflows or introduces horizontal scroll (REQ-UX-102).

##### 4.12 Coverage check

Every inventory surface is covered by a rule above: landing/home timeline (§4.1), profile (§4.2),
compose (§4.3), sign-up/sign-in/recovery/edit-profile/report/export forms (§4.4), settings home +
deletion/export sub-pages (§4.5), modals/dialogs (§4.6), chirp thread (§4.7, conditional), onboarding
(§4.8), search (§4.9), admin/support (§4.10), privacy/terms + sign-out + media (§4.11). **No screen is
left without a responsive rule, and no screen or state could not be served by the reflow approach** — so
nothing is flagged as a responsive gap. The screen inventory, state vocabulary, and id
scheme are used as frozen and not redefined.

---

#### 5. Input & ergonomics

Cross-cutting rules that apply to touch input across all screens (the *value* of the touch minimum is
the visual system's `layout.touch.min` = 44px; this section defines its **application**, and the
accessibility section verifies conformance against the stated WCAG level):

- **Touch targets.** Every interactive control — nav items, tabs, the FAB, buttons, follow/like/overflow
  controls, settings rows, menu items, dialog actions — presents a touch target of at least 44×44px on
  touch, **including icon-only controls** whose glyph (icon `sm` 16 / `md` 20 / `lg` 24) is smaller than
  the hit area: the tappable area is padded to the minimum (REQ-UX-098).
- **Spacing between targets.** Adjacent touch targets carry enough gap (from the spacing scale) to avoid
  mis-taps — especially in the bottom tab bar, the chirp action row (like / overflow), and dialog action
  rows.
- **Thumb zone.** On Compact the core-loop controls live in the bottom third: the bottom tab bar and the
  compose FAB are in natural thumb reach; destructive or rare actions are not placed where a core action
  is expected.
- **On-screen keyboard.** For every form and the composer, the **active field and the primary submit
  action remain visible above the keyboard** — the layout scrolls the focused field into view and never
  traps content or the submit button behind the keyboard; `validation`/`error` messages for the focused
  field stay visible (REQ-UX-099; supports REQ-PROD-002/016/017).
- **Touch equivalents for hover-only affordances.** Any affordance the visual system reveals on *hover*
  — the "Unfollow" label revealed on hovering **Following**, the absolute timestamp on hovering a
  relative one, row actions revealed on card hover — MUST have a **touch-accessible equivalent** at
  Compact/touch (a visible control, a tap, or a long-press), so a phone user is never locked out of an
  action that a pointer user can reach (REQ-UX-100; supports REQ-PROD-020).
- **No horizontal scrolling of primary content** at any breakpoint; the only permitted horizontal scroll
  is contained within a clearly-scrollable region (the admin table at Compact, §4.10).
- **Safe-area insets** are respected so fixed chrome (top app bar, bottom tab bar, FAB) is not occluded
  by notches or the home indicator.

---

#### 6. Handoff notes (for sibling sections)

- **Coverage:** this section covers every screen in the inventory (§4.12), composes against
  the IA navigation model and the visual-system `layout.*` tokens and components without redefining them,
  and mints REQ-UX only within `REQ-UX-090..104`. No responsive gap was found; none is reported.
- **For the accessibility section (verify):** the 44px touch-target application (§5) against the stated
  WCAG target-size criterion; that the reflowed nav forms (bottom bar, top bar, rail, account sheet)
  preserve a sensible focus order and that icon-forward rail items retain accessible names; that
  touch-equivalents for hover affordances (§5) are operable by keyboard too; that reflow to a small
  viewport does not break reading order. This section states the responsive intent; the audit is yours.
- **For screens & flows:** each screen's state blocks reflow with their host column (§4); the per-screen
  copy for empty/error/not-found is yours — this section fixes only how they lay out and adapt.
- **For visual language & design system:** this section consumes `layout.content.max`, `layout.wide.max`,
  `layout.gutter`, `layout.touch.min`, the nav component states, the modal cap, and the icon sizes
  exactly as defined; it introduces no token or component.

---

#### 7. Requirements & acceptance criteria — REQ-UX-090 .. REQ-UX-104

Composes against `req_ux_schema` (id · statement · acceptance_criteria · release_gating · traces_to) and
the parent id scheme. Acceptance criteria are written so a stranger can exercise the live product **on a
real phone and a real desktop** and reach the same yes/no. `traces_to` references `REQ-PROD-*` and/or
journeys `Jn`; some also note the accessibility section, which verifies the stated intent.

###### REQ-UX-090 — A defined responsive breakpoint set the whole product reflows against
- **Statement:** The product has a defined set of breakpoints, and every screen adapts its layout across them rather than rendering a single fixed width.
- **Acceptance criteria:** Resizing the browser from a narrow phone width to a wide desktop width, the layout visibly changes form at least across a compact (phone), a medium, and an expanded/wide (desktop) tier — e.g. the navigation and the content column rearrange — rather than staying a single fixed-width column at all sizes (as it does today at `c805525`). The same tiers apply consistently across screens.
- **Release gating:** true
- **Traces to:** consumer-web-app kind (phone + desktop); REQ-PROD-013, REQ-PROD-014; J1, J5.

###### REQ-UX-091 — One readable content column that reflows without horizontal scroll
- **Statement:** Primary content sits in a single readable column that is fluid on a phone and centred (capped) on larger screens, and no primary content requires horizontal scrolling.
- **Acceptance criteria:** On a phone the timeline/compose/profile/settings content fills the width minus a side gutter as one column; on a wide display the same content sits in a single centred column rather than stretching full width; on no screen and at no window width does the primary content scroll sideways (a contained, clearly-scrollable admin table is the only exception).
- **Release gating:** true
- **Traces to:** REQ-PROD-014, REQ-PROD-018; J1, J5. (Builds on visual REQ-UX-074.)

###### REQ-UX-092 — Navigation reflows while keeping every surface reachable
- **Statement:** The navigation changes form across breakpoints (bottom tab bar on a phone, top bar at medium, left rail on desktop) while every navigable surface — including sign-out and settings — stays reachable at every size.
- **Acceptance criteria:** On a phone, the core-loop destinations and a compose action are reachable from a persistent bottom navigation and/or an on-screen action, and sign-out and settings are reachable from the account menu (opened from the avatar); at a desktop width the same destinations are reachable from a persistent navigation (rail or top bar); at no width is any consumer surface (Home, Compose, Search-where-present, Profile, Settings, Privacy & terms, Sign out) unreachable or available only via the URL bar.
- **Release gating:** true
- **Traces to:** J1, J3, J5, J6; REQ-PROD-005, REQ-PROD-009, REQ-PROD-015. (Preserves IA REQ-UX-001/002/004/005.)

###### REQ-UX-093 — Compose is reachable and usable at every breakpoint
- **Statement:** A signed-in user can compose a chirp at any screen size, with the composer and its controls usable with the on-screen keyboard and the draft preserved.
- **Acceptance criteria:** On a phone, a compose affordance is reachable in one tap (e.g. a floating action) and opens a composer whose text area, character counter, and post button are all visible above the on-screen keyboard; at larger sizes compose is reachable from the navigation and opens inline/modal without leaving the current context; in all cases typing text, seeing the counter, and posting work without the controls being hidden or the draft lost on a re-render or transient failure.
- **Release gating:** true
- **Traces to:** J5; REQ-PROD-016, REQ-PROD-017. (Composes IA REQ-UX-012.)

###### REQ-UX-094 — Timeline reflows to a usable single-column feed
- **Statement:** The landing and home timelines present as a single-column feed of chirps at every size, with paging and state blocks usable on a phone.
- **Acceptance criteria:** On a phone, chirps appear one per row filling the column (never side-by-side), the loading/empty/error/end-of-list presentations fill the column without overflow, and "load more" is reachable; on a wide display the feed is the same single centred column; scroll position is retained when the window is resized or the device rotated.
- **Release gating:** true
- **Traces to:** J1, J5; REQ-PROD-013, REQ-PROD-018, REQ-PROD-025.

###### REQ-UX-095 — Profile reflows with its header and actions reachable
- **Statement:** A profile's header (identity, counts, primary action) and chirps list adapt to the screen size and stay usable on a phone.
- **Acceptance criteria:** On a phone, the profile shows the identity cluster, the follower/following counts, and the primary action (Follow/Following for another user, or Edit profile + Settings route on one's own) without truncation and with the action thumb-reachable (counts may wrap below the identity); on a wide display the header sits comfortably on fewer rows; the chirps list reflows like the timeline including its empty and not-found states.
- **Release gating:** true
- **Traces to:** J5, J6; REQ-PROD-008, REQ-PROD-019, REQ-PROD-020.

###### REQ-UX-096 — Forms and settings reflow to a usable single column
- **Statement:** Sign-up, sign-in, recovery, edit-profile, report, export-request, and the settings screens present as a single-column form/list that is usable on a phone.
- **Acceptance criteria:** On a phone, each form shows labels above full-width fields stacked vertically, with the submit button and any inline validation visible with the keyboard up; on a wide display the same form is capped to the readable column rather than stretching full width; the settings home is a single-column list of rows each large enough to tap.
- **Release gating:** true
- **Traces to:** J2, J6, J7; REQ-PROD-001, REQ-PROD-002, REQ-PROD-008, REQ-PROD-009, REQ-PROD-010, REQ-PROD-011.

###### REQ-UX-097 — Modals and dialogs reflow (full-screen sheet on a phone, centred dialog on desktop)
- **Statement:** Confirmation and form dialogs adapt to the screen size while keeping their message and actions fully visible.
- **Acceptance criteria:** On a phone, a dialog (e.g. the delete-account confirmation) presents as a full-screen/near-full sheet whose consequence text and action buttons are fully visible and thumb-reachable, with the destructive action distinct and not the default focus; on a desktop the same dialog is a centred box over a scrim, capped to the readable width; at no size is the dialog's text truncated or its actions off-screen.
- **Release gating:** true
- **Traces to:** J5, J7; REQ-PROD-010, REQ-PROD-012, REQ-PROD-027. (Consumes visual modal; composes IA REQ-UX-018.)

###### REQ-UX-098 — Touch targets meet the minimum size and spacing
- **Statement:** On touch devices, every interactive control is large enough to tap reliably and spaced to avoid mis-taps, including icon-only controls.
- **Acceptance criteria:** On a phone, buttons, navigation items, the compose action, follow/like/overflow controls, settings rows, menu items, and dialog actions each present a touch target of at least ~44×44px (icon-only controls have a padded hit area even though the glyph is smaller), and adjacent targets are separated enough that the intended one is reliably hit.
- **Release gating:** true
- **Traces to:** J5; REQ-PROD-016, REQ-PROD-020, REQ-PROD-021. (Applies visual `layout.touch.min`; conformance verified by the accessibility section.)

###### REQ-UX-099 — Forms and compose remain usable with the on-screen keyboard
- **Statement:** When the on-screen keyboard is open, the active field and the primary action stay visible; no content is trapped behind the keyboard.
- **Acceptance criteria:** On a phone, focusing a field in sign-up/sign-in/recovery/edit-profile/compose scrolls that field into view and keeps the submit/post action and the field's validation message reachable above the keyboard; dismissing the keyboard restores the full layout; at no point is the submit control or the focused field hidden behind the keyboard.
- **Release gating:** true
- **Traces to:** J2, J5; REQ-PROD-002, REQ-PROD-016, REQ-PROD-017.

###### REQ-UX-100 — Hover-only affordances have a touch-accessible equivalent
- **Statement:** Any action or information the design reveals on pointer hover is also reachable on a touch device.
- **Acceptance criteria:** On a phone, the unfollow action (revealed on hover of "Following" on a pointer device), the absolute timestamp (shown on hover), and any card/row actions revealed on hover are each reachable by a visible control, a tap, or a long-press — no action is available only to a user with a hovering pointer.
- **Release gating:** true
- **Traces to:** J5, J6; REQ-PROD-020, REQ-PROD-018. (Bridges visual hover states; keyboard equivalence verified by the accessibility section.)

###### REQ-UX-101 — The admin/support surface reflows without breaking the app layout
- **Statement:** The data-dense operator/admin surface uses the wider column on desktop and reflows on small screens without forcing the primary content to scroll horizontally.
- **Acceptance criteria:** On a desktop, the admin user-lookup/detail uses a wider column (up to the design's wide max) to show its table/detail without cramping; on a phone, the same information reflows to stacked, labelled rows/cards — or scrolls horizontally only within a clearly contained table pane — so the overall screen never scrolls sideways; the key fields (handle, status, primary action) remain visible when stacked.
- **Release gating:** true
- **Traces to:** J8; REQ-PROD-028, REQ-PROD-033. (Operator capability/authorization are REQ-ID/REQ-OPS, out of UX scope.)

###### REQ-UX-102 — Avatars and media scale to the layout and never overflow
- **Statement:** Avatars, the monogram placeholder, and any chirp media scale with the layout and stay within the content column at every size.
- **Acceptance criteria:** On any screen size, avatars render at their intended size for that context and the monogram placeholder scales with them; no image or media element overflows the content column or introduces horizontal scrolling; a profile header avatar is appropriately sized on both a phone and a desktop.
- **Release gating:** true
- **Traces to:** J5, J6; REQ-PROD-008, REQ-PROD-019. (Consumes visual avatar tokens / REQ-UX-080.)

###### REQ-UX-103 — Usable on a small phone, portrait and landscape, without losing state
- **Statement:** The gating core loop is fully usable on a common small phone in portrait, and the layout survives rotation and resize without breaking or losing screen state.
- **Acceptance criteria:** On a phone viewport as narrow as ~360px wide in portrait, a person can read the timeline, open a profile, sign in, compose and post, and reach settings and sign-out, with no clipped controls, no horizontal scroll of primary content, and controls honouring safe-area insets; rotating the device to landscape (or resizing the window) reflows the layout to the appropriate tier without throwing the user to the top of the list, closing an open composer draft, or losing form input.
- **Release gating:** true
- **Traces to:** consumer-web-app kind; J1, J5; REQ-PROD-013, REQ-PROD-016, REQ-PROD-018.

###### REQ-UX-104 — Enhanced wide-desktop multi-region layout (supplementary rail)
- **Statement:** On a wide desktop the product may add a supplementary region (e.g. search/discovery or "who to follow") beside the navigation rail and the centred column, without that region ever appearing dead or empty.
- **Acceptance criteria:** Where discovery is shipped, a wide desktop shows a navigation rail, the centred content column, and a supplementary region using the extra width; where discovery is not shipped, the supplementary region is absent and the column remains centred — no empty or non-functional rail is shown. The gating desktop experience (rail + centred column) works whether or not this enhancement is present.
- **Release gating:** false — the multi-region desktop layout is tied to the non-gating search/discovery surface (REQ-PROD-024 / IA REQ-UX-008); the gating floor is the rail + centred single column (REQ-UX-091, REQ-UX-092). Recorded cut, not an omission.
- **Traces to:** J1, J5, J6; REQ-PROD-024.

---

#### 8. Traceability summary (REQ-UX → REQ-PROD / journeys)

| REQ-UX | Gating | Traces to |
|---|---|---|
| REQ-UX-090 | true | REQ-PROD-013, 014; J1, J5 (+ consumer-web-app kind) |
| REQ-UX-091 | true | REQ-PROD-014, 018; J1, J5 |
| REQ-UX-092 | true | REQ-PROD-005, 009, 015; J1, J3, J5, J6 |
| REQ-UX-093 | true | REQ-PROD-016, 017; J5 |
| REQ-UX-094 | true | REQ-PROD-013, 018, 025; J1, J5 |
| REQ-UX-095 | true | REQ-PROD-008, 019, 020; J5, J6 |
| REQ-UX-096 | true | REQ-PROD-001, 002, 008, 009, 010, 011; J2, J6, J7 |
| REQ-UX-097 | true | REQ-PROD-010, 012, 027; J5, J7 |
| REQ-UX-098 | true | REQ-PROD-016, 020, 021; J5 |
| REQ-UX-099 | true | REQ-PROD-002, 016, 017; J2, J5 |
| REQ-UX-100 | true | REQ-PROD-018, 020; J5, J6 |
| REQ-UX-101 | true | REQ-PROD-028, 033; J8 |
| REQ-UX-102 | true | REQ-PROD-008, 019; J5, J6 |
| REQ-UX-103 | true | REQ-PROD-013, 016, 018; J1, J5 (+ consumer-web-app kind) |
| REQ-UX-104 | false | REQ-PROD-024; J1, J5, J6 |

All 15 ids are unique and fall within the owned range `REQ-UX-090..104`; 14 are release-gating and 1
(REQ-UX-104) is an honest, recorded cut tied to the non-gating discovery surface.

---

## 5. Accessibility

*(section: `accessibility`)*

### Accessibility

*(section: `accessibility`)*

#### What this section settles, and what it leaves to its siblings

This section states **the accessibility standard chirp is held to, and makes it concrete enough to build
against and to verify**. Accessibility is not a feature bolted onto the product — it is part of whether a
person can *use and trust* chirp at all. A product that a keyboard-only user, a screen-reader user, a
low-vision user, or someone who cannot distinguish certain colours cannot operate is not finished, however
polished it looks to everyone else.

The section does four things:

1. names the **conformance target** (the WCAG level and version) and what it implies;
2. defines the **cross-cutting operability and perceivability requirements** — keyboard & focus, assistive-
   technology semantics, contrast, forms — over the design-system components and interaction patterns the
   sibling sections defined;
3. applies the standard **per screen** across the frozen inventory, so coverage is checkable; and
4. carries the **audit evidence** (a WCAG contrast measurement of the visual system's colour tokens) and
   the specific **fixes** the visual system must make for the product to meet the target.

**It is design intent and testable requirements, not implementation.** Where it names ARIA roles,
name/role/value, live regions, or landmarks, these are *required behaviours from an assistive-technology
user's side* — the mechanism (framework, component code, exact attributes) is a later implementation's.

**Boundaries to sibling sections** (so nothing is designed twice or left in a seam):

- The **components and their visual states** (focus-ring appearance, status colours, the state blocks,
  the dialog/menu/toast anatomy) belong to **visual_language_and_design_system**. This section states the
  *accessibility conformance* those components must meet, **verifies** the contrast/focus/motion intent the
  visual system flagged (visual-system.md §6), and — where a token value fails its intended ratio — states
  the required fix. Per the visual system's own handoff rule, **a token's NAME stays stable when its value
  must be darkened** to meet a ratio; this section never renames or redesigns a component.
- The **navigation model, the screen map, and the interaction patterns** (auth-gating, optimistic update +
  reconcile, confirmation, the logged-out↔signed-in transition) belong to **information_architecture_and_
  interaction**. This section states how those structures and patterns must be **operable by keyboard and
  intelligible to assistive technology** (focus on route change, announcing an optimistic result, trapping
  focus in a confirm dialog); it does not redefine the structures themselves.
- The **per-screen content, copy, and the empty/error/loading state designs** belong to **screens_and_
  flows** / **empty_error_loading_states**. This section states the accessibility requirements those
  screens and states must satisfy (e.g. an error state must be *announced*, not only shown).
- **Breakpoints and reflow** belong to **responsive_layout**. This section states the *reflow and
  resize-text* conformance requirements (WCAG 1.4.10, 1.4.4) the responsive strategy must satisfy; it does
  not fix the breakpoints.

**Grounding.** Grounded in `Nousix-LLC/nswe-demo` at `c805525` (the current SPA —
timeline, compose+sign-in, profile; handle-only login; minimal inline CSS with class hooks but no semantic
landmarks, no skip link, no managed focus, no live regions), in the product requirements (journeys J1–J8,
register `REQ-PROD-001..034`), and in the two dependency outputs. Contrast figures are computed from the WCAG 2.1 relative-luminance
and contrast-ratio formulae (W3C WCAG 2.1 §§ *relative luminance*, *contrast ratio*) applied to the visual
system's declared token values.

---

#### 1. Conformance target

**chirp is held to WCAG 2.1 Level AA** (Web Content Accessibility Guidelines 2.1, Level AA — which
includes all Level A criteria), across the **gating** surfaces of the screen inventory, in the current and
the one-step-down-from-current desktop and mobile browsers chirp supports.

**What that implies, concretely:**

- Every Level A and Level AA success criterion that applies to chirp's content is met on the gating
  screens. This section enumerates the criteria that bite for a product of this shape (a text-first,
  interactive SPA with forms, async actions, overlays, and infinite-scroll lists) and turns each into a
  testable requirement (§§2–5) and a per-screen obligation (§6).
- **AA, not AAA**, is a deliberate, recorded scope choice (§7, Honest cuts): AAA criteria (e.g. 7:1
  contrast, sign-language, no-exception reading level) are out of the first-release bar, though several are
  met incidentally (chirp's body text exceeds 7:1).
- The standard applies to **the real, interactive product**, including its dynamic states — loading,
  empty, error, not-found, session-expired, validation, busy, success — not just a static happy-path
  render. A state a sighted user perceives (a spinner, a toast, an inline error) must also be perceivable
  to an assistive-technology user (§3.4, §4, §5).
- Conformance is **verified**, not asserted (§7): a keyboard-only pass, a screen-reader pass, an automated
  check, and a contrast/zoom measurement, run against the live product on each gating screen.

**How the target maps to the rest of this section.** The WCAG principles (Perceivable, Operable,
Understandable, Robust) organise the requirements: Operable → §2 (keyboard & focus); Robust + Perceivable
(for AT) → §3 (semantics & assistive tech); Perceivable → §4 (contrast, resize, colour, text
alternatives, motion); Understandable → §5 (forms & validation). §6 applies all of them per screen.

---

#### 2. Keyboard operability & focus management

*(WCAG 2.1.1 Keyboard, 2.1.2 No Keyboard Trap, 2.1.4 Character Key Shortcuts, 2.4.1 Bypass Blocks,
2.4.3 Focus Order, 2.4.7 Focus Visible, 1.4.11 Non-text Contrast [focus indicator], 3.2.1 On Focus.)*

The product must be **fully operable without a pointing device**. The current SPA relies on native links
and buttons (which are keyboard-operable by default) but adds no skip link, no managed focus on
view/route change, and — once compose becomes a modal and menus/dialogs appear — will introduce overlays
that are the classic place keyboard access breaks. This section fixes the rules before that happens.

##### 2.1 Everything is operable by keyboard
- Every interactive element in the design system (button, link, text input, textarea, search field,
  follow control, like control, nav item, account menu + items, overflow menu + items, modal and its
  actions, toast action/dismiss, tabs/segmented control, settings row/toggle, "load more", retry, the
  character counter's associated submit) is reachable and operable with the keyboard alone — Tab/Shift-Tab
  to move, Enter/Space to activate, Arrow keys within composite widgets (menus, tabs), Esc to dismiss
  overlays.
- **No keyboard traps** (2.1.2): focus can always move away from any component by standard keys; the only
  intentional containment is a modal's focus trap (§2.4), which Esc and the dialog's own controls always
  release.
- **No action is pointer-only.** Hover-revealed affordances (e.g. "Following" → "Unfollow" on hover, the
  absolute timestamp on hover, a chirp's overflow trigger) have a keyboard-reachable equivalent: the
  control is focusable and its function is available on focus/activation, not only on hover (2.1.1, 1.4.13).
- Any character-key shortcut, if introduced later, is remappable or can be turned off (2.1.4). None is
  required for the first release.

##### 2.2 Visible focus on every interactive element
- A **clearly visible focus indicator** appears on every element that receives keyboard focus (2.4.7).
  This verifies and is the same indicator the visual system defines (visual-system.md §3.0, REQ-UX-088):
  a 2px ring with a 2px offset.
- The focus indicator must meet **non-text contrast ≥ 3:1 against every background it can appear on**
  (1.4.11) — not only the white app background. **Audit finding (§4.1):** the ring colour
  `color.accent.base` measures exactly **3.00:1 on `color.bg` (#FFFFFF)** but only **2.84:1 on
  `color.bg.subtle` (#F7F9FA)** (hover rows) and **2.70:1 on `color.accent.subtle` (#E8F5FE)** (selected/
  active rows) — so a focus ring drawn in `accent.base` **fails 1.4.11 on tinted rows**. Required fix:
  the focus ring uses a value that holds ≥ 3:1 on all these backgrounds (e.g. `color.accent.strong`
  `#1565A0`, measured 6.17:1 on white and ≥ 5.5:1 on the tinted backgrounds), **or** the ring is drawn as
  a two-tone indicator (an inner/outer pair, one light one dark) so one edge always clears 3:1. Focus is
  never conveyed by colour alone — the ring is a shape change, and focus is never removed (no
  `outline:none` without an equivalent) (1.4.1, 2.4.7).
- Focus must never be hidden behind sticky chrome or clipped by an overflow container; a focused element
  is scrolled into view.

##### 2.3 Logical focus order and bypass
- **Focus order follows reading/DOM order** and the visual layout (2.4.3): nav → (skip target) main
  content → secondary. Tabbing through a screen produces a sensible sequence; it never jumps
  unpredictably.
- A **skip-to-content link** (2.4.1) is the first focusable element on every screen, visible on focus,
  moving focus past the persistent navigation straight to the main content region (§3.1 landmark). This
  closes a gap the current SPA has entirely.
- Within the chirp card, a predictable order: the card/author link, then body links/mentions (if
  navigable), then the action row (like, overflow). A card that is itself a navigation target (opens the
  thread) is a single focus stop with a clear name (§3.3); its inner controls are not redundant extra
  stops that confuse order.

##### 2.4 Focus management for overlays, async surfaces, and route changes
This is where an SPA most often fails AT users, and where the IA's patterns (modal compose, confirm
dialogs, toasts, the logged-out↔signed-in transition, route changes) create the risk.

- **Modal / dialog** (compose-as-modal, confirm dialogs for deletion/block/delete-chirp, the export/report
  form dialog): on open, focus moves into the dialog; focus is **trapped** within it (Tab cycles inside)
  until it closes; **Esc closes it**; on close, **focus returns to the control that opened it** (2.4.3,
  2.1.2). Background content is inert to AT while the dialog is open (`aria-modal`/`inert` semantics —
  mechanism deferred). Per the visual system and IA, a destructive dialog's **default focus is the safe
  (Cancel) control**, never the destructive action (REQ-UX-076, REQ-UX-018).
- **Route / view change** (timeline → profile, open thread, open settings, logged-out → signed-in): on
  navigation, focus moves to a sensible target — the new view's main heading (`<h1>`) or main landmark —
  and the **page title/`document.title` updates** so a screen-reader user hears where they are (2.4.2
  Page Titled, 2.4.3). Focus is not silently left on a stale control or reset to the top with no
  announcement.
- **Toast / snackbar**: a transient confirmation (`success`) or non-blocking error **does not steal
  focus**; it is announced via a live region instead (§3.4) and, if it carries an action
  (Undo/Retry/dismiss), that action is keyboard-reachable while the toast is visible (2.2.1 — see §4.5
  for its timing rule). Blocking/actionable errors use an inline alert the user can reach in flow, not a
  toast (visual-system.md §3.12/§3.13).
- **Auth-gating prompt & session-expired** (IA §3.3): when a gated action raises a sign-in prompt, focus
  moves to the prompt; on success, focus returns to resume the attempted action (mirrors the IA's
  return-to-intent) without discarding typed input. A `session_expired` outcome routes through this same
  managed prompt — never a silent focus loss or a dead end.

---

#### 3. Semantic structure & assistive technology

*(WCAG 1.3.1 Info and Relationships, 1.3.2 Meaningful Sequence, 2.4.6 Headings and Labels, 4.1.2
Name/Role/Value, 4.1.3 Status Messages, 2.4.2 Page Titled, 2.4.4 Link Purpose.)*

The product must expose its structure and its controls to assistive technology, so a screen-reader user
perceives the same information and can take the same actions as a sighted user. Today's SPA uses `div`-based
markup with class hooks and no landmarks, headings, or accessible names on icon controls; this section
sets the required semantics for the design system.

##### 3.1 Landmarks & document structure
- Each screen exposes standard **landmark regions** (1.3.1): a banner/header containing the primary
  navigation (`nav`), a single **`main`** region (the skip-link target), and a contentinfo/footer where
  the logged-out chrome's Privacy & terms lives. The operator/admin surface has its own `main`.
- A single, correct **heading hierarchy** per screen (2.4.6, 1.3.1): one `<h1>` naming the screen (e.g.
  the profile's display name, "Settings", "Create account"), with `<h2>`/`<h3>` for sections in order, no
  skipped levels used for styling. Headings are how a screen-reader user scans; the type scale
  (visual-system.md §2.2) maps to real heading levels, not just visual size.
- **Lists are lists** (1.3.1): the timeline, profile chirps, search results, and settings list are marked
  up as lists so AT announces position and count ("item 3 of 20").

##### 3.2 Name/role/value for every component
Every design-system component exposes an accurate **name, role, and value/state** to AT (4.1.2). The
required semantics, component by component:

| Component (visual-system ref) | Role | Accessible name | State/value exposed to AT |
|---|---|---|---|
| Button / primary-secondary-tertiary-destructive (§3.1) | button | Its visible label | disabled; **busy** announced (e.g. `aria-busy`/"Posting…") so a submit's in-flight state is perceivable |
| Icon-only button — overflow, like, close, search, sign-out, compose (§3.1, §2.7) | button | **A text name** (e.g. "More actions", "Like", "Close", "Search", "Sign out") — verifies REQ-UX-087 | pressed/expanded where it toggles a menu |
| Link / author link / mention (§3.2) | link | Purpose clear from text or context (2.4.4) — e.g. "View @handle's profile" not a bare avatar | — |
| Text input / textarea / search (§3.3) | textbox / searchbox | Programmatically associated `<label>` (§5.1) | required; invalid (`aria-invalid`) tied to its error message (§5.2) |
| Character counter (§3.4) | status text | Associated with its field; reads **remaining** count | announces approaching/over-limit politely, without spamming each keystroke (§5.3) |
| Follow control (§3.7) | button (or switch) | "Follow @handle" / "Following @handle" | **state reflects reality**: the follow/following/busy state is exposed and the result announced (§3.4); verifies REQ-UX-081 for AT |
| Like control (§3.8) | button (or toggle) | "Like" / "Unlike", with the count in the name or an adjacent read ("Like, 1,024 likes") | pressed = liked; count change announced on toggle; not colour-alone (§4.3); verifies REQ-UX-082 for AT |
| Nav + nav item (§3.9) | navigation / link | Item label | **current** section exposed (`aria-current`) in addition to the visual indicator — active state not colour-alone (§4.3) |
| Account menu / overflow menu (§3.10) | menu / menubutton + menuitems | Trigger named ("Account menu"); items named | expanded/collapsed; arrow-key navigable (§2.1) |
| Modal / dialog (§3.11) | dialog (`aria-modal`), named by its title | Its `<h2>` title | focus trapped/returned (§2.4) |
| Toast / snackbar (§3.12) | status / alert (per severity) | — | announced via live region (§3.4), not focus-stealing |
| Inline alert / banner (§3.13) | status (success/info) or alert (error) | — | announced when it appears (§3.4) |
| State blocks — loading, empty, error, not-found, end-of-list, session-expired (§3.14) | see §3.4 | Their title/message | loading announced "Loading…"; error announced; etc. |
| Avatar / monogram (§3.5) | img or decorative | Alt = the user's name, or marked decorative when the name is already adjacent (§4.4) | — |
| Tabs / segmented control (§3.15) | tablist/tab/tabpanel | Each tab's label | selected tab exposed; arrow-key navigable |
| Settings row / toggle (§3.15) | link / switch | Row label; toggle labelled | toggle on/off state exposed |

Custom composite controls (menus, tabs, the dialog) must present a **complete** name/role/value — a
`div` styled as a button that exposes no role/name is a 4.1.2 failure and is not acceptable.

##### 3.3 Reading order & relationships
- The DOM/reading order matches the meaningful visual order (1.3.2) — e.g. in a chirp card, author then
  timestamp then body then actions; the character counter and any error are associated with their field,
  not floating unlabelled (1.3.1).
- Where information is conveyed by position or proximity for sighted users (a count next to an icon, an
  error under a field), the relationship is made programmatic (label association, `aria-describedby`), not
  left to visual adjacency alone.

##### 3.4 Live-region announcement of asynchronous state changes
*(WCAG 4.1.3 Status Messages — the criterion that most directly bites an interactive SPA.)* Every state
change in the frozen `state_vocabulary` that happens **without a focus move** must be announced to AT, so
an async result is never silent for a screen-reader user:

- **`loading`** → a polite "Loading…" (and its resolution) for a region fetch; a `busy` submit announces
  its in-flight and resolved state.
- **`success`** → the toast/inline success is announced politely ("Chirp posted", "Profile saved",
  "Export ready").
- **`error`** → an actionable/loading error is announced assertively (role `alert`) so it is not missed;
  it names what failed and that Retry is available.
- **optimistic `follow`/`like` results** → the settled true state and count are announced, and a
  **reconcile-on-failure** (IA §3.2) is announced as the revert + error, so AT state always matches
  reality (verifies REQ-UX-017, REQ-UX-081/082 from the AT side).
- **`validation`** → see §5.2 (announced, tied to the field).
- **`end_of_list`** → the "you're all caught up" terminal marker is perceivable (not a silent stop).
- **`session_expired`** → the re-sign-in prompt is announced and receives focus (§2.4).

Live regions are present in the DOM before the update (so the first message is announced), are the right
politeness (polite for status, assertive for errors), and do not announce so often they become noise.

---

#### 4. Perceivable — contrast, colour, resize, text alternatives, motion

##### 4.1 Colour contrast — audit of the visual system's tokens
*(WCAG 1.4.3 Contrast (Minimum) — 4.5:1 normal text, 3:1 large text; 1.4.11 Non-text Contrast — 3:1 for
UI component boundaries, states, and meaningful graphics/icons.)*

I measured every token pairing the visual system flagged for verification (visual-system.md §6), using the
WCAG 2.1 relative-luminance and contrast-ratio formulae on the declared token values against
`color.bg` `#FFFFFF`. **Results:**

**Text on the app background — all PASS 4.5:1** (normal-text AA):

| Token (use) | Value | Ratio vs #FFFFFF | 4.5:1 |
|---|---|---|---|
| `text.primary` (body, titles) | `#0F1419` | **18.51:1** | PASS (exceeds AAA) |
| `text.secondary` (handles, meta, helper) | `#536471` | **6.12:1** | PASS |
| `accent.strong` (links, accent text) | `#1565A0` | **6.17:1** | PASS |
| `success.ink` (status text) | `#0E6B41` | **6.57:1** | PASS |
| `error.ink` (status text) | `#A51C2B` | **7.50:1** | PASS |
| `warning.ink` (status text) | `#8A4B00` | **6.80:1** | PASS |

**Status `*.ink` on its own `*.bg` banner, and info text on accent tint — all PASS 4.5:1:** success.ink
on success.bg **5.79:1**; error.ink on error.bg **6.47:1**; warning.ink on warning.bg **6.09:1**;
accent.strong on accent.subtle **5.56:1**.

**Inverse (white) text on solid fills (buttons/labels) — 4.5:1 required:** inverse on `accent.strong`
**6.17:1** PASS; on `accent.pressed` **8.71:1** PASS; on `error.base` (destructive) **4.96:1** PASS; on
`success.base` **4.53:1** PASS (thin margin); **on `warning.base` `#E8820E` 2.75:1 — FAIL.**

**UI boundaries / focus ring / large & icon fills — 3:1 required:** `accent.base` as large/icon **3.00:1**
PASS (exact); `success.base` **4.53:1** PASS; `error.base` **4.96:1** PASS; **`color.border` `#CFD9DE`
1.44:1 — FAIL**; **`warning.base` `#E8820E` 2.75:1 — FAIL** as an icon/large fill; focus-ring `accent.base`
**3.00:1 on white but 2.84:1 / 2.70:1 on tinted rows — FAIL** on tinted backgrounds (see §2.2).

**Role-exempt (verified usage only):** `text.tertiary` `#8899A6` **2.94:1** — below 4.5:1 and correctly
reserved for placeholder/disabled text; **requirement:** it MUST NOT be the sole carrier of any essential
text (see §4.2). `accent.base` as *normal text* 3.00:1 — correctly never used for body text (the system
already routes accent text to `accent.strong`).

**Three required fixes** (token NAME stays stable; only the value darkens — per the visual system's handoff
rule. These are flagged for resolution as cross-section fixes the visual system owns; this section
does not edit the visual system):

1. **`color.border` (1.44:1 → ≥ 3:1).** Where `color.border` is the **sole visual boundary of an essential
   control** (text input, textarea, search field outline; card/control outline that carries meaning), it
   must meet 1.4.11's 3:1. Measured passing target: **≈ `#808D97` (3.40:1)** or darker. Purely decorative
   dividers (`color.border.subtle` between rows) are **exempt** from 1.4.11 and may stay light.
2. **`color.warning.base` (2.75:1 → ≥ 4.5:1 for text, ≥ 3:1 for icon).** White text must **never** sit on
   `warning.base` at its current value, and a warning icon/large fill fails 3:1. Measured passing target:
   **≈ `#B35E08` (4.64:1 vs white, and 4.64:1 for white-on-fill)**, which satisfies both the 3:1 icon rule
   and the 4.5:1 text rule. Alternatively, warnings continue to use `warning.ink` on `warning.bg` (which
   passes at 6.09:1) and never use a solid white-on-`warning.base` chip.
3. **Focus ring (3.00:1 on white but < 3:1 on tinted rows → ≥ 3:1 on all backgrounds).** Draw the ring in
   a value that clears 3:1 on `bg`, `bg.subtle`, and `accent.subtle` (e.g. `accent.strong` `#1565A0`), or
   as a two-tone indicator (§2.2).

Until these three are fixed, the contrast requirement (REQ-UX-113) is **not met** and the visual system's
REQ-UX-072 is only partially satisfied; these should be treated as release-gating remediations
on the gating screens.

##### 4.2 Information not conveyed by colour alone
*(WCAG 1.4.1 Use of Color.)* Verifies REQ-UX-073 from the visual system. Every status/state signal is
carried by **more than colour** — icon and/or text and/or shape/weight:

- error/success/warning → icon + text (inline alert, toast, field message), never a colour-only border;
- **liked** → filled icon + count change, not just a colour swap (REQ-UX-082);
- **following** → label text "Following" + state, not colour alone (REQ-UX-081);
- **active nav** → indicator shape **and** weight **and** `aria-current`, not colour alone (REQ-UX-073/§3.2);
- **validation error vs success** on a field → icon + message text, not only a red/green border;
- `text.tertiary` (the low-contrast neutral) is never the sole means of conveying essential information.

##### 4.3 Text resize, reflow & spacing
*(WCAG 1.4.4 Resize Text, 1.4.10 Reflow, 1.4.12 Text Spacing.)* Requirements the responsive strategy
(responsive_layout) and the type system must jointly satisfy:

- **Resize to 200%** (1.4.4): text can be enlarged to 200% (browser zoom / larger base size) with no loss
  of content or function — nothing is clipped, overlapped, or made unusable. The rem-based type scale
  (visual-system.md §2.2) supports this; this section requires it be verified.
- **Reflow at 320px** (1.4.10): content reflows to a 320 CSS-px-wide viewport (≈ 400% zoom on a 1280px
  screen) without two-dimensional scrolling for the primary reading content (the single 600px column
  collapses to a one-column, vertical-scroll layout). This is a conformance requirement on the responsive
  breakpoints, verified here.
- **Text spacing** (1.4.12): applying the WCAG text-spacing overrides (line-height 1.5×, paragraph spacing
  2×, letter 0.12em, word 0.16em) causes no loss of content or function.

##### 4.4 Text alternatives for non-text content
*(WCAG 1.1.1 Non-text Content.)*

- **Avatars / monogram placeholder** (visual-system.md §3.5): an avatar that conveys *which user* has a
  text alternative of the user's name, **or** is marked decorative when the display name is already
  programmatically adjacent (avoiding a doubled "Jane Jane"). The monogram placeholder is never announced
  as a broken/empty image.
- **Icons**: decorative icons are hidden from AT; **meaningful icon-only controls carry an accessible
  name** (§3.2, REQ-UX-087); status icons are supported by adjacent text (§4.2).
- **Any user media** (if images/media are ever attached to a chirp — not in the current product): a text
  alternative is required; until media exists, this is a forward requirement, not a current gap.

##### 4.5 Motion & timing
*(WCAG 2.2.1 Timing Adjustable, 2.2.2 Pause/Stop/Hide, 2.3.1 Three Flashes.)*

- **No content flashes** more than three times per second (2.3.1) — nothing in the design does, and nothing
  introduced may.
- **Reduced motion**: all motion (visual-system.md §2.8 `motion.*`, optimistic count animations, modal/
  toast transitions) **honours a reduced-motion preference** — states change without non-essential motion
  when the user requests it. This verifies the visual system's stated intent.
- **Auto-dismissing toasts** (2.2.1): a success/info toast that auto-dismisses either persists long enough
  to be read and can be **paused/extended** (e.g. stays while hovered/focused, with a manual dismiss), or
  the information is also available in a non-transient place — so a user who needs more time is not
  penalised. Essential, blocking messages use the inline alert (persistent), not a toast.

---

#### 5. Forms & validation

*(WCAG 1.3.1, 1.3.5 Identify Input Purpose, 2.4.6 Headings and Labels, 3.3.1 Error Identification,
3.3.2 Labels or Instructions, 3.3.3 Error Suggestion, 3.3.4 Error Prevention, 4.1.2.)* chirp's forms —
sign-up (handle, display name, password-mechanism-owned-by-REQ-ID), sign-in, recovery, compose, edit
profile, report reason, data-export request, account-deletion confirmation — are where accessibility
most often silently fails. The rules:

##### 5.1 Programmatic labels & instructions
- Every field has a **programmatically associated, visible label** (1.3.1, 3.3.2) — not a placeholder
  standing in for a label (the placeholder `text.tertiary` disappears on input and fails contrast, §4.1).
- Instructions and constraints are provided **before** they are needed and associated with the field
  (3.3.2): the handle's rules (≤ 15 chars, allowed characters) and compose's 280-char limit are stated
  and tied to the field, not discoverable only by failing.
- Where it applies, fields declare their **input purpose** (1.3.5) so a browser/AT can autofill (e.g. the
  sign-in identifier), to the extent the field maps to a standard purpose.

##### 5.2 Error identification, suggestion & announcement
- An invalid field is **identified in text** (3.3.1), the error message **describes how to fix it** (3.3.3)
  in plain language — "That handle is taken, try another" / "Handles can be up to 15 letters, numbers, or
  underscores" — not a raw API code (verifies the visual system's REQ-UX-077 and the IA's inline-
  validation pattern from the AT side).
- The error is **programmatically tied to its field** (`aria-describedby`, `aria-invalid`) so a screen
  reader reads the error when the field is focused, and is **announced** when it appears (4.1.3 / §3.4).
- Errors are **never colour-only** (§4.2): the field shows an icon + message, not just a red border.
- On submit of a form with multiple errors, focus moves to the first error (or an error summary), so a
  keyboard/AT user is taken to what needs fixing rather than guessing.

##### 5.3 Character counter & length limits
- The character counter (visual-system.md §3.4) is associated with its field and exposes the **remaining**
  count to AT; approaching-limit and over-limit are announced **politely** (not on every keystroke) and
  the over-limit state both disables submit and is perceivable to AT (§3.2).
- **Typed text is never lost** on a validation block or a failed send (REQ-PROD-017, REQ-UX-078) — this
  is an accessibility concern too: an AT user who cannot easily re-type must not be forced to.

##### 5.4 Error prevention for consequential actions
*(WCAG 3.3.4.)* For the legally/data significant, irreversible actions — **account deletion** and
**data-export** — the action is reversible, checked, or **confirmed** (3.3.4). chirp meets this via the
confirmation dialog pattern (IA §3.4, visual-system.md §3.11): deletion states exactly what is removed
(account, chirps, follows, likes) and requires explicit confirmation, with the safe choice as the default
focus (§2.4). The confirmation's consequence text is perceivable to AT as the dialog's body (§3.2).

##### 5.5 The error / validation / session-expired states made accessible
The seam states that live on forms and submits are each made accessible, closing the loop with §3.4:
`validation` (inline, associated, announced), `error` (announced, recoverable, draft preserved),
`busy_submitting` (announced busy, submit disabled), `success` (announced), and `session_expired` (routed
to the managed, announced re-sign-in prompt, §2.4) — never a silent fail or a false success for any user.

---

#### 6. Per-screen accessibility requirements (over the frozen inventory)

Every surface in the screen inventory is held to the standard. The table names the
accessibility obligations that bite per screen (beyond the cross-cutting §§2–5, which apply everywhere).
**Gating** reflects the screen's own gating flag in the inventory: a **gating** screen's accessibility is
**release-gating**; a **non-gating** screen (search, thread, onboarding) is designed to the same AA
standard but its accessibility **inherits the screen's non-gating status** (honest cut, §7). The operator
`SCR-admin_support` is held to keyboard + labels + contrast + forms AA; deeper AT optimisation of the
operator-only surface is a recorded, lower-priority item (§7).

| Screen | A11y gating | Key per-screen accessibility requirements |
|---|---|---|
| `SCR-landing_timeline` | yes | Landmarks + `<h1>`; skip link; the list-of-chirps is a list; loading/empty/error states announced (§3.4); logged-out nav's Sign in/Sign up keyboard-reachable; readable with no account (no AT gating behind auth). |
| `SCR-signup` | yes | Full §5: labelled handle/display-name fields, inline announced validation + suggestion, counter for handle limit, draft preserved; focus to first error on submit; success transitions focus to the next destination (§2.4). |
| `SCR-signin` | yes | §5 labels/errors; "Can't sign in?" recovery link and "Create account" link keyboard-reachable and clearly named; sign-in failure announced as an inline alert (§3.4); return-to-intent focus on success. |
| `SCR-recovery` | yes | §5 labelled fields + announced errors; reachable by keyboard from sign-in; clear confirmation of what happens next, announced. |
| `SCR-home_timeline` | yes | List semantics + position/count; "load more" is a keyboard-operable button and newly loaded items are perceivable (§3.4); `end_of_list` announced; compose affordance reachable; like/follow results announced and reconciled (§3.4); error-with-retry announced. |
| `SCR-compose` | yes | If modal: focus trap + return + Esc (§2.4); labelled textarea; counter exposes remaining + over-limit announced; busy/success/error announced; **draft never lost** (§5.3); "at most one primary" is the labelled Post button. |
| `SCR-profile` | yes | `<h1>` = display name (avatar alt not doubled, §4.4); own vs other exposed (edit/settings vs follow) and distinguishable without colour; follow control state exposed + announced (§3.2/§3.4); chirps list semantics; not-found state for unknown handle announced (§3.4). |
| `SCR-chirp_thread` | no (inherits) | If built: openable from the card as a single named focus stop; thread read order correct (1.3.2); reply form meets §5; the "deliberately absent" alternative exposes **no** dead reply/thread affordance to AT either (IA §2.3). |
| `SCR-onboarding` | no (inherits) | Keyboard-operable follow suggestions / first-post prompt; headings; any skippable step is keyboard-skippable and the skip is perceivable. |
| `SCR-search_discovery` | no (inherits) | Labelled search field (searchbox role); results as a list; results/empty/loading announced (§3.4); clear control keyboard-reachable; no dead entry if not shipped (IA REQ-UX-008). |
| `SCR-settings_home` | yes | Settings list as a navigable list with clearly named rows (edit profile, export, deletion, sign-out, privacy); toggle sub-components expose on/off (§3.2); sign-out reachable and named here and in the account menu (REQ-UX-005). |
| `SCR-edit_profile` | yes | §5 labelled display-name + bio fields; bio counter announced; save busy/success/error announced; saved changes reflected and perceivable. |
| `SCR-account_deletion` | yes | §5.4 error-prevention: confirm dialog with consequence text as dialog body (§3.2); **default focus = Cancel** (§2.4, REQ-UX-076); destructive control named and distinct (not colour-only); post-delete signed-out + not-found states announced. |
| `SCR-data_export` | yes | Export request is a keyboard-operable action; progress (determinate/indeterminate) announced (§3.4); the download/result is reachable and named; monospace machine values (export file name/id) still meet contrast and have text equivalents. |
| `SCR-signout` | yes | A named control (not icon-only without a name) in the account menu and settings; activating it is keyboard-operable; the transition to logged-out updates the landmark/nav and moves/announces focus (§2.4). |
| `SCR-report_block` | yes | Reached from a named overflow item; report reason form meets §5; block is a consequential action with a consequence-stating confirm (§5.4); confirmation announced (§3.4). |
| `SCR-privacy_terms` | yes | Reachable by keyboard in both nav states; correct heading structure for a long text document (2.4.6/1.3.1) so AT users can navigate sections; readable at 200% / reflow (§4.3). |
| `SCR-admin_support` | yes (keyboard/labels/contrast/forms); deeper AT = recorded lower-priority | Operator lookup form labelled (§5); results table uses real table semantics (headers associated) so values are perceivable; moderation actions keyboard-operable and confirmed where destructive; monospace ids meet contrast. Kept light per the inventory note; full AT optimisation of the operator surface is a recorded follow-up (§7). |

This table is the coverage evidence the coverage requirement checks: every
inventory screen is covered by the accessibility standard, or (for the three non-gating screens) covered to
the same standard with its release-gating inherited from the screen.

---

#### 7. How this is verified, and the honest cuts

##### 7.1 Verification method (how a stranger reaches the same yes/no)
Conformance is checked on **each gating screen in each applicable state** by:

1. **Keyboard-only pass** — unplug the mouse: reach and operate every control; confirm visible focus on
   each (on white and on tinted rows); confirm skip link, logical order, no trap, modal trap+return, Esc,
   and route-change focus.
2. **Screen-reader pass** — with a current screen reader: confirm landmarks, one correct heading order,
   accurate name/role/value for every control (especially icon-only and custom controls), and that
   loading/success/error/optimistic-reconcile/validation/session-expired changes are **announced** (§3.4).
3. **Automated check** — an automated accessibility checker (e.g. an axe-class ruleset) run on each screen/
   state with zero critical violations; automated checks are necessary but **not sufficient** — they do
   not replace passes 1–2.
4. **Contrast & zoom measurement** — the §4.1 token audit re-run after the three fixes land (all required
   pairings ≥ their ratio), plus a 200%-resize and 320px-reflow check (§4.3) with no loss.

##### 7.2 Deliberate scope boundaries (honest cuts, recorded not hidden)
- **AA, not AAA.** AAA criteria (7:1 contrast, no-exception reading level, sign-language, etc.) are out of
  the first-release bar. (Body text incidentally exceeds 7:1.)
- **Non-gating screens' accessibility inherits their non-gating status.** `SCR-search_discovery`,
  `SCR-chirp_thread`, and `SCR-onboarding` are designed to the same AA standard, but — because the screens
  themselves are non-gating product requirements — their accessibility is not a release blocker. They must
  still ship no *dead* or *inaccessible* affordance if present.
- **Operator/admin deep AT optimisation deferred.** `SCR-admin_support` meets keyboard + labels + contrast
  + forms AA; richer AT optimisation of this operator-only, small-audience surface is a recorded follow-up,
  consistent with the inventory's "keep it light" note.
- **Dark-theme contrast** is deferred with the theme itself (visual-system REQ-UX-089, non-gating): *if*
  a dark theme ships, it must meet the same contrast bar as the light theme; the semantic tokens keep this
  a value-only re-audit later.
- **Web product only.** chirp is a browser SPA; native-mobile-platform accessibility APIs are out of scope.
- **Formal published conformance statement / VPAT** is not produced in this release; §7.1 is the internal
  verification bar.

---

#### 8. Requirements & acceptance criteria — REQ-UX-105 .. REQ-UX-119

Composes against `req_ux_schema` (id · statement · acceptance_criteria · release_gating · traces_to) and
the parent id scheme. Acceptance criteria are written so a stranger can exercise the live product
(keyboard, a screen reader, a contrast/zoom tool) and reach the same yes/no. `traces_to` references
`REQ-PROD-*` and/or journeys `Jn`; several also realise or verify a sibling REQ-UX. All ids are unique and
fall within the owned range `REQ-UX-105..119`.

###### REQ-UX-105 — Stated conformance target: WCAG 2.1 AA
- **Statement:** chirp meets WCAG 2.1 Level AA on its gating screens, in supported current browsers.
- **Acceptance criteria:** For each gating screen, the §7.1 verification (keyboard pass, screen-reader
  pass, automated check with no critical violations, contrast/zoom measurement) passes; no gating screen
  has an unresolved Level A or AA failure.
- **Release gating:** true
- **Traces to:** J1–J8; REQ-PROD-009, REQ-PROD-014, REQ-PROD-015 (a product usable by, and trustworthy to,
  all its users).

###### REQ-UX-106 — Full keyboard operability, no keyboard trap
- **Statement:** Every function of the product is usable with the keyboard alone, and focus can always
  move away from any component.
- **Acceptance criteria:** With no pointing device, a person can navigate, read, compose, like, follow,
  open and use menus and dialogs, submit every form, and sign out; Tab/Shift-Tab/Enter/Space/Arrows/Esc
  behave as expected; no element traps focus except a modal, which Esc and its own controls always release.
- **Release gating:** true
- **Traces to:** J3, J5, J6, J7; REQ-PROD-005, REQ-PROD-016. (WCAG 2.1.1, 2.1.2, 2.1.4.)

###### REQ-UX-107 — Visible focus indicator meeting non-text contrast on every background
- **Statement:** Every keyboard-focusable element shows a visible focus indicator that is distinguishable
  on every background it can appear on.
- **Acceptance criteria:** Tabbing through any screen moves a clearly visible focus ring across every
  interactive element; the ring remains visible (≥ 3:1 non-text contrast) on the white background, on
  hovered/subtle rows, and on selected/accent-tinted rows; focus is never removed or invisible.
- **Release gating:** true
- **Traces to:** J3, J6; REQ-PROD-005, REQ-PROD-009. Verifies/realises REQ-UX-088. (WCAG 2.4.7, 1.4.11,
  1.4.1.)

###### REQ-UX-108 — Logical focus order and a skip-to-content link
- **Statement:** Focus order is logical and a skip link lets keyboard/AT users bypass the navigation.
- **Acceptance criteria:** Tabbing produces an order matching the visual/reading layout on every screen;
  the first focusable element is a skip link that becomes visible on focus and jumps focus into the main
  content past the nav.
- **Release gating:** true
- **Traces to:** J1, J5, J6; REQ-PROD-009, REQ-PROD-018. (WCAG 2.4.3, 2.4.1.)

###### REQ-UX-109 — Managed focus for overlays, async surfaces, and route changes
- **Statement:** Opening/closing overlays and changing views manage focus predictably, and transient
  messages do not steal focus.
- **Acceptance criteria:** Opening a dialog moves focus into it and traps it; Esc closes it and focus
  returns to the opener; a destructive dialog focuses the safe (Cancel) choice; changing view/route moves
  focus to the new screen's main heading and updates the page title; toasts announce without taking focus;
  the sign-in prompt for a gated/expired action takes focus and returns to the attempted action on success.
- **Release gating:** true
- **Traces to:** J2, J3, J5, J7; REQ-PROD-006, REQ-PROD-010, REQ-PROD-016. Realises IA §2.4/§3.3 for AT.
  (WCAG 2.4.2, 2.4.3, 2.1.2.)

###### REQ-UX-110 — Landmark regions and a correct heading hierarchy
- **Statement:** Each screen exposes landmark regions and a single correct heading structure.
- **Acceptance criteria:** A screen reader finds a banner/nav, one main region, and a footer where
  applicable; each screen has one `<h1>` naming it and ordered sub-headings with no skipped levels used for
  styling; the timeline/profile/search/settings lists are announced as lists with item counts.
- **Release gating:** true
- **Traces to:** J1, J5, J6, J7; REQ-PROD-009, REQ-PROD-018. (WCAG 1.3.1, 2.4.6, 2.4.1.)

###### REQ-UX-111 — Accurate name, role, and value for every control
- **Statement:** Every interactive element — including icon-only and custom controls — exposes an accurate
  name, role, and state to assistive technology.
- **Acceptance criteria:** A screen reader announces a meaningful name and correct role for every button,
  link, input, menu, dialog, tab, toggle, and the follow/like controls; icon-only controls (overflow,
  like, close, search, sign-out, compose) have text names; toggled/selected/expanded/busy/disabled states
  are exposed; no control is announced as an unlabelled generic element.
- **Release gating:** true
- **Traces to:** J3, J5, J6; REQ-PROD-005, REQ-PROD-020, REQ-PROD-021, REQ-PROD-027. Verifies REQ-UX-087.
  (WCAG 4.1.2.)

###### REQ-UX-112 — Asynchronous state changes are announced
- **Statement:** State changes that happen without a focus move are announced to assistive technology.
- **Acceptance criteria:** With a screen reader, loading, success, and error outcomes are announced;
  posting a chirp, saving a profile, liking, following, requesting an export, and deleting each announce
  their busy and resolved state; an optimistic like/follow that fails announces the revert and the error;
  reaching the end of a list and a session-expired prompt are announced — none is silent.
- **Release gating:** true
- **Traces to:** J1, J5, J6; REQ-PROD-006, REQ-PROD-018, REQ-PROD-020, REQ-PROD-021. Realises IA §3.1/§3.2
  for AT. (WCAG 4.1.3.)

###### REQ-UX-113 — Text and UI contrast meet AA (with the required token fixes)
- **Statement:** Text, interface boundaries, meaningful icons, and the focus indicator meet their WCAG AA
  contrast ratios.
- **Acceptance criteria:** Measured against their backgrounds, body/meta/link/status-ink text meet ≥ 4.5:1;
  button/label text on fills meets ≥ 4.5:1; essential control boundaries, meaningful icons, and the focus
  ring meet ≥ 3:1 on every background they appear on. Specifically, the three audited failures are fixed
  (token names unchanged): `color.border` darkened to ≥ 3:1 where it bounds an essential control;
  `color.warning.base` darkened so white-on-fill ≥ 4.5:1 and icon ≥ 3:1 (or warnings use `warning.ink` on
  `warning.bg`); the focus ring drawn so it holds ≥ 3:1 on white, subtle, and accent-tinted backgrounds.
- **Release gating:** true
- **Traces to:** J1; REQ-PROD-014. Verifies/completes REQ-UX-072; reports fixes to the visual system. (WCAG 1.4.3, 1.4.11.)

###### REQ-UX-114 — Information is never conveyed by colour alone
- **Statement:** Every status, selection, and state signal is conveyed by more than colour.
- **Acceptance criteria:** Error, success, warning, liked, following, active-nav, and field-validation
  states are each accompanied by an icon, text, shape, weight, and/or programmatic state, so a person who
  cannot distinguish the colours can still tell the state; no essential information is carried only by
  `text.tertiary` or by a colour difference alone.
- **Release gating:** true
- **Traces to:** J5; REQ-PROD-018, REQ-PROD-020, REQ-PROD-021. Verifies REQ-UX-073. (WCAG 1.4.1.)

###### REQ-UX-115 — Text resizes and content reflows without loss
- **Statement:** The product remains usable when text is enlarged and the viewport is narrow.
- **Acceptance criteria:** Zooming text to 200% loses no content or function (nothing clipped or
  overlapped); content reflows to a 320px-wide viewport with no two-dimensional scrolling of the primary
  reading content; applying the WCAG text-spacing adjustments breaks nothing.
- **Release gating:** true
- **Traces to:** J1, J5; REQ-PROD-014. Depends on responsive_layout. (WCAG 1.4.4, 1.4.10, 1.4.12.)

###### REQ-UX-116 — Text alternatives for non-text content
- **Statement:** Non-text content has an appropriate text alternative.
- **Acceptance criteria:** An avatar that identifies a user exposes the user's name (or is marked
  decorative when the name is already adjacent) and the monogram placeholder is never announced as a broken
  image; decorative icons are hidden from AT and meaningful icon-only controls carry names; any attached
  media (if introduced) has a text alternative.
- **Release gating:** true
- **Traces to:** J5, J6; REQ-PROD-008, REQ-PROD-019. (WCAG 1.1.1.)

###### REQ-UX-117 — Accessible forms: labels, instructions, and actionable announced errors
- **Statement:** Every form field is labelled and its validation is perceivable, actionable, and
  announced.
- **Acceptance criteria:** Each field (sign-up/sign-in/recovery, compose, edit profile, report reason,
  export, deletion confirm) has a visible, programmatically associated label (not a placeholder standing
  in for one); constraints are stated up front; an invalid field shows an icon + plain-language message
  that says how to fix it, tied to the field and announced by a screen reader; on submit, focus goes to
  the first error; typed text is never lost on a block or failure.
- **Release gating:** true
- **Traces to:** J2, J3, J6, J7; REQ-PROD-002, REQ-PROD-003, REQ-PROD-008, REQ-PROD-016, REQ-PROD-017.
  Verifies REQ-UX-077/078 for AT. (WCAG 1.3.1, 3.3.1, 3.3.2, 3.3.3, 3.3.4.)

###### REQ-UX-118 — Motion and timing respect user needs
- **Statement:** Motion honours reduced-motion, nothing flashes dangerously, and time-limited messages
  can be read.
- **Acceptance criteria:** With a reduced-motion preference set, non-essential animation (transitions,
  optimistic count animation, modal/toast motion) is removed or minimised while the product stays fully
  usable; no content flashes more than three times per second; an auto-dismissing toast persists long
  enough to read and can be paused/extended (e.g. on hover/focus) or its information is also available
  non-transiently; essential messages are not auto-dismissed.
- **Release gating:** true
- **Traces to:** J5; REQ-PROD-016, REQ-PROD-018. Verifies the visual system's motion/reduced-motion intent.
  (WCAG 2.2.1, 2.2.2, 2.3.1.)

###### REQ-UX-119 — Accessibility verified across the whole screen inventory
- **Statement:** The accessibility standard is applied and verified across every screen in the inventory,
  with gating screens' accessibility release-gating and non-gating screens held to the same standard.
- **Acceptance criteria:** Every surface in the screen inventory is covered by the per-screen requirements
  in §6 and passes the §7.1 verification in its applicable states; no inventory screen is omitted; where a
  screen is recorded non-gating, its accessibility is still designed to AA and ships no dead or
  inaccessible affordance. The §6 coverage table is the evidence the coverage requirement
  checks.
- **Release gating:** true (for the gating screens; non-gating screens' a11y inherits their status — §7.2)
- **Traces to:** J1–J8; REQ-PROD-009, REQ-PROD-014, REQ-PROD-015. (WCAG 2.1 AA applied per screen.)

---

#### 9. Traceability summary (REQ-UX → REQ-PROD / journeys · WCAG SC)

| REQ-UX | Gating | Traces to | Principal WCAG 2.1 SC |
|---|---|---|---|
| REQ-UX-105 | true | REQ-PROD-009, 014, 015; J1–J8 | Conformance (2.1 AA) |
| REQ-UX-106 | true | REQ-PROD-005, 016; J3, J5, J6, J7 | 2.1.1, 2.1.2, 2.1.4 |
| REQ-UX-107 | true | REQ-PROD-005, 009; J3, J6 | 2.4.7, 1.4.11, 1.4.1 |
| REQ-UX-108 | true | REQ-PROD-009, 018; J1, J5, J6 | 2.4.3, 2.4.1 |
| REQ-UX-109 | true | REQ-PROD-006, 010, 016; J2, J3, J5, J7 | 2.4.2, 2.4.3, 2.1.2 |
| REQ-UX-110 | true | REQ-PROD-009, 018; J1, J5, J6, J7 | 1.3.1, 2.4.6, 2.4.1 |
| REQ-UX-111 | true | REQ-PROD-005, 020, 021, 027; J3, J5, J6 | 4.1.2 |
| REQ-UX-112 | true | REQ-PROD-006, 018, 020, 021; J1, J5, J6 | 4.1.3 |
| REQ-UX-113 | true | REQ-PROD-014; J1 | 1.4.3, 1.4.11 |
| REQ-UX-114 | true | REQ-PROD-018, 020, 021; J5 | 1.4.1 |
| REQ-UX-115 | true | REQ-PROD-014; J1, J5 | 1.4.4, 1.4.10, 1.4.12 |
| REQ-UX-116 | true | REQ-PROD-008, 019; J5, J6 | 1.1.1 |
| REQ-UX-117 | true | REQ-PROD-002, 003, 008, 016, 017; J2, J3, J6, J7 | 1.3.1, 3.3.1–3.3.4 |
| REQ-UX-118 | true | REQ-PROD-016, 018; J5 | 2.2.1, 2.2.2, 2.3.1 |
| REQ-UX-119 | true | REQ-PROD-009, 014, 015; J1–J8 | 2.1 AA per screen |

---

## 6. Requirements & acceptance criteria — the unified REQ-UX register

*(section: `requirements_and_acceptance_criteria`)*

This is the single, authoritative experience-design requirement register: **119 requirements,
`REQ-UX-001 … REQ-UX-119`, contiguous, with no gaps and no id collisions.** Each requirement is
stated from the user's (or operator's) side, carries **testable acceptance criteria a stranger can
check against the live product**, a **release-gating** flag, and **traces to** ≥ 1 `REQ-PROD-*`
and/or journey `Jn`. Each concern owns a disjoint id range:

| Range | Concern (section) | Count |
|---|---|---|
| `REQ-UX-001 … 019` | Information architecture & interaction (§1) | 19 |
| `REQ-UX-020 … 069` | Screens, flows & states (§2) | 50 |
| `REQ-UX-070 … 089` | Visual language & design system (§3) | 20 |
| `REQ-UX-090 … 104` | Responsive layout (§4) | 15 |
| `REQ-UX-105 … 119` | Accessibility (§5) | 15 |

**The full statement and acceptance criteria for each requirement live in its owning section above**
(e.g. REQ-UX-043's full criteria are in §2's compose screen; REQ-UX-072's in §3). The table below is
the consolidated index and release bar — id, the requirement in brief, whether it gates release, and
what it traces to. **109 of the 119 are release-gating; 10 are honest, recorded cuts** (non-gating):
`REQ-UX-008, 013, 014, 047, 048, 049, 050, 051, 089, 104` — each nonetheless designed in full.

### 6.1 Information architecture & interaction — `REQ-UX-001 … 019`

| REQ-UX | Requirement (brief) | Gating | Traces to |
|---|---|---|---|
| REQ-UX-001 | Persistent signed-in primary navigation over the core loop | true | J5, J6; REQ-PROD-009, 016 |
| REQ-UX-002 | Logged-out navigation makes reading and joining obvious | true | J1; REQ-PROD-013, 014, 015 |
| REQ-UX-003 | Every inventory surface reachable through the product (not API-only) | true | J1–J8; REQ-PROD-009, 015 |
| REQ-UX-004 | A reachable account-management hub gathers the lifecycle surfaces | true | J6, J7; REQ-PROD-009 |
| REQ-UX-005 | Sign-out has a single, discoverable, consistent placement | true | J3; REQ-PROD-005 |
| REQ-UX-006 | Own-profile exposes edit & settings; own vs other distinguished | true | J5, J6; REQ-PROD-008 |
| REQ-UX-007 | Privacy & terms reachable in both navigation states | true | J7; REQ-PROD-029 |
| REQ-UX-008 | A discoverable entry point to search / discovery | false | J1, J5, J6; REQ-PROD-024 |
| REQ-UX-009 | The operator/admin surface is segregated from consumer navigation | true | J8; REQ-PROD-028, 033 |
| REQ-UX-010 | A coherent screen map with stable destinations and hierarchy | true | J1, J5, J6, J7; REQ-PROD-009, 018, 019 |
| REQ-UX-011 | A designed logged-out → signed-in transition | true | J2, J3; REQ-PROD-001, 004, 006, 022 |
| REQ-UX-012 | Compose reachable in-context without losing state | true | J5; REQ-PROD-016, 017 |
| REQ-UX-013 | Chirp thread/detail coherent-if-present, never a dead affordance | false | J5; REQ-PROD-023 |
| REQ-UX-014 | First signed-in destination after sign-up is a guided first-run | false | J2, J6; REQ-PROD-022 |
| REQ-UX-015 | A consistent signed-in / logged-out state indicator | true | J3, J6; REQ-PROD-006 |
| REQ-UX-016 | Auth-gating at the point of action, never a silent fail | true | J3, J5; REQ-PROD-006, 013 |
| REQ-UX-017 | Optimistic updates reconcile to the true state on failure | true | J5; REQ-PROD-020, 021 |
| REQ-UX-018 | Destructive actions confirm consequences; reversible ones don't over-confirm | true | J7, J5; REQ-PROD-010, 012, 027 |
| REQ-UX-019 | The state vocabulary is surfaced consistently product-wide | true | J1, J5, J6; REQ-PROD-014, 018, 019 |

### 6.2 Screens, flows & states — `REQ-UX-020 … 069`

| REQ-UX | Requirement (brief) | Gating | Traces to |
|---|---|---|---|
| REQ-UX-020 | State model — one consistent model across the product | true | REQ-PROD-014, 018, 019; J1, J5, J6 |
| REQ-UX-021 | State model — loading shown in place | true | REQ-PROD-018, 019, 014; J1, J5 |
| REQ-UX-022 | State model — empty explains itself & offers a next action | true | REQ-PROD-014, 018, 022, 024; J1, J5, J6 |
| REQ-UX-023 | State model — errors honest, in-place, input-safe | true | REQ-PROD-018, 017, 019; J1, J5 |
| REQ-UX-024 | State model — not-found honest, with a way back | true | REQ-PROD-019, 012; J5, J7 |
| REQ-UX-025 | State model — end-of-list explicit, distinct from empty | true | REQ-PROD-018; J5 |
| REQ-UX-026 | State model — session expiry prompts re-sign-in | true | REQ-PROD-006; J3, J6 |
| REQ-UX-027 | State model — validation inline & at/before submit | true | REQ-PROD-002, 003, 016; J2, J5 |
| REQ-UX-028 | State model — busy, no duplicate submit, reconcile on fail | true | REQ-PROD-016, 017, 020, 021; J5 |
| REQ-UX-029 | State model — success honest, observable, never false | true | REQ-PROD-016, 020, 021; J5 |
| REQ-UX-030 | SCR-landing_timeline — says what chirp is + a join route | true | REQ-PROD-013, 014, 015; J1 |
| REQ-UX-031 | SCR-landing_timeline — reading needs no account | true | REQ-PROD-013; J1 |
| REQ-UX-032 | SCR-landing_timeline — honest loading/empty/error/end | true | REQ-PROD-014, 018; J1 |
| REQ-UX-033 | SCR-signup — in-product account creation | true | REQ-PROD-001, 015; J1, J2 |
| REQ-UX-034 | SCR-signup — handle validation & uniqueness inline | true | REQ-PROD-002, 003; J2 |
| REQ-UX-035 | SCR-signup — honest busy/error/success, no lost input | true | REQ-PROD-001, 017; J2 |
| REQ-UX-036 | SCR-signin — credentialed, interlinked with signup + recovery | true | REQ-PROD-004, 015; J3 |
| REQ-UX-037 | SCR-signin — honest, non-enumerating, never false success | true | REQ-PROD-004, 006; J3 |
| REQ-UX-038 | SCR-recovery — reachable; regain the same account | true | REQ-PROD-007; J4 |
| REQ-UX-039 | SCR-recovery — honest, non-enumerating states | true | REQ-PROD-007; J4 |
| REQ-UX-040 | SCR-home_timeline — reverse-chron feed, honest states, paging | true | REQ-PROD-018 (025 non-gating); J5, J6 |
| REQ-UX-041 | SCR-home_timeline — like/follow/compose honest; reconcile | true | REQ-PROD-016, 020, 021; J5 |
| REQ-UX-042 | SCR-home_timeline — feed action on expired session prompts | true | REQ-PROD-006; J6 |
| REQ-UX-043 | SCR-compose — compose & post, visible ≤280, non-optimistic | true | REQ-PROD-016; J5 |
| REQ-UX-044 | SCR-compose — draft survives; failure/expiry honest | true | REQ-PROD-017, 006; J5, J6 |
| REQ-UX-045 | SCR-profile — honest, own-vs-other, real states | true | REQ-PROD-019, 008, 013, 012; J5, J6 |
| REQ-UX-046 | SCR-profile — follow/unfollow optimistic, reconciled | true | REQ-PROD-020; J5, J6 |
| REQ-UX-047 | SCR-chirp_thread — open thread, honest states, no dead affordance | false | REQ-PROD-023, 018, 019; J5 |
| REQ-UX-048 | SCR-chirp_thread — reply coherent, validated, non-optimistic | false | REQ-PROD-023, 016, 017; J5 |
| REQ-UX-049 | SCR-onboarding — first-run next action + observable payoff | false | REQ-PROD-022; J2, J6 |
| REQ-UX-050 | SCR-search_discovery — find accounts & content | false | REQ-PROD-024; J1, J5, J6 |
| REQ-UX-051 | SCR-search_discovery — honest search states (running ≠ none) | false | REQ-PROD-024; J1, J5, J6 |
| REQ-UX-052 | SCR-settings_home — reachable account-management hub | true | REQ-PROD-009; J6, J7 |
| REQ-UX-053 | SCR-edit_profile — view & edit display name/bio, persists | true | REQ-PROD-008; J6 |
| REQ-UX-054 | SCR-edit_profile — validates inline, never loses the edit | true | REQ-PROD-008, 002, 017; J6 |
| REQ-UX-055 | SCR-account_deletion — states removal + explicit confirm | true | REQ-PROD-010; J7 |
| REQ-UX-056 | SCR-account_deletion — signed out + not-found after | true | REQ-PROD-012; J7 |
| REQ-UX-057 | SCR-data_export — export own data, portable, downloadable | true | REQ-PROD-011; J7 |
| REQ-UX-058 | SCR-data_export — honest progress → real download or error | true | REQ-PROD-011, 018; J7 |
| REQ-UX-059 | SCR-signout — consistent control; logged out after | true | REQ-PROD-005; J3 |
| REQ-UX-060 | Account surfaces — session expiry prompts, never silent | true | REQ-PROD-006; J6, J3 |
| REQ-UX-061 | SCR-settings_home — privacy & terms reachable from hub | true | REQ-PROD-029, 009; J7 |
| REQ-UX-062 | SCR-report_block — report a chirp/account, told it filed | true | REQ-PROD-027, 017; J5 |
| REQ-UX-063 | SCR-report_block — block (consequences stated) & mute | true | REQ-PROD-027; J5 |
| REQ-UX-064 | SCR-report_block — honest states, never silent/false | true | REQ-PROD-027, 017, 006, 018; J5 |
| REQ-UX-065 | SCR-privacy_terms — reachable both nav states, no account | true | REQ-PROD-029, 013; J7 |
| REQ-UX-066 | SCR-privacy_terms — presented plainly & readably | true | REQ-PROD-029; J7 |
| REQ-UX-067 | SCR-admin_support — operator looks up account by handle/id | true | REQ-PROD-033; J8 |
| REQ-UX-068 | SCR-admin_support — detail enough to answer support | true | REQ-PROD-033; J8 |
| REQ-UX-069 | SCR-admin_support — act on abuse, effect visible to users | true | REQ-PROD-028, 033; J8 |

### 6.3 Visual language & design system — `REQ-UX-070 … 089`

| REQ-UX | Requirement (brief) | Gating | Traces to |
|---|---|---|---|
| REQ-UX-070 | One consistent visual language across the product | true | REQ-PROD-014; J1, J5, J6 |
| REQ-UX-071 | Readable typographic hierarchy | true | REQ-PROD-016, 018; J5 |
| REQ-UX-072 | Colour system with sufficient contrast *(completed by REQ-UX-113 fixes)* | true | REQ-PROD-014; J1 |
| REQ-UX-073 | Status is never carried by colour alone | true | REQ-PROD-018, 020, 021 |
| REQ-UX-074 | Consistent spacing and a readable content column | true | REQ-PROD-014; J1, J5 |
| REQ-UX-075 | A button system with full interaction states | true | REQ-PROD-016, 010; J5, J7 |
| REQ-UX-076 | Destructive actions are distinct and confirmed | true | REQ-PROD-010, 027, 028; J7 |
| REQ-UX-077 | Inputs with inline validation styling | true | REQ-PROD-002, 003, 008, 016; J2, J6 |
| REQ-UX-078 | Character-limit feedback that never loses the draft | true | REQ-PROD-016, 017, 002; J2, J5 |
| REQ-UX-079 | One consistent chirp card everywhere | true | REQ-PROD-016, 018, 019; J5 |
| REQ-UX-080 | Consistent identity presentation with a placeholder avatar | true | REQ-PROD-008, 019; J5, J6 |
| REQ-UX-081 | Follow control reflects the true state | true | REQ-PROD-020; J5, J6 |
| REQ-UX-082 | Like control reflects the true state and count | true | REQ-PROD-021; J5 |
| REQ-UX-083 | Standard state blocks used consistently | true | REQ-PROD-014, 018, 019; J1, J5 |
| REQ-UX-084 | Honest, recoverable error state with retry | true | REQ-PROD-018, 019; J1, J5 |
| REQ-UX-085 | Uniform submission feedback | true | REQ-PROD-016, 017, 020, 021; J5 |
| REQ-UX-086 | Standard feedback surfaces (dialog, toast, inline alert) | true | REQ-PROD-010, 016, 027; J5, J7 |
| REQ-UX-087 | Consistent iconography with labelled icon-only controls | true | REQ-PROD-005, 027; J3, J5 |
| REQ-UX-088 | Visible focus on every interactive element | true | REQ-PROD-005, 009; J3, J6 |
| REQ-UX-089 | Optional dark theme via semantic tokens | false | REQ-PROD-014 (consumer-web-app theme expectation) |

### 6.4 Responsive layout — `REQ-UX-090 … 104`

| REQ-UX | Requirement (brief) | Gating | Traces to |
|---|---|---|---|
| REQ-UX-090 | A defined responsive breakpoint set the whole product reflows against | true | REQ-PROD-013, 014; J1, J5 (consumer-web-app kind) |
| REQ-UX-091 | One readable content column that reflows without horizontal scroll | true | REQ-PROD-014, 018; J1, J5 |
| REQ-UX-092 | Navigation reflows while keeping every surface reachable | true | REQ-PROD-005, 009, 015; J1, J3, J5, J6 |
| REQ-UX-093 | Compose is reachable and usable at every breakpoint | true | REQ-PROD-016, 017; J5 |
| REQ-UX-094 | Timeline reflows to a usable single-column feed | true | REQ-PROD-013, 018, 025; J1, J5 |
| REQ-UX-095 | Profile reflows with its header and actions reachable | true | REQ-PROD-008, 019, 020; J5, J6 |
| REQ-UX-096 | Forms and settings reflow to a usable single column | true | REQ-PROD-001, 002, 008, 009, 010, 011; J2, J6, J7 |
| REQ-UX-097 | Modals and dialogs reflow (full-screen sheet on phone, centred on desktop) | true | REQ-PROD-010, 012, 027; J5, J7 |
| REQ-UX-098 | Touch targets meet the minimum size and spacing | true | REQ-PROD-016, 020, 021; J5 |
| REQ-UX-099 | Forms and compose remain usable with the on-screen keyboard | true | REQ-PROD-002, 016, 017; J2, J5 |
| REQ-UX-100 | Hover-only affordances have a touch-accessible equivalent | true | REQ-PROD-018, 020; J5, J6 |
| REQ-UX-101 | The admin/support surface reflows without breaking the app layout | true | REQ-PROD-028, 033; J8 |
| REQ-UX-102 | Avatars and media scale to the layout and never overflow | true | REQ-PROD-008, 019; J5, J6 |
| REQ-UX-103 | Usable on a small phone, portrait and landscape, without losing state | true | REQ-PROD-013, 016, 018; J1, J5 (consumer-web-app kind) |
| REQ-UX-104 | Enhanced wide-desktop multi-region layout (supplementary rail) | false | REQ-PROD-024; J1, J5, J6 |

### 6.5 Accessibility — `REQ-UX-105 … 119`

| REQ-UX | Requirement (brief) | Gating | Traces to | WCAG 2.1 SC |
|---|---|---|---|---|
| REQ-UX-105 | Stated conformance target: WCAG 2.1 AA | true | REQ-PROD-009, 014, 015; J1–J8 | Conformance (2.1 AA) |
| REQ-UX-106 | Full keyboard operability, no keyboard trap | true | REQ-PROD-005, 016; J3, J5, J6, J7 | 2.1.1, 2.1.2, 2.1.4 |
| REQ-UX-107 | Visible focus indicator meeting non-text contrast on every background | true | REQ-PROD-005, 009; J3, J6 | 2.4.7, 1.4.11, 1.4.1 |
| REQ-UX-108 | Logical focus order and a skip-to-content link | true | REQ-PROD-009, 018; J1, J5, J6 | 2.4.3, 2.4.1 |
| REQ-UX-109 | Managed focus for overlays, async surfaces, and route changes | true | REQ-PROD-006, 010, 016; J2, J3, J5, J7 | 2.4.2, 2.4.3, 2.1.2 |
| REQ-UX-110 | Landmark regions and a correct heading hierarchy | true | REQ-PROD-009, 018; J1, J5, J6, J7 | 1.3.1, 2.4.6, 2.4.1 |
| REQ-UX-111 | Accurate name, role, and value for every control | true | REQ-PROD-005, 020, 021, 027; J3, J5, J6 | 4.1.2 |
| REQ-UX-112 | Asynchronous state changes are announced | true | REQ-PROD-006, 018, 020, 021; J1, J5, J6 | 4.1.3 |
| REQ-UX-113 | Text and UI contrast meet AA (with the required token fixes) | true | REQ-PROD-014; J1 | 1.4.3, 1.4.11 |
| REQ-UX-114 | Information is never conveyed by colour alone | true | REQ-PROD-018, 020, 021; J5 | 1.4.1 |
| REQ-UX-115 | Text resizes and content reflows without loss | true | REQ-PROD-014; J1, J5 | 1.4.4, 1.4.10, 1.4.12 |
| REQ-UX-116 | Text alternatives for non-text content | true | REQ-PROD-008, 019; J5, J6 | 1.1.1 |
| REQ-UX-117 | Accessible forms: labels, instructions, and actionable announced errors | true | REQ-PROD-002, 003, 008, 016, 017; J2, J3, J6, J7 | 1.3.1, 3.3.1–3.3.4 |
| REQ-UX-118 | Motion and timing respect user needs | true | REQ-PROD-016, 018; J5 | 2.2.1, 2.2.2, 2.3.1 |
| REQ-UX-119 | Accessibility verified across the whole screen inventory | true | REQ-PROD-009, 014, 015; J1–J8 | 2.1 AA per screen |

### 6.6 Register integrity

- **Uniqueness & range:** all 119 ids are unique, contiguous `REQ-UX-001 … 119`, each within its
  owning concern's assigned range; no two concerns mint the same id; no gaps.
- **Testable:** every requirement has user/operator-side acceptance criteria a stranger can check
  against the live product; the full criteria are in the owning section (§§1–5). The accessibility
  requirements additionally name their governing WCAG 2.1 success criteria.
- **Gating honesty:** 109 release-gating; 10 recorded non-gating cuts (`008, 013, 014, 047, 048,
  049, 050, 051, 089, 104`), each still designed in full, each an explicit decision rather than an
  omission.
- **Traceability:** every requirement traces to ≥ 1 `REQ-PROD-*` and/or journey `Jn` from the
  product requirements.
