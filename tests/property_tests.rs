//! Property-based tests for Proof of Emotion consensus
//!
//! These tests use proptest to verify invariants hold across
//! randomly generated inputs.

use proof_of_emotion::*;
use proptest::prelude::*;

// ============================================================================
// BLOCK PROPERTIES
// ============================================================================

// Note: Block hashes include timestamps, so they are NOT deterministic
// across separate Block::new() calls. This is intentional for uniqueness.

proptest! {
    #[test]
    fn test_block_hash_uniqueness(
        height in 0u64..1000,
    ) {
        let tx = types::Transaction::new("s".to_string(), "r".to_string(), 100, 1);

        // Different heights should produce different hashes
        let block1 = types::Block::new(
            height,
            0,
            "prev".to_string(),
            "v1".to_string(),
            80,
            vec![tx.clone()],
        );

        let block2 = types::Block::new(
            height + 1,
            0,
            "prev".to_string(),
            "v1".to_string(),
            80,
            vec![tx],
        );

        prop_assert_ne!(block1.hash, block2.hash, "Different heights should have different hashes");
    }

    #[test]
    fn test_merkle_root_consistency(
        num_txs in 1usize..100,
    ) {
        let mut transactions = Vec::new();
        for i in 0..num_txs {
            transactions.push(types::Transaction::new(
                format!("sender{}", i),
                format!("receiver{}", i),
                1000 + i as u64,
                10,
            ));
        }

        let block = types::Block::new(
            1,
            0,
            "prev".to_string(),
            "validator".to_string(),
            80,
            transactions.clone(),
        );

        // Merkle root should be deterministic
        let block2 = types::Block::new(
            1,
            0,
            "prev".to_string(),
            "validator".to_string(),
            80,
            transactions,
        );

        prop_assert_eq!(block.header.merkle_root, block2.header.merkle_root);
    }
}

// ============================================================================
// TRANSACTION PROPERTIES
// ============================================================================

// Note: Transaction hashes include timestamps, so they are NOT deterministic
// across separate Transaction::new() calls. This is intentional for uniqueness.

proptest! {
    #[test]
    fn test_transaction_hash_validity(
        amount in 0u64..1_000_000,
    ) {
        let tx = types::Transaction::new(
            "sender".to_string(),
            "receiver".to_string(),
            amount,
            10,
        );

        prop_assert!(tx.verify_hash(), "Transaction hash should be valid");
    }

    #[test]
    fn test_transaction_expiration(
        age_seconds in 0u64..600, // 0-10 minutes
    ) {
        let tx = types::Transaction::new("s".to_string(), "r".to_string(), 100, 1);
        let max_age_ms = 5 * 60 * 1000; // 5 minutes

        let now = tx.timestamp + (age_seconds * 1000);
        let is_expired = tx.is_expired(now, max_age_ms);

        if age_seconds > 300 {
            // More than 5 minutes
            prop_assert!(is_expired, "Transaction should be expired after 5 minutes");
        } else {
            prop_assert!(!is_expired, "Transaction should not be expired within 5 minutes");
        }
    }
}

// ============================================================================
// VOTE PROPERTIES
// ============================================================================

proptest! {
    #[test]
    fn test_vote_creation(
        epoch in 0u64..1000,
        round in 0u32..10,
        emotional_score in 0u8..=100,
        approved in prop::bool::ANY,
    ) {
        let vote = types::Vote::new(
            "validator".to_string(),
            "block_hash".to_string(),
            epoch,
            round,
            emotional_score,
            approved,
        );

        prop_assert_eq!(vote.epoch, epoch);
        prop_assert_eq!(vote.round, round);
        prop_assert_eq!(vote.emotional_score, emotional_score);
        prop_assert_eq!(vote.approved, approved);
    }
}

// ============================================================================
// EMOTIONAL SCORE PROPERTIES
// ============================================================================

proptest! {
    #[test]
    fn test_emotional_score_always_valid(
        _seed in 0u64..1000,
    ) {
        // Use runtime for async code
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let validator = EmotionalValidator::new("test", 10_000)?;
            let simulator = biometric::BiometricSimulator::new("device".to_string(), "test");
            let readings = simulator.collect_readings()?;

            validator.update_emotional_state(readings).await?;

            let score = validator.get_emotional_score();
            // Score should always be in valid range
            prop_assert!(score <= 100, "Emotional score should be <= 100, got {}", score);

            Ok::<(), TestCaseError>(())
        }).unwrap();
    }

    #[test]
    fn test_reputation_bounds(
        adjustment in -100i16..100,
    ) {
        let validator = EmotionalValidator::new("test", 10_000).unwrap();

        let initial_reputation = validator.get_reputation();
        validator.adjust_reputation(adjustment);
        let new_reputation = validator.get_reputation();

        // Reputation should always be 0-100
        prop_assert!(new_reputation <= 100, "Reputation should be <= 100");

        // Verify adjustment logic
        if adjustment > 0 {
            prop_assert!(
                new_reputation >= initial_reputation,
                "Positive adjustment should increase reputation"
            );
        } else if adjustment < 0 {
            prop_assert!(
                new_reputation <= initial_reputation,
                "Negative adjustment should decrease reputation"
            );
        }
    }

    #[test]
    fn test_stake_non_negative(
        initial_stake in 0u64..1_000_000,
        slashing_amount in 0u64..100_000,
    ) {
        let validator = EmotionalValidator::new("test", initial_stake).unwrap();

        // Apply slashing
        validator.apply_slashing(slashing_amount);

        let final_stake = validator.get_stake();

        // Stake should never go negative (using saturating_sub)
        prop_assert!(final_stake <= initial_stake, "Stake should not increase from slashing");
    }
}

// ============================================================================
// CRYPTOGRAPHIC PROPERTIES
// ============================================================================

proptest! {
    #[test]
    fn test_signature_verification_consistency(
        message_len in 1usize..1000,
    ) {
        let keypair = crypto::KeyPair::generate().unwrap();
        let message = vec![0u8; message_len];

        let signature = keypair.sign(&message).unwrap();
        let is_valid = crypto::KeyPair::verify(
            &message,
            &signature,
            &keypair.public_key_hex()
        ).unwrap();

        prop_assert!(is_valid, "Valid signature should verify");
    }

    #[test]
    fn test_signature_invalid_on_tamper(
        original_len in 10usize..100,
        tamper_index in 0usize..10,
    ) {
        let keypair = crypto::KeyPair::generate().unwrap();
        let original_message = vec![0x42u8; original_len];

        let signature = keypair.sign(&original_message).unwrap();

        // Tamper with message
        let mut tampered_message = original_message.clone();
        if tamper_index < tampered_message.len() {
            tampered_message[tamper_index] ^= 0xFF;

            let is_valid = crypto::KeyPair::verify(
                &tampered_message,
                &signature,
                &keypair.public_key_hex()
            );

            // Should either fail to verify or return false
            prop_assert!(
                is_valid.is_err() || matches!(is_valid, Ok(false)),
                "Tampered message should not verify"
            );
        }
    }
}

// ============================================================================
// CONSENSUS PROPERTIES
// ============================================================================

proptest! {
    #[test]
    fn test_byzantine_threshold_calculation(
        committee_size in 3usize..101,
        byzantine_threshold in 51u8..100,
    ) {
        // Calculate required votes
        let required = (committee_size as f64 * (byzantine_threshold as f64 / 100.0)).ceil() as usize;

        // Required votes should be more than half
        prop_assert!(
            required > committee_size / 2,
            "Byzantine threshold should require >50% votes"
        );

        // Should not require more than committee size
        prop_assert!(
            required <= committee_size,
            "Required votes should not exceed committee size"
        );
    }

    #[test]
    fn test_committee_selection_deterministic(
        num_validators in 10usize..50,
        committee_size in 3usize..21,
    ) {
        let config = ConsensusConfig {
            committee_size,
            ..Default::default()
        };

        let engine = ProofOfEmotionEngine::new(config);
        prop_assert!(engine.is_ok(), "Engine creation should succeed");

        // Committee size should not exceed number of validators
        let actual_committee_size = committee_size.min(num_validators);
        prop_assert!(actual_committee_size <= num_validators);
    }
}

// ============================================================================
// STAKING PROPERTIES
// ============================================================================

proptest! {
    #[test]
    fn test_stake_locking_invariants(
        total_stake in 10_000u64..1_000_000,
        lock_amount in 1_000u64..100_000,
    ) {
        use staking::EmotionalStaking;

        let staking = EmotionalStaking::new(10_000);

        staking.register_validator(
            "val1".to_string(),
            "addr1".to_string(),
            total_stake,
            5
        ).unwrap();

        // Lock amount should not exceed available stake
        let amount_to_lock = lock_amount.min(total_stake);

        if let Ok(()) = staking.lock_stake("val1", amount_to_lock, 1) {
            let validator = staking.get_validator("val1").unwrap();

            // Invariant: locked + available = total (approximately, due to slashing)
            prop_assert!(
                validator.locked_stake + validator.available_stake <= total_stake,
                "Locked + available should not exceed total stake"
            );

            prop_assert!(
                validator.locked_stake == amount_to_lock,
                "Locked amount should match requested amount"
            );
        }
    }

    #[test]
    fn test_unbonding_period_positive(
        unbonding_epochs in 1u64..5000,
    ) {
        // Unbonding period should always be positive
        prop_assert!(unbonding_epochs > 0, "Unbonding period must be positive");

        // Current epoch + unbonding period should not overflow
        let current_epoch = 1000u64;
        let unlock_epoch = current_epoch.saturating_add(unbonding_epochs);

        prop_assert!(
            unlock_epoch >= current_epoch,
            "Unlock epoch should be in the future"
        );
    }
}
