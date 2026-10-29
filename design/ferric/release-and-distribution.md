# Ferric 1.0 — Release & Distribution

This document covers four things a maintainer must have in place to ship Ferric as a credible 1.0: the
crates can be published to crates.io with their metadata complete and their license declared; the semver
contract tells an author what a version number promises; the changelog records what each release changed;
and the versioning and compatibility policy states what "1.0" guarantees across the 1.x line. All four are
stated as testable `REQ-REL-*` requirements so a stranger could check the live crates and repository and
reach the same verdict.

**What this document settles, and what it does not.** It is the `REQ-REL-*` register for release &
distribution — the publishing readiness, the semver contract, the changelog practice, and the 1.x
compatibility policy, each expressed as a checkable requirement. It does not own any other register; cross-
references to `REQ-FW-*`, `REQ-SSR-*`, `REQ-API-*`, `REQ-ASYNC-*`, `REQ-RT-*`, `REQ-HTTP-*`, `REQ-DX-*`,
or `REQ-PERF-*` are read-only. It designs **what**, not **how**: the mechanism of any publishing automation,
changelog generator, or CI semver gate is deferred to later code DAGs.

**Grounding.** Every observation about Ferric's current state is grounded in `Nousix-LLC/nswe-demo` @ `main`
(baseline `276138f`). The workspace is a virtual workspace (`members = ["crates/*"]`, `resolver = "2"`) with
`[workspace.package] version = "0.1.0"`, `edition = "2021"`, `rust-version = "1.82"`, `repository =
"https://github.com/Nousix-LLC/nswe-demo"`. The three Ferric crates (`ferric`, `ferric-router`,
`ferric-http`) each inherit `version.workspace = true` and `rust-version.workspace = true`, versioning in
lockstep at `0.1.0` today. No manifest has a `license` or `license-file` field, and there is no `LICENSE`
file at the repo root — this is the headline publishing gap. `ferric-router` depends on `ferric` via
`{ path = "../ferric" }` with no `version`, which also blocks its publish. There is no `CHANGELOG.md`
anywhere in the repository.

---

## 1. crates.io Publishing Readiness & Multi-Crate Release (`REQ-REL-001..011`)

Every requirement in this section is `stage: one_oh`, `release_gating: true`. Publishing to crates.io is
the primary 1.0 add: an author building a real app can work against path or git dependencies before 1.0;
the published, depend-on-normally story is what 1.0 delivers (`REQ-FW-001`).

### Licensing

#### REQ-REL-001 — Every published crate declares a permissive license
- **Statement:** Each Ferric crate published to crates.io declares a license in its manifest, and the chosen
  license is a permissive one compatible with the Rust ecosystem.
- **Rationale:** crates.io will not publish a crate that declares neither `license` nor `license-file` (Cargo
  book, *Publishing on crates.io*, lists `license` or `license-file` among the fields that must be filled
  out). This is the one gap that blocks publishing outright today — no manifest has a license and there is
  no `LICENSE` file. The Rust API Guidelines (C-PERMISSIVE) recommend `license = "MIT OR Apache-2.0"` with
  both `LICENSE-APACHE` and `LICENSE-MIT` at the repository root for maximum ecosystem compatibility; Ferric's
  Rust peers publish under the same permissive convention. A permissive choice also keeps `REQ-REL-011`'s
  "depend only on permissively-licensed crates" tractable.
- **Acceptance criteria:**
  - Every published crate's `Cargo.toml` carries a `license` (an SPDX expression) or a `license-file`; the
    same expression is used across the three crates. (`doc_inspection`)
  - `cargo publish --dry-run` no longer fails for a missing-license reason for any crate. (`publishability`)
  - If the `MIT OR Apache-2.0` convention is taken, the corresponding `LICENSE-APACHE` and `LICENSE-MIT`
    files exist where the manifests reference them and the README states the license. (`doc_inspection`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-001` (installable from crates.io); `REQ-REL-006` (the license files ship in the
  package).

### Per-crate crates.io metadata

#### REQ-REL-002 — Complete per-crate crates.io metadata
- **Statement:** Each published crate carries the metadata crates.io requires and the metadata an evaluating
  author expects to see on the crate's crates.io page: `description`, `repository`, `readme`,
  `documentation`, `homepage`, `keywords`, and `categories`.
- **Rationale:** `description` is required to publish (Cargo book, *Publishing on crates.io*); `repository`,
  `readme`, and `homepage` are the fields the same chapter recommends, and the Rust API Guidelines (C-METADATA)
  call for complete metadata so the crate is discoverable and its source and docs are reachable. An evaluating
  author (persona A1 in the scope doc) judges a framework in an afternoon from its crates.io landing page;
  missing keywords/categories make it undiscoverable and a missing repository/docs link reads as unfinished.
  Today only `description` is present. `readme` need not be set explicitly if a `README.md` sits beside the
  manifest (Cargo auto-detects it) — the requirement is that the rendered crate shows a README.
- **Acceptance criteria:**
  - Each published crate's manifest (or workspace-inherited metadata) provides `description`, `repository`,
    `documentation`, `homepage`, `keywords`, and `categories`, and the crate renders a README on crates.io.
    (`doc_inspection`)
  - `categories` uses only slugs from the crates.io category list, and `keywords`/`categories` stay within
    crates.io's count limits. (`publishability`)
  - The three crates' metadata is consistent where it should be (shared `repository`, coherent keyword set),
    and each `description` reads as that crate's own one-line summary. (`doc_inspection`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-001`; `REQ-REL-003` (the `documentation` link points at the docs.rs build).

#### REQ-REL-003 — API documentation builds on docs.rs for every published crate
- **Statement:** Each published crate's API documentation builds successfully on docs.rs so that a reader
  reaches rendered docs for the public surface from the crate's crates.io page.
- **Rationale:** docs.rs builds and hosts documentation automatically for every crate published to crates.io;
  a crate whose docs.rs build fails ships to adopters with a broken "docs" link. Ferric's crates are
  WASM-targeted but their binding dependencies compile off-`wasm32` (the repo already relies on this for
  native tests of `ferric-http`), so the default docs.rs target should build; where a crate genuinely needs
  a WASM target, that is declared in `[package.metadata.docs.rs]`. The workspace already enforces
  `missing_docs = "warn"`, so the public surface is documented.
- **Acceptance criteria:**
  - `cargo doc --no-deps` builds the public API docs for each crate without error on the target docs.rs will
    use. (`automated_test`)
  - After publish, each crate's docs.rs page renders the public `prelude` surface, and the crate's
    `documentation` field (`REQ-REL-002`) resolves to it. (`doc_inspection`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-002`; `REQ-FW-011`.

### The path→version dependency conversion

#### REQ-REL-004 — Intra-workspace dependencies carry both a path and a version
- **Statement:** Every dependency between Ferric crates is declared with both a `path` (for local
  development) and a `version` requirement (for the published crate), so each crate is publishable while the
  workspace still builds locally against its sibling sources.
- **Rationale:** Cargo will not publish a crate with a path dependency that lacks a version requirement
  (Cargo book, *Specifying Dependencies*: a path dependency must also carry a `version` to be published,
  because the published crate must resolve its dependency from crates.io). Today `ferric-router` depends on
  `ferric` via `{ path = "../ferric" }` with no version, which blocks its publish. The fix is
  `{ path = "../ferric", version = "..." }`: Cargo uses the path when building inside the workspace and
  records the version requirement in the published manifest.
- **Acceptance criteria:**
  - `ferric-router`'s dependency on `ferric` (and any future intra-workspace dependency) carries both `path`
    and a `version` requirement matching the released version line. (`doc_inspection`)
  - `cargo publish --dry-run` for `ferric-router` no longer fails for a path-dependency-without-version
    reason. (`publishability`)
  - A local `cargo build` of the workspace still compiles `ferric-router` against the sibling `ferric`
    source. (`automated_test`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-001`; `REQ-REL-009` (lockstep release); `REQ-REL-040..059` (the versioning/
  compatibility policy owns how the version requirement stays coherent across the workspace).

### `cargo publish` / `cargo package` readiness

#### REQ-REL-005 — Each crate passes `cargo publish --dry-run` / `cargo package`
- **Statement:** Each Ferric crate packages and verifies cleanly — `cargo publish --dry-run` (equivalently
  `cargo package`) succeeds for it — including while public-surface leaf bodies are still `todo!()`.
- **Rationale:** `cargo publish --dry-run` runs the verification crates.io performs: it checks the package,
  compresses the source into a `.crate`, and extracts and compiles it in a temporary directory. Making this
  the bar catches a missing license (`REQ-REL-001`), a version-less path dependency (`REQ-REL-004`), and
  packaging problems (`REQ-REL-006`) before a real publish. A `todo!()` body is a compiling expression that
  panics only at runtime, so a frozen-but-unimplemented surface still compiles and packages.
- **Acceptance criteria:**
  - `cargo package` (or `cargo publish --dry-run`) completes without error for `ferric`, `ferric-router`,
    and `ferric-http`, with the verification compile succeeding. (`publishability`)
  - The dry-run produces no publish-blocking warnings about missing required metadata. (`publishability`)
  - The run succeeds even with `todo!()` leaf bodies present. (`publishability`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-001`, `REQ-REL-004`, `REQ-REL-006`.

#### REQ-REL-006 — The packaged crate contains the license, README, and only intended files
- **Statement:** Each crate's packaged `.crate` archive contains the files a consumer and crates.io need —
  the source, the README, and the license file(s) the manifest references — and excludes files that do not
  belong in a published crate.
- **Rationale:** A published crate is self-contained: a `license-file`, or the `LICENSE-APACHE`/`LICENSE-MIT`
  files a `MIT OR Apache-2.0` crate ships, must be inside the `.crate` or the published crate has no license
  text even though the manifest names one. Because the crates live under `crates/*` in a virtual workspace,
  each crate's package must pull in the files it needs from its own directory. Keeping stray or large
  non-source files out holds the WASM framework's published footprint to the source that belongs in it,
  consistent with `REQ-FW-013`.
- **Acceptance criteria:**
  - `cargo package --list` for each crate shows its source, its README, and the license file(s) its manifest
    references, and does not list unintended files. (`doc_inspection`)
  - For a `MIT OR Apache-2.0` crate, both license files are present in each crate's package. (`publishability`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-001`; `REQ-REL-002`; `REQ-FW-013` (lean footprint).

### Multi-crate workspace release

#### REQ-REL-007 — The crate names are owned on crates.io
- **Statement:** The names `ferric`, `ferric-router`, and `ferric-http` (and any new 1.0 crate) are
  registered to and owned by the project on crates.io before the 1.0 release depends on them.
- **Rationale:** crates.io names are first-come and permanent; a 1.0 that promises `cargo add ferric` cannot
  rest on a name the project does not control. The first publish of each crate claims the name. Confirming
  ownership removes a late, unrecoverable surprise.
- **Acceptance criteria:**
  - Each crate name resolves on crates.io to a crate owned by the project (or is confirmed unregistered and
    reserved by the initial 1.0 publish). (`doc_inspection`)
  - The manifest `name` of each crate matches the owned crates.io name. (`doc_inspection`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-001`.

#### REQ-REL-008 — Crates publish in dependency order
- **Statement:** A coordinated release publishes the crates in an order that respects their dependency graph:
  a crate is published only after every Ferric crate it depends on is already available on crates.io at the
  version it requires.
- **Rationale:** crates.io resolves a published crate's dependencies from the registry at publish time, so a
  dependent cannot be published before its dependency exists there at the required version. For Ferric the
  order is `ferric` first, then `ferric-router`; `ferric-http` has no intra-workspace dependency and may
  publish independently. Peer multi-crate frameworks (Leptos, Dioxus, Yew) release their companion crates as
  a coordinated set for exactly this reason.
- **Acceptance criteria:**
  - The release publishes `ferric` before `ferric-router`; `ferric-http` has no Ferric dependency and orders
    freely. (`doc_inspection`)
  - Publishing the dependents succeeds because their required `ferric` version is already on crates.io.
    (`publishability`)
  - The required order is derivable from the manifests, not from tribal knowledge. (`doc_inspection`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-004`; `REQ-REL-009`.

#### REQ-REL-009 — The workspace releases as one coordinated, lockstep version set
- **Statement:** Because every Ferric crate inherits `version.workspace = true`, a release bumps them
  together to one shared version and publishes them as a single coordinated set, rather than versioning or
  releasing a crate on its own.
- **Rationale:** The workspace already pins a single `[workspace.package] version` that all three crates
  inherit, so the crates version in lockstep by construction; the release design honors that rather than
  fighting it. A single shared version across companion crates is the model Ferric's peers use (Leptos,
  Dioxus, Yew version their core and companion crates together), and it keeps the intra-workspace `version`
  requirements of `REQ-REL-004` trivially coherent. The detailed compatibility rules of the lockstep policy
  are owned by `REQ-REL-040..059`; this requirement fixes only the release mechanics: one version, released
  as a set.
- **Acceptance criteria:**
  - A release carries one version across all Ferric crates (the inherited `[workspace.package] version`),
    and all are published from the same release. (`doc_inspection`)
  - No Ferric crate is published at a version out of step with its siblings for the same release.
    (`publishability`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-004`, `REQ-REL-008`; `REQ-REL-040..059` (MSRV and bump meaning); `REQ-REL-020..039`
  (breaking-vs-additive classification).

#### REQ-REL-010 — A documented, repeatable release procedure
- **Statement:** The repository documents the steps to cut a coordinated workspace release — set the version,
  the publish order, the per-crate dry-run, and the publish — so a release can be performed the same way by
  someone who did not design it.
- **Rationale:** A three-crate release with an ordering constraint and a shared version is error-prone when
  it lives in one person's memory; a written procedure is what makes the release repeatable and reviewable.
  This is a documentation requirement — whether the steps are later encoded in a script or a CI job is left
  to a code DAG.
- **Acceptance criteria:**
  - A reviewer finds a release procedure in the repository that states the version bump, the
    `ferric`-before-`ferric-router` publish order, the per-crate `cargo publish --dry-run` check, and the
    publish step. (`doc_inspection`)
  - Following the documented steps on the current workspace reaches a successful set of per-crate dry-runs.
    (`publishability`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-005`, `REQ-REL-008`, `REQ-REL-009`; `REQ-REL-020..039` (the changelog practice).

### Published-manifest hygiene

#### REQ-REL-011 — Published manifests carry only resolvable crates.io version requirements
- **Statement:** Every dependency recorded in a published crate's manifest resolves from crates.io at a real
  version requirement — no path-only or git-only requirement reaches the published crate.
- **Rationale:** A consumer running `cargo add ferric` gets the published manifest; any dependency there that
  cannot resolve from crates.io makes the crate uninstallable, defeating `REQ-FW-001`. The path→version
  conversion of `REQ-REL-004` is the specific case; this requirement is the general invariant across all
  dependencies (including the direct crates.io dependencies `ferric-http` already declares). The Rust API
  Guidelines' C-PERMISSIVE note that a crate's distribution is constrained by its dependencies' licenses
  also lands here: a published crate should depend only on permissively-licensed, registry-resolvable crates.
- **Acceptance criteria:**
  - In each crate's packaged manifest, every normal (non-dev) dependency carries a version requirement
    resolvable from crates.io; none is path-only or git-only. (`publishability`)
  - A `cargo add` of each published crate into a fresh project resolves and builds without a git or path
    override. (`automated_test`)
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-004`; `REQ-FW-001`; `REQ-FW-013`.

---

## 2. Semver Contract & Changelog Practice (`REQ-REL-020..033`)

### A. The surface the semver contract governs

#### REQ-REL-020 — The public API each crate's semver promise covers is defined and enumerable
- **Statement:** Each Ferric crate has a defined public API — the set of items an app author may name —
  that the semver contract governs, and that set is the crate's `prelude` module (plus `ferric-http`'s
  top-level re-exports). Items reachable only through a non-public path or marked `#[doc(hidden)]` are
  internal and carry no semver promise.
- **Rationale:** Breaking-vs-additive is undecidable without a fixed boundary for what is public. The
  baseline already froze these `prelude` surfaces; the contract names them so a reviewer (and a mechanical
  checker) can enumerate exactly what must not break silently. Drawing the line at the `prelude` also keeps
  the internal `todo!()` bodies and helper modules free to change at patch level.
- **Acceptance criteria:** For each of `ferric`, `ferric-router`, `ferric-http`, the repository states which
  module(s) constitute the public API the semver contract covers, and the enumerated set matches the items
  actually exported there. A reviewer can list the governed items from the documentation and confirm the
  list equals the crate's exported public surface; a tool that snapshots the public API (for example
  `cargo-semver-checks`'s baseline, or `cargo public-api`) reports the same set.
- **Verification:** doc_inspection, automated_test
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-FW-009, REQ-FW-010 (read-only); REQ-REL-040..059 (frozen-`prelude` stability promise).
  Standards: Cargo book "SemVer Compatibility".

### B. The semver level contract

#### REQ-REL-021 — Each semver level has a defined meaning for the public surface
- **Statement:** For the 1.0 line, a **MAJOR** bump signals that the release contains at least one breaking
  change to the public surface; a **MINOR** bump signals only additive, backward-compatible changes (new
  public items, no removals or redefinitions); a **PATCH** bump signals no change to the public surface —
  bug fixes and behavior-preserving internal changes only.
- **Rationale:** This is what "governed by semver" in REQ-FW-010 means in practice: the number is a
  contract, not a label. Library authors (persona A3) decide whether an upgrade is safe to take
  automatically from the level, so the level must carry a defined, enforced meaning.
- **Acceptance criteria:** The repository documents, for the 1.0 line, what MAJOR, MINOR, and PATCH each
  permit and forbid with respect to the public surface, consistent with Semantic Versioning 2.0.0 and the
  Cargo book's SemVer chapter. A reviewer can map any classified change kind in §2C to exactly one required
  minimum level using this statement.
- **Verification:** doc_inspection
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-FW-010 (read-only); REQ-REL-040..059 (the 0.x→1.0 transition and cross-crate
  coordination). Standards: Semantic Versioning 2.0.0; Cargo book "SemVer Compatibility".

#### REQ-REL-022 — The pre-1.0 (0.x) semver convention is stated as the bridge to 1.0
- **Statement:** While Ferric is below 1.0 (today `0.1.0`), the Cargo 0.x convention holds: the leftmost
  non-zero component carries the breaking-change signal, so a breaking change bumps the **minor** (`0.y`)
  and an additive change bumps the **patch** (`0.y.z`). The contract states this so that pre-1.0 releases
  are versioned under a defined rule, and names 1.0 as the point at which the standard `MAJOR.MINOR.PATCH`
  meaning in `REQ-REL-021` takes effect.
- **Rationale:** Ferric is at `0.1.0` and will make pre-1.0 releases before the 1.0 cut; without a stated
  0.x rule those releases are versioned ad hoc. Cargo and SemVer 2.0.0 both define the 0.x behavior;
  stating it removes the ambiguity.
- **Acceptance criteria:** The repository states the 0.x versioning rule (breaking → minor bump, additive →
  patch bump) and the point at which 1.0 switches to the standard meaning. A reviewer can version a
  hypothetical pre-1.0 breaking change and a pre-1.0 additive change correctly from the statement.
- **Verification:** doc_inspection
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-REL-021; REQ-REL-040..059 (the actual 0.x→1.0 transition mechanics). Standards:
  Semantic Versioning 2.0.0 §4; Cargo book "SemVer Compatibility".

### C. Breaking-vs-additive classification

#### REQ-REL-023 — Breaking (major) changes are classified
- **Statement:** A change to the public surface is **breaking** (requires a MAJOR bump at 1.0) when it can
  stop a conforming downstream crate from compiling or change the documented behavior it relies on. The
  classification names at least: removing or renaming a public item (function, type, trait, variant, field,
  module, or re-export); changing a function or method signature (parameters, return type, or a tightened
  bound); adding a required trait method without a default; adding a variant to an enum that is not
  `#[non_exhaustive]`; adding a field to, or making newly public, a struct whose existing fields are all
  public and that is not `#[non_exhaustive]`; and making a previously public item private or `#[doc(hidden)]`.
- **Rationale:** This is the operative content of the semver contract — the list a reviewer applies to a
  pull request. It is grounded in the Cargo book's "major change" catalogue and RFC 1105 so the
  classification matches what the Rust ecosystem already means by a breaking change. REQ-FW-009 promises the
  frozen `prelude` surfaces are carried forward additively; this requirement defines precisely what would
  violate that promise.
- **Acceptance criteria:** The repository lists the change kinds that require a MAJOR bump, covering at least
  the cases above and marked as consistent with the Cargo book's SemVer chapter. A reviewer handed a diff
  of the public surface can classify each changed item as breaking or not using the list, and
  `cargo-semver-checks` (`REQ-REL-026`) flags the mechanically detectable members.
- **Verification:** doc_inspection, automated_test
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-FW-009 (read-only); REQ-REL-026. Standards: Cargo book "SemVer Compatibility"; RFC 1105.

#### REQ-REL-024 — Additive (minor) changes are classified
- **Statement:** A change is **additive** (requires only a MINOR bump at 1.0) when it extends the public
  surface without breaking a conforming downstream crate. The classification names at least: adding a new
  public item (a new export, function, method, type, or module); adding a defaulted method to an existing
  trait; adding a variant to a `#[non_exhaustive]` enum; and adding a field to a `#[non_exhaustive]` struct
  or one with a private field.
- **Rationale:** REQ-FW-009 commits the frozen surfaces to additive-only evolution; this requirement defines
  what "additive" admits, so "extend, never silently redefine" is checkable. It is the counterpart to
  `REQ-REL-023`: together they let a reviewer sort any public-surface change into exactly one of breaking,
  additive, or neither.
- **Acceptance criteria:** The repository lists the change kinds that are additive and permitted in a MINOR
  release, covering at least the cases above and consistent with the Cargo book's SemVer chapter. A reviewer
  can confirm a stated additive change is classified MINOR and requires no MAJOR bump.
- **Verification:** doc_inspection, automated_test
- **Stage:** usable_early · **Release gating:** true
- **References:** REQ-FW-009 (read-only); REQ-REL-027. Standards: Cargo book "SemVer Compatibility"; RFC 1105.

#### REQ-REL-025 — Patch changes, and the `todo!()` fill-in case, are classified
- **Statement:** A change is a **PATCH** when it leaves the public surface unchanged — a bug fix, a
  documentation correction, or a behavior-preserving internal change. Filling in a body that was `todo!()`
  at the baseline (making a frozen-signature function actually do what its signature and docs describe) is a
  PATCH when the signature and documented contract are unchanged: the `todo!()` panic was never a promised
  behavior, so replacing it with the specified behavior fixes a defect. A fill-in that must change the
  signature or the documented behavior to work is classified by `REQ-REL-023`/`REQ-REL-024` instead.
- **Rationale:** Ferric is distinctive here: its public shape is frozen but many bodies are still `todo!()`,
  so a large part of the 1.0 work is filling them in. Without an explicit rule, each fill-in invites a
  case-by-case semver argument. Anchoring the rule to "the signature and documented contract are the
  promise, the panic is not" keeps the common case at PATCH.
- **Acceptance criteria:** The repository states the PATCH classification and the `todo!()` fill-in rule. A
  reviewer can take a stated example — a function whose `todo!()` body is implemented to its documented
  signature — and confirm it is classified PATCH, and take a second example whose correct implementation
  requires a signature change and confirm it is classified MAJOR.
- **Verification:** doc_inspection
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-REL-023, REQ-REL-024. Standards: Cargo book "SemVer Compatibility".

#### REQ-REL-026 — A breaking change is caught mechanically, not by reviewer memory alone
- **Statement:** The classification in `REQ-REL-023` is enforced by a mechanical check that compares a
  release's public API against the previously published version and reports any breaking change.
  `cargo-semver-checks` is named as that check.
- **Rationale:** A written classification that depends on every reviewer remembering it fails silently. The
  ecosystem's standard answer is `cargo-semver-checks`, which lints the public API diff against the SemVer
  rules; naming it as the required check turns "we classify breaking changes" into "a break that is not
  matched by a MAJOR bump is reported before release." This is a requirement's acceptance method — the
  implementation is a later code DAG's work.
- **Acceptance criteria:** `cargo-semver-checks` (or an equivalent that lints the public-API diff against
  the SemVer rules) runs against each crate comparing the release to its prior published version, and a
  release whose public-surface change is breaking but whose version bump is not MAJOR is reported as a
  violation. A reviewer can confirm the check is part of the stated release procedure.
- **Verification:** automated_test
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-REL-023; REQ-REL-001..019 (the release procedure). Standards: `cargo-semver-checks`;
  Cargo book "SemVer Compatibility".

#### REQ-REL-027 — Public enums and structs are future-proofed so later additions stay additive
- **Statement:** Public enums and structs on the frozen surface that are expected to gain members after 1.0
  are marked `#[non_exhaustive]` (or given an equivalent sealed-extension pattern) before the 1.0 freeze, so
  that adding a variant or field later is an additive (MINOR) change. `EventKind` (new event kinds will be
  added as event coverage grows) and `Method` are the named candidates; the contract states which public
  types are frozen as exhaustive and which are open.
- **Rationale:** This decision can only be made *before* 1.0: once an exhaustive enum ships, adding a
  variant is a breaking change forever on that line. REQ-FW-006 commits the 1.0 to the full DOM event range,
  which means `EventKind` will grow; marking it `#[non_exhaustive]` now is what lets that growth stay
  additive under `REQ-REL-024`.
- **Acceptance criteria:** The repository states, for each public enum and struct on the governed surface,
  whether it is frozen exhaustive or `#[non_exhaustive]`, with a reason for the types expected to grow (at
  least `EventKind` and `Method`). A reviewer can confirm a stated post-1.0 addition of an `EventKind`
  variant is classified MINOR under this decision, and `cargo-semver-checks` agrees.
- **Verification:** doc_inspection, automated_test
- **Stage:** usable_early · **Release gating:** true
- **References:** REQ-FW-006, REQ-FW-009 (read-only); REQ-REL-024, REQ-REL-026. Standards: Cargo book
  "SemVer Compatibility"; Rust API Guidelines (future-proofing).

#### REQ-REL-028 — Behavior documented for a public item is part of the semver contract
- **Statement:** The semver contract covers documented behavior, not only type signatures. A change that
  keeps a signature intact but alters the behavior its documentation promises, in a way a conforming
  downstream crate relies on, is a breaking change under `REQ-REL-023`.
- **Rationale:** Signature-only semver misses the breaks that hurt most: a router that silently changes how
  it matches a path parameter, or a client that changes how it treats a non-2xx status, breaks callers
  without changing a single type. Tying the contract to documented behavior is what the Cargo book's
  "behavioral changes" guidance and RFC 1105 intend.
- **Acceptance criteria:** The repository states that documented behavior of a public item is covered by the
  semver contract and classified as breaking when changed incompatibly. A reviewer can take a stated
  behavior-only change to a public item and confirm it is classified MAJOR even though the signature is
  unchanged.
- **Verification:** doc_inspection
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-REL-023. Standards: Cargo book "SemVer Compatibility" (behavioral changes); RFC 1105.

#### REQ-REL-029 — Removal of a public item is preceded by a deprecation
- **Statement:** A public item is not removed without first being marked `#[deprecated]`, with a note naming
  the replacement, in a released MINOR version; the removal itself happens only in a later MAJOR release.
  Adding the `#[deprecated]` attribute is an additive (MINOR) change; the removal is a breaking (MAJOR)
  change recorded in the changelog under `REQ-REL-031`.
- **Rationale:** A 1.0 that removes items abruptly breaks the library authors (A3) it promised stability to,
  even when the removal is correctly signalled by a MAJOR bump. Deprecate-then-remove gives downstream crates
  a release in which the item still works but the compiler warns, so the upgrade can be staged.
- **Acceptance criteria:** The repository states the deprecate-before-remove policy and the semver level of
  each step. A reviewer can confirm a stated removal was preceded by a `#[deprecated]` item in an earlier
  MINOR release whose changelog recorded the deprecation.
- **Verification:** doc_inspection, automated_test
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-FW-010 (read-only); REQ-REL-023, REQ-REL-031. Standards: Rust API Guidelines; Cargo
  book "SemVer Compatibility".

### D. Changelog practice

The repository has no `CHANGELOG.md` today. The practice below follows Keep a Changelog 1.1.0 and ties each
entry to the semver level the change demands.

#### REQ-REL-030 — Every release records its changes in a maintained changelog
- **Statement:** Ferric maintains a changelog that records each release's changes in human-readable form,
  kept current as part of making a release, following the Keep a Changelog 1.1.0 structure: a
  reverse-chronological list of released versions each with its release date, plus an `Unreleased` section
  at the top for changes not yet released.
- **Rationale:** REQ-FW-010 and journey JA5 require that an author can "read a changelog, understand what a
  version bump changes, and upgrade without silent breakage." No changelog exists today, so the practice is
  designed from zero. Keep a Changelog is the ecosystem's common structure and is what an evaluating author
  (A1) expects to find when judging whether the project is maintained.
- **Acceptance criteria:** The repository contains a changelog in Keep a Changelog form with per-version,
  date-stamped sections in reverse-chronological order and an `Unreleased` section; a reviewer can confirm
  a released version's section exists and matches that release.
- **Verification:** doc_inspection
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-FW-010 (read-only); JA5. Standards: Keep a Changelog 1.1.0.

#### REQ-REL-031 — Each changelog entry states change type, affected crate, and whether it is breaking
- **Statement:** Each changelog entry is grouped under a Keep a Changelog change type — `Added`, `Changed`,
  `Deprecated`, `Removed`, `Fixed`, `Security` — names the crate(s) it affects, and marks whether the
  change is breaking or additive per the §2C classification.
- **Rationale:** A changelog that only lists commit subjects does not answer the author's real question —
  "will this upgrade break me, and where?" Grouping by change type and marking breaking-vs-additive makes
  the entry carry the §2C classification. Naming the affected crate matters because the workspace ships
  three crates that version together (`REQ-REL-032`).
- **Acceptance criteria:** A reviewer can take a release's changelog section and, for each entry, read its
  change type, the crate(s) affected, and a breaking/additive marking; a `Removed` or breaking `Changed`
  entry is marked breaking and an `Added` entry is marked additive, consistent with §2C.
- **Verification:** doc_inspection
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-REL-023, REQ-REL-024, REQ-REL-032. Standards: Keep a Changelog 1.1.0.

#### REQ-REL-032 — A release's semver level is justified by its changelog entries
- **Statement:** The semver level of a release is consistent with, and derivable from, its changelog
  entries: a release containing any `Removed` entry or any breaking-marked `Changed` entry is a MAJOR
  release; a release whose strongest entry is an additive `Added` or `Deprecated` is a MINOR release; a
  release containing only `Fixed` (and no public-surface change) is a PATCH release. The changelog and the
  version bump cannot disagree.
- **Rationale:** This makes the changelog the human-readable justification of the version number, and the
  version number a claim the changelog must support. Where `cargo-semver-checks` proves the code's break is
  signalled, this proves the recorded change set agrees.
- **Acceptance criteria:** A reviewer can take any released version, read its changelog section, and confirm
  the version-level bump from the prior release is the minimum level the entries require under
  `REQ-REL-021`; a release with a `Removed` entry but a non-MAJOR bump is a detectable inconsistency.
- **Verification:** doc_inspection
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-REL-021, REQ-REL-026, REQ-REL-031. Standards: Keep a Changelog 1.1.0; Semantic
  Versioning 2.0.0.

#### REQ-REL-033 — The changelog covers the multi-crate workspace, with per-crate changes attributable
- **Statement:** The changelog records changes for all three crates (`ferric`, `ferric-router`,
  `ferric-http`), and a reader can attribute each change to the crate it affects. Because the crates version
  in lockstep today (`version.workspace = true`), a single workspace-level `CHANGELOG.md` with per-crate
  grouping within each release satisfies this; should a crate ever version independently, that crate carries
  its own `CHANGELOG.md`. The requirement fixes the outcome — per-crate attribution across the workspace —
  not a single file layout.
- **Rationale:** A workspace that ships three crates from one repository can hide which crate a change
  touched if the changelog is flat. The lockstep versioning observed at the baseline makes one workspace
  changelog with per-crate sections the simple fit. The cross-crate *version-coordination* rule itself is
  owned by `REQ-REL-040..059`; this requirement governs only how that coordination is recorded.
- **Acceptance criteria:** A reviewer can find, for a given release, the changes to each of the three crates
  and tell which crate each entry affects; where crates version in lockstep a single workspace changelog
  with per-crate grouping is accepted.
- **Verification:** doc_inspection
- **Stage:** one_oh · **Release gating:** true
- **References:** REQ-REL-030, REQ-REL-031; REQ-REL-040..059 (cross-crate version coordination).
  Standards: Keep a Changelog 1.1.0.

---

## 3. Versioning & Compatibility Policy (`REQ-REL-040..047`)

### Policy summary

1. **MSRV.** 1.0 states a supported minimum Rust version and keeps it as a declared floor. The floor may
   rise within 1.x, but only as a minor release, announced, and never silently.
2. **Frozen surfaces.** The public `prelude` surfaces carried from the baseline remain usable for the life
   of the 1.x line. Code that compiles against them at 1.0 keeps compiling against every later 1.x.
3. **Additive-only.** Within 1.x the public surface grows by addition. Removing or redefining a public item
   is a breaking change, and a breaking change is deferred to a future major (2.0).
4. **Cross-crate coordination.** `ferric`, `ferric-router`, and `ferric-http` version in lockstep and
   release together at one version.
5. **Stated scope.** The promise covers the documented public surfaces. Items outside them — anything not
   re-exported in a `prelude`, the runtime behaviour of leaf bodies still `todo!()` at a given release, and
   anything behind a future unstable flag — are named as outside the 1.x stability promise.

#### REQ-REL-040 — 1.0 states a supported minimum Rust version (MSRV)
- **Statement:** Ferric 1.0 declares a minimum supported Rust version and every published crate builds on
  that version of stable Rust. The declared MSRV is discoverable from the crates themselves, not only from
  prose.
- **Rationale:** A "1.0" that does not say which Rust versions it supports leaves an app author guessing
  whether an upgrade of either Ferric or their toolchain will compile. The workspace already pins
  `rust-version = "1.82"` in `[workspace.package]`, so the floor exists; the requirement is that 1.0
  commits to it as a promise. The `rust-version` field (RFC 2495, *the minimum-supported-rust-version
  field*) is the ecosystem's standard way to state this, and Cargo's MSRV-aware resolver (RFC 3537) reads
  it during dependency resolution. Peer frameworks declare an MSRV through this field and track recent
  stable: Leptos ≈ 1.88, Yew ≈ 1.84, and Dioxus ≈ 1.83 (as reported by their published crates);
  Ferric's 1.82 is consistent with that range.
- **Acceptance criteria:**
  - `doc_inspection`: each published crate carries a `rust-version` field, and the repository states the
    supported-MSRV promise where an author will find it (the compatibility policy of `REQ-REL-046`).
  - `automated_test`: a build of each crate on the declared MSRV toolchain targeting
    `wasm32-unknown-unknown` succeeds.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-003` (stable Rust + MSRV), `REQ-FW-010` (compatibility policy);
  `REQ-REL-046` (where the promise is written).

#### REQ-REL-041 — How the MSRV may rise within the 1.x line
- **Statement:** The supported MSRV may increase within 1.x, but only in a minor release, and each increase
  is recorded so an author can see it before upgrading. Raising the MSRV within 1.x is not treated as a
  major-breaking change and does not require a 2.0.
- **Rationale:** Pinning an MSRV for the life of a major line would freeze Ferric out of compiler
  improvements for years; raising it without warning would break downstream builds silently. The Cargo
  Book's *SemVer Compatibility* chapter classifies increasing the minimum required Rust version as a
  possibly-breaking change, and the Rust API Guidelines MSRV discussion (rust-lang/api-guidelines #231)
  gives the convention that a ≥1.0 crate raising its MSRV bumps the minor version rather than the major.
  A Ferric minor upgrade may ask for a newer Rust; a patch upgrade never will.
- **Acceptance criteria:**
  - `doc_inspection`: the compatibility policy states that an MSRV increase occurs only in a minor (or
    major) release, never a patch, and is announced in the changelog.
  - `automated_test`: for any release that raises the declared `rust-version`, the version bump is at
    least a minor bump.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-020..039` (semver level of a change; changelog practice); `REQ-FW-010`.

#### REQ-REL-042 — The frozen `prelude` surfaces stay usable across 1.x
- **Statement:** The public `prelude` surfaces carried from the baseline remain usable for the whole 1.x
  line. Code that compiles against the 1.0 preludes continues to compile against every later 1.x release
  without change.
- **Rationale:** This is the policy side of `REQ-FW-009`. The baseline already froze the public shape of
  three preludes: `ferric::prelude` (`Component`, `mount`, `batch`, `create_effect`, `create_memo`,
  `create_signal`, `Memo`, `Signal`, `EventKind`, `VElement`, `VNode`); `ferric_router::prelude` (`link`,
  `Params`, `RoutePattern`, `Router`, `navigate`, `current_path`); and `ferric_http`'s top-level re-exports
  plus `prelude` (`Client`, `ClientBuilder`, `Error`, `Method`, `RequestBuilder`, `Response`, `Result`).
  Apps are already written against these. Many leaf bodies are still `todo!()`, but the public *shape* is
  fixed; this requirement is about that shape.
- **Acceptance criteria:**
  - `automated_test`: an example (or a `trybuild`-style compile test) that imports and uses the baseline
    `prelude` items of all three crates compiles against the 1.0 release and against a later 1.x release
    unchanged.
  - `doc_inspection`: the compatibility policy names the `prelude` surfaces as the guaranteed-stable
    public API for 1.x.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-009` (frozen surfaces additive); `REQ-REL-043` (additive-only evolution).

#### REQ-REL-043 — The public surface evolves by addition only within 1.x
- **Statement:** Within the 1.x line the public API grows by addition — new exports, new methods, new
  optional parameters handled compatibly. Removing a public item, renaming it, or changing its type or
  documented behaviour is a breaking change, and a breaking change is deferred to a future major version
  (2.0).
- **Rationale:** Additive-only evolution is what lets an author upgrade a minor release and trust their
  code still builds; it is the discipline that keeps the walking skeleton a dependable base while leaf
  bodies get implemented. The classification of which concrete changes are additive versus breaking is the
  semver contract's (`REQ-REL-020..039`); this requirement is the *commitment* that, having that
  classification, 1.x only ever makes additive changes and defers every breaking one. Peer frameworks
  follow the same rule across their crate families — Leptos, Dioxus, and Yew each reserve breaking
  public-API changes for a new major.
- **Acceptance criteria:**
  - `automated_test`: a public-API check (for example `cargo-semver-checks`) run between two 1.x releases
    reports no removed or redefined public item.
  - `doc_inspection`: the compatibility policy states the additive-only rule for 1.x and that
    removals/redefinitions are deferred to 2.0.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-009`; `REQ-REL-020..039` (breaking-vs-additive classification, the semver gate);
  `REQ-REL-042`.

#### REQ-REL-044 — The crates version in lockstep and release together
- **Statement:** `ferric`, `ferric-router`, and `ferric-http` share one version number and are released
  together: every 1.0 release publishes all of them at the same version. 1.0 keeps the workspace's lockstep
  versioning (`version.workspace = true`) rather than letting the crates drift to independent version
  numbers.
- **Rationale:** The workspace inherits a single `[workspace.package] version` through `version.workspace =
  true`. Lockstep is the simplest contract an app author can hold: one version number describes the whole
  set, and there is no matrix of "which `ferric-router` works with which `ferric`" to reason about. This
  matches how the peer families ship — Leptos, Dioxus, and Yew coordinate their companion crates at a
  shared version within a release. Independent per-crate versioning (as in the Tokio family) buys finer-
  grained release cadence at the cost of a compatibility matrix; for a three-crate set that is designed and
  tested as one framework, lockstep is the better trade.
- **Acceptance criteria:**
  - `doc_inspection`: the compatibility policy states lockstep versioning for the 1.x line; the workspace
    manifest shows `version.workspace = true` and each crate inherits it.
  - `publishability`: a coordinated workspace release publishes `ferric`, `ferric-router`, and `ferric-http`
    at the same version.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-010` (version-coordination rule); `REQ-REL-001..019` (multi-crate release order);
  `REQ-REL-045`.

#### REQ-REL-045 — Cross-crate compatibility an app author can rely on
- **Statement:** An app author can depend on `ferric`, `ferric-router`, and `ferric-http` at the same 1.x
  version and trust that the three compose without version conflicts: a `ferric-router` 1.x and a
  `ferric-http` 1.x work with `ferric` 1.x, because the set is built, tested, and released together at
  that version.
- **Rationale:** The compatibility an author needs is simple to state under lockstep: pick a 1.x version,
  use it for all three. This is the compatibility half of `REQ-FW-010` and the stability leg of journey
  JA5 ("rely on a public API that does not shift … with a documented compatibility policy across the
  multi-crate workspace").
- **Acceptance criteria:**
  - `example_build`: a stated example app that depends on all three crates at the same 1.x version builds
    and runs the described behaviour (routing + a JSON request + rendering) over their public surfaces.
  - `doc_inspection`: the compatibility policy states that the three crates are compatible when taken at
    the same 1.x version and that this is the supported way to depend on the set.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-007`, `REQ-FW-010`; `REQ-REL-044`; `REQ-REL-001..019`.

#### REQ-REL-046 — The compatibility policy is written down and discoverable
- **Statement:** The versioning and compatibility policy for 1.0 — the MSRV promise and its raise rule,
  the frozen-surface guarantee, additive-only evolution, and cross-crate coordination — is written in the
  repository where an author will find it before depending on or upgrading Ferric.
- **Rationale:** `REQ-FW-010` requires a *documented* compatibility policy: a reader must be able to learn,
  from the repository, what a given version bump is allowed to change. Stating the policy in one
  discoverable place (a `COMPATIBILITY.md`, a documented section of the top-level README, or the published
  crate docs) turns the preceding requirements from internal intent into a promise an author can hold the
  project to.
- **Acceptance criteria:**
  - `doc_inspection`: a reviewer finds, in the repository, a stated compatibility policy covering MSRV and
    its raise rule, the frozen-`prelude` guarantee, additive-only 1.x evolution, and lockstep cross-crate
    versioning; it is linked from where an author starts (README / crate docs).
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-010`; `REQ-REL-040..047` (the policy content it collects);
  `REQ-REL-020..039` (semver + changelog it sits beside).

#### REQ-REL-047 — What the 1.x promise does not cover (stated, not implied)
- **Statement:** The 1.x stability promise covers the documented public `prelude` surfaces. It does not
  cover: items not re-exported in a `prelude`; the runtime behaviour of leaf bodies still `todo!()` at a
  given release, until the work-item that implements them lands; and anything placed behind a future unstable
  or experimental feature flag. These exclusions are named in the compatibility policy.
- **Rationale:** The baseline ships a fixed public *shape* with many `todo!()` leaf bodies, so a stability
  promise that did not separate "this API shape is stable" from "this behaviour is implemented" would either
  overclaim or get quietly narrowed later. Naming the exclusions is what keeps the promise accurate. A
  hidden cut — discovering after adoption that some re-exported-looking item was never in scope — costs more
  trust than an exclusion stated at 1.0.
- **Acceptance criteria:**
  - `doc_inspection`: the compatibility policy states that the stability guarantee applies to the documented
    `prelude` surfaces, and names the three exclusions (non-prelude items, not-yet-implemented `todo!()`
    behaviour, and future unstable-flagged APIs).
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-REL-042` (the guaranteed surface); `REQ-FW-009`; `REQ-REL-046`.

---

## Full register summary

| Id | Concern | Facet | Stage | Gating | Verification |
|---|---|---|---|---|---|
| REQ-REL-001 | Every published crate declares a permissive license | publishing | one_oh | true | doc_inspection, publishability |
| REQ-REL-002 | Complete per-crate crates.io metadata | publishing | one_oh | true | doc_inspection, publishability |
| REQ-REL-003 | API documentation builds on docs.rs | publishing | one_oh | true | automated_test, doc_inspection |
| REQ-REL-004 | Intra-workspace dependencies carry path + version | publishing | one_oh | true | doc_inspection, publishability, automated_test |
| REQ-REL-005 | Each crate passes `cargo publish --dry-run` | publishing | one_oh | true | publishability |
| REQ-REL-006 | Packaged crate contains license, README, intended files | publishing | one_oh | true | doc_inspection, publishability |
| REQ-REL-007 | Crate names are owned on crates.io | publishing | one_oh | true | doc_inspection |
| REQ-REL-008 | Crates publish in dependency order | publishing | one_oh | true | doc_inspection, publishability |
| REQ-REL-009 | Workspace releases as one coordinated lockstep version set | publishing | one_oh | true | doc_inspection, publishability |
| REQ-REL-010 | A documented, repeatable release procedure | publishing | one_oh | true | doc_inspection, publishability |
| REQ-REL-011 | Published manifests carry only resolvable version requirements | publishing | one_oh | true | publishability, automated_test |
| REQ-REL-020 | The public API the semver promise covers is defined | semver contract | one_oh | true | doc_inspection, automated_test |
| REQ-REL-021 | Each semver level has a defined meaning for the public surface | semver contract | one_oh | true | doc_inspection |
| REQ-REL-022 | The pre-1.0 (0.x) semver convention is stated | semver contract | one_oh | true | doc_inspection |
| REQ-REL-023 | Breaking (major) changes are classified | breaking-vs-additive | one_oh | true | doc_inspection, automated_test |
| REQ-REL-024 | Additive (minor) changes are classified | breaking-vs-additive | usable_early | true | doc_inspection, automated_test |
| REQ-REL-025 | Patch changes and `todo!()` fill-in case are classified | breaking-vs-additive | one_oh | true | doc_inspection |
| REQ-REL-026 | A breaking change is caught mechanically (`cargo-semver-checks`) | breaking-vs-additive | one_oh | true | automated_test |
| REQ-REL-027 | Public enums/structs are future-proofed `#[non_exhaustive]` | breaking-vs-additive | usable_early | true | doc_inspection, automated_test |
| REQ-REL-028 | Documented behavior is part of the semver contract | semver contract | one_oh | true | doc_inspection |
| REQ-REL-029 | Removal of a public item is preceded by a deprecation | semver contract | one_oh | true | doc_inspection, automated_test |
| REQ-REL-030 | Every release records its changes in a maintained changelog | changelog | one_oh | true | doc_inspection |
| REQ-REL-031 | Each changelog entry states change type, crate, and breaking flag | changelog | one_oh | true | doc_inspection |
| REQ-REL-032 | A release's semver level is justified by its changelog entries | changelog | one_oh | true | doc_inspection |
| REQ-REL-033 | The changelog covers the multi-crate workspace per crate | changelog | one_oh | true | doc_inspection |
| REQ-REL-040 | 1.0 states a supported MSRV | compat policy | one_oh | true | doc_inspection, automated_test |
| REQ-REL-041 | How the MSRV may rise within 1.x | compat policy | one_oh | true | doc_inspection, automated_test |
| REQ-REL-042 | The frozen `prelude` surfaces stay usable across 1.x | compat policy | one_oh | true | automated_test, doc_inspection |
| REQ-REL-043 | The public surface evolves by addition only within 1.x | compat policy | one_oh | true | automated_test, doc_inspection |
| REQ-REL-044 | The crates version in lockstep and release together | compat policy | one_oh | true | doc_inspection, publishability |
| REQ-REL-045 | Cross-crate compatibility an app author can rely on | compat policy | one_oh | true | example_build, doc_inspection |
| REQ-REL-046 | The compatibility policy is written down and discoverable | compat policy | one_oh | true | doc_inspection |
| REQ-REL-047 | What the 1.x promise does not cover (stated, not implied) | compat policy | one_oh | true | doc_inspection |

**Unused ranges (reserved, no gaps in used sequences):** REQ-REL-012..019, REQ-REL-034..039,
REQ-REL-048..059.

---

## Peer and standards comparison

The requirements above track the Rust ecosystem's own standards and what Ferric's named peers already do.
An evaluating author (A1) meets conventions they already know; the choices are not invented for Ferric.

**Reference standards.** Semantic Versioning 2.0.0 (semver.org) provides the `MAJOR.MINOR.PATCH` contract
and the 0.x rule. The Cargo book's *SemVer Compatibility* chapter is the Rust-specific catalogue of major-
vs-minor changes and is the basis for the breaking-vs-additive classification in §2C. RFC 1105 (*Rust API
evolution*) and the Rust API Guidelines provide the direction on behavioral changes, deprecation-before-
removal, and MSRV policy. RFC 2495 (*the minimum-supported-rust-version field*) and RFC 3537 (*the MSRV-
aware resolver*) are the mechanisms `REQ-REL-040` commits to using. Keep a Changelog 1.1.0 is the structure
`REQ-REL-030` requires. `cargo-semver-checks` is the mechanical gate `REQ-REL-026` names.

**Leptos.** Publishes to crates.io as a multi-crate workspace — `leptos`, `leptos_router`, `leptos_meta`,
and others — under MIT, versioned together within a release. Maintains a repository `CHANGELOG.md`
following Keep a Changelog conventions. Declares an MSRV through the `rust-version` field, sitting around
Rust 1.88 in its published crates. Made breaking changes across its `0.x` line under the Cargo 0.x
convention, which is why Ferric's `REQ-REL-022` states the pre-1.0 rule explicitly.

**Dioxus.** Similarly structured as a multi-crate workspace on crates.io (`dioxus`, `dioxus-router`, and
companions), published under MIT/Apache-2.0 dual-license, coordinated version releases. Maintains a
`CHANGELOG.md`. Declares an MSRV around Rust 1.83. Used `cargo-semver-checks` as part of its release
tooling — consistent with `REQ-REL-026`.

**Yew.** Published as `yew` and `yew-router` (and formerly `yew-agent`) under MIT/Apache-2.0,
version-coordinated. Documents releases in its project documentation and in GitHub Releases. Declares an
MSRV around Rust 1.84. Like Leptos and Dioxus, reserves breaking API changes for new major versions.

**Tokio** is named as the counter-example of independent per-crate versioning (`tokio`, `tokio-util`,
`tokio-stream` at different version numbers) with a stated MSRV policy. For a three-crate set that is
designed and tested as one framework, lockstep (as in `REQ-REL-044`) is the better trade — as Leptos,
Dioxus, and Yew each chose.

**Where Ferric differs from its peers.** Ferric's `todo!()`-body situation (`REQ-REL-025`) has no direct
peer analogue — it comes from shipping a frozen public shape ahead of its implementation. This is why the
contract classifies the fill-in case explicitly rather than borrowing a peer's rule.

---

## Metadata

- **Owning task:** `SUBTASK_rel_synthesis` (synthesis gate for concern 7, release & distribution).
- **Register:** `REQ-REL-001..011` (publishing readiness), `REQ-REL-020..033` (semver contract + changelog
  practice), `REQ-REL-040..047` (versioning & compatibility policy). All are `release_gating: true`.
- **Unused ranges (reserved):** `REQ-REL-012..019`, `REQ-REL-034..039`, `REQ-REL-048..059`.
- **Conforms to:** `../../contracts/id_and_traceability_schema.yaml` (REQ fields, `verification_method` +
  `stage` vocabularies, `release_gating`); `../contracts/_MANIFEST.yaml` (the `REQ-REL-*` partition and
  merge identities J1–J7).
- **Sources merged:** `SUBTASK_publishing/publishing.md` (REQ-REL-001..011),
  `SUBTASK_semver_changelog/semver-changelog.md` (REQ-REL-020..033),
  `SUBTASK_versioning_compat/versioning-compat.md` (REQ-REL-040..047).
- **Repo target:** `design/ferric/release-and-distribution.md` (placed by `SUBTASK_design_pr`).
- **Grounding commit:** `Nousix-LLC/nswe-demo` @ `276138f` (`main`).
