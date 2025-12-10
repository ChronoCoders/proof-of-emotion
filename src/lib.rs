pub mod biometric;
pub mod consensus;
pub mod crypto;
pub mod error;
pub mod staking;
pub mod types;
pub mod utils;

pub use consensus::{
    ConsensusConfig, ConsensusMetrics, ConsensusRound, ConsensusState, 
    ProofOfEmotionEngine, RoundPhase
};
pub use biometric::{BiometricDevice, BiometricReading, EmotionalProfile, EmotionalValidator};
pub use crypto::{EmotionalProof, KeyPair, Signature};
pub use error::{ConsensusError, Result};
pub use staking::{EmotionalStaking, RewardDistribution, SlashingEvent, Validator};
pub use types::{Block, BlockHeader, Transaction, Vote, VotingResult};

pub const TICKER: &str = "POE";
pub const MIN_VALIDATOR_STAKE: u64 = 10_000;
pub const MAX_COMMITTEE_SIZE: usize = 101;
pub const DEFAULT_EMOTIONAL_THRESHOLD: u8 = 75;
pub const DEFAULT_BYZANTINE_THRESHOLD: u8 = 67;
pub const DEFAULT_EPOCH_DURATION: u64 = 30_000;
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
