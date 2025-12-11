//! Core types for Proof of Emotion consensus

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    /// Block height
    pub height: u64,
    /// Hash of previous block
    pub previous_hash: String,
    /// Merkle root of transactions
    pub merkle_root: String,
    /// Block timestamp (Unix milliseconds)
    pub timestamp: u64,
    /// Block difficulty (unused in POE)
    pub difficulty: u32,
    /// Nonce (unused in POE)
    pub nonce: u64,
    /// Validator ID who proposed this block
    pub validator_id: String,
    /// Emotional score of proposing validator
    pub emotional_score: u8,
    /// Consensus strength (percentage)
    pub consensus_strength: u8,
}

/// Transaction structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Transaction hash
    pub hash: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Amount in POE tokens (smallest unit)
    pub amount: u64,
    /// Transaction fee in POE tokens
    pub fee: u64,
    /// Transaction timestamp
    pub timestamp: u64,
    /// Transaction signature
    pub signature: String,
    /// Sender's public key
    pub public_key: String,
    /// Optional transaction data
    pub data: Vec<u8>,
}

/// Block structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// Block hash
    pub hash: String,
    /// List of transactions
    pub transactions: Vec<Transaction>,
    /// Block signature by proposer
    pub signature: String,
    /// Proposer's public key
    pub proposer_public_key: String,
    /// Emotional proof for this block
    pub emotional_proof: Option<Vec<u8>>,
    /// Consensus metadata
    pub consensus_metadata: Option<ConsensusMetadata>,
}

/// Consensus metadata attached to finalized blocks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsensusMetadata {
    /// Number of validators who participated
    pub participant_count: usize,
    /// Consensus strength achieved (percentage)
    pub consensus_strength: u8,
    /// Average emotional fitness of participants
    pub emotional_fitness: u8,
    /// Number of Byzantine failures detected
    pub byzantine_failures: usize,
    /// Timestamp when block was finalized
    pub finalized_at: u64,
    /// List of validator IDs who participated
    pub participants: Vec<String>,
}

/// Vote cast by a validator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vote {
    /// Validator ID who cast this vote
    pub validator_id: String,
    /// Block hash being voted on
    pub block_hash: String,
    /// Emotional score of validator at vote time
    pub emotional_score: u8,
    /// Vote signature
    pub signature: String,
    /// Vote timestamp
    pub timestamp: u64,
    /// Whether the vote approves the block
    pub approved: bool,
    /// Optional reason for rejection
    pub reason: Option<String>,
}

/// Result of a voting round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResult {
    /// Whether consensus was reached
    pub success: bool,
    /// Consensus strength (percentage)
    pub consensus_strength: u8,
    /// Number of participants
    pub participant_count: usize,
    /// Number of Byzantine failures
    pub byzantine_count: usize,
    /// Average emotional score
    pub average_emotional_score: u8,
    /// List of validator IDs who participated
    pub participants: Vec<String>,
    /// All votes cast
    pub votes: Vec<Vote>,
    /// Optional reason for failure
    pub reason: Option<String>,
}

impl Block {
    /// Create a new block
    pub fn new(
        height: u64,
        previous_hash: String,
        validator_id: String,
        emotional_score: u8,
        transactions: Vec<Transaction>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let merkle_root = Self::calculate_merkle_root(&transactions);

        let header = BlockHeader {
            height,
            previous_hash,
            merkle_root,
            timestamp,
            difficulty: 0,
            nonce: 0,
            validator_id: validator_id.clone(),
            emotional_score,
            consensus_strength: 0,
        };

        let hash = Self::calculate_block_hash(&header, &transactions);

        Self {
            header,
            hash,
            transactions,
            signature: String::new(),
            proposer_public_key: String::new(),
            emotional_proof: None,
            consensus_metadata: None,
        }
    }

    /// Calculate block hash
    pub fn calculate_block_hash(header: &BlockHeader, transactions: &[Transaction]) -> String {
        let mut hasher = Sha256::new();

        hasher.update(header.height.to_le_bytes());
        hasher.update(header.previous_hash.as_bytes());
        hasher.update(header.merkle_root.as_bytes());
        hasher.update(header.timestamp.to_le_bytes());
        hasher.update(header.validator_id.as_bytes());
        hasher.update([header.emotional_score]);

        for tx in transactions {
            hasher.update(tx.hash.as_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Calculate Merkle root of transactions
    pub fn calculate_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return hex::encode(Sha256::digest(b"empty"));
        }

        let mut hashes: Vec<Vec<u8>> = transactions
            .iter()
            .map(|tx| hex::decode(&tx.hash).unwrap_or_default())
            .collect();

        while hashes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]);
                }
                next_level.push(hasher.finalize().to_vec());
            }

            hashes = next_level;
        }

        hex::encode(&hashes[0])
    }

    /// Verify block hash
    pub fn verify_hash(&self) -> bool {
        let calculated_hash = Self::calculate_block_hash(&self.header, &self.transactions);
        calculated_hash == self.hash
    }

    /// Get block size in bytes
    pub fn size(&self) -> usize {
        bincode::serialize(self).map(|b| b.len()).unwrap_or(0)
    }

    /// Sign the block with a key pair
    pub fn sign(&mut self, key_pair: &crate::crypto::KeyPair) -> Result<(), String> {
        // Serialize the data to be signed (header + transactions)
        let mut data_to_sign = Vec::new();

        // Include all header fields
        data_to_sign.extend_from_slice(&self.header.height.to_le_bytes());
        data_to_sign.extend_from_slice(self.header.previous_hash.as_bytes());
        data_to_sign.extend_from_slice(self.header.merkle_root.as_bytes());
        data_to_sign.extend_from_slice(&self.header.timestamp.to_le_bytes());
        data_to_sign.extend_from_slice(self.header.validator_id.as_bytes());
        data_to_sign.push(self.header.emotional_score);

        // Include block hash
        data_to_sign.extend_from_slice(self.hash.as_bytes());

        // Include all transaction hashes
        for tx in &self.transactions {
            data_to_sign.extend_from_slice(tx.hash.as_bytes());
        }

        // Sign the data
        let sig = key_pair
            .sign(&data_to_sign)
            .map_err(|e| format!("Failed to sign block: {}", e))?;

        // Serialize signature to JSON string
        self.signature = serde_json::to_string(&sig)
            .map_err(|e| format!("Failed to serialize signature: {}", e))?;
        self.proposer_public_key = key_pair.public_key_hex();

        Ok(())
    }

    /// Verify the block signature
    pub fn verify_signature(&self) -> Result<bool, String> {
        if self.signature.is_empty() {
            return Err("Block has no signature".to_string());
        }

        if self.proposer_public_key.is_empty() {
            return Err("Block has no public key".to_string());
        }

        // Deserialize signature from JSON
        let sig: crate::crypto::Signature = serde_json::from_str(&self.signature)
            .map_err(|e| format!("Failed to deserialize signature: {}", e))?;

        // Reconstruct the signed data
        let mut data_to_verify = Vec::new();

        data_to_verify.extend_from_slice(&self.header.height.to_le_bytes());
        data_to_verify.extend_from_slice(self.header.previous_hash.as_bytes());
        data_to_verify.extend_from_slice(self.header.merkle_root.as_bytes());
        data_to_verify.extend_from_slice(&self.header.timestamp.to_le_bytes());
        data_to_verify.extend_from_slice(self.header.validator_id.as_bytes());
        data_to_verify.push(self.header.emotional_score);
        data_to_verify.extend_from_slice(self.hash.as_bytes());

        for tx in &self.transactions {
            data_to_verify.extend_from_slice(tx.hash.as_bytes());
        }

        // Verify signature
        crate::crypto::KeyPair::verify(&data_to_verify, &sig, &self.proposer_public_key)
            .map_err(|e| format!("Signature verification failed: {}", e))
    }
}

impl Transaction {
    /// Create a new transaction
    pub fn new(from: String, to: String, amount: u64, fee: u64) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let hash = Self::calculate_tx_hash(&from, &to, amount, fee, timestamp);

        Self {
            hash,
            from,
            to,
            amount,
            fee,
            timestamp,
            signature: String::new(),
            public_key: String::new(),
            data: Vec::new(),
        }
    }

    /// Calculate transaction hash
    pub fn calculate_tx_hash(
        from: &str,
        to: &str,
        amount: u64,
        fee: u64,
        timestamp: u64,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(from.as_bytes());
        hasher.update(to.as_bytes());
        hasher.update(amount.to_le_bytes());
        hasher.update(fee.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    /// Verify transaction hash
    pub fn verify_hash(&self) -> bool {
        let calculated_hash =
            Self::calculate_tx_hash(&self.from, &self.to, self.amount, self.fee, self.timestamp);
        calculated_hash == self.hash
    }

    /// Sign the transaction with a key pair
    pub fn sign(&mut self, key_pair: &crate::crypto::KeyPair) -> Result<(), String> {
        // Serialize the transaction data to be signed
        let mut data_to_sign = Vec::new();

        data_to_sign.extend_from_slice(self.hash.as_bytes());
        data_to_sign.extend_from_slice(self.from.as_bytes());
        data_to_sign.extend_from_slice(self.to.as_bytes());
        data_to_sign.extend_from_slice(&self.amount.to_le_bytes());
        data_to_sign.extend_from_slice(&self.fee.to_le_bytes());
        data_to_sign.extend_from_slice(&self.timestamp.to_le_bytes());
        data_to_sign.extend_from_slice(&self.data);

        // Sign the data
        let sig = key_pair
            .sign(&data_to_sign)
            .map_err(|e| format!("Failed to sign transaction: {}", e))?;

        // Serialize signature to JSON string
        self.signature = serde_json::to_string(&sig)
            .map_err(|e| format!("Failed to serialize signature: {}", e))?;
        self.public_key = key_pair.public_key_hex();

        Ok(())
    }

    /// Verify the transaction signature
    pub fn verify_signature(&self) -> Result<bool, String> {
        if self.signature.is_empty() {
            return Err("Transaction has no signature".to_string());
        }

        if self.public_key.is_empty() {
            return Err("Transaction has no public key".to_string());
        }

        // Deserialize signature from JSON
        let sig: crate::crypto::Signature = serde_json::from_str(&self.signature)
            .map_err(|e| format!("Failed to deserialize signature: {}", e))?;

        // Reconstruct the signed data
        let mut data_to_verify = Vec::new();

        data_to_verify.extend_from_slice(self.hash.as_bytes());
        data_to_verify.extend_from_slice(self.from.as_bytes());
        data_to_verify.extend_from_slice(self.to.as_bytes());
        data_to_verify.extend_from_slice(&self.amount.to_le_bytes());
        data_to_verify.extend_from_slice(&self.fee.to_le_bytes());
        data_to_verify.extend_from_slice(&self.timestamp.to_le_bytes());
        data_to_verify.extend_from_slice(&self.data);

        // Verify signature
        crate::crypto::KeyPair::verify(&data_to_verify, &sig, &self.public_key)
            .map_err(|e| format!("Transaction signature verification failed: {}", e))
    }
}

impl Vote {
    /// Create a new vote
    pub fn new(
        validator_id: String,
        block_hash: String,
        emotional_score: u8,
        approved: bool,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            validator_id,
            block_hash,
            emotional_score,
            signature: String::new(),
            timestamp,
            approved,
            reason: None,
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block {{ height: {}, hash: {}..., txs: {}, validator: {} }}",
            self.header.height,
            &self.hash[..8],
            self.transactions.len(),
            self.header.validator_id
        )
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tx {{ hash: {}..., from: {}..., to: {}..., amount: {} POE }}",
            &self.hash[..8],
            &self.from[..8],
            &self.to[..8],
            self.amount
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new("addr1".to_string(), "addr2".to_string(), 1000, 10);

        assert!(tx.verify_hash());
        assert_eq!(tx.amount, 1000);
        assert_eq!(tx.fee, 10);
    }

    #[test]
    fn test_block_creation() {
        let txs = vec![
            Transaction::new("addr1".to_string(), "addr2".to_string(), 1000, 10),
            Transaction::new("addr3".to_string(), "addr4".to_string(), 2000, 20),
        ];

        let block = Block::new(1, "0".repeat(64), "validator1".to_string(), 85, txs);

        assert!(block.verify_hash());
        assert_eq!(block.header.height, 1);
        assert_eq!(block.transactions.len(), 2);
    }

    #[test]
    fn test_merkle_root() {
        let txs = vec![Transaction::new(
            "addr1".to_string(),
            "addr2".to_string(),
            1000,
            10,
        )];

        let root1 = Block::calculate_merkle_root(&txs);
        let root2 = Block::calculate_merkle_root(&txs);

        assert_eq!(root1, root2);
        assert!(!root1.is_empty());
    }

    #[test]
    fn test_vote_creation() {
        let vote = Vote::new(
            "validator1".to_string(),
            "blockhash123".to_string(),
            85,
            true,
        );

        assert!(vote.approved);
        assert_eq!(vote.emotional_score, 85);
    }
}
