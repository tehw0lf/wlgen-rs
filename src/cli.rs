//! Command-line interface for wlgen-rs.
//!
//! Provides a maskprocessor-compatible CLI for generating wordlists.

use anyhow::{anyhow, Context, Result};
use clap::Parser;

/// High-performance wordlist generator for hashcat
///
/// Generates wordlists from custom character sets using mask patterns,
/// similar to hashcat's maskprocessor but with 100-200M combinations/second.
///
/// # Examples
///
/// ```bash
/// # Generate simple 2-character wordlist
/// wlgen-rs -1 'abc' -2 '123' '?1?2'
///
/// # Pipe to hashcat for WPA2 cracking
/// wlgen-rs -1 'ABCDEF' -2 '0123456789' '?1?1?2?2?2?2?2?2' | hashcat -m 2500 capture.hccapx
///
/// # Complex pattern with multiple charsets
/// wlgen-rs -1 'ABCDEF' -2 '0123456789' -3 '!@#$' '?1?1?2?2?3'
/// ```
#[derive(Parser, Debug)]
#[command(name = "wlgen-rs")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Custom charset 1
    #[arg(short = '1', long = "custom-charset1", value_name = "CS")]
    pub charset1: Option<String>,

    /// Custom charset 2
    #[arg(short = '2', long = "custom-charset2", value_name = "CS")]
    pub charset2: Option<String>,

    /// Custom charset 3
    #[arg(short = '3', long = "custom-charset3", value_name = "CS")]
    pub charset3: Option<String>,

    /// Custom charset 4
    #[arg(short = '4', long = "custom-charset4", value_name = "CS")]
    pub charset4: Option<String>,

    /// Custom charset 5
    #[arg(short = '5', long = "custom-charset5", value_name = "CS")]
    pub charset5: Option<String>,

    /// Custom charset 6
    #[arg(short = '6', long = "custom-charset6", value_name = "CS")]
    pub charset6: Option<String>,

    /// Custom charset 7
    #[arg(short = '7', long = "custom-charset7", value_name = "CS")]
    pub charset7: Option<String>,

    /// Custom charset 8
    #[arg(short = '8', long = "custom-charset8", value_name = "CS")]
    pub charset8: Option<String>,

    /// Custom charset 9
    #[arg(short = '9', long = "custom-charset9", value_name = "CS")]
    pub charset9: Option<String>,

    /// Mask pattern (e.g., "?1?1?2?2")
    ///
    /// Use ?1-?9 to reference custom charsets defined with -1 through -9.
    /// Built-in charsets (like ?l, ?u, ?d from hashcat) are not yet supported.
    pub mask: String,
}

impl Cli {
    /// Parses the mask pattern and returns a vector of charsets for each position.
    ///
    /// The mask pattern uses ?N placeholders where N is 1-9, referencing the
    /// custom charsets defined via command-line arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The mask references an undefined charset
    /// - The mask contains invalid syntax
    /// - Any referenced charset is empty
    ///
    /// # Example
    ///
    /// ```
    /// # use wlgen_rs::Cli;
    /// # use clap::Parser;
    /// let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "-2", "123", "?1?2"]);
    /// let charsets = cli.parse_mask().unwrap();
    /// assert_eq!(charsets.len(), 2);
    /// assert_eq!(charsets[0], b"abc");
    /// assert_eq!(charsets[1], b"123");
    /// ```
    pub fn parse_mask(&self) -> Result<Vec<Vec<u8>>> {
        let mut charsets = Vec::new();
        let mask_bytes = self.mask.as_bytes();
        let mut i = 0;

        while i < mask_bytes.len() {
            if mask_bytes[i] == b'?' {
                if i + 1 >= mask_bytes.len() {
                    return Err(anyhow!("incomplete placeholder at end of mask"));
                }

                let placeholder = mask_bytes[i + 1];

                // Handle ?1 through ?9
                if (b'1'..=b'9').contains(&placeholder) {
                    let charset_num = (placeholder - b'0') as usize;
                    let charset = self
                        .get_charset(charset_num)
                        .with_context(|| format!("charset ?{charset_num} not defined"))?;

                    if charset.is_empty() {
                        return Err(anyhow!("charset ?{charset_num} is empty"));
                    }

                    charsets.push(charset.as_bytes().to_vec());
                    i += 2; // Skip '?' and digit
                } else {
                    return Err(anyhow!(
                        "invalid placeholder: ?{} (only ?1-?9 are supported)",
                        placeholder as char
                    ));
                }
            } else {
                // Literal character (not a placeholder)
                charsets.push(vec![mask_bytes[i]]);
                i += 1;
            }
        }

        if charsets.is_empty() {
            return Err(anyhow!("mask cannot be empty"));
        }

        Ok(charsets)
    }

    /// Gets the charset for a given number (1-9).
    ///
    /// Returns `None` if the charset was not defined.
    fn get_charset(&self, num: usize) -> Option<&String> {
        match num {
            1 => self.charset1.as_ref(),
            2 => self.charset2.as_ref(),
            3 => self.charset3.as_ref(),
            4 => self.charset4.as_ref(),
            5 => self.charset5.as_ref(),
            6 => self.charset6.as_ref(),
            7 => self.charset7.as_ref(),
            8 => self.charset8.as_ref(),
            9 => self.charset9.as_ref(),
            _ => None,
        }
    }

    /// Validates that all charsets referenced in the mask are defined.
    ///
    /// # Errors
    ///
    /// Returns an error if the mask references an undefined charset.
    pub fn validate(&self) -> Result<()> {
        // Try to parse the mask - this will catch most errors
        self.parse_mask()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_mask() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "-2", "123", "?1?2"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 2);
        assert_eq!(charsets[0], b"abc");
        assert_eq!(charsets[1], b"123");
    }

    #[test]
    fn test_parse_repeated_charset() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "?1?1"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 2);
        assert_eq!(charsets[0], b"abc");
        assert_eq!(charsets[1], b"abc");
    }

    #[test]
    fn test_parse_all_charsets() {
        let cli = Cli::parse_from(vec![
            "wlgen-rs",
            "-1",
            "a",
            "-2",
            "b",
            "-3",
            "c",
            "-4",
            "d",
            "-5",
            "e",
            "-6",
            "f",
            "-7",
            "g",
            "-8",
            "h",
            "-9",
            "i",
            "?1?2?3?4?5?6?7?8?9",
        ]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 9);
        assert_eq!(charsets[0], b"a");
        assert_eq!(charsets[8], b"i");
    }

    #[test]
    fn test_parse_literal_characters() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "x?1y"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 3);
        assert_eq!(charsets[0], b"x");
        assert_eq!(charsets[1], b"abc");
        assert_eq!(charsets[2], b"y");
    }

    #[test]
    fn test_undefined_charset_error() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "?1?2"]);
        let result = cli.parse_mask();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("charset ?2 not defined"));
    }

    #[test]
    fn test_incomplete_placeholder_error() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "?1?"]);
        let result = cli.parse_mask();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("incomplete placeholder"));
    }

    #[test]
    fn test_invalid_placeholder_error() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "?1?x"]);
        let result = cli.parse_mask();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid placeholder"));
    }

    #[test]
    fn test_empty_mask_error() {
        let cli = Cli::parse_from(vec!["wlgen-rs", ""]);
        let result = cli.parse_mask();

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("mask cannot be empty"));
    }

    #[test]
    fn test_validate_success() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "-2", "123", "?1?2"]);
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn test_validate_failure() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "abc", "?1?2"]);
        assert!(cli.validate().is_err());
    }
}
