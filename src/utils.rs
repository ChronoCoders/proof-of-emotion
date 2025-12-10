//! Utility functions for Proof of Emotion consensus

/// Calculate stake weight with square root to reduce whale dominance
pub fn calculate_stake_weight(stake: u64) -> f64 {
    (stake as f64).sqrt()
}

/// Calculate emotional bonus multiplier
pub fn calculate_emotional_multiplier(emotional_score: u8, threshold: u8) -> f64 {
    if emotional_score < threshold {
        let penalty = (threshold - emotional_score) as f64 / 100.0;
        1.0 - (penalty * 0.5).min(0.5)
    } else {
        let bonus = (emotional_score - threshold) as f64 / 100.0;
        1.0 + (bonus * 0.3).min(0.3)
    }
}

/// Calculate variance of a sequence
pub fn calculate_variance(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let squared_diffs: f64 = values.iter().map(|v| (v - mean).powi(2)).sum();
    
    squared_diffs / values.len() as f64
}

/// Calculate Pearson correlation coefficient
pub fn calculate_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let n = x.len() as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let sum_xx: f64 = x.iter().map(|xi| xi * xi).sum();
    let sum_yy: f64 = y.iter().map(|yi| yi * yi).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = ((n * sum_xx - sum_x * sum_x) * (n * sum_yy - sum_y * sum_y)).sqrt();

    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

/// Calculate simple moving average
pub fn calculate_sma(values: &[f64], period: usize) -> Vec<f64> {
    if values.len() < period || period == 0 {
        return vec![];
    }

    let mut sma = Vec::new();
    for i in period - 1..values.len() {
        let start = i.saturating_sub(period - 1);
        let sum: f64 = values[start..=i].iter().sum();
        sma.push(sum / period as f64);
    }
    sma
}

/// Detect anomalies using standard deviation
pub fn detect_anomalies(values: &[f64], std_threshold: f64) -> Vec<usize> {
    if values.is_empty() {
        return vec![];
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = calculate_variance(values);
    let std_dev = variance.sqrt();

    values
        .iter()
        .enumerate()
        .filter_map(|(i, &v)| {
            if (v - mean).abs() > std_threshold * std_dev {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Format POE amount with decimals
pub fn format_poe_amount(amount: u64) -> String {
    let whole = amount / 1_000_000;
    let decimal = amount % 1_000_000;
    format!("{}.{:06} POE", whole, decimal)
}

/// Calculate percentage
pub fn calculate_percentage(part: usize, total: usize) -> u8 {
    if total == 0 {
        return 0;
    }
    ((part as f64 / total as f64) * 100.0) as u8
}

/// Clamp value between min and max
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Generate a deterministic seed from a string
pub fn string_to_seed(s: &str) -> u64 {
    s.bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stake_weight() {
        assert_eq!(calculate_stake_weight(10000), 100.0);
        assert_eq!(calculate_stake_weight(40000), 200.0);
    }

    #[test]
    fn test_emotional_multiplier() {
        assert!(calculate_emotional_multiplier(50, 75) < 1.0);
        assert_eq!(calculate_emotional_multiplier(75, 75), 1.0);
        assert!(calculate_emotional_multiplier(90, 75) > 1.0);
    }

    #[test]
    fn test_variance() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let variance = calculate_variance(&values);
        assert!(variance > 0.0);
    }

    #[test]
    fn test_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let corr = calculate_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_sma() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = calculate_sma(&values, 3);
        assert_eq!(sma.len(), 3);
        assert_eq!(sma[0], 2.0);
    }

    #[test]
    fn test_anomaly_detection() {
        let values = vec![1.0, 2.0, 3.0, 100.0, 4.0, 5.0];
        let anomalies = detect_anomalies(&values, 2.0);
        assert!(!anomalies.is_empty());
        assert!(anomalies.contains(&3));
    }

    #[test]
    fn test_format_poe() {
        assert_eq!(format_poe_amount(1_500_000), "1.500000 POE");
        assert_eq!(format_poe_amount(10_000_000), "10.000000 POE");
    }

    #[test]
    fn test_percentage() {
        assert_eq!(calculate_percentage(50, 100), 50);
        assert_eq!(calculate_percentage(1, 3), 33);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5, 0, 10), 5);
        assert_eq!(clamp(-5, 0, 10), 0);
        assert_eq!(clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_string_to_seed() {
        let seed1 = string_to_seed("validator-1");
        let seed2 = string_to_seed("validator-1");
        let seed3 = string_to_seed("validator-2");
        
        assert_eq!(seed1, seed2);
        assert_ne!(seed1, seed3);
    }
}
