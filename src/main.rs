mod db;
mod ops;
mod command;

use clap::Parser;
use command::EnvelopeCmd;

#[derive(Parser)]
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
