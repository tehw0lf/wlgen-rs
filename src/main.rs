//! Command-line entry point for wlgen-rs.

use anyhow::Result;
use clap::Parser;
use std::io::stdout;
use wlgen_rs::{Cli, WordlistGenerator};

fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Validate the CLI arguments (checks mask references valid charsets)
    cli.validate()?;

    // Parse the mask pattern into charsets
    let charsets = cli.parse_mask()?;

    // Create the wordlist generator
    let mut generator = WordlistGenerator::new(charsets);

    // Stream words to stdout (buffered for performance)
    generator.write_to(stdout())?;

    Ok(())
}
