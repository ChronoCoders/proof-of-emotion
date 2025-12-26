//! Byzantine fault detection and slashing
//!
//! This module implements Byzantine behavior detection including:
//! - Double voting: voting differently on the same block/epoch
//! - Double signing: proposing multiple blocks at the same height
//! - Equivocation: making conflicting statements

use crate::staking::{SlashingEvent, SlashingOffense, SlashingSeverity};
use crate::types::Vote;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Evidence of a block proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalEvidence {
    /// Validator who proposed the block
    pub validator_id: String,
    /// Block height
    pub height: u64,
    /// Block hash
    pub block_hash: String,
    /// Timestamp of proposal
    pub timestamp: u64,
}

/// Byzantine fault detector
pub struct ByzantineDetector {
    /// Track votes by validator per (validator_id, epoch, block_hash)
    /// Maps (validator_id, epoch) -> list of votes
    votes: Arc<DashMap<(String, u64), Vec<Vote>>>,

    /// Track proposed blocks by validator per height
    /// Maps (validator_id, height) -> list of block hashes
    proposals: Arc<DashMap<(String, u64), Vec<ProposalEvidence>>>,

    /// Slashing events detected
    slashing_events: Arc<RwLock<Vec<SlashingEvent>>>,
}

impl ByzantineDetector {
    /// Create a new Byzantine detector
    pub fn new() -> Self {
        Self {
            votes: Arc::new(DashMap::new()),
            proposals: Arc::new(DashMap::new()),
            slashing_events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a vote for Byzantine detection
    ///
    /// This stores the vote and checks for double voting
    pub async fn record_vote(&self, vote: &Vote) -> Result<(), String> {
        let key = (vote.validator_id.clone(), vote.epoch);

        // Get or create vote list for this validator/epoch
        let mut votes = self.votes.entry(key.clone()).or_default();

        // Check for double voting before adding
        if let Some(event) = self.detect_double_voting_internal(&votes, vote) {
            warn!(
                validator_id = %vote.validator_id,
                epoch = vote.epoch,
                round = vote.round,
                block_hash = %vote.block_hash,
                event_type = "double_voting",
                "Byzantine behavior: double voting detected"
            );

            let mut events = self.slashing_events.write().await;
            events.push(event.clone());
            drop(events);

            return Err(format!(
                "Double voting detected for validator {} in epoch {}",
                vote.validator_id, vote.epoch
            ));
        }

        // Add the vote
        votes.push(vote.clone());

        Ok(())
    }

    /// Record a block proposal for Byzantine detection
    ///
    /// This stores the proposal and checks for double signing
    pub async fn record_proposal(
        &self,
        validator_id: &str,
        height: u64,
        block_hash: &str,
    ) -> Result<(), String> {
        let key = (validator_id.to_string(), height);

        let evidence = ProposalEvidence {
            validator_id: validator_id.to_string(),
            height,
            block_hash: block_hash.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("System time error: {}", e))?
                .as_millis() as u64,
        };

        // Get or create proposal list for this validator/height
        let mut proposals = self.proposals.entry(key.clone()).or_default();

        // Check for double signing
        if let Some(event) = self.detect_double_signing_internal(&proposals, &evidence) {
            warn!(
                validator_id = %validator_id,
                block_height = height,
                block_hash = %block_hash,
                event_type = "double_signing",
                "Byzantine behavior: double signing detected"
            );

            let mut events = self.slashing_events.write().await;
            events.push(event.clone());
            drop(events);

            return Err(format!(
                "Double signing detected for validator {} at height {}",
                validator_id, height
            ));
        }

        // Add the proposal
        proposals.push(evidence);

        Ok(())
    }

    /// Detect double voting for a specific validator and epoch
    ///
    /// Double voting occurs when a validator votes differently on blocks in the same epoch
    pub fn detect_double_voting(&self, validator_id: &str, epoch: u64) -> Option<SlashingEvent> {
        let key = (validator_id.to_string(), epoch);

        if let Some(votes) = self.votes.get(&key) {
            // Check if there are conflicting votes
            let mut seen_votes: std::collections::HashMap<String, bool> =
                std::collections::HashMap::new();

            for vote in votes.iter() {
                let block_hash = &vote.block_hash;

                if let Some(&previous_approval) = seen_votes.get(block_hash) {
                    // Check if voting differently on same block
                    if previous_approval != vote.approved {
                        return Some(self.create_double_voting_event(
                            validator_id,
                            epoch,
                            votes.clone(),
                        ));
                    }
                } else {
                    seen_votes.insert(block_hash.clone(), vote.approved);
                }
            }

            // Check for votes on different blocks in same epoch (equivocation)
            if seen_votes.len() > 1 {
                return Some(self.create_equivocation_event(validator_id, epoch, votes.clone()));
            }
        }

        None
    }

    /// Internal detection during vote recording
    fn detect_double_voting_internal(
        &self,
        existing_votes: &[Vote],
        new_vote: &Vote,
    ) -> Option<SlashingEvent> {
        for existing_vote in existing_votes {
            // Same block hash but different approval - double voting
            if existing_vote.block_hash == new_vote.block_hash
                && existing_vote.approved != new_vote.approved
            {
                let mut votes = existing_votes.to_vec();
                votes.push(new_vote.clone());
                return Some(self.create_double_voting_event(
                    &new_vote.validator_id,
                    new_vote.epoch,
                    votes,
                ));
            }

            // Different block hash in same epoch - equivocation
            if existing_vote.block_hash != new_vote.block_hash {
                let mut votes = existing_votes.to_vec();
                votes.push(new_vote.clone());
                return Some(self.create_equivocation_event(
                    &new_vote.validator_id,
                    new_vote.epoch,
                    votes,
                ));
            }
        }
        None
    }

    /// Detect double signing for a specific validator and height
    ///
    /// Double signing occurs when a validator proposes multiple different blocks at the same height
    pub fn detect_double_signing(&self, validator_id: &str, height: u64) -> Option<SlashingEvent> {
        let key = (validator_id.to_string(), height);

        if let Some(proposals) = self.proposals.get(&key) {
            if proposals.len() > 1 {
                // Check if there are actually different blocks
                let unique_hashes: std::collections::HashSet<_> =
                    proposals.iter().map(|p| &p.block_hash).collect();

                if unique_hashes.len() > 1 {
                    return Some(self.create_double_signing_event(
                        validator_id,
                        height,
                        proposals.clone(),
                    ));
                }
            }
        }

        None
    }

    /// Internal detection during proposal recording
    fn detect_double_signing_internal(
        &self,
        existing_proposals: &[ProposalEvidence],
        new_proposal: &ProposalEvidence,
    ) -> Option<SlashingEvent> {
        for existing in existing_proposals {
            if existing.block_hash != new_proposal.block_hash {
                let mut proposals = existing_proposals.to_vec();
                proposals.push(new_proposal.clone());
                return Some(self.create_double_signing_event(
                    &new_proposal.validator_id,
                    new_proposal.height,
                    proposals,
                ));
            }
        }
        None
    }

    /// Detect equivocation (conflicting statements)
    ///
    /// Equivocation occurs when a validator votes on multiple different blocks in the same epoch
    pub fn detect_equivocation(&self, validator_id: &str) -> Option<SlashingEvent> {
        // Check all epochs for this validator
        for entry in self.votes.iter() {
            let (vid, epoch) = entry.key();
            if vid == validator_id {
                let votes = entry.value();

                // Get unique block hashes voted on
                let unique_blocks: std::collections::HashSet<_> =
                    votes.iter().map(|v| &v.block_hash).collect();

                if unique_blocks.len() > 1 {
                    return Some(self.create_equivocation_event(
                        validator_id,
                        *epoch,
                        votes.clone(),
                    ));
                }
            }
        }

        None
    }

    /// Create a double voting slashing event
    fn create_double_voting_event(
        &self,
        validator_id: &str,
        epoch: u64,
        votes: Vec<Vote>,
    ) -> SlashingEvent {
        let evidence = format!(
            "Double voting in epoch {}: {} conflicting votes on same block",
            epoch,
            votes.len()
        );

        SlashingEvent {
            id: format!("double-vote-{}-{}", validator_id, epoch),
            validator_id: validator_id.to_string(),
            offense: SlashingOffense::DoubleSigning, // Reuse DoubleSigning for now
            severity: SlashingSeverity::Critical,
            slashing_rate: 15.0, // Critical offense: 15% slash
            amount: 0,           // Will be calculated based on stake
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time before UNIX_EPOCH - clock may be misconfigured")
                .as_millis() as u64,
            evidence,
        }
    }

    /// Create a double signing slashing event
    fn create_double_signing_event(
        &self,
        validator_id: &str,
        height: u64,
        proposals: Vec<ProposalEvidence>,
    ) -> SlashingEvent {
        let block_hashes: Vec<_> = proposals.iter().map(|p| &p.block_hash).collect();
        let evidence = format!(
            "Double signing at height {}: proposed {} different blocks: {:?}",
            height,
            proposals.len(),
            block_hashes
        );

        SlashingEvent {
            id: format!("double-sign-{}-{}", validator_id, height),
            validator_id: validator_id.to_string(),
            offense: SlashingOffense::DoubleSigning,
            severity: SlashingSeverity::Critical,
            slashing_rate: 15.0, // Critical offense: 15% slash
            amount: 0,           // Will be calculated based on stake
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time before UNIX_EPOCH - clock may be misconfigured")
                .as_millis() as u64,
            evidence,
        }
    }

    /// Create an equivocation slashing event
    fn create_equivocation_event(
        &self,
        validator_id: &str,
        epoch: u64,
        votes: Vec<Vote>,
    ) -> SlashingEvent {
        let block_hashes: std::collections::HashSet<_> =
            votes.iter().map(|v| &v.block_hash).collect();

        let evidence = format!(
            "Equivocation in epoch {}: voted on {} different blocks",
            epoch,
            block_hashes.len()
        );

        SlashingEvent {
            id: format!("equivocation-{}-{}", validator_id, epoch),
            validator_id: validator_id.to_string(),
            offense: SlashingOffense::DoubleSigning, // Reuse DoubleSigning for now
            severity: SlashingSeverity::Major,
            slashing_rate: 5.0, // Major offense: 5% slash
            amount: 0,          // Will be calculated based on stake
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time before UNIX_EPOCH - clock may be misconfigured")
                .as_millis() as u64,
            evidence,
        }
    }

    /// Get all detected slashing events
    pub async fn get_slashing_events(&self) -> Vec<SlashingEvent> {
        self.slashing_events.read().await.clone()
    }

    /// Clear old detection data (for memory management)
    ///
    /// Removes data older than the specified number of epochs
    pub fn cleanup_old_data(&self, current_epoch: u64, retention_epochs: u64) {
        let cutoff_epoch = current_epoch.saturating_sub(retention_epochs);

        // Clean up old votes
        self.votes.retain(|(_, epoch), _| *epoch >= cutoff_epoch);

        info!(
            current_epoch = current_epoch,
            cutoff_epoch = cutoff_epoch,
            retention_epochs = retention_epochs,
            votes_retained = self.votes.len(),
            proposals_retained = self.proposals.len(),
            "Byzantine detector cleanup completed"
        );
    }
}

impl Default for ByzantineDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detector_creation() {
        let detector = ByzantineDetector::new();
        let events = detector.get_slashing_events().await;
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_double_voting_detection() {
        let detector = ByzantineDetector::new();

        // Create first vote (approve)
        let vote1 = Vote::new(
            "validator-1".to_string(),
            "block-hash-1".to_string(),
            1, // epoch
            0, // round
            80,
            true, // approved
        );

        detector.record_vote(&vote1).await.unwrap();

        // Create conflicting vote (reject same block)
        let vote2 = Vote::new(
            "validator-1".to_string(),
            "block-hash-1".to_string(),
            1, // same epoch
            0, // same round
            80,
            false, // rejected - conflicting!
        );

        // This should fail and create a slashing event
        let result = detector.record_vote(&vote2).await;
        assert!(result.is_err());

        let events = detector.get_slashing_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].offense, SlashingOffense::DoubleSigning);
    }

    #[tokio::test]
    async fn test_double_signing_detection() {
        let detector = ByzantineDetector::new();

        // Propose first block at height 1
        detector
            .record_proposal("validator-1", 1, "block-hash-1")
            .await
            .unwrap();

        // Propose different block at same height - Byzantine!
        let result = detector
            .record_proposal("validator-1", 1, "block-hash-2")
            .await;

        assert!(result.is_err());

        let events = detector.get_slashing_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].offense, SlashingOffense::DoubleSigning);
    }

    #[tokio::test]
    async fn test_equivocation_detection() {
        let detector = ByzantineDetector::new();

        // Vote on first block
        let vote1 = Vote::new(
            "validator-1".to_string(),
            "block-hash-1".to_string(),
            1, // epoch
            0,
            80,
            true,
        );
        detector.record_vote(&vote1).await.unwrap();

        // Vote on different block in same epoch - equivocation!
        let vote2 = Vote::new(
            "validator-1".to_string(),
            "block-hash-2".to_string(),
            1, // same epoch
            0,
            80,
            true,
        );

        let result = detector.record_vote(&vote2).await;
        // This will be detected as equivocation
        assert!(result.is_err());
    }
}
