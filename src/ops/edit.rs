use std::io::{BufRead, BufReader, Write};

use anyhow::{Context, Result};

use crate::db::model::EnvironmentRow;
use crate::db::{EnvelopeDb, Truncate};
use crate::editor;

#[derive(Debug)]
pub struct EditorData {
    delete: Vec<String>,
    upsert: Vec<(String, String)>,
}

fn parse(bufr: BufReader<&[u8]>) -> Result<EditorData> {
    let (mut delete, mut upsert) = (Vec::new(), Vec::new());

    for kv in bufr.lines() {
        let kv = kv.context("failed to read edited environment")?;

        if let Some(comment) = kv.strip_prefix('#') {
            if let Some((key, _)) = comment.split_once('=') {
                delete.push(key.trim().into());
            }
        } else if let Some((key, value)) = kv.split_once('=') {
            upsert.push((key.trim().into(), value.trim().into()));
        }
    }

    Ok(EditorData { delete, upsert })
}

pub async fn edit(db: &EnvelopeDb, env: &str) -> Result<()> {
    let mut kv_list = Vec::new();
    let envs: Vec<EnvironmentRow> = db.list_kv_in_env_alt(env, Truncate::None, "da").await?;
    for env in envs {
        writeln!(&mut kv_list, "{}={}", env.key, env.value)?;
    }

    let bytes = editor::spawn_with(&kv_list)?;
    let EditorData { delete, upsert } = parse(BufReader::new(&bytes))?;

    db.apply_batch(env, &delete, &upsert).await
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_upsert() {
        let bytes = b"key1=value1\nkey2=value2\n\nkey3=value3\n\n\nk4=v4";
        let EditorData { delete, upsert } = parse(BufReader::new(bytes)).unwrap();
        assert_eq!(4, upsert.len());
        for (i, kv) in vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")]
            .into_iter()
            .enumerate()
        {
            assert_eq!((upsert[i].0.as_str(), upsert[i].1.as_str()), kv);
        }
        assert!(delete.is_empty());
    }

    #[test]
    fn test_delete() {
        let bytes = b"#key1=value1\n#  key2=value2\n\n\n#key3=value3";
        let EditorData { delete, upsert } = parse(BufReader::new(bytes)).unwrap();
        assert_eq!(3, delete.len());
        for (i, k) in ["key1", "key2", "key3"].iter().enumerate() {
            assert_eq!(&delete[i], k);
        }
        assert!(upsert.is_empty());
    }

    #[test]
    fn test_upsert_delete() {
        let bytes = b"#key1=value1\n  key2=value2\n\n\n#key3=value3";
        let EditorData { delete, upsert } = parse(BufReader::new(bytes)).unwrap();
        assert_eq!(2, delete.len());
        assert_eq!(1, upsert.len());
        assert_eq!("key1", delete[0]);
        assert_eq!("key3", delete[1]);
        assert_eq!("key2", upsert[0].0);
        assert_eq!("value2", upsert[0].1);
    }

    #[test]
    fn test_parse_propagates_input_errors() {
        let bytes = b"KEY=value\n\xffinvalid";

        let error = parse(BufReader::new(bytes)).expect_err("invalid UTF-8 input should fail");

        assert!(
            error
                .to_string()
                .contains("failed to read edited environment")
        );
    }
}
