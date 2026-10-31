use anyhow::Result;

use crate::db::EnvelopeDb;

/// Deletes every key found in an environment.
pub async fn delete_env(db: &EnvelopeDb, env: &str) -> Result<()> {
    db.soft_delete_env(env).await
}

/// Deletes a key from every environment.
pub async fn delete_var_globally(db: &EnvelopeDb, key: &str) -> Result<()> {
    db.soft_delete_keys(key).await
}

/// Deletes a key in a specific environment.
pub async fn delete_var_in_env(db: &EnvelopeDb, env: &str, key: &str) -> Result<()> {
    db.soft_delete_key_in_env(env, key).await
}
