use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Environment {
    pub env: String,
}

#[cfg(test)]
impl Environment {
    pub(crate) fn from(e: &str) -> Self {
        Self { env: e.to_owned() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct EnvironmentRow {
    pub env: String,
    pub key: String,
    pub value: String,
}

#[cfg(test)]
impl EnvironmentRow {
    pub(crate) fn from(e: &str, k: &str, v: &str) -> Self {
        Self {
            env: e.to_owned(),
            key: k.to_owned(),
            value: v.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct EnvironmentRowNullable {
    pub env: String,
    pub key: String,
    pub value: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
impl EnvironmentRowNullable {
    pub(crate) fn from(e: &str, k: &str, v: Option<&str>, date: &str) -> Self {
        Self {
            env: e.to_owned(),
            key: k.to_owned(),
            value: v.map(|x| x.to_owned()),
            created_at: date.to_owned(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EnvironmentDiff {
    InFirst(String, String),
    InSecond(String, String),
    Different(String, String, String),
}

impl FromRow<'_, SqliteRow> for EnvironmentDiff {
    fn from_row(row: &SqliteRow) -> sqlx::Result<Self> {
        let (key, value) = (
            row.try_get::<String, _>("key")?,
            row.try_get::<String, _>("value")?,
        );

        let val = match row.try_get::<String, _>("type")?.as_str() {
            "+" => Self::InFirst(key, value),
            "-" => Self::InSecond(key, value),
            "/" => Self::Different(key, value, row.try_get("diff")?),
            _ => panic!("unknown value"),
        };

        Ok(val)
    }
}
