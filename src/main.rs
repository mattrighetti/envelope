mod command;
mod core;
mod db;
mod editor;
mod ops;
mod subproc;
mod utils;

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use command::{CommandOutcome, EnvelopeCmd};

const VERSION: &str = env!("CARGO_PKG_VERSION");

static HELP_TEMPLATE: &str = "\
{about}

{usage-heading} {usage}

{all-args}{after-help}";

/// A modern environment variables manager
#[derive(Parser)]
#[command(
    author = "Mattia Righetti <matt95.righetti@gmail.com>",
    version = VERSION,
    help_template(HELP_TEMPLATE),
)]
struct Envelope {
    #[command(subcommand)]
    envelope: Option<EnvelopeCmd>,
}

impl Envelope {
    #[tokio::main(flavor = "current_thread")]
    async fn run(self) -> Result<CommandOutcome> {
        if let Some(cmd) = self.envelope {
            cmd.run().await
        } else if !io::stdin().is_terminal() {
            ops::print_from_stdin().await?;
            Ok(CommandOutcome::Success)
        } else {
            Self::command().print_help()?;
            Ok(CommandOutcome::Success)
        }
    }
}

fn main() -> ExitCode {
    match Envelope::parse().run() {
        Ok(outcome) => outcome.into(),
        Err(err) => {
            _ = writeln!(io::stderr(), "error: {err}");
            ExitCode::FAILURE
        }
    }
}
