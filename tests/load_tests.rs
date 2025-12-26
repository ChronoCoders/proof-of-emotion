//! Load and performance tests for Proof of Emotion consensus
//!
//! Run with: cargo test --test load_tests -- --ignored --test-threads=1
//!
//! These tests verify the system can handle:
//! - Large numbers of validators (1000+)
//! - High transaction throughput
//! - Extended runtime without memory leaks

use proof_of_emotion::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::test]
#[ignore] // Run with: cargo test --ignored test_1000_validators
async fn test_1000_validators() {
    let config = ConsensusConfig {
        committee_size: 100,
        epoch_duration: 30_000,
        emotional_threshold: 50,
        minimum_stake: 10_000,
        ..Default::default()
    };

    let engine = Arc::new(ProofOfEmotionEngine::new(config).unwrap());

    println!("Registering 1000 validators...");

    // Register 1000 validators
    for i in 0..1000 {
        let stake = 10_000 + (i * 100); // Varying stakes
        let validator = EmotionalValidator::new(format!("validator-{}", i), stake).unwrap();
        engine.register_validator(validator).await.unwrap();

        if (i + 1) % 100 == 0 {
            println!("  Registered {} validators", i + 1);
        }
    }

    assert_eq!(engine.get_validator_count(), 1000);
    println!("✅ All 1000 validators registered");

    // Start consensus
    println!("Starting consensus engine...");
    Arc::clone(&engine).start().await.unwrap();

    // Monitor for 10 epochs
    println!("Monitoring for 10 epochs...");
    for epoch in 1..=10 {
        time::sleep(Duration::from_secs(30)).await;

        let state = engine.get_state().await;
        let metrics = engine.get_metrics().await;

        println!(
            "Epoch {}: consensus_strength={}%, participation={}%, health={}%",
            state.current_epoch, state.consensus_strength, state.participation_rate, state.network_health
        );

        // Verify consensus still working
        assert!(
            state.consensus_strength >= 67,
            "Consensus strength dropped below 67% in epoch {}",
            epoch
        );
        assert_eq!(
            metrics.failed_epochs, 0,
            "Failed epochs detected: {}",
            metrics.failed_epochs
        );
    }

    engine.stop().await.unwrap();
    println!("✅ Consensus maintained with 1000 validators");
}

#[tokio::test]
#[ignore]
async fn test_high_transaction_throughput() {
    let config = ConsensusConfig {
        epoch_duration: 15_000, // Faster epochs
        committee_size: 21,
        ..Default::default()
    };

    let engine = Arc::new(ProofOfEmotionEngine::new(config).unwrap());

    // Register validators
    for i in 0..30 {
        let validator = EmotionalValidator::new(format!("validator-{}", i), 10_000).unwrap();
        engine.register_validator(validator).await.unwrap();
    }

    println!("Submitting 10,000 transactions...");
    let start_time = std::time::Instant::now();

    // Submit 10,000 transactions
    for i in 0..10_000 {
        let tx = types::Transaction::new(
            format!("sender-{}", i % 100),
            format!("receiver-{}", (i + 1) % 100),
            1000 + i,
            10,
        );
        engine.submit_transaction(tx).await.unwrap();

        if (i + 1) % 1000 == 0 {
            println!("  Submitted {} transactions", i + 1);
        }
    }

    let submission_time = start_time.elapsed();
    println!("✅ Submitted 10,000 transactions in {:?}", submission_time);

    // Start consensus
    Arc::clone(&engine).start().await.unwrap();

    // Wait for transactions to be processed
    println!("Waiting for transaction processing...");
    let processing_start = std::time::Instant::now();

    loop {
        time::sleep(Duration::from_secs(5)).await;

        let state = engine.get_state().await;
        let blocks = engine.get_finalized_blocks().await;
        let total_txs: usize = blocks.iter().map(|b| b.transactions.len()).sum();

        println!(
            "  Processed {} transactions in {} blocks (pending: {})",
            total_txs,
            blocks.len(),
            state.pending_transactions
        );

        // Stop when most transactions processed or timeout (5 minutes)
        if state.pending_transactions < 100 || processing_start.elapsed().as_secs() > 300 {
            break;
        }
    }

    let total_time = processing_start.elapsed();
    let blocks = engine.get_finalized_blocks().await;
    let total_txs: usize = blocks.iter().map(|b| b.transactions.len()).sum();
    let tps = total_txs as f64 / total_time.as_secs_f64();

    println!("✅ Processed {} transactions in {:?}", total_txs, total_time);
    println!("   Throughput: {:.2} TPS", tps);

    engine.stop().await.unwrap();

    assert!(total_txs > 5000, "Should process significant number of transactions");
}

#[tokio::test]
#[ignore]
async fn test_memory_stability() {
    use std::process::Command;

    let config = ConsensusConfig {
        epoch_duration: 10_000,
        committee_size: 21,
        ..Default::default()
    };

    let engine = Arc::new(ProofOfEmotionEngine::new(config).unwrap());

    // Register validators
    for i in 0..50 {
        let validator = EmotionalValidator::new(format!("validator-{}", i), 10_000).unwrap();
        engine.register_validator(validator).await.unwrap();
    }

    Arc::clone(&engine).start().await.unwrap();

    println!("Monitoring memory usage for 5 minutes...");

    // Get initial memory
    let get_memory = || -> Option<u64> {
        let output = Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()?;
        String::from_utf8(output.stdout)
            .ok()?
            .trim()
            .parse()
            .ok()
    };

    let initial_memory = get_memory().unwrap_or(0);
    println!("Initial memory: {} KB", initial_memory);

    let mut memory_samples = Vec::new();

    // Monitor for 5 minutes
    for minute in 1..=5 {
        time::sleep(Duration::from_secs(60)).await;

        if let Some(current_memory) = get_memory() {
            memory_samples.push(current_memory);
            let growth = (current_memory as f64 - initial_memory as f64) / initial_memory as f64
                * 100.0;

            println!(
                "Minute {}: {} KB (growth: {:.2}%)",
                minute, current_memory, growth
            );

            let state = engine.get_state().await;
            println!(
                "  Epoch: {}, Pending TXs: {}",
                state.current_epoch, state.pending_transactions
            );
        }

        // Submit some transactions to create load
        for i in 0..100 {
            let tx = types::Transaction::new(
                format!("sender-{}", i),
                format!("receiver-{}", i),
                1000,
                10,
            );
            engine.submit_transaction(tx).await.unwrap();
        }
    }

    engine.stop().await.unwrap();

    // Check memory didn't grow unbounded
    let final_memory = memory_samples.last().copied().unwrap_or(initial_memory);
    let max_memory = memory_samples.iter().max().copied().unwrap_or(initial_memory);

    println!("\n📊 Memory Statistics:");
    println!("  Initial: {} KB", initial_memory);
    println!("  Final: {} KB", final_memory);
    println!("  Peak: {} KB", max_memory);
    println!(
        "  Growth: {:.2}%",
        (final_memory as f64 - initial_memory as f64) / initial_memory as f64 * 100.0
    );

    // Memory should not grow more than 200% over 5 minutes
    let growth_factor = final_memory as f64 / initial_memory as f64;
    assert!(
        growth_factor < 3.0,
        "Memory grew by {}x - possible leak!",
        growth_factor
    );

    println!("✅ Memory usage stable (no significant leaks detected)");
}

#[tokio::test]
#[ignore]
async fn test_byzantine_attacks_at_scale() {
    use proof_of_emotion::byzantine::ByzantineDetector;

    let detector = Arc::new(ByzantineDetector::new());

    println!("Simulating 100 Byzantine attacks...");

    // Simulate 100 double voting attempts
    for i in 0..100 {
        let vote1 = types::Vote::new(
            format!("byzantine-{}", i),
            "block-abc".to_string(),
            1,
            0,
            80,
            true,
        );
        detector.record_vote(&vote1).await.unwrap();

        let vote2 = types::Vote::new(
            format!("byzantine-{}", i),
            "block-abc".to_string(),
            1,
            0,
            80,
            false, // Different vote!
        );

        let result = detector.record_vote(&vote2).await;
        assert!(result.is_err(), "Attack {} should be detected", i);
    }

    let events = detector.get_slashing_events().await;
    assert_eq!(
        events.len(),
        100,
        "All 100 Byzantine attacks should be detected"
    );

    println!("✅ Detected all 100 Byzantine attacks");
}

#[tokio::test]
#[ignore]
async fn test_concurrent_consensus_rounds() {
    let config = ConsensusConfig {
        epoch_duration: 5_000, // Very fast epochs
        committee_size: 10,
        ..Default::default()
    };

    let engine = Arc::new(ProofOfEmotionEngine::new(config).unwrap());

    // Register validators
    for i in 0..20 {
        let validator = EmotionalValidator::new(format!("validator-{}", i), 10_000).unwrap();
        engine.register_validator(validator).await.unwrap();
    }

    println!("Testing rapid consensus rounds...");
    Arc::clone(&engine).start().await.unwrap();

    // Run for 30 seconds with very fast epochs
    time::sleep(Duration::from_secs(30)).await;

    let state = engine.get_state().await;
    let metrics = engine.get_metrics().await;

    println!("Completed {} epochs in 30 seconds", state.current_epoch);
    println!(
        "Success rate: {:.2}%",
        (metrics.successful_epochs as f64 / metrics.total_epochs as f64) * 100.0
    );

    engine.stop().await.unwrap();

    assert!(
        state.current_epoch >= 5,
        "Should complete at least 5 epochs"
    );
    assert!(
        metrics.successful_epochs > 0,
        "Should have some successful epochs"
    );

    println!("✅ Rapid consensus rounds successful");
}
