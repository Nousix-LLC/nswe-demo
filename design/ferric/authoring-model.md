# Ferric 1.0 — Authoring model

This document specifies the declarative authoring model for Ferric 1.0: what an app author writes to build
real UI. It owns the `REQ-API-*` register and specializes parent `REQ-FW-005` (declarative authoring),
`REQ-FW-009` (additive over frozen surfaces), and `REQ-FW-014` (actionable diagnostics). It states what
the author writes and what must hold — not how the macros expand or how the runtime is built.

The model has three pieces that compose into one authoring story:

1. **The `view!` macro** — angle-bracket declarative view trees that lower onto the existing `VNode` model.
2. **The component model** — `#[component]` functions with typed props, `children`, and named slots.
3. **Context** — `provide_context`/`use_context`/`expect_context` for shared values without prop drilling.

Each piece is designed separately; together they satisfy the complete authoring surface. The invocation seam
(IA2) connects the macro to the component model. The context seam (IA3) connects context to the reactive
owner scope and to the unit the macro renders.

**Grounding.** All claims about today's Ferric are read from the baseline at `taskflow/dags/ferric-foundation/
checkout/crates/ferric/` (repository `Nousix-LLC/nswe-demo` @ `276138f`): `src/vdom.rs`, `src/component.rs`,
`src/signal.rs`, `src/dom.rs`, `src/lib.rs`, `examples/counter.rs`. Peer comparisons name each framework and
rest on its documented public behavior.

---

## 1. The baseline this model replaces

Today an author composes a view tree by hand. The counter example shows the full baseline authoring surface:

```rust
// examples/counter.rs — the baseline
let count = create_signal(0i32);
let app = move || -> VNode {
    VElement::new("div")
        .attr("class", "counter")
        .child(
            VElement::new("button")
                .on(EventKind::Click, move || count.update(|n| *n += 1))
                .child(text("increment")),
        )
        .child(VElement::new("p").child(text(format!("count: {}", count.get()))))
        .into()
};
mount(root, app);
```

Four facts about this baseline constrain everything the authoring model may and may not do:

1. **The node model is closed and frozen.** `VNode` is exactly `Element(VElement) | Text(String)`. A
   `VElement` carries `tag`, `key`, `attrs`, `events`, and `children`. The builders `VElement::new`,
   `.key`, `.attr`, `.on`, `.child`, `.children`, `vdom::text`, and `From<VElement> for VNode` are the
   only construction paths. `EventKind` is `Click | Input` today. `EventHandler` is `Rc<dyn Fn()>` — a
   zero-argument closure. The authoring model MUST produce nodes through these constructors and MUST NOT
   redefine any of them (`REQ-FW-009`, identity IA4).

2. **Rendering is coarse-grained: whole-component re-render plus keyed diff.** `mount` wraps the entire
   component render in one `create_effect`. Every `Signal`/`Memo` read anywhere in the render subscribes
   that one effect; any later write re-runs the whole render, and the keyed `diff` patches the live DOM.
   Ferric has no fine-grained per-node bindings — this is the Vue 3 / Dioxus / Yew virtual-DOM model, not
   the Leptos/Sycamore fine-grained model.

3. **Lists reconcile by key only when every sibling is a keyed element.** `diff_children` uses the keyed
   path only when `all_keyed(old) && all_keyed(new)`. A single unkeyed sibling drops that parent's child
   list to positional diffing. The authoring model is designed to make a fully-keyed list natural to write.

4. **Handlers re-bind only when the set of bound `EventKind`s changes, and read signals live.** Closures
   are opaque, so the diff cannot compare them; it re-emits `SetEvents` only when the set of event kinds
   changes. A retained handler reads its signals at call time; keeping it across a re-render is sound.

The authoring model adds no runtime capability the builders lack. It is a more ergonomic way to write the
same `VNode`.

---

## 2. The `view!` macro

The counter, written with the authoring model:

```rust
let count = create_signal(0i32);
let app = move || view! {
    <div class="counter">
        <button on:click={move || count.update(|n| *n += 1)}>"increment"</button>
        <p>"count: " {count.get()}</p>
    </div>
};
mount(root, app);
```

The `view!` and the hand-built tree produce byte-identical `VNode`s.

### 2.1 Syntax flavor

Ferric uses an HTML/JSX-style `view!` with angle-bracket tags and brace-delimited Rust interpolation — the
family of Leptos's `view!` and Yew's `html!`. The choice is a trade-off (§6 compares the alternatives by
name): angle brackets read like the HTML an app author already knows, which matters for a VDOM framework
whose mental model is "a tree that re-renders." The flavor is independent of lowering: a different syntax
would produce the same builder calls.

### 2.2 Elements, text, and attributes

- **Elements.** `<tag>…</tag>` and self-closing `<tag/>`. Lowercase is an HTML element; a capitalized tag
  is a component invocation (§2.7).
- **Text.** A string literal in child position is a text node: `<p>"hello"</p>`. Text is quoted, not bare,
  so the macro never guesses where markup ends and content begins — this eliminates a class of whitespace
  ambiguity (§2.10) and is the known cost of the HTML-like flavor.
- **Attributes.** `name="literal"` for a static value and `name={expr}` for a dynamic one. Because
  attributes are `(String, String)` pairs, a dynamic value must be `impl Into<String>`.

```rust
view! { <a href="/x" class={active_class}>"link"</a> }
```

A non-string value is converted explicitly — `width={n.to_string()}` — so the macro has a precise place
to emit a diagnostic when the type is wrong (§2.9).

### 2.3 Event handlers

A handler is written `on:<name>={closure}` and lowers to `.on(EventKind::<Variant>, closure)`:

```rust
view! { <button on:click={move || count.update(|n| *n += 1)}>"+"</button> }
```

Two baseline-driven facts shape handlers:

- **Handlers are zero-argument.** The frozen `EventHandler` is `Rc<dyn Fn()>`, so the handler closure
  takes no arguments. An event-payload form (receiving the `web_sys::Event`) is an additive refinement
  owned by `REQ-ASYNC-*`; this section names the seam without inventing a signature the frozen
  `EventHandler` does not have.
- **The event-name set is open.** `on:click` and `on:input` resolve today. As `REQ-ASYNC-*` extends
  `EventKind`, those names resolve with no change to the macro grammar. A name with no matching variant is
  a compile-time diagnostic (§2.9), not a silently dropped handler.

### 2.4 Interpolated reactive values

A `{expr}` in child position is a Rust expression evaluated during the component's render:

```rust
view! { <p>"count: " {count.get()}</p> }
```

Because `mount` wraps the whole render in one effect (baseline fact 2), reading `count` inside `{…}`
subscribes that effect. A later `count.set(..)` re-runs the whole component and the keyed diff patches
what changed.

**Ferric's reactivity is coarse-grained.** A signal read inside a view re-renders the *entire component*,
which the keyed diff reconciles against the live DOM — it does not create a fine-grained binding that
updates only that one text node. This is the VDOM model Ferric shares with Vue 3, Dioxus, and Yew, and
it follows from the frozen `mount`/`diff` foundation, not from a gap the macro could paper over. An author
arriving from Leptos or Solid will expect fine-grained per-node updates and must be told, up front, that
Ferric does not provide them.

The ergonomic payoff: `{expr}` is a plain Rust expression. A signal read inside it is reactive without
wrapping it in a closure, because the read already happens inside the render effect.

### 2.5 Control flow

Conditional rendering is a Rust `if`/`match`/ternary in interpolation position that yields a `VNode`:

```rust
view! {
    <div>
        { if signed_in.get() {
              view! { <span class="who">{user.get()}</span> }
          } else {
              view! { <a href="/login">"sign in"</a> }
          } }
    </div>
}
```

Notes from the baseline:

- **An "empty" branch still yields a node.** `VNode` has no null variant, so the "render nothing" branch
  produces an explicit empty text node (`""`). The macro provides an empty-node form for this branch so the
  author does not reach for an ad-hoc `text("")`.
- **A conditional that changes node kind or tag reconciles as a `Replace`.** When two branches differ in
  kind (element↔text) or tag (`<span>` vs `<a>`), the diff emits a wholesale `Replace` at that position.
  The guide states this so an author understands that toggling between two different elements rebuilds the
  subtree.

### 2.6 Lists and keys

Keyed reconciliation has a hard precondition (baseline fact 3): the keyed path engages only when every
sibling in the list is a keyed element on both sides. The list syntax makes that the default:

```rust
view! {
    <ul>
        <For each={todos.get()} key={|t| t.id.to_string()} let:todo>
            <li>{todo.label.clone()}</li>
        </For>
    </ul>
}
```

This lowers to a `Vec<VNode>` in which every element carries its key, spliced via `.children(..)`. Because
the key is part of the form, the produced sibling list satisfies `all_keyed`, so inserts, moves, and
removes reconcile by identity (the baseline `keyed_children_*` behavior).

Key expressions yield the baseline `Key` (`String`); `key={|t| t.id.to_string()}` is the standard form.

Two pieces of guidance the list facet carries:

- **Do not mix a keyed list with unkeyed static siblings.** One unkeyed `<li>` beside the `<For>` output
  drops the whole `<ul>` to positional diffing, silently losing identity reconciliation. Let the list be
  the sole content of its container, or key the static siblings too.
- **A standalone `key={expr}` on any element** lowers to `.key(expr)` and is available for hand-rolled
  lists.

### 2.7 Invoking a component (the IA2 macro side)

A capitalized tag is a component invocation; a lowercase tag is an HTML element. This convention is shared
by every peer (Leptos, Yew, Dioxus, Sycamore), and it lets the macro route by tag token alone.

```rust
view! {
    <TodoList items={todos.get()}>          // props
        <TodoHeader/>                        // child content
        <slot:footer><Pager/></slot:footer>  // named slot fill
    </TodoList>
}
```

The macro has exactly three passing forms, each matching a declaration form the component model defines
(§3, identity IA2):

| Macro-side form | Component-side declaration |
|---|---|
| `name={value}` on a component tag | A declared prop (name, type, required/optional/default) |
| Children nested inside a component element | The component's `children: Children` |
| `<slot:name>…</slot:name>` | A named slot the component declares as `name: Slot` |

This section states only the macro side. The component model (§3) owns the declaration side. §5 closes
the seam in both directions.

### 2.8 Context, referenced from a component body (IA3)

A context value is read inside a component body — the unit the macro renders. The macro places no obstacle:
a context read is an ordinary Rust expression usable in a `{…}` interpolation or in a handler closure,
exactly like a signal read. The macro adds no context syntax. Context is owned by §4; IA3 (context
lifetime tied to the reactive owner scope) is asserted there.

### 2.9 Diagnostics (the `trybuild`-checkable bar)

The bar is that a malformed view produces an error that names the author's mistake in terms of their code,
at the span of the offending tokens — not a wall of internal macro errors. The bar is checkable with
`trybuild`: a fixture of malformed views, each with a committed expected `stderr` (REQ-API-028).

| Author mistake | Expected diagnostic |
|---|---|
| Unclosed element — `<div>` with no `</div>` | "unclosed element `<div>`" at the open tag |
| Mismatched close — `<div>…</span>` | "closing tag `</span>` does not match open tag `<div>`" |
| Multiple top-level nodes | "a `view!` must have a single root element; wrap these in one element" |
| Dynamic attr/text value not `Into<String>` | Type error at the `{expr}` span, naming the required conversion |
| Unknown event name — `on:klick=` | "unknown event `klick`; bound events are: click, input" |
| Handler value is not a closure | "`on:click` expects a handler closure" at the value span |
| `<For>` item without a key | "list item needs `key={…}` for keyed reconciliation" (best-effort) |
| Capitalized tag not in scope | The ordinary "cannot find `TodoList` in this scope" at the tag span |

Two limits the bar states plainly:

- **Keyless-list detection is best-effort.** The macro can flag a missing `key` on the direct `<For>` form.
  It cannot catch every keyless list built via `{iter.map(..)}`, because those elements are produced by
  opaque Rust the macro does not see into. The `<For>` form makes keying the default; the arbitrary-
  iterator case carries a documented convention, not a guarantee.
- **No handler-identity diagnostics.** The runtime cannot compare closures (baseline fact 4), so the macro
  promises nothing about handler equality.

### 2.10 Fragments and single-root rule

`VNode` is a single node and the enum is frozen, so no `Fragment` variant can be added without redefining
it — which IA4 forbids. A top-level `view!` yields exactly one root `VNode`; multiple top-level nodes are
a compile diagnostic (§2.9). In child position, multiple nodes — `{vec_of_vnodes}`, a `<For>`, or
sibling tags — splice into the parent's child list via `.children(..)`. Fragmentation is a child-list
concern, which the `Vec<VNode>→.children(..)` lowering already expresses.

### 2.11 Interop and whitespace

- **Interop is two-way.** `view!` produces an ordinary `VNode`, usable anywhere the builders are; and a
  builder-produced `VNode` is spliceable into a view via `{expr}`. An author can adopt the macro
  incrementally and drop to builders for a case the macro does not cover.
- **Escape hatch.** Any `{expr}` evaluating to a `VNode` splices directly as a child.
- **Whitespace.** Only quoted text is rendered. Whitespace and newlines between tags are insignificant and
  discarded — no injected or collapsed spaces from indentation.

### 2.12 Lowering table (additive; nothing redefined)

Every row uses a frozen constructor. No frozen item is redefined (identity IA4, parent `REQ-FW-009`).

| Author writes | Lowers to |
|---|---|
| `<div>…</div>` / `<br/>` | `VElement::new("div").children([…])` / `VElement::new("br")` |
| `"text"` | `vdom::text("text")` |
| `{expr}` in child position (`expr: Into<String>`) | `vdom::text(expr)` |
| `{expr}` in child position (`expr: VNode`) | the `VNode` itself, spliced as a child |
| `{vec}` / `<For …>` in child position | the `Vec<VNode>`, via `.children(vec)` |
| `name="v"` / `name={e}` | `.attr("name", v)` / `.attr("name", e)` (`e: Into<String>`) |
| `key={e}` | `.key(e)` (`e: Into<Key>`, `Key = String`) |
| `on:click={h}` / `on:input={h}` | `.on(EventKind::Click, h)` / `.on(EventKind::Input, h)` |
| `on:<name>={h}` for a future `EventKind` | `.on(EventKind::<Variant>, h)` once `REQ-ASYNC-*` adds it |
| `<Capitalized … />` | component model invocation (§3), yielding a `VNode` spliced as a child |
| the element at the end of `view!` | `.into()` to a `VNode` |

The `view!` macro and the `vdom::text` helper are new, additive items. `text` is in the `vdom` module
but not in today's frozen `prelude`; adding `view!` and re-exporting `text` into the prelude are additive
changes — no existing export is removed or redefined.

---

## 3. The component model — props, children, slots, and composition

Today a component is any `Fn() -> VNode` with no declared surface: an author captures inputs by hand,
builds the tree with `.child(..)` calls, and has no type-checked way for a caller to fill props, pass
children, or fill named regions. A reusable `Card` is possible but fragile — its API is the captured
environment, and callers must know it without a contract.

The component model adds a declared, typed surface, additive over the frozen `Component` trait
(`component.rs` baseline: one method `fn render(&self) -> VNode`, with a blanket impl over `Fn() -> VNode`).

### 3.1 `#[component]` function with a generated typed props builder

An author declares a component as an ordinary Rust function annotated with `#[component]`. Its parameters
are its props; its body returns a `VNode`:

```rust
use ferric::prelude::*;

#[component]
fn Greeting(name: String, #[prop(default = 3)] times: usize) -> VNode {
    view! { <p>{format!("Hello {name} ×{times}")}</p> }
}
```

What the author writes to use it:

```rust
view! { <Greeting name={"ferric".into()} times=5 /> }   // both props set
view! { <Greeting name={"ferric".into()} /> }           // times takes its default, 3
```

`#[component] fn Greeting(..) -> VNode` is sugar over the frozen `Component` trait. It generates three
things the author never writes by hand:

1. A props struct `GreetingProps` with one field per parameter.
2. A builder with one setter per field, honoring each field's kind (§3.2).
3. A `Component` implementation whose `render` runs the function body with the props.

This is the design every credible Rust peer converged on. The alternatives:

| Declaration style | Why not chosen as the primary surface |
|---|---|
| Positional function arguments, no struct | Rust has no optional or defaulted function arguments and no named call arguments. Every prop would be required and positional; adding or reordering breaks every caller. |
| Hand-written builder | Works and is explicit, but the author writes the builder plumbing for every component. The cost the framework should absorb is paid at every declaration. |
| **`#[component]` + generated typed props builder** (chosen) | Compile-time required/optional/defaulted props, named arguments, additive evolution (a new optional prop does not break existing callers), and the author writes a plain function. |

**Divergence from Yew/Dioxus:** Ferric does not require props to be `PartialEq`. Change detection is
signal-based (`Signal<T>`/`Memo<T>` passed as props re-render the parts that read them when they change),
not prop-diffing. Yew and Dioxus require `PartialEq` to skip re-renders; Ferric does not because it does
not use that mechanism. Leptos and Sycamore take the same position for the same reason.

### 3.2 Typed props

A prop is a parameter of the `#[component]` function. Its kind is read from its type and `#[prop(..)]`:

- **Required** — a plain typed parameter, no attribute. Omission is a compile error naming the missing
  prop (§3.7).

  ```rust
  #[component]
  fn Avatar(src: String, alt: String) -> VNode { /* both required */ }
  ```

- **Optional** — `#[prop(optional)]`, making the prop an `Option<T>` that defaults to `None`:

  ```rust
  #[component]
  fn Avatar(src: String, #[prop(optional)] alt: Option<String>) -> VNode { }
  ```

- **Defaulted** — `#[prop(default = expr)]`, supplying `expr` when the caller omits the prop:

  ```rust
  #[component]
  fn Avatar(src: String, #[prop(default = 48)] size_px: u32) -> VNode { }
  ```

- **Converting** — `#[prop(into)]`, accepting any `T: Into<FieldType>` so a caller can pass a `&str`
  where the prop is `String`:

  ```rust
  #[component]
  fn Avatar(#[prop(into)] src: String) -> VNode { /* called as src="/a.png" */ }
  ```

- **Reactive** — a prop whose type is `Signal<T>` or `Memo<T>`. These are `Copy` handles into the
  reactive runtime, so passing one to a component is cheap; reading it in the body re-renders on change:

  ```rust
  #[component]
  fn Counter(count: Signal<i32>) -> VNode {
      view! { <p>{move || count.get()}</p> }
  }
  ```

A component with no props is the baseline function component unchanged. `#[component]` on a zero-parameter
function generates an empty props type and leaves the bare `Fn() -> VNode` blanket impl undisturbed
(`REQ-FW-009`).

### 3.3 Children

A component accepts child content by declaring a parameter named `children` of type `Children`:

```rust
use ferric::prelude::*;

#[component]
fn Card(title: String, children: Children) -> VNode {
    view! {
        <section class="card">
            <h2>{title}</h2>
            <div class="body">{children()}</div>
        </section>
    }
}
```

Caller fills it by nesting content inside the component element:

```rust
view! {
    <Card title={"Welcome".into()}>
        <p>"First paragraph."</p>
        <p>"Second paragraph."</p>
    </Card>
}
```

`Children` is a new additive type — a deferred, reference-counted thunk:

```rust
pub type Children = std::rc::Rc<dyn Fn() -> Vec<VNode>>;
```

It returns `Vec<VNode>` rather than `VNode` because a children block may hold several sibling nodes, and
the frozen `VNode` has no fragment variant (`Vec<VNode>` carries multiple roots without touching `VNode`).
The component splices with the existing `.children(..)` builder: `{children()}` lowers to
`.children(children())`.

Children are required by default. A component accepts optional children with
`#[prop(optional)] children: Option<Children>` and renders them only when present. A component with no
`children` parameter does not accept children; nesting content inside it in `view!` is a diagnostic that
names the component (§3.7).

### 3.4 Named slots

Children are one unlabelled region. A component that needs more than one labelled region declares each as
a `Slot` field:

```rust
use ferric::prelude::*;

#[component]
fn Page(
    header: Slot,
    #[prop(optional)] footer: Option<Slot>,
    children: Children,
) -> VNode {
    view! {
        <article>
            <header>{header()}</header>
            <main>{children()}</main>
            { footer.map(|f| view! { <footer>{f()}</footer> }) }
        </article>
    }
}
```

Caller fills it:

```rust
view! {
    <Page>
        <slot:header><h1>"Title"</h1></slot:header>
        <p>"Main body."</p>
        <slot:footer>"© 2026"</slot:footer>
    </Page>
}
```

`Slot` has the same shape as `Children` — a deferred `Rc<dyn Fn() -> Vec<VNode>>` — but is a distinct,
named field the author fills by label. Rules:

- **One region per slot.** Each slot is filled at most once. Filling an undeclared slot or filling one
  twice is a diagnostic naming the slot and component (§3.7).
- **Required vs optional.** `Slot` is required; `Option<Slot>` is optional and absent when omitted.
- **Slots and children coexist.** A component may declare named slots and a `children` field; unlabelled
  content fills `children`, labelled content fills the matching slot.

**Parameterized slots** let a component pass a value into the caller-provided content — the pattern a
list rendering a per-item template needs. A parameterized slot is declared as `SlotFn<T>`:

```rust
#[component]
fn List<T: Clone + 'static>(items: Vec<T>, row: SlotFn<T>) -> VNode {
    view! {
        <ul>
            { items.into_iter()
                   .map(|it| view! { <li>{row(it)}</li> })
                   .collect::<Vec<_>>() }
        </ul>
    }
}
```

where `SlotFn<T> = Rc<dyn Fn(T) -> Vec<VNode>>`. Basic named slots are in `usable_early`; parameterized
slots are `one_oh`, since a real app can be built with plain named slots in the interim.

### 3.5 Composition

Composition is nesting: one component is placed in another's `view!` body, filled through its declared
surface. Because every component renders to one `VNode` and `VNode`s nest through `VElement.children`,
composition needs no rule beyond the lowering in §3.6.

```rust
#[component]
fn Dashboard(user: Signal<User>) -> VNode {
    view! {
        <Page>
            <slot:header><Greeting name={user.get().name} /></slot:header>
            <Card title={"Activity".into()}>
                <List items={user.get().events} row={Rc::new(|e| view! { <span>{e.label}</span> })} />
            </Card>
        </Page>
    }
}
```

`Dashboard` fills `Page`'s header slot with a `Greeting` and its body with a `Card` that holds a `List`.
Each inner component uses the same declared surface a leaf caller uses.

**The escape hatch.** `#[component]` is sugar; any component can be written by hand against the frozen
trait, and hand-written and generated components compose freely:

```rust
struct Clock { tz: String }
impl Component for Clock {
    fn render(&self) -> VNode { view! { <time>{now_in(&self.tz)}</time> } }
}
let spacer = || -> VNode { VElement::new("hr").into() };
```

All three forms — `#[component]`, `impl Component`, and bare `Fn() -> VNode` — are the same frozen trait.
The baseline's object-safe dynamic dispatch (`&dyn Component`, `Box<dyn Component>`) is carried forward:
a component may hold `Box<dyn Component>` children where the concrete set is dynamic.

### 3.6 Additive types and lowering

The model introduces exactly these items, all additive — nothing redefined (identity IA4):

| New item | Shape | Role |
|---|---|---|
| `#[component]` | attribute macro on `fn(..) -> VNode` | Generates the props type, builder, and `Component` impl. |
| `<Name>Props` + its builder | generated struct + builder | The typed declaration surface the `view!` macro fills. |
| `#[prop(optional)]` / `#[prop(default = ..)]` / `#[prop(into)]` | field attributes | Mark a prop optional, defaulted, or converting. |
| `Children` | `Rc<dyn Fn() -> Vec<VNode>>` | Deferred, reactive child content. |
| `Slot` | `Rc<dyn Fn() -> Vec<VNode>>` | One named, deferred, reactive content region. |
| `SlotFn<T>` | `Rc<dyn Fn(T) -> Vec<VNode>>` | A parameterized slot (render-prop pattern). |

Lowering: a component invocation `<Name .. />` builds `NameProps` through the builder, constructs the
component value, and calls `Component::render(&value)` — the frozen trait method — to get one `VNode`,
appended via `.child(..)`. Props become struct fields. Children and slots are captured as thunks and
spliced where the component calls them with the existing `.children(..)` / `.child(..)` builders.

### 3.7 Diagnostics (the testable failure bar — IA6)

The model commits to a `trybuild`-checkable bar for component mistakes. Each case is a compile-fail
fixture with a message naming the author's error in terms of their code:

| Author mistake | Required diagnostic |
|---|---|
| Omit a required prop | "missing required prop `name` on `Greeting`" — not a raw builder-state type error. |
| Pass a prop of the wrong type | "prop `size_px` on `Avatar` expects `u32`" |
| Pass a prop the component does not declare | "`Avatar` has no prop `widht`" (typo case) |
| Nest children in a component with no `children` field | States the component does not accept children. |
| Fill a slot the component did not declare, or fill one twice | Names the slot and the component. |
| Omit a required slot | Names the missing slot. |

### 3.8 The component side of the invocation seam (IA2)

Every macro passing form has a declaration counterpart; every declaration form is reachable from the macro.
The table is closed in both directions (no orphan either side):

| Macro form (§2.7) | Component declaration | Closure |
|---|---|---|
| `name=value` attribute | Prop → generated builder setter → props field (§3.2) | Every attribute resolves to one setter; every declared prop has one setter reachable as an attribute. |
| Attribute omitted | Optional (`→ None`) or defaulted prop; or a named diagnostic if required | Omission is defined for every prop. |
| `name={expr}` where `expr` is `Signal`/`Memo` | A reactive prop (§3.2) | Reactive values pass as ordinary props; no separate form. |
| Bare content nested inside `<Comp>…</Comp>` | `children: Children` (or `Option<Children>`) | Nesting corresponds to the children field; no field makes nesting a diagnostic. |
| `<slot:label>…</slot:label>` | Named slot field `label: Slot` (or `Option<Slot>`) | Every label resolves to a declared slot or is a diagnostic; every declared slot is fillable by its label. |
| `<Comp/>` in child position | `Component::render` output, one `VNode`, spliced via `.child(..)` | Every component invocation lowers to a `VNode`; every component is invocable. |

---

## 4. Context — sharing state without prop drilling

Today a value shared by many components must be passed as a prop through every component between the
owner and the reader — even those that never touch it. Context removes that threading.

Ferric's context is three functions added to `ferric::prelude`:

```rust
// Additive over the frozen prelude.
pub fn provide_context<T: Clone + 'static>(value: T);
pub fn use_context<T: Clone + 'static>() -> Option<T>;   // None when no ancestor provided T
pub fn expect_context<T: Clone + 'static>() -> T;        // panics with a located message when absent
```

The value is keyed by its Rust type. `provide_context::<Theme>(..)` makes a `Theme` available to the
subtree; `use_context::<Theme>()` retrieves it as `Option<T>`; `expect_context::<Theme>()` is the
convenience for the common case where a provider is required, returning `T` and panicking with a message
that names the missing type.

Context carries a value; it is not itself reactive. When the provided value is a `Signal<T>` or `Memo<T>`,
a consumer that reads the handle inside its body re-renders on change — because the signal read tracks
the change, not because the lookup does. The lifetime of a provided value is bounded by the reactive owner
scope it was provided into; when that scope is disposed, the entry is released.

### 4.1 The authoring surface

```rust
use ferric::prelude::*;

#[derive(Clone, Copy)]
struct Theme { dark: Signal<bool> }

fn app() -> VNode {
    let theme = Theme { dark: create_signal(false) };
    provide_context(theme);          // visible to every descendant from here down
    view! { <Toolbar /> }
}

fn theme_toggle() -> VNode {
    let theme = expect_context::<Theme>();
    view! {
        <button on:click=move |_| theme.dark.update(|d| *d = !*d)>
            { move || if theme.dark.get() { "Switch to light" } else { "Switch to dark" } }
        </button>
    }
}
```

The `view!` forms (`<Toolbar />`, `on:click`, the interpolated closure) are fixed by §2 and §3. Context
composes against them at one point: `provide_context` and `use_context`/`expect_context` are ordinary
calls in a component body (identity IA3).

### 4.2 The no-provider case

A lookup with no matching provider is a case the author handles, not a silent default.

```rust
fn maybe_themed() -> VNode {
    match use_context::<Theme>() {
        Some(theme) => render_with(theme),
        None        => render_default(),
    }
}

fn requires_theme() -> VNode {
    let theme = expect_context::<Theme>();
    // panics: no context of type `my_app::Theme` was found; an ancestor component must call
    //         `provide_context::<my_app::Theme>(..)` before this component is rendered
    render_with(theme)
}
```

The located panic message is part of the diagnostics bar (parent `REQ-FW-014`): a missing provider is one
of the common context mistakes.

### 4.3 Reactivity through the provided handle

Context is a typed channel. When the provided value contains a reactive handle, change propagation is the
handle's — unchanged from reading a signal obtained any other way.

```rust
fn nav_bar() -> VNode {
    let session = expect_context::<Session>();
    view! {
        <span>{ move || match session.user.get() {
            Some(id) => format!("signed in as {id}"),
            None     => "signed out".to_string(),
        } }</span>
    }
}
```

`use_context::<Session>()` resolves the same `Session` every render (the lookup is not a reactive
dependency). The UI tracks `session.user` because the body reads that signal.

### 4.4 Scoped cleanup

A provided value lives exactly as long as the reactive owner scope it was provided into. When a subtree
is disposed — a component unmounts, a conditional branch is torn down — the context entries provided
within it are released with the scope, the same disposal that unsubscribes that scope's effects and memos.
No reader outside a disposed scope observes a value from it.

```rust
fn container() -> VNode {
    let show = create_signal(true);
    view! {
        { move || if show.get() { view! { <Panel /> } } else { view! { <Empty /> } } }
    }
}
// When show flips to false, Panel's subtree disposes; any `PanelState` it provided is released
// together with its scope's effects — the same drop the baseline `no_stale_subscriber_leak` tests.
```

Context lifetime is tied to the owner scope, which reuses the baseline `ReactiveScope` owner tree
directly. Context adds no second lifetime rule.

### 4.5 Design decisions

**Type-keyed lookup.** Ferric keys context by the value's Rust type: `use_context::<T>()` finds the
nearest ancestor-provided `T`. This is the convention across the Rust peers — Leptos, Dioxus (`TypeId`),
Sycamore, and Yew all use type-keyed lookup — and it composes with `Copy` signal handles: you provide a
`Signal<T>` (or a small `Copy`/`Clone` struct of them) and read it back by type with no wrapper to carry.

The alternative is Solid's model: `createContext(default)` returns an explicit `Context` object passed to
`useContext(ctx)` and to a `<Provider>`. That lets two providers of the same underlying type coexist and
carries a compile-time default. The cost is an explicit context object that every provider and reader must
route — the prop threading context exists to remove, reintroduced one level up. Ferric takes the type-
keyed side; the Rust newtype (`struct Theme(Signal<bool>)` vs `struct Sidebar(Signal<bool>)`) is the
idiomatic way to distinguish two values of the same underlying type, and its cost is local.

**`use_context → Option<T>` plus `expect_context → T`.** Ferric makes the fallible read the plain
`use_context`, matching Leptos and Yew. Dioxus and Sycamore make the plain `use_context::<T>()` panic on
a missing provider and offer `try_use_context → Option<T>` for the fallible path. The two designs cover
the same two needs. Ferric chooses the fallible read as the unadorned name because the name an author
reaches for first should not be the one that panics; the `expect` pattern (opt into the panic explicitly)
is the shape every Rust author already reads correctly from `Option::expect`.

**Value bound `T: Clone + 'static`; read clones out.** `use_context`/`expect_context` return `T` by
cloning. For Ferric the bound is close to free in the case that matters: `Signal<T>` and `Memo<T>` are
`Copy`, so a context value is a handle or a small struct of handles, and the clone is a key copy. An
author sharing a large owned value provides a `Signal` or `Rc` of it.

**Nearest-ancestor resolution with shadowing.** When the same type is provided at more than one level,
`use_context::<T>()` resolves to the nearest ancestor; an inner provider shadows an outer one for its
subtree only. This is the shared behavior of Leptos and Sycamore. It lets a subtree override app-wide
context — a modal providing a restricted `Session` — without disturbing the rest of the tree.

### 4.6 How context sits on the baseline reactive runtime

Context reuses the ownership tree `signal.rs` already maintains:

- The runtime keeps an owner stack; every reactive node records the owner that created it, and an owner
  holds its children for disposal. A component renders inside a `create_effect`, so while a component
  body runs there is a current owner on the stack.
- `provide_context::<T>(v)` records `v` against the current owner scope, keyed by `TypeId::of::<T>()`.
  `use_context::<T>()` walks the `owner` parent chain and returns the first `T`, or `None`.
- The existing `dispose` path that unsubscribes a scope's effects and memos also releases its context
  entries — one disposal path, not two. This is the `no_stale_subscriber_leak` property, unchanged.

`ReactiveScope` stays crate-internal. The author's surface is the three free functions; the owner scope
they bind to is the one the runtime already pushes around a rendering component.

### 4.7 The context seam (IA3)

The contract's context seam requires: context must be readable from the unit the macro renders (a
component), not only at mount; and must not outlive its owner scope.

1. **Readable inside a component body.** `provide_context`/`use_context`/`expect_context` are free
   functions called in a component body. The component/macro model in §2–3 renders components as
   `Fn() -> VNode` bodies running inside the mount effect, so a context call sits in the same place a
   prop or child read does. REQ-API-066.

2. **Bounded by the owner scope.** A provided value is owned by the reactive scope it was provided into
   and released by that scope's disposal — the same `no_stale_subscriber_leak` disposal path, with no
   second lifetime rule. REQ-API-068.

Every way context is provided or read is a call reachable from a rendered component. No form on either
side is orphaned.

---

## 5. How the three pieces compose

A complete example that uses all three:

```rust
use ferric::prelude::*;

// Shared theme state provided at the app root.
#[derive(Clone, Copy)]
struct Theme { dark: Signal<bool> }

// A list item component using typed props.
#[component]
fn TodoItem(label: String, done: Signal<bool>) -> VNode {
    let theme = expect_context::<Theme>();
    view! {
        <li class={ if theme.dark.get() { "dark" } else { "light" } }>
            <input type="checkbox" on:click={move || done.update(|d| *d = !*d)} />
            <span class={ if done.get() { "done" } else { "" } }>{label}</span>
        </li>
    }
}

// A panel component with a header slot and children.
#[component]
fn Panel(header: Slot, children: Children) -> VNode {
    view! {
        <section>
            <header>{header()}</header>
            <main>{children()}</main>
        </section>
    }
}

// The application root.
fn app() -> VNode {
    let theme = Theme { dark: create_signal(false) };
    let todos: Signal<Vec<(String, Signal<bool>)>> = create_signal(vec![
        ("Write design".into(), create_signal(false)),
    ]);
    provide_context(theme);
    view! {
        <Panel>
            <slot:header>
                <h1>"Todos"</h1>
                <button on:click={move || theme.dark.update(|d| *d = !*d)}>"toggle theme"</button>
            </slot:header>
            <ul>
                <For each={todos.get()} key={|(label, _)| label.clone()} let:item>
                    <TodoItem label={item.0.clone()} done={item.1} />
                </For>
            </ul>
        </Panel>
    }
}
```

This example exercises:

- The `view!` macro: element/text lowering, attribute binding, event handler, conditional class, keyed
  list, component invocation with props, named slot fill, children nesting.
- The component model: `#[component]` with typed props (`label: String`, `done: Signal<bool>`, reactive);
  a `Slot` and a `Children` on `Panel`; composition (both inside the slot and inside children).
- Context: provided at the root, read by `TodoItem` three levels deep with no prop threading.

**Invocation seam (IA2):** `<TodoItem label={..} done={..} />` passes props through the macro; `Panel`
receives a named slot fill (`<slot:header>`) and children. Every macro passing form has a component-side
declaration form; every declaration form is reachable from the macro.

**Context seam (IA3):** `expect_context::<Theme>()` is called inside `TodoItem`'s body — the unit
`view!` renders — with no special mounting hook. The theme's scope is the app root; when the app unmounts,
the theme's `Signal<bool>` and context entry dispose together with the root scope.

---

## 6. Peer comparison

All four Rust peers provide a declarative view macro. The differences that matter to an author are
reactivity model, keyed lists, and props/children ergonomics — not syntax flavor.

### 6.1 View syntax and reactivity

| Peer | Syntax | Reactivity | Keyed lists | Bearing on Ferric |
|---|---|---|---|---|
| **Leptos `view!`** | Angle-bracket HTML-like | Fine-grained signals, no VDOM — an interpolation is a fine-grained binding | `<For each key let:item>` | Ferric follows the same angle-bracket flavor; but `{expr}` in Ferric is a component re-render, not a fine-grained binding — the key difference |
| **Yew `html!`** | JSX-like angle brackets | React-like VDOM + diff | `key={..}` per item | Closest to Ferric (VDOM + macro); informs the single-root and `key` design |
| **Dioxus `rsx!`** | Rust-struct-like — `div { class: "x", .. }` | React-like VDOM + diff | `key: ..` per item | A considered flavor, rejected for the HTML-like one; the VDOM model is shared |
| **Sycamore `view!`** | Terse — `div(class="x") { "text" }` | Fine-grained signals, no VDOM | `Keyed`/`Indexed` views | A considered flavor; the same fine-grained caveat as Leptos |
| **Vue 3 / Solid (JS analogs)** | Templates / JSX | Vue 3: signals + VDOM; Solid: fine-grained | `v-for :key` / keyed `For` | Vue 3 is Ferric's closest architectural analog |

**Syntax flavor** is the least consequential axis. Angle brackets (Leptos/Yew) read like markup; Dioxus
and Sycamore are quieter and more Rust-native but read less like HTML. Ferric picks angle brackets to
match the mental model of an author evaluating it against JS frameworks.

**Reactivity model** is where the design must be precise. Leptos and Sycamore update only the text node
an interpolation feeds, without re-running the component. Ferric, like Yew/Dioxus/Vue 3, re-runs the
component and diffs. An author porting a Leptos mental model will expect fine-grained updates and must
be told, up front, that Ferric's `{expr}` triggers a full component re-render reconciled by the keyed
diff. The macro is equally ergonomic to write; the performance characteristics differ.

**Keyed lists** converge: every peer makes the author supply a key. Ferric's constraint is sharper — the
diff only keys a sibling list when all siblings are keyed on both sides — which is why the `<For>` form
bakes the key in rather than leaving it optional.

### 6.2 Props, children, and context

| Peer | Props declaration | Props `PartialEq` required? | Children | Context |
|---|---|---|---|---|
| **Leptos** | `#[component]` fn; `#[prop(optional/default/into)]` | No (signal-reactive) | `Children: ChildrenFn` | `provide_context` / `use_context → Option` / `expect_context` |
| **Sycamore** | `#[component]` + `Props`-deriving struct | No (signal-reactive) | `Children` callable | `provide_context` / `use_context` (panics) / `try_use_context → Option` |
| **Dioxus** | `#[component]` + `#[derive(Props)]`; `#[props(default/optional)]` | Yes | `children: Element` | `use_context_provider` / `use_context::<T>()` / `try_use_context` |
| **Yew** | `#[derive(Properties, PartialEq)]` on a struct + `#[function_component]` | **Yes** (prop-diffing) | `children: Children`/`Html` | `ContextProvider<T>` + `use_context::<T>()`→`Option` |
| **Ferric** (this model) | `#[component]` fn; `#[prop(optional/default/into)]` | **No** (signal-reactive) | `children: Children` callable | `provide_context` / `use_context → Option` / `expect_context` |

Ferric follows Leptos most closely in props declaration vocabulary (deliberately; the attribute names are
adopted directly), and in not requiring `PartialEq` for the same signal-reactive reason. The sharpest
divergence is with Yew and Dioxus, which require `PartialEq` for prop-diff-based re-render skipping;
Ferric does not use that mechanism.

---

## 7. Requirements register — `REQ-API-001 … REQ-API-079`

All requirements are `release_gating: true` unless noted. `verification_method` ∈ {`automated_test`,
`example_build`, `doc_inspection`} (inherited from `../../contracts/id_and_traceability_schema.yaml`,
identity IA6). Stage: `usable_early` = the declarative authoring floor (parent `REQ-FW-005`/`REQ-FW-014`);
`one_oh` = additional capabilities. Cross-references to `REQ-FW-*`, `REQ-ASYNC-*`, `REQ-PERF-*` are
read-only.

### Band A — the `view!` macro (REQ-API-001 … 029)

Owned by `SUBTASK_view_macro`. Ids zero-padded, monotonic within band (IA1).

---

**REQ-API-001 — A `view!` macro is the declarative authoring entry point.**
- *Statement:* An author can describe an element tree declaratively with `view!` instead of hand-building
  `VElement` builder chains.
- *Acceptance:* `view! { <div class="counter"><p>"hi"</p></div> }` compiles and evaluates to a `VNode`
  equal to the equivalent builder tree. *(automated_test, example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-002 — Element nodes lower to `VElement::new(tag)`.**
- *Statement:* `<tag>…</tag>` and `<tag/>` produce an element with that tag.
- *Acceptance:* `<br/>` produces `VElement::new("br")`; `<div>…</div>` produces a `div` with nested
  children in order. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-003 — Text children lower to `vdom::text`.**
- *Statement:* A quoted string literal, and a `{expr}` whose value is `impl Into<String>`, each produce a
  text node.
- *Acceptance:* `<p>"count: " {n.to_string()}</p>` produces two text children. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-004 — Static and dynamic attributes lower to `.attr(name, value)`.**
- *Statement:* `name="literal"` and `name={expr}` (expr `Into<String>`) set an attribute.
- *Acceptance:* `<a href="/x" class={c}>` produces attrs `("href","/x")` and `("class", c.into())`.
  *(automated_test, example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-005 — Nesting and mixed children lower to ordered `.child` calls.**
- *Statement:* Nested elements and interleaved text/element children appear as children in source order.
- *Acceptance:* a `<div>` with text, an element, and an interpolation produces three children in written
  order. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-006 — `on:<name>={closure}` lowers to `.on(EventKind::<Variant>, closure)`.**
- *Statement:* An author binds an event with `on:click`/`on:input`, supplying a `Fn()` closure.
- *Acceptance:* `<button on:click={h}>` produces `(EventKind::Click, _)`; the counter example increments
  on click. *(automated_test, example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005, REQ-FW-014.

**REQ-API-007 — The event-name set is open and additive.**
- *Statement:* `click`/`input` resolve today; additional names resolve as `REQ-ASYNC-*` extends
  `EventKind`, with no change to the macro grammar.
- *Acceptance:* `on:click` and `on:input` compile against the baseline; a doc note states new names
  become available when `EventKind` gains the variant. An unmapped name fails per REQ-API-026.
  *(doc_inspection, automated_test)*
- *Stage:* usable_early · *Refs:* REQ-ASYNC-* (read-only), REQ-FW-006.

**REQ-API-008 — Interpolated reactive values re-render on change.**
- *Statement:* A `{expr}` that reads a `Signal`/`Memo` re-renders the component when that signal changes.
- *Acceptance:* a headless `wasm-bindgen-test` mounts `<p>{count.get()}</p>`, calls `count.set(..)`,
  and observes the rendered text update. *(automated_test, example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-009 — The reactivity model is stated as coarse-grained.**
- *Statement:* The docs state that a signal read in a view re-renders the whole component, and that Ferric
  provides no fine-grained per-node binding.
- *Acceptance:* a reviewer confirms the authoring guide states the coarse-grained model and contrasts it
  with fine-grained peers. *(doc_inspection)*
- *Stage:* usable_early · *Refs:* REQ-FW-005; REQ-PERF-* (read-only).

**REQ-API-010 — Conditional rendering yields a `VNode`, with an explicit empty node.**
- *Statement:* A Rust `if`/`match`/ternary in interpolation position produces a `VNode`; the "render
  nothing" branch produces an explicit empty node.
- *Acceptance:* a conditional view compiles with both branches producing a `VNode`; the empty branch
  renders no visible content. *(automated_test, example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-011 — Kind/tag-changing conditionals reconcile as a `Replace`.**
- *Statement:* When a conditional's branches differ in node kind or tag, toggling rebuilds that position
  wholesale, and this is documented.
- *Acceptance:* a test toggles between `<span>` and `<a>` and observes a `Replace`-shaped patch; the
  guide states this. *(automated_test, doc_inspection)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-012 — A keyed-list form produces a `Vec<VNode>` spliced via `.children`.**
- *Statement:* `<For each key let:item>` iterates items and produces a child list.
- *Acceptance:* a `<For>` over three items produces a `<ul>` with three `<li>` children in order.
  *(automated_test, example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-013 — The keyed-list form produces a fully-keyed sibling list.**
- *Statement:* Every element produced by the list form carries its key, satisfying the diff's `all_keyed`
  precondition; reordering reconciles by identity.
- *Acceptance:* reordering items produces `MoveChild` patches, no `Replace` of keyed nodes.
  *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-014 — `key={expr}` lowers to `.key(expr)`; mixing keyed and unkeyed siblings is documented.**
- *Statement:* A standalone `key={expr}` sets an element's key; the guide states that an unkeyed sibling
  in the same parent drops that parent to positional diffing.
- *Acceptance:* `<li key={id}>` produces `.key(id)`; a reviewer confirms the caveat is documented.
  *(automated_test, doc_inspection)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-015 — Capitalized tag = component; lowercase tag = HTML element.**
- *Statement:* The macro routes by tag token alone.
- *Acceptance:* `<div/>` is an element; `<TodoList/>` is a component invocation. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-005; band B (IA2).

**REQ-API-016 — A macro prop `name={value}` corresponds to a declared prop.**
- *Statement:* Passing `name={value}` maps to a prop the component declares (band B).
- *Acceptance:* a stated example passes a prop through the macro; it compiles and renders.
  *(example_build, automated_test)*
- *Stage:* usable_early · *Refs:* band B (IA2).

**REQ-API-017 — Children nested in a component element correspond to its declared children acceptance.**
- *Statement:* Nesting children inside `<Comp>…</Comp>` maps to the component's declared `children`.
- *Acceptance:* a stated example nests children; it compiles and renders. *(example_build)*
- *Stage:* usable_early · *Refs:* band B (IA2).

**REQ-API-018 — A `<slot:name>` fill corresponds to a component-declared named slot.**
- *Statement:* Filling `<slot:name>…</slot:name>` maps to a named slot the component declares.
- *Acceptance:* a stated example fills a named slot; it compiles and renders into the slot position.
  *(example_build)*
- *Stage:* usable_early · *Refs:* band B (IA2).

**REQ-API-019 — A component invocation lowers to the component model's invocation form, yielding a `VNode`.**
- *Statement:* The macro splices the `VNode` the component model produces; it does not redefine the
  component model.
- *Acceptance:* the macro's component output is a `VNode` child; no frozen component API is redefined.
  *(automated_test, doc_inspection)*
- *Stage:* usable_early · *Refs:* band B (IA2), REQ-FW-009 (IA4).

**REQ-API-020 — A top-level `view!` yields exactly one root node.**
- *Statement:* Multiple top-level nodes are rejected with a diagnostic.
- *Acceptance:* a single-root `view!` compiles; a two-root `view!` fails per REQ-API-024. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-009 (IA4).

**REQ-API-021 — Child-position splicing of multiple nodes and the `VNode` escape hatch.**
- *Statement:* `{Vec<VNode>}`, `<For>`, and sibling tags splice multiple children; any `{expr}` yielding
  a `VNode` is spliced directly.
- *Acceptance:* `{vec_of_vnodes}` renders each as a child; a `{builder_vnode}` splice renders the
  builder-produced node. *(automated_test, example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-022 — Two-way interop with the builder API.**
- *Statement:* `view!` output is an ordinary `VNode` usable anywhere the builders are, and builder-produced
  `VNode`s are usable inside a view via interpolation.
- *Acceptance:* a `view!`-produced node is passed to `.child(..)` and vice versa; both compile and render.
  *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-009.

**REQ-API-023 — Every construct lowers additively onto the frozen surface; nothing is redefined (IA4).**
- *Statement:* All macro output uses frozen constructors; `view!` and re-exported `text` are additive only.
- *Acceptance:* a `view!` tree and the equivalent hand-built tree produce equal `VNode`s; an app against the
  baseline `prelude` still compiles with the macro present. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-009 (IA4).

**REQ-API-024 — Structural view errors produce a diagnostic at the author's span.**
- *Statement:* Unclosed elements, mismatched closing tags, and multiple top-level nodes each name the
  problem at the offending tokens.
- *Acceptance:* `trybuild` fixtures produce expected `stderr` for each case. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-014.

**REQ-API-025 — A wrong-typed attribute or text value is diagnosed at the expression span.**
- *Statement:* A dynamic attribute or text interpolation not `Into<String>` produces a type error pointing
  at the `{expr}`.
- *Acceptance:* a `trybuild` fixture fails at the expression span. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-014.

**REQ-API-026 — An unknown event name is diagnosed with the bound set.**
- *Statement:* `on:<name>` with no matching `EventKind` variant produces an error naming the unknown event
  and listing the bound kinds.
- *Acceptance:* a `trybuild` fixture `on:klick=` fails with "unknown event `klick`; bound events are:
  click, input". *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-014, REQ-ASYNC-* (read-only).

**REQ-API-027 — The keyless-list case is diagnosed best-effort, with its limit documented.**
- *Statement:* The direct `<For>` form flags a missing `key`; the docs state that `{iter.map(..)}` lists
  cannot be fully checked and that `<For>` is the keyed default.
- *Acceptance:* a `trybuild` fixture for a keyless `<For>` fails; a reviewer confirms the documented limit.
  *(automated_test, doc_inspection)*
- *Stage:* usable_early · *Refs:* REQ-FW-014.

**REQ-API-028 — The diagnostics story as a whole is `trybuild`-checkable (IA6).**
- *Statement:* A fixture suite of malformed views, each with a committed expected `stderr`, is part of the
  test suite.
- *Acceptance:* `cargo test` runs the `trybuild` suite and all fixtures match their committed `stderr`.
  *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-014.

**REQ-API-029 — Whitespace handling is predictable and documented.**
- *Statement:* Only quoted text is rendered; whitespace between tags is insignificant and discarded, and
  the rule is documented.
- *Acceptance:* a multi-line view renders only its quoted text with no injected spaces; a reviewer confirms
  the documented rule. *(automated_test, doc_inspection)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

---

### Band B — the component model (REQ-API-030 … 059)

Owned by `SUBTASK_components_props_slots`. Ids 030–052 assigned; 053–059 reserved headroom (IA1).

---

**REQ-API-030 — A component declares typed props as a `#[component]` function's parameters.**
- *Statement:* `#[component] fn Name(..) -> VNode` parameters are its typed props; the attribute generates
  a typed props type and builder the `view!` macro fills.
- *Acceptance:* a stated `#[component]` with two props compiles; `view!`-invoking it renders the described
  UI. *(example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-031 — Required props must be supplied.**
- *Statement:* A prop with no default and no `optional` must be supplied; omission is a compile error.
- *Acceptance:* supplying every required prop builds and renders; omission covered by REQ-API-050.
  *(example_build)*
- *Stage:* usable_early.

**REQ-API-032 — Optional props default to absent.**
- *Statement:* `#[prop(optional)]` makes the prop `Option<T>` defaulting to `None` when omitted.
- *Acceptance:* a stated component with an optional prop renders both when supplied and when omitted (the
  omitted case observably takes the `None` branch). *(example_build)*
- *Stage:* usable_early.

**REQ-API-033 — Defaulted props take a declared default.**
- *Statement:* `#[prop(default = expr)]` takes `expr` when the caller omits the prop.
- *Acceptance:* a stated component renders with the default when omitted and with the supplied value when
  present. *(example_build)*
- *Stage:* usable_early.

**REQ-API-034 — Converting props accept `Into` conversions.**
- *Statement:* `#[prop(into)]` accepts any `T: Into<FieldType>`.
- *Acceptance:* a stated component with an `into` `String` prop compiles when called with a `&str` literal.
  *(example_build)*
- *Stage:* one_oh · *Refs:* Leptos `#[prop(into)]`.

**REQ-API-035 — Props may be reactive handles.**
- *Statement:* A `Signal<T>` or `Memo<T>` prop passes the handle; reading it in the body re-renders the
  parts that read it on change.
- *Acceptance:* a headless test mounts a component taking `Signal<i32>`, changes the signal, and observes
  the DOM update. *(automated_test)*
- *Stage:* usable_early · *Refs:* baseline `signal.rs`.

**REQ-API-036 — Props do not require `PartialEq`.**
- *Statement:* Declaring and using props does not require prop types to implement `PartialEq`.
- *Acceptance:* a stated component whose prop type does not implement `PartialEq` compiles and renders.
  *(example_build)*
- *Stage:* usable_early · *Refs:* contrast Yew `#[derive(Properties, PartialEq)]`.

**REQ-API-037 — A zero-prop component needs no props struct.**
- *Statement:* A component with no props is the baseline function component unchanged; `#[component]` on a
  zero-parameter function generates an empty props type.
- *Acceptance:* the baseline `counter` bare closure still compiles and renders as a `Component` at 1.0; a
  stated zero-prop `#[component]` also compiles. *(example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-009; baseline `component.rs`.

**REQ-API-038 — A component accepts children via a `Children` prop.**
- *Statement:* Declaring `children: Children` accepts child content as a deferred thunk.
- *Acceptance:* a stated `Card`-style component renders caller-nested content at the point it calls
  `children()`; a multi-node block renders all nodes. *(example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-039 — Children lower onto `VElement.children` additively and stay reactive.**
- *Statement:* `{children()}` lowers onto the baseline `.children(..)` builder; signals read within child
  content stay tracked.
- *Acceptance:* a headless test mounts a component whose children read a signal, changes the signal, and
  observes the children update; the children block produces the same `VElement.children` a hand-built tree
  would. *(automated_test)*
- *Stage:* usable_early · *Refs:* baseline `vdom.rs`; IA4.

**REQ-API-040 — Optional children.**
- *Statement:* `#[prop(optional)] children: Option<Children>` accepts children only when provided.
- *Acceptance:* a stated component renders correctly both with and without a children block.
  *(example_build)*
- *Stage:* usable_early.

**REQ-API-041 — A component declares named slots as typed fields.**
- *Statement:* A component needing more than one labelled region declares each as a `Slot` field; the
  author fills each by its label.
- *Acceptance:* a stated `Page`-style component with `header` and `footer` slots renders labelled content
  in matching regions. *(example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-042 — Each named slot is filled at most once.**
- *Statement:* Filling a slot a component did not declare, or filling one twice, is a diagnostic.
- *Acceptance:* a single correct fill per slot builds and renders; error cases covered by REQ-API-052.
  *(example_build)*
- *Stage:* usable_early.

**REQ-API-043 — Required vs optional slots.**
- *Statement:* A `Slot` field is required; an `Option<Slot>` field is optional.
- *Acceptance:* a stated component renders with an optional slot both filled and omitted; missing-required-
  slot case covered by REQ-API-052. *(example_build)*
- *Stage:* usable_early.

**REQ-API-044 — Named slots and children coexist.**
- *Statement:* A component may declare named slots and a `children` field together.
- *Acceptance:* a stated component with `header`/`footer` slots and a `children` body renders labelled and
  unlabelled content correctly from one `view!` block. *(example_build)*
- *Stage:* usable_early.

**REQ-API-045 — Parameterized slots (render-prop).**
- *Statement:* A `SlotFn<T>` slot lets a component pass a value into caller-provided content.
- *Acceptance:* a stated generic `List<T>` with `SlotFn<T>` renders one templated row per item.
  *(example_build)*
- *Stage:* one_oh · *Refs:* Dioxus render-props.

**REQ-API-046 — Components compose by nesting through their declared surface.**
- *Statement:* A component is invoked in another's `view!` body, filled through its declared surface; the
  invocation lowers to its `Component::render` output spliced as a `VNode`.
- *Acceptance:* a stated multi-level composition (component in a slot, component in children) compiles and
  renders the combined UI. *(example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-047 — Composition is additive over the frozen `Component`/`VNode` surface.**
- *Statement:* Generated components, `impl Component`, and `Fn() -> VNode` closures all compose; no frozen
  item is redefined.
- *Acceptance:* a stated composition mixes all three forms and renders; a baseline app still compiles at
  1.0. *(example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-009; IA4.

**REQ-API-048 — The declaration macro is a removable escape-hatch sugar.**
- *Statement:* `#[component]` is sugar over `Component`; any component can be written against the trait
  directly; hand-written and generated components interoperate.
- *Acceptance:* a stated component is written once with `#[component]` and once as `impl Component`, and
  both are used interchangeably. *(example_build)*
- *Stage:* usable_early.

**REQ-API-049 — Dynamic-dispatch composition is carried forward.**
- *Statement:* The baseline's `&dyn Component` / `Box<dyn Component>` dispatch remains available.
- *Acceptance:* a stated component holding `Vec<Box<dyn Component>>` renders each; the baseline trait-
  object test still passes. *(automated_test)*
- *Stage:* one_oh · *Refs:* baseline `component.rs`.

**REQ-API-050 — Missing or mistyped required prop is an actionable diagnostic.**
- *Statement:* Omitting a required prop or passing a wrong-typed prop produces a diagnostic naming the
  prop, the component, and the expected type — not a raw generated-builder error.
- *Acceptance:* `trybuild` fixtures for (a) missing required prop and (b) wrong-typed prop each fail with
  a message naming the prop and component. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-014.

**REQ-API-051 — Unknown prop is an actionable diagnostic.**
- *Statement:* Passing a prop the component does not declare produces a diagnostic naming the unknown prop
  and the component.
- *Acceptance:* a `trybuild` fixture fails with the unknown prop and component named. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-014.

**REQ-API-052 — Children and slot misuse are actionable diagnostics.**
- *Statement:* Nesting children in a childless component, filling an undeclared slot, filling a slot
  twice, or omitting a required slot each produces a diagnostic naming the component and region.
- *Acceptance:* `trybuild` fixtures for each case fail with a message naming the component and region.
  *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-014.

---

### Band C — context (REQ-API-060 … 079)

Owned by `SUBTASK_context`. Ids 060–070 assigned; 071–079 reserved headroom (IA1). All `usable_early`.

---

**REQ-API-060 — Provide a value to a subtree.**
- *Statement:* `provide_context(value)` once in an ancestor makes the value available to the subtree
  without passing it as a prop to intermediates.
- *Acceptance:* an example with a root caller and a descendant three or more components deep reads the
  value with no intermediate forwarding, compiles and renders. *(example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-061 — Read a provided value without prop drilling.**
- *Statement:* `use_context::<T>()` returns `Option<T>` from inside any descendant component body.
- *Acceptance:* a headless test provides a `T`, reads it via `use_context::<T>()`, and asserts the
  returned `Some(T)` equals the provided value. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-062 — Typed lookup keyed by the value's type; nearest-ancestor resolution.**
- *Statement:* `use_context::<T>()` resolves to the nearest ancestor-provided `T`; an inner-scope provider
  shadows an outer one for that subtree only.
- *Acceptance:* a test provides `T` at outer and inner scopes; a reader in the inner subtree sees the inner
  value; a reader outside sees the outer. A second case confirms two distinct types provided at one scope
  are both readable. *(automated_test)*
- *Stage:* usable_early.

**REQ-API-063 — The no-provider case is explicit and located.**
- *Statement:* `use_context::<T>()` returns `None` when no ancestor provided `T`; `expect_context::<T>()`
  panics with a message naming `T` and stating an ancestor must provide it.
- *Acceptance:* a test with no provider asserts `use_context` is `None`; a second case asserts
  `expect_context` panics with the type name in the message. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-014.

**REQ-API-064 — A provided reactive value drives consumer re-render.**
- *Statement:* When the provided value contains a `Signal<T>`/`Memo<T>`, a consumer that reads the handle
  re-renders when the handle changes.
- *Acceptance:* a headless test provides a struct holding a `Signal`, renders a consumer that reads it,
  mutates the signal, and asserts the consumer re-rendered. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-005.

**REQ-API-065 — Context lookup is not itself a reactive dependency.**
- *Statement:* `use_context`/`expect_context` establish no reactive subscription; re-render is driven only
  by signal reads from the returned value.
- *Acceptance:* a headless test provides a non-reactive value, reads it via `use_context` in a component,
  and asserts the component does not re-render when unrelated signals change; a `doc_inspection` confirms
  the guide states the lookup is untracked. *(automated_test)*
- *Stage:* usable_early.

**REQ-API-066 — Context is readable from inside a component body (seam IA3).**
- *Statement:* `provide_context`/`use_context`/`expect_context` are callable from inside a component
  body — the unit the `view!` macro renders.
- *Acceptance:* an example in which a `view!`-rendered function component both calls `expect_context` and
  `provide_context` for its own children compiles and renders; a `doc_inspection` confirms the context and
  component sections agree that context calls occur in a component body. *(example_build)*
- *Stage:* usable_early.

**REQ-API-067 — App-global context from the application root.**
- *Statement:* `provide_context` in the root component covers the whole application, using the existing
  `mount` with no signature change.
- *Acceptance:* an example that calls `provide_context` in the root and reads it from a leaf builds and
  renders the shared value. *(example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-009.

**REQ-API-068 — Context lifetime is bounded by the reactive owner scope (scoped cleanup, seam IA3).**
- *Statement:* A value provided within a scope is released when that scope disposes; after disposal no
  provided value from that scope is reachable, and no stale reference survives.
- *Acceptance:* a test provides context inside a scope, disposes the scope, and asserts (a) a subsequent
  lookup does not return the released value and (b) the owner node and context entry are reclaimed —
  mirroring the baseline `no_stale_subscriber_leak` tests. *(automated_test)*
- *Stage:* usable_early · *Refs:* REQ-FW-009.

**REQ-API-069 — Context surface is additive over the frozen prelude (IA4).**
- *Statement:* `provide_context`, `use_context`, and `expect_context` are added to `ferric::prelude` as
  new items; no existing item changes; `ReactiveScope` stays crate-internal.
- *Acceptance:* a `doc_inspection` confirms the three functions are additions and nothing frozen is re-
  signed; an `example_build` confirms a program using only the baseline prelude still compiles. *(doc_inspection, example_build)*
- *Stage:* usable_early · *Refs:* REQ-FW-009.

**REQ-API-070 — Value bound and clone-out read semantics.**
- *Statement:* Context values satisfy `T: Clone + 'static`; reads clone out. The idiomatic value is a
  `Copy` reactive handle, so the clone is a handle copy.
- *Acceptance:* a test provides a `Copy` handle struct, reads it twice via `use_context`, and asserts both
  reads refer to the same underlying signal; a `doc_inspection` confirms the guide states the `Clone +
  'static` bound and the handle idiom. *(automated_test)*
- *Stage:* usable_early.

---

## 8. Traceability and metadata

- **Owns:** `REQ-API-001 … REQ-API-079`.
  - Band A (001–029): `SUBTASK_view_macro`.
  - Band B (030–052): `SUBTASK_components_props_slots`; 053–059 reserved.
  - Band C (060–070): `SUBTASK_context`; 071–079 reserved.
- **Specializes (read-only):** parent `REQ-FW-005` (declarative authoring model), `REQ-FW-009` (additive
  over frozen surfaces), `REQ-FW-014` (actionable diagnostics).
- **Cross-area references (read-only):** `REQ-ASYNC-*` (event-payload handler form, full `EventKind` set,
  forms/events), `REQ-PERF-*` (cost of coarse-grained re-render).
- **Identities asserted (this level):** IA1 (disjoint bands), IA2 (invocation seam closed), IA3 (context
  seam closed), IA4 (additive over frozen surface), IA5 (one deliverable, no collision), IA6 (trybuild-
  checkable diagnostics, inherited verification methods).
- **Contract:** `contracts/` at the authoring-model subtree workspace — `_MANIFEST.yaml` and
  `req_api_partition.yaml`. Extends `../../contracts/` (Ferric 1.0 id/traceability schema, stage/
  verification vocabulary). Overrides nothing.
- **Grounding:** `Nousix-LLC/nswe-demo` @ `276138f`, baseline `taskflow/dags/ferric-foundation/checkout/
  crates/ferric/` (`vdom.rs`, `component.rs`, `signal.rs`, `dom.rs`, `lib.rs`, `examples/counter.rs`).
- **Peers compared by name:** Leptos, Yew, Dioxus, Sycamore (Rust); Solid, Vue 3 (JS analogs).
- **Register summary:** 63 requirements assigned (`REQ-API-001..029`, `030..052`, `060..070`);
  16 reserved headroom (`053..059`, `071..079`). Stage split: `usable_early` — 001–033, 035–044, 046–048,
  050–052, 060–070; `one_oh` — 034, 045, 049. All `release_gating: true`.
