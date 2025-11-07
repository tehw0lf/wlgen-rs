# wlgen-rs

CPU-based Rust wordlist generator achieving **~40M combinations/second**.

> **⚡ For Maximum Performance:** See [GPU Scatter-Gather Wordlist Generator](../gpu-scatter-gather/) for 500M-1B words/s using GPU acceleration and novel algorithms.

## Overview

`wlgen-rs` is a CPU-based Rust wordlist generator that uses an efficient "odometer" algorithm (similar to hashcat's maskprocessor) to generate wordlists. It's designed as a CPU fallback and reference implementation.

### Performance

**Current Performance (2025-11-07 Benchmark):**
- **~41.8M combinations/second** on modern hardware (actual measured: 676M words in 16.18s)
- **51x faster** than Python implementation (~780K/s)
- **0.29x speed** compared to maskprocessor (~144.8M/s)
- **O(1) memory usage** - single buffer reused for all words
- **Zero-copy iteration** - no string allocations per word
- **Fully saturates WPA2-PSK cracking** (911.8 KH/s on RTX 4070) with 46x surplus

### Project Status & Purpose

This project serves as:
- ✅ **CPU fallback** when GPU is unavailable
- ✅ **Reference implementation** for correctness validation
- ✅ **Learning project** demonstrating Rust performance over Python (51x speedup)
- ⚠️ **Not performance-competitive** with maskprocessor (CPU) or GPU implementations

**For high-performance wordlist generation (500M-1B words/s), use the GPU Scatter-Gather project instead.**

### Use Cases

**Suitable for:**
- ✅ WPA2-PSK cracking (44x surplus over hashcat's 911 KH/s on RTX 4070)
- ✅ Slow hash algorithms (bcrypt, scrypt, Argon2)
- ✅ CPU-only environments (no GPU available)
- ✅ Learning Rust systems programming

**Not recommended for:**
- ❌ Fast hash algorithms (MD5, NTLM, SHA-256) - use maskprocessor or GPU Scatter-Gather
- ❌ Maximum performance requirements - use GPU Scatter-Gather project instead

## Installation

### From Source

```bash
git clone https://github.com/tehw0lf/wlgen-rs
cd wlgen-rs
cargo build --release
```

The binary will be available at `target/release/wlgen-rs`.

### Using Cargo

```bash
cargo install wlgen-rs
```

## Usage

### Basic Examples

```bash
# Generate simple 2-character wordlist
wlgen-rs -1 'abc' -2 '123' '?1?2'
# Output: a1, a2, a3, b1, b2, b3, c1, c2, c3

# Pipe to hashcat for WPA2 cracking
wlgen-rs -1 'ABCDEF' -2 '0123456789' '?1?1?2?2?2?2?2?2' | hashcat -m 2500 capture.hccapx

# Complex pattern with multiple charsets
wlgen-rs -1 'ABCDEF' -2 '0123456789' -3 '!@#$' '?1?1?2?2?3'

# Repeated charset for longer patterns
wlgen-rs -1 'abcdefghijklmnopqrstuvwxyz' '?1?1?1?1?1?1?1?1'

# Mix literal characters with placeholders
wlgen-rs -1 'abc' 'prefix?1?1suffix'
```

### Command-Line Options

```
wlgen-rs [OPTIONS] <MASK>

Arguments:
  <MASK>  Mask pattern (e.g., "?1?1?2?2")

Options:
  -1, --custom-charset1 <CS>  Custom charset 1
  -2, --custom-charset2 <CS>  Custom charset 2
  -3, --custom-charset3 <CS>  Custom charset 3
  -4, --custom-charset4 <CS>  Custom charset 4
  -5, --custom-charset5 <CS>  Custom charset 5
  -6, --custom-charset6 <CS>  Custom charset 6
  -7, --custom-charset7 <CS>  Custom charset 7
  -8, --custom-charset8 <CS>  Custom charset 8
  -9, --custom-charset9 <CS>  Custom charset 9
  -h, --help                  Print help
  -V, --version               Print version
```

### Mask Syntax

Mask patterns use `?N` placeholders where `N` is 1-9, referencing custom charsets defined via command-line arguments.

Examples:
- `?1?2` - Two positions using charset 1 and charset 2
- `?1?1?1` - Three positions all using charset 1
- `prefix?1suffix` - Literal characters mixed with charset placeholder

Note: Built-in charsets (like `?l`, `?u`, `?d` from hashcat) are not yet supported in this version.

## Architecture

### Odometer Algorithm

Based on hashcat's maskprocessor implementation, `wlgen-rs` uses an "odometer" pattern:

1. Maintain a single mutable buffer for the current word
2. Increment position indices from right to left (like an odometer)
3. When a position overflows, reset it and carry to the left
4. Continue until all positions overflow

This approach achieves:
- **In-place mutation** - No string allocations per word
- **Single allocation** - Reuse same buffer for all iterations
- **Cache-friendly access** - Sequential memory access patterns
- **Maximum performance** - Minimal overhead per word generated

### Code Example

```rust
use wlgen_rs::WordlistGenerator;

let charsets = vec![
    b"abc".to_vec(),
    b"123".to_vec(),
];

let mut gen = WordlistGenerator::new(charsets);
for word in gen {
    println!("{}", word);
}
// Prints: a1, a2, a3, b1, b2, b3, c1, c2, c3
```

## Development

### Building

```bash
# Debug build
cargo build

# Release build (with optimizations)
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration
```

### Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench -- small_wordlist
```

Expected performance on modern hardware:
- Small wordlists (< 1K): ~200M combinations/second
- Medium wordlists (1K-100K): ~150M combinations/second
- Large wordlists (> 100K): ~100M combinations/second

### Release Configuration

The `Cargo.toml` includes aggressive optimizations for release builds:

```toml
[profile.release]
opt-level = 3        # Maximum optimizations
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
strip = true         # Strip symbols for smaller binary
```

## Performance Comparison

| Tool | Speed (words/s) | Relative Performance | Benchmark Date |
|------|----------------|---------------------|----------------|
| **GPU Scatter-Gather** | 500M-1B | 12-24x faster than wlgen-rs | TBD |
| **maskprocessor (CPU)** | ~144.8M | 3.46x faster than wlgen-rs | 2025-11-07 |
| **wlgen-rs (CPU)** | ~41.8M | Baseline (this project) | 2025-11-07 |
| **Python wlgen** | ~780K | 0.02x (51x slower) | 2025-10-15 |

### Performance Profiling Insights (2025-11-07)

Using `cargo flamegraph`, we identified the time distribution:

**Time Breakdown:**
- **Core wordlist generation: 94.66%**
  - Odometer algorithm and buffer manipulation
  - String/buffer operations: ~45%
  - Character manipulation: ~29%
- **I/O operations (libc): 12.29%**
  - System calls for stdout writing (already well-optimized with buffering)
- **Overhead: ~5%**
  - Program startup, teardown, misc

**Key Bottleneck Identified:**
- UTF-8 validation in `current_word()` called on every iteration
- Each word requires `std::str::from_utf8()` conversion before writing
- **Optimization opportunity:** Write raw bytes directly to eliminate validation overhead

**Potential Improvements:**
1. Write buffer bytes directly without UTF-8 conversion (estimated 10-15% speedup)
2. SIMD operations for character updates (estimated 20-30% speedup)
3. Batch multiple words before writing (estimated 5-10% speedup)

These optimizations could potentially bring wlgen-rs closer to maskprocessor's performance (~60-80M words/s).

## Roadmap

### Current Status (v0.1.0)

- ✅ Core odometer algorithm
- ✅ CLI with maskprocessor-compatible interface
- ✅ Custom charsets (?1-?9)
- ✅ Literal characters in masks
- ✅ Comprehensive test suite
- ✅ Performance benchmarks

### Future Enhancements

#### Phase 2: Advanced Features
- [ ] Built-in charsets (?l, ?u, ?d, ?s from hashcat)
- [ ] Resume from specific position (distributed workloads)
- [ ] Progress reporting and ETA
- [ ] Output to file with optional compression (gzip, zstd)

#### Phase 3: Performance
- [ ] SIMD optimization for character lookups
- [ ] Multi-threaded generation with work stealing
- [ ] Memory-mapped file I/O

#### Phase 4: Integration
- [ ] Python bindings (PyO3)
- [ ] Optional Python package with Rust extension
- [ ] Integration with Python wlgen library

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Run tests before submitting: `cargo test`
2. Run benchmarks to verify performance: `cargo bench`
3. Follow Rust idioms and best practices
4. Add tests for new features
5. Update documentation for API changes

## References

- **hashcat maskprocessor**: https://github.com/hashcat/maskprocessor
- **Python wlgen**: https://github.com/tehw0lf/wlgen
- **Hashcat**: https://hashcat.net/hashcat/

## Author

tehw0lf <tehwolf@protonmail.com>

## Acknowledgments

- Inspired by hashcat's maskprocessor
- Built on the excellent Rust ecosystem (clap, criterion, anyhow)
