# CLAUDE.md

This file provides guidance to Claude Code when working with the wlgen-rs project.

## Project Overview

**wlgen-rs** is a high-performance CPU-based wordlist generator written in Rust, achieving **~164M combinations/second** - **11% faster than maskprocessor**! It serves as a high-performance CPU solution and reference implementation for wordlist generation, particularly useful for WPA2-PSK cracking and slow hash algorithms.

### Technology Stack
- **Language**: Rust 2021 Edition
- **CLI Framework**: clap v4.5 with derive macros
- **Error Handling**: anyhow v1.0
- **Build System**: Cargo with aggressive release optimizations
- **Testing**: Built-in Rust testing + integration tests
- **CI/CD**: GitHub Actions for multi-platform builds

### Project Structure
```
wlgen-rs/
├── src/
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # Library exports
│   ├── cli.rs          # Command-line argument parsing
│   └── generator.rs    # Core wordlist generation algorithm
├── tests/
│   └── integration.rs  # Integration tests
├── benches/
│   └── wordlist_bench.rs # Performance benchmarks
├── .github/
│   └── workflows/      # CI/CD workflows
├── Cargo.toml          # Project configuration
└── README.md           # User documentation
```

## Development Commands

### Building

```bash
# Debug build (for development)
cargo build

# Release build (optimized for performance)
cargo build --release

# The optimized binary will be at:
# target/release/wlgen-rs
```

### Testing

```bash
# Run all tests (unit + integration + doc tests)
cargo test

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration

# Run tests with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_cli_simple_wordlist
```

### Running

```bash
# Run with cargo (debug build)
cargo run -- -1 'abc' -2 '123' '?1?2'

# Run release build directly
./target/release/wlgen-rs -1 'abc' -2 '123' '?1?2'

# Pipe to file
./target/release/wlgen-rs -1 'ABCDEF' -2 '0123456789' '?1?1?2?2?2?2?2?2' > wordlist.txt

# Pipe to hashcat
./target/release/wlgen-rs -1 'ABCDEF' -2 '0123456789' '?1?1?2?2?2?2?2?2' | hashcat -m 2500 capture.hccapx
```

### Benchmarking

```bash
# Run criterion benchmarks
cargo bench

# Run simple performance test
time ./target/release/wlgen-rs -1 'ABCDEFGHIJKLMNOPQRSTUVWXYZ' -2 '0123456789' '?1?1?2?2?2?2?2?2' > /dev/null

# Count generated words
./target/release/wlgen-rs -1 'abc' -2 '123' '?1?2' | wc -l
```

### Profiling

```bash
# Install flamegraph (one-time)
cargo install flamegraph

# Generate flamegraph profile
cargo flamegraph --bin wlgen-rs -- -1 'ABCDEFGHIJKLMNOPQRSTUVWXYZ' -2 '0123456789' '?1?1?2?2?2?2' > /dev/null

# Output: flamegraph.svg (open in browser)
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt --check

# Run clippy linter
cargo clippy

# Run clippy with all warnings as errors
cargo clippy --all-targets --all-features -- -D warnings

# Generate documentation
cargo doc --open
```

## Pre-commit Validation

**IMPORTANT**: Always run these commands before committing:

```bash
cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test && cargo build --release
```

Or more concisely:
```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

## Performance Characteristics

### Current Benchmarks (2025-11-07 - After Optimization)
- **Performance**: ~164M words/s average, 168M peak (676M words in 4.1s)
- **Comparison**: **111% of maskprocessor performance** (~147.5M words/s) - **11% faster!**
- **vs Initial**: 3.93x speedup from 41.8M words/s
- **Memory**: O(1) - single buffer reused for all words
- **I/O**: Buffered writer with 1MB buffer (optimized from 64KB)

### Optimization History
**Initial Performance (2025-11-07)**: 41.8M words/s

**Optimization 1: Remove UTF-8 Validation** (3.58x speedup)
- **Problem**: `writeln!` called `std::str::from_utf8()` on every word (28.7% overhead)
- **Solution**: Write buffer bytes directly using `write_all()`
- **Result**: 41.8M → 146.2M words/s

**Optimization 2: Increase Buffer Size** (1.16x additional speedup)
- **Problem**: Small 64KB buffer caused frequent syscalls (12.29% libc overhead)
- **Solution**: Increase BufWriter capacity to 1MB
- **Benchmarking**: Tested 64KB, 1MB, 2MB - found 1MB optimal (2MB slower due to cache)
- **Result**: 146.2M → 164.3M words/s

**Final Achievement**: **11% faster than maskprocessor using pure safe Rust!**

### Further Optimization Opportunities (Diminishing Returns)
- SIMD operations for character updates (~5-10% gain, requires unsafe/nightly)
- Multi-threaded generation (~20-30% gain, complex for stdout streaming)
- Unsafe optimizations: unchecked array access (~5% gain, sacrifices safety)

## Algorithm Overview

**Odometer Pattern** (similar to hashcat's maskprocessor):
1. Maintain single mutable buffer for current word
2. Increment position indices from right to left (like odometer)
3. When position overflows, reset and carry left
4. Continue until all positions overflow

**Key Implementation Details**:
- Zero allocations per word (reuses buffer)
- Cache-friendly sequential memory access
- Buffered I/O to minimize syscalls
- No string allocations during iteration

## CI/CD

### GitHub Actions Workflows

**ci.yml** - Runs on every push/PR:
- Test suite on Linux, macOS, Windows
- Rustfmt formatting check
- Clippy linting
- Release builds for all platforms
- Performance benchmark

**release.yml** - Runs on version tags (e.g., `v0.1.0`):
- Creates GitHub release
- Builds binaries for all platforms (5 targets)
- Uploads pre-built binaries as release assets
- Optionally publishes to crates.io

### Creating a Release

```bash
# Tag the release
git tag v0.1.0
git push origin v0.1.0

# GitHub Actions will automatically:
# - Create a GitHub release
# - Build binaries for all platforms
# - Upload binaries as release assets
```

## Common Tasks

### Adding a New Feature

1. Write tests first (TDD approach)
2. Implement feature in appropriate module
3. Update documentation
4. Run validation: `cargo fmt && cargo clippy && cargo test`
5. Commit with descriptive message

### Debugging

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo run -- -1 'abc' -2 '123' '?1?2'

# Run with full backtrace
RUST_BACKTRACE=full cargo run -- -1 'abc' -2 '123' '?1?2'

# Use rust-gdb for debugging
rust-gdb target/debug/wlgen-rs
```

### Comparing with maskprocessor

```bash
# Benchmark wlgen-rs
time ./target/release/wlgen-rs -1 'ABCDEFGHIJKLMNOPQRSTUVWXYZ' -2 '0123456789' '?1?1?2?2?2?2?2?2' > /dev/null

# Benchmark maskprocessor
time ./maskprocessor/mp64.bin -1 'ABCDEFGHIJKLMNOPQRSTUVWXYZ' -2 '0123456789' '?1?1?2?2?2?2?2?2' > /dev/null
```

## Project Status

### Completed Phases
- ✅ Phase 1: Repository & Project Setup
- ✅ Phase 2: Core Implementation (CLI + Generator)
- ✅ Phase 3: Testing (19 unit tests + 9 integration tests + 6 doc tests)
- ✅ Phase 5: Performance Validation & Optimization
  - Benchmarked initial implementation (41.8M words/s)
  - Profiled with flamegraph to identify bottlenecks
  - Optimized UTF-8 validation (3.58x speedup)
  - Optimized buffer size (1.16x additional speedup)
  - **Final: 164M words/s (11% faster than maskprocessor)**
- ✅ Phase 6: Documentation (updated with optimization details)
- ✅ Phase 7: CI/CD Setup (GitHub Actions workflows)

### Optional/Future Enhancements
- Phase 4: Python Integration (PyO3 bindings)
- Built-in charsets (?l, ?u, ?d, ?s like hashcat)
- Progress reporting and ETA
- Further optimizations (SIMD, multi-threading, unsafe code) - diminishing returns

## References

- **hashcat maskprocessor**: https://github.com/hashcat/maskprocessor
- **Rust Book**: https://doc.rust-lang.org/book/
- **Cargo Book**: https://doc.rust-lang.org/cargo/
- **clap Documentation**: https://docs.rs/clap/latest/clap/

## Notes

### Memory Safety
All code is safe Rust - no `unsafe` blocks. Performance optimizations that require `unsafe` are deferred to future enhancements.

### Unicode Support
Currently assumes UTF-8 charsets. Non-UTF-8 input will panic during validation. Consider adding explicit UTF-8 validation at CLI level if needed.

### Platform Compatibility
- Linux: Fully supported and tested
- macOS: Supported (both x86_64 and Apple Silicon)
- Windows: Supported (x86_64)
- Cross-compilation: Configured for Linux aarch64

### Performance Trade-offs
This is a CPU implementation optimized for readability and safety. For maximum performance (500M-1B words/s), see the GPU Scatter-Gather Wordlist Generator project.
