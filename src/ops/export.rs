use crate::db::EnvelopeDb;

use std::fs;
use std::io;
use std::io::{Result, Write};

pub async fn export_dotenv(
    db: &EnvelopeDb,
    env: &str,
    buf: &mut io::BufWriter<fs::File>,
) -> Result<()> {
    for env in db.list_var_in_env(env).await? {
        writeln!(buf, "{}={}", &env.key, &env.value)?;
    }

    Ok(())
}
