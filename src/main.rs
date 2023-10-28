mod command;
mod db;
mod ops;

use clap::Parser;
use command::EnvelopeCmd;
use std::io::Write;

const VERSION: &str = env!("CARGO_PKG_VERSION");

static HELP_TEMPLATE: &str = "\
{about}

{usage-heading} {usage}

{all-args}{after-help}";

/// A self-contained .env manager
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
    async fn run(self) -> std::io::Result<()> {
        match self.envelope {
            Some(envelope) => {
                envelope.run().await?;
            }
            None => {
                ops::print_from_stdin().await?;
            }
        }

        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    if let Err(err) = Envelope::parse().run() {
        writeln!(std::io::stdout(), "error: {}", err)?;
        std::process::exit(1);
    }

    Ok(())
}
