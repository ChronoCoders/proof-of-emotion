//! Fork detection and resolution for Proof of Emotion consensus

use crate::error::{ConsensusError, Result};
use crate::types::Block;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Information about a detected fork
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkInfo {
    /// Height where fork occurred
    pub height: u64,
    /// Competing block hashes
    pub competing_hashes: Vec<String>,
    /// Timestamp when fork was detected
    pub detected_at: u64,
    /// Resolution method used
    pub resolution_method: Option<String>,
    /// Winning chain hash
    pub winning_hash: Option<String>,
}

/// Fork detection and resolution system
pub struct ForkDetector {
    /// Map of height -> set of block hashes seen at that height
    blocks_at_height: DashMap<u64, HashSet<String>>,
    /// Canonical chain (list of block hashes)
    canonical_chain: Arc<RwLock<Vec<String>>>,
    /// Detected forks
    forks: Arc<RwLock<Vec<ForkInfo>>>,
    /// Block metadata for fork resolution
    block_metadata: DashMap<String, BlockMetadata>,
}

/// Metadata about a block for fork resolution
#[derive(Debug, Clone)]
struct BlockMetadata {
    height: u64,
    emotional_score: u8,
    consensus_strength: u8,
    timestamp: u64,
}

impl ForkDetector {
    /// Create a new fork detector
    pub fn new() -> Self {
        Self {
            blocks_at_height: DashMap::new(),
            canonical_chain: Arc::new(RwLock::new(Vec::new())),
            forks: Arc::new(RwLock::new(Vec::new())),
            block_metadata: DashMap::new(),
        }
    }

    /// Record a block and detect if it creates a fork
    pub async fn record_block(&self, block: &Block) -> Result<()> {
        let height = block.header.height;
        let hash = block.hash.clone();

        // Store block metadata for potential fork resolution
        self.block_metadata.insert(
            hash.clone(),
            BlockMetadata {
                height,
                emotional_score: block.header.emotional_score,
                consensus_strength: block.header.consensus_strength,
                timestamp: block.header.timestamp,
            },
        );

        // Get or create the set of blocks at this height
        let mut blocks = self
            .blocks_at_height
            .entry(height)
            .or_default();

        // Check if this creates a fork
        if !blocks.is_empty() && !blocks.contains(&hash) {
            // Fork detected!
            let competing_hashes: Vec<String> = blocks.iter().cloned().collect();
            warn!(
                block_height = height,
                competing_blocks = competing_hashes.len() + 1,
                new_block_hash = %hash,
                "Fork detected - multiple blocks at same height"
            );

            // Add the competing block to the set (so has_fork works correctly)
            blocks.insert(hash.clone());

            // Record the fork
            let fork_info = ForkInfo {
                height,
                competing_hashes: {
                    let mut hashes = competing_hashes.clone();
                    hashes.push(hash.clone());
                    hashes
                },
                detected_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("System time before UNIX_EPOCH")
                    .as_millis() as u64,
                resolution_method: None,
                winning_hash: None,
            };

            self.forks.write().await.push(fork_info);

            return Err(ConsensusError::fork_detected(height));
        }

        // No fork, add the block hash
        blocks.insert(hash.clone());

        // Update canonical chain if this extends it
        let mut canonical = self.canonical_chain.write().await;
        if canonical.is_empty() || height == canonical.len() as u64 {
            canonical.push(hash);
        }

        Ok(())
    }

    /// Resolve a fork at the given height
    ///
    /// Uses the Proof of Emotion fork choice rule:
    /// 1. Choose chain with highest cumulative emotional score
    /// 2. If tied, choose chain with highest consensus strength
    /// 3. If still tied, choose chain with earliest timestamp
    pub async fn resolve_fork(&self, height: u64) -> Result<String> {
        let blocks = self
            .blocks_at_height
            .get(&height)
            .ok_or_else(|| ConsensusError::internal(format!("No blocks at height {}", height)))?;

        if blocks.is_empty() {
            return Err(ConsensusError::internal(format!(
                "No blocks to resolve at height {}",
                height
            )));
        }

        // If only one block, no fork to resolve
        if blocks.len() == 1 {
            return Ok(blocks.iter().next().unwrap().clone());
        }

        info!(
            block_height = height,
            competing_blocks = blocks.len(),
            "Resolving fork using PoE fork choice rule"
        );

        // Get metadata for all competing blocks
        let mut candidates: Vec<(String, BlockMetadata)> = blocks
            .iter()
            .filter_map(|hash| {
                self.block_metadata
                    .get(hash)
                    .map(|meta| (hash.clone(), meta.value().clone()))
            })
            .collect();

        // Apply fork choice rule
        candidates.sort_by(|a, b| {
            // 1. Highest emotional score wins
            match b.1.emotional_score.cmp(&a.1.emotional_score) {
                std::cmp::Ordering::Equal => {
                    // 2. Highest consensus strength wins
                    match b.1.consensus_strength.cmp(&a.1.consensus_strength) {
                        std::cmp::Ordering::Equal => {
                            // 3. Earliest timestamp wins (avoid timestamp manipulation)
                            a.1.timestamp.cmp(&b.1.timestamp)
                        }
                        other => other,
                    }
                }
                other => other,
            }
        });

        let winning_hash = candidates[0].0.clone();
        let winner_meta = &candidates[0].1;

        info!(
            block_height = height,
            winning_hash = %&winning_hash[..12],
            emotional_score = winner_meta.emotional_score,
            consensus_strength = winner_meta.consensus_strength,
            timestamp = winner_meta.timestamp,
            competing_blocks = candidates.len(),
            resolution_method = "emotional_score_priority",
            "Fork resolved successfully"
        );

        // Update fork info with resolution
        let mut forks = self.forks.write().await;
        if let Some(fork) = forks.iter_mut().find(|f| f.height == height && f.winning_hash.is_none()) {
            fork.resolution_method = Some("Emotional Score Rule".to_string());
            fork.winning_hash = Some(winning_hash.clone());
        }

        Ok(winning_hash)
    }

    /// Get all detected forks
    pub async fn get_forks(&self) -> Vec<ForkInfo> {
        self.forks.read().await.clone()
    }

    /// Get the canonical chain
    pub async fn get_canonical_chain(&self) -> Vec<String> {
        self.canonical_chain.read().await.clone()
    }

    /// Check if a fork exists at the given height
    pub fn has_fork(&self, height: u64) -> bool {
        self.blocks_at_height
            .get(&height)
            .map(|blocks| blocks.len() > 1)
            .unwrap_or(false)
    }

    /// Clear old fork data (for memory management)
    ///
    /// Removes fork data for heights more than `keep_height` blocks old
    pub async fn cleanup_old_forks(&self, current_height: u64, keep_height: u64) {
        if current_height <= keep_height {
            return;
        }

        let cutoff = current_height - keep_height;

        // Remove old blocks
        self.blocks_at_height.retain(|height, _| *height > cutoff);

        // Remove old metadata
        self.block_metadata.retain(|_, meta| meta.height > cutoff);

        // Remove old fork records (keep for historical analysis)
        // We keep forks for debugging, but could optionally clean them
        let old_fork_count = self.forks.read().await.len();
        let mut forks = self.forks.write().await;
        forks.retain(|fork| fork.height > cutoff);
        let new_fork_count = forks.len();

        if old_fork_count > new_fork_count {
            info!(
                forks_removed = old_fork_count - new_fork_count,
                cutoff_height = cutoff,
                forks_retained = new_fork_count,
                "Old fork records cleaned up"
            );
        }
    }

    /// Get fork statistics
    pub async fn get_fork_statistics(&self) -> ForkStatistics {
        let forks = self.forks.read().await;
        let total_forks = forks.len();
        let resolved_forks = forks.iter().filter(|f| f.winning_hash.is_some()).count();
        let unresolved_forks = total_forks - resolved_forks;

        ForkStatistics {
            total_forks,
            resolved_forks,
            unresolved_forks,
            heights_with_forks: self.blocks_at_height.len(),
        }
    }
}

impl Default for ForkDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about fork detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkStatistics {
    /// Total forks detected
    pub total_forks: usize,
    /// Forks that have been resolved
    pub resolved_forks: usize,
    /// Forks awaiting resolution
    pub unresolved_forks: usize,
    /// Number of heights where blocks were recorded
    pub heights_with_forks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BlockHeader;

    fn create_test_block(height: u64, hash: &str, emotional_score: u8, validator: &str) -> Block {
        Block {
            header: BlockHeader {
                height,
                epoch: 0,
                previous_hash: "0".repeat(64),
                merkle_root: "merkle".to_string(),
                timestamp: 1000000,
                difficulty: 0,
                nonce: 0,
                validator_id: validator.to_string(),
                emotional_score,
                consensus_strength: 80,
            },
            hash: hash.to_string(),
            transactions: vec![],
            signature: String::new(),
            proposer_public_key: String::new(),
            emotional_proof: None,
            consensus_metadata: None,
        }
    }

    #[tokio::test]
    async fn test_no_fork_single_block() {
        let detector = ForkDetector::new();
        let block = create_test_block(1, "hash1", 85, "validator1");

        let result = detector.record_block(&block).await;
        assert!(result.is_ok());
        assert!(!detector.has_fork(1));
    }

    #[tokio::test]
    async fn test_fork_detection() {
        let detector = ForkDetector::new();

        // First block at height 1
        let block1 = create_test_block(1, "hash1", 85, "validator1");
        detector.record_block(&block1).await.unwrap();

        // Second block at height 1 (fork!)
        let block2 = create_test_block(1, "hash2", 90, "validator2");
        let result = detector.record_block(&block2).await;

        assert!(result.is_err());
        assert!(detector.has_fork(1));
    }

    #[tokio::test]
    async fn test_fork_resolution_by_emotional_score() {
        let detector = ForkDetector::new();

        // Create fork with two blocks
        let block1 = create_test_block(1, "hash1", 85, "validator1");
        let block2 = create_test_block(1, "hash2", 90, "validator2");

        detector.record_block(&block1).await.unwrap();
        let _ = detector.record_block(&block2).await; // Fork!

        // Resolve fork
        let winner = detector.resolve_fork(1).await.unwrap();

        // Block with higher emotional score should win
        assert_eq!(winner, "hash2");
    }

    #[tokio::test]
    async fn test_fork_statistics() {
        let detector = ForkDetector::new();

        let block1 = create_test_block(1, "hash1", 85, "validator1");
        let block2 = create_test_block(1, "hash2", 90, "validator2");

        detector.record_block(&block1).await.unwrap();
        let _ = detector.record_block(&block2).await; // Fork

        let stats = detector.get_fork_statistics().await;
        assert_eq!(stats.total_forks, 1);
        assert_eq!(stats.unresolved_forks, 1);

        // Resolve the fork
        detector.resolve_fork(1).await.unwrap();

        let stats = detector.get_fork_statistics().await;
        assert_eq!(stats.resolved_forks, 1);
        assert_eq!(stats.unresolved_forks, 0);
    }

    #[tokio::test]
    async fn test_cleanup_old_forks() {
        let detector = ForkDetector::new();

        // Create blocks at heights 1, 2, 3
        for height in 1..=3 {
            let block = create_test_block(height, &format!("hash{}", height), 85, "validator1");
            detector.record_block(&block).await.unwrap();
        }

        // Cleanup blocks older than height 2
        detector.cleanup_old_forks(10, 8).await;

        // Heights 1, 2 should be removed
        assert!(!detector.blocks_at_height.contains_key(&1));
        assert!(!detector.blocks_at_height.contains_key(&2));
        assert!(detector.blocks_at_height.contains_key(&3));
    }
}
