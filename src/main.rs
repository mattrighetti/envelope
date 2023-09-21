mod db;
mod ops;
mod command;

use clap::Parser;
use command::EnvelopeCmd;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const SHA: &str = env!("GIT_HASH");

static HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{author}
{about}

{usage-heading}
  {usage}

{all-args}{after-help}";

#[derive(Parser)]
#[command(
    author = "Mattia Righetti <matt95.righetti@gmail.com>",
    version = VERSION,
    help_template(HELP_TEMPLATE),
)]
struct Envelope {
    #[command(subcommand)]
    envelope: EnvelopeCmd,
}

impl Envelope {
    fn run(self) -> std::io::Result<()> {
        self.envelope.run()
    }
}

fn main() -> std::io::Result<()> {
    Envelope::parse().run()
}
