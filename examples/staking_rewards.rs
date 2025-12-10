//! Staking and rewards example

use proof_of_emotion::staking::{
    EmotionalStaking, SlashingOffense,
};
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    println!("💰 Staking and Rewards Example\n");

    let staking = EmotionalStaking::new(10_000);

    println!("👥 Registering validators...\n");
    
    let validators = vec![
        ("Alice", "poe1alice", 10_000, 5),
        ("Bob", "poe1bob", 15_000, 3),
        ("Charlie", "poe1charlie", 20_000, 7),
    ];

    for (name, address, stake, commission) in validators {
        staking.register_validator(
            name.to_string(),
            address.to_string(),
            stake,
            commission,
        )?;
        println!("   ✓ {} - {} POE stake, {}% commission", name, stake, commission);
    }

    println!("\n💼 Delegating stakes...\n");
    
    staking.delegate_stake(
        "Alice".to_string(),
        "delegator1".to_string(),
        5_000,
        21 * 24 * 60 * 60,
    )?;
    println!("   ✓ Delegator1 → Alice: 5,000 POE");

    staking.delegate_stake(
        "Bob".to_string(),
        "delegator2".to_string(),
        8_000,
        21 * 24 * 60 * 60,
    )?;
    println!("   ✓ Delegator2 → Bob: 8,000 POE");

    println!("\n💓 Simulating epoch with emotional scores...\n");
    
    let mut scores = HashMap::new();
    scores.insert("Alice".to_string(), 85);
    scores.insert("Bob".to_string(), 90);
    scores.insert("Charlie".to_string(), 82);

    for (validator, score) in &scores {
        println!("   {} emotional score: {}%", validator, score);
    }

    println!("\n🎁 Distributing rewards...\n");
    let distribution = staking.distribute_rewards(scores)?;

    println!("   Total rewards: {} POE", distribution.total_rewards);
    println!("\n   Validator rewards:");
    for (validator, reward) in &distribution.validator_rewards {
        println!("      {} → {} POE", validator, reward);
    }

    println!("\n⚠️  Applying slashing to Charlie...\n");
    
    let charlie_before = staking.get_validator("Charlie").unwrap();
    println!("   Charlie stake before: {} POE", charlie_before.stake);

    staking.slash_validator(
        "Charlie",
        SlashingOffense::PoorEmotionalBehavior,
        "Emotional score dropped below 40%".to_string(),
    )?;

    let charlie_after = staking.get_validator("Charlie").unwrap();
    println!("   Charlie stake after: {} POE", charlie_after.stake);
    println!("   Slashed: {} POE", charlie_before.stake - charlie_after.stake);

    println!("\n✅ Staking example completed!");
    Ok(())
}
