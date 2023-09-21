use sqlx::SqlitePool;
use crate::db::EnvironmentRow;

use std::fs;
use std::io;
use std::io::{Error, ErrorKind, Write};

pub async fn export_dotenv(db: &SqlitePool, env: &str, buf: &mut io::BufWriter<fs::File>) -> io::Result<()> {
    let envs = sqlx::query_as::<_, EnvironmentRow>(
        r"SELECT env, key, value, created_at
        FROM environments
        WHERE env = ?
        GROUP BY env, key
        HAVING MAX(created_at)
        ORDER BY env, key"
    )
    .bind(env)
    .fetch_all(db)
    .await
    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    for env in envs {
        writeln!(buf, "{}={}", &env.key, &env.value)?;
    }

    Ok(())
}
