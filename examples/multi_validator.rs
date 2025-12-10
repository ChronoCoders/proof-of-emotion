//! Multi-validator example with parallel consensus

use proof_of_emotion::{
    ConsensusConfig, EmotionalValidator, ProofOfEmotionEngine,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🚀 Multi-Validator Consensus Example\n");

    let config = ConsensusConfig {
        epoch_duration: 20_000,
        emotional_threshold: 70,
        byzantine_threshold: 67,
        committee_size: 10,
        minimum_stake: 10_000,
        ..Default::default()
    };

    let engine = Arc::new(ProofOfEmotionEngine::new(config)?);

    // Register 20 validators with varying stakes
    println!("👥 Registering 20 validators...\n");
    for i in 1..=20 {
        let stake = 10_000 + (i * 1_000);
        let validator = EmotionalValidator::new(
            format!("validator-{:02}", i),
            stake
        )?;
        engine.register_validator(validator).await?;
        println!("   ✓ validator-{:02} with {} POE", i, stake);
    }

    println!("\n📊 Starting consensus with {} validators", engine.get_validator_count());
    Arc::clone(&engine).start().await?;

    // Monitor for 2 minutes
    for round in 1..=6 {
        time::sleep(Duration::from_secs(20)).await;
        
        let state = engine.get_state().await;
        println!("\n📊 Round {} - Epoch {}", round, state.current_epoch);
        println!("   Network Health: {}%", state.network_health);
        println!("   Consensus: {}%", state.consensus_strength);
        println!("   Participation: {}%", state.participation_rate);
    }

    engine.stop().await?;
    println!("\n✅ Multi-validator example completed!");
    Ok(())
}
