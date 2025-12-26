//! Prometheus metrics export for consensus monitoring

use crate::consensus::ConsensusMetrics;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, Opts, Registry,
};
use std::sync::Arc;

/// Prometheus metrics for the Proof of Emotion consensus engine
pub struct PrometheusMetrics {
    // Counters - monotonically increasing values
    pub blocks_finalized: Counter,
    pub transactions_processed: Counter,
    pub byzantine_detected: Counter,
    pub votes_cast: Counter,
    pub epochs_completed: Counter,
    pub epochs_failed: Counter,

    // Gauges - values that can go up and down
    pub active_validators: Gauge,
    pub current_epoch: Gauge,
    pub committee_size: Gauge,
    pub consensus_strength: Gauge,
    pub network_health: Gauge,
    pub participation_rate: Gauge,
    pub pending_transactions: Gauge,
    pub last_finalized_height: Gauge,

    // Histograms - distributions of values
    pub block_proposal_duration: Histogram,
    pub voting_duration: Histogram,
    pub epoch_duration: Histogram,
    pub emotional_scores: Histogram,
    pub consensus_strength_hist: Histogram,

    // Vector metrics - metrics with labels
    pub validator_stakes: GaugeVec,
    pub validator_reputations: GaugeVec,
    pub byzantine_events: CounterVec,
}

impl PrometheusMetrics {
    /// Create a new PrometheusMetrics instance and register all metrics
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        // Counters
        let blocks_finalized = Counter::with_opts(Opts::new(
            "poe_blocks_finalized_total",
            "Total number of finalized blocks",
        ))?;
        registry.register(Box::new(blocks_finalized.clone()))?;

        let transactions_processed = Counter::with_opts(Opts::new(
            "poe_transactions_processed_total",
            "Total number of processed transactions",
        ))?;
        registry.register(Box::new(transactions_processed.clone()))?;

        let byzantine_detected = Counter::with_opts(Opts::new(
            "poe_byzantine_detected_total",
            "Total number of Byzantine behaviors detected",
        ))?;
        registry.register(Box::new(byzantine_detected.clone()))?;

        let votes_cast = Counter::with_opts(Opts::new(
            "poe_votes_cast_total",
            "Total number of votes cast",
        ))?;
        registry.register(Box::new(votes_cast.clone()))?;

        let epochs_completed = Counter::with_opts(Opts::new(
            "poe_epochs_completed_total",
            "Total number of successfully completed epochs",
        ))?;
        registry.register(Box::new(epochs_completed.clone()))?;

        let epochs_failed = Counter::with_opts(Opts::new(
            "poe_epochs_failed_total",
            "Total number of failed epochs",
        ))?;
        registry.register(Box::new(epochs_failed.clone()))?;

        // Gauges
        let active_validators = Gauge::with_opts(Opts::new(
            "poe_active_validators",
            "Current number of active validators",
        ))?;
        registry.register(Box::new(active_validators.clone()))?;

        let current_epoch = Gauge::with_opts(Opts::new(
            "poe_current_epoch",
            "Current epoch number",
        ))?;
        registry.register(Box::new(current_epoch.clone()))?;

        let committee_size = Gauge::with_opts(Opts::new(
            "poe_committee_size",
            "Current committee size",
        ))?;
        registry.register(Box::new(committee_size.clone()))?;

        let consensus_strength = Gauge::with_opts(Opts::new(
            "poe_consensus_strength",
            "Current consensus strength (0-100)",
        ))?;
        registry.register(Box::new(consensus_strength.clone()))?;

        let network_health = Gauge::with_opts(Opts::new(
            "poe_network_health",
            "Network health percentage (0-100)",
        ))?;
        registry.register(Box::new(network_health.clone()))?;

        let participation_rate = Gauge::with_opts(Opts::new(
            "poe_participation_rate",
            "Validator participation rate (0-100)",
        ))?;
        registry.register(Box::new(participation_rate.clone()))?;

        let pending_transactions = Gauge::with_opts(Opts::new(
            "poe_pending_transactions",
            "Number of pending transactions",
        ))?;
        registry.register(Box::new(pending_transactions.clone()))?;

        let last_finalized_height = Gauge::with_opts(Opts::new(
            "poe_last_finalized_height",
            "Height of the last finalized block",
        ))?;
        registry.register(Box::new(last_finalized_height.clone()))?;

        // Histograms
        let block_proposal_duration = Histogram::with_opts(
            HistogramOpts::new(
                "poe_block_proposal_duration_seconds",
                "Time taken to propose a block",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        )?;
        registry.register(Box::new(block_proposal_duration.clone()))?;

        let voting_duration = Histogram::with_opts(
            HistogramOpts::new("poe_voting_duration_seconds", "Time taken for voting phase")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        )?;
        registry.register(Box::new(voting_duration.clone()))?;

        let epoch_duration = Histogram::with_opts(
            HistogramOpts::new("poe_epoch_duration_seconds", "Duration of each epoch")
                .buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0]),
        )?;
        registry.register(Box::new(epoch_duration.clone()))?;

        let emotional_scores = Histogram::with_opts(
            HistogramOpts::new("poe_emotional_scores", "Distribution of emotional scores")
                .buckets(vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0]),
        )?;
        registry.register(Box::new(emotional_scores.clone()))?;

        let consensus_strength_hist = Histogram::with_opts(
            HistogramOpts::new(
                "poe_consensus_strength_distribution",
                "Distribution of consensus strength values",
            )
            .buckets(vec![0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0]),
        )?;
        registry.register(Box::new(consensus_strength_hist.clone()))?;

        // Vector metrics
        let validator_stakes = GaugeVec::new(
            Opts::new("poe_validator_stake", "Stake amount per validator"),
            &["validator_id"],
        )?;
        registry.register(Box::new(validator_stakes.clone()))?;

        let validator_reputations = GaugeVec::new(
            Opts::new("poe_validator_reputation", "Reputation score per validator"),
            &["validator_id"],
        )?;
        registry.register(Box::new(validator_reputations.clone()))?;

        let byzantine_events = CounterVec::new(
            Opts::new("poe_byzantine_events_total", "Byzantine events by type"),
            &["event_type", "validator_id"],
        )?;
        registry.register(Box::new(byzantine_events.clone()))?;

        Ok(Self {
            blocks_finalized,
            transactions_processed,
            byzantine_detected,
            votes_cast,
            epochs_completed,
            epochs_failed,
            active_validators,
            current_epoch,
            committee_size,
            consensus_strength,
            network_health,
            participation_rate,
            pending_transactions,
            last_finalized_height,
            block_proposal_duration,
            voting_duration,
            epoch_duration,
            emotional_scores,
            consensus_strength_hist,
            validator_stakes,
            validator_reputations,
            byzantine_events,
        })
    }

    /// Update metrics from consensus state
    pub fn update_from_consensus(&self, metrics: &ConsensusMetrics) {
        self.blocks_finalized
            .inc_by(metrics.blocks_finalized as f64);
        self.transactions_processed
            .inc_by(metrics.transactions_processed as f64);
        self.epochs_completed
            .inc_by(metrics.successful_epochs as f64);
        self.epochs_failed.inc_by(metrics.failed_epochs as f64);

        self.active_validators
            .set(metrics.active_validators as f64);

        // Update histograms
        if metrics.average_duration_ms > 0 {
            self.epoch_duration
                .observe(metrics.average_duration_ms as f64 / 1000.0);
        }

        if metrics.average_emotional_score > 0 {
            self.emotional_scores
                .observe(metrics.average_emotional_score as f64);
        }

        if metrics.average_proposal_time_ms > 0 {
            self.block_proposal_duration
                .observe(metrics.average_proposal_time_ms as f64 / 1000.0);
        }

        if metrics.average_voting_time_ms > 0 {
            self.voting_duration
                .observe(metrics.average_voting_time_ms as f64 / 1000.0);
        }
    }

    /// Record a Byzantine event
    pub fn record_byzantine_event(&self, event_type: &str, validator_id: &str) {
        self.byzantine_detected.inc();
        self.byzantine_events
            .with_label_values(&[event_type, validator_id])
            .inc();
    }

    /// Update validator stake
    pub fn update_validator_stake(&self, validator_id: &str, stake: u64) {
        self.validator_stakes
            .with_label_values(&[validator_id])
            .set(stake as f64);
    }

    /// Update validator reputation
    pub fn update_validator_reputation(&self, validator_id: &str, reputation: u8) {
        self.validator_reputations
            .with_label_values(&[validator_id])
            .set(reputation as f64);
    }

    /// Record block proposal time
    pub fn observe_block_proposal(&self, duration_secs: f64) {
        self.block_proposal_duration.observe(duration_secs);
    }

    /// Record voting time
    pub fn observe_voting(&self, duration_secs: f64) {
        self.voting_duration.observe(duration_secs);
    }

    /// Increment vote count
    pub fn inc_votes(&self, count: u64) {
        self.votes_cast.inc_by(count as f64);
    }

    /// Update network health
    pub fn set_network_health(&self, health: f64) {
        self.network_health.set(health);
    }

    /// Update committee size
    pub fn set_committee_size(&self, size: usize) {
        self.committee_size.set(size as f64);
    }

    /// Update pending transactions
    pub fn set_pending_transactions(&self, count: usize) {
        self.pending_transactions.set(count as f64);
    }
}

/// Create a default Prometheus registry with all PoE metrics
pub fn create_default_registry() -> Result<(Registry, Arc<PrometheusMetrics>), prometheus::Error> {
    let registry = Registry::new();
    let metrics = Arc::new(PrometheusMetrics::new(&registry)?);
    Ok((registry, metrics))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let registry = Registry::new();
        let metrics = PrometheusMetrics::new(&registry);
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_counter_increment() {
        let registry = Registry::new();
        let metrics = PrometheusMetrics::new(&registry).unwrap();

        metrics.blocks_finalized.inc();
        assert_eq!(metrics.blocks_finalized.get(), 1.0);

        metrics.blocks_finalized.inc_by(5.0);
        assert_eq!(metrics.blocks_finalized.get(), 6.0);
    }

    #[test]
    fn test_gauge_set() {
        let registry = Registry::new();
        let metrics = PrometheusMetrics::new(&registry).unwrap();

        metrics.active_validators.set(10.0);
        assert_eq!(metrics.active_validators.get(), 10.0);

        metrics.active_validators.set(20.0);
        assert_eq!(metrics.active_validators.get(), 20.0);
    }

    #[test]
    fn test_histogram_observe() {
        let registry = Registry::new();
        let metrics = PrometheusMetrics::new(&registry).unwrap();

        metrics.observe_block_proposal(0.5);
        metrics.observe_block_proposal(1.0);
        metrics.observe_block_proposal(0.25);

        // Histogram should have 3 samples
        let metric_families = registry.gather();
        let block_proposal_metric = metric_families
            .iter()
            .find(|mf| mf.get_name() == "poe_block_proposal_duration_seconds");

        assert!(block_proposal_metric.is_some());
    }

    #[test]
    fn test_byzantine_event_recording() {
        let registry = Registry::new();
        let metrics = PrometheusMetrics::new(&registry).unwrap();

        metrics.record_byzantine_event("double_signing", "validator1");
        metrics.record_byzantine_event("double_voting", "validator2");

        assert_eq!(metrics.byzantine_detected.get(), 2.0);
    }

    #[test]
    fn test_validator_metrics() {
        let registry = Registry::new();
        let metrics = PrometheusMetrics::new(&registry).unwrap();

        metrics.update_validator_stake("alice", 10000);
        metrics.update_validator_reputation("alice", 95);

        let metric_families = registry.gather();
        assert!(!metric_families.is_empty());
    }

    #[test]
    fn test_create_default_registry() {
        let result = create_default_registry();
        assert!(result.is_ok());

        let (registry, metrics) = result.unwrap();
        let metric_families = registry.gather();
        assert!(!metric_families.is_empty());

        // Verify we can use the metrics
        metrics.blocks_finalized.inc();
        assert_eq!(metrics.blocks_finalized.get(), 1.0);
    }
}
