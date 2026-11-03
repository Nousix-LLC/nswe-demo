-- Migration 0003: invalidate any plaintext credentials written before the argon2id KDF was
-- introduced (by this migration\s parent commit).  The credentials.secret column stores
-- PHC-format argon2id hashes after this change; any row written by an earlier build holds a
-- plaintext string that PasswordHash::new() will reject, causing every verify to surface as
-- ServerError::Internal (corrupt stored credential) rather than Unauthorized.
--
-- Clearing the table is the safe, self-consistent choice: affected users will need to reset
-- their credential via account-recovery or a re-registration flow.  A pre-production database
-- has no users whose credentials must be preserved, so the cost is zero.
DELETE FROM credentials;
