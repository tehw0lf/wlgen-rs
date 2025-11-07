# CLAUDE.md

This file provides guidance to Claude Code when working with the wlgen-rs project.

## Project Overview

**wlgen-rs** is a high-performance CPU-based wordlist generator written in Rust, achieving ~41.8M combinations/second. It serves as a CPU fallback and reference implementation for wordlist generation, particularly useful for WPA2-PSK cracking and slow hash algorithms.

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

### Current Benchmarks (2025-11-07)
- **Performance**: ~41.8M words/s (676M words in 16.18s)
- **Comparison**: 28.9% of maskprocessor performance (~144.8M words/s)
- **Memory**: O(1) - single buffer reused for all words
- **I/O**: Buffered writer with 64KB buffer

### Known Bottlenecks (from flamegraph profiling)
1. **UTF-8 validation** (primary bottleneck - 28.7% of time)
   - `current_word()` calls `std::str::from_utf8()` on every iteration
   - **Optimization opportunity**: Write raw bytes directly
2. **I/O operations** (12.29% of time)
   - Already well-optimized with buffering
3. **Core algorithm** (94.66% of time)
   - Expected; this is the actual work being done

### Optimization Opportunities
- Write buffer bytes directly without UTF-8 conversion (10-15% speedup)
- SIMD operations for character updates (20-30% speedup)
- Batch multiple words before writing (5-10% speedup)
- Estimated potential: ~60-80M words/s with these optimizations

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
- ✅ Phase 3: Testing (19 unit tests + 9 integration tests)
- ✅ Phase 5: Performance Validation (benchmarked + profiled)
- ✅ Phase 6: Documentation
- ✅ Phase 7: CI/CD Setup

### Optional/Future Enhancements
- Phase 4: Python Integration (PyO3 bindings)
- Built-in charsets (?l, ?u, ?d, ?s like hashcat)
- Progress reporting and ETA
- Performance optimizations (SIMD, unsafe UTF-8, batching)

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
