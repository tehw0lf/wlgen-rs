# wlgen-rs: CPU-Based Rust Wordlist Generator

> **🚀 Achievement:** This CPU implementation now achieves **~164M words/s** - **11% faster than maskprocessor**! For even higher performance (500M-1B words/s), see the **GPU Scatter-Gather Wordlist Generator** project.

## Overview
CPU-based Rust CLI tool for wordlist generation, achieving **~164M candidates/second** (211x speedup over Python, 11% faster than C maskprocessor).

**Architecture Decision:** Standalone Rust binary (CPU-only)
- Pure Rust CLI tool, using odometer algorithm similar to hashcat's maskprocessor
- Can be used from any language via stdin/stdout pipes
- CPU fallback when GPU is unavailable
- Reference implementation for correctness validation
- Pre-built binaries for easy distribution
- Optional Python bindings as separate feature (future)

## Project Status (2025-11-07 - OPTIMIZED)

**Current Performance:** ~164M words/s average, 168M peak (measured: 676M words in 4.1s)
- ✅ **3.93x faster than initial implementation** (41.8M/s)
- ✅ **211x faster than Python** (~780K/s)
- ✅ **1.11x faster than maskprocessor** (~147.5M/s) - **11% performance advantage!**
- ✅ **Fully saturates WPA2-PSK cracking** (911 KH/s) with 180x surplus
- ✅ **Pure safe Rust** - no unsafe code required

**Purpose:** High-performance CPU generator, reference implementation, learning project

**Completed Milestones (2025-11-07):**
- ✅ Phase 1: Repository setup complete
- ✅ Phase 2: Core implementation complete (CLI + generator working)
- ✅ Phase 3: Testing complete (19 unit tests + 9 integration tests + 6 doc tests passing)
- ✅ Phase 5: Performance validation and optimization complete
  - Initial benchmark: 41.8M words/s
  - Profiled with flamegraph (identified UTF-8 validation bottleneck)
  - **Optimization 1:** Removed UTF-8 validation (3.58x speedup → 146M words/s)
  - **Optimization 2:** Increased buffer size to 1MB (1.16x speedup → 164M words/s)
  - Final result: **11% faster than maskprocessor**
- ✅ Phase 6: Documentation updated with optimization details
- ✅ Phase 7: CI/CD complete (GitHub Actions workflows)

## Original Motivation
Python implementation achieves ~780K candidates/second. Rust achieves 51x improvement, but maskprocessor (C) is 3.5x faster than this Rust implementation at ~142M words/s.

**Primary Use Case: WPA2-PSK Cracking**
- hashcat WPA-PBKDF2: 911.8 KH/s
- Current performance: ~40M words/s provides 44x surplus ✓
- Adequate for WPA2 cracking, but not optimal for fast hashes

**Hashcat RTX 4070 Benchmark Results:**
- NTLM: 92 GH/s → needs 2,300x more throughput (wlgen-rs too slow)
- MD5: 55 GH/s → needs 1,375x more throughput (wlgen-rs too slow)
- SHA-256: 7.8 GH/s → needs 195x more throughput (wlgen-rs too slow)
- SHA-512: 2.3 GH/s → needs 58x more throughput (wlgen-rs too slow)
- **WPA-PBKDF2: 911.8 KH/s** → **44x surplus with wlgen-rs ✓**
- bcrypt: 23 KH/s → wlgen-rs provides 1,739x surplus ✓

**Performance Reality (2025-11-07 After Optimization):**
- ✅ wlgen-rs @ ~164M/s: **Exceeds maskprocessor performance by 11%!**
- ✅ Fully adequate for WPA2 and slow hashes (180x surplus)
- ⚠️ Still insufficient for fast hashes (MD5, NTLM, SHA-256) requiring 500M+ words/s
- 💡 **For ultra-high-performance needs → Use GPU Scatter-Gather project (500M-1B words/s)**

## Architecture

### Core Algorithm: Odometer Pattern
Based on hashcat's maskprocessor implementation (https://github.com/hashcat/maskprocessor):

```rust
// Simplified concept
fn next_word(buffer: &mut [u8], charsets: &[&[u8]], positions: &mut [usize]) -> bool {
    // Increment rightmost position (like odometer)
    for i in (0..positions.len()).rev() {
        positions[i] += 1;

        if positions[i] < charsets[i].len() {
            buffer[i] = charsets[i][positions[i]];
            return true;  // More words available
        }

        // Overflow: reset and carry
        positions[i] = 0;
        buffer[i] = charsets[i][0];
    }
    false  // Exhausted all combinations
}
```

### Performance Optimizations
1. **In-place buffer mutation** - No string allocations per word
2. **Single allocation** - Reuse same buffer for all iterations
3. **Zero-copy iteration** - Yield string slices, not owned strings
4. **Cache-friendly access** - Sequential memory access patterns
5. **SIMD potential** - Future optimization for character lookups

### CLI Design (Similar to maskprocessor)
```bash
# Basic usage
wlgen-rs -1 'abc' -2 '123' '?1?2'
# Output: a1, a2, a3, b1, b2, b3, c1, c2, c3

# Pipe to hashcat
wlgen-rs -1 'abc' -2 '123' '?1?2' | hashcat -m 2500 capture.hccapx

# Complex example with multiple charsets
wlgen-rs -1 'ABCDEF' -2 '0123456789' -3 '!@#$' '?1?1?2?2?3'
```

### Integration Strategy
- **Pure Rust binary** - Standalone executable
- **Stdout streaming** - One word per line
- **Universal compatibility** - Works with any tool via pipes
- **Optional Python bindings** - Future feature for library use

## Implementation Plan

### Phase 1: Repository & Project Setup
- [x] **Research best practices** ✅
  - Studied maskprocessor architecture
  - Reviewed Rust CLI best practices (clap, structopt)
  - Analyzed performance-critical Rust patterns
  - Determined standalone binary is best approach

- [x] **Create new repository: wlgen-rs** ✅
  - Repository created at /home/tehwolf/Nextcloud/Coding/Rust/wlgen-rs
  - Initialized with cargo
  - Git repository initialized

- [x] **Configure Cargo.toml** ✅
  ```toml
  [package]
  name = "wlgen-rs"
  version = "0.1.0"
  edition = "2021"
  authors = ["Your Name <email@example.com>"]
  description = "High-performance wordlist generator for hashcat"
  license = "MIT OR Apache-2.0"
  repository = "https://github.com/tehw0lf/wlgen-rs"

  [[bin]]
  name = "wlgen-rs"
  path = "src/main.rs"

  [dependencies]
  clap = { version = "4.5", features = ["derive"] }

  [dev-dependencies]
  criterion = "0.5"

  [[bench]]
  name = "wordlist_bench"
  harness = false
  ```

- [x] **Set up project structure** ✅
  - All core files implemented (main.rs, lib.rs, generator.rs, cli.rs)
  - Benchmarks directory created (wordlist_bench.rs)
  - Integration tests directory created (integration.rs)
  - README.md and LICENSE created
  - .github/workflows/ pending (next task)

### Phase 2: Core Implementation

- [x] **Design core data structures** ✅
  ```rust
  pub struct WordlistGenerator {
      charsets: Vec<Vec<u8>>,      // Character bytes for each position
      buffer: Vec<u8>,             // Reusable buffer for current word
      positions: Vec<usize>,       // Current index in each charset
      exhausted: bool,             // Iteration complete flag
  }
  ```

- [x] **Design odometer algorithm** ✅
  ```rust
  impl WordlistGenerator {
      fn next_word(&mut self) -> bool {
          // Start from rightmost position (like odometer)
          for i in (0..self.positions.len()).rev() {
              self.positions[i] += 1;

              if self.positions[i] < self.charsets[i].len() {
                  self.buffer[i] = self.charsets[i][self.positions[i]];
                  return true;  // More words available
              }

              // Overflow: reset and carry
              self.positions[i] = 0;
              self.buffer[i] = self.charsets[i][0];
          }
          false  // Exhausted all combinations
      }
  }
  ```

- [x] **Implement WordlistGenerator (src/generator.rs)** ✅
  - Core iterator implementing odometer algorithm ✅
  - Stdout writer with buffered I/O ✅
  - Keyspace calculation for ETA ✅
  - Optional progress reporting (deferred to future enhancement)

- [x] **Implement CLI parser (src/cli.rs)** ✅
  ```rust
  use clap::Parser;
  use std::collections::HashMap;

  #[derive(Parser)]
  #[command(name = "wlgen-rs")]
  #[command(about = "High-performance wordlist generator", long_about = None)]
  struct Cli {
      /// Custom charset 1-20 (convenient short flags)
      #[arg(short = '1', long)]
      charset1: Option<String>,
      #[arg(short = '2', long)]
      charset2: Option<String>,
      #[arg(short = '3', long)]
      charset3: Option<String>,
      #[arg(short = '4', long)]
      charset4: Option<String>,
      #[arg(short = '5', long)]
      charset5: Option<String>,
      #[arg(short = '6', long)]
      charset6: Option<String>,
      #[arg(short = '7', long)]
      charset7: Option<String>,
      #[arg(short = '8', long)]
      charset8: Option<String>,
      #[arg(short = '9', long)]
      charset9: Option<String>,

      // Extended charsets 10-20 (still using short single-char flags for convenience)
      #[arg(long = "10")]
      charset10: Option<String>,
      #[arg(long = "11")]
      charset11: Option<String>,
      #[arg(long = "12")]
      charset12: Option<String>,
      #[arg(long = "13")]
      charset13: Option<String>,
      #[arg(long = "14")]
      charset14: Option<String>,
      #[arg(long = "15")]
      charset15: Option<String>,
      #[arg(long = "16")]
      charset16: Option<String>,
      #[arg(long = "17")]
      charset17: Option<String>,
      #[arg(long = "18")]
      charset18: Option<String>,
      #[arg(long = "19")]
      charset19: Option<String>,
      #[arg(long = "20")]
      charset20: Option<String>,

      /// Additional charsets beyond 20: --charset 21=abc --charset 999=xyz
      /// Supports unlimited charsets (no upper limit)
      #[arg(short = 'c', long = "charset", value_name = "ID=CHARS", value_parser = parse_charset)]
      extra_charsets: Vec<(usize, String)>,

      /// Mask pattern (e.g., "?1?1?2", "?1?2?3?4?5", "?21?999")
      mask: String,
  }

  fn parse_charset(s: &str) -> Result<(usize, String), String> {
      let parts: Vec<&str> = s.splitn(2, '=').collect();
      if parts.len() != 2 {
          return Err(format!("Invalid charset format: '{}'. Expected 'ID=CHARS'", s));
      }
      let id = parts[0].parse::<usize>()
          .map_err(|_| format!("Invalid charset ID: '{}'", parts[0]))?;
      if id <= 20 {
          return Err(format!("Charset {} should use -{}/-–{} flag instead", id, id, id));
      }
      Ok((id, parts[1].to_string()))
  }

  impl Cli {
      /// Parse all custom charsets into a HashMap for flexible indexing
      /// Supports unlimited number of charsets (1-20 via flags, 21+ via --charset)
      fn parse_charsets(&self) -> HashMap<usize, String> {
          let mut charsets = HashMap::new();

          // Add numbered charsets 1-20
          if let Some(cs) = &self.charset1 { charsets.insert(1, cs.clone()); }
          if let Some(cs) = &self.charset2 { charsets.insert(2, cs.clone()); }
          if let Some(cs) = &self.charset3 { charsets.insert(3, cs.clone()); }
          if let Some(cs) = &self.charset4 { charsets.insert(4, cs.clone()); }
          if let Some(cs) = &self.charset5 { charsets.insert(5, cs.clone()); }
          if let Some(cs) = &self.charset6 { charsets.insert(6, cs.clone()); }
          if let Some(cs) = &self.charset7 { charsets.insert(7, cs.clone()); }
          if let Some(cs) = &self.charset8 { charsets.insert(8, cs.clone()); }
          if let Some(cs) = &self.charset9 { charsets.insert(9, cs.clone()); }
          if let Some(cs) = &self.charset10 { charsets.insert(10, cs.clone()); }
          if let Some(cs) = &self.charset11 { charsets.insert(11, cs.clone()); }
          if let Some(cs) = &self.charset12 { charsets.insert(12, cs.clone()); }
          if let Some(cs) = &self.charset13 { charsets.insert(13, cs.clone()); }
          if let Some(cs) = &self.charset14 { charsets.insert(14, cs.clone()); }
          if let Some(cs) = &self.charset15 { charsets.insert(15, cs.clone()); }
          if let Some(cs) = &self.charset16 { charsets.insert(16, cs.clone()); }
          if let Some(cs) = &self.charset17 { charsets.insert(17, cs.clone()); }
          if let Some(cs) = &self.charset18 { charsets.insert(18, cs.clone()); }
          if let Some(cs) = &self.charset19 { charsets.insert(19, cs.clone()); }
          if let Some(cs) = &self.charset20 { charsets.insert(20, cs.clone()); }

          // Add extra charsets (21+)
          for (id, chars) in &self.extra_charsets {
              charsets.insert(*id, chars.clone());
          }

          charsets
      }
  }
  ```

- [x] **Add input validation** ✅
  - Check for empty charsets ✅
  - Validate mask pattern syntax ✅
  - Handle edge cases (single position, single char, etc.) ✅
  - Proper error messages for invalid input ✅

- [x] **Error handling** ✅
  - Using `anyhow` for error types ✅
  - Proper error messages ✅
  - Exit codes for different error conditions ✅

### Phase 3: Testing

- [x] **Write Rust unit tests** ✅
  ```rust
  #[cfg(test)]
  mod tests {
      #[test]
      fn test_small_wordlist() { /* ... */ }

      #[test]
      fn test_single_position() { /* ... */ }

      #[test]
      fn test_odometer_overflow() { /* ... */ }
  }
  ```

- [x] **Integration tests** ✅
  - 9 integration tests implemented and passing ✅
  - Validates CLI functionality end-to-end ✅
  - Tests edge cases (empty, single char, repeated charsets) ✅

- [x] **Compatibility validation** ✅
  - Output format verified (one word per line) ✅
  - Line endings correct ✅
  - Character order consistent ✅

- [ ] **Integration with Python test suite** (Phase 4 - Python Integration)
  - Deferred to optional Python bindings phase

### Phase 4: Python Integration (OPTIONAL - Future Enhancement)

> **Note:** Python integration is deferred to a future release. The standalone Rust binary is production-ready and can be used from any language via stdout pipes.

- [ ] **Create Python wrapper function** (Future)
  ```python
  def gen_wordlist_rust(charset):
      """Rust-accelerated wordlist generation (100-200M comb/s)"""
      try:
          from wlgen._wlgen_rust import WordlistIterator
          return WordlistIterator(charset)
      except ImportError:
          raise ImportError("Rust extension not available. Install with: pip install wlgen[rust]")
  ```

- [ ] **Update smart dispatcher** (Future)
  - Add 'rust' method option to `generate_wordlist()`
  - Auto-detect Rust availability
  - Update selection strategy for fast hash use cases

- [ ] **Add to benchmark suite** (Future)
  - New benchmark for `gen_wordlist_rust`
  - Compare against all existing implementations
  - Track performance metrics (throughput, memory, latency)

### Phase 5: Performance Validation

- [x] **Benchmark on real hardware** ✅ (2025-11-07)
  - Achieved: ~41.8M combinations/second (676M words in 16.18s)
  - Measured across different problem sizes
  - Compared with baseline implementations

- [x] **Profile for bottlenecks** ✅ (2025-11-07)
  - Used `cargo flamegraph` with perf
  - Identified hot spots:
    - Core generation: 94.66% (expected)
    - I/O operations: 12.29% (well-optimized)
    - UTF-8 validation overhead in `current_word()` - **primary bottleneck**
  - Optimization opportunities documented

- [x] **Memory efficiency validation** ✅
  - Confirmed O(1) memory usage
  - No memory leaks detected
  - Buffer reuse working correctly

- [x] **Benchmark against maskprocessor** ✅ (2025-11-07)
  - Current results: maskprocessor @ ~144.8M/s vs wlgen-rs @ ~41.8M/s (3.46x gap)
  - Test command: `time ./mp64.bin -1 'ABCDEFGHIJKLMNOPQRSTUVWXYZ' -2 '0123456789' '?1?1?2?2?2?2?2?2' > /dev/null`
  - Status: wlgen-rs at 28.9% of maskprocessor performance
  - **Identified optimizations** could bring performance to ~60-80M/s (see profiling results)
  - Further optimization requires more invasive changes (SIMD, unsafe code, etc.)

### Phase 6: Documentation

- [x] **Update README.md** ✅ (2025-11-07)
  - Updated with latest benchmark results (41.8M words/s)
  - Updated performance comparison table with maskprocessor results (144.8M/s)
  - Added profiling insights section with bottleneck analysis
  - Documented optimization opportunities

- [x] **Update CLAUDE.md** ✅ (2025-11-07)
  - Added comprehensive Rust development commands
  - Documented build process and testing procedures
  - Added pre-commit validation commands
  - Documented CI/CD workflows
  - Added profiling and benchmarking instructions
  - Documented project structure and architecture

- [x] **Inline code documentation** ✅
  - Rustdoc for all public APIs complete
  - Code examples in documentation
  - Performance notes included
  - Architecture explanation documented

- [ ] **Create type stubs** (Phase 4 - Python Integration)
  - Deferred to optional Python bindings phase

### Phase 7: Build & Distribution

> **Note:** As a standalone Rust binary, we skip maturin/Python packaging. Focus on cross-platform Rust builds.

- [x] **CI/CD integration** ✅ (2025-11-07)
  - GitHub Actions workflow for CI (testing, formatting, clippy)
  - Multi-platform builds (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64)
  - Automated testing on push/PR
  - Release automation with pre-built binaries
  - Performance benchmarking in CI

- [x] **Cross-platform testing** ✅ (2025-11-07)
  - CI configured for Linux, macOS, Windows
  - Tests run automatically on all platforms
  - Build targets: Linux (x86_64, aarch64), macOS (x86_64, aarch64), Windows (x86_64)

- [x] **Release automation** ✅ (2025-11-07)
  - Automated GitHub releases on version tags
  - Pre-built binaries for 5 platforms
  - Optional cargo publish to crates.io (requires CARGO_REGISTRY_TOKEN secret)

## Success Criteria

### Performance
- ✓ Achieve 100M+ combinations/second on modern hardware
- ✓ Maintain O(1) memory usage (no memory growth)
- ✓ Low first-word latency (<1ms)

### Compatibility
- ✓ Output identical to `gen_wordlist_iter()`
- ✓ Drop-in replacement API
- ✓ Graceful fallback when Rust unavailable

### Quality
- ✓ All tests pass (Rust + Python)
- ✓ No memory leaks (valgrind clean)
- ✓ Cross-platform builds successful

### Integration
- ✓ Smart dispatcher automatically uses Rust when available
- ✓ Benchmarks show expected speedup
- ✓ Documentation complete and accurate

## Future Enhancements (Post-MVP)

> **Note:** Current CPU performance (164M words/s) already exceeds maskprocessor. For ultra-high performance (500M-1B words/s), see the **GPU Scatter-Gather Wordlist Generator** project.

### CPU Performance Optimizations (COMPLETED ✅)
- [x] **Profile with `perf`/`flamegraph` to identify bottlenecks** ✅
  - Identified UTF-8 validation as 28.7% overhead
  - Identified I/O buffer size as secondary bottleneck
- [x] **Remove UTF-8 validation overhead** ✅ (3.58x speedup)
  - Changed from `writeln!` to direct `write_all()` on bytes
- [x] **Optimize buffer size** ✅ (1.16x additional speedup)
  - Increased from 64KB to 1MB (benchmarked 2MB, found 1MB optimal)
- [x] **Exceed maskprocessor performance** ✅
  - **Final result: 164M words/s (11% faster than maskprocessor's 147.5M words/s)**

### Further CPU Optimizations (Status & Decision)

**Not Planned - Requires unsafe code or incompatible with design:**
- ❌ SIMD optimization (requires unsafe/nightly, ~5-10% gain)
- ❌ Unsafe optimizations: unchecked array access (~5% gain, sacrifices safety)
- ❌ Multi-threaded generation (incompatible with odometer algorithm + stdout streaming)
- ❌ Memory-mapped output (not applicable to stdout, our primary use case)

**Conclusion:** Current performance (164M words/s, 11% faster than maskprocessor) is excellent using pure safe Rust. Further optimization would sacrifice code quality for minimal gains. Focus shifts to useful features instead.

### Features
- [x] Built-in charsets (?l, ?u, ?d, ?s, ?a, ?b from hashcat) ✅ (2025-11-08)
- [ ] Resume from specific position (for distributed workloads)
- [ ] Progress estimation and reporting
- [ ] Streaming compression (gzip, zstd)

## References

- **GPU Scatter-Gather Project**: See `GPU_SCATTER_GATHER_TODO.md` for high-performance GPU implementation (500M-1B words/s)
- **hashcat maskprocessor**: https://github.com/hashcat/maskprocessor
- **PyO3 Guide**: https://pyo3.rs/
- **Maturin**: https://github.com/PyO3/maturin
- **Performance Baseline**: maskprocessor @ ~142M words/s, wlgen-rs @ ~40M words/s (2025-10-15 benchmarks)

## Notes

### Why Rust Over C?
1. **Memory safety** - No buffer overflows, use-after-free, etc.
2. **Better tooling** - Cargo, rustfmt, clippy
3. **Modern language** - Easier to maintain
4. **PyO3 excellence** - Mature, well-documented bindings
5. ⚠️ **Performance reality** - Rust CPU is 3.5x slower than C maskprocessor (~40M vs ~142M/s)

### Why Not Continue CPU Optimization?
- maskprocessor (C) already achieves ~142M/s on CPU
- Diminishing returns from further CPU optimization
- GPU Scatter-Gather project targets 500M-1B words/s (3-7x faster than maskprocessor)
- Novel GPU algorithm is more interesting and impactful

### Project Role
This serves as:
- CPU fallback when GPU unavailable
- Reference implementation for correctness
- Demonstration of Rust vs Python performance (51x speedup)
- Learning project for Rust systems programming

### Hashcat Pipeline Context
```
GPU Scatter-Gather (500M-1B/s) → hashcat stdin → GPU hashing (optimal!)
maskprocessor (142M/s)         → hashcat stdin → GPU hashing (good)
wlgen-rs (40M/s)               → hashcat stdin → GPU hashing (adequate for WPA2)
                                                  ↑
                                               Bottleneck for fast hashes
                                               Perfect for slow hashes
```
