# wlgen-rs

CPU-based Rust wordlist generator achieving **~164M combinations/second** - **11% faster than maskprocessor**!

> **⚡ For Maximum Performance:** See [GPU Scatter-Gather Wordlist Generator](../gpu-scatter-gather/) for 500M-1B words/s using GPU acceleration and novel algorithms.

## Overview

`wlgen-rs` is a CPU-based Rust wordlist generator that uses an efficient "odometer" algorithm (similar to hashcat's maskprocessor) to generate wordlists. It's designed as a CPU fallback and reference implementation.

### Performance

**Current Performance (2025-11-07 Optimized):**
- **~164M combinations/second** average, 168M peak (actual measured: 676M words in 4.1s)
- **3.93x faster** than initial implementation (41.8M/s)
- **211x faster** than Python implementation (~780K/s)
- **1.11x faster** than maskprocessor (~147.5M/s) - **11% performance advantage!**
- **O(1) memory usage** - single buffer reused for all words
- **Zero-copy iteration** - no string allocations per word
- **Fully saturates WPA2-PSK cracking** (911.8 KH/s on RTX 4070) with 180x surplus

### Project Status & Purpose

This project serves as:
- ✅ **High-performance CPU generator** - 11% faster than maskprocessor
- ✅ **CPU fallback** when GPU is unavailable
- ✅ **Reference implementation** for correctness validation
- ✅ **Learning project** demonstrating Rust performance over Python (211x speedup)

**For high-performance wordlist generation (500M-1B words/s), use the GPU Scatter-Gather project instead.**

### Use Cases

**Suitable for:**
- ✅ WPA2-PSK cracking (180x surplus over hashcat's 911 KH/s on RTX 4070)
- ✅ Slow hash algorithms (bcrypt, scrypt, Argon2)
- ✅ CPU-only environments (no GPU available)
- ✅ General-purpose CPU wordlist generation (faster than maskprocessor!)
- ✅ Learning Rust systems programming

**Consider GPU alternative for:**
- 💡 Fast hash algorithms requiring 500M+ words/s (MD5, NTLM, SHA-256) - use GPU Scatter-Gather
- 💡 Distributed workloads requiring extreme throughput

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
| **GPU Scatter-Gather** | 500M-1B | 3-6x faster than wlgen-rs | TBD |
| **wlgen-rs (CPU, optimized)** | ~164M | **11% faster than maskprocessor!** | 2025-11-07 |
| **maskprocessor (CPU)** | ~147.5M | 0.90x (baseline C implementation) | 2025-11-07 |
| **wlgen-rs (CPU, initial)** | ~41.8M | 0.25x (before optimizations) | 2025-11-07 |
| **Python wlgen** | ~780K | 0.005x (211x slower) | 2025-10-15 |

### Performance Optimizations Applied (2025-11-07)

Through systematic profiling and optimization, we achieved a **3.93x speedup** over the initial implementation:

**Optimization 1: Remove UTF-8 Validation (3.58x speedup)**
- **Problem**: `writeln!` macro called `std::str::from_utf8()` on every word
- **Profiling**: Flamegraph showed 28.7% of time in UTF-8 validation
- **Solution**: Write buffer bytes directly using `write_all()`
- **Result**: 41.8M → 146.2M words/s

**Optimization 2: Increase Buffer Size (1.16x additional speedup)**
- **Problem**: Small 64KB buffer caused frequent syscalls (shown as 12.29% libc overhead)
- **Solution**: Increase BufWriter capacity to 1MB (16x larger)
- **Benchmarking**: Tested 64KB, 1MB, 2MB - found 1MB optimal
- **Result**: 146.2M → 164.3M words/s

**Final Performance:**
- **164.3M words/s average** (676M words in 4.1s)
- **11.4% faster than maskprocessor** (147.5M words/s)
- **Pure safe Rust** - no unsafe code required!

**Remaining Bottlenecks:**
- Core algorithm: ~56% (libc write operations)
- Buffer/memory operations: ~38%
- Overhead: ~6%

Further optimization would require unsafe code (SIMD, unchecked access) for ~5-10% gains, but at the cost of memory safety guarantees.

## Roadmap

### Current Status (v0.1.0)

- ✅ Core odometer algorithm
- ✅ CLI with maskprocessor-compatible interface
- ✅ Custom charsets (?1-?9)
- ✅ Literal characters in masks
- ✅ Comprehensive test suite (19 unit + 9 integration + 6 doc tests)
- ✅ Performance benchmarks
- ✅ **Performance optimizations** (3.93x speedup, 11% faster than maskprocessor)
  - ✅ UTF-8 validation removal
  - ✅ Buffer size optimization
  - ✅ Flamegraph profiling and bottleneck analysis

### Future Enhancements

#### Phase 2: Advanced Features
- [ ] Built-in charsets (?l, ?u, ?d, ?s from hashcat)
- [ ] Resume from specific position (distributed workloads)
- [ ] Progress reporting and ETA
- [ ] Output to file with optional compression (gzip, zstd)

#### Phase 3: Further Performance (Optional)
- [ ] SIMD optimization for character lookups (requires unsafe/nightly)
- [ ] Multi-threaded generation with work stealing
- [ ] Memory-mapped file I/O
- [ ] Unsafe optimizations (unchecked array access)

Note: Current performance (164M words/s) already exceeds maskprocessor. Further optimization would require sacrificing memory safety for diminishing returns (~5-10% gains).

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
