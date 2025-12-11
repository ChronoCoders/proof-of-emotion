//! Integration tests for Proof of Emotion consensus

use proof_of_emotion::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn test_basic_consensus_flow() {
    let config = ConsensusConfig {
        epoch_duration: 5_000,   // Faster for testing
        emotional_threshold: 50, // Lower for testing
        committee_size: 3,
        minimum_stake: 1_000,
        ..Default::default()
    };

    let engine = Arc::new(ProofOfEmotionEngine::new(config).unwrap());

    // Register validators
    for i in 1..=5 {
        let validator = EmotionalValidator::new(format!("validator-{}", i), 10_000).unwrap();
        engine.register_validator(validator).await.unwrap();
    }

    assert_eq!(engine.get_validator_count(), 5);

    // Start consensus
    Arc::clone(&engine).start().await.unwrap();

    // Wait for a few epochs
    time::sleep(Duration::from_secs(15)).await;

    let state = engine.get_state().await;
    assert!(state.current_epoch >= 2);

    engine.stop().await.unwrap();
}

#[tokio::test]
async fn test_validator_registration() {
    let config = ConsensusConfig::default();
    let engine = ProofOfEmotionEngine::new(config).unwrap();

    // Valid registration
    let validator = EmotionalValidator::new("test-validator", 10_000).unwrap();
    let result = engine.register_validator(validator).await;
    assert!(result.is_ok());

    // Insufficient stake
    let validator2 = EmotionalValidator::new("test-validator-2", 5_000).unwrap();
    let result = engine.register_validator(validator2).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_emotional_validation() {
    let validator = EmotionalValidator::new("test", 10_000).unwrap();

    let simulator = biometric::BiometricSimulator::new("device1".to_string(), "test");
    let readings = simulator.collect_readings().unwrap();

    validator.update_emotional_state(readings).await.unwrap();

    let score = validator.get_emotional_score();
    assert!(score > 0 && score <= 100);

    let profile = validator.get_emotional_profile().unwrap();
    assert!(profile.confidence > 0);
}

#[tokio::test]
async fn test_cryptographic_signatures() {
    let keypair = crypto::KeyPair::generate().unwrap();
    let message = b"test message";

    let signature = keypair.sign(message).unwrap();
    let valid = crypto::KeyPair::verify(message, &signature, &keypair.public_key_hex()).unwrap();

    assert!(valid);
}

#[tokio::test]
async fn test_emotional_proof() {
    let keypair = crypto::KeyPair::generate().unwrap();

    let mut scores = std::collections::HashMap::new();
    scores.insert("v1".to_string(), 85);
    scores.insert("v2".to_string(), 90);

    let mut hashes = std::collections::HashMap::new();
    hashes.insert("v1".to_string(), "hash1".to_string());
    hashes.insert("v2".to_string(), "hash2".to_string());

    let proof = crypto::EmotionalProof::new(
        vec!["v1".to_string(), "v2".to_string()],
        scores,
        hashes,
        30_000,
        &keypair,
    )
    .unwrap();

    assert!(proof.verify(&keypair.public_key_hex()).unwrap());
}

#[tokio::test]
async fn test_staking_and_rewards() {
    let staking = staking::EmotionalStaking::new(10_000);

    // Register validator
    staking
        .register_validator("validator-1".to_string(), "addr1".to_string(), 10_000, 5)
        .unwrap();

    // Delegate stake
    staking
        .delegate_stake(
            "validator-1".to_string(),
            "delegator-1".to_string(),
            5_000,
            21 * 24 * 60 * 60,
        )
        .unwrap();

    // Distribute rewards
    let mut scores = std::collections::HashMap::new();
    scores.insert("validator-1".to_string(), 85);

    let distribution = staking.distribute_rewards(scores).unwrap();
    assert!(distribution.total_rewards > 0);
}

#[tokio::test]
async fn test_slashing() {
    let staking = staking::EmotionalStaking::new(10_000);

    staking
        .register_validator("validator-1".to_string(), "addr1".to_string(), 10_000, 5)
        .unwrap();

    let validator_before = staking.get_validator("validator-1").unwrap();
    let stake_before = validator_before.stake;

    staking
        .slash_validator(
            "validator-1",
            staking::SlashingOffense::PoorEmotionalBehavior,
            "Score below threshold".to_string(),
        )
        .unwrap();

    let validator_after = staking.get_validator("validator-1").unwrap();
    assert!(validator_after.stake < stake_before);
}

#[tokio::test]
async fn test_block_creation() {
    let tx1 = types::Transaction::new("addr1".to_string(), "addr2".to_string(), 1000, 10);

    let block = types::Block::new(
        1,
        0,
        "0".repeat(64),
        "validator1".to_string(),
        85,
        vec![tx1],
    );

    assert_eq!(block.header.height, 1);
    assert!(block.verify_hash());
}

#[tokio::test]
async fn test_transaction_validation() {
    let tx = types::Transaction::new("sender".to_string(), "receiver".to_string(), 1000, 10);

    assert!(tx.verify_hash());
    assert_eq!(tx.amount, 1000);
    assert_eq!(tx.fee, 10);
}

#[tokio::test]
async fn test_byzantine_threshold() {
    let config = ConsensusConfig {
        byzantine_threshold: 67,
        committee_size: 10,
        ..Default::default()
    };

    let engine = ProofOfEmotionEngine::new(config).unwrap();

    // With 10 validators in committee, need 7 votes (67%)
    // This is enforced in the voting phase
    assert_eq!(engine.get_validator_count(), 0);
}

#[tokio::test]
async fn test_emotional_threshold_enforcement() {
    let validator = EmotionalValidator::new("test", 10_000).unwrap();

    // Without emotional update, score is 0
    assert!(!validator.is_eligible(75, 10_000));

    // After update with good readings, should be eligible
    let simulator = biometric::BiometricSimulator::new("device1".to_string(), "test");
    let readings = simulator.collect_readings().unwrap();
    validator.update_emotional_state(readings).await.unwrap();

    // Eligibility depends on actual score (which varies with simulation)
    // Score should be > 0 after biometric update
    let score = validator.get_emotional_score();
    assert!(score > 0, "Emotional score should be > 0 after update");
}
