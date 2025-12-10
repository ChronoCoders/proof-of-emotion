# Proof of Emotion - Quick Start Guide

## Prerequisites

1. **Install Rust** (if not already installed):
   - Windows: Download from https://rustup.rs/ and run the installer
   - Linux/Mac: Run `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

2. **Verify installation**:
   ```bash
   rustc --version
   cargo --version
   ```

## Building the Project

### Option 1: Use the build script

**Windows:**
```cmd
build.bat
```

**Linux/Mac:**
```bash
chmod +x build.sh
./build.sh
```

### Option 2: Manual build

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Build examples
cargo build --examples
```

## Running Examples

### Basic Consensus Example
```bash
cargo run --example basic_consensus
```

This will:
- Register 5 validators (Alice, Bob, Charlie, Diana, Eve)
- Start consensus with 30-second epochs
- Monitor for 90 seconds (3 epochs)
- Display real-time metrics

### Multi-Validator Example
```bash
cargo run --example multi_validator
```

This demonstrates:
- 20 validators with varying stakes
- Parallel consensus processing
- Real-time monitoring for 2 minutes

### Staking and Rewards Example
```bash
cargo run --example staking_rewards
```

This shows:
- Validator registration with different commissions
- Stake delegation
- Reward distribution based on emotional scores
- Slashing for poor behavior

## Project Structure

```
Proof of Emotion/
├── Cargo.toml              # Project configuration
├── README.md               # Full documentation
├── QUICKSTART.md          # This file
├── build.sh / build.bat   # Build scripts
├── src/
│   ├── lib.rs             # Library entry point
│   ├── error.rs           # Error types
│   ├── types.rs           # Core types (Block, Transaction, Vote)
│   ├── crypto.rs          # Cryptography (ECDSA, signatures)
│   ├── biometric.rs       # Biometric validation
│   ├── consensus.rs       # Main consensus engine
│   ├── staking.rs         # Staking and rewards
│   └── utils.rs           # Utility functions
├── examples/              # Example programs
├── tests/                 # Integration tests
└── benches/              # Benchmarks

```

## Key Features

- **POE Token**: The consensus uses POE (Proof of Emotion) tokens
- **Minimum Stake**: 10,000 POE to become a validator
- **Emotional Threshold**: 75% minimum emotional fitness required
- **Byzantine Threshold**: 67% honest validators required
- **Epoch Duration**: 30 seconds per consensus round
- **Committee Size**: 21 validators per round (configurable)

## Common Commands

```bash
# Build for production
cargo build --release

# Run all tests
cargo test

# Run specific test
cargo test test_validator_registration

# Generate documentation
cargo doc --open

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Clean build artifacts
cargo clean
```

## Troubleshooting

### Build Errors

1. **Missing dependencies**: Run `cargo update`
2. **Compiler errors**: Make sure you have the latest Rust: `rustup update`
3. **Permission errors**: On Unix, run `chmod +x build.sh`

### Test Failures

If tests timeout, increase the timeout in the test configuration or run with:
```bash
cargo test -- --test-threads=1
```

### Performance Issues

For faster builds during development:
```bash
cargo build  # Debug build (faster compile, slower runtime)
```

For production:
```bash
cargo build --release  # Optimized build
```

## Next Steps

1. Read the full README.md for detailed documentation
2. Explore the examples in the `examples/` directory
3. Check the source code in `src/` to understand the implementation
4. Run the integration tests: `cargo test --test integration_tests`
5. Modify the examples to experiment with different configurations

## Configuration

You can customize consensus parameters:

```rust
let config = ConsensusConfig {
    epoch_duration: 30_000,      // 30 seconds
    emotional_threshold: 75,      // 75% minimum
    byzantine_threshold: 67,      // 67% BFT
    committee_size: 21,           // 21 validators
    minimum_stake: 10_000,        // 10k POE
    voting_timeout: 8_000,
    proposal_timeout: 10_000,
    finality_timeout: 2_000,
};
```

## Getting Help

- Check the README.md for comprehensive documentation
- Look at examples for working code
- Review the inline documentation: `cargo doc --open`
- Check the tests for usage examples

## License

MIT License - See LICENSE file for details

---

Built with ❤️ and Rust by ChronoCoders
