//! Basic Proof of Emotion consensus example
//!
//! This example demonstrates:
//! - Creating a consensus engine
//! - Registering validators
//! - Starting consensus
//! - Monitoring state and metrics

use proof_of_emotion::{ConsensusConfig, EmotionalValidator, ProofOfEmotionEngine};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Proof of Emotion Consensus - Basic Example\n");

    // Create consensus configuration
    let config = ConsensusConfig {
        epoch_duration: 30_000,  // 30 second epochs
        emotional_threshold: 75, // 75% minimum emotional fitness
        byzantine_threshold: 67, // 67% BFT requirement
        committee_size: 5,       // 5 validators for this example
        minimum_stake: 10_000,   // 10,000 POE minimum
        voting_timeout: 8_000,
        proposal_timeout: 10_000,
        finality_timeout: 2_000,
    };

    println!("⚙️  Configuration:");
    println!("   - Epoch Duration: {}ms", config.epoch_duration);
    println!("   - Emotional Threshold: {}%", config.emotional_threshold);
    println!("   - Byzantine Threshold: {}%", config.byzantine_threshold);
    println!("   - Committee Size: {}", config.committee_size);
    println!("   - Minimum Stake: {} POE\n", config.minimum_stake);

    // Create consensus engine
    let engine = Arc::new(ProofOfEmotionEngine::new(config)?);
    println!("✅ Consensus engine created\n");

    // Register validators
    println!("👥 Registering validators...");

    let validators = vec![
        ("Alice", 10_000),
        ("Bob", 15_000),
        ("Charlie", 20_000),
        ("Diana", 12_000),
        ("Eve", 18_000),
    ];

    for (name, stake) in validators {
        let validator = EmotionalValidator::new(name, stake)?;
        engine.register_validator(validator).await?;
        println!("   ✓ {} registered with {} POE stake", name, stake);
    }

    println!("\n📊 Total validators: {}\n", engine.get_validator_count());

    // Start consensus
    println!("🎬 Starting consensus...\n");
    Arc::clone(&engine).start().await?;

    // Monitor consensus for 3 epochs
    println!("📈 Monitoring consensus (90 seconds)...\n");

    for i in 1..=3 {
        time::sleep(Duration::from_secs(30)).await;

        let state = engine.get_state().await;
        let metrics = engine.get_metrics().await;

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📊 Status Update #{}", i);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Epoch: {}", state.current_epoch);
        println!("Network Health: {}%", state.network_health);
        println!("Consensus Strength: {}%", state.consensus_strength);
        println!("Emotional Fitness: {}%", state.emotional_fitness);
        println!("Participation Rate: {}%", state.participation_rate);
        println!("Last Finalized Block: {}", state.last_finalized_height);
        println!("Pending Transactions: {}", state.pending_transactions);
        println!("\n📈 Metrics:");
        println!("Total Epochs: {}", metrics.total_epochs);
        println!("Successful: {}", metrics.successful_epochs);
        println!("Failed: {}", metrics.failed_epochs);
        println!("Avg Duration: {}ms", metrics.average_duration_ms);
        println!("Avg Emotional Score: {}%", metrics.average_emotional_score);
        println!("Byzantine Failures: {}", metrics.byzantine_failures);
        println!();
    }

    // Get finalized blocks
    let blocks = engine.get_finalized_blocks().await;
    println!("📦 Finalized blocks: {}\n", blocks.len());

    if !blocks.is_empty() {
        println!("Latest block:");
        let block = blocks.last().unwrap();
        println!("   Height: {}", block.header.height);
        println!("   Hash: {}...", &block.hash[..16]);
        println!("   Validator: {}", block.header.validator_id);
        println!("   Emotional Score: {}%", block.header.emotional_score);
        println!(
            "   Consensus Strength: {}%",
            block.header.consensus_strength
        );
        println!("   Transactions: {}", block.transactions.len());
    }

    // Stop consensus
    println!("\n🛑 Stopping consensus...");
    engine.stop().await?;

    println!("\n✅ Example completed successfully!");

    Ok(())
}
