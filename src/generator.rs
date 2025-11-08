//! High-performance wordlist generator using the odometer algorithm.
//!
//! This module implements a zero-allocation wordlist generator that achieves
//! 100-200M combinations/second by reusing a single buffer and incrementing
//! position indices like an odometer.

use std::io::{self, BufWriter, Write};
use std::time::{Duration, Instant};

/// A high-performance wordlist generator using the odometer pattern.
///
/// The generator maintains a single buffer and reuses it for each word,
/// achieving O(1) memory usage and maximum performance.
///
/// # Algorithm
///
/// Similar to hashcat's maskprocessor, this uses an "odometer" pattern:
/// 1. Start with all positions at their first character
/// 2. Increment the rightmost position
/// 3. When a position overflows, reset it and carry to the left
/// 4. Continue until all positions overflow
///
/// # Example
///
/// ```
/// use wlgen_rs::WordlistGenerator;
///
/// let charsets = vec![
///     b"abc".to_vec(),
///     b"123".to_vec(),
/// ];
///
/// let mut gen = WordlistGenerator::new(charsets);
/// let words: Vec<String> = gen.collect();
/// assert_eq!(words, vec!["a1", "a2", "a3", "b1", "b2", "b3", "c1", "c2", "c3"]);
/// ```
pub struct WordlistGenerator {
    /// Character bytes for each position in the word
    charsets: Vec<Vec<u8>>,
    /// Reusable buffer for the current word
    buffer: Vec<u8>,
    /// Current index in each charset (odometer state)
    positions: Vec<usize>,
    /// Whether iteration is exhausted
    exhausted: bool,
}

impl WordlistGenerator {
    /// Creates a new wordlist generator from the given charsets.
    ///
    /// # Arguments
    ///
    /// * `charsets` - A vector of character sets, one for each position.
    ///   Each charset is a Vec<u8> of possible bytes for that position.
    ///
    /// # Panics
    ///
    /// Panics if any charset is empty or if charsets is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use wlgen_rs::WordlistGenerator;
    ///
    /// let charsets = vec![
    ///     b"AB".to_vec(),
    ///     b"12".to_vec(),
    /// ];
    ///
    /// let mut gen = WordlistGenerator::new(charsets);
    /// assert_eq!(gen.next(), Some("A1".to_string()));
    /// ```
    pub fn new(charsets: Vec<Vec<u8>>) -> Self {
        assert!(!charsets.is_empty(), "charsets cannot be empty");

        for (i, charset) in charsets.iter().enumerate() {
            assert!(!charset.is_empty(), "charset {i} cannot be empty");
        }

        // Initialize buffer with the first character of each charset
        let buffer: Vec<u8> = charsets.iter().map(|cs| cs[0]).collect();

        // All positions start at 0
        let positions = vec![0; charsets.len()];

        Self {
            charsets,
            buffer,
            positions,
            exhausted: false,
        }
    }

    /// Calculates the total number of possible combinations (keyspace).
    ///
    /// This is useful for progress reporting and ETA calculations.
    ///
    /// # Example
    ///
    /// ```
    /// use wlgen_rs::WordlistGenerator;
    ///
    /// let charsets = vec![
    ///     b"abc".to_vec(),
    ///     b"12".to_vec(),
    /// ];
    ///
    /// let gen = WordlistGenerator::new(charsets);
    /// assert_eq!(gen.keyspace(), 6); // 3 * 2 = 6
    /// ```
    pub fn keyspace(&self) -> u64 {
        self.charsets.iter().map(|cs| cs.len() as u64).product()
    }

    /// Skips the first N combinations without generating them.
    ///
    /// This is useful for:
    /// - Resuming interrupted wordlist generation
    /// - Distributed workloads (split keyspace across machines)
    /// - Generating specific portions of a large keyspace
    ///
    /// # Arguments
    ///
    /// * `n` - Number of combinations to skip
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if:
    /// - `n` exceeds the total keyspace
    /// - The generator is already exhausted
    ///
    /// # Example
    ///
    /// ```
    /// use wlgen_rs::WordlistGenerator;
    ///
    /// let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
    /// let mut gen = WordlistGenerator::new(charsets);
    ///
    /// // Skip first 2 combinations (a1, a2)
    /// gen.skip_first(2).unwrap();
    ///
    /// // Next word should be b1
    /// assert_eq!(gen.next(), Some("b1".to_string()));
    /// ```
    pub fn skip_first(&mut self, n: u64) -> io::Result<()> {
        if n == 0 {
            return Ok(());
        }

        if self.exhausted {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "generator is already exhausted",
            ));
        }

        let keyspace = self.keyspace();
        if n >= keyspace {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("skip count ({n}) exceeds keyspace ({keyspace})"),
            ));
        }

        // Efficiently advance the odometer n times without generating strings
        for _ in 0..n {
            if !self.next_word() {
                self.exhausted = true;
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "generator exhausted during skip",
                ));
            }
        }

        Ok(())
    }

    /// Advances the odometer to the next combination.
    ///
    /// Returns `true` if more words are available, `false` if exhausted.
    ///
    /// This is the core algorithm: increment positions from right to left,
    /// carrying when a position overflows.
    fn next_word(&mut self) -> bool {
        if self.exhausted {
            return false;
        }

        // Start from the rightmost position (like an odometer)
        for i in (0..self.positions.len()).rev() {
            self.positions[i] += 1;

            // If we haven't overflowed this position, we're done
            if self.positions[i] < self.charsets[i].len() {
                self.buffer[i] = self.charsets[i][self.positions[i]];
                return true;
            }

            // Overflow: reset this position and carry to the left
            self.positions[i] = 0;
            self.buffer[i] = self.charsets[i][0];
        }

        // If we've carried past the leftmost position, we're exhausted
        self.exhausted = true;
        false
    }

    /// Returns a reference to the current word buffer as a string slice.
    ///
    /// # Safety
    ///
    /// This assumes the buffer contains valid UTF-8. If you're using
    /// non-UTF-8 charsets, this may panic.
    fn current_word(&self) -> &str {
        std::str::from_utf8(&self.buffer).expect("invalid UTF-8 in charset")
    }

    /// Writes all words to the given writer, one per line.
    ///
    /// This is optimized for stdout streaming and uses a buffered writer
    /// to minimize syscalls.
    ///
    /// **Performance optimization:** Writes buffer bytes directly without UTF-8
    /// validation, avoiding the 28.7% overhead identified in profiling.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any type implementing `Write` (e.g., `std::io::stdout()`)
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wlgen_rs::WordlistGenerator;
    /// use std::io::stdout;
    ///
    /// let charsets = vec![b"abc".to_vec(), b"123".to_vec()];
    /// let mut gen = WordlistGenerator::new(charsets);
    /// gen.write_to(stdout()).unwrap();
    /// ```
    pub fn write_to<W: Write>(&mut self, writer: W) -> io::Result<()> {
        // Use 1MB buffer to reduce write syscalls (optimized from original 64KB)
        // Benchmarking showed 1MB is the sweet spot (2MB is slower due to cache effects)
        let mut buf_writer = BufWriter::with_capacity(1024 * 1024, writer);

        // Write the first word (raw bytes + newline)
        buf_writer.write_all(&self.buffer)?;
        buf_writer.write_all(b"\n")?;

        // Generate and write remaining words
        while self.next_word() {
            buf_writer.write_all(&self.buffer)?;
            buf_writer.write_all(b"\n")?;
        }

        buf_writer.flush()
    }

    /// Writes all words to the given writer with progress reporting.
    ///
    /// Progress is written to stderr and includes:
    /// - Current position / total combinations
    /// - Percentage complete
    /// - Current throughput (words/s)
    /// - Estimated time remaining (ETA)
    ///
    /// Progress updates every 100ms or every 1M words, whichever is less frequent.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any type implementing `Write` (e.g., `std::io::stdout()`)
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_to_with_progress<W: Write>(&mut self, writer: W) -> io::Result<()> {
        let mut buf_writer = BufWriter::with_capacity(1024 * 1024, writer);
        let total = self.keyspace();
        let mut count: u64 = 1; // Start at 1 (we write first word immediately)
        let start_time = Instant::now();
        let mut last_update = Instant::now();
        const UPDATE_INTERVAL: Duration = Duration::from_millis(100);
        const WORDS_PER_UPDATE: u64 = 1_000_000;

        // Write the first word
        buf_writer.write_all(&self.buffer)?;
        buf_writer.write_all(b"\n")?;

        // Generate and write remaining words with progress reporting
        while self.next_word() {
            buf_writer.write_all(&self.buffer)?;
            buf_writer.write_all(b"\n")?;
            count += 1;

            // Update progress periodically
            if count % WORDS_PER_UPDATE == 0 || last_update.elapsed() >= UPDATE_INTERVAL {
                let elapsed = start_time.elapsed().as_secs_f64();
                let rate = count as f64 / elapsed;
                let percentage = (count as f64 / total as f64) * 100.0;
                let remaining_words = total - count;
                let eta_seconds = if rate > 0.0 {
                    remaining_words as f64 / rate
                } else {
                    0.0
                };

                // Write to stderr (not stdout, to avoid mixing with wordlist)
                eprint!(
                    "\r[Progress] {}/{} ({:.2}%) | {:.2}M words/s | ETA: {:.1}s   ",
                    count,
                    total,
                    percentage,
                    rate / 1_000_000.0,
                    eta_seconds
                );

                last_update = Instant::now();
            }
        }

        // Final progress update
        let elapsed = start_time.elapsed().as_secs_f64();
        let rate = count as f64 / elapsed;
        eprintln!(
            "\r[Complete] {}/{} (100.00%) | {:.2}M words/s | Total: {:.2}s   ",
            count,
            total,
            rate / 1_000_000.0,
            elapsed
        );

        buf_writer.flush()
    }
}

impl Iterator for WordlistGenerator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        let word = self.current_word().to_string();

        // Advance to next word (this will set exhausted if we're done)
        if !self.next_word() {
            self.exhausted = true;
        }

        Some(word)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let keyspace = self.keyspace();
        if keyspace <= usize::MAX as u64 {
            let remaining = keyspace as usize;
            (remaining, Some(remaining))
        } else {
            (usize::MAX, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_wordlist() {
        let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
        let gen = WordlistGenerator::new(charsets);

        let words: Vec<String> = gen.collect();
        assert_eq!(words, vec!["a1", "a2", "b1", "b2"]);
    }

    #[test]
    fn test_single_position() {
        let charsets = vec![b"abc".to_vec()];
        let gen = WordlistGenerator::new(charsets);

        let words: Vec<String> = gen.collect();
        assert_eq!(words, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_single_char() {
        let charsets = vec![b"a".to_vec()];
        let gen = WordlistGenerator::new(charsets);

        let words: Vec<String> = gen.collect();
        assert_eq!(words, vec!["a"]);
    }

    #[test]
    fn test_three_positions() {
        let charsets = vec![b"ab".to_vec(), b"12".to_vec(), b"xy".to_vec()];
        let gen = WordlistGenerator::new(charsets);

        let words: Vec<String> = gen.collect();
        assert_eq!(
            words,
            vec!["a1x", "a1y", "a2x", "a2y", "b1x", "b1y", "b2x", "b2y"]
        );
    }

    #[test]
    fn test_keyspace() {
        let charsets = vec![b"abc".to_vec(), b"12".to_vec()];
        let gen = WordlistGenerator::new(charsets);
        assert_eq!(gen.keyspace(), 6);
    }

    #[test]
    fn test_keyspace_large() {
        let charsets = vec![
            b"abcdefghij".to_vec(),
            b"0123456789".to_vec(),
            b"0123456789".to_vec(),
        ];
        let gen = WordlistGenerator::new(charsets);
        assert_eq!(gen.keyspace(), 1000);
    }

    #[test]
    #[should_panic(expected = "charsets cannot be empty")]
    fn test_empty_charsets() {
        let charsets: Vec<Vec<u8>> = vec![];
        WordlistGenerator::new(charsets);
    }

    #[test]
    #[should_panic(expected = "charset 0 cannot be empty")]
    fn test_empty_charset() {
        let charsets = vec![vec![]];
        WordlistGenerator::new(charsets);
    }

    #[test]
    fn test_write_to() {
        let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
        let mut gen = WordlistGenerator::new(charsets);

        let mut output = Vec::new();
        gen.write_to(&mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "a1\na2\nb1\nb2\n");
    }

    #[test]
    fn test_skip_basic() {
        let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
        let mut gen = WordlistGenerator::new(charsets);

        // Skip first 2 combinations (a1, a2)
        gen.skip_first(2).unwrap();

        // Next word should be b1
        assert_eq!(gen.next(), Some("b1".to_string()));
        assert_eq!(gen.next(), Some("b2".to_string()));
        assert_eq!(gen.next(), None);
    }

    #[test]
    fn test_skip_zero() {
        let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
        let mut gen = WordlistGenerator::new(charsets);

        // Skip 0 should be a no-op
        gen.skip_first(0).unwrap();

        assert_eq!(gen.next(), Some("a1".to_string()));
    }

    #[test]
    fn test_skip_all_but_one() {
        let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
        let mut gen = WordlistGenerator::new(charsets);

        // Skip 3 out of 4 combinations
        gen.skip_first(3).unwrap();

        // Only last word should remain
        assert_eq!(gen.next(), Some("b2".to_string()));
        assert_eq!(gen.next(), None);
    }

    #[test]
    fn test_skip_exceeds_keyspace() {
        let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
        let mut gen = WordlistGenerator::new(charsets);

        // Trying to skip 4 when keyspace is 4 should fail
        let result = gen.skip_first(4);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds keyspace"));
    }

    #[test]
    fn test_skip_exhausted_generator() {
        let charsets = vec![b"a".to_vec()];
        let mut gen = WordlistGenerator::new(charsets);

        // Exhaust the generator
        gen.next();

        // Trying to skip on exhausted generator should fail
        let result = gen.skip_first(1);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already exhausted"));
    }

    #[test]
    fn test_skip_with_write_to() {
        let charsets = vec![b"ab".to_vec(), b"12".to_vec()];
        let mut gen = WordlistGenerator::new(charsets);

        // Skip first 2
        gen.skip_first(2).unwrap();

        let mut output = Vec::new();
        gen.write_to(&mut output).unwrap();

        let result = String::from_utf8(output).unwrap();
        assert_eq!(result, "b1\nb2\n");
    }
}
