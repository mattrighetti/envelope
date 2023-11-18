use std::io::{BufRead, BufReader, Result, Write};

use crate::{
    db::{EnvelopeDb, EnvironmentRow, Truncate},
    editor,
};

pub struct EditorData {
    delete: Vec<String>,
    upsert: Vec<(String, String)>,
}

fn parse(bufr: BufReader<&[u8]>) -> EditorData {
    let (mut delete, mut upsert) = (Vec::new(), Vec::new());

    for kv in bufr.lines() {
        if kv.is_err() {
            continue;
        }

        let kv = kv.unwrap();

        match kv.starts_with("#") {
            true => {
                if let Some((k, _)) = kv.split_once("=") {
                    delete.push(k[1..k.len()].trim().into());
                }
            }
            false => {
                if let Some((k, v)) = kv.split_once("=") {
                    upsert.push((k.trim().into(), v.trim().into()));
                }
            }
        }
    }

    EditorData { delete, upsert }
}

pub async fn edit(db: &EnvelopeDb, env: &str) -> Result<()> {
    let mut kv_list = Vec::new();
    let envs: Vec<EnvironmentRow> = db.list_all_var_in_env(env, Truncate::None).await?;
    for env in envs {
        writeln!(&mut kv_list, "{}={}", &env.key, &env.value)?;
    }

    let bytes = editor::spawn_with(&kv_list)?;
    let EditorData { delete, upsert } = parse(BufReader::new(&bytes));

    for k in delete {
        db.delete_var_for_env(env, &k).await?;
    }

    for (k, v) in upsert {
        db.insert(env, &k, &v).await?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_upsert() {
        let bytes = b"key1=value1\nkey2=value2\n\nkey3=value3\n\n\nk4=v4";
        let EditorData { delete, upsert } = parse(BufReader::new(bytes));
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
        let EditorData { delete, upsert } = parse(BufReader::new(bytes));
        assert_eq!(3, delete.len());
        for (i, k) in vec!["key1", "key2", "key3"].iter().enumerate() {
            assert_eq!(&delete[i], k);
        }
        assert!(upsert.is_empty());
    }

    #[test]
    fn test_upsert_delete() {
        let bytes = b"#key1=value1\n  key2=value2\n\n\n#key3=value3";
        let EditorData { delete, upsert } = parse(BufReader::new(bytes));
        assert_eq!(2, delete.len());
        assert_eq!(1, upsert.len());
        assert_eq!("key1", delete[0]);
        assert_eq!("key3", delete[1]);
        assert_eq!("key2", upsert[0].0);
        assert_eq!("value2", upsert[0].1);
    }
}
