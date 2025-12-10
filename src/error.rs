//! Error types for Proof of Emotion consensus

use thiserror::Error;

/// Result type alias for consensus operations
pub type Result<T> = std::result::Result<T, ConsensusError>;

/// Errors that can occur during consensus operations
#[derive(Error, Debug, Clone)]
pub enum ConsensusError {
    /// Validator does not meet emotional fitness requirements
    #[error("Insufficient emotional fitness: {score} < {threshold}")]
    InsufficientEmotionalFitness { score: u8, threshold: u8 },

    /// Validator stake is below minimum requirement
    #[error("Insufficient stake: {stake} < {minimum}")]
    InsufficientStake { stake: u64, minimum: u64 },

    /// Byzantine behavior detected
    #[error("Byzantine behavior detected: {reason}")]
    ByzantineFailure { reason: String },

    /// Validator not found
    #[error("Validator not found: {id}")]
    ValidatorNotFound { id: String },

    /// Invalid block proposal
    #[error("Invalid block: {reason}")]
    InvalidBlock { reason: String },

    /// Invalid vote
    #[error("Invalid vote: {reason}")]
    InvalidVote { reason: String },

    /// Consensus round timeout
    #[error("Consensus round timed out after {duration_ms}ms")]
    RoundTimeout { duration_ms: u64 },

    /// Network partition detected
    #[error("Network partition detected")]
    NetworkPartition,

    /// Cryptographic verification failed
    #[error("Signature verification failed: {reason}")]
    SignatureVerificationFailed { reason: String },

    /// Biometric data validation failed
    #[error("Biometric validation failed: {reason}")]
    BiometricValidationFailed { reason: String },

    /// Committee selection failed
    #[error("Committee selection failed: {reason}")]
    CommitteeSelectionFailed { reason: String },

    /// Fork detected
    #[error("Fork detected at height {height}")]
    ForkDetected { height: u64 },

    /// Storage error
    #[error("Storage error: {message}")]
    StorageError { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Already running
    #[error("Consensus engine is already running")]
    AlreadyRunning,

    /// Not running
    #[error("Consensus engine is not running")]
    NotRunning,

    /// Internal error
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl ConsensusError {
    /// Create an insufficient emotional fitness error
    pub fn insufficient_emotional_fitness(score: u8, threshold: u8) -> Self {
        Self::InsufficientEmotionalFitness { score, threshold }
    }

    /// Create an insufficient stake error
    pub fn insufficient_stake(stake: u64, minimum: u64) -> Self {
        Self::InsufficientStake { stake, minimum }
    }

    /// Create a Byzantine failure error
    pub fn byzantine_failure(reason: impl Into<String>) -> Self {
        Self::ByzantineFailure {
            reason: reason.into(),
        }
    }

    /// Create a validator not found error
    pub fn validator_not_found(id: impl Into<String>) -> Self {
        Self::ValidatorNotFound { id: id.into() }
    }

    /// Create an invalid block error
    pub fn invalid_block(reason: impl Into<String>) -> Self {
        Self::InvalidBlock {
            reason: reason.into(),
        }
    }

    /// Create an invalid vote error
    pub fn invalid_vote(reason: impl Into<String>) -> Self {
        Self::InvalidVote {
            reason: reason.into(),
        }
    }

    /// Create a round timeout error
    pub fn round_timeout(duration_ms: u64) -> Self {
        Self::RoundTimeout { duration_ms }
    }

    /// Create a signature verification failed error
    pub fn signature_verification_failed(reason: impl Into<String>) -> Self {
        Self::SignatureVerificationFailed {
            reason: reason.into(),
        }
    }

    /// Create a biometric validation failed error
    pub fn biometric_validation_failed(reason: impl Into<String>) -> Self {
        Self::BiometricValidationFailed {
            reason: reason.into(),
        }
    }

    /// Create a committee selection failed error
    pub fn committee_selection_failed(reason: impl Into<String>) -> Self {
        Self::CommitteeSelectionFailed {
            reason: reason.into(),
        }
    }

    /// Create a fork detected error
    pub fn fork_detected(height: u64) -> Self {
        Self::ForkDetected { height }
    }

    /// Create a storage error
    pub fn storage_error(message: impl Into<String>) -> Self {
        Self::StorageError {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ConsensusError::insufficient_emotional_fitness(65, 75);
        assert!(err.to_string().contains("65"));
        assert!(err.to_string().contains("75"));

        let err = ConsensusError::byzantine_failure("double voting");
        assert!(err.to_string().contains("double voting"));
    }
}
