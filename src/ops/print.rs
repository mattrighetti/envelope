use sqlx::SqlitePool;
use crate::db::EnvironmentRow;

use std::io;
use std::io::{Error, ErrorKind};

fn red(s: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", s)
}

fn blue(s: &str) -> String {
    format!("\x1b[34m{}\x1b[0m", s)
}

pub async fn print(pool: &SqlitePool, env: &str) -> io::Result<()> {
    let envs = sqlx::query_as::<_, EnvironmentRow>("SELECT * FROM environments WHERE env = ?")
        .bind(env)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    for env in envs {
        println!("{}  {}", red(&env.key), blue(&env.value));
    }

    Ok(())
}
