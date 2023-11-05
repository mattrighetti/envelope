use std::{
    collections::HashSet,
    io::{BufRead, Result},
};

use crate::{
    db::{EnvelopeDb, Truncate},
    editor, other_err,
};

pub async fn edit(db: &EnvelopeDb, env: &str) -> Result<()> {
    let kvs_hs: HashSet<String> = db
        .list_all_var_in_env(env, Truncate::None)
        .await?
        .iter_mut()
        .map(|x| x.kv_string())
        .collect();

    let data = kvs_hs.clone().into_iter().collect::<Vec<_>>().join("\n");
    let status = editor::spawn_with(data.as_bytes());
    if let Err(e) = status {
        return Err(other_err!("error running child process: {}", e));
    }

    for kv in status.unwrap().lines() {
        if kv.is_err() {
            continue;
        }

        let kv = kv.unwrap();
        if kv.starts_with('#') {
            // all the values marked with # were previously set
            // variables and must be set to null, which is done
            // by the delete operation
            if let Some((k, _)) = kv.split_once("=") {
                db.delete_var_for_env(env, k.trim()).await?;
            }

            continue;
        }

        match (kvs_hs.contains(&kv), kv.split_once("=")) {
            // only insert values that changed
            (false, Some((k, v))) => db.insert(env, k, v).await?,
            _ => {
                dbg!(kv);
                continue;
            }
        }
    }

    Ok(())
}
