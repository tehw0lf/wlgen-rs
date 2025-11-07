//! Command-line interface for wlgen-rs.
//!
//! Provides a maskprocessor-compatible CLI for generating wordlists.

use anyhow::{anyhow, Context, Result};
use clap::Parser;

/// Built-in charsets compatible with hashcat/maskprocessor
pub mod builtin {
    /// ?l = lowercase letters
    pub const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

    /// ?u = uppercase letters
    pub const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";

    /// ?d = digits
    pub const DIGITS: &[u8] = b"0123456789";

    /// ?s = special characters
    pub const SPECIAL: &[u8] = b" !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~";

    /// Get the built-in charset for a given placeholder character
    pub fn get(placeholder: u8) -> Option<Vec<u8>> {
        match placeholder {
            b'l' => Some(LOWERCASE.to_vec()),
            b'u' => Some(UPPERCASE.to_vec()),
            b'd' => Some(DIGITS.to_vec()),
            b's' => Some(SPECIAL.to_vec()),
            b'a' => {
                // ?a = all printable ASCII (?l?u?d?s)
                let mut all = Vec::new();
                all.extend_from_slice(LOWERCASE);
                all.extend_from_slice(UPPERCASE);
                all.extend_from_slice(DIGITS);
                all.extend_from_slice(SPECIAL);
                Some(all)
            }
            b'b' => {
                // ?b = all bytes (0x00-0xFF)
                Some((0u8..=255u8).collect())
            }
            _ => None,
        }
    }
}

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

                // Handle ?1 through ?9 (custom charsets)
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
                }
                // Handle built-in charsets (?l, ?u, ?d, ?s, ?a, ?b)
                else if let Some(charset) = builtin::get(placeholder) {
                    charsets.push(charset);
                    i += 2; // Skip '?' and charset character
                } else {
                    return Err(anyhow!(
                        "invalid placeholder: ?{} (supported: ?1-?9, ?l, ?u, ?d, ?s, ?a, ?b)",
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

    // Tests for built-in charsets

    #[test]
    fn test_builtin_lowercase() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "?l"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 1);
        assert_eq!(charsets[0], b"abcdefghijklmnopqrstuvwxyz");
    }

    #[test]
    fn test_builtin_uppercase() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "?u"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 1);
        assert_eq!(charsets[0], b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }

    #[test]
    fn test_builtin_digits() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "?d"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 1);
        assert_eq!(charsets[0], b"0123456789");
    }

    #[test]
    fn test_builtin_special() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "?s"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 1);
        assert_eq!(charsets[0], b" !\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~");
    }

    #[test]
    fn test_builtin_all_ascii() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "?a"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 1);
        assert_eq!(charsets[0].len(), 95); // lowercase + uppercase + digits + special
        assert!(charsets[0].contains(&b'a')); // lowercase
        assert!(charsets[0].contains(&b'Z')); // uppercase
        assert!(charsets[0].contains(&b'5')); // digits
        assert!(charsets[0].contains(&b'!')); // special
    }

    #[test]
    fn test_builtin_all_bytes() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "?b"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 1);
        assert_eq!(charsets[0].len(), 256); // 0x00-0xFF
        assert_eq!(charsets[0][0], 0);
        assert_eq!(charsets[0][255], 255);
    }

    #[test]
    fn test_builtin_mixed_with_custom() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "-1", "XYZ", "?l?1?d"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 3);
        assert_eq!(charsets[0], b"abcdefghijklmnopqrstuvwxyz");
        assert_eq!(charsets[1], b"XYZ");
        assert_eq!(charsets[2], b"0123456789");
    }

    #[test]
    fn test_builtin_repeated() {
        let cli = Cli::parse_from(vec!["wlgen-rs", "?d?d?d"]);
        let charsets = cli.parse_mask().unwrap();

        assert_eq!(charsets.len(), 3);
        assert_eq!(charsets[0], b"0123456789");
        assert_eq!(charsets[1], b"0123456789");
        assert_eq!(charsets[2], b"0123456789");
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
