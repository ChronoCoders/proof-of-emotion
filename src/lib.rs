//! # Proof of Emotion (POE) Consensus
//!
//! A Byzantine fault-tolerant consensus mechanism that combines traditional blockchain
//! consensus with real-time biometric validation and emotional state verification.
//!
//! ## Features
//!
//! - **Byzantine Fault Tolerance**: 67% honest validator requirement
//! - **Biometric Validation**: Real-time heart rate, stress, and focus monitoring
//! - **Emotional Scoring**: Dynamic validator fitness based on physiological authenticity
//! - **High Performance**: Optimized for 1000+ validators with parallel processing
//! - **Cryptographic Security**: ECDSA signatures, Merkle proofs, ZK-proof support
//! - **Economic Incentives**: Stake-weighted rewards with emotional multipliers
//!
//! ## Quick Start
//!
//! ```rust
//! use proof_of_emotion::{ProofOfEmotionEngine, ConsensusConfig, EmotionalValidator};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ConsensusConfig::default();
//!     let mut engine = ProofOfEmotionEngine::new(config)?;
//!     
//!     // Register validators
//!     let validator = EmotionalValidator::new("validator-1", 10000)?;
//!     engine.register_validator(validator).await?;
//!     
//!     // Start consensus
//!     engine.start().await?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod biometric;
pub mod consensus;
pub mod crypto;
pub mod error;
pub mod staking;
pub mod types;
pub mod utils;

// Re-exports for convenience
pub use consensus::{
    ConsensusConfig, ConsensusMetrics, ConsensusRound, ConsensusState, 
    ProofOfEmotionEngine, RoundPhase
};
pub use biometric::{BiometricDevice, BiometricReading, EmotionalProfile, EmotionalValidator};
pub use crypto::{EmotionalProof, KeyPair, Signature};
pub use error::{ConsensusError, Result};
pub use staking::{EmotionalStaking, RewardDistribution, SlashingEvent, Validator};
pub use types::{Block, BlockHeader, Transaction, Vote, VotingResult};

/// POE token ticker symbol
pub const TICKER: &str = "POE";

/// Minimum POE tokens required to become a validator
pub const MIN_VALIDATOR_STAKE: u64 = 10_000;

/// Maximum number of active validators in a committee
pub const MAX_COMMITTEE_SIZE: usize = 101;

/// Default emotional fitness threshold (0-100)
pub const DEFAULT_EMOTIONAL_THRESHOLD: u8 = 75;

/// Default Byzantine fault tolerance threshold (percentage)
pub const DEFAULT_BYZANTINE_THRESHOLD: u8 = 67;

/// Default epoch duration in milliseconds
pub const DEFAULT_EPOCH_DURATION: u64 = 30_000;

/// Version of the POE consensus protocol
pub const PROTOCOL_VERSION: &str = "1.0.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(TICKER, "POE");
        assert_eq!(MIN_VALIDATOR_STAKE, 10_000);
        assert!(DEFAULT_EMOTIONAL_THRESHOLD >= 50);
        assert!(DEFAULT_BYZANTINE_THRESHOLD >= 67);
    }
}
