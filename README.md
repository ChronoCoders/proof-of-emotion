# Proof of Emotion (POE) Consensus

## Overview

Proof of Emotion is a next-generation consensus mechanism that combines traditional Byzantine Fault Tolerance with real-time biometric validation and emotional state verification. By requiring validators to maintain authentic emotional responses during consensus participation, POE creates a unique blend of cryptographic security and human authenticity.

## Key Features

- **Byzantine Fault Tolerance**: 67% honest validator requirement with robust defense against malicious actors
- **Biometric Validation**: Real-time heart rate, stress level, and focus monitoring
- **Emotional Scoring**: Dynamic validator fitness based on physiological authenticity
- **High Performance**: Optimized for 1000+ validators with parallel processing
- **Cryptographic Security**: ECDSA signatures, Merkle proofs, and optional ZK-proof support
- **Economic Incentives**: Stake-weighted rewards with emotional multipliers
- **Fault Tolerance**: Fork detection, checkpoint system, and crash recovery
- **Observability**: Prometheus metrics, health checks, and structured logging
- **Comprehensive Testing**: 63 passing tests including Byzantine fault detection, load testing, and property-based tests

## Quick Start

```bash
# Clone the repository
git clone https://github.com/ChronoCoders/proof-of-emotion.git
cd proof-of-emotion

# Run tests
cargo test

# Run examples
cargo run --example basic_consensus
cargo run --example multi_validator
cargo run --example staking_rewards
```

## Architecture

```
┌────────────────────────────────────────────────────┐
│         Proof of Emotion Engine                    │
├────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────────┐            │
│  │  Biometric   │  │    Consensus     │            │
│  │  Validation  │──│     Protocol     │            │
│  └──────────────┘  └──────────────────┘            │
│         │                   │                      │
│  ┌──────────────┐  ┌──────────────────┐            │
│  │  Emotional   │  │  Cryptographic   │            │
│  │   Scoring    │──│    Security      │            │
│  └──────────────┘  └──────────────────┘            │
│         │                   │                      │
│  ┌──────────────┐  ┌──────────────────┐            │
│  │  Byzantine   │  │  Fork Detection  │            │
│  │  Detection   │──│  & Resolution    │            │
│  └──────────────┘  └──────────────────┘            │
│         │                   │                      │
│  ┌──────────────┐  ┌──────────────────┐            │
│  │  Checkpoint  │  │    Observability │            │
│  │   System     │──│  (Metrics/Health)│            │
│  └──────────────┘  └──────────────────┘            │
│         │                   │                      │
│  ┌─────────────────────────────────────────────┐   │
│  │      Economic Incentives Layer              │   │
│  │  (Staking, Rewards, Slashing)               │   │
│  └─────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────┘
```

## Core Components

### 1. Consensus Engine
- Committee-based block proposal and voting
- Byzantine fault tolerance with 67% threshold
- Configurable epoch duration and timeouts
- Automatic validator selection and rotation

### 2. Biometric Validation
- Real-time heart rate monitoring (60-100 BPM optimal)
- Stress level assessment (low stress required)
- Focus tracking (sustained attention verification)
- Anti-spoofing with variance and trend analysis

### 3. Emotional Scoring
- Multi-dimensional fitness calculation
- Adaptive thresholds (75% minimum by default)
- Historical trend analysis
- Reward multipliers for consistent performance

### 4. Economic Layer
- Minimum stake requirement (10,000 POE)
- Delegated staking support
- Commission-based rewards
- Graduated slashing for violations

### 5. Fault Tolerance System
- **Fork Detection**: Identifies competing blocks at same height
- **Fork Resolution**: Emotional score-based fork choice rule
- **Checkpoint System**: Byzantine fault-tolerant state snapshots (67% validator signatures required)
- **Crash Recovery**: Restore from checkpoints with block replay and state validation

### 6. Observability & Monitoring
- **Prometheus Metrics**: 19 metric types including counters, gauges, histograms, and vector metrics
- **Health Checks**: Liveness and readiness probes with degraded/critical state detection
- **Structured Logging**: Rich contextual fields for consensus phases, Byzantine events, and performance metrics

## Performance Benchmarks

Configuration: 21 validators, 30s epochs, 1000 tx/block

| Metric | Value |
|--------|-------|
| Block Time | 30 seconds |
| Transactions per Block | 1,000 |
| Actual TPS (Measured) | ~33 (1000 tx/block, 30s epochs) |
| Theoretical Max TPS | ~300 (optimized batching, 10s epochs) |
| Committee Selection | 150ms |
| Block Validation | 200ms |
| Voting Phase | 8s |
| Finalization | 2s |
| Max Validators | 1,000+ |
| Memory per Node | < 500MB |
| Byzantine Tolerance | 33% malicious nodes |
| Emotional Threshold | 75% (configurable) |

**To improve TPS:**
- Reduce epoch duration to 10s → 100 TPS
- Increase tx/block to 3000 → 100 TPS
- Parallel validation → 2x improvement

## Installation

### Prerequisites
- Rust 1.91 or higher
- Cargo package manager

### Build from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/ChronoCoders/proof-of-emotion.git
cd proof-of-emotion
cargo build --release
```

### Run Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests
cargo test --test integration_tests

# Specific module tests
cargo test fork
cargo test checkpoint
cargo test byzantine
cargo test metrics
cargo test health

# With output
cargo test -- --nocapture

# Linting
cargo clippy --all-targets --all-features
```

## Usage Examples

### Basic Consensus

```rust
use proof_of_emotion::{ProofOfEmotionEngine, ConsensusConfig, EmotionalValidator};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create consensus configuration
    let config = ConsensusConfig::default();
    let engine = Arc::new(ProofOfEmotionEngine::new(config)?);

    // Register validators
    let validator = EmotionalValidator::new("validator-1", 10_000)?;
    engine.register_validator(validator).await?;

    // Start consensus
    Arc::clone(&engine).start().await?;

    Ok(())
}
```

### Staking and Rewards

```rust
use proof_of_emotion::staking::EmotionalStaking;

fn main() -> anyhow::Result<()> {
    let staking = EmotionalStaking::new(10_000);

    // Register validator
    staking.register_validator(
        "alice".to_string(),
        "poe1alice".to_string(),
        15_000,  // stake
        5,       // 5% commission
    )?;

    // Delegate stake
    staking.delegate_stake(
        "alice".to_string(),
        "delegator1".to_string(),
        5_000,
        21 * 24 * 60 * 60, // 21 day lock
    )?;

    Ok(())
}
```

### Crash Recovery

```rust
use proof_of_emotion::ProofOfEmotionEngine;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = Arc::new(ProofOfEmotionEngine::new(config)?);

    // Recover from crash using checkpoints
    engine.recover_from_crash().await?;

    // Continue normal operation
    Arc::clone(&engine).start().await?;

    Ok(())
}
```

### Health Monitoring

```rust
use proof_of_emotion::health::HealthStatus;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = Arc::new(ProofOfEmotionEngine::new(config)?);

    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    // Check health status
    let health = HealthStatus::from_consensus(&engine, start_time).await;

    println!("Status: {:?}", health.status);
    println!("Consensus Strength: {}%", health.consensus_strength);
    println!("Active Validators: {}", health.active_validators);

    if !health.is_healthy() {
        println!("Issues: {}", health.status_message());
    }

    Ok(())
}
```

### Prometheus Metrics

```rust
use proof_of_emotion::metrics::{create_default_registry, PrometheusMetrics};
use std::sync::Arc;

fn main() -> anyhow::Result<()> {
    // Create Prometheus registry with all POE metrics
    let (registry, metrics) = create_default_registry()?;

    // Update metrics from consensus engine
    let consensus_metrics = engine.get_metrics().await;
    metrics.update_from_consensus(&consensus_metrics);

    // Record custom events
    metrics.record_byzantine_event("double_signing", "validator-1");
    metrics.update_validator_stake("validator-1", 50000);

    // Export metrics (integrate with Prometheus server)
    let metric_families = registry.gather();

    Ok(())
}
```

## Testing

The project includes comprehensive testing:

- **63 unit tests**: Individual component testing
- **19 integration tests**: Full consensus flow validation
- **14 property tests**: Invariant verification
- **5 load tests**: Performance benchmarking (ignored by default)
- **1 doctest**: API documentation verification
- **3 examples**: Live demonstrations

```bash
# Quick test (recommended for CI)
./run_all_tests.sh    # Unix/Linux/Mac
run_all_tests.bat     # Windows

# Individual test suites
cargo test --lib                      # Unit tests
cargo test --test integration_tests   # Integration tests
cargo test --test property_tests      # Property-based tests
cargo test --doc                      # Doctests

# With verbose output
cargo test -- --show-output
```

## Benchmarking

```bash
# Run all benchmarks
cargo bench

# Specific benchmarks
cargo bench consensus
cargo bench emotional_validation
cargo bench staking
```

## Monitoring and Observability

### Structured Logging

All critical operations emit structured logs with rich contextual fields:

```rust
// Validator registration
info!(
    validator_id = "validator-1",
    stake = 10000,
    total_validators = 5,
    "Validator registered"
);

// Byzantine detection
error!(
    validator_id = "malicious-validator",
    block_height = 42,
    event_type = "double_signing",
    "Byzantine behavior detected"
);
```

### Health Endpoints

- **Liveness**: Basic process health check
- **Readiness**: Full system readiness (consensus strength, participation, etc.)
- **Health Status**: Detailed health report with issues and recommendations

### Prometheus Metrics

Available metrics include:
- `poe_blocks_finalized_total` - Total finalized blocks
- `poe_byzantine_detected_total` - Byzantine behaviors detected
- `poe_consensus_strength` - Current consensus strength (0-100)
- `poe_active_validators` - Number of active validators
- `poe_epoch_duration_seconds` - Epoch duration distribution
- Plus 14 additional metrics for comprehensive monitoring

## Documentation

- [Quick Start Guide](QUICKSTART.md) - Get started in 5 minutes
- [Testing Guide](TESTING_GUIDE.md) - Comprehensive testing strategy
- [Windows Notes](WINDOWS_NOTES.md) - Windows-specific instructions
- [Project Summary](PROJECT_SUMMARY.md) - Technical deep dive
- [Security Policy](SECURITY.md) - Security considerations and responsible disclosure

## Security Considerations

### Current Status
- **NOT PRODUCTION READY**
- **NOT AUDITED**
- **EXPERIMENTAL RESEARCH**

### Known Security Issues

1. **Biometric Privacy**
   - Biometric data hashes stored on-chain
   - Potential correlation attacks
   - Consider ZK-proofs for production

2. **Centralization Risks**
   - Biometric devices can be compromised
   - No decentralized biometric verification
   - Single point of failure in device trust

3. **Attack Vectors**
   - Nothing-at-stake (mitigated by stake locking)
   - Biometric spoofing (requires hardware security)
   - Long-range attacks (mitigated by checkpointing)
   - Eclipse attacks (need peer diversity)

4. **Economic Assumptions**
   - Emotional scoring is gameable
   - Validators may optimize for rewards over authenticity
   - Needs formal game theory analysis

### Before Production

- [ ] External security audit ($50k-100k)
- [ ] Formal verification of consensus safety
- [ ] Penetration testing
- [ ] Economic analysis
- [ ] Real biometric hardware integration
- [ ] Bug bounty program

### Responsible Disclosure

If you discover a security vulnerability, please email:
**security@chronocoders.example** (do NOT open public issues)

For more details, see [SECURITY.md](SECURITY.md).

## Roadmap

### Phase 1: Core Implementation (COMPLETED)
- [x] Byzantine fault-tolerant consensus
- [x] Biometric validation system
- [x] Emotional scoring algorithm
- [x] Economic incentives layer
- [x] Fork detection and resolution
- [x] Checkpoint system for crash recovery
- [x] Prometheus metrics and health checks
- [x] Structured logging
- [x] Comprehensive test suite (63 tests)

### Phase 2: Network Layer (Q1 2026)
- [ ] P2P networking with libp2p
- [ ] Block gossiping protocol
- [ ] Peer discovery mechanism
- [ ] Persistent storage (sled/RocksDB)

### Phase 3: Testnet (Q2 2026)
- [ ] Private testnet deployment
- [ ] Public testnet launch
- [ ] Block explorer
- [ ] Monitoring dashboard

### Phase 4: Production (Q3-Q4 2026)
- [ ] Security audit
- [ ] Mainnet launch
- [ ] Governance mechanism
- [ ] Mobile wallet support

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Write tests for new features
- Ensure `cargo clippy` passes with zero warnings
- Format code with `cargo fmt`
- Update documentation as needed
- Use structured logging with contextual fields
- Add Prometheus metrics for new components

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [Tokio](https://tokio.rs/)
- Inspired by Byzantine Fault Tolerance research
- Cryptography powered by [secp256k1](https://github.com/rust-bitcoin/rust-secp256k1)
- Metrics powered by [Prometheus](https://prometheus.io/)

## Contact

- GitHub Issues: [Report bugs or request features](https://github.com/ChronoCoders/proof-of-emotion/issues)
- Discussions: [Join the conversation](https://github.com/ChronoCoders/proof-of-emotion/discussions)

---

**IMPORTANT DISCLAIMER**

This is an **experimental consensus mechanism for research purposes only**.

- **NOT audited** for production use
- **NOT tested** with real biometric hardware
- **NOT secure** against all known attack vectors
- **NOT suitable** for handling real value or sensitive data

Use at your own risk. See [SECURITY.md](SECURITY.md) for detailed security considerations.
