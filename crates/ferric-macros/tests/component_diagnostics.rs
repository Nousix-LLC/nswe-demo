//! Compile-fail fixture runner for component authoring diagnostics.
//!
//! Each fixture in `compile_fail_component/` is a small Rust program that
//! triggers exactly one component authoring mistake.  The committed `.stderr`
//! files are the acceptance contract for BAR-FERRIC-010 — the messages must
//! name the author's mistake in terms of their code, not in terms of macro
//! internals or generated type names.
//!
//! Run `TRYBUILD=overwrite cargo test --test component_diagnostics` to
//! regenerate the `.stderr` snapshots after a diagnostic change.

#[test]
fn component_diagnostics() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail_component/*.rs");
}
