use anyhow::Result;
use clap::Parser;

use crate::db::{self, EnvelopeDb};
use crate::ops;

/// Valid sorting types
#[derive(Debug, Clone, clap::ValueEnum)]
enum Sort {
    #[clap(alias = "k")]
    Key,
    #[clap(alias = "v")]
    Value,
    #[clap(alias = "d")]
    Date,
    #[clap(alias = "kd")]
    KeyDesc,
    #[clap(alias = "vd")]
    ValueDesc,
    #[clap(alias = "dd")]
    DateDesc,
}

impl Sort {
    fn to_str(&self) -> &str {
        match self {
            Self::Date => "d",
            Self::DateDesc => "dd",
            Self::Key => "k",
            Self::KeyDesc => "kd",
            Self::Value => "v",
            Self::ValueDesc => "vd",
        }
    }
}

/// List saved environments and/or their variables
#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to list.
    /// If not provided, all environments will be listed.
    env: Option<String>,

    #[arg(long, short)]
    pretty_print: bool,

    /// Output variables for a shell (default: kv)
    #[arg(long, requires = "env", conflicts_with = "pretty_print")]
    shell: Option<ops::RawOutputFormat>,

    #[arg(long, short)]
    truncate: bool,

    /// How envelope should sort result
    #[arg(long, short, default_value = "d")]
    sort: Sort,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        match &self.env {
            None => ops::list_envs(&mut std::io::stdout(), db).await?,
            Some(env) => {
                if self.pretty_print {
                    let truncate = if self.truncate {
                        db::Truncate::Max(60)
                    } else {
                        db::Truncate::None
                    };
                    ops::table_list(db, env, truncate, self.sort.to_str()).await?;
                } else {
                    ops::list_raw(
                        &mut std::io::stdout(),
                        db,
                        env,
                        self.sort.to_str(),
                        self.shell.unwrap_or_default(),
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_aliases_parse() {
        for (alias, expected) in [
            ("bash", ops::RawOutputFormat::Sh),
            ("zsh", ops::RawOutputFormat::Sh),
            ("nushell", ops::RawOutputFormat::Nu),
            ("pwsh", ops::RawOutputFormat::Powershell),
        ] {
            let cmd = Cmd::try_parse_from(["list", "dev", "--shell", alias]).unwrap();
            assert_eq!(cmd.shell, Some(expected));
        }
    }

    #[test]
    fn shell_requires_environment_and_conflicts_with_table_output() {
        for args in [
            vec!["list", "--shell", "sh"],
            vec!["list", "dev", "--shell", "sh", "--pretty-print"],
            vec!["list", "dev", "--shell", "unknown"],
            vec!["list", "dev", "--shell", "cmd"],
        ] {
            assert!(Cmd::try_parse_from(args).is_err());
        }
    }
}
