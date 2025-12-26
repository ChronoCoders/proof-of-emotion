//! Health check system for consensus monitoring

use crate::consensus::ProofOfEmotionEngine;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Overall health status of the consensus engine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatus {
    /// Current health state
    pub status: HealthState,
    /// Software version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Current epoch number
    pub current_epoch: u64,
    /// Consensus strength (0-100)
    pub consensus_strength: u8,
    /// Total number of validators
    pub validator_count: usize,
    /// Number of active validators
    pub active_validators: usize,
    /// Last finalized block height
    pub last_finalized_block: u64,
    /// Number of pending transactions
    pub pending_transactions: usize,
    /// Network participation rate (0-100)
    pub participation_rate: u8,
    /// List of current health issues
    pub issues: Vec<HealthIssue>,
    /// Timestamp when health check was performed
    pub checked_at: u64,
}

/// Health state categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthState {
    /// All systems operating normally
    Healthy,
    /// Some issues detected but system is functional
    Degraded,
    /// Critical issues detected, system may not be functional
    Critical,
}

/// Specific health issues that can be detected
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthIssue {
    /// Consensus strength below Byzantine threshold
    LowConsensus(u8),
    /// Participation rate too low
    LowParticipation(u8),
    /// No blocks finalized recently
    StaleChain(u64),
    /// Not enough active validators
    InsufficientValidators(usize),
    /// Too many pending transactions
    TransactionBacklog(usize),
    /// Network not responding
    NetworkUnresponsive,
    /// High rate of Byzantine failures
    HighByzantineRate(f64),
}

impl HealthStatus {
    /// Create a health status from the consensus engine state
    pub async fn from_consensus(
        engine: &ProofOfEmotionEngine,
        start_time: u64,
    ) -> HealthStatus {
        let state = engine.get_state().await;
        let metrics = engine.get_metrics().await;

        let mut issues = Vec::new();

        // Check consensus strength (should be >= 67% for Byzantine fault tolerance)
        if state.consensus_strength < 67 {
            issues.push(HealthIssue::LowConsensus(state.consensus_strength));
        }

        // Check participation rate (should be >= 50%)
        if state.participation_rate < 50 {
            issues.push(HealthIssue::LowParticipation(state.participation_rate));
        }

        // Check if chain is stale (no blocks finalized in recent epochs)
        if state.last_finalized_height == 0 && state.current_epoch > 5 {
            issues.push(HealthIssue::StaleChain(state.current_epoch));
        }

        // Check validator count (need at least 4 for Byzantine tolerance)
        if state.total_validators < 4 {
            issues.push(HealthIssue::InsufficientValidators(state.total_validators));
        }

        // Check for transaction backlog
        if state.pending_transactions > 1000 {
            issues.push(HealthIssue::TransactionBacklog(
                state.pending_transactions,
            ));
        }

        // Check Byzantine failure rate
        if metrics.total_epochs > 0 {
            let byzantine_rate =
                metrics.byzantine_failures as f64 / metrics.total_epochs as f64;
            if byzantine_rate > 0.1 {
                // More than 10% Byzantine failures
                issues.push(HealthIssue::HighByzantineRate(byzantine_rate));
            }
        }

        // Determine overall health state
        let health_state = if issues.is_empty() {
            HealthState::Healthy
        } else if issues.len() <= 2
            && !issues
                .iter()
                .any(|i| matches!(i, HealthIssue::NetworkUnresponsive))
        {
            HealthState::Degraded
        } else {
            HealthState::Critical
        };

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        HealthStatus {
            status: health_state,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: current_time.saturating_sub(start_time),
            current_epoch: state.current_epoch,
            consensus_strength: state.consensus_strength,
            validator_count: state.total_validators,
            active_validators: state.active_validators,
            last_finalized_block: state.last_finalized_height,
            pending_transactions: state.pending_transactions,
            participation_rate: state.participation_rate,
            issues,
            checked_at: current_time,
        }
    }

    /// Check if the system is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, HealthState::Healthy)
    }

    /// Check if the system is degraded
    pub fn is_degraded(&self) -> bool {
        matches!(self.status, HealthState::Degraded)
    }

    /// Check if the system is in critical state
    pub fn is_critical(&self) -> bool {
        matches!(self.status, HealthState::Critical)
    }

    /// Get a human-readable status message
    pub fn status_message(&self) -> String {
        match self.status {
            HealthState::Healthy => "All systems operational".to_string(),
            HealthState::Degraded => format!(
                "System degraded with {} issue(s): {}",
                self.issues.len(),
                self.issues_summary()
            ),
            HealthState::Critical => format!(
                "System critical with {} issue(s): {}",
                self.issues.len(),
                self.issues_summary()
            ),
        }
    }

    /// Get a summary of all issues
    fn issues_summary(&self) -> String {
        self.issues
            .iter()
            .map(|issue| match issue {
                HealthIssue::LowConsensus(strength) => {
                    format!("Low consensus ({}%)", strength)
                }
                HealthIssue::LowParticipation(rate) => {
                    format!("Low participation ({}%)", rate)
                }
                HealthIssue::StaleChain(epoch) => {
                    format!("Stale chain (epoch {})", epoch)
                }
                HealthIssue::InsufficientValidators(count) => {
                    format!("Insufficient validators ({})", count)
                }
                HealthIssue::TransactionBacklog(count) => {
                    format!("Transaction backlog ({})", count)
                }
                HealthIssue::NetworkUnresponsive => "Network unresponsive".to_string(),
                HealthIssue::HighByzantineRate(rate) => {
                    format!("High Byzantine rate ({:.1}%)", rate * 100.0)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Simple health check result for liveness probes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessCheck {
    pub alive: bool,
    pub timestamp: u64,
}

impl LivenessCheck {
    pub fn new() -> Self {
        Self {
            alive: true,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
        }
    }
}

impl Default for LivenessCheck {
    fn default() -> Self {
        Self::new()
    }
}

/// Readiness check result for readiness probes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCheck {
    pub ready: bool,
    pub reason: Option<String>,
    pub timestamp: u64,
}

impl ReadinessCheck {
    /// Create a readiness check from health status
    pub fn from_health(health: &HealthStatus) -> Self {
        let ready = !health.is_critical();
        let reason = if ready {
            None
        } else {
            Some(health.status_message())
        };

        Self {
            ready,
            reason,
            timestamp: health.checked_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_state_healthy() {
        let health = HealthStatus {
            status: HealthState::Healthy,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            current_epoch: 100,
            consensus_strength: 95,
            validator_count: 10,
            active_validators: 9,
            last_finalized_block: 100,
            pending_transactions: 5,
            participation_rate: 90,
            issues: vec![],
            checked_at: 1234567890,
        };

        assert!(health.is_healthy());
        assert!(!health.is_degraded());
        assert!(!health.is_critical());
        assert_eq!(health.status_message(), "All systems operational");
    }

    #[test]
    fn test_health_state_degraded() {
        let health = HealthStatus {
            status: HealthState::Degraded,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            current_epoch: 100,
            consensus_strength: 60,
            validator_count: 10,
            active_validators: 9,
            last_finalized_block: 100,
            pending_transactions: 5,
            participation_rate: 45,
            issues: vec![
                HealthIssue::LowConsensus(60),
                HealthIssue::LowParticipation(45),
            ],
            checked_at: 1234567890,
        };

        assert!(!health.is_healthy());
        assert!(health.is_degraded());
        assert!(!health.is_critical());
        assert!(health.status_message().contains("degraded"));
        assert!(health.status_message().contains("Low consensus"));
    }

    #[test]
    fn test_health_state_critical() {
        let health = HealthStatus {
            status: HealthState::Critical,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            current_epoch: 100,
            consensus_strength: 30,
            validator_count: 2,
            active_validators: 2,
            last_finalized_block: 50,
            pending_transactions: 2000,
            participation_rate: 20,
            issues: vec![
                HealthIssue::LowConsensus(30),
                HealthIssue::LowParticipation(20),
                HealthIssue::InsufficientValidators(2),
                HealthIssue::TransactionBacklog(2000),
            ],
            checked_at: 1234567890,
        };

        assert!(!health.is_healthy());
        assert!(!health.is_degraded());
        assert!(health.is_critical());
        assert!(health.status_message().contains("critical"));
    }

    #[test]
    fn test_liveness_check() {
        let liveness = LivenessCheck::new();
        assert!(liveness.alive);
        assert!(liveness.timestamp > 0);
    }

    #[test]
    fn test_readiness_check_from_healthy() {
        let health = HealthStatus {
            status: HealthState::Healthy,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            current_epoch: 100,
            consensus_strength: 95,
            validator_count: 10,
            active_validators: 9,
            last_finalized_block: 100,
            pending_transactions: 5,
            participation_rate: 90,
            issues: vec![],
            checked_at: 1234567890,
        };

        let readiness = ReadinessCheck::from_health(&health);
        assert!(readiness.ready);
        assert!(readiness.reason.is_none());
    }

    #[test]
    fn test_readiness_check_from_critical() {
        let health = HealthStatus {
            status: HealthState::Critical,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            current_epoch: 100,
            consensus_strength: 30,
            validator_count: 2,
            active_validators: 2,
            last_finalized_block: 50,
            pending_transactions: 2000,
            participation_rate: 20,
            issues: vec![HealthIssue::LowConsensus(30)],
            checked_at: 1234567890,
        };

        let readiness = ReadinessCheck::from_health(&health);
        assert!(!readiness.ready);
        assert!(readiness.reason.is_some());
    }

    #[test]
    fn test_health_issue_messages() {
        let issues = vec![
            HealthIssue::LowConsensus(50),
            HealthIssue::LowParticipation(40),
            HealthIssue::StaleChain(10),
            HealthIssue::InsufficientValidators(3),
            HealthIssue::TransactionBacklog(1500),
            HealthIssue::HighByzantineRate(0.15),
        ];

        let health = HealthStatus {
            status: HealthState::Critical,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            current_epoch: 100,
            consensus_strength: 50,
            validator_count: 3,
            active_validators: 3,
            last_finalized_block: 50,
            pending_transactions: 1500,
            participation_rate: 40,
            issues,
            checked_at: 1234567890,
        };

        let summary = health.issues_summary();
        assert!(summary.contains("Low consensus"));
        assert!(summary.contains("Low participation"));
        assert!(summary.contains("Stale chain"));
        assert!(summary.contains("Insufficient validators"));
        assert!(summary.contains("Transaction backlog"));
        assert!(summary.contains("High Byzantine rate"));
    }
}
