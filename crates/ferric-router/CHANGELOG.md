# Changelog — ferric-router

All notable changes to `ferric-router` are documented here.
Versioning follows the workspace `version.workspace = true` setting.

---

## Unreleased (r1 additions)

The following items are **additive**: the frozen public surface from the initial
implementation is preserved unchanged. No existing signatures were removed or
modified.

### Added

#### `navigate_replace(path: &str)` — replace navigation (REQ-RT-003)

Programmatically navigate to `path` by **replacing** the current History entry
(`history.replaceState`) instead of pushing a new one. Use for redirects, login
flows, or wizard steps where the intermediate URL should not be reachable via the
browser back button.

- On wasm32: calls `history.replaceState(null, "", path)` — no history growth.
- On all targets: updates the reactive `current_path()` signal, triggering outlet
  re-renders.
- Native tests verify `current_path()` updates after `navigate_replace`.
- wasm32 tests (wasm-bindgen-test) verify history length does not grow on replace.
- Exported from `ferric_router::prelude` as `pub use crate::router::navigate_replace`.

#### Active-route awareness on `Link`/`link` via `aria-current="page"` (REQ-RT-007)

`Link::build()` (and the `link()` shorthand which delegates to it) now calls
`current_path()` at build time and conditionally adds `aria-current="page"` to the
emitted element when the link's destination equals the current path.

- **Active case** (`self.to == current_path()`): the element carries
  `attr("aria-current", "page")`.
- **Inactive case** (`self.to != current_path()`): the element is structurally
  identical to the previous frozen surface — no `aria-current` attribute, no class
  change.
- When built inside a `ferric` render effect (the normal nav-bar usage), the render
  subscribes to path changes automatically via the `current_path()` reactive read.
- The frozen `link(to, label) -> VNode` signature is unchanged.

#### `RoutePattern::match_prefix` — prefix matching (REQ-RT-005 prerequisite)

New method on `RoutePattern` that matches the pattern's segments against the
**beginning** of `path` rather than the whole path:

```rust
pub fn match_prefix(&self, path: &str) -> Option<(Params, String)>
```

Returns `Some((params, remaining))` where `params` holds any `:name` captures and
`remaining` is the path suffix after the matched prefix segments, normalized as a
`/`-led string. Returns `None` if any pattern segment fails to match.

Examples:
- `/settings` against `/settings/profile` → `Some((Params::default(), "/profile"))`
- `/settings` against `/settings` → `Some((Params::default(), "/"))`
- `/user/:id` against `/user/42/edit` → `Some((params{id="42"}, "/edit"))`
- `/user/:id` against `/user` → `None` (too few segments)

#### `Router::nest` — nested routes with per-level layouts (REQ-RT-005)

New builder method on `Router`:

```rust
pub fn nest(
    self,
    prefix: &str,
    layout: impl Fn(&Params, &dyn Component) -> VNode + 'static,
    child_builder: impl FnOnce(Router) -> Router,
) -> Self
```

When the current path begins with `prefix`, the `layout` closure is called with
the matched prefix params and a `&dyn Component` for the child outlet, which renders
the matched child route's view (using the remaining path after the prefix).

`child_builder` receives a fresh `Router::new()` and returns the configured child
router. Child routes are matched against the remaining path after the prefix.

**Resolution order**: `Router::outlet()` tries nested routes (prefix matching) before
flat routes (exact matching), in declaration order within each group.

**Per-level fallback**: the child router's `.fallback()` renders for unmatched child
paths; the app-root router's `.fallback()` renders for paths that don't match the
prefix at all.

The existing flat `Router::route` / `.outlet()` behaviour is the degenerate
single-level case and is unchanged.
