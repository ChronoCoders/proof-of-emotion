//! Cryptographic primitives for Proof of Emotion

use crate::error::{ConsensusError, Result};
use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    Message, PublicKey, Secp256k1, SecretKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// ECDSA key pair for validator identity
#[derive(Clone)]
pub struct KeyPair {
    secret_key: SecretKey,
    public_key: PublicKey,
}

/// Cryptographic signature
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Signature {
    /// Signature bytes (hex encoded)
    pub signature: String,
    /// Recovery ID
    pub recovery_id: u8,
    /// Signature algorithm
    pub algorithm: String,
}

/// Emotional proof containing cryptographic evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalProof {
    /// List of validator IDs
    pub validators: Vec<String>,
    /// Emotional scores by validator
    pub emotional_scores: std::collections::HashMap<String, u8>,
    /// Biometric data hashes (for privacy)
    pub biometric_hashes: std::collections::HashMap<String, String>,
    /// Temporal window (milliseconds)
    pub temporal_window: u64,
    /// Proof timestamp
    pub timestamp: u64,
    /// Consensus strength
    pub consensus_strength: u8,
    /// Merkle root of proof data
    pub merkle_root: String,
    /// Proof signature
    pub signature: Signature,
}

impl KeyPair {
    /// Generate a new random key pair
    pub fn generate() -> Result<Self> {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());

        Ok(Self {
            secret_key,
            public_key,
        })
    }

    /// Create key pair from secret key bytes
    pub fn from_secret_bytes(bytes: &[u8]) -> Result<Self> {
        let secret_key = SecretKey::from_slice(bytes)
            .map_err(|e| ConsensusError::internal(format!("Invalid secret key: {}", e)))?;

        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(Self {
            secret_key,
            public_key,
        })
    }

    /// Get public key as hex string
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.serialize())
    }

    /// Get secret key as hex string (⚠️ sensitive!)
    pub fn secret_key_hex(&self) -> String {
        hex::encode(self.secret_key.secret_bytes())
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        let secp = Secp256k1::new();
        let message_hash = Sha256::digest(message);
        let message = Message::from_digest_slice(&message_hash)
            .map_err(|e| ConsensusError::internal(format!("Invalid message: {}", e)))?;

        let signature = secp.sign_ecdsa_recoverable(&message, &self.secret_key);
        let (recovery_id, signature_bytes) = signature.serialize_compact();

        Ok(Signature {
            signature: hex::encode(signature_bytes),
            recovery_id: recovery_id.to_i32() as u8,
            algorithm: "ECDSA-secp256k1".to_string(),
        })
    }

    /// Verify a signature
    pub fn verify(message: &[u8], signature: &Signature, public_key_hex: &str) -> Result<bool> {
        let secp = Secp256k1::new();

        let public_key_bytes = hex::decode(public_key_hex)
            .map_err(|e| ConsensusError::internal(format!("Invalid public key hex: {}", e)))?;
        let public_key = PublicKey::from_slice(&public_key_bytes)
            .map_err(|e| ConsensusError::internal(format!("Invalid public key: {}", e)))?;

        let signature_bytes = hex::decode(&signature.signature)
            .map_err(|e| ConsensusError::internal(format!("Invalid signature hex: {}", e)))?;
        let recovery_id = RecoveryId::from_i32(signature.recovery_id as i32)
            .map_err(|e| ConsensusError::internal(format!("Invalid recovery ID: {}", e)))?;
        let recoverable_sig = RecoverableSignature::from_compact(&signature_bytes, recovery_id)
            .map_err(|e| ConsensusError::internal(format!("Invalid signature: {}", e)))?;

        let message_hash = Sha256::digest(message);
        let message = Message::from_digest_slice(&message_hash)
            .map_err(|e| ConsensusError::internal(format!("Invalid message: {}", e)))?;

        let recovered_key = secp
            .recover_ecdsa(&message, &recoverable_sig)
            .map_err(|e| {
                ConsensusError::signature_verification_failed(format!("Recovery failed: {}", e))
            })?;

        Ok(recovered_key == public_key)
    }

    /// Get the public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }
}

impl EmotionalProof {
    /// Create a new emotional proof
    pub fn new(
        validators: Vec<String>,
        emotional_scores: std::collections::HashMap<String, u8>,
        biometric_hashes: std::collections::HashMap<String, String>,
        temporal_window: u64,
        key_pair: &KeyPair,
    ) -> Result<Self> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let consensus_strength = Self::calculate_consensus_strength(&emotional_scores);

        let merkle_root = Self::calculate_merkle_root(
            &validators,
            &emotional_scores,
            &biometric_hashes,
            temporal_window,
            timestamp,
        );

        let proof_data = format!(
            "{}:{}:{}:{}:{}",
            validators.join(","),
            serde_json::to_string(&emotional_scores).unwrap(),
            serde_json::to_string(&biometric_hashes).unwrap(),
            temporal_window,
            timestamp
        );

        let signature = key_pair.sign(proof_data.as_bytes())?;

        Ok(Self {
            validators,
            emotional_scores,
            biometric_hashes,
            temporal_window,
            timestamp,
            consensus_strength,
            merkle_root,
            signature,
        })
    }

    /// Calculate consensus strength from emotional scores
    fn calculate_consensus_strength(scores: &std::collections::HashMap<String, u8>) -> u8 {
        if scores.is_empty() {
            return 0;
        }

        let sum: u32 = scores.values().map(|&s| s as u32).sum();
        let avg = sum / scores.len() as u32;

        let variance: f64 = scores
            .values()
            .map(|&s| {
                let diff = s as f64 - avg as f64;
                diff * diff
            })
            .sum::<f64>()
            / scores.len() as f64;

        let variance_penalty = (variance.sqrt() / 5.0).min(20.0);

        (avg as f64 - variance_penalty).clamp(0.0, 100.0) as u8
    }

    /// Calculate merkle root of proof data
    fn calculate_merkle_root(
        validators: &[String],
        emotional_scores: &std::collections::HashMap<String, u8>,
        biometric_hashes: &std::collections::HashMap<String, String>,
        temporal_window: u64,
        timestamp: u64,
    ) -> String {
        let mut hasher = Sha256::new();

        for validator in validators {
            hasher.update(validator.as_bytes());
        }
        hasher.update(serde_json::to_string(emotional_scores).unwrap().as_bytes());
        hasher.update(serde_json::to_string(biometric_hashes).unwrap().as_bytes());
        hasher.update(temporal_window.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());

        hex::encode(hasher.finalize())
    }

    /// Verify the emotional proof
    pub fn verify(&self, public_key_hex: &str) -> Result<bool> {
        let proof_data = format!(
            "{}:{}:{}:{}:{}",
            self.validators.join(","),
            serde_json::to_string(&self.emotional_scores).unwrap(),
            serde_json::to_string(&self.biometric_hashes).unwrap(),
            self.temporal_window,
            self.timestamp
        );

        let signature_valid =
            KeyPair::verify(proof_data.as_bytes(), &self.signature, public_key_hex)?;

        if !signature_valid {
            return Ok(false);
        }

        let expected_merkle_root = Self::calculate_merkle_root(
            &self.validators,
            &self.emotional_scores,
            &self.biometric_hashes,
            self.temporal_window,
            self.timestamp,
        );

        if expected_merkle_root != self.merkle_root {
            return Ok(false);
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if now - self.timestamp > 300_000 {
            return Ok(false);
        }

        let expected_strength = Self::calculate_consensus_strength(&self.emotional_scores);
        if (expected_strength as i16 - self.consensus_strength as i16).abs() > 1 {
            return Ok(false);
        }

        Ok(true)
    }
}

/// Hash biometric data for privacy
pub fn hash_biometric_data(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let keypair = KeyPair::generate().unwrap();
        assert!(!keypair.public_key_hex().is_empty());
        assert!(!keypair.secret_key_hex().is_empty());
    }

    #[test]
    fn test_signing_and_verification() {
        let keypair = KeyPair::generate().unwrap();
        let message = b"test message";

        let signature = keypair.sign(message).unwrap();
        let valid = KeyPair::verify(message, &signature, &keypair.public_key_hex()).unwrap();

        assert!(valid);
    }

    #[test]
    fn test_invalid_signature() {
        let keypair1 = KeyPair::generate().unwrap();
        let keypair2 = KeyPair::generate().unwrap();
        let message = b"test message";

        let signature = keypair1.sign(message).unwrap();
        let valid = KeyPair::verify(message, &signature, &keypair2.public_key_hex()).unwrap();

        assert!(!valid);
    }

    #[test]
    fn test_emotional_proof_creation() {
        let keypair = KeyPair::generate().unwrap();
        let mut scores = std::collections::HashMap::new();
        scores.insert("validator1".to_string(), 85);
        scores.insert("validator2".to_string(), 90);

        let mut hashes = std::collections::HashMap::new();
        hashes.insert("validator1".to_string(), "hash1".to_string());
        hashes.insert("validator2".to_string(), "hash2".to_string());

        let proof = EmotionalProof::new(
            vec!["validator1".to_string(), "validator2".to_string()],
            scores,
            hashes,
            30000,
            &keypair,
        )
        .unwrap();

        assert!(proof.verify(&keypair.public_key_hex()).unwrap());
    }

    #[test]
    fn test_consensus_strength_calculation() {
        let mut scores = std::collections::HashMap::new();
        scores.insert("v1".to_string(), 85);
        scores.insert("v2".to_string(), 87);
        scores.insert("v3".to_string(), 83);

        let strength = EmotionalProof::calculate_consensus_strength(&scores);
        assert!(strength > 80);
    }
}
