//! Cross-cutting concurrency & invariant suite for the durable `ChirpRepository` backing.
//!
//! This is the verification harness of the durable-persistence work: it drives **concurrent load**
//! against the durable store and asserts the subtree identities hold under that load, distinct from
//! the per-method unit tests the implementation stages already wrote. It also guarantees that the
//! default `cargo test` stays runnable **store-free** against the retained in-memory implementor.
//!
//! # Two halves, two gates
//!
//! * **Store-free half** (`store_free`, always compiled). Exercises the retained
//!   [`InMemoryRepository`] under concurrency through the public [`ChirpRepository`] trait. It needs
//!   no database and no `postgres` feature, so it runs on a plain `cargo test` and is the positive,
//!   automated evidence that the in-memory implementor still backs the test suite store-free.
//! * **Durable half** (`durable`, `#[cfg(feature = "postgres")]`). Drives the same kinds of
//!   concurrent load against the durable [`PostgresRepository`] and asserts every identity against
//!   the durable backing. It is gated two ways so the durable backing is *selectable* without making
//!   the default test run require a store:
//!     1. the whole module compiles only under `--features postgres` (a plain `cargo test` never
//!        pulls in `sqlx` or this code at all); and
//!     2. even under `--features postgres`, each test **skips** (returns early) unless
//!        `CHIRP_TEST_DATABASE_URL` names a reachable test database — mirroring the per-method live
//!        tests' convention, so `cargo test --features postgres` with no database stays green.
//!
//!   To run the durable half, point `CHIRP_TEST_DATABASE_URL` at a throwaway database and run
//!   `cargo test --features postgres`.
//!
//! # Identity → test coverage map (durable half)
//!
//! Every identity is asserted by at least one test **under concurrent access** against the durable
//! backing; a violation surfaces as a test failure (there are no silently-passing gaps — each test
//! both drives real concurrency and reconciles the stored state against the edge set it implies).
//!
//! | Identity | Asserted by |
//! |----------|-------------|
//! | `counts_match_edges` | `follower_count_matches_edge_set_under_concurrent_distinct_followers`, `like_count_matches_edge_set_under_concurrent_distinct_likers`, `concurrent_like_then_unlike_settles_to_the_edge_set`, `delete_account_cascade_reconciles_counts_atomically_under_concurrency` |
//! | `multiwrite_atomic` | `concurrent_duplicate_follow_by_one_follower_counts_exactly_once`, `concurrent_like_then_unlike_settles_to_the_edge_set`, `delete_account_cascade_reconciles_counts_atomically_under_concurrency`, the follower/like fan-outs above |
//! | `ids_opaque_monotonic_stable` | `concurrently_allocated_ids_are_unique_and_stable_across_a_fresh_connection`, `timeline_cursor_paging_is_exactly_once_under_concurrent_readers` |
//! | `confirmed_write_durable` | `confirmed_concurrent_writes_are_durable_across_a_fresh_connection`, `total_state_is_durable_and_pod_independent_under_concurrency` |
//! | `domain_boundary_validated` | `entity_invariants_hold_at_the_durable_boundary_under_concurrency`, `timeline_cursor_paging_is_exactly_once_under_concurrent_readers` (reverse-chronological) |
//! | `total_state_coverage` | `total_state_is_durable_and_pod_independent_under_concurrency`, `confirmed_concurrent_writes_are_durable_across_a_fresh_connection` |

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use chirp_server::ChirpRepository;
use chirp_server::InMemoryRepository;
use chirp_types::api::{CreateChirpRequest, CreateUserRequest, TimelineQuery};
use chirp_types::domain::{Chirp, ChirpText, User, Username};
use chirp_types::ids::{ChirpId, UserId};

/// A repository under test, as the object-safe trait object the production transport layer holds
/// (`Arc<dyn ChirpRepository>`). Both backings are exercised through exactly this shared surface, so
/// the concurrency harness is identical for the in-memory and the durable store.
type Repo = Arc<dyn ChirpRepository>;

/// A conservative concurrent fan-out. Kept at or below the durable pool's connection ceiling
/// (`DEFAULT_MAX_CONNECTIONS = 10`) so the suite exercises genuine in-flight concurrency without
/// ever starving the pool and turning a correctness test into a capacity test.
const FANOUT: usize = 8;

// ---------------------------------------------------------------------------------------------
// Shared, backing-agnostic harness. These helpers drive the public trait only (an integration test
// is a separate crate and sees public API exclusively), so they compose identically against the
// in-memory and the durable store.
// ---------------------------------------------------------------------------------------------

/// A process-unique, run-unique seed so handles minted by this binary never collide with a previous
/// run's handles left in a shared durable store.
fn run_seed() -> u64 {
    static SEED: OnceLock<u64> = OnceLock::new();
    *SEED.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
    })
}

/// Lowercase base36 of a `u64` (digits + ASCII letters — all valid [`Username`] characters).
fn base36(mut value: u64) -> String {
    const ALPHABET: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if value == 0 {
        return "0".to_owned();
    }
    let mut bytes = Vec::new();
    while value > 0 {
        bytes.push(ALPHABET[(value % 36) as usize]);
        value /= 36;
    }
    bytes.reverse();
    String::from_utf8(bytes).expect("base36 alphabet is ASCII")
}

/// Mints a fresh, valid, unique handle (`<= 15` chars, `[a-z0-9_]`). Uniqueness is guaranteed within
/// a run by a monotonic counter and across runs by the run seed; a multiplicative mix spreads the
/// values so the truncation to the handle-length ceiling does not collide in practice.
fn unique_username() -> Username {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let mixed = run_seed()
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(n);
    let mut handle = format!("u{}", base36(mixed));
    handle.truncate(15);
    Username::parse(handle).expect("generated handle is well-formed")
}

/// Registers one fresh user against the backing.
async fn make_user(repo: &Repo) -> User {
    let username = unique_username();
    let display_name = format!("test {}", username.as_str());
    repo.create_user(CreateUserRequest {
        username,
        display_name,
        bio: None,
    })
    .await
    .expect("create_user succeeds for a fresh handle")
}

/// Registers `n` fresh users sequentially (deterministic set-up; the concurrency is in the act under
/// test, not the fixture construction).
async fn make_users(repo: &Repo, n: usize) -> Vec<User> {
    let mut users = Vec::with_capacity(n);
    for _ in 0..n {
        users.push(make_user(repo).await);
    }
    users
}

/// Posts one chirp authored by `author`.
async fn make_chirp(repo: &Repo, author: UserId, body: &str) -> Chirp {
    repo.create_chirp(
        author,
        CreateChirpRequest {
            text: ChirpText::parse(body).expect("valid chirp body"),
            reply_to: None,
        },
    )
    .await
    .expect("create_chirp succeeds for an existing author")
}

/// Drives `count` concurrent tasks, each produced by `task(index)`, and returns their outputs once
/// all have joined. Each task owns its captures (typically a cloned [`Repo`] handle), so they run
/// genuinely in flight on the multi-threaded runtime.
async fn run_concurrent<F, Fut, T>(count: usize, task: F) -> Vec<T>
where
    F: Fn(usize) -> Fut,
    Fut: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let mut set = tokio::task::JoinSet::new();
    for index in 0..count {
        set.spawn(task(index));
    }
    let mut outputs = Vec::with_capacity(count);
    while let Some(joined) = set.join_next().await {
        outputs.push(joined.expect("a concurrent task panicked"));
    }
    outputs
}

/// Pages `viewer`'s whole timeline with a small page size, following the opaque cursor to the end,
/// and returns the chirp ids in the order seen. The iteration is bounded so a paging bug that never
/// terminates (a stuck cursor) fails as a bounded wrong answer rather than hanging the suite.
async fn drain_timeline(repo: &Repo, viewer: UserId, page_limit: u32) -> Vec<ChirpId> {
    let mut ids = Vec::new();
    let mut cursor = None;
    for _ in 0..1_000 {
        let page = repo
            .timeline(
                viewer,
                TimelineQuery {
                    limit: Some(page_limit),
                    cursor: cursor.clone(),
                },
            )
            .await
            .expect("timeline page read succeeds");
        ids.extend(page.items.iter().map(|chirp| chirp.id));
        match page.next_cursor {
            Some(next) => cursor = Some(next),
            None => return ids,
        }
    }
    ids
}

// ---------------------------------------------------------------------------------------------
// Store-free half — always compiled; runs on a plain `cargo test` with no database and no
// `postgres` feature. Positive evidence that the retained in-memory implementor still backs the
// test suite store-free, including under concurrency.
// ---------------------------------------------------------------------------------------------

mod store_free {
    use super::*;

    /// `cargo test` stays runnable store-free against the retained [`InMemoryRepository`]: a
    /// concurrent follow/like fan-out against the in-memory backing keeps the denormalized counts
    /// equal to their edge sets, with no database involved.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn cargo_test_runs_store_free_against_the_in_memory_implementor() {
        let repo: Repo = Arc::new(InMemoryRepository::new());

        let followee = make_user(&repo).await;
        let followers = make_users(&repo, FANOUT).await;
        let author = make_user(&repo).await;
        let chirp = make_chirp(&repo, author.id, "store-free concurrency").await;

        // Concurrent distinct followers + concurrent distinct likers, all against the in-memory store.
        let followee_id = followee.id;
        let follower_ids: Vec<UserId> = followers.iter().map(|u| u.id).collect();
        let follows = run_concurrent(FANOUT, |i| {
            let repo = repo.clone();
            let follower = follower_ids[i];
            async move { repo.follow(follower, followee_id).await }
        })
        .await;
        for result in &follows {
            assert!(result.is_ok(), "a concurrent follow failed: {result:?}");
        }

        let chirp_id = chirp.id;
        let likes = run_concurrent(FANOUT, |i| {
            let repo = repo.clone();
            let user = follower_ids[i];
            async move { repo.like(user, chirp_id).await }
        })
        .await;
        for result in &likes {
            assert!(result.is_ok(), "a concurrent like failed: {result:?}");
        }

        // counts_match_edges on the store-free backing.
        let followee_after = repo.get_user(followee_id).await.expect("followee readable");
        assert_eq!(
            followee_after.follower_count, FANOUT as u64,
            "in-memory follower_count must equal the concurrent follower set"
        );
        let liked = drain_timeline(&repo, author.id, 50)
            .await
            .into_iter()
            .collect::<HashSet<_>>();
        assert!(
            liked.contains(&chirp_id),
            "author's chirp is on their timeline"
        );
        let page = repo
            .timeline(author.id, TimelineQuery::default())
            .await
            .expect("author timeline");
        let stored = page
            .items
            .iter()
            .find(|c| c.id == chirp_id)
            .expect("the chirp is present");
        assert_eq!(
            stored.like_count, FANOUT as u64,
            "in-memory like_count must equal the concurrent liker set"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Durable half — only compiled under `--features postgres`, and each test skips unless
// `CHIRP_TEST_DATABASE_URL` names a reachable test database. These assert the identities under
// concurrent access against the durable backing.
// ---------------------------------------------------------------------------------------------

#[cfg(feature = "postgres")]
mod durable {
    use super::*;

    use chirp_server::PostgresRepository;
    use chirp_types::api::LoginRequest;

    /// The durable test store URL, or `None` when unset/blank (the test then skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// Connects (and migrates) a fresh durable repository as the shared trait object. A new call is
    /// a new connection pool — the proxy this suite uses for "a different pod / after restart".
    async fn connect(url: &str) -> Repo {
        Arc::new(
            PostgresRepository::connect(url)
                .await
                .expect("connect + migrate the durable test store"),
        )
    }

    /// Skips the calling test with a note when no durable store is configured; otherwise yields the
    /// URL. Keeps `cargo test --features postgres` green without a database.
    macro_rules! store_or_skip {
        () => {
            match test_database_url() {
                Some(url) => url,
                None => {
                    eprintln!("skipping: CHIRP_TEST_DATABASE_URL is not set");
                    return;
                }
            }
        };
    }

    // ---- counts_match_edges / multiwrite_atomic -------------------------------------------------

    /// `counts_match_edges` + `multiwrite_atomic`: when distinct followers follow one account
    /// concurrently, the account's denormalized `follower_count` ends equal to the actual durable
    /// follow-edge set (no lost increment), and each follower's `following_count` is exactly one.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn follower_count_matches_edge_set_under_concurrent_distinct_followers() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        let followee = make_user(&repo).await;
        let followers = make_users(&repo, FANOUT).await;
        let followee_id = followee.id;
        let follower_ids: Vec<UserId> = followers.iter().map(|u| u.id).collect();

        let results = run_concurrent(FANOUT, |i| {
            let repo = repo.clone();
            let follower = follower_ids[i];
            async move { repo.follow(follower, followee_id).await }
        })
        .await;
        for result in &results {
            let response = result.as_ref().expect("a concurrent follow failed");
            assert!(response.following, "follow must report the edge as present");
        }

        // The stored denormalized count equals the concurrent follower set...
        let followee_after = repo.get_user(followee_id).await.expect("followee readable");
        assert_eq!(
            followee_after.follower_count, FANOUT as u64,
            "follower_count lost a concurrent increment (count != edges)"
        );
        // ...and equals the actual durable edge set read back through the public listing API.
        let edge_count = count_all(&repo, |q| {
            let repo = repo.clone();
            async move { repo.list_followers(followee_id, q).await }
        })
        .await;
        assert_eq!(
            edge_count, FANOUT,
            "the follower edge set does not match the denormalized follower_count"
        );
        for follower in &follower_ids {
            let after = repo.get_user(*follower).await.expect("follower readable");
            assert_eq!(
                after.following_count, 1,
                "each concurrent follower must end following exactly one account"
            );
        }
    }

    /// `counts_match_edges` + `multiwrite_atomic`: distinct users liking one chirp concurrently drive
    /// the chirp's denormalized `like_count` to exactly the number of likers (no lost increment).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn like_count_matches_edge_set_under_concurrent_distinct_likers() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        let author = make_user(&repo).await;
        let chirp = make_chirp(&repo, author.id, "liked concurrently").await;
        let likers = make_users(&repo, FANOUT).await;
        let chirp_id = chirp.id;
        let liker_ids: Vec<UserId> = likers.iter().map(|u| u.id).collect();

        let results = run_concurrent(FANOUT, |i| {
            let repo = repo.clone();
            let user = liker_ids[i];
            async move { repo.like(user, chirp_id).await }
        })
        .await;
        for result in &results {
            let response = result.as_ref().expect("a concurrent like failed");
            assert!(response.liked, "like must report the edge as present");
        }

        let stored = find_chirp(&repo, author.id, chirp_id).await;
        assert_eq!(
            stored.like_count, FANOUT as u64,
            "like_count lost a concurrent increment (count != edges)"
        );
    }

    /// `multiwrite_atomic`: the same follower following one account concurrently and repeatedly is
    /// idempotent — exactly one edge and a `follower_count` of one, never a double count.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_duplicate_follow_by_one_follower_counts_exactly_once() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        let follower = make_user(&repo).await;
        let followee = make_user(&repo).await;
        let (follower_id, followee_id) = (follower.id, followee.id);

        let results = run_concurrent(FANOUT, |_| {
            let repo = repo.clone();
            async move { repo.follow(follower_id, followee_id).await }
        })
        .await;
        for result in &results {
            assert!(
                result.is_ok(),
                "a concurrent duplicate follow failed: {result:?}"
            );
        }

        let followee_after = repo.get_user(followee_id).await.expect("followee readable");
        assert_eq!(
            followee_after.follower_count, 1,
            "concurrent duplicate follows double-counted (not idempotent)"
        );
        let edge_count = count_all(&repo, |q| {
            let repo = repo.clone();
            async move { repo.list_followers(followee_id, q).await }
        })
        .await;
        assert_eq!(edge_count, 1, "a duplicate follow created a second edge");
    }

    /// `counts_match_edges` + `multiwrite_atomic`: a concurrent like-then-unlike churn by distinct
    /// users nets to the empty edge set, leaving `like_count` at zero (and the schema's non-negative
    /// check means it was never driven below the edge set).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrent_like_then_unlike_settles_to_the_edge_set() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        let author = make_user(&repo).await;
        let chirp = make_chirp(&repo, author.id, "liked then unliked").await;
        let users = make_users(&repo, FANOUT).await;
        let chirp_id = chirp.id;
        let user_ids: Vec<UserId> = users.iter().map(|u| u.id).collect();

        let results = run_concurrent(FANOUT, |i| {
            let repo = repo.clone();
            let user = user_ids[i];
            async move {
                repo.like(user, chirp_id).await?;
                repo.unlike(user, chirp_id).await
            }
        })
        .await;
        for result in &results {
            assert!(
                result.is_ok(),
                "a concurrent like/unlike failed: {result:?}"
            );
        }

        let stored = find_chirp(&repo, author.id, chirp_id).await;
        assert_eq!(
            stored.like_count, 0,
            "like_count did not settle to the empty edge set after a like/unlike churn"
        );
    }

    /// `multiwrite_atomic` + `counts_match_edges` + `total_state_coverage`: deleting a subset of an
    /// account's followers concurrently cascades their follow and like edges and atomically
    /// reconciles the survivors' denormalized counts, so after the cascade every count still equals
    /// its durable edge set.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn delete_account_cascade_reconciles_counts_atomically_under_concurrency() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        let followee = make_user(&repo).await;
        let chirp = make_chirp(&repo, followee.id, "followee's chirp").await;
        let followers = make_users(&repo, FANOUT).await;
        let followee_id = followee.id;
        let chirp_id = chirp.id;
        let follower_ids: Vec<UserId> = followers.iter().map(|u| u.id).collect();

        // Every follower follows the account and likes its chirp (sequential fixture).
        for follower in &follower_ids {
            repo.follow(*follower, followee_id).await.expect("follow");
            repo.like(*follower, chirp_id).await.expect("like");
        }
        assert_eq!(
            repo.get_user(followee_id)
                .await
                .expect("followee")
                .follower_count,
            FANOUT as u64
        );
        assert_eq!(
            find_chirp(&repo, followee_id, chirp_id).await.like_count,
            FANOUT as u64
        );

        // Concurrently delete half of the followers.
        let to_delete = FANOUT / 2;
        let deleted: Vec<UserId> = follower_ids.iter().take(to_delete).copied().collect();
        let results = run_concurrent(to_delete, |i| {
            let repo = repo.clone();
            let victim = deleted[i];
            async move { repo.delete_account(victim).await }
        })
        .await;
        for result in &results {
            assert!(
                result.is_ok(),
                "a concurrent delete_account failed: {result:?}"
            );
        }

        let survivors = FANOUT - to_delete;
        // The cascade removed the deleted accounts entirely...
        for victim in &deleted {
            let missing = repo.get_user(*victim).await;
            assert!(
                missing.is_err(),
                "a deleted account is still readable: {victim}"
            );
        }
        // ...and both denormalized counts were atomically reconciled to the survivor edge set.
        let followee_after = repo.get_user(followee_id).await.expect("followee readable");
        assert_eq!(
            followee_after.follower_count, survivors as u64,
            "follower_count was not atomically reconciled to the survivor edge set"
        );
        let edge_count = count_all(&repo, |q| {
            let repo = repo.clone();
            async move { repo.list_followers(followee_id, q).await }
        })
        .await;
        assert_eq!(
            edge_count, survivors,
            "the follower edge set does not match the reconciled follower_count after cascade"
        );
        assert_eq!(
            find_chirp(&repo, followee_id, chirp_id).await.like_count,
            survivors as u64,
            "like_count was not atomically reconciled after the like edges cascaded"
        );
    }

    // ---- ids_opaque_monotonic_stable ------------------------------------------------------------

    /// `ids_opaque_monotonic_stable`: ids allocated under concurrent creation are all distinct and
    /// never reused, and they remain stable when read back through a fresh connection pool (the
    /// across-restart proxy).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn concurrently_allocated_ids_are_unique_and_stable_across_a_fresh_connection() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        // Concurrent user creation: every id must be distinct (opaque, server-assigned, never reused).
        let users = run_concurrent(FANOUT, |_| {
            let repo = repo.clone();
            async move { make_user(&repo).await }
        })
        .await;
        let user_ids: Vec<UserId> = users.iter().map(|u| u.id).collect();
        let distinct: HashSet<UserId> = user_ids.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            user_ids.len(),
            "concurrent user id allocation reused an id: {user_ids:?}"
        );

        // Concurrent chirp creation by one author: chirp ids must likewise be distinct.
        let author = make_user(&repo).await;
        let author_id = author.id;
        let chirps = run_concurrent(FANOUT, |i| {
            let repo = repo.clone();
            async move { make_chirp(&repo, author_id, &format!("chirp {i}")).await }
        })
        .await;
        let chirp_ids: HashSet<ChirpId> = chirps.iter().map(|c| c.id).collect();
        assert_eq!(
            chirp_ids.len(),
            FANOUT,
            "concurrent chirp id allocation reused an id"
        );

        // Stable across a fresh connection pool: the same ids resolve to the same records.
        let reconnected = connect(&url).await;
        for user in &users {
            let again = reconnected
                .get_user(user.id)
                .await
                .expect("id stable across reconnect");
            assert_eq!(
                again.id, user.id,
                "a user id was not stable across reconnect"
            );
            assert_eq!(
                again.username, user.username,
                "the record behind the id changed"
            );
        }
    }

    /// `ids_opaque_monotonic_stable` (exactly-once paging) + `domain_boundary_validated`
    /// (reverse-chronological ordering): concurrent readers paging the same timeline each see every
    /// chirp exactly once, in strictly reverse-chronological id order — no skips, no duplicates.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn timeline_cursor_paging_is_exactly_once_under_concurrent_readers() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        let author = make_user(&repo).await;
        let author_id = author.id;
        const CHIRPS: usize = 25;
        let mut expected = HashSet::with_capacity(CHIRPS);
        for i in 0..CHIRPS {
            let chirp = make_chirp(&repo, author_id, &format!("timeline chirp {i}")).await;
            expected.insert(chirp.id);
        }

        const READERS: usize = 6;
        let pages = run_concurrent(READERS, |_| {
            let repo = repo.clone();
            async move { drain_timeline(&repo, author_id, 4).await }
        })
        .await;

        assert_eq!(pages.len(), READERS);
        for ids in &pages {
            assert_eq!(
                ids.len(),
                CHIRPS,
                "a concurrent reader saw a wrong number of chirps (skip or duplicate)"
            );
            assert_exactly_once_descending(ids, &expected);
        }
    }

    // ---- confirmed_write_durable / total_state_coverage -----------------------------------------

    /// `confirmed_write_durable` + `total_state_coverage`: every concurrent create that returned
    /// success is durable — each created record is readable through a *fresh* connection pool (a
    /// different pod / after restart), so the write was durable before success was returned.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn confirmed_concurrent_writes_are_durable_across_a_fresh_connection() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        let users = run_concurrent(FANOUT, |_| {
            let repo = repo.clone();
            async move { make_user(&repo).await }
        })
        .await;

        // A different connection pool than the one that confirmed the writes.
        let reconnected = connect(&url).await;
        for user in &users {
            let durable = reconnected
                .get_user(user.id)
                .await
                .expect("a confirmed create is durable and visible from a fresh connection");
            assert_eq!(
                durable.username, user.username,
                "the durable record diverged from the confirmation"
            );
        }
    }

    /// `total_state_coverage` + `confirmed_write_durable`: a concurrent cohort writes across every
    /// durable state class — accounts, credentials, sessions, chirps, and follow edges — and every
    /// class survives to a *fresh* connection pool, including sessions and credentials that were
    /// in-process-only before this work. The in-memory repository is no longer the production backing.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn total_state_is_durable_and_pod_independent_under_concurrency() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        let anchor = make_user(&repo).await;
        let anchor_id = anchor.id;

        // Each concurrent member: create account, set a credential, open a session, post a chirp,
        // and follow the anchor — exercising every durable state class at once.
        struct Member {
            user: UserId,
            username: Username,
            secret: String,
            token: chirp_types::api::SessionToken,
            chirp: ChirpId,
        }
        let members = run_concurrent(FANOUT, |i| {
            let repo = repo.clone();
            async move {
                let user = make_user(&repo).await;
                let secret = format!("secret-{i}-{}", user.id.get());
                repo.set_credential(user.id, &secret)
                    .await
                    .expect("set_credential");
                let auth = repo
                    .login(LoginRequest {
                        username: user.username.clone(),
                    })
                    .await
                    .expect("login issues a session");
                let chirp = make_chirp(&repo, user.id, &format!("member {i}")).await;
                repo.follow(user.id, anchor_id)
                    .await
                    .expect("follow the anchor");
                Member {
                    user: user.id,
                    username: user.username,
                    secret,
                    token: auth.token,
                    chirp: chirp.id,
                }
            }
        })
        .await;

        // A fresh pool: pod-independent, nothing left in process memory.
        let reconnected = connect(&url).await;
        for member in &members {
            // Account state.
            let user = reconnected
                .get_user(member.user)
                .await
                .expect("account durable");
            assert_eq!(user.username, member.username);
            // Session state (previously in-process-only) is durable: the token still resolves.
            let resolved = reconnected
                .resolve_token(&member.token)
                .await
                .expect("the session is durable and resolves from a fresh connection");
            assert_eq!(
                resolved, member.user,
                "the session resolved to the wrong user"
            );
            // Credential state (previously in-process-only) is durable: the secret still verifies.
            let verified = reconnected
                .verify_credential(&member.username, &member.secret)
                .await
                .expect("the credential is durable and verifies from a fresh connection");
            assert_eq!(
                verified, member.user,
                "the credential verified to the wrong user"
            );
            // Content + edge state: the chirp is on the member's timeline, the follow edge persisted.
            let timeline = drain_timeline(&reconnected, member.user, 50).await;
            assert!(
                timeline.contains(&member.chirp),
                "the member's chirp is not durably on their timeline"
            );
        }
        let anchor_after = reconnected
            .get_user(anchor_id)
            .await
            .expect("anchor durable");
        assert_eq!(
            anchor_after.follower_count, FANOUT as u64,
            "the concurrently created follow edges are not durably reflected in the count"
        );
    }

    // ---- domain_boundary_validated --------------------------------------------------------------

    /// `domain_boundary_validated`: the entities' durable-boundary invariants hold under concurrency
    /// — a self-follow is refused and never creates an edge, and at most one like exists per
    /// `(user, chirp)` even when the same user likes concurrently. (An invalid `Username`/`ChirpText`
    /// is unrepresentable in the type system, so it cannot reach the store at all.)
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn entity_invariants_hold_at_the_durable_boundary_under_concurrency() {
        let url = store_or_skip!();
        let repo = connect(&url).await;

        // No self-follow: concurrent self-follow attempts are all refused and create no edge.
        let loner = make_user(&repo).await;
        let loner_id = loner.id;
        let self_follows = run_concurrent(FANOUT, |_| {
            let repo = repo.clone();
            async move { repo.follow(loner_id, loner_id).await }
        })
        .await;
        for result in &self_follows {
            assert!(
                result.is_err(),
                "a self-follow was accepted at the durable boundary"
            );
        }
        let after = repo.get_user(loner_id).await.expect("user readable");
        assert_eq!(
            after.follower_count, 0,
            "a refused self-follow still moved a count"
        );
        assert_eq!(
            after.following_count, 0,
            "a refused self-follow still moved a count"
        );

        // At most one like per (user, chirp): the same user liking concurrently yields count one.
        let author = make_user(&repo).await;
        let chirp = make_chirp(&repo, author.id, "one like per user").await;
        let liker = make_user(&repo).await;
        let (chirp_id, liker_id) = (chirp.id, liker.id);
        let likes = run_concurrent(FANOUT, |_| {
            let repo = repo.clone();
            async move { repo.like(liker_id, chirp_id).await }
        })
        .await;
        for result in &likes {
            assert!(
                result.is_ok(),
                "a concurrent repeat like failed: {result:?}"
            );
        }
        let stored = find_chirp(&repo, author.id, chirp_id).await;
        assert_eq!(
            stored.like_count, 1,
            "the same user's concurrent likes violated the at-most-one-like invariant"
        );
    }

    // ---- durable-half helpers -------------------------------------------------------------------

    /// Asserts `ids` is exactly `expected` seen once each (no skips, no duplicates) and in strictly
    /// descending id order (the reverse-chronological timeline order).
    fn assert_exactly_once_descending(ids: &[ChirpId], expected: &HashSet<ChirpId>) {
        let seen: HashSet<ChirpId> = ids.iter().copied().collect();
        assert_eq!(
            seen.len(),
            ids.len(),
            "paging returned a duplicate chirp id (not exactly-once): {ids:?}"
        );
        assert_eq!(
            &seen, expected,
            "paging did not return exactly the expected chirp set (a skip or an extra): {ids:?}"
        );
        assert!(
            ids.windows(2).all(|w| w[0].get() > w[1].get()),
            "timeline is not strictly reverse-chronological by id: {ids:?}"
        );
    }

    /// Reads `chirp` back as the authoritative durable record via the author's timeline.
    async fn find_chirp(repo: &Repo, author: UserId, chirp: ChirpId) -> Chirp {
        let page = repo
            .timeline(
                author,
                TimelineQuery {
                    limit: Some(100),
                    cursor: None,
                },
            )
            .await
            .expect("author timeline read");
        page.items
            .into_iter()
            .find(|c| c.id == chirp)
            .expect("the chirp is durably present on the author's timeline")
    }

    /// Counts every item of a cursor-paged `User` listing, following the cursor to the end.
    async fn count_all<F, Fut>(_repo: &Repo, page: F) -> usize
    where
        F: Fn(TimelineQuery) -> Fut,
        Fut: std::future::Future<
            Output = Result<chirp_types::api::Page<User>, chirp_server::ServerError>,
        >,
    {
        let mut total = 0;
        let mut cursor = None;
        for _ in 0..1_000 {
            let result = page(TimelineQuery {
                limit: Some(100),
                cursor: cursor.clone(),
            })
            .await
            .expect("listing page read");
            total += result.items.len();
            match result.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        total
    }
}
