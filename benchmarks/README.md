# Zeo Performance Benchmarks

This directory contains the comprehensive benchmark framework for comparing Zeo runtime performance against Bun.

## Structure

```
benchmarks/
├── config.toml              # Benchmark configuration
├── src/                     # Rust benchmark framework
│   ├── main.rs             # Entry point
│   ├── lib.rs              # Core benchmark logic
│   ├── reporting.rs        # Report generation
│   └── monitoring.rs       # System monitoring
├── tests/                   # Test cases
│   ├── javascript_execution.js
│   ├── modules/
│   ├── io/
│   ├── network/
│   └── ai/
├── scripts/                 # Utility scripts
├── reports/                 # Generated reports
└── README.md
```

## Usage

### Quick Start

```bash
# Run all benchmarks
./run_benchmarks.sh

# Run specific test category
cargo run --release -- --scenario javascript_execution

# Run mobile benchmarks
./scripts/run_mobile_benchmarks.sh --ios
./scripts/run_mobile_benchmarks.sh --android
```

### Configuration

Edit `config.toml` to customize:

- **Targets**: Runtime executables to test
- **Metrics**: Performance metrics to collect
- **Scenarios**: Test cases and iterations
- **Reporting**: Output formats and goals

### Test Scenarios

1. **Startup Time**: Cold start performance
2. **JavaScript Execution**: Computational benchmarks
3. **Module Loading**: Import and resolution speed
4. **File I/O**: Read/write operations
5. **Network Requests**: HTTP performance
6. **AI Agent Execution**: AI workload simulation

## Performance Goals

Zeo targets **50%+ performance improvement** over Bun across all scenarios.

### Success Criteria

- ✅ Startup time < 50ms
- ✅ Memory usage < 100MB
- ✅ Execution speed 1.5x faster than Bun
- ✅ CPU usage < 80%
- ✅ Battery consumption < 50mWh (mobile)

## Reports

After running benchmarks, reports are generated in `./reports/`:

- `benchmark_report.json` - Raw data
- `benchmark_report.md` - Markdown summary
- `benchmark_report.html` - Interactive HTML report

## Continuous Monitoring

```bash
# Run benchmarks every hour
./scripts/continuous_monitoring.sh 3600

# Generate trend analysis
./scripts/generate_trend_report.sh
```

## Mobile Testing

Mobile benchmarks test battery efficiency and resource usage:

```bash
# iOS device testing
./scripts/run_mobile_benchmarks.sh --ios

# Android device testing
./scripts/run_mobile_benchmarks.sh --android --device <device_id>
```

## Contributing

When adding new benchmarks:

1. Create test file in appropriate `tests/` subdirectory
2. Add scenario configuration to `config.toml`
3. Implement benchmark logic in `src/lib.rs`
4. Update this README

## License

Part of the Zeo project.