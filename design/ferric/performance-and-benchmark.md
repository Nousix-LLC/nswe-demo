# Ferric — Performance & Benchmark Plan

This document designs Ferric's performance bar: a reproducible **js-framework-benchmark** entry that measures
speed, memory, and bundle size, and a comparison method against named peers that states its conditions and keeps
measured results separate from expectations. It owns the `REQ-PERF-*` register (identity I1) and answers the
cross-cutting requirement `REQ-FW-012` (reproducible, named-peer performance characterization) and `REQ-FW-013`
(a lean, documented dependency footprint) from the side of the person deciding whether to adopt Ferric.

**What this document settles, and what it does not.** It specifies *what must be measured, under what conditions,
and how a stranger reruns it to reach comparable numbers* — the benchmark plan and the performance bar. It does
**not** write the benchmark implementation (the entry's Rust code, the build scripts, the CI wiring); that is an
implementation concern deferred to the performance work-item in the release plan. Every requirement here reads as
something a reviewer or a code DAG could check against the framework.

**What, not how.** A requirement states the measurement an author can rely on and the condition a reviewer can
confirm — "a keyed entry exists and runs in the suite and produces the standard operation durations" — not the
data structures the entry uses or the shape of its render loop. The entry's internals are the benchmark
implementation's to choose.

**Grounding.** Ferric's baseline is grounded in `Nousix-LLC/nswe-demo` at `276138f` on `main`: fine-grained
signal reactivity (`create_signal` / `create_memo` / `create_effect`, `batch`, scoped cleanup), components as
`Fn() -> VNode`, a **keyed** virtual-DOM diff applied to the live DOM through `web-sys`, `click`/`input` events,
and `dom::mount`; the only binding dependencies are `wasm-bindgen`, `web-sys`, `js-sys`. This reactivity-plus-keyed-VDOM
design is what makes performance a claim worth measuring rather than asserting: the keyed diff is exactly what the
benchmark's swap/remove/update operations exercise. Peer frameworks are named from their public, documented
behavior; the benchmark project referenced is the ecosystem-standard **js-framework-benchmark** (the
`krausest/js-framework-benchmark` project), whose Rust/WASM entries include **Leptos**, **Sycamore**, **Dioxus**,
and **Yew**.

**Measured vs. expected — the rule that governs every number in this document.** No measured result exists yet.
Every numeric figure Ferric might cite is a **target or a plan** until the entry has been run and its output
recorded. This document states targets and the method; it states **no measured number**. §8 fixes how measured
results, once produced, are reported and kept distinct from the targets in §7.

The sections below are: the benchmark entry (§1); what is measured (§2); the comparison method (§3); the
`REQ-PERF-*` register (§4); reproducibility (§5); the lean-footprint link (§6); targets, stated as targets (§7);
and the measured-vs-expected reporting discipline (§8).

---

## 1. The benchmark entry — what a Ferric entry in js-framework-benchmark is

*(section: `benchmark_entry`)*

js-framework-benchmark measures frameworks against one shared application: a table of rows, each row carrying a
numeric id and a randomly-assembled text label, driven by a fixed set of buttons. Every framework in the suite
implements the **same** app, so a Ferric entry is defined by implementing that app in Ferric and nothing more —
the app is the specification, not Ferric's to redesign.

### 1.1 The shape the entry must provide

A Ferric entry is a small Ferric application that exposes the suite's standard controls over a row table:

- **Create 1,000 rows** — replace the table with 1,000 freshly-built rows.
- **Create 10,000 rows** — replace the table with 10,000 rows.
- **Append 1,000 rows** — add 1,000 rows to the existing table.
- **Update every 10th row** — change the label of every 10th row in place.
- **Clear** — remove all rows.
- **Swap rows** — swap two rows in a populated table.
- **Select / remove a row** — clicking a row selects it; a per-row control removes it.

Each row is identified by its id, and the entry MUST be a **keyed** implementation: row reconciliation is by the
row's stable key, so swap, remove, and update move or mutate the existing DOM nodes for those keys rather than
rebuilding the list. This is the category the suite recommends as the one to compare, and it is the behavior
Ferric's keyed `VElement` diff already implements — the entry exercises Ferric as a real app author would use it
(signals holding the row list, a keyed list in the view, event handlers), not through a private fast path.

### 1.2 What the entry exercises

The entry is a real use of Ferric's public surface: the row list lives in a signal; the table is a keyed list
in the view; the buttons mutate the signal; `batch` coalesces the bulk mutations the way an app author would write
them. It MUST NOT reach past the public API into a bespoke benchmark-only rendering path — a number produced by a
path no app author can use is not a number about Ferric. (The authoring surface the entry is written against is
owned by `REQ-API-*`; this entry is a consumer of it, and once a `view!`-style macro lands the entry SHOULD be
written in it so the measured code is the code authors write.)

### 1.3 How the entry is run

The entry runs **inside the js-framework-benchmark harness, unmodified**: the project drives a Chromium browser
through its automation (chromedriver / the project's runner), performs the suite's warmup iterations, repeats each
operation the suite's configured number of times, and records per-operation durations and memory from the
browser's own timeline. Ferric does not supply its own timing code; using the suite's harness is what makes the
numbers comparable to every other entry. The run conditions that MUST be pinned and recorded are specified in §5.

---

## 2. What is measured

*(section: `what_is_measured`)*

Three metric families, each captured by an established mechanism. Ferric adds no measurement of its own beyond
what the suite and the standard build tooling already produce; the design choice is *which* metrics gate the 1.0,
not inventing new ones.

### 2.1 Speed — the CPU operations

The suite's duration metrics, captured by its harness from Chromium's timeline, with the suite's warmup and
repeat policy:

| Operation | What it times |
|---|---|
| Create 1,000 rows | Building 1,000 rows after load (no warmup). |
| Replace all rows | Replacing 1,000 rows with the create cycle (warmed). |
| Partial update | Updating the label of every 10th row in a 10,000-row table (warmed). |
| Select row | Highlighting a row on click (warmed). |
| Swap rows | Swapping two rows in a 1,000-row table (warmed). |
| Remove row | Removing one row from a 1,000-row table (warmed). |
| Create 10,000 rows | Building 10,000 rows (no warmup). |
| Append 1,000 rows | Appending 1,000 rows to a 10,000-row table (no warmup). |
| Clear rows | Clearing a 10,000-row table (no warmup). |

Swap, remove, and partial update are the operations that directly exercise Ferric's keyed diff; they are the ones
where a keyed VDOM is expected to show its cost and its benefit, and they MUST be reported individually, not
folded away into a single summary figure (see §3).

### 2.2 Memory

The suite's memory metrics, read from the browser after defined points:

| Metric | Captured after |
|---|---|
| Ready memory | Page load, before any operation. |
| Run memory | Creating 1,000 rows. |
| Update memory | Five update clicks on a 1,000-row table. |
| Replace memory | Five create-1,000 cycles. |
| Repeated clear memory | Creating and clearing 1,000 rows five times. |

Memory is where WASM and JS frameworks are least directly comparable (§3.3); the metric is still captured and
reported, with the caveat stated rather than omitted.

### 2.3 Bundle and startup size

Bundle size is first-order for a WASM framework because the bundle is **downloaded and compiled before the app
runs** — it is the cost an evaluating author (persona A1) weighs first. Captured as:

| Metric | What it is |
|---|---|
| Transfer size | The compressed (gzip, and brotli where the suite reports it) over-the-wire size of the entry's shipped assets — the WASM bundle plus its JS glue — for the optimized release build. |
| Uncompressed size | The on-disk size of the optimized `.wasm` and glue, for context. |
| Startup / time-to-interactive | The suite's startup metric (the Lighthouse-derived interactive measure the project reports). |

The optimized release build means the standard WASM release path: a `--release` build for `wasm32-unknown-unknown`,
`wasm-bindgen` processing, and a size pass with `wasm-opt`. The entry's transfer size MUST be the size of that
optimized artifact, not a debug build, because the debug number would understate nothing and overstate the cost by
a large factor — the reported figure must be the one an author actually ships.

---

## 3. The comparison method

*(section: `comparison_method`)*

The point of the entry is a comparison an evaluating author can trust: one that names its peers, states its
conditions, reports the whole picture rather than the flattering slice, and never presents an expectation as a
result. This section fixes that method; `REQ-PERF-006`–`REQ-PERF-009` make it checkable.

### 3.1 Which peers Ferric is compared against

- **Rust/WASM peers (the primary comparison):** **Leptos**, **Sycamore**, **Dioxus**, and **Yew** — the four
  frameworks an evaluating author weighs Ferric against, each with an existing keyed entry in the suite. Leptos is
  the standing reference for a fast Rust/WASM framework and is the demanding peer to be read against; Sycamore is
  the closest on the signal model; Dioxus and Yew are the VDOM-based Rust peers, architecturally nearest to
  Ferric's keyed-VDOM design.
- **JavaScript reference points (for context, not as the primary bar):** the suite's **vanillajs** baseline (the
  floor every framework is measured against), and the JS frameworks the Rust ecosystem is routinely read against —
  **Solid**, **Svelte**, **Vue**, and **React**. These place Ferric in the wider field; they are reported as
  context, with the memory caveat in §3.3 attached.

All comparisons use the **keyed** category. Non-keyed numbers are not mixed into the comparison.

### 3.2 The conditions that make a comparison valid

A comparison is only as good as the conditions it holds constant. The method requires that Ferric's entry and
every peer it is compared against be measured:

- against the **same js-framework-benchmark commit**;
- in the **same Chromium version**, on the **same machine**, in one run session;
- in the **keyed** category;
- through the suite's **own harness**, with the suite's warmup and repeat policy unchanged.

Where Ferric cites a peer number it did **not** measure in the same session (for example the project's published
results table), it MUST say so and cite the source and its date — a measured-here number and a cited-elsewhere
number are never presented as if produced under one condition.

### 3.3 Like-with-like, especially for memory

WASM and JS frameworks allocate and report memory differently; a raw memory ranking that mixes them invites a
wrong conclusion. The method therefore reads Ferric **first against its Rust/WASM peers** (Leptos, Sycamore,
Dioxus, Yew), where the comparison is like-with-like, and treats the JS frameworks as context with the
WASM-vs-JS difference stated. The same discipline applies to bundle size: a WASM bundle and a JS bundle are
different kinds of artifact, and the comparison names that rather than implying a single scale.

### 3.4 No cherry-picking — reporting the whole picture

The rules that keep the comparison from flattering Ferric:

- **Report every operation.** The full per-operation table (all of §2.1, §2.2, §2.3) is published, not a chosen
  subset. An operation where Ferric is slower is reported with the same prominence as one where it is faster.
- **Use the suite's own summary, not a hand-made one.** Where a single headline figure is given, it is the
  suite's **weighted geometric-mean slowdown** relative to the vanillajs baseline — the number the project
  computes for every entry the same way — not a Ferric-specific average that could be shaped by metric choice.
- **State the run conditions with the numbers.** Every published result carries the commit, Chromium version,
  machine class, and date (§5), so the reader can judge and reproduce it.
- **Separate measured from expected.** Targets (§7) and measured results (§8) live in distinct, labelled places;
  a target is never shown in the results table, and a measured number is never adjusted toward a target.
- **No silent omission.** If an operation cannot be measured (for example a suite operation the entry does not yet
  support), that is recorded as a stated gap, not left blank as if it did not exist.

---

## 4. Requirements & acceptance criteria (`REQ-PERF-*`)

*(section: `requirements_and_acceptance_criteria`)*

Each requirement states what must be true from the side of someone judging Ferric's performance, with acceptance
criteria a stranger could check. Verification methods are drawn from the contract's closed vocabulary; most are
`benchmark_run` (the entry runs and produces the stated numbers), with the comparison-method and publication
statements verified by `doc_inspection`. All carry `stage: one_oh` and back `REQ-FW-012`/`REQ-FW-013`.
Conformance to `../contracts/id_and_traceability_schema.yaml`.

### REQ-PERF-001 — A keyed js-framework-benchmark entry for Ferric
- **Statement:** Ferric has an entry in the js-framework-benchmark suite that implements the suite's standard row-table application through Ferric's public API, classified in the **keyed** category.
- **Rationale:** js-framework-benchmark is the ecosystem-standard way WASM and JS frameworks state rendering performance, and all of Ferric's Rust peers (Leptos, Sycamore, Dioxus, Yew) have keyed entries there. An entry in the same suite is what lets an evaluating author compare Ferric like-for-like; a bespoke benchmark would not be comparable.
- **Acceptance criteria:** The entry implements every standard operation (create 1,000 / 10,000, append 1,000, update every 10th, clear, swap, select, remove); it is written against Ferric's public surface with no benchmark-only private render path; it passes the suite's keyed-category validation; and it runs to completion in the suite's harness.
- **Verification:** `benchmark_run` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`; A1. Entry implementation → the performance work-item in the release plan.

### REQ-PERF-002 — Speed metrics captured across the standard operations
- **Statement:** The entry produces a duration for each of the suite's standard CPU operations, captured by the suite's own harness.
- **Rationale:** Rendering speed on create/update/swap/remove is the core of what the benchmark exists to measure, and the keyed operations (swap, remove, partial update) are exactly where Ferric's keyed VDOM is tested. Per-operation durations, not a single blended figure, are what let a reader see where Ferric is strong and where it is not.
- **Acceptance criteria:** A benchmark run yields a duration for each operation in §2.1, recorded with the suite's warmup/repeat policy and its central-tendency statistics (mean/median), as a published result row.
- **Verification:** `benchmark_run` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`; A1, A4.

### REQ-PERF-003 — Memory metrics captured
- **Statement:** The entry produces the suite's memory metrics (ready, run, update, replace, repeated-clear).
- **Rationale:** Memory footprint under sustained operation is a real adoption concern, and leaks or growth show up in the repeated-clear and update metrics. Capturing them — with the WASM-vs-JS caveat stated — lets a reader judge Ferric's memory behavior against its Rust peers.
- **Acceptance criteria:** A benchmark run yields each memory metric in §2.2 as a published result; the WASM-vs-JS comparability caveat (§3.3) is attached wherever these are compared to JS frameworks.
- **Verification:** `benchmark_run` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`; A4.

### REQ-PERF-004 — Bundle and startup size captured for the optimized build
- **Statement:** The entry's transfer size (compressed), uncompressed size, and the suite's startup metric are captured for the optimized WASM release build.
- **Rationale:** A WASM bundle is downloaded and compiled before the app runs, so size is the first cost an evaluating author sees; a number from a debug build would misstate it. Reporting the optimized-build size is reporting the cost an author actually ships.
- **Acceptance criteria:** Size figures are captured from a `--release`, `wasm-bindgen`-processed, `wasm-opt`-optimized build; transfer size is the compressed over-the-wire size of the shipped WASM plus glue; the suite's startup metric is recorded. The build path that produced the figure is documented.
- **Verification:** `benchmark_run` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`, `REQ-FW-013`; A1, A4.

### REQ-PERF-005 — A reproducible run a stranger can repeat
- **Statement:** A reader can rerun Ferric's benchmark from documented steps, with pinned versions, and reach numbers comparable to the published ones.
- **Rationale:** A benchmark number nobody else can reproduce is a claim, not evidence; reproducibility is what separates a measurement from an assertion, and it is explicit in `REQ-FW-012`'s acceptance.
- **Acceptance criteria:** The repository documents the exact rerun procedure — the js-framework-benchmark commit, the Rust toolchain version, the `wasm-bindgen`/`wasm-opt` versions, the target Chromium version, and the suite commands — such that a reviewer following them produces the same operation set and results within the run-to-run variance the suite itself exhibits. The pinned versions and conditions are recorded alongside the results (§5).
- **Verification:** `benchmark_run` + `doc_inspection` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`; A1, A4.

### REQ-PERF-006 — A stated comparison method that avoids cherry-picking
- **Statement:** The repository states the comparison method — same-conditions runs, keyed category, like-with-like reading, full per-operation reporting, and the measured-vs-expected separation — and the published comparison follows it.
- **Rationale:** A comparison with no stated method can be shaped to flatter; stating the method and holding the results to it is what makes the comparison checkable by someone who was not in the room.
- **Acceptance criteria:** A reviewer can confirm the repository states the §3 method (named peers, same commit/Chromium/machine, keyed-only, full-table reporting, the suite's weighted geometric-mean as the only headline figure, conditions published with results, measured kept separate from expected); and that the published comparison reports every operation, not a subset.
- **Verification:** `doc_inspection` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`; A1.

### REQ-PERF-007 — Compared against the named Rust/WASM peers
- **Statement:** Ferric's results are presented against Leptos, Sycamore, Dioxus, and Yew in the keyed category, under the conditions of §3.2.
- **Rationale:** These four are the frameworks an evaluating author actually chooses between, and they are the like-with-like comparison (Rust/WASM, keyed). Comparing against them is the comparison that answers the adoption question.
- **Acceptance criteria:** The published comparison includes Leptos, Sycamore, Dioxus, and Yew by name, in the keyed category; where a peer number is cited rather than measured in the same session, its source and date are given (§3.2).
- **Verification:** `benchmark_run` + `doc_inspection` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`; A1.

### REQ-PERF-008 — Positioned against the JavaScript field, with the WASM-vs-JS caveat
- **Statement:** Ferric's results are placed against the suite's vanillajs baseline and the JS reference frameworks (Solid, Svelte, Vue, React) as context, with the WASM-vs-JS memory and bundle caveats stated.
- **Rationale:** An author's real alternative is often a JS framework, so the wider field is useful context; but memory and bundle size do not compare one-to-one across the WASM/JS boundary, and presenting them as if they did would mislead. The caveat is what stops the context from misleading.
- **Acceptance criteria:** The published comparison shows Ferric against the vanillajs baseline and the named JS frameworks as context; the WASM-vs-JS caveat (§3.3) is attached to the memory and bundle comparisons; the JS field is not presented as the primary like-with-like bar.
- **Verification:** `benchmark_run` + `doc_inspection` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`; A1.

### REQ-PERF-009 — Results published in-repo, measured kept separate from targets
- **Statement:** The measured results are published in the repository with their run conditions, using the suite's weighted geometric-mean as the summary figure, and are kept distinct from the pre-measurement targets.
- **Rationale:** Results that live only on a maintainer's machine cannot be trusted or rerun; and a target shown as a result is the exact failure the measured-vs-expected rule exists to prevent. Publishing with conditions, and labelling targets as targets, is what lets a reader rely on the numbers.
- **Acceptance criteria:** The repository contains a results artifact carrying the per-operation table, the memory and size figures, the suite's weighted geometric-mean score, and the run conditions (§5); targets appear only in a separately-labelled targets section and never in the results table; no measured figure is adjusted toward a target.
- **Verification:** `doc_inspection` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-012`; A1, A4.

### REQ-PERF-010 — Bundle size reflects, and guards, the lean footprint
- **Statement:** The measured bundle size reflects Ferric's actual dependency set, and the performance story is the check that keeps the footprint lean (`REQ-FW-013`): a dependency added during the 1.0 work shows up as measured bytes.
- **Rationale:** Leanness is one of Ferric's stated differentiators, and in WASM every dependency is downloaded bytes; without a measured size that tracks the real dependency set, the footprint can erode unnoticed. The benchmark's size metric is the objective check on `REQ-FW-013`.
- **Acceptance criteria:** The dependency set that produced the measured bundle is documented alongside the size figure; a change to the core dependency set is reflected in a re-measured bundle size; the size figure in §2.3 is the one used to back `REQ-FW-013`'s claim.
- **Verification:** `benchmark_run` + `doc_inspection` · **Stage:** one_oh · **Release gating:** true
- **Traceability:** `REQ-FW-013`, `REQ-FW-012`; A1, A4. Per-crate dependency choices → each owning area.

### Requirement → cross-cutting coverage

| `REQ-FW-*` | Covered by |
|---|---|
| REQ-FW-012 (reproducible, named-peer performance) | REQ-PERF-001…009 |
| REQ-FW-013 (lean, documented footprint) | REQ-PERF-004, REQ-PERF-010 |

---

## 5. Reproducibility — the pinned conditions

*(section: `reproducibility`)*

Reproducibility is a first-class requirement (`REQ-PERF-005`), not a footnote. For another person to reach
comparable numbers, the following MUST be recorded with every published result set, and the rerun steps MUST
reference them:

| Pinned item | Why it must be fixed |
|---|---|
| js-framework-benchmark commit | The suite's operations and scoring change over time; a result is only comparable to peers measured at the same commit. |
| Rust toolchain version (stable channel) | Codegen and optimization change the numbers; `REQ-FW-003` already fixes stable Rust as the target. |
| `wasm-bindgen` + `wasm-opt` versions | They determine the shipped bundle and part of the runtime cost. |
| Chromium version | The harness reads the browser's timeline; a different engine version shifts durations and memory. |
| Machine class | Absolute durations are machine-dependent; the suite's slowdown-vs-vanillajs ratio mitigates this, but the machine is still recorded. |
| Run session | Ferric and the peers it is compared against are run in one session so conditions are identical. |

The rerun procedure a stranger follows: check out the pinned benchmark commit, build Ferric's entry with the
pinned toolchain through the suite's standard build, run the suite's keyed benchmark for Ferric and the named
peers, and read the produced results. The procedure lives in the repository (its exact location is the
performance work-item's to place; the docs area, `REQ-DX-*`, links to it). Run-to-run variance is expected; the
reproducibility bar is "comparable within the suite's own variance," not "bit-identical."

---

## 6. The lean-footprint link

*(section: `lean_footprint_link`)*

`REQ-FW-013` commits Ferric to a lean dependency footprint; the benchmark's size metric (§2.3) is the objective
check on that commitment. The baseline core depends only on `wasm-bindgen`, `web-sys`, and `js-sys`; the HTTP
client's `serde` / `wasm-bindgen-futures` are the expected narrowly-scoped additions. `REQ-PERF-010` ties the
measured bundle to the documented dependency set so that any growth is visible as bytes rather than discovered
after release. This is the one place the performance work and the dependency-discipline work meet: the number
produced here is the evidence `REQ-FW-013` is held to.

---

## 7. Targets — stated as targets, not results

*(section: `targets`)*

These are **pre-measurement expectations**, recorded so the 1.0 work has something to aim at and so a reader can
later see how the measured numbers compared to what was expected. **None of these is a measured result.** They
MUST NOT appear in the results artifact (§8); they live here, labelled, and are replaced by nothing — the
measured results sit in their own place.

- **Speed (expectation, not measured):** Ferric's keyed-VDOM-plus-fine-grained-reactivity design is expected to
  land in the competitive band of Rust/WASM frameworks on the create/update/swap/remove operations — in the
  neighbourhood of the VDOM-based Rust peers (Dioxus, Yew), with the signal-only peers (Leptos, Sycamore)
  expected to lead on the operations where avoiding a VDOM helps. This is a hypothesis the benchmark tests, not a
  claim; the measured table may refute it, and if it does, the measured table stands.
- **Memory (expectation, not measured):** expected to be within the Rust/WASM peer band; the repeated-clear and
  update metrics are the ones that would reveal a reactivity-graph leak, and the target is "no growth across the
  repeated-clear cycles."
- **Bundle size (expectation, not measured):** the lean dependency set is expected to keep the optimized transfer
  size competitive with the leaner Rust/WASM peers; the target is that the measured size stays defensible against
  named peers and that it tracks the documented dependency set (§6).

Each target is written so that the measured result can be laid beside it and the difference read plainly. The
value of stating them is precisely that they can be wrong; they are expectations the measurement adjudicates.

---

## 8. Measured-vs-expected reporting discipline

*(section: `measured_vs_expected`)*

When the entry has been run, results are reported under this discipline — the operational form of the
no-cherry-picking rule (§3.4) and `REQ-PERF-009`:

1. **One results artifact, in the repository.** The measured numbers live in a published results artifact (the
   performance work-item places it; the docs area links it), carrying the full per-operation speed table (§2.1),
   the memory metrics (§2.2), the size figures (§2.3), the suite's weighted geometric-mean summary, and the §5
   run conditions.
2. **Targets stay in §7; results stay in the artifact.** The two are never interleaved. A target is not shown in
   the results table; a measured number is not moved into §7.
3. **The whole table, every time.** Every operation is reported, including those where Ferric is slower. A
   summary figure, if given, is the suite's weighted geometric-mean, computed the suite's way.
4. **Conditions travel with numbers.** No result is published without its run conditions, so it can be judged and
   rerun.
5. **Cited peer numbers are marked as cited.** A peer figure taken from the project's published table rather than
   measured in Ferric's session carries its source and date.

The reporting shape the artifact will take (shown empty — it holds no measured data yet):

| Operation | Ferric | Leptos | Sycamore | Dioxus | Yew | vanillajs |
|---|---|---|---|---|---|---|
| Create 1,000 | — | — | — | — | — | — |
| Create 10,000 | — | — | — | — | — | — |
| Append 1,000 | — | — | — | — | — | — |
| Update every 10th | — | — | — | — | — | — |
| Select row | — | — | — | — | — | — |
| Swap rows | — | — | — | — | — | — |
| Remove row | — | — | — | — | — | — |
| Clear rows | — | — | — | — | — | — |
| Memory (ready / run / update / replace / repeated-clear) | — | — | — | — | — | — |
| Transfer size (compressed) | — | — | — | — | — | — |
| Weighted geo-mean (vs vanillajs) | — | — | — | — | — | 1.00 |

*(Dashes mark unmeasured cells. The vanillajs baseline geo-mean is 1.00 by definition of the suite's scoring; it
is the one non-dash entry because it is a definition, not a measurement of Ferric.)*

---

## Metadata

- **Owning task:** `SUBTASK_performance_and_benchmark` (owns the `REQ-PERF-*` register; identity I1).
- **Repository target:** `design/ferric/performance-and-benchmark.md` (placed by `SUBTASK_design_pr`).
- **Conforms to:** `../contracts/id_and_traceability_schema.yaml` (REQ fields, `verification_method` vocabulary
  incl. `benchmark_run`, stage vocabulary, `release_gating`) and `../contracts/_MANIFEST.yaml` (ownership,
  identity I1).
- **Grounding commit:** `Nousix-LLC/nswe-demo` @ `276138f` (`main`).
- **Benchmark referenced:** the js-framework-benchmark project (`krausest/js-framework-benchmark`); Rust/WASM
  peer entries: Leptos, Sycamore, Dioxus, Yew.
- **Register summary:** `REQ-PERF-001`–`REQ-PERF-010`, all `stage: one_oh`, `release_gating: true`; verification
  `benchmark_run` (001–005, 007, 008, 010) or `doc_inspection` (006, 009), backing `REQ-FW-012` and `REQ-FW-013`.
- **Measured results:** none in this document. Every number here is a target or a plan (§7, §8); measured results
  are produced only when the entry is run, and are kept separate from the targets.
- **Cross-references** to `REQ-FW-*`, `REQ-API-*`, `REQ-DX-*` are read-only; those registers are owned by their
  sibling spokes.
</content>
</invoke>
