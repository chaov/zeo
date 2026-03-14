# zeo - High-Performance AI Agent Execution Engine

zeo is a lightweight, high-performance AI Agent execution engine designed for mobile devices with switchable JavaScript engine support and 50%+ performance improvement over Bun.

## Project Structure

```
zeo/
├── src/
│   ├── zeo/                    # zeo execution engine (Rust)
│   │   ├── core/               # Core execution engine
│   │   ├── engine/             # JavaScript engine abstraction
│   │   ├── resource/           # Resource management
│   │   ├── monitor/            # Performance monitoring
│   │   └── integration/        # Compatibility layers
│   ├── liteclaw/               # LiteClaw AI Agent (TypeScript)
│   └── tests/                  # Test suites
├── benchmarks/                  # Performance benchmarking
├── docs/                        # Documentation
│   ├── research/               # Technical research reports
│   ├── architecture/           # System architecture documents
│   ├── design/                 # Design documents
│   └── product/               # Product planning materials
└── README.md
```

## Prerequisites

### For Building zeo (Rust)

- Rust 1.70+ with cargo
- For QuickJS support: `quick-js` crate
- For V8 support: `rusty_v8` crate (optional)

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### For Building LiteClaw (TypeScript)

- Node.js 22+
- pnpm or npm
- TypeScript 5.0+

Install Node.js dependencies:
```bash
cd src/liteclaw
pnpm install
```

## Building

### Build zeo Engine

```bash
# Build with QuickJS (default)
cargo build --manifest-path src/zeo/Cargo.toml --release

# Build with V8
cargo build --manifest-path src/zeo/Cargo.toml --release --features v8

# Build for mobile
cargo build --manifest-path src/zeo/Cargo.toml --release --features mobile
```

### Build LiteClaw

```bash
cd src/liteclaw
pnpm build
```

## Usage

### Using zeo C API

```c
#include "zeo.h"

// Initialize zeo with QuickJS engine
int result = zeo_init(0); // 0 = QuickJS, 1 = JavaScriptCore, 2 = V8

// Execute JavaScript code
char* output = zeo_execute("console.log('Hello, zeo!');");

// Free the result
zeo_free(output);

// Cleanup
```

### Using zeo from TypeScript

```typescript
import * as zeo from 'zeo';

// Initialize
zeo.init(0); // QuickJS

// Execute code
const result = zeo.execute("1 + 1");
console.log(result); // 2

// Call function
const sum = zeo.call("add", [1, 2]);
console.log(sum); // 3
```

## Performance Goals

| Metric | Bun | zeo Target | Improvement |
|--------|------|------------|-------------|
| Startup Time | ~50ms | <25ms | 50%+ |
| Memory Usage | ~30MB | <15MB | 50%+ |
| Response Time | ~10ms | <5ms | 50%+ |
| Concurrent Agents | Excellent | Superior | 50%+ |

## Features

### zeo Execution Engine

- **Multiple JavaScript Engines**: QuickJS, JavaScriptCore, V8
- **Engine Hot-Switching**: Change engines at runtime
- **Memory Optimization**: Memory pools, object reuse, zero-copy
- **Performance Monitoring**: Real-time metrics and bottleneck detection
- **Mobile Optimization**: Battery-aware scheduling, offline execution
- **OpenClaw Compatibility**: Full OpenClaw protocol support
- **Node.js Compatibility**: Node.js API compatibility layer

### LiteClaw AI Agent

- **Lightweight Agent Framework**: Minimal dependencies and footprint
- **Type-Safe**: Full TypeScript support
- **Tool System**: Extensible tool registry with permission control
- **Session Management**: Efficient session handling with persistence
- **Channel Adapters**: Multiple communication channels
- **LLM Integration**: Multi-provider LLM support with streaming

## Benchmarking

Run the benchmark suite:

```bash
cd benchmarks
./run_benchmarks.sh
```

This will:
1. Run performance tests against zeo and Bun
2. Generate detailed performance reports
3. Verify 50%+ improvement goals
4. Provide optimization recommendations

## Testing

### Unit Tests

```bash
# Rust tests
cargo test --manifest-path src/zeo/Cargo.toml

# TypeScript tests
cd src/liteclaw
pnpm test
```

### Integration Tests

```bash
# Full integration test suite
pnpm test:integration
```

## Development

### Code Style

- Rust: Use `cargo fmt` and `cargo clippy`
- TypeScript: Use `prettier` and `eslint`

### Pre-commit Hooks

```bash
# Install git hooks
git config core.hooksPath .githooks
```

### Code Review Process

1. All code must pass tests
2. Performance benchmarks must meet targets
3. Code must be reviewed by at least one team member
4. Documentation must be updated

## Documentation

- [Technical Research Report](docs/research/technical-research.md)
- [System Architecture](docs/architecture/system-architecture.md)
- [Design Document](docs/design/design-doc.md)
- [Product Planning](docs/product/product-planning.pptx)

## Team

- **Alex** - Chief Architect
- **Sarah** - Product Manager
- **Mike** - Performance Engineer
- **David** - Rust Developer
- **Emily** - JavaScript/TypeScript Developer
- **Lisa** - QA Engineer

## License

MIT

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to the main branch.