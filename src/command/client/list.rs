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

/// Valid shell output formats for raw key/value listing.
#[derive(Debug, Clone, clap::ValueEnum)]
enum Shell {
    Kv,
    #[clap(alias = "bash")]
    #[clap(alias = "zsh")]
    Sh,
    Fish,
    #[clap(alias = "nushell")]
    Nu,
    Cmd,
    #[clap(alias = "pwsh")]
    Powershell,
}

impl Shell {
    fn to_output_format(&self) -> ops::RawOutputFormat {
        match self {
            Self::Kv => ops::RawOutputFormat::Kv,
            Self::Sh => ops::RawOutputFormat::Sh,
            Self::Fish => ops::RawOutputFormat::Fish,
            Self::Nu => ops::RawOutputFormat::Nu,
            Self::Cmd => ops::RawOutputFormat::Cmd,
            Self::Powershell => ops::RawOutputFormat::PowerShell,
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

    /// Output variables for the specified shell/format (default: kv).
    #[arg(long, requires = "env", conflicts_with = "pretty_print")]
    shell: Option<Shell>,

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
                if !self.pretty_print {
                    let output_format = self
                        .shell
                        .as_ref()
                        .map(Shell::to_output_format)
                        .unwrap_or(ops::RawOutputFormat::Kv);

                    ops::list_raw(
                        &mut std::io::stdout(),
                        db,
                        env,
                        self.sort.to_str(),
                        output_format,
                    )
                    .await?;
                } else {
                    let truncate = match self.truncate {
                        true => db::Truncate::Max(60),
                        false => db::Truncate::None,
                    };
                    ops::table_list(db, env, truncate, self.sort.to_str()).await?;
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
    fn parses_pwsh_alias_for_shell_flag() {
        let cmd = <Cmd as clap::Parser>::try_parse_from(["list", "dev", "--shell", "pwsh"])
            .expect("--shell pwsh should parse");

        assert_eq!(
            cmd.shell
                .as_ref()
                .expect("shell should be set")
                .to_output_format(),
            ops::RawOutputFormat::PowerShell
        );
    }

    #[test]
    fn parses_bash_alias_for_shell_flag() {
        let cmd = <Cmd as clap::Parser>::try_parse_from(["list", "dev", "--shell", "bash"])
            .expect("--shell bash should parse");

        assert_eq!(
            cmd.shell
                .as_ref()
                .expect("shell should be set")
                .to_output_format(),
            ops::RawOutputFormat::Sh
        );
    }

    #[test]
    fn shell_flag_requires_environment_argument() {
        let err = <Cmd as clap::Parser>::try_parse_from(["list", "--shell", "sh"])
            .err()
            .expect("--shell without env should fail");

        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }
}
