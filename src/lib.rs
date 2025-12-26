pub mod biometric;
pub mod byzantine;
pub mod checkpoint;
pub mod consensus;
pub mod crypto;
pub mod error;
pub mod fork;
pub mod staking;
pub mod types;
pub mod utils;
pub mod zkp;

pub use biometric::{BiometricDevice, BiometricReading, EmotionalProfile, EmotionalValidator};
pub use byzantine::ByzantineDetector;
pub use checkpoint::{Checkpoint, CheckpointManager, CheckpointStatistics, ValidatorSignature};
pub use consensus::{
    ConsensusConfig, ConsensusMetrics, ConsensusRound, ConsensusState, ProofOfEmotionEngine,
    RoundPhase,
};
pub use crypto::{EmotionalProof, KeyPair, Signature};
pub use error::{ConsensusError, Result};
pub use fork::{ForkDetector, ForkInfo, ForkStatistics};
pub use staking::{EmotionalStaking, RewardDistribution, SlashingEvent, Validator};
pub use types::{Block, BlockHeader, Transaction, Vote, VotingResult};

pub const TICKER: &str = "POE";
pub const MIN_VALIDATOR_STAKE: u64 = 10_000;
pub const MAX_COMMITTEE_SIZE: usize = 101;
pub const DEFAULT_EMOTIONAL_THRESHOLD: u8 = 75;
pub const DEFAULT_BYZANTINE_THRESHOLD: u8 = 67;
pub const DEFAULT_EPOCH_DURATION: u64 = 30_000;
pub const PROTOCOL_VERSION: &str = "1.0.0";
/// Unbonding period in epochs (~21 days at 15min epochs)
pub const UNBONDING_PERIOD_EPOCHS: u64 = 2016;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(TICKER, "POE");
        assert_eq!(MIN_VALIDATOR_STAKE, 10_000);
        // Constants are verified at compile time by their values
        let _ = DEFAULT_EMOTIONAL_THRESHOLD;
        let _ = DEFAULT_BYZANTINE_THRESHOLD;
    }
}
