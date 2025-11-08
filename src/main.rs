//! Command-line entry point for wlgen-rs.

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{self, stdout, Write};
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

    // Skip first N combinations if requested (for resuming or distributed workloads)
    if cli.skip > 0 {
        generator.skip_first(cli.skip)?;
    }

    // Determine output destination and compression
    match &cli.output {
        None => {
            // Write to stdout
            if cli.progress {
                generator.write_to_with_progress(stdout())?;
            } else {
                generator.write_to(stdout())?;
            }
        }
        Some(path) => {
            // Write to file with optional compression based on extension
            let file = File::create(path)
                .with_context(|| format!("failed to create output file: {path}"))?;

            #[cfg(feature = "compression")]
            {
                if path.ends_with(".gz") {
                    // Gzip compression
                    let encoder =
                        flate2::write::GzEncoder::new(file, flate2::Compression::default());
                    write_with_progress(&mut generator, encoder, cli.progress)?;
                } else if path.ends_with(".zst") {
                    // Zstandard compression
                    let encoder = zstd::Encoder::new(file, 3)?; // Compression level 3 (balanced)
                    write_with_progress(&mut generator, encoder.auto_finish(), cli.progress)?;
                } else {
                    // No compression
                    write_with_progress(&mut generator, file, cli.progress)?;
                }
            }

            #[cfg(not(feature = "compression"))]
            {
                if path.ends_with(".gz") || path.ends_with(".zst") {
                    eprintln!("Warning: Compression requested but not enabled. Rebuild with --features compression");
                }
                write_with_progress(&mut generator, file, cli.progress)?;
            }
        }
    }

    Ok(())
}

/// Helper function to write with or without progress reporting
fn write_with_progress<W: Write>(
    generator: &mut WordlistGenerator,
    writer: W,
    progress: bool,
) -> io::Result<()> {
    if progress {
        generator.write_to_with_progress(writer)
    } else {
        generator.write_to(writer)
    }
}
