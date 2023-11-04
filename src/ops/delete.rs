use std::io::Result;

use crate::db::EnvelopeDb;

/// Deletes every key found in an enviroment
pub async fn delete_env(db: &EnvelopeDb, env: &str) -> Result<()> {
    db.delete_env(env).await
}

/// Deletes a key for every environments
pub async fn delete_var_globally(db: &EnvelopeDb, key: &str) -> Result<()> {
    db.delete_var_all(key).await
}

/// Deletes a key in a specific env
pub async fn delete_var_in_env(db: &EnvelopeDb, env: &str, key: &str) -> Result<()> {
    db.delete_var_for_env(env, key).await
}
