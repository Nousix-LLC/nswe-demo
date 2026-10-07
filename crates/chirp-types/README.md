# chirp-types

The **shared domain model and client↔server API contract** for `chirp`, the Twitter-style
application in this engagement (work-item #7).

This crate is the *single source of truth for every type that crosses the wire*. Both
[`chirp-server`](../../) (#8, axum) and [`chirp-frontend`](../../) (#9, the ferric SPA) depend on
it, so a request type defined here once is the identical type on both ends of the call — the
server cannot encode a shape the frontend cannot decode, because there is only one definition.

It is deliberately **dependency-light and side-effect-free**: pure data types, their validation,
and `serde` derives. No I/O, no framework, no async, no `wasm`/server dependencies. That is what
lets a native server and a `wasm` frontend both import it unchanged.

## What's inside

| Module | Contents |
|--------|----------|
| [`ids`]    | Opaque, type-safe id newtypes (`UserId`, `ChirpId`) and `Timestamp` (epoch-millis, UTC). |
| [`domain`] | Entities (`User`, `Chirp`, `Follow`, `Like`, `Timeline`) and validated value objects (`Username`, `ChirpText`). |
| [`api`]    | Request/response DTOs, the generic `Page<T>` pagination envelope, and the wire error contract (`ApiError` / `ErrorCode`). |

[`ids`]: ./src/ids.rs
[`domain`]: ./src/domain.rs
[`api`]: ./src/api.rs

## The serde field-naming convention (stable contract)

Every type derives `Serialize` + `Deserialize`. The wire conventions are **explicit and stable**:

- **camelCase object fields.** Every multi-word field is renamed with
  `#[serde(rename_all = "camelCase")]` — `created_at` → `createdAt`, `replyTo`, `followerCount`,
  and so on. The rename is pinned by the attribute, *not* by the Rust field identifier, so a
  future refactor that renames a Rust field does **not** change the JSON. The wire format is a
  deliberate boundary, not an accidental reflection of the Rust internals.
- **Transparent opaque wrappers.** Id newtypes, `Cursor`, and `SessionToken` use
  `#[serde(transparent)]`, so they serialize as the bare inner value — `UserId(42)` is just `42`,
  not `{"0": 42}`.
- **Validation on deserialize.** `Username` and `ChirpText` use `#[serde(try_from = "String")]`:
  deserializing an invalid value *fails* rather than yielding an unchecked instance. The same
  guarantee their constructors give, extended to the wire.
- **SCREAMING_SNAKE_CASE error codes.** `ErrorCode` serializes as e.g. `"VALIDATION_ERROR"`, the
  conventional machine-readable REST error-code shape.
- **Timestamps are JSON numbers** — milliseconds since the Unix epoch, UTC. No date-library
  dependency; consumers convert at the edge.

### Example JSON

```jsonc
// A Chirp
{
  "id": 100,
  "authorId": 1,
  "text": "hello, chirp!",
  "createdAt": 1700000000000,
  "likeCount": 0,
  "replyTo": null
}

// An error
{ "code": "VALIDATION_ERROR", "message": "invalid input",
  "details": [ { "field": "username", "message": "must not be empty" } ] }
```

## How the server and frontend consume it

Add it as a path/workspace dependency and import through the prelude:

```rust
use chirp_types::prelude::*;

// Frontend: build a request the server will accept.
// `ChirpText::parse` validates at the boundary and returns Result<_, ValidationError>.
let text = ChirpText::parse("hello, chirp!").expect("valid body");
let req = CreateChirpRequest { text, reply_to: None };

// Server: decode the same `CreateChirpRequest`, act, and reply with a shared
// domain/DTO type (`Chirp`, `User`, `Page<Chirp>`, ...).
```

(The crate root carries a compiled doctest of this flow; run `cargo test -p chirp-types`.)

- **Server (#8)**: accepts the `*Request` DTOs as handler input and returns the entity types
  (`User`, `Chirp`) and the `*Response` DTOs. It chooses the serialization format (JSON via its
  framework); this crate does not impose one.
- **Frontend (#9)**: constructs the `*Request` DTOs and decodes the responses from the same
  definitions. Validated value objects (`Username`, `ChirpText`) let the UI reject bad input
  *before* a round-trip.

Where a response simply *is* an entity, the contract reuses the domain type directly rather than
cloning its shape into a parallel DTO. Dedicated DTOs exist only where the wire shape genuinely
differs from an entity: client inputs, the pagination envelope, action results, and errors. The
URLs in [`api`]'s endpoint map are illustrative — this crate fixes the **types**, not the routes.

## Design decisions & trade-offs

- **Ids are `u64` newtypes, not UUIDs.** Keeps the contract dependency-free and the wire shape a
  plain number. Ids are opaque and server-assigned; the newtype means the representation can
  change later without touching the field contract. (A `uuid` dependency was considered and
  rejected under the dependency-light constraint.)
- **No `data`/`meta` success envelope.** Endpoints return the resource representation directly;
  only errors use the structured `ApiError`. A wrapping envelope was considered and rejected as
  per-endpoint coupling the contract does not need.
- **`serde_json` is a dev-dependency only.** The types just derive `Serialize`/`Deserialize`;
  choosing a concrete format is the consumer's call, so consumers are not forced onto
  `serde_json`.

## Testing

Native, no browser or network required:

```sh
cargo test -p chirp-types
```

Tests cover serde round-trips, the camelCase/transparent/SCREAMING_SNAKE conventions, and the
value-object invariants (including validation-on-deserialize).
