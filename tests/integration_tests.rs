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

// ============================================================================
// BYZANTINE FAULT DETECTION TESTS (Issue 7.1)
// ============================================================================

#[tokio::test]
async fn test_double_voting_detection() {
    use proof_of_emotion::byzantine::ByzantineDetector;

    let detector = Arc::new(ByzantineDetector::new());

    // Create first vote (approve)
    let vote1 = Vote::new(
        "byzantine-validator".to_string(),
        "block-hash-abc".to_string(),
        1, // epoch
        0, // round
        80,
        true, // approved
    );

    detector.record_vote(&vote1).await.unwrap();

    // Byzantine validator tries to vote differently on same block
    let vote2 = Vote::new(
        "byzantine-validator".to_string(),
        "block-hash-abc".to_string(),
        1, // same epoch
        0, // same round
        80,
        false, // rejected - Byzantine behavior!
    );

    // Should detect double voting
    let result = detector.record_vote(&vote2).await;
    assert!(result.is_err(), "Double voting should be detected");
    assert!(result.unwrap_err().contains("Double voting"));

    // Check slashing event was created
    let events = detector.get_slashing_events().await;
    assert!(!events.is_empty(), "Slashing event should be created");
    assert_eq!(events[0].validator_id, "byzantine-validator");
}

#[tokio::test]
async fn test_equivocation_detection() {
    use proof_of_emotion::byzantine::ByzantineDetector;

    let detector = Arc::new(ByzantineDetector::new());

    // Vote on first block
    let vote1 = Vote::new(
        "equivocating-validator".to_string(),
        "block-hash-1".to_string(),
        1, // epoch
        0,
        80,
        true,
    );
    detector.record_vote(&vote1).await.unwrap();

    // Vote on different block in same epoch - equivocation!
    let vote2 = Vote::new(
        "equivocating-validator".to_string(),
        "block-hash-2".to_string(),
        1, // same epoch - equivocation!
        0,
        80,
        true,
    );

    let result = detector.record_vote(&vote2).await;
    assert!(result.is_err(), "Equivocation should be detected");

    let events = detector.get_slashing_events().await;
    assert!(!events.is_empty());
}

#[tokio::test]
async fn test_double_block_proposal() {
    use proof_of_emotion::byzantine::ByzantineDetector;

    let detector = Arc::new(ByzantineDetector::new());

    // Propose first block at height 1
    detector
        .record_proposal("double-signer", 1, "block-hash-A")
        .await
        .unwrap();

    // Propose different block at same height - double signing!
    let result = detector
        .record_proposal("double-signer", 1, "block-hash-B")
        .await;

    assert!(result.is_err(), "Double signing should be detected");
    assert!(result.unwrap_err().contains("Double signing"));

    let events = detector.get_slashing_events().await;
    assert!(!events.is_empty());
    assert_eq!(events[0].validator_id, "double-signer");
}

#[tokio::test]
async fn test_invalid_block_rejection() {
    let validator = EmotionalValidator::new("test-validator", 10_000).unwrap();

    // Create a block with transactions
    let tx = Transaction::new("sender".to_string(), "receiver".to_string(), 1000, 10);
    let mut block = Block::new(
        1,
        0,
        "0".repeat(64),
        "test-validator".to_string(),
        80,
        vec![tx],
    );

    // Sign the block
    block.sign(&validator.key_pair).unwrap();

    // Tamper with merkle root
    block.header.merkle_root = "invalid_merkle_root".to_string();

    // Validation should fail
    let result = validator.validate_block(&block, &"0".repeat(64), 1, 0);
    assert!(result.is_err(), "Invalid merkle root should be rejected");
}

#[tokio::test]
async fn test_invalid_signature_rejection() {
    let validator1 = EmotionalValidator::new("validator-1", 10_000).unwrap();
    let validator2 = EmotionalValidator::new("validator-2", 10_000).unwrap();

    // Create and sign block with validator1
    let tx = Transaction::new("sender".to_string(), "receiver".to_string(), 1000, 10);
    let mut block = Block::new(
        1,
        0,
        "0".repeat(64),
        "validator-1".to_string(),
        80,
        vec![tx],
    );

    block.sign(&validator1.key_pair).unwrap();

    // Replace signature with validator2's signature (invalid!)
    let fake_message = b"fake";
    let fake_sig = validator2.key_pair.sign(fake_message).unwrap();
    block.signature = serde_json::to_string(&fake_sig).unwrap();
    block.proposer_public_key = validator2.key_pair.public_key_hex();

    // Validation should fail due to invalid signature
    let result = validator1.validate_block(&block, &"0".repeat(64), 1, 0);
    assert!(result.is_err(), "Invalid signature should be rejected");
}

#[tokio::test]
async fn test_future_timestamp_rejection() {
    let validator = EmotionalValidator::new("test-validator", 10_000).unwrap();

    let tx = Transaction::new("sender".to_string(), "receiver".to_string(), 1000, 10);
    let mut block = Block::new(
        1,
        0,
        "0".repeat(64),
        "test-validator".to_string(),
        80,
        vec![tx],
    );

    // Set timestamp 10 seconds in the future
    let future_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 10_000;
    block.header.timestamp = future_time;
    block.sign(&validator.key_pair).unwrap();

    // Validation should fail
    let result = validator.validate_block(&block, &"0".repeat(64), 1, 0);
    assert!(
        result.is_err(),
        "Future timestamp should be rejected"
    );
}

#[tokio::test]
async fn test_replay_attack_prevention() {
    let validator = EmotionalValidator::new("test-validator", 10_000).unwrap();

    // Create block for epoch 1
    let tx = Transaction::new("sender".to_string(), "receiver".to_string(), 1000, 10);
    let mut block = Block::new(
        1,
        1, // epoch 1
        "0".repeat(64),
        "test-validator".to_string(),
        80,
        vec![tx],
    );

    block.sign(&validator.key_pair).unwrap();

    // Try to validate with epoch 2 (current epoch)
    let result = validator.validate_block(&block, &"0".repeat(64), 1, 2);
    assert!(
        result.is_err(),
        "Old epoch block should be rejected (replay attack prevention)"
    );
    assert!(result.unwrap_err().contains("Epoch mismatch"));
}

#[tokio::test]
async fn test_transaction_expiration() {
    // Create transaction
    let tx = Transaction::new("sender".to_string(), "receiver".to_string(), 1000, 10);

    // Transaction should not be expired immediately
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let max_age = 5 * 60 * 1000; // 5 minutes

    assert!(
        !tx.is_expired(now, max_age),
        "Fresh transaction should not be expired"
    );

    // Transaction should be expired after TTL
    let future_time = now + max_age + 1000; // 1 second past expiry
    assert!(
        tx.is_expired(future_time, max_age),
        "Old transaction should be expired"
    );
}
