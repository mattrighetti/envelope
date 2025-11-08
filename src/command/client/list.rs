use std::io;
use std::io::Result;

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

    #[arg(long, short)]
    truncate: bool,

    /// How envelope should sort result
    #[arg(long, short, default_value = "d")]
    sort: Sort,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        match &self.env {
            None => ops::list_envs(&mut io::stdout(), db).await?,
            Some(env) => {
                if !self.pretty_print {
                    ops::list_raw(&mut io::stdout(), db, env, self.sort.to_str()).await?;
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
