# Benchmark Guide

This project includes comprehensive benchmarks using [Criterion.rs](https://github.com/bheisler/criterion.rs) to measure performance.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench throughput
cargo bench --bench compression
cargo bench --bench network

# Run specific tests within a suite
cargo bench transform        # All transform benchmarks
cargo bench image           # All image benchmarks
cargo bench encode          # All encode benchmarks

# Build benchmarks without running (check for errors)
cargo bench --no-run
```

## Benchmark Suites

### 1. Throughput (`benches/throughput.rs`)

Measures message encoding/decoding performance.

**Tests:**
- `transform_encode` - Transform message encoding speed
- `transform_decode` - Transform message decoding speed
- `status_encode` - Status message encoding speed
- `status_decode` - Status message decoding speed
- `image_encode/{size}` - Image encoding by size (64x64 to 1024x1024)
- `image_decode/{size}` - Image decoding by size (64x64 to 1024x1024)

**Example Results:**
```
transform_encode        time:   [307 ns 308 ns 309 ns]
transform_decode        time:   [160 ns 160 ns 160 ns]
image_encode/256x256    time:   [194 µs]
                        thrpt:  [322 MiB/s]
```

**Run:**
```bash
cargo bench --bench throughput
```

### 2. Compression (`benches/compression.rs`)

Measures compression performance with different data sizes and levels.

**Tests:**
- `compress_by_size/{size}` - Compression speed (1KB to 1MB)
- `decompress_by_size/{size}` - Decompression speed (1KB to 1MB)
- `compress_by_level/{level}` - Performance by compression level (fast/default/best)

**Run:**
```bash
cargo bench --bench compression
```

### 3. Network (`benches/network.rs`)

Measures end-to-end network performance with async I/O.

**Tests:**
- `async_roundtrip_status` - Full client-server round-trip latency
- `async_throughput_10_messages` - Throughput for 10 consecutive messages

**Run:**
```bash
cargo bench --bench network
```

## Viewing Results

### Terminal Output

Results are displayed immediately after each benchmark completes:
```
Benchmarking transform_encode
Benchmarking transform_encode: Warming up for 3.0000 s
Benchmarking transform_encode: Collecting 100 samples in estimated 5.0005 s (16M iterations)
Benchmarking transform_encode: Analyzing
transform_encode        time:   [307.13 ns 307.99 ns 308.87 ns]
```

### HTML Reports

Detailed HTML reports with graphs are generated in `target/criterion/`:

```bash
# Open HTML report in browser
open target/criterion/report/index.html

# Or navigate manually
ls target/criterion/
```

Reports include:
- Time vs iteration graphs
- Performance comparison with previous runs
- Statistical analysis (mean, median, std dev)
- Outlier detection

### Comparing Results

Criterion automatically compares new results with previous runs:

```
transform_encode        time:   [307.13 ns 307.99 ns 308.87 ns]
                        change: [-2.5% -1.8% -1.2%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## Performance Baselines

Expected performance on modern hardware (2020+ MacBook Pro / Linux):

| Benchmark | Performance |
|-----------|-------------|
| Transform encode | ~300-400 ns |
| Transform decode | ~150-200 ns |
| Status encode | ~400-500 ns |
| Status decode | ~250-350 ns |
| Image encode (256x256) | ~190-200 µs (320+ MiB/s) |
| Image decode (256x256) | ~190-200 µs (320+ MiB/s) |
| Image encode (1024x1024) | ~3.0-3.2 ms (320+ MiB/s) |
| Compression (1MB) | ~5-10 ms (depends on level) |
| Network round-trip | ~1-2 ms (localhost) |

## Tips

### Fast Iteration During Development

```bash
# Quick check that benchmarks compile
cargo bench --no-run

# Run only specific benchmark (faster)
cargo bench --bench throughput transform_encode
```

### Accurate Measurements

For most accurate results:
1. Close other applications
2. Disable CPU frequency scaling (if possible)
3. Run multiple times and compare
4. Use release builds only (benchmarks use `--release` automatically)

### Saving Baselines

```bash
# Save current results as baseline
cargo bench -- --save-baseline my-baseline

# Compare against saved baseline
cargo bench -- --baseline my-baseline
```

## Troubleshooting

### Benchmarks Taking Too Long

Each benchmark runs for ~5 seconds. To reduce time:
```bash
# Run only specific tests
cargo bench transform
```

### Gnuplot Warning

If you see "Gnuplot not found, using plotters backend", this is normal. Criterion will use its built-in plotting backend (Plotters) instead.

To install Gnuplot (optional):
```bash
# macOS
brew install gnuplot

# Ubuntu/Debian
sudo apt install gnuplot

# Arch
sudo pacman -S gnuplot
```

### Network Benchmarks Fail

Network benchmarks create local servers. If they fail:
- Check if port 0 (random port) is available
- Ensure no firewall is blocking localhost connections
- Try running again (port collision is rare but possible)

## CI/CD Integration

To run benchmarks in CI:

```bash
# Just build (no execution, faster for CI)
cargo bench --no-run

# Or run with smaller sample size
cargo bench -- --sample-size 10
```

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
