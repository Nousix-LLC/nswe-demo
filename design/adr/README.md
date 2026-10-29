# Architecture Decision Records

This directory holds the architecture decision records (ADRs) for `chirp`. Each ADR captures one
significant decision, the context that forced it, the alternatives weighed, and the consequences
accepted. ADRs are append-only history: a decision is never rewritten in place — it is superseded
by a later ADR that links back to it.

An ADR here records *how* a release-bar requirement is met in a way the design documents under
`design/` deliberately leave open (the design states *what* must be true; an ADR fixes the
engineering choice). Each ADR traces its rationale to the `BAR-*` / `REQ-*` ids it serves.

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [0001](0001-durable-store-and-persistence-posture.md) | Durable store, replica/single-writer posture, and schema-versioning approach | Accepted | 2026-10-13 |
